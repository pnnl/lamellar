use crate::FI_ADDR_UNSPEC;
use crate::cq::ReadCq;
use crate::enums::AtomicFetchMsgOptions;
use crate::enums::AtomicMsgOptions;
use crate::ep::EndpointBase;
use crate::ep::EndpointImplBase;
use crate::eq::ReadEq;
use crate::fid::AsRawTypedFid;
use crate::fid::EpRawFid;
use crate::infocapsoptions::AtomicCap;
use crate::infocapsoptions::ReadMod;
use crate::infocapsoptions::WriteMod;
use crate::mr::DataDescriptor;
use crate::mr::MappedMemoryRegionKey;
use crate::utils::check_error;
use crate::utils::to_fi_datatype;
use crate::xcontext::ReceiveContext;
use crate::xcontext::ReceiveContextBase;
use crate::xcontext::ReceiveContextImpl;
use crate::xcontext::ReceiveContextImplBase;
use crate::xcontext::TransmitContext;
use crate::xcontext::TransmitContextBase;
use crate::xcontext::TransmitContextImpl;
use crate::xcontext::TransmitContextImplBase;
use super::message::extract_raw_addr_and_ctx;

pub(crate) trait AtomicWriteEpImpl: AtomicWriteEp + AsRawTypedFid<Output = EpRawFid> + AtomicValidEp{
    #[allow(clippy::too_many_arguments)]
    fn atomic_impl<T: 'static, T0>(&self, buf: &[T],  desc: &mut impl DataDescriptor, dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.as_raw_typed_fid(), buf.as_ptr().cast(), buf.len(), desc.get_desc(), raw_addr, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), ctx)};
        check_error(err)
    }
    
    #[allow(clippy::too_many_arguments)]
    fn atomicv_impl<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : Option<*mut T0>) -> Result<(), crate::error::Error>{
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.as_raw_typed_fid(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), raw_addr, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), ctx)};
        check_error(err)
    }

    fn atomicmsg_impl(&self, msg: &crate::msg::MsgAtomic, options: AtomicMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicmsg(self.as_raw_typed_fid(), msg.c_msg_atomic, options.get_value()) };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn inject_atomic_impl<T: 'static>(&self, buf: &[T], dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let raw_addr = if let Some(addr) = dest_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };
        let err = unsafe{ libfabric_sys::inlined_fi_inject_atomic(self.as_raw_typed_fid(), buf.as_ptr().cast(), buf.len(), raw_addr, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value())};
        check_error(err)
    }
}

pub trait AtomicWriteEp {
    #[allow(clippy::too_many_arguments)]
    fn atomic_to<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn atomic_to_with_context<T: 'static, T0>(&self, buf: &[T],  desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context: &mut T0) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn atomic_with_context<T: 'static, T0>(&self, buf: &[T],  desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context: &mut T0) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn atomicv_to<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor],  dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn atomicv_to_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor],  mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> ;
    fn atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> ;
    fn atomicmsg(&self, msg: &crate::msg::MsgAtomic, options: AtomicMsgOptions) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn inject_atomic_to<T: 'static>(&self, buf: &[T], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn inject_atomic<T: 'static>(&self, buf: &[T], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> ;
}

impl<EP: AtomicWriteEpImpl> AtomicWriteEp for EP {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn atomic_to<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        self.atomic_impl::<T, ()>(buf, desc, Some(dest_addr), mem_addr, mapped_key, op, None)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        self.atomic_impl::<T, ()>(buf, desc, None, mem_addr, mapped_key, op, None)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn atomic_to_with_context<T: 'static, T0>(&self, buf: &[T],  desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context: &mut T0) -> Result<(), crate::error::Error> {
        self.atomic_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, op, Some(context))
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn atomic_with_context<T: 'static, T0>(&self, buf: &[T],  desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context: &mut T0) -> Result<(), crate::error::Error> {
        self.atomic_impl(buf, desc, None, mem_addr, mapped_key, op, Some(context))
    }


    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn atomicv_to<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor],  dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        self.atomicv_impl::<T, ()>(ioc, desc, Some(dest_addr), mem_addr, mapped_key, op, None)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn atomicv_to_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        self.atomicv_impl(ioc, desc, Some(dest_addr), mem_addr, mapped_key, op, Some(context))
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor],  mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        self.atomicv_impl::<T, ()>(ioc, desc, None, mem_addr, mapped_key, op, None)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        self.atomicv_impl(ioc, desc, None, mem_addr, mapped_key, op, Some(context))
    }

    fn atomicmsg(&self, msg: &crate::msg::MsgAtomic, options: AtomicMsgOptions) -> Result<(), crate::error::Error> {
        self.atomicmsg_impl(msg, options)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn inject_atomic_to<T: 'static>(&self, buf: &[T], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        self.inject_atomic_impl(buf, Some(dest_addr), mem_addr, mapped_key, op)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn inject_atomic<T: 'static>(&self, buf: &[T], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        self.inject_atomic_impl(buf, None, mem_addr, mapped_key, op)
    }
}

// impl<E: AtomicCap+ WriteMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> EndpointBase<E> {
impl<EP: AtomicCap + WriteMod, EQ: ?Sized, CQ: ?Sized + ReadCq>  AtomicWriteEpImpl for EndpointImplBase<EP, EQ, CQ> {}
impl<E: AtomicWriteEpImpl>  AtomicWriteEpImpl for EndpointBase<E> {}



pub(crate) trait AtomicReadEpImpl: AtomicReadEp  + AsRawTypedFid<Output = EpRawFid> + AtomicValidEp {
    #[allow(clippy::too_many_arguments)]
    fn fetch_atomic_impl<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : Option<*mut T0>) -> Result<(), crate::error::Error>{
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.as_raw_typed_fid(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), res.as_mut_ptr().cast(), res_desc.get_desc().cast(), raw_addr, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), ctx)};
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn fetch_atomicv_impl<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : Option<*mut T0>) -> Result<(), crate::error::Error>{
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.as_raw_typed_fid(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), raw_addr, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), ctx)};
        check_error(err)
    }

    fn fetch_atomicmsg_impl<T: 'static>(&self, msg: &crate::msg::MsgAtomic,  resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], options: AtomicFetchMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicmsg(self.as_raw_typed_fid(), msg.c_msg_atomic, resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), options.get_value()) };
        check_error(err)
    }
}

pub trait AtomicReadEp {
    #[allow(clippy::too_many_arguments)]
    fn fetch_atomic_from<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn fetch_atomic_from_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn fetch_atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn fetch_atomic_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn fetch_atomicv_from<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor],  dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn fetch_atomicv_from_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn fetch_atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor],  mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn fetch_atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> ;
    fn fetch_atomicmsg<T: 'static>(&self, msg: &crate::msg::MsgAtomic,  resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], options: AtomicFetchMsgOptions) -> Result<(), crate::error::Error> ;
}

impl<EP: AtomicReadEpImpl> AtomicReadEp for EP {
    #[allow(clippy::too_many_arguments)]
    fn fetch_atomic_from<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        self.fetch_atomic_impl::<T, ()>(buf, desc, res, res_desc, Some(dest_addr), mem_addr, mapped_key, op, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn fetch_atomic_from_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        self.fetch_atomic_impl(buf, desc, res, res_desc, Some(dest_addr), mem_addr, mapped_key, op, Some(context))
    }

    #[allow(clippy::too_many_arguments)]
    fn fetch_atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        self.fetch_atomic_impl::<T, ()>(buf, desc, res, res_desc, None, mem_addr, mapped_key, op, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn fetch_atomic_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, res: &mut [T], res_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        self.fetch_atomic_impl(buf, desc, res, res_desc, None, mem_addr, mapped_key, op, Some(context))
    }

    #[allow(clippy::too_many_arguments)]
    fn fetch_atomicv_from<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor],  dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        self.fetch_atomicv_impl::<T, ()>(ioc, desc, resultv, res_desc, Some(dest_addr), mem_addr, mapped_key, op, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn fetch_atomicv_from_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        self.fetch_atomicv_impl(ioc, desc, resultv, res_desc, Some(dest_addr), mem_addr, mapped_key, op, Some(context))
    }

    #[allow(clippy::too_many_arguments)]
    fn fetch_atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor],  mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        self.fetch_atomicv_impl::<T, ()>(ioc, desc, resultv, res_desc, None, mem_addr, mapped_key, op, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn fetch_atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error>{
        self.fetch_atomicv_impl(ioc, desc, resultv, res_desc, None, mem_addr, mapped_key, op, Some(context))
    }

    fn fetch_atomicmsg<T: 'static>(&self, msg: &crate::msg::MsgAtomic,  resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], options: AtomicFetchMsgOptions) -> Result<(), crate::error::Error> {
        self.fetch_atomicmsg_impl(msg, resultv, res_desc, options)
    }
}

impl<EP: AtomicCap + ReadMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> AtomicReadEpImpl for EndpointImplBase<EP, EQ, CQ> {}
impl<E: AtomicReadEpImpl> AtomicReadEpImpl for EndpointBase<E> {}


pub(crate) trait AtomicReadWriteEpImpl: AtomicReadWriteEp + AsRawTypedFid<Output = EpRawFid> + AtomicValidEp{
    #[allow(clippy::too_many_arguments)]
    fn compare_atomic_impl<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
        result: &mut [T], result_desc: &mut impl DataDescriptor, dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
        
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.as_raw_typed_fid(), buf.as_ptr().cast(), buf.len(), desc.get_desc().cast(), compare.as_mut_ptr().cast(), 
            compare_desc.get_desc().cast(), result.as_mut_ptr().cast(), result_desc.get_desc().cast(), raw_addr, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), ctx)};
            check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn compare_atomicv_impl<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: Option<&crate::MappedAddress>, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
            
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.as_raw_typed_fid(), ioc.as_ptr().cast(), desc.as_mut_ptr().cast(), ioc.len(), comparetv.as_mut_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparetv.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), raw_addr, mem_addr, mapped_key.get_key(), to_fi_datatype::<T>(), op.get_value(), ctx)};
        check_error(err)
    }
    
    #[allow(clippy::too_many_arguments)]
    fn compare_atomicmsg_impl<T: 'static>(&self, msg: &crate::msg::MsgAtomic, comparev: &[crate::iovec::Ioc<T>], compare_desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], options: AtomicMsgOptions) -> Result<(), crate::error::Error> {
        let err: isize = unsafe { libfabric_sys::inlined_fi_compare_atomicmsg(self.as_raw_typed_fid(), msg.c_msg_atomic, comparev.as_ptr().cast(), compare_desc.as_mut_ptr().cast(), comparev.len(), resultv.as_mut_ptr().cast(), res_desc.as_mut_ptr().cast(), resultv.len(), options.get_value()) };

        check_error(err)
    }
}

pub trait AtomicReadWriteEp {
    #[allow(clippy::too_many_arguments)]
    fn compare_atomic_to<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> ;

    #[allow(clippy::too_many_arguments)]
    fn compare_atomic_to_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> ;

    #[allow(clippy::too_many_arguments)]
    fn compare_atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> ;

    #[allow(clippy::too_many_arguments)]
    fn compare_atomic_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> ;

    #[allow(clippy::too_many_arguments)]
    fn compare_atomicv_to<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor], 
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> ;

    #[allow(clippy::too_many_arguments)]
    fn compare_atomicv_to_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> ;

    #[allow(clippy::too_many_arguments)]
    fn compare_atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor], 
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> ;

    #[allow(clippy::too_many_arguments)]
    fn compare_atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> ;

    #[allow(clippy::too_many_arguments)]
    fn compare_atomicmsg<T: 'static>(&self, msg: &crate::msg::MsgAtomic, comparev: &[crate::iovec::Ioc<T>], compare_desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], options: AtomicMsgOptions) -> Result<(), crate::error::Error> ;
}


impl<EP: AtomicReadWriteEpImpl> AtomicReadWriteEp for EP {
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn compare_atomic_to<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        self.compare_atomic_impl::<T, ()>(buf, desc, compare, compare_desc, result, result_desc, Some(dest_addr), mem_addr, mapped_key, op, None)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn compare_atomic_to_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        self.compare_atomic_impl(buf, desc, compare, compare_desc, result, result_desc, Some(dest_addr), mem_addr, mapped_key, op, Some(context))
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn compare_atomic<T: 'static>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {

        self.compare_atomic_impl::<T, ()>(buf, desc, compare, compare_desc, result, result_desc, None, mem_addr, mapped_key, op, None)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn compare_atomic_with_context<T: 'static, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, compare: &mut [T], compare_desc: &mut impl DataDescriptor, 
            result: &mut [T], result_desc: &mut impl DataDescriptor, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        self.compare_atomic_impl(buf, desc, compare, compare_desc, result, result_desc, None, mem_addr, mapped_key, op, Some(context))
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn compare_atomicv_to<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor], 
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        
        self.compare_atomicv_impl::<T, ()>(ioc, desc, comparetv, compare_desc, resultv, res_desc, Some(dest_addr), mem_addr, mapped_key, op, None)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn compare_atomicv_to_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], dest_addr: &crate::MappedAddress, mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        self.compare_atomicv_impl(ioc, desc, comparetv, compare_desc, resultv, res_desc, Some(dest_addr), mem_addr, mapped_key, op, Some(context))
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn compare_atomicv<T: 'static>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor], 
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op) -> Result<(), crate::error::Error> {

        self.compare_atomicv_impl::<T, ()>(ioc, desc, comparetv, compare_desc, resultv, res_desc, None, mem_addr, mapped_key, op, None)
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn compare_atomicv_with_context<T: 'static, T0>(&self, ioc: &[crate::iovec::Ioc<T>], desc: &mut [impl DataDescriptor], comparetv: &mut [crate::iovec::Ioc<T>],  compare_desc: &mut [impl DataDescriptor],
        resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], mem_addr: u64, mapped_key: &MappedMemoryRegionKey, op: crate::enums::Op, context : &mut T0) -> Result<(), crate::error::Error> {
        
        self.compare_atomicv_impl(ioc, desc, comparetv, compare_desc, resultv, res_desc, None, mem_addr, mapped_key, op, Some(context))
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn compare_atomicmsg<T: 'static>(&self, msg: &crate::msg::MsgAtomic, comparev: &[crate::iovec::Ioc<T>], compare_desc: &mut [impl DataDescriptor], resultv: &mut [crate::iovec::Ioc<T>],  res_desc: &mut [impl DataDescriptor], options: AtomicMsgOptions) -> Result<(), crate::error::Error> {
        self.compare_atomicmsg_impl(msg, comparev, compare_desc, resultv, res_desc, options)
    }
}


impl<EP: AtomicReadEpImpl + AtomicWriteEpImpl> AtomicReadWriteEpImpl for EP {}

pub trait AtomicValidEp: AsRawTypedFid<Output = EpRawFid> {
    fn atomicvalid<T: 'static>(&self, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_atomicvalid(self.as_raw_typed_fid(), to_fi_datatype::<T>(), op.get_value(), &mut count as *mut usize)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }

    fn fetch_atomicvalid<T: 'static>(&self, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_fetch_atomicvalid(self.as_raw_typed_fid(), to_fi_datatype::<T>(), op.get_value(), &mut count as *mut usize)};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }

    fn compare_atomicvalid<T: 'static>(&self, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_compare_atomicvalid(self.as_raw_typed_fid(), to_fi_datatype::<T>(), op.get_value(), &mut count as *mut usize)};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }
}

impl<E: AtomicValidEp> AtomicValidEp for EndpointBase<E> {}
impl<EP: AtomicCap, EQ: ?Sized, CQ: ?Sized + ReadCq> AtomicValidEp for EndpointImplBase<EP, EQ, CQ> {}

impl<CQ: ReadCq> AtomicWriteEpImpl for TransmitContextBase<CQ> {}
impl<CQ: ReadCq> AtomicWriteEpImpl for TransmitContextImplBase<CQ> {}
impl<CQ: ReadCq> AtomicReadEpImpl for ReceiveContextBase<CQ> {}
impl<CQ: ReadCq> AtomicReadEpImpl for ReceiveContextImplBase<CQ> {}
impl<CQ: ReadCq> AtomicValidEp for TransmitContextBase<CQ> {}
impl<CQ: ReadCq> AtomicValidEp for TransmitContextImplBase<CQ> {}
impl<CQ: ReadCq> AtomicValidEp for ReceiveContextBase<CQ> {}
impl<CQ: ReadCq> AtomicValidEp for ReceiveContextImplBase<CQ> {}

pub struct AtomicAttr {
    pub(crate) c_attr : libfabric_sys::fi_atomic_attr,
}

impl AtomicAttr {
    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_atomic_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_atomic_attr {
        &mut self.c_attr
    }
}