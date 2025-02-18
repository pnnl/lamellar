use std::marker::PhantomData;

use crate::{
    enums::{AtomicOp, AtomicOperation, CompareAtomicOp, FetchAtomicOp},
    iovec,
    mr::DataDescriptor,
    AsFiType, Context, MappedAddress, FI_ADDR_UNSPEC,
};

pub struct Msg<'a> {
    pub(crate) c_msg: libfabric_sys::fi_msg,
    phantom: PhantomData<&'a ()>,
    pub(crate) context: &'a mut Context,
}

impl<'a> Msg<'a> {
    fn new(
        iovs: &'a [iovec::IoVec],
        descs: &'a mut [impl DataDescriptor],
        mapped_addr: Option<&'a MappedAddress>,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        assert_eq!(iovs.len(), descs.len());

        Self {
            c_msg: libfabric_sys::fi_msg {
                msg_iov: iovs.as_ptr().cast(),
                desc: descs.as_mut_ptr().cast(),
                iov_count: descs.len(),
                addr: mapped_addr.map_or_else(|| FI_ADDR_UNSPEC, |v| v.raw_addr()),
                context: context.inner_mut(),
                data: data.unwrap_or(0),
            },
            phantom: PhantomData,
            context,
        }
    }

    pub fn from_iov_slice(
        iovs: &'a [iovec::IoVec],
        descs: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Msg::new(iovs, descs, Some(mapped_addr), data, context)
    }

    pub fn from_iov(
        iov: &'a iovec::IoVec,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Msg::new(
            std::slice::from_ref(iov),
            std::slice::from_mut(desc),
            Some(mapped_addr),
            data,
            context,
        )
    }

    pub fn context(&mut self) -> &mut Context {
        &mut self.context
    }

    pub fn data(&self) -> Option<u64> {
        if self.c_msg.data != 0 {
            Some(self.c_msg.data)
        } else {
            None
        }
    }

    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg {
        &self.c_msg
    }

    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg {
        &mut self.c_msg
    }
}

pub struct MsgConnected<'a> {
    msg: Msg<'a>,
}

impl<'a> MsgConnected<'a> {
    pub fn from_iov(
        iov: &'a iovec::IoVec,
        desc: &'a mut impl DataDescriptor,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: Msg::new(
                std::slice::from_ref(iov),
                std::slice::from_mut(desc),
                None,
                data,
                context,
            ),
        }
    }

    pub fn from_iov_slice(
        iovs: &'a [iovec::IoVec],
        descs: &'a mut [impl DataDescriptor],
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: Msg::new(iovs, descs, None, data, context),
        }
    }

    pub fn context(&mut self) -> &mut Context {
        &mut self.msg.context
    }

    pub fn data(&self) -> Option<u64> {
        self.msg.data()
    }

    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg {
        self.msg.inner()
    }

    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg {
        self.msg.inner_mut()
    }
}

pub struct MsgMut<'a> {
    pub(crate) c_msg: libfabric_sys::fi_msg,
    phantom: PhantomData<&'a mut ()>,
    pub(crate) context: &'a mut Context,
}

impl<'a> MsgMut<'a> {
    fn new(
        iovs: &'a mut [iovec::IoVecMut],
        descs: &'a mut [impl DataDescriptor],
        mapped_addr: Option<&'a MappedAddress>,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        assert_eq!(iovs.len(), descs.len());

        Self {
            c_msg: libfabric_sys::fi_msg {
                msg_iov: iovs.as_ptr().cast(),
                desc: descs.as_mut_ptr().cast(),
                iov_count: descs.len(),
                addr: mapped_addr.map_or_else(|| FI_ADDR_UNSPEC, |v| v.raw_addr()),
                context: context.inner_mut(),
                data: data.unwrap_or(0),
            },
            phantom: PhantomData,
            context,
        }
    }

    pub fn from_iov(
        iov: &'a mut iovec::IoVecMut,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        MsgMut::new(
            std::slice::from_mut(iov),
            std::slice::from_mut(desc),
            Some(mapped_addr),
            data,
            context,
        )
    }

    pub fn from_iov_slice(
        iov: &'a mut [iovec::IoVecMut],
        desc: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        MsgMut::new(iov, desc, Some(mapped_addr), data, context)
    }

    pub fn context(&mut self) -> &mut Context {
        &mut self.context
    }

    pub fn data(&self) -> Option<u64> {
        if self.c_msg.data != 0 {
            Some(self.c_msg.data)
        } else {
            None
        }
    }

    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg {
        &self.c_msg
    }

    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg {
        &mut self.c_msg
    }
}

pub struct MsgConnectedMut<'a> {
    msg: MsgMut<'a>,
}

impl<'a> MsgConnectedMut<'a> {
    pub fn from_iov(
        iov: &'a mut iovec::IoVecMut,
        desc: &'a mut impl DataDescriptor,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: MsgMut::new(
                std::slice::from_mut(iov),
                std::slice::from_mut(desc),
                None,
                data,
                context,
            ),
        }
    }

    pub fn from_iov_slice(
        iovs: &'a mut [iovec::IoVecMut],
        descs: &'a mut [impl DataDescriptor],
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: MsgMut::new(iovs, descs, None, data, context),
        }
    }

    pub fn context(&mut self) -> &mut Context {
        &mut self.msg.context
    }

    pub fn data(&self) -> Option<u64> {
        self.msg.data()
    }

    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg {
        self.msg.inner()
    }

    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg {
        self.msg.inner_mut()
    }
}

pub struct MsgTagged<'a> {
    pub(crate) c_msg_tagged: libfabric_sys::fi_msg_tagged,
    phantom: PhantomData<&'a ()>,
    pub(crate) context: &'a mut Context,
}

impl<'a> MsgTagged<'a> {
    fn new(
        iovs: &'a [iovec::IoVec],
        descs: &'a mut [impl DataDescriptor],
        mapped_addr: Option<&'a MappedAddress>,
        data: Option<u64>,
        tag: u64,
        ignore: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        assert_eq!(iovs.len(), descs.len());

        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iovs.as_ptr().cast(),
                desc: descs.as_mut_ptr().cast(),
                iov_count: iovs.len(),
                addr: mapped_addr.map_or_else(|| FI_ADDR_UNSPEC, |v| v.raw_addr()),
                context: context.inner_mut(),
                data: data.unwrap_or(0),
                tag,
                ignore: ignore.unwrap_or(0),
            },
            phantom: PhantomData,
            context,
        }
    }

    pub fn from_iov_slice(
        iovs: &'a [iovec::IoVec],
        descs: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        data: Option<u64>,
        tag: u64,
        ignore: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        MsgTagged::new(iovs, descs, Some(mapped_addr), data, tag, ignore, context)
    }

    pub fn from_iov(
        iov: &'a iovec::IoVec,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        data: Option<u64>,
        tag: u64,
        ignore: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        MsgTagged::new(
            std::slice::from_ref(iov),
            std::slice::from_mut(desc),
            Some(mapped_addr),
            data,
            tag,
            ignore,
            context,
        )
    }

    pub fn data(&self) -> Option<u64> {
        if self.c_msg_tagged.data != 0 {
            Some(self.c_msg_tagged.data)
        } else {
            None
        }
    }

    pub fn context(&mut self) -> &mut Context {
        &mut self.context
    }

    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg_tagged {
        &self.c_msg_tagged
    }

    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg_tagged {
        &mut self.c_msg_tagged
    }
}

pub struct MsgTaggedConnected<'a> {
    msg: MsgTagged<'a>,
}

impl<'a> MsgTaggedConnected<'a> {
    pub fn from_iov_slice(
        iovs: &'a [iovec::IoVec],
        descs: &'a mut [impl DataDescriptor],
        data: Option<u64>,
        tag: u64,
        ignore: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: MsgTagged::new(iovs, descs, None, data, tag, ignore, context),
        }
    }

    pub fn from_iov(
        iov: &'a iovec::IoVec,
        desc: &'a mut impl DataDescriptor,
        data: Option<u64>,
        tag: u64,
        ignore: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: MsgTagged::new(
                std::slice::from_ref(iov),
                std::slice::from_mut(desc),
                None,
                data,
                tag,
                ignore,
                context,
            ),
        }
    }

    pub fn data(&self) -> Option<u64> {
        self.msg.data()
    }

    pub fn context(&mut self) -> &mut Context {
        &mut self.msg.context
    }

    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg_tagged {
        self.msg.inner()
    }

    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg_tagged {
        self.msg.inner_mut()
    }
}

pub struct MsgTaggedMut<'a> {
    pub(crate) c_msg_tagged: libfabric_sys::fi_msg_tagged,
    phantom: PhantomData<&'a mut ()>,
    pub(crate) context: &'a mut Context,
}

impl<'a> MsgTaggedMut<'a> {
    fn new(
        iovs: &'a mut [iovec::IoVecMut],
        descs: &'a mut [impl DataDescriptor],
        mapped_addr: Option<&'a MappedAddress>,
        data: Option<u64>,
        tag: u64,
        ignore: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        assert_eq!(iovs.len(), descs.len());

        Self {
            c_msg_tagged: libfabric_sys::fi_msg_tagged {
                msg_iov: iovs.as_ptr().cast(),
                desc: descs.as_mut_ptr().cast(),
                iov_count: iovs.len(),
                addr: mapped_addr.map_or_else(|| FI_ADDR_UNSPEC, |v| v.raw_addr()),
                context: context.inner_mut(),
                data: data.unwrap_or(0),
                tag,
                ignore: ignore.unwrap_or(0),
            },
            phantom: PhantomData,
            context,
        }
    }

    pub fn from_iov(
        iov: &'a mut iovec::IoVecMut,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        data: Option<u64>,
        tag: u64,
        ignore: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        MsgTaggedMut::new(
            std::slice::from_mut(iov),
            std::slice::from_mut(desc),
            Some(mapped_addr),
            data,
            tag,
            ignore,
            context,
        )
    }

    pub fn from_iov_slice(
        iovs: &'a mut [iovec::IoVecMut],
        descs: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        data: Option<u64>,
        tag: u64,
        ignore: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        MsgTaggedMut::new(iovs, descs, Some(mapped_addr), data, tag, ignore, context)
    }

    pub fn data(&self) -> Option<u64> {
        if self.c_msg_tagged.data != 0 {
            Some(self.c_msg_tagged.data)
        } else {
            None
        }
    }

    pub(crate) fn context(&mut self) -> &mut Context {
        &mut self.context
    }

    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg_tagged {
        &self.c_msg_tagged
    }

    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg_tagged {
        &mut self.c_msg_tagged
    }
}

pub struct MsgTaggedConnectedMut<'a> {
    msg: MsgTaggedMut<'a>,
}

impl<'a> MsgTaggedConnectedMut<'a> {
    pub fn from_iov_slice(
        iovs: &'a mut [iovec::IoVecMut],
        descs: &'a mut [impl DataDescriptor],
        data: Option<u64>,
        tag: u64,
        ignore: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: MsgTaggedMut::new(iovs, descs, None, data, tag, ignore, context),
        }
    }

    pub fn from_iov(
        iov: &'a mut iovec::IoVecMut,
        desc: &'a mut impl DataDescriptor,
        data: Option<u64>,
        tag: u64,
        ignore: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: MsgTaggedMut::new(
                std::slice::from_mut(iov),
                std::slice::from_mut(desc),
                None,
                data,
                tag,
                ignore,
                context,
            ),
        }
    }

    pub(crate) fn context(&mut self) -> &mut Context {
        &mut self.msg.context
    }

    pub fn data(&self) -> Option<u64> {
        if self.msg.c_msg_tagged.data != 0 {
            Some(self.msg.c_msg_tagged.data)
        } else {
            None
        }
    }

    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg_tagged {
        self.msg.inner()
    }

    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg_tagged {
        self.msg.inner_mut()
    }
}

pub struct MsgAtomicBase<'a, T: AsFiType, OP: AtomicOperation> {
    pub(crate) c_msg_atomic: libfabric_sys::fi_msg_atomic,
    phantom: PhantomData<&'a T>,
    phantom_op: PhantomData<OP>,
    pub(crate) context: &'a mut Context,
}

impl<'a, T: AsFiType, OP: AtomicOperation> MsgAtomicBase<'a, T, OP> {
    fn new(
        iov: &'a [iovec::Ioc<T>],
        desc: &'a mut [impl DataDescriptor],
        mapped_addr: Option<&'a MappedAddress>,
        rma_iov: &'a [iovec::RmaIoc],
        op: OP,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        assert_eq!(iov.len(), desc.len());
        assert_eq!(rma_iov.len(), desc.len());
        Self {
            c_msg_atomic: libfabric_sys::fi_msg_atomic {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.map_or_else(|| FI_ADDR_UNSPEC, |v| v.raw_addr()),
                context: context.inner_mut(),
                rma_iov: rma_iov.as_ptr().cast(),
                rma_iov_count: rma_iov.len(),
                datatype: T::as_fi_datatype(),
                op: op.as_raw(),
                data: data.unwrap_or(0),
            },
            phantom: PhantomData,
            phantom_op: PhantomData,
            context,
        }
    }

    pub fn from_ioc_slice(
        iovs: &'a [iovec::Ioc<T>],
        descs: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        rma_iovs: &'a [iovec::RmaIoc],
        op: OP,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        MsgAtomicBase::new(iovs, descs, Some(mapped_addr), rma_iovs, op, data, context)
    }

    pub fn from_ioc(
        iov: &'a iovec::Ioc<T>,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        rma_ioc: &'a iovec::RmaIoc,
        op: OP,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        MsgAtomicBase::new(
            std::slice::from_ref(iov),
            std::slice::from_mut(desc),
            Some(mapped_addr),
            std::slice::from_ref(rma_ioc),
            op,
            data,
            context,
        )
    }

    pub(crate) fn context(&mut self) -> &mut Context {
        &mut self.context
    }

    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg_atomic {
        &self.c_msg_atomic
    }

    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg_atomic {
        &mut self.c_msg_atomic
    }
}

pub type MsgAtomic<'a, T> = MsgAtomicBase<'a, T, AtomicOp>;
pub type MsgFetchAtomic<'a, T> = MsgAtomicBase<'a, T, FetchAtomicOp>;
pub type MsgCompareAtomic<'a, T> = MsgAtomicBase<'a, T, CompareAtomicOp>;

pub struct MsgAtomicConnectedBase<'a, T: AsFiType, OP: AtomicOperation> {
    msg: MsgAtomicBase<'a, T, OP>,
}

impl<'a, T: AsFiType, OP: AtomicOperation> MsgAtomicConnectedBase<'a, T, OP> {
    pub fn from_ioc_slice(
        iovs: &'a [iovec::Ioc<T>],
        descs: &'a mut [impl DataDescriptor],
        rma_iovs: &'a [iovec::RmaIoc],
        op: OP,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: MsgAtomicBase::new(iovs, descs, None, rma_iovs, op, data, context),
        }
    }

    pub fn from_ioc(
        iov: &'a iovec::Ioc<T>,
        desc: &'a mut impl DataDescriptor,
        rma_ioc: &'a iovec::RmaIoc,
        op: OP,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: MsgAtomicBase::new(
                std::slice::from_ref(iov),
                std::slice::from_mut(desc),
                None,
                std::slice::from_ref(rma_ioc),
                op,
                data,
                context,
            ),
        }
    }

    pub(crate) fn context(&mut self) -> &mut Context {
        &mut self.msg.context
    }

    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg_atomic {
        self.msg.inner()
    }

    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg_atomic {
        self.msg.inner_mut()
    }
}

pub type MsgAtomicConnected<'a, T> = MsgAtomicConnectedBase<'a, T, AtomicOp>;
pub type MsgFetchAtomicConnected<'a, T> = MsgAtomicConnectedBase<'a, T, FetchAtomicOp>;
pub type MsgCompareAtomicConnected<'a, T> = MsgAtomicConnectedBase<'a, T, CompareAtomicOp>;

pub struct MsgAtomicMutBase<'a, T: AsFiType, OP: AtomicOperation> {
    c_msg_atomic: libfabric_sys::fi_msg_atomic,
    phantom: PhantomData<&'a mut T>,
    phantom_op: PhantomData<OP>,
    pub(crate) context: &'a mut Context,
}

impl<'a, T: AsFiType, OP: AtomicOperation> MsgAtomicMutBase<'a, T, OP> {
    fn new(
        iovs: &'a [iovec::IocMut<T>],
        descs: &'a mut [impl DataDescriptor],
        mapped_addr: Option<&'a MappedAddress>,
        rma_iovs: &'a [iovec::RmaIoc],
        op: OP,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            c_msg_atomic: libfabric_sys::fi_msg_atomic {
                msg_iov: iovs.as_ptr().cast(),
                desc: descs.as_mut_ptr().cast(),
                iov_count: iovs.len(),
                addr: mapped_addr.map_or_else(|| FI_ADDR_UNSPEC, |v| v.raw_addr()),
                context: context.inner_mut(),
                rma_iov: rma_iovs.as_ptr().cast(),
                rma_iov_count: rma_iovs.len(),
                datatype: T::as_fi_datatype(),
                op: op.as_raw(),
                data: data.unwrap_or(0),
            },
            phantom: PhantomData,
            phantom_op: PhantomData,
            context,
        }
    }

    pub fn from_ioc_slice(
        iovs: &'a [iovec::IocMut<T>],
        descs: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        rma_iovs: &'a [iovec::RmaIoc],
        op: OP,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self::new(iovs, descs, Some(&mapped_addr), rma_iovs, op, data, context)
    }

    pub fn from_ioc(
        iov: &'a mut iovec::IocMut<T>,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        rma_ioc: &'a iovec::RmaIoc,
        op: OP,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self::new(
            std::slice::from_ref(iov),
            std::slice::from_mut(desc),
            Some(mapped_addr),
            std::slice::from_ref(rma_ioc),
            op,
            data,
            context,
        )
    }

    pub fn data(&self) -> Option<u64> {
        if self.c_msg_atomic.data != 0 {
            Some(self.c_msg_atomic.data)
        } else {
            None
        }
    }

    pub(crate) fn context(&mut self) -> &mut Context {
        &mut self.context
    }

    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg_atomic {
        &self.c_msg_atomic
    }

    #[allow(dead_code)]
    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg_atomic {
        &mut self.c_msg_atomic
    }
}

pub type MsgAtomicMut<'a, T> = MsgAtomicMutBase<'a, T, AtomicOp>;
pub type MsgFetchAtomicMut<'a, T> = MsgAtomicMutBase<'a, T, FetchAtomicOp>;
pub type MsgCompareAtomicMut<'a, T> = MsgAtomicMutBase<'a, T, CompareAtomicOp>;

pub struct MsgAtomicConnectedMutBase<'a, T: AsFiType, OP: AtomicOperation> {
    msg: MsgAtomicMutBase<'a, T, OP>,
}

impl<'a, T: AsFiType, OP: AtomicOperation> MsgAtomicConnectedMutBase<'a, T, OP> {
    pub fn from_ioc_slice(
        iovs: &'a [iovec::IocMut<T>],
        descs: &'a mut [impl DataDescriptor],
        rma_iovs: &'a [iovec::RmaIoc],
        data: Option<u64>,
        op: OP,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: MsgAtomicMutBase::new(iovs, descs, None, rma_iovs, op, data, context),
        }
    }

    pub fn from_ioc(
        iov: &'a mut iovec::IocMut<T>,
        desc: &'a mut impl DataDescriptor,
        rma_ioc: &'a iovec::RmaIoc,
        data: Option<u64>,
        op: OP,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: MsgAtomicMutBase::new(
                std::slice::from_ref(iov),
                std::slice::from_mut(desc),
                None,
                std::slice::from_ref(rma_ioc),
                op,
                data,
                context,
            ),
        }
    }

    pub fn data(&self) -> Option<u64> {
        self.msg.data()
    }

    pub(crate) fn context(&mut self) -> &mut Context {
        &mut self.msg.context
    }

    #[allow(dead_code)]
    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg_atomic {
        self.msg.inner()
    }

    #[allow(dead_code)]
    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg_atomic {
        self.msg.inner_mut()
    }
}

pub type MsgAtomicConnectedMut<'a, T> = MsgAtomicConnectedMutBase<'a, T, AtomicOp>;
pub type MsgFetchAtomicConnectedMut<'a, T> = MsgAtomicConnectedMutBase<'a, T, FetchAtomicOp>;
pub type MsgCompareAtomicConnectedMut<'a, T> = MsgAtomicConnectedMutBase<'a, T, CompareAtomicOp>;

pub struct MsgRma<'a> {
    c_msg_rma: libfabric_sys::fi_msg_rma,
    phantom: PhantomData<&'a ()>,
    context: &'a mut Context,
}

impl<'a> MsgRma<'a> {
    fn new(
        iov: &'a [iovec::IoVec],
        desc: &'a mut [impl DataDescriptor],
        mapped_addr: Option<&'a MappedAddress>,
        rma_iov: &'a [iovec::RmaIoVec],
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        assert_eq!(iov.len(), desc.len());
        assert_eq!(iov.len(), rma_iov.len());
        Self {
            c_msg_rma: libfabric_sys::fi_msg_rma {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.map_or_else(|| FI_ADDR_UNSPEC, |v| v.raw_addr()),
                context: context.inner_mut(),
                rma_iov: rma_iov.as_ptr().cast(),
                rma_iov_count: rma_iov.len(),
                data: data.unwrap_or(0),
            },
            phantom: PhantomData,
            context,
        }
    }

    pub fn from_iov_slice(
        iovs: &'a [iovec::IoVec],
        descs: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        rma_iovs: &'a [iovec::RmaIoVec],
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self::new(iovs, descs, Some(mapped_addr), rma_iovs, data, context)
    }

    pub fn from_iov(
        iov: &'a iovec::IoVec,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        rma_iov: &'a iovec::RmaIoVec,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self::new(
            std::slice::from_ref(iov),
            std::slice::from_mut(desc),
            Some(mapped_addr),
            std::slice::from_ref(rma_iov),
            data,
            context,
        )
    }

    pub(crate) fn context(&mut self) -> &mut Context {
        &mut self.context
    }

    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg_rma {
        &self.c_msg_rma
    }

    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg_rma {
        &mut self.c_msg_rma
    }
}

pub struct MsgRmaConnected<'a> {
    msg: MsgRma<'a>,
}

impl<'a> MsgRmaConnected<'a> {
    pub fn from_iov_slice(
        iov: &'a [iovec::IoVec],
        desc: &'a mut [impl DataDescriptor],
        rma_iov: &'a [iovec::RmaIoVec],
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: MsgRma::new(iov, desc, None, rma_iov, data, context),
        }
    }

    pub fn from_iov(
        iov: &'a iovec::IoVec,
        desc: &'a mut impl DataDescriptor,
        rma_iov: &'a iovec::RmaIoVec,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: MsgRma::new(
                std::slice::from_ref(iov),
                std::slice::from_mut(desc),
                None,
                std::slice::from_ref(rma_iov),
                data,
                context,
            ),
        }
    }

    pub(crate) fn context(&mut self) -> &mut Context {
        &mut self.msg.context
    }

    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg_rma {
        self.msg.inner()
    }

    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg_rma {
        self.msg.inner_mut()
    }
}

pub struct MsgRmaMut<'a> {
    c_msg_rma: libfabric_sys::fi_msg_rma,
    phantom: PhantomData<&'a ()>,
    context: &'a mut Context,
}

impl<'a> MsgRmaMut<'a> {
    fn new(
        iov: &'a mut [iovec::IoVecMut],
        desc: &'a mut [impl DataDescriptor],
        mapped_addr: Option<&'a MappedAddress>,
        rma_iov: &'a [iovec::RmaIoVec],
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        assert_eq!(iov.len(), desc.len());
        assert_eq!(iov.len(), rma_iov.len());
        Self {
            c_msg_rma: libfabric_sys::fi_msg_rma {
                msg_iov: iov.as_ptr().cast(),
                desc: desc.as_mut_ptr().cast(),
                iov_count: iov.len(),
                addr: mapped_addr.map_or_else(|| FI_ADDR_UNSPEC, |v| v.raw_addr()),
                context: context.inner_mut(),
                rma_iov: rma_iov.as_ptr().cast(),
                rma_iov_count: rma_iov.len(),
                data: data.unwrap_or(0),
            },
            phantom: PhantomData,
            context,
        }
    }

    pub fn from_iov_slice(
        iov: &'a mut [iovec::IoVecMut],
        desc: &'a mut [impl DataDescriptor],
        mapped_addr: &'a MappedAddress,
        rma_iov: &'a [iovec::RmaIoVec],
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self::new(iov, desc, Some(mapped_addr), rma_iov, data, context)
    }

    pub fn from_iov(
        iov: &'a mut iovec::IoVecMut,
        desc: &'a mut impl DataDescriptor,
        mapped_addr: &'a MappedAddress,
        rma_iov: &'a iovec::RmaIoVec,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self::new(
            std::slice::from_mut(iov),
            std::slice::from_mut(desc),
            Some(mapped_addr),
            std::slice::from_ref(rma_iov),
            data,
            context,
        )
    }

    pub fn data(&self) -> Option<u64> {
        if self.c_msg_rma.data != 0 {
            Some(self.c_msg_rma.data)
        } else {
            None
        }
    }

    pub(crate) fn context(&mut self) -> &mut Context {
        &mut self.context
    }

    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg_rma {
        &self.c_msg_rma
    }

    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg_rma {
        &mut self.c_msg_rma
    }
}

pub struct MsgRmaConnectedMut<'a> {
    msg: MsgRmaMut<'a>,
}

impl<'a> MsgRmaConnectedMut<'a> {
    pub fn from_iov_slice(
        iovs: &'a mut [iovec::IoVecMut],
        descs: &'a mut [impl DataDescriptor],
        rma_iovs: &'a [iovec::RmaIoVec],
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: MsgRmaMut::new(iovs, descs, None, rma_iovs, data, context),
        }
    }

    pub fn from_iov(
        iov: &'a mut iovec::IoVecMut,
        desc: &'a mut impl DataDescriptor,
        rma_iov: &'a iovec::RmaIoVec,
        data: Option<u64>,
        context: &'a mut Context,
    ) -> Self {
        Self {
            msg: MsgRmaMut::new(
                std::slice::from_mut(iov),
                std::slice::from_mut(desc),
                None,
                std::slice::from_ref(rma_iov),
                data,
                context,
            ),
        }
    }

    pub fn data(&self) -> Option<u64> {
        self.msg.data()
    }

    pub(crate) fn context(&mut self) -> &mut Context {
        &mut self.msg.context
    }

    pub(crate) fn inner(&self) -> &libfabric_sys::fi_msg_rma {
        self.msg.inner()
    }

    pub(crate) fn inner_mut(&mut self) -> &mut libfabric_sys::fi_msg_rma {
        self.msg.inner_mut()
    }
}
