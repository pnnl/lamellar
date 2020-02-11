extern crate rofisys;

fn main() {

        unsafe{
    let arg0 = std::ffi::CString::new("127.0.0.1")
        .expect("CString::new failed");

    let mut retval : std::os::raw::c_int = 0;
    retval = rofisys::rofi_init(
        arg0.into_raw(),
        0,
        1
    ); // -> ::std::os::raw::c_int;

    println!("rofi_init = {}\n",retval);
    rofisys::rofi_finit();
        
        }




}