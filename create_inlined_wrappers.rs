use std::io::BufRead;

fn read_lines<P>(filename: P) -> std::io::Result<std::io::Lines<std::io::BufReader<std::fs::File>>> where P: AsRef<std::path::Path>, {
    let file = std::fs::File::open(filename)?;
    Ok(std::io::BufReader::new(file).lines())
}

pub fn read_file(filepath: &str) -> (Vec<String>, Vec<String>){
    let mut inlined_funcs_proto = Vec::new();
    let mut inlined_funcs_impl = Vec::new();
    if let Ok(lines) = read_lines(filepath) {
        let mut keep = false;
        let mut curr_func = String::new();
        for oline in lines {
            match oline {
                Ok(line) => {
                    if let Some(stripped_line) = line.strip_prefix("static inline") {
                        keep = true;
                        if let Some(idx) = line.find('{') {
                            curr_func.push_str(&stripped_line[..idx]);
                            curr_func.push(';');
                        }
                        else {
                            curr_func.push_str(stripped_line);
                        }
                    }
                    else if keep {
                        if let Some(idx) = line.find('{') {
                            if ! line.starts_with('{') {
                                curr_func.push(' ');
                            }
                            curr_func.push_str(line[0..idx].trim());
                            curr_func.push(';');
                            let res = generate_wrapper_proto_and_body(&curr_func);
                            inlined_funcs_proto.push(res.0);
                            inlined_funcs_impl.push(res.1);
                            curr_func = String::new();
                            keep = false;
                        }
                        else {
                            curr_func.push(' ');
                            curr_func.push_str(line.trim());
                        }
                    }
                }
                Err(_) => todo!(),
            }
        }
    }

    (inlined_funcs_proto, inlined_funcs_impl)
}

pub fn generate_wrapper_proto_and_body(func_proto: &str) -> (String, String){
    let lpos = func_proto.find('(').unwrap() + 1;
    let rpos = func_proto.find(')').unwrap();
    let mut wrapper_proto = func_proto[..func_proto.len()-1].trim().to_string();
    let mut wrapper_impl = String::new();
    if let Some(name_pos) =  wrapper_proto[0..lpos].rfind("fi_") {
        wrapper_proto.insert_str(name_pos, "inlined_");
        wrapper_impl.push_str(&wrapper_proto);
        wrapper_proto.push(';');
        wrapper_impl.push_str("\n{\n");
        wrapper_impl.push_str("\treturn ");
        wrapper_impl.push_str(&func_proto[name_pos..lpos-1].replace('*', ""));
        let args = func_proto[lpos..rpos].split(',').map(|x| x.split(' ').last().unwrap().replace('*', "").replace("void", "")).collect::<Vec::<String>>().join(",");
        wrapper_impl.push_str(&format!("({});", args));
        wrapper_impl.push_str("\n}\n");
    }
    

    (wrapper_proto, wrapper_impl)
}

// fn main() {

//     let inlined_funcs = read_file("target/release/build/libfabrics-sys-58d2aac918303fbf/out/libfabric/build/include/rdma/fi_atomic.h");
//     println!("Found {} inlined functions\n", inlined_funcs.0.len());
//     for f in std::iter::zip(inlined_funcs.0, inlined_funcs.1) {
//         println!("{}\n{}", f.0, f.1);
//     }
// }