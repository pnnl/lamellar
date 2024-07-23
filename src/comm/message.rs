
use crate::{FI_ADDR_UNSPEC, enums::{RecvMsgOptions, SendMsgOptions}, ep::{EndpointBase, EndpointImplBase}, mr::DataDescriptor, utils::check_error, MappedAddress, eq::ReadEq, fid::{AsRawTypedFid, EpRawFid}, cq::ReadCq, xcontext::{TransmitContext, ReceiveContext, TransmitContextImpl, ReceiveContextImpl}, infocapsoptions::{MsgCap, RecvMod, SendMod}};

pub(crate) fn extract_raw_ctx<T0>(context: Option<*mut T0>) -> *mut std::ffi::c_void {
    if let Some(ctx) = context {
        ctx.cast()
    }
    else {
        std::ptr::null_mut()
    } 
}

pub(crate) fn extract_raw_addr_and_ctx<T0>(mapped_addr: Option<&MappedAddress>, context: Option<*mut T0>) -> (u64, *mut std::ffi::c_void) {
            
    let ctx = if let Some(ctx) = context {
        ctx.cast()
    }
    else {
        std::ptr::null_mut()
    };

    let raw_addr = if let Some(addr) = mapped_addr {
        addr.raw_addr()
    }
    else {
        FI_ADDR_UNSPEC
    };

    (raw_addr, ctx)
}

pub(crate) trait RecvEpImpl: RecvEp + AsRawTypedFid<Output = EpRawFid> {
    fn recv_impl<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: Option<&crate::MappedAddress>, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, ctx) };
        check_error(err)
    }

    fn recvv_impl<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&crate::MappedAddress>,  context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, ctx) };
        check_error(err)    
    }

    
    fn recvmsg_impl(&self, msg: &crate::msg::Msg, options: RecvMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.as_raw_typed_fid(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
        check_error(err)
    }
}


pub trait RecvEp {
    fn recv_from<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> ;
    fn recv<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor) -> Result<(), crate::error::Error> ;  
    fn recv_from_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, context: &mut T0) -> Result<(), crate::error::Error> ;
    fn recv_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, context: &mut T0) -> Result<(), crate::error::Error> ;    
	fn recvv_from<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> ;
	fn recvv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor]) -> Result<(), crate::error::Error> ;
	fn recvv_from_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress,  context: &mut T0) -> Result<(), crate::error::Error> ;
	fn recvv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], context: &mut T0) -> Result<(), crate::error::Error> ;
    fn recvmsg(&self, msg: &crate::msg::Msg, options: RecvMsgOptions) -> Result<(), crate::error::Error> ;
}

impl<EP: RecvEpImpl> RecvEp for EP {
    #[inline]
    fn recv_from<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        self.recv_impl::<T,()>(buf, desc, Some(mapped_addr), None)
    }

    #[inline]
    fn recv<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor) -> Result<(), crate::error::Error> {
        self.recv_impl::<T,()>(buf, desc, None, None)
    }
    
    #[inline]
    fn recv_from_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, context: &mut T0) -> Result<(), crate::error::Error> {
        self.recv_impl(buf, desc, Some(mapped_addr), Some(context))
    }
    
    #[inline]
    fn recv_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, context: &mut T0) -> Result<(), crate::error::Error> {
        self.recv_impl(buf, desc, None, Some(context))
    }

    #[inline]
	fn recvv_from<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        self.recvv_impl::<T, ()>(iov, desc, Some(mapped_addr), None)
    }

    #[inline]
	fn recvv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor]) -> Result<(), crate::error::Error> {
        self.recvv_impl::<T, ()>(iov, desc, None, None)
    }
    
    #[inline]
	fn recvv_from_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress,  context: &mut T0) -> Result<(), crate::error::Error> {
        self.recvv_impl(iov, desc, Some(mapped_addr), Some(context))
    }
    
    #[inline]
	fn recvv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], context: &mut T0) -> Result<(), crate::error::Error> {
        self.recvv_impl(iov, desc, None, Some(context))
    }

    #[inline]
    fn recvmsg(&self, msg: &crate::msg::Msg, options: RecvMsgOptions) -> Result<(), crate::error::Error> {
        self.recvmsg_impl(msg, options)
    }
}

impl<EP: MsgCap + RecvMod, EQ: ?Sized, CQ: ?Sized + ReadCq> RecvEpImpl for EndpointImplBase<EP, EQ, CQ> {}

// impl<E: MsgCap + RecvMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> EndpointBase<E, EQ, CQ> {
impl<E: MsgCap + RecvMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> RecvEpImpl for EndpointBase<E, EQ, CQ> {}

pub(crate) trait SendEpImpl: SendEp + AsRawTypedFid<Output = EpRawFid> {
    fn sendv_impl<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&crate::MappedAddress>, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.as_raw_typed_fid(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), raw_addr, ctx) };
        check_error(err)
    }

    fn send_impl<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: Option<&crate::MappedAddress>, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.as_raw_typed_fid(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, ctx) };
        check_error(err)
    }

    fn sendmsg_impl(&self, msg: &crate::msg::Msg, options: SendMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.as_raw_typed_fid(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
        check_error(err)
    }

    fn senddata_impl<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: Option<&crate::MappedAddress>, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, raw_addr, ctx) };
        check_error(err)
    }

    fn inject_impl<T>(&self, buf: &[T], mapped_addr: Option<&crate::MappedAddress>) -> Result<(), crate::error::Error> {
        let raw_addr = if let Some(addr) = mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };
        let err = unsafe{ libfabric_sys::inlined_fi_inject(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), raw_addr) };
        check_error(err)
    }

    fn injectdata_impl<T>(&self, buf: &[T], data: u64, mapped_addr: Option<&crate::MappedAddress>) -> Result<(), crate::error::Error> {
        let raw_addr = if let Some(addr) = mapped_addr {
            addr.raw_addr()
        }
        else {
            FI_ADDR_UNSPEC
        };
        let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, raw_addr) };
        check_error(err)
    }
}

pub trait SendEp {
	fn sendv_to<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> ;
	fn sendv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor]) -> Result<(), crate::error::Error> ;
	fn sendv_to_with_context<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> ;
	fn sendv_with_context<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], context : &mut T0) -> Result<(), crate::error::Error> ;
    fn send_to<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> ;
    fn send<T>(&self, buf: &[T], desc: &mut impl DataDescriptor) -> Result<(), crate::error::Error> ;
    fn send_to_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> ;
    fn send_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, context : &mut T0) -> Result<(), crate::error::Error> ;
    fn sendmsg(&self, msg: &crate::msg::Msg, options: SendMsgOptions) -> Result<(), crate::error::Error> ;
    fn senddata_to<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> ;
    fn senddata<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64) -> Result<(), crate::error::Error> ;
    fn senddata_to_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> ;
    fn senddata_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, context : &mut T0) -> Result<(), crate::error::Error> ;
    fn inject<T>(&self, buf: &[T]) -> Result<(), crate::error::Error> ;
    fn inject_to<T>(&self, buf: &[T], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> ;
    fn injectdata_to<T>(&self, buf: &[T], data: u64, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> ;
    fn injectdata<T>(&self, buf: &[T], data: u64) -> Result<(), crate::error::Error> ;
}

impl<EP: SendEpImpl> SendEp for EP {
    #[inline]
    fn sendv_to<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        self.sendv_impl::<T, ()>(iov, desc, Some(mapped_addr), None)
    }

    #[inline]
	fn sendv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor]) -> Result<(), crate::error::Error> { 
        self.sendv_impl::<T,()>(iov, desc, None, None)
    }
    
    #[inline]
	fn sendv_to_with_context<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        self.sendv_impl(iov, desc, Some(mapped_addr), Some(context))
    }
    
    #[inline]
	fn sendv_with_context<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        self.sendv_impl(iov, desc, None, Some(context))
    }

    #[inline]
    fn send_to<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        self.send_impl::<T, ()>(buf, desc, Some(mapped_addr), None)
    }

    #[inline]
    fn send<T>(&self, buf: &[T], desc: &mut impl DataDescriptor) -> Result<(), crate::error::Error> {
        self.send_impl::<T,()>(buf, desc, None, None)
    }

    #[inline]
    fn send_to_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> {
        self.send_impl(buf, desc, Some(mapped_addr), Some(context))
    }

    #[inline]
    fn send_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, context : &mut T0) -> Result<(), crate::error::Error> {
        self.send_impl(buf, desc, None, Some(context))
    }

    #[inline]
    fn sendmsg(&self, msg: &crate::msg::Msg, options: SendMsgOptions) -> Result<(), crate::error::Error> {
        self.sendmsg_impl(msg, options)
    }

    #[inline]
    fn senddata_to<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        self.senddata_impl::<T,()>(buf, desc, data, Some(mapped_addr), None)
    }

    #[inline]
    fn senddata<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64) -> Result<(), crate::error::Error> {
        self.senddata_impl::<T,()>(buf, desc, data, None, None)
    }

    #[inline]
    fn senddata_to_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> {
        self.senddata_impl(buf, desc, data, Some(mapped_addr), Some(context))
    }

    #[inline]
    fn senddata_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, context : &mut T0) -> Result<(), crate::error::Error> {
        self.senddata_impl(buf, desc, data, None, Some(context))
    }

    #[inline]
    fn inject_to<T>(&self, buf: &[T], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        self.inject_impl(buf, Some(mapped_addr))
    }

    #[inline]
    fn inject<T>(&self, buf: &[T]) -> Result<(), crate::error::Error> {
        self.inject_impl(buf, None)
    }

    #[inline]
    fn injectdata_to<T>(&self, buf: &[T], data: u64, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        self.injectdata_impl(buf, data, Some(mapped_addr))
    }

    #[inline]
    fn injectdata<T>(&self, buf: &[T], data: u64) -> Result<(), crate::error::Error> {
        self.injectdata_impl(buf, data, None)
    }
}

// impl<E: MsgCap + SendMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> EndpointBase<E, EQ, CQ> {
impl<EP: MsgCap + SendMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> SendEpImpl for EndpointImplBase<EP, EQ, CQ> {}
impl<E: MsgCap + SendMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> SendEpImpl for EndpointBase<E, EQ, CQ> {}

impl SendEpImpl for TransmitContext{}
impl SendEpImpl for TransmitContextImpl{}
impl RecvEpImpl for ReceiveContext{}
impl RecvEpImpl for ReceiveContextImpl{}