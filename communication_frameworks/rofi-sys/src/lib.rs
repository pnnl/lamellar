#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// use std::ptr;
// use std::os::raw;
// use std::ffi::CString;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn it_may_work() {
        unsafe {
            let c_str = std::ffi::CString::new("verbs").unwrap();
            let retval = crate::rofi_init(c_str.as_ptr() as *mut _) as i32;

            println!("rofi_init = {}\n", retval);
            crate::rofi_finit();
        }
    }
}
