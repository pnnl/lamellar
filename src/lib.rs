#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::path::PathBuf;

// include!(concat!(env!("OUT_DIR"), "/bindings.rs"));



pub fn prterun_path() -> PathBuf {
    
    PathBuf::from(concat!(env!("OUT_DIR"), "/bin/prterun"))
}