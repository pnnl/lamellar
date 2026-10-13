extern crate rofisys;

fn main() {
    unsafe {
        let verbs_str = std::ffi::CString::new("verbs").unwrap();
        let domain_str = std::ffi::CString::new("").unwrap();
        let retval =
            rofisys::rofi_init(verbs_str.as_ptr() as *mut _, domain_str.as_ptr() as *mut _) as i32;
        println!("rofi_init = {}\n", retval);
        rofisys::rofi_finit();
    }
}
