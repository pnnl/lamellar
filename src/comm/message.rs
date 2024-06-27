
use crate::{FI_ADDR_UNSPEC, enums::{RecvMsgOptions, SendMsgOptions}, ep::EndpointBase, infocapsoptions::{MsgCap, RecvMod, SendMod}, mr::DataDescriptor, utils::check_error, MappedAddressBase, eq::EventQueueImplT, fid::AsRawTypedFid};
pub(crate) fn extract_raw_addr_and_ctx<T0, EQ: EventQueueImplT>(mapped_addr: Option<&MappedAddressBase<EQ>>, context: Option<*mut T0>) -> (u64, *mut std::ffi::c_void) {
            
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

impl<E: MsgCap + RecvMod, EQ: EventQueueImplT, CQ> EndpointBase<E, EQ, CQ> {

    fn recv_impl<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: Option<&crate::MappedAddressBase<EQ>>, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);

        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, ctx) };
        check_error(err)
    }

    pub fn recv<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddressBase<EQ>) -> Result<(), crate::error::Error> {
        self.recv_impl::<T,()>(buf, desc, Some(mapped_addr), None)
    }

    pub fn recv_connected<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor) -> Result<(), crate::error::Error> {
        self.recv_impl::<T,()>(buf, desc, None, None)
    }
    
    pub fn recv_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddressBase<EQ>, context: &mut T0) -> Result<(), crate::error::Error> {
        self.recv_impl(buf, desc, Some(mapped_addr), Some(context))
    }
    
    pub fn recv_connected_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, context: &mut T0) -> Result<(), crate::error::Error> {
        self.recv_impl(buf, desc, None, Some(context))
    }
    
    fn recvv_impl<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&crate::MappedAddressBase<EQ>>,  context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, ctx) };
        check_error(err)    
    }

	pub fn recvv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddressBase<EQ>) -> Result<(), crate::error::Error> {
        self.recvv_impl::<T, ()>(iov, desc, Some(mapped_addr), None)
    }

	pub fn recvv_connected<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor]) -> Result<(), crate::error::Error> {
        self.recvv_impl::<T, ()>(iov, desc, None, None)
    }
    
	pub fn recvv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddressBase<EQ>,  context: &mut T0) -> Result<(), crate::error::Error> {
        self.recvv_impl(iov, desc, Some(mapped_addr), Some(context))
    }
    
	pub fn recvv_connected_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], context: &mut T0) -> Result<(), crate::error::Error> {
        self.recvv_impl(iov, desc, None, Some(context))
    }

    pub fn recvmsg(&self, msg: &crate::msg::Msg, options: RecvMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.as_raw_typed_fid(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
        check_error(err)
    }
}

impl<E: MsgCap + SendMod, EQ: EventQueueImplT, CQ> EndpointBase<E, EQ, CQ> {

    fn sendv_impl<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&crate::MappedAddressBase<EQ>>, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.as_raw_typed_fid(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), raw_addr, ctx) };
        check_error(err)
    }

	pub fn sendv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddressBase<EQ>) -> Result<(), crate::error::Error> {
        self.sendv_impl::<T, ()>(iov, desc, Some(mapped_addr), None)
    }

	pub fn sendv_connected<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor]) -> Result<(), crate::error::Error> { 
        self.sendv_impl::<T,()>(iov, desc, None, None)
    }
    
	pub fn sendv_with_context<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddressBase<EQ>, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        self.sendv_impl(iov, desc, Some(mapped_addr), Some(context))
    }
    
	pub fn sendv_connected_with_context<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        self.sendv_impl(iov, desc, None, Some(context))
    }

    fn send_impl<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: Option<&crate::MappedAddressBase<EQ>>, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.as_raw_typed_fid(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, ctx) };
        check_error(err)
    }

    pub fn send<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddressBase<EQ>) -> Result<(), crate::error::Error> {
        self.send_impl::<T, ()>(buf, desc, Some(mapped_addr), None)
    }

    pub fn send_connected<T>(&self, buf: &[T], desc: &mut impl DataDescriptor) -> Result<(), crate::error::Error> {
        self.send_impl::<T,()>(buf, desc, None, None)
    }

    pub fn send_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddressBase<EQ>, context : &mut T0) -> Result<(), crate::error::Error> {
        self.send_impl(buf, desc, Some(mapped_addr), Some(context))
    }

    pub fn send_connected_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, context : &mut T0) -> Result<(), crate::error::Error> {
        self.send_impl(buf, desc, None, Some(context))
    }

    pub fn sendmsg(&self, msg: &crate::msg::Msg, options: SendMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.as_raw_typed_fid(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
        check_error(err)
    }

    fn senddata_impl<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: Option<&crate::MappedAddressBase<EQ>>, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, raw_addr, ctx) };
        check_error(err)
    }

    pub fn senddata<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddressBase<EQ>) -> Result<(), crate::error::Error> {
        self.senddata_impl::<T,()>(buf, desc, data, Some(mapped_addr), None)
    }

    pub fn senddata_connected<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64) -> Result<(), crate::error::Error> {
        self.senddata_impl::<T,()>(buf, desc, data, None, None)
    }

    pub fn senddata_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddressBase<EQ>, context : &mut T0) -> Result<(), crate::error::Error> {
        self.senddata_impl(buf, desc, data, Some(mapped_addr), Some(context))
    }

    pub fn senddata_connected_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, context : &mut T0) -> Result<(), crate::error::Error> {
        self.senddata_impl(buf, desc, data, None, Some(context))
    }

    pub fn inject_connected<T>(&self, buf: &[T]) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), FI_ADDR_UNSPEC) };
        check_error(err)
    }

    pub fn inject<T>(&self, buf: &[T], mapped_addr: &crate::MappedAddressBase<EQ>) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), mapped_addr.raw_addr()) };
        check_error(err)
    }

    pub fn injectdata<T>(&self, buf: &[T], data: u64, mapped_addr: &crate::MappedAddressBase<EQ>) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, mapped_addr.raw_addr()) };
        check_error(err)
    }

    pub fn injectdata_connected<T>(&self, buf: &[T], data: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, FI_ADDR_UNSPEC) };
        check_error(err)
    }
}

// impl TransmitContext {

//     fn sendv_impl<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&crate::MappedAddressBase<EQ>>, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
//         let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
//         let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.as_raw_typed_fid(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), raw_addr, ctx) };
//         check_error(err)
//     }

// 	pub fn sendv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddressBase<EQ>) -> Result<(), crate::error::Error> {
//         self.sendv_impl::<T, ()>(iov, desc, Some(mapped_addr), None)
//     }

// 	pub fn sendv_connected<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor]) -> Result<(), crate::error::Error> { 
//         self.sendv_impl::<T,()>(iov, desc, None, None)
//     }
    
// 	pub fn sendv_with_context<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddressBase<EQ>, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
//         self.sendv_impl(iov, desc, Some(mapped_addr), Some(context))
//     }
    
// 	pub fn sendv_connected_with_context<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
//         self.sendv_impl(iov, desc, None, Some(context))
//     }

//     fn send_impl<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: Option<&crate::MappedAddressBase<EQ>>, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
//         let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
//         let err = unsafe{ libfabric_sys::inlined_fi_send(self.as_raw_typed_fid(), buf.as_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, ctx) };
//         check_error(err)
//     }

//     pub fn send<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddressBase<EQ>) -> Result<(), crate::error::Error> {
//         self.send_impl::<T, ()>(buf, desc, Some(mapped_addr), None)
//     }

//     pub fn send_connected<T>(&self, buf: &[T], desc: &mut impl DataDescriptor) -> Result<(), crate::error::Error> {
//         self.send_impl::<T,()>(buf, desc, None, None)
//     }

//     pub fn send_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddressBase<EQ>, context : &mut T0) -> Result<(), crate::error::Error> {
//         self.send_impl(buf, desc, Some(mapped_addr), Some(context))
//     }

//     pub fn send_connected_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, context : &mut T0) -> Result<(), crate::error::Error> {
//         self.send_impl(buf, desc, None, Some(context))
//     }

//     pub fn sendmsg(&self, msg: &crate::msg::Msg, options: SendMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.as_raw_typed_fid(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
//         check_error(err)
//     }

//     fn senddata_impl<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: Option<&crate::MappedAddressBase<EQ>>, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
//         let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
//         let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, raw_addr, ctx) };
//         check_error(err)
//     }

//     pub fn senddata<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddressBase<EQ>) -> Result<(), crate::error::Error> {
//         self.senddata_impl::<T,()>(buf, desc, data, Some(mapped_addr), None)
//     }

//     pub fn senddata_connected<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64) -> Result<(), crate::error::Error> {
//         self.senddata_impl::<T,()>(buf, desc, data, None, None)
//     }

//     pub fn senddata_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddressBase<EQ>, context : &mut T0) -> Result<(), crate::error::Error> {
//         self.senddata_impl(buf, desc, data, Some(mapped_addr), Some(context))
//     }

//     pub fn senddata_connected_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, context : &mut T0) -> Result<(), crate::error::Error> {
//         self.senddata_impl(buf, desc, data, None, Some(context))
//     }

//     pub fn inject_connected<T>(&self, buf: &[T]) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_inject(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), FI_ADDR_UNSPEC) };
//         check_error(err)
//     }

//     pub fn inject<T>(&self, buf: &[T], mapped_addr: &crate::MappedAddressBase<EQ>) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_inject(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), mapped_addr.raw_addr()) };
//         check_error(err)
//     }

//     pub fn injectdata<T>(&self, buf: &[T], data: u64, mapped_addr: &crate::MappedAddressBase<EQ>) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, mapped_addr.raw_addr()) };
//         check_error(err)
//     }

//     pub fn injectdata_connected<T>(&self, buf: &[T], data: u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.as_raw_typed_fid(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, FI_ADDR_UNSPEC) };
//         check_error(err)
//     }
// }

// impl ReceiveContext {

//     fn recv_impl<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: Option<&crate::MappedAddressBase<EQ>>, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        
//         let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);

//         let err = unsafe{ libfabric_sys::inlined_fi_recv(self.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), raw_addr, ctx) };
//         check_error(err)
//     }

//     pub fn recv<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddressBase<EQ>) -> Result<(), crate::error::Error> {
//         self.recv_impl::<T,()>(buf, desc, Some(mapped_addr), None)
//     }

//     pub fn recv_connected<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor) -> Result<(), crate::error::Error> {
//         self.recv_impl::<T,()>(buf, desc, None, None)
//     }
    
//     pub fn recv_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddressBase<EQ>, context: &mut T0) -> Result<(), crate::error::Error> {
//         self.recv_impl(buf, desc, Some(mapped_addr), Some(context))
//     }
    
//     pub fn recv_connected_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, context: &mut T0) -> Result<(), crate::error::Error> {
//         self.recv_impl(buf, desc, None, Some(context))
//     }
    
//     fn recvv_impl<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: Option<&crate::MappedAddressBase<EQ>>,  context: Option<*mut T0>) -> Result<(), crate::error::Error> {
//         let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
//         let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.as_raw_typed_fid(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, ctx) };
//         check_error(err)    
//     }

// 	pub fn recvv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddressBase<EQ>) -> Result<(), crate::error::Error> {
//         self.recvv_impl::<T, ()>(iov, desc, Some(mapped_addr), None)
//     }

// 	pub fn recvv_connected<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor]) -> Result<(), crate::error::Error> {
//         self.recvv_impl::<T, ()>(iov, desc, None, None)
//     }
    
// 	pub fn recvv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddressBase<EQ>,  context: &mut T0) -> Result<(), crate::error::Error> {
//         self.recvv_impl(iov, desc, Some(mapped_addr), Some(context))
//     }
    
// 	pub fn recvv_connected_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], context: &mut T0) -> Result<(), crate::error::Error> {
//         self.recvv_impl(iov, desc, None, Some(context))
//     }

//     pub fn recvmsg(&self, msg: &crate::msg::Msg, options: RecvMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.as_raw_typed_fid(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
//         check_error(err)
//     }
// }