use crate::{iovec, mr::DataDescriptor, MappedAddress};

pub struct Msg {
    pub(crate) c_msg: libfabric_sys::fi_msg,
}

impl Msg {

    pub fn new<T>(iov: &[iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress) -> Self {
        Msg {
            c_msg : libfabric_sys::fi_msg {
                msg_iov: iov.as_ptr() as *const libfabric_sys::iovec,
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.raw_addr(),
                context: std::ptr::null_mut(), // [TODO]
                data: 0,
            }
        }
    }
}


pub struct MsgTagged {
    pub(crate) c_msg_tagged: libfabric_sys::fi_msg_tagged,
}

impl MsgTagged {
    pub fn new<T>(iov: &[iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress, data: u64, tag: u64, ignore: u64) -> Self {
    
        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iov.as_ptr() as *const libfabric_sys::iovec,
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.raw_addr(),
                context: std::ptr::null_mut(), // [TODO]
                data,
                tag,
                ignore,
            }
        }
    }
}

pub struct MsgAtomic {
    pub(crate) c_msg_atomic: *mut libfabric_sys::fi_msg_atomic,
}

pub struct MsgRma {
    pub(crate) c_msg_rma: libfabric_sys::fi_msg_rma,
}

impl MsgRma {
    pub fn new<T, T0>(iov: &[iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &MappedAddress, rma_iov: &[iovec::RmaIoVec], context: &mut T0, data: u64) -> Self {
        Self {
            c_msg_rma : libfabric_sys::fi_msg_rma {
                msg_iov: iov.as_ptr() as *const libfabric_sys::iovec,
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.raw_addr(),
                rma_iov: rma_iov.as_ptr() as *const libfabric_sys::fi_rma_iov,
                rma_iov_count: rma_iov.len(),
                context: context as *mut T0 as *mut std::ffi::c_void,
                data,
            }
        }
    }
}
