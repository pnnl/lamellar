use std::marker::PhantomData;

use crate::{enums::Op, iovec, mr::DataDescriptor, utils::AsFiType, MappedAddress, FI_ADDR_UNSPEC};

pub struct Msg<'a> {
    pub(crate) c_msg: libfabric_sys::fi_msg,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Msg<'a> {

    pub fn new(iov: &'a [iovec::IoVec], desc: &'a mut [impl DataDescriptor], mapped_addr: &'a MappedAddress) -> Self {
        Msg {
            c_msg : libfabric_sys::fi_msg {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.raw_addr(),
                context: std::ptr::null_mut(), // [TODO]
                data: 0,
            },
            phantom: PhantomData,
        }
    }

    pub fn new_connected(iov: &'a [iovec::IoVec], desc: &'a mut [impl DataDescriptor]) -> Self {
        Msg {
            c_msg : libfabric_sys::fi_msg {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: FI_ADDR_UNSPEC,
                context: std::ptr::null_mut(), // [TODO]
                data: 0,
            },
            phantom: PhantomData,
        }
    }
}

pub struct MsgMut<'a> {
    pub(crate) c_msg: libfabric_sys::fi_msg,
    phantom: PhantomData<&'a ()>,
}

impl<'a> MsgMut<'a> {

    pub fn new(iov: &'a [iovec::IoVecMut], desc: &'a mut [impl DataDescriptor], mapped_addr: &'a MappedAddress) -> Self {
        MsgMut {
            c_msg : libfabric_sys::fi_msg {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.raw_addr(),
                context: std::ptr::null_mut(), // [TODO]
                data: 0,
            },
            phantom: PhantomData,
        }
    }

    pub fn new_connected(iov: &'a [iovec::IoVecMut], desc: &'a mut [impl DataDescriptor]) -> Self {
        MsgMut {
            c_msg : libfabric_sys::fi_msg {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: FI_ADDR_UNSPEC,
                context: std::ptr::null_mut(), // [TODO]
                data: 0,
            },
            phantom: PhantomData,
        }
    }
    
}


pub struct MsgTagged<'a> {
    pub(crate) c_msg_tagged: libfabric_sys::fi_msg_tagged,
    phantom: PhantomData<&'a ()>
}

impl<'a> MsgTagged<'a> {
    pub fn new(iov: &'a [iovec::IoVec], desc: &'a mut [impl DataDescriptor], mapped_addr: &'a MappedAddress, data: u64, tag: u64, ignore: u64) -> Self {
    
        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.raw_addr(),
                context: std::ptr::null_mut(), // [TODO]
                data,
                tag,
                ignore,
            },
            phantom: PhantomData,
        }
    }

    pub fn new_connected(iov: &'a [iovec::IoVec], desc: &'a mut [impl DataDescriptor], data: u64, tag: u64, ignore: u64) -> Self {
    
        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: FI_ADDR_UNSPEC,
                context: std::ptr::null_mut(), // [TODO]
                data,
                tag,
                ignore,
            },
            phantom: PhantomData,
        }
    }
}

pub struct MsgTaggedMut<'a> {
    pub(crate) c_msg_tagged: libfabric_sys::fi_msg_tagged,
    phantom: PhantomData<&'a ()>
}

impl<'a> MsgTaggedMut<'a> {
    pub fn new(iov: &'a [iovec::IoVecMut], desc: &'a mut [impl DataDescriptor], mapped_addr: &'a MappedAddress, data: u64, tag: u64, ignore: u64) -> Self {
    
        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.raw_addr(),
                context: std::ptr::null_mut(), // [TODO]
                data,
                tag,
                ignore,
            },
            phantom: PhantomData,
        }
    }

    pub fn new_connected(iov: &'a [iovec::IoVecMut], desc: &'a mut [impl DataDescriptor], data: u64, tag: u64, ignore: u64) -> Self {
    
        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: FI_ADDR_UNSPEC,
                context: std::ptr::null_mut(), // [TODO]
                data,
                tag,
                ignore,
            },
            phantom: PhantomData,
        }
    }
}

pub struct MsgAtomic<'a, T> {
    c_msg_atomic: libfabric_sys::fi_msg_atomic,
    phantom: PhantomData<&'a T>
}


impl<'a, T: AsFiType> MsgAtomic<'a, T> {
    pub fn new(iov: &'a [iovec::Ioc<T>], desc: &'a mut [impl DataDescriptor], mapped_addr: &'a MappedAddress, rma_iov: &'a [iovec::RmaIoVec], op: Op) -> Self {
        Self {
            c_msg_atomic: libfabric_sys::fi_msg_atomic {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.raw_addr(),
                rma_iov: rma_iov.as_ptr().cast(),
                rma_iov_count: rma_iov.len(),
                datatype: T::as_fi_datatype(),
                op: op.as_raw(),
                context: std::ptr::null_mut(),
                data: 0,
            },
            phantom: PhantomData
        }
    }

    pub(crate) fn get(&self) -> &libfabric_sys::fi_msg_atomic {
        &self.c_msg_atomic
    }
}

pub struct MsgRma<'a> {
    c_msg_rma: libfabric_sys::fi_msg_rma,
    phantom: PhantomData<&'a ()>,
}

impl<'a> MsgRma<'a> {
    pub fn new<T0>(iov: &'a [iovec::IoVec], desc: &'a mut [impl DataDescriptor], mapped_addr: &'a MappedAddress, rma_iov: &'a [iovec::RmaIoVec], context: &mut T0, data: u64) -> Self {
        Self {
            c_msg_rma : libfabric_sys::fi_msg_rma {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.raw_addr(),
                rma_iov: rma_iov.as_ptr().cast(),
                rma_iov_count: rma_iov.len(),
                context: (context as *mut T0).cast(),
                data,
            },
            phantom: PhantomData,
        }
    }

    pub(crate) fn get(&self) -> &libfabric_sys::fi_msg_rma {
        &self.c_msg_rma
    }

    pub(crate) fn get_mut(&mut self) -> &mut libfabric_sys::fi_msg_rma {
        &mut self.c_msg_rma
    }
}

pub struct MsgRmaMut<'a> {
    c_msg_rma: libfabric_sys::fi_msg_rma,
    phantom: PhantomData<&'a ()>,
}

impl<'a> MsgRmaMut<'a> {
    pub fn new<T0>(iov: &'a [iovec::IoVec], desc: &'a mut [impl DataDescriptor], mapped_addr: &'a MappedAddress, rma_iov: &'a [iovec::RmaIoVec], context: &mut T0, data: u64) -> Self {
        Self {
            c_msg_rma : libfabric_sys::fi_msg_rma {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.raw_addr(),
                rma_iov: rma_iov.as_ptr().cast(),
                rma_iov_count: rma_iov.len(),
                context: (context as *mut T0).cast(),
                data,
            },
            phantom: PhantomData,
        }
    }

    pub(crate) fn get(&self) -> &libfabric_sys::fi_msg_rma {
        &self.c_msg_rma
    }

    pub(crate) fn get_mut(&mut self) -> &mut libfabric_sys::fi_msg_rma {
        &mut self.c_msg_rma
    }
}
