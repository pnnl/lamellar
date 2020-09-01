extern crate rofisys;

fn main() {
    unsafe {

        let retval = rofisys::rofi_init(); // -> ::std::os::raw::c_int;
        println!("rofi_init = {}\n", retval);
        rofisys::rofi_finit();
    }
}
