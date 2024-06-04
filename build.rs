mod create_inlined_wrappers;
use std::{io::{Write, BufWriter}, fs::{ReadDir, File}};

extern crate bindgen;
fn iter_dir(dir: ReadDir, inlined_funcs: &mut Vec<String>, writer: &mut BufWriter<File>, writer_inlined: &mut BufWriter<File>) {
    for file in dir {
        if file.as_ref().unwrap().file_type().unwrap().is_file() {
            // Create the wrappers (prototype, definition) for all inlined functions
            let mut funcs = crate::create_inlined_wrappers::read_file(file.as_ref().unwrap().path().to_str().unwrap());

            // Store the prototype for later
            inlined_funcs.append(&mut funcs.0);
            
            // Write the definition to inlined.c
            for f in funcs.1 {
                let _ = writer_inlined.write_all((f+"\n").as_bytes());
            }
            
            // #include the header to fabric_sys.h
            let _ = writer.write_all(("#include<".to_owned()+file.as_ref().unwrap().path().to_str().unwrap()+">\n").as_bytes());
        }
        else if file.as_ref().unwrap().file_type().unwrap().is_dir() {
            iter_dir(std::fs::read_dir(file.unwrap().path()).unwrap() , inlined_funcs, writer, writer_inlined)
        }
    }
}

fn main(){
    
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let src_path = std::fs::canonicalize(std::path::PathBuf::from("libfabric")).unwrap();
    let build_path = out_path.join("build");
    let custom_path = match std::env::var("LIBFABRIC_ROOT") {
        Ok(val) => val,
        Err(_) => "".to_string(),
    };
    
    let libfabrics_path;
    if custom_path.is_empty(){
        
        libfabrics_path = build_path.to_str().unwrap().to_owned();
    
        // If /libfabric/build does not exist we have not built libfabric yet
        if !build_path.exists(){
            std::process::Command::new("sh").current_dir(src_path.to_str().unwrap()).arg("autogen.sh").status().unwrap();
            std::process::Command::new("sh").current_dir(src_path.to_str().unwrap()).arg("configure").arg("CFLAGS=-fPIC").arg("--prefix=".to_owned()+build_path.to_str().unwrap()).arg("--enable-static").status().unwrap();
            
            
            // Let make figure out if we need to build libfabric again. (Should we ?)    
            std::process::Command::new("make").current_dir(src_path.to_str().unwrap()).arg("-j").arg("install").status().unwrap();
        }
    }
    else{

        libfabrics_path = custom_path;
        // println!("cargo:rerun-if-changed={}", libfabrics_path.clone()+"/include/fabric_sys.h");
    }
    
    // libfabric has multiple header file that we are required to generate bindings for so we create a
    // new header named fabric_sys.h and #include all headers there
    let header_path = std::path::PathBuf::from(out_path.to_str().unwrap().to_owned()+"/fabric_sys.h");
    let lib_path = out_path.join("libinlined.so");
    if !header_path.exists() || !lib_path.exists(){

        let header_file = std::fs::File::create(out_path.to_str().unwrap().to_owned()+"/fabric_sys.h").unwrap();
        let mut writer = std::io::BufWriter::new(header_file);
        
        // Another problem is that there are several static inline functions in these header files which do
        // not create symbols in the libfabric.so and thus cannot be linked from rust.
        // For this reason, we create wrapper functions for each inline one and store them in a file called inlined.c
        // which resides in the /build/ directory 
        let inlined_file = std::fs::File::create(out_path.to_str().unwrap().to_owned()+"/inlined.c").unwrap();
        let mut writer_inlined = std::io::BufWriter::new(inlined_file);
        let _ = writer_inlined.write_all(b"#include<fabric_sys.h>\n");
        
        // We keep the prototypes of all the wrappers fo rinline functions here and will append them to fabric_sys.h
        // so that they are visible to bindgen
        let mut inlined_funcs: Vec<String> = Vec::new();
        
        let headers = std::fs::read_dir(libfabrics_path.clone()+"/include/rdma/").unwrap();

        // For each file in libfabric/include/
        iter_dir(headers, &mut inlined_funcs, &mut writer, &mut writer_inlined);
        
        // Append the prototypes of the wrappers for the inline functions to fabric_sys.h
        for f in inlined_funcs {
            let _ = writer.write_all((f+"\n").as_bytes());
        }
        
        // Make sure the files are written completely
        let _ = writer.flush();
        let _ = writer_inlined.flush();
        
        // Create a new lib, libinlined.so that we expose the wrappers for rust to use
        println!("{}", out_path.to_str().unwrap());
        println!("{}", libfabrics_path);

        std::process::Command::new("gcc").current_dir(out_path.to_str().unwrap()).arg("-Wno-everything").arg("-fPIC").arg("-O3").arg("-I".to_owned()+&libfabrics_path+"/include/").arg("-I".to_owned()+out_path.to_str().unwrap()).arg("-c").arg("-o").arg("inlined.o").arg(out_path.to_str().unwrap().to_owned()+"/inlined.c").status().unwrap();
        std::process::Command::new("ar").current_dir(out_path.to_str().unwrap()).arg("-rc").arg("libinlined.a").arg("inlined.o").status().unwrap();
    }
    // Generate the rust bindings
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}",libfabrics_path.clone()+"/include/"))
        .header(out_path.to_str().unwrap().to_owned()+"/fabric_sys.h")
        .generate()
        .expect("Unable to generate bindings");

    // Write them to the respective file
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    
    // Link with the libfabric and libinlined libraries to access their symbols. 
    println!("cargo:rustc-link-search={}", libfabrics_path.clone() +"/lib/");
    println!("cargo:rustc-link-search={}", out_path.to_str().unwrap().to_owned());
    //-linlined -lfabric -lrt -lrdmacm -libverbs -latomic -lpthread -ldl required by libfabric
    println!("cargo:rustc-link-lib=static=inlined");
    println!("cargo:rustc-link-lib=static=fabric");
    println!("cargo:rustc-link-lib=rt");
    println!("cargo:rustc-link-lib=rdmacm");
    println!("cargo:rustc-link-lib=ibverbs");
    println!("cargo:rustc-link-lib=atomic");
    println!("cargo:rustc-link-lib=pthread");
    println!("cargo:rustc-link-lib=dl");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", build_path.to_str().unwrap());
}