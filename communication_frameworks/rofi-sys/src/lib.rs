#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ptr;
use std::os::raw;

use std::ffi::CString;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn it_may_work() {

        unsafe{
    let arg0 = std::ffi::CString::new("127.0.0.1")
        .expect("CString::new failed");

    let mut retval : std::os::raw::c_int = 0;
    retval = crate::rofi_init(
        arg0.into_raw(),
        0,
        1
    ); // -> ::std::os::raw::c_int;

    println!("rofi_init = {}\n",retval);
    crate::rofi_finit();
        }
    }
}

