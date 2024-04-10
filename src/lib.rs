use core::panic;
use std::{marker::PhantomData, rc::Rc};

use mr::DataDescriptor;
use utils::to_fi_datatype;

pub mod ep;
pub mod domain;
pub mod eq;
pub mod fabric;
pub mod enums;
pub mod av;
pub mod mr;
pub mod sync;
pub mod cntr;
pub mod cq;
pub mod comm;
pub mod error;
pub mod xcontext;
pub mod eqoptions;
pub mod cqoptions;
pub mod cntroptions;
pub mod infocapsoptions;
pub mod nic;
pub mod info;
mod utils;
mod fid;
pub mod iovec;
pub mod msg;

pub type Address = libfabric_sys::fi_addr_t; 
pub type DataType = libfabric_sys::fi_datatype;
pub type CollectiveOp = libfabric_sys::fi_collective_op;



// pub struct Stx {

//     #[allow(dead_code)]
//     c_stx: *mut libfabric_sys::fid_stx,
// }

// impl Stx {
//     pub(crate) fn new<T0>(domain: &crate::domain::Domain, mut attr: crate::TxAttr, context: &mut T0) -> Result<Stx, error::Error> {
//         let mut c_stx: *mut libfabric_sys::fid_stx = std::ptr::null_mut();
//         let c_stx_ptr: *mut *mut libfabric_sys::fid_stx = &mut c_stx;
//         let err = unsafe { libfabric_sys::inlined_fi_stx_context(domain.c_domain, attr.get_mut(), c_stx_ptr, context as *mut T0 as *mut std::ffi::c_void) };

//         if err != 0 {
//             Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
//         }
//         else {
//             Ok(
//                 Self { c_stx }
//             )
//         }

//     }
// }

// pub struct SrxAttr {
//     c_attr: libfabric_sys::fi_srx_attr,
// }

// impl SrxAttr {
//     pub(crate) fn get(&self) -> *const libfabric_sys::fi_srx_attr {
//         &self.c_attr
//     }

//     pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_srx_attr {
//         &mut self.c_attr
//     }
// }





pub fn rx_addr(addr: Address, rx_index: i32, rx_ctx_bits: i32) -> Address {
    unsafe { libfabric_sys::inlined_fi_rx_addr(addr, rx_index, rx_ctx_bits) }
}




// struct fi_param {
// 	const char *name;
// 	enum fi_param_type type;
// 	const char *help_string;
// 	const char *value;
// };

// int fi_getparams(struct fi_param **params, int *count);
// void fi_freeparams(struct fi_param *params);


// pub struct Param {
//     c_param : libfabric_sys::fi_param,
// }

// pub fn get_params() -> Vec<Param> {
//     let mut len = 0 as i32;
//     let len_ptr : *mut i32 = &mut len;
//     let mut c_params: *mut libfabric_sys::fi_param = std::ptr::null_mut();
//     let mut c_params_ptr: *mut *mut libfabric_sys::fi_param = &mut c_params;
    
//     let err = libfabric_sys::fi_getparams(c_params_ptr, len_ptr);
//     if err != 0 {
//         panic!("fi_getparams failed {}", err);
//     }

//     let mut params = Vec::<Param>::new();
//     for i  in 0..len {
//         params.push(Param { c_param: unsafe{ c_params.add(i as usize) } });
//     }

//     params
// }


// pub struct Param {
//     c_param: libfabric_sys::fi_param,
// }





pub struct Context {
    c_val: libfabric_sys::fi_context,
}

impl Context {
    pub fn new() -> Self {
        Self {
            c_val : {
                libfabric_sys::fi_context { internal: [std::ptr::null_mut(); 4] }
            }
        }
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_context {
        &mut self.c_val
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Context2 {
    c_val: libfabric_sys::fi_context2,
}

impl Context2 {
    pub fn new() -> Self {
        Self {
            c_val : {
                libfabric_sys::fi_context2 { internal: [std::ptr::null_mut(); 8] }
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_context2 {
        &mut self.c_val
    }
}

impl Default for Context2 {
    fn default() -> Self {
        Self::new()
    }
}

pub trait BindImpl{}
pub trait Bind {
    fn inner(&self) -> Rc<dyn BindImpl>;
}



pub trait FdRetrievable{}
pub trait Waitable{}
pub trait Writable{}
pub trait WaitRetrievable{}