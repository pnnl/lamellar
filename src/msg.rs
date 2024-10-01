use std::marker::PhantomData;

use crate::{
    enums::{AtomicOp, AtomicOperation, CompareAtomicOp, FetchAtomicOp},
    iovec,
    mr::DataDescriptor,
    AsFiType, MappedAddress, FI_ADDR_UNSPEC,
};

pub struct Msg<'a> {
    pub(crate) c_msg: libfabric_sys::fi_msg,
    phantom: PhantomData<&'a ()>,
}

impl<'a> Msg<'a> {
    pub fn from_iov(
        iov: &'a iovec::IoVec,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        data: u64,
    ) -> Self {
        Self {
            c_msg: libfabric_sys::fi_msg {
                msg_iov: iov.get(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: mapped_addr.raw_addr(),
                context: std::ptr::null_mut(), // [TODO]
                data,
            },
            phantom: PhantomData,
        }
    }

    pub fn from_iov_slice(
        iovs: &'a [iovec::IoVec],
        descs: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        data: u64,
    ) -> Self {
        assert_eq!(iovs.len(), descs.len());
        Self {
            c_msg: libfabric_sys::fi_msg {
                msg_iov: iovs.as_ptr().cast(),
                desc: descs.as_mut_ptr().cast(),
                iov_count: descs.len(),
                addr: mapped_addr.raw_addr(),
                context: std::ptr::null_mut(), // [TODO]
                data,
            },
            phantom: PhantomData,
        }
    }
}

pub struct MsgConnected<'a> {
    pub(crate) c_msg: libfabric_sys::fi_msg,
    phantom: PhantomData<&'a ()>,
}

impl<'a> MsgConnected<'a> {
    pub fn from_iov(iov: &'a iovec::IoVec, desc: &'a mut impl DataDescriptor, data: u64) -> Self {
        Self {
            c_msg: libfabric_sys::fi_msg {
                msg_iov: iov.get(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: FI_ADDR_UNSPEC,
                context: std::ptr::null_mut(), // [TODO]
                data,
            },
            phantom: PhantomData,
        }
    }

    pub fn from_iov_slice(
        iov: &'a [iovec::IoVec],
        desc: &'a mut [impl DataDescriptor],
        data: u64,
    ) -> Self {
        assert!(iov.len() == desc.len());
        Self {
            c_msg: libfabric_sys::fi_msg {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: FI_ADDR_UNSPEC,
                context: std::ptr::null_mut(), // [TODO]
                data,
            },
            phantom: PhantomData,
        }
    }
}

pub struct MsgMut<'a> {
    pub(crate) c_msg: libfabric_sys::fi_msg,
    phantom: PhantomData<&'a mut ()>,
}

impl<'a> MsgMut<'a> {
    pub fn from_iov(
        iov: &'a mut iovec::IoVecMut,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
    ) -> Self {
        Self {
            c_msg: libfabric_sys::fi_msg {
                msg_iov: iov.get_mut(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: mapped_addr.raw_addr(),
                context: std::ptr::null_mut(), // [TODO]
                data: 0,
            },
            phantom: PhantomData,
        }
    }

    pub fn from_iov_slice(
        iov: &'a [iovec::IoVecMut],
        desc: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
    ) -> Self {
        Self {
            c_msg: libfabric_sys::fi_msg {
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

    pub fn data(&self) -> u64 {
        self.c_msg.data
    }
}

pub struct MsgConnectedMut<'a> {
    pub(crate) c_msg: libfabric_sys::fi_msg,
    phantom: PhantomData<&'a mut ()>,
}

impl<'a> MsgConnectedMut<'a> {
    pub fn from_iov(iov: &'a mut iovec::IoVecMut, desc: &'a mut impl DataDescriptor) -> Self {
        Self {
            c_msg: libfabric_sys::fi_msg {
                msg_iov: iov.get_mut(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: FI_ADDR_UNSPEC,
                context: std::ptr::null_mut(), // [TODO]
                data: 0,
            },
            phantom: PhantomData,
        }
    }

    pub fn from_iov_slice(iov: &'a [iovec::IoVecMut], desc: &'a mut [impl DataDescriptor]) -> Self {
        Self {
            c_msg: libfabric_sys::fi_msg {
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

    pub fn data(&self) -> u64 {
        self.c_msg.data
    }
}

pub struct MsgTagged<'a> {
    pub(crate) c_msg_tagged: libfabric_sys::fi_msg_tagged,
    phantom: PhantomData<&'a ()>,
}

impl<'a> MsgTagged<'a> {
    pub fn from_iov_slice(
        iov: &'a [iovec::IoVec],
        desc: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        data: u64,
        tag: u64,
        ignore: u64,
    ) -> Self {
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

    pub fn from_iov(
        iov: &'a iovec::IoVec,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        data: u64,
        tag: u64,
        ignore: u64,
    ) -> Self {
        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iov.get(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: mapped_addr.raw_addr(),
                context: std::ptr::null_mut(), // [TODO]
                data,
                tag,
                ignore,
            },
            phantom: PhantomData,
        }
    }

    pub(crate) fn get_mut(&mut self) -> &mut libfabric_sys::fi_msg_tagged {
        &mut self.c_msg_tagged
    }
}

pub struct MsgTaggedConnected<'a> {
    pub(crate) c_msg_tagged: libfabric_sys::fi_msg_tagged,
    phantom: PhantomData<&'a ()>,
}

impl<'a> MsgTaggedConnected<'a> {
    pub fn from_iov_slice(
        iov: &'a [iovec::IoVec],
        desc: &'a mut [impl DataDescriptor],
        data: u64,
        tag: u64,
        ignore: u64,
    ) -> Self {
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

    pub fn from_iov(
        iov: &'a iovec::IoVec,
        desc: &'a mut impl DataDescriptor,
        data: u64,
        tag: u64,
        ignore: u64,
    ) -> Self {
        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iov.get(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: FI_ADDR_UNSPEC,
                context: std::ptr::null_mut(), // [TODO]
                data,
                tag,
                ignore,
            },
            phantom: PhantomData,
        }
    }

    pub(crate) fn get_mut(&mut self) -> &mut libfabric_sys::fi_msg_tagged {
        &mut self.c_msg_tagged
    }
}

pub struct MsgTaggedMut<'a> {
    pub(crate) c_msg_tagged: libfabric_sys::fi_msg_tagged,
    phantom: PhantomData<&'a ()>,
}

impl<'a> MsgTaggedMut<'a> {
    pub fn from_iov_slice(
        iov: &'a [iovec::IoVecMut],
        desc: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        tag: u64,
        ignore: u64,
    ) -> Self {
        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.raw_addr(),
                context: std::ptr::null_mut(), // [TODO]
                data: 0,
                tag,
                ignore,
            },
            phantom: PhantomData,
        }
    }
    pub fn from_iov(
        iov: &'a mut iovec::IoVecMut,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        tag: u64,
        ignore: u64,
    ) -> Self {
        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iov.get_mut(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: mapped_addr.raw_addr(),
                context: std::ptr::null_mut(), // [TODO]
                data: 0,
                tag,
                ignore,
            },
            phantom: PhantomData,
        }
    }

    pub fn data(&self) -> u64 {
        self.c_msg_tagged.data
    }

    pub(crate) fn get_mut(&mut self) -> &mut libfabric_sys::fi_msg_tagged {
        &mut self.c_msg_tagged
    }
}

pub struct MsgTaggedConnectedMut<'a> {
    pub(crate) c_msg_tagged: libfabric_sys::fi_msg_tagged,
    phantom: PhantomData<&'a ()>,
}

impl<'a> MsgTaggedConnectedMut<'a> {
    pub fn from_iov_slice(
        iov: &'a [iovec::IoVecMut],
        desc: &'a mut [impl DataDescriptor],
        tag: u64,
        ignore: u64,
    ) -> Self {
        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: FI_ADDR_UNSPEC,
                context: std::ptr::null_mut(), // [TODO]
                data: 0,
                tag,
                ignore,
            },
            phantom: PhantomData,
        }
    }
    pub fn from_iov(
        iov: &'a mut iovec::IoVecMut,
        desc: &'a mut impl DataDescriptor,
        tag: u64,
        ignore: u64,
    ) -> Self {
        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iov.get_mut(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: FI_ADDR_UNSPEC,
                context: std::ptr::null_mut(), // [TODO]
                data: 0,
                tag,
                ignore,
            },
            phantom: PhantomData,
        }
    }

    pub fn data(&self) -> u64 {
        self.c_msg_tagged.data
    }

    pub(crate) fn get_mut(&mut self) -> &mut libfabric_sys::fi_msg_tagged {
        &mut self.c_msg_tagged
    }
}

pub struct MsgAtomicBase<'a, T: AsFiType, OP: AtomicOperation> {
    c_msg_atomic: libfabric_sys::fi_msg_atomic,
    phantom: PhantomData<&'a T>,
    phantom_op: PhantomData<OP>,
}

impl<'a, T: AsFiType, OP: AtomicOperation> MsgAtomicBase<'a, T, OP> {
    pub fn from_ioc_slice(
        iov: &'a [iovec::Ioc<T>],
        desc: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        rma_iov: &'a [iovec::RmaIoc],
        op: OP,
        data: u64,
    ) -> Self {
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
                data,
            },
            phantom: PhantomData,
            phantom_op: PhantomData,
        }
    }

    pub fn from_ioc(
        iov: &'a iovec::Ioc<T>,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        rma_ioc: &'a iovec::RmaIoc,
        op: OP,
        data: u64,
    ) -> Self {
        Self {
            c_msg_atomic: libfabric_sys::fi_msg_atomic {
                msg_iov: iov.get(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: mapped_addr.raw_addr(),
                rma_iov: rma_ioc.get(),
                rma_iov_count: 1,
                datatype: T::as_fi_datatype(),
                op: op.as_raw(),
                context: std::ptr::null_mut(),
                data,
            },
            phantom: PhantomData,
            phantom_op: PhantomData,
        }
    }

    pub(crate) fn get(&self) -> &libfabric_sys::fi_msg_atomic {
        &self.c_msg_atomic
    }
}

pub type MsgAtomic<'a, T> = MsgAtomicBase<'a, T, AtomicOp>;
pub type MsgFetchAtomic<'a, T> = MsgAtomicBase<'a, T, FetchAtomicOp>;
pub type MsgCompareAtomic<'a, T> = MsgAtomicBase<'a, T, CompareAtomicOp>;

pub struct MsgAtomicConnectedBase<'a, T: AsFiType, OP: AtomicOperation> {
    c_msg_atomic: libfabric_sys::fi_msg_atomic,
    phantom: PhantomData<&'a T>,
    phantom_op: PhantomData<OP>,
}

impl<'a, T: AsFiType, OP: AtomicOperation> MsgAtomicConnectedBase<'a, T, OP> {
    pub fn from_ioc_slice(
        iov: &'a [iovec::Ioc<T>],
        desc: &'a mut [impl DataDescriptor],
        rma_iov: &'a [iovec::RmaIoc],
        op: OP,
        data: u64,
    ) -> Self {
        Self {
            c_msg_atomic: libfabric_sys::fi_msg_atomic {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: FI_ADDR_UNSPEC,
                rma_iov: rma_iov.as_ptr().cast(),
                rma_iov_count: rma_iov.len(),
                datatype: T::as_fi_datatype(),
                op: op.as_raw(),
                context: std::ptr::null_mut(),
                data,
            },
            phantom: PhantomData,
            phantom_op: PhantomData,
        }
    }

    pub fn from_ioc(
        iov: &'a iovec::Ioc<T>,
        desc: &'a mut impl DataDescriptor,
        rma_ioc: &'a iovec::RmaIoc,
        op: OP,
        data: u64,
    ) -> Self {
        Self {
            c_msg_atomic: libfabric_sys::fi_msg_atomic {
                msg_iov: iov.get(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: FI_ADDR_UNSPEC,
                rma_iov: rma_ioc.get(),
                rma_iov_count: 1,
                datatype: T::as_fi_datatype(),
                op: op.as_raw(),
                context: std::ptr::null_mut(),
                data,
            },
            phantom: PhantomData,
            phantom_op: PhantomData,
        }
    }

    pub(crate) fn get(&self) -> &libfabric_sys::fi_msg_atomic {
        &self.c_msg_atomic
    }
}

pub type MsgAtomicConnected<'a, T> = MsgAtomicConnectedBase<'a, T, AtomicOp>;
pub type MsgFetchAtomicConnected<'a, T> = MsgAtomicConnectedBase<'a, T, FetchAtomicOp>;
pub type MsgCompareAtomicConnected<'a, T> = MsgAtomicConnectedBase<'a, T, CompareAtomicOp>;

pub struct MsgAtomicMutBase<'a, T: AsFiType, OP: AtomicOperation> {
    c_msg_atomic: libfabric_sys::fi_msg_atomic,
    phantom: PhantomData<&'a T>,
    phantom_op: PhantomData<OP>,
}

impl<'a, T: AsFiType, OP: AtomicOperation> MsgAtomicMutBase<'a, T, OP> {
    pub fn from_ioc_slice(
        iov: &'a [iovec::IocMut<T>],
        desc: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        rma_iov: &'a [iovec::RmaIoc],
        op: OP,
    ) -> Self {
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
            phantom: PhantomData,
            phantom_op: PhantomData,
        }
    }

    pub fn from_ioc(
        iov: &'a mut iovec::IocMut<T>,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        rma_ioc: &'a iovec::RmaIoc,
        op: OP,
    ) -> Self {
        Self {
            c_msg_atomic: libfabric_sys::fi_msg_atomic {
                msg_iov: iov.get_mut(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: mapped_addr.raw_addr(),
                rma_iov: rma_ioc.get(),
                rma_iov_count: 1,
                datatype: T::as_fi_datatype(),
                op: op.as_raw(),
                context: std::ptr::null_mut(),
                data: 0,
            },
            phantom: PhantomData,
            phantom_op: PhantomData,
        }
    }

    pub fn data(&self) -> u64 {
        self.c_msg_atomic.data
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> &libfabric_sys::fi_msg_atomic {
        &self.c_msg_atomic
    }
}

pub type MsgAtomicMut<'a, T> = MsgAtomicMutBase<'a, T, AtomicOp>;
pub type MsgFetchAtomicMut<'a, T> = MsgAtomicMutBase<'a, T, FetchAtomicOp>;
pub type MsgCompareAtomicMut<'a, T> = MsgAtomicMutBase<'a, T, CompareAtomicOp>;

pub struct MsgAtomicConnectedMutBase<'a, T: AsFiType, OP: AtomicOperation> {
    c_msg_atomic: libfabric_sys::fi_msg_atomic,
    phantom: PhantomData<&'a T>,
    phantom_op: PhantomData<OP>,
}

impl<'a, T: AsFiType, OP: AtomicOperation> MsgAtomicConnectedMutBase<'a, T, OP> {
    pub fn from_ioc_slice(
        iov: &'a [iovec::IocMut<T>],
        desc: &'a mut [impl DataDescriptor],
        rma_iov: &'a [iovec::RmaIoc],
        op: OP,
    ) -> Self {
        Self {
            c_msg_atomic: libfabric_sys::fi_msg_atomic {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: FI_ADDR_UNSPEC,
                rma_iov: rma_iov.as_ptr().cast(),
                rma_iov_count: rma_iov.len(),
                datatype: T::as_fi_datatype(),
                op: op.as_raw(),
                context: std::ptr::null_mut(),
                data: 0,
            },
            phantom: PhantomData,
            phantom_op: PhantomData,
        }
    }

    pub fn from_ioc(
        iov: &'a mut iovec::IocMut<T>,
        desc: &'a mut impl DataDescriptor,
        rma_ioc: &'a iovec::RmaIoc,
        op: OP,
    ) -> Self {
        Self {
            c_msg_atomic: libfabric_sys::fi_msg_atomic {
                msg_iov: iov.get_mut(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: FI_ADDR_UNSPEC,
                rma_iov: rma_ioc.get(),
                rma_iov_count: 1,
                datatype: T::as_fi_datatype(),
                op: op.as_raw(),
                context: std::ptr::null_mut(),
                data: 0,
            },
            phantom: PhantomData,
            phantom_op: PhantomData,
        }
    }

    pub fn data(&self) -> u64 {
        self.c_msg_atomic.data
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> &libfabric_sys::fi_msg_atomic {
        &self.c_msg_atomic
    }
}

pub type MsgAtomicConnectedMut<'a, T> = MsgAtomicConnectedMutBase<'a, T, AtomicOp>;
pub type MsgFetchAtomicConnectedMut<'a, T> = MsgAtomicConnectedMutBase<'a, T, FetchAtomicOp>;
pub type MsgCompareAtomicConnectedMut<'a, T> = MsgAtomicConnectedMutBase<'a, T, CompareAtomicOp>;

pub struct MsgRma<'a> {
    c_msg_rma: libfabric_sys::fi_msg_rma,
    phantom: PhantomData<&'a ()>,
}

impl<'a> MsgRma<'a> {
    pub fn from_iov_slice(
        iov: &'a [iovec::IoVec],
        desc: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        rma_iov: &'a [iovec::RmaIoVec],
        data: u64,
    ) -> Self {
        Self {
            c_msg_rma: libfabric_sys::fi_msg_rma {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.raw_addr(),
                rma_iov: rma_iov.as_ptr().cast(),
                rma_iov_count: rma_iov.len(),
                context: std::ptr::null_mut(),
                data,
            },
            phantom: PhantomData,
        }
    }

    pub fn from_iov(
        iov: &'a iovec::IoVec,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        rma_iov: &'a iovec::RmaIoVec,
        data: u64,
    ) -> Self {
        Self {
            c_msg_rma: libfabric_sys::fi_msg_rma {
                msg_iov: iov.get(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: mapped_addr.raw_addr(),
                rma_iov: rma_iov.get(),
                rma_iov_count: 1,
                context: std::ptr::null_mut(),
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

pub struct MsgRmaConnected<'a> {
    c_msg_rma: libfabric_sys::fi_msg_rma,
    phantom: PhantomData<&'a ()>,
}

impl<'a> MsgRmaConnected<'a> {
    pub fn from_iov_slice(
        iov: &'a [iovec::IoVec],
        desc: &'a mut [impl DataDescriptor],
        rma_iov: &'a [iovec::RmaIoVec],
        data: u64,
    ) -> Self {
        Self {
            c_msg_rma: libfabric_sys::fi_msg_rma {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: FI_ADDR_UNSPEC,
                rma_iov: rma_iov.as_ptr().cast(),
                rma_iov_count: rma_iov.len(),
                context: std::ptr::null_mut(),
                data,
            },
            phantom: PhantomData,
        }
    }

    pub fn from_iov(
        iov: &'a iovec::IoVec,
        desc: &'a mut impl DataDescriptor,
        rma_iov: &'a iovec::RmaIoVec,
        data: u64,
    ) -> Self {
        Self {
            c_msg_rma: libfabric_sys::fi_msg_rma {
                msg_iov: iov.get(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: FI_ADDR_UNSPEC,
                rma_iov: rma_iov.get(),
                rma_iov_count: 1,
                context: std::ptr::null_mut(),
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
    pub fn from_iov_slice(
        iov: &'a [iovec::IoVecMut],
        desc: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        rma_iov: &'a [iovec::RmaIoVec],
    ) -> Self {
        Self {
            c_msg_rma: libfabric_sys::fi_msg_rma {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.raw_addr(),
                rma_iov: rma_iov.as_ptr().cast(),
                rma_iov_count: rma_iov.len(),
                context: std::ptr::null_mut(),
                data: 0,
            },
            phantom: PhantomData,
        }
    }

    pub fn from_iov(
        iov: &'a mut iovec::IoVecMut,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        rma_iov: &'a iovec::RmaIoVec,
    ) -> Self {
        Self {
            c_msg_rma: libfabric_sys::fi_msg_rma {
                msg_iov: iov.get_mut(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: mapped_addr.raw_addr(),
                rma_iov: rma_iov.get(),
                rma_iov_count: 1,
                context: std::ptr::null_mut(),
                data: 0,
            },
            phantom: PhantomData,
        }
    }

    pub fn data(&self) -> u64 {
        self.c_msg_rma.data
    }

    pub(crate) fn get(&self) -> &libfabric_sys::fi_msg_rma {
        &self.c_msg_rma
    }

    pub(crate) fn get_mut(&mut self) -> &mut libfabric_sys::fi_msg_rma {
        &mut self.c_msg_rma
    }
}

pub struct MsgRmaConnectedMut<'a> {
    c_msg_rma: libfabric_sys::fi_msg_rma,
    phantom: PhantomData<&'a ()>,
}

impl<'a> MsgRmaConnectedMut<'a> {
    pub fn from_iov_slice(
        iov: &'a [iovec::IoVecMut],
        desc: &'a mut [impl DataDescriptor],
        rma_iov: &'a [iovec::RmaIoVec],
    ) -> Self {
        Self {
            c_msg_rma: libfabric_sys::fi_msg_rma {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: FI_ADDR_UNSPEC,
                rma_iov: rma_iov.as_ptr().cast(),
                rma_iov_count: rma_iov.len(),
                context: std::ptr::null_mut(),
                data: 0,
            },
            phantom: PhantomData,
        }
    }

    pub fn from_iov(
        iov: &'a mut iovec::IoVecMut,
        desc: &'a mut impl DataDescriptor,
        rma_iov: &'a iovec::RmaIoVec,
    ) -> Self {
        Self {
            c_msg_rma: libfabric_sys::fi_msg_rma {
                msg_iov: iov.get_mut(),
                desc: desc.get_desc_ptr(),
                iov_count: 1,
                addr: FI_ADDR_UNSPEC,
                rma_iov: rma_iov.get(),
                rma_iov_count: 1,
                context: std::ptr::null_mut(),
                data: 0,
            },
            phantom: PhantomData,
        }
    }

    pub fn data(&self) -> u64 {
        self.c_msg_rma.data
    }

    pub(crate) fn get(&self) -> &libfabric_sys::fi_msg_rma {
        &self.c_msg_rma
    }

    pub(crate) fn get_mut(&mut self) -> &mut libfabric_sys::fi_msg_rma {
        &mut self.c_msg_rma
    }
}
