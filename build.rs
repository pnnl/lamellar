extern crate bindgen;

fn main(){
    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let src_path = out_path.join("libfabric");
    let build_path = src_path.join("build");
    let custom_path = match std::env::var("LIBFABRIC_ROOT") {
        Ok(val) => val,
        Err(_) => "".to_string(),
    };

    let libfabrics_path;
    if custom_path == ""{

        libfabrics_path = build_path.to_str().unwrap().to_owned();
        
        // If ./libfabric does not exist we need to fetch it first
        if !std::path::Path::new(&(out_path.to_str().unwrap().to_owned()+ "/libfabric")).exists(){
    
            std::process::Command::new("git").arg("clone").arg("https://github.com/ofiwg/libfabric.git").arg(src_path.to_str().unwrap()).status().unwrap();
            std::process::Command::new("git").current_dir(src_path.to_str().unwrap()).arg("fetch").arg("--tags").status().unwrap();
            std::process::Command::new("git").current_dir(src_path.to_str().unwrap()).arg("checkout").arg("tags/v1.19.0").status().unwrap();
        }
    
    
        // If /libfabric/build does not exist we have not built libfabric yet
        if !std::path::Path::new(&(out_path.to_str().unwrap().to_owned()+ "/libfabric/build")).exists(){
    
            std::process::Command::new("sh").current_dir(src_path.to_str().unwrap()).arg("autogen.sh").status().unwrap();
            std::process::Command::new("sh").current_dir(src_path.to_str().unwrap()).arg("configure").arg("--prefix=".to_owned()+build_path.to_str().unwrap()).status().unwrap();
        }
        
        
        // Let make figure out if we need to build libfabric again. (Should we ?)    
        std::process::Command::new("make").current_dir(src_path.to_str().unwrap()).arg("install").status().unwrap();
    }
    else{

        libfabrics_path = custom_path;
    }
    
    
    println!("cargo::rustc-link-search={}", libfabrics_path);
    println!("cargo::rustc-link-lib=fabric");

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}",libfabrics_path.clone()+"/include/"))
        .header(libfabrics_path+"/include/rdma/fabric.h")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

}