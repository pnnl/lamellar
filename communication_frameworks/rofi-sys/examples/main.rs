extern crate rofisys;

fn main() {
    unsafe {
        let c_str = std::ffi::CString::new("verbs").unwrap();
        let retval = rofisys::rofi_init(c_str.as_ptr() as *mut _) as i32;
        println!("rofi_init = {}\n", retval);
        rofisys::rofi_finit();
    }
}
