use crate::FI_ADDR_UNSPEC;
use crate::MappedAddressBase;
use crate::enums::TaggedRecvMsgOptions;
use crate::enums::TaggedSendMsgOptions;
use crate::ep::ActiveEndpointImpl;
use crate::ep::Endpoint;
use crate::ep::EndpointBase;
use crate::eq::EventQueueImplT;
use crate::infocapsoptions::RecvMod;
use crate::infocapsoptions::SendMod;
use crate::infocapsoptions::TagCap;
use crate::mr::DataDescriptor;
use crate::utils::check_error;
use crate::xcontext::ReceiveContext;
use crate::xcontext::TransmitContext;

use super::message::extract_raw_addr_and_ctx;




impl<E: TagCap + RecvMod, EQ: EventQueueImplT, CQ> EndpointBase<E, EQ, CQ> {

    fn trecv_impl<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddressBase<EQ>>, tag: u64, ignore:u64, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), raw_addr, tag, ignore, ctx) };
        check_error(err)
    }

    pub fn trecv<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddressBase<EQ>, tag: u64, ignore:u64) -> Result<(), crate::error::Error> {
        self.trecv_impl::<T,()>(buf, desc, Some(mapped_addr), tag, ignore, None)
    }

    pub fn trecv_connected<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64) -> Result<(), crate::error::Error> {
        self.trecv_impl::<T,()>(buf, desc, None, tag, ignore, None)
    }
    
    pub fn trecv_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddressBase<EQ>, tag: u64, ignore:u64, context: &mut T0) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, Some(mapped_addr), tag, ignore, Some(context))
    }
    
    pub fn trecv_connected_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64, context: &mut T0) -> Result<(), crate::error::Error> {
        self.trecv_impl(buf, desc, None, tag, ignore, Some(context))
    }

    fn trecvv_impl<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], src_mapped_addr: Option<&MappedAddressBase<EQ>>, tag: u64, ignore:u64, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(src_mapped_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_trecvv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), raw_addr, tag, ignore, ctx) };
        check_error(err) 
    }

	pub fn trecvv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], src_mapped_addr: &MappedAddressBase<EQ>, tag: u64, ignore:u64) -> Result<(), crate::error::Error> {
        self.trecvv_impl::<T, ()>(iov, desc, Some(src_mapped_addr), tag, ignore, None)
    }

	pub fn trecvv_connected<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64) -> Result<(), crate::error::Error> { //[TODO]
        self.trecvv_impl::<T, ()>(iov, desc, None, tag, ignore, None)
    }

	pub fn trecvv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], src_mapped_addr: &MappedAddressBase<EQ>, tag: u64, ignore:u64, context : &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        self.trecvv_impl(iov, desc, Some(src_mapped_addr), tag, ignore, Some(context))
    }

	pub fn trecvv_connected_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64, context : &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        self.trecvv_impl(iov, desc, None, tag, ignore, Some(context))
    }

    pub fn trecvmsg(&self, msg: &crate::msg::MsgTagged, options: TaggedRecvMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_trecvmsg(self.handle(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, options.get_value()) };
        check_error(err)
    }
}

impl<E: TagCap + SendMod, EQ: EventQueueImplT, CQ> EndpointBase<E, EQ, CQ> {

    fn tsend_impl<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddressBase<EQ>>, tag:u64, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), raw_addr, tag, ctx) };
        check_error(err) 
    }

    pub fn tsend<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddressBase<EQ>, tag:u64) -> Result<(), crate::error::Error> {
        self.tsend_impl::<T,()>(buf, desc, Some(mapped_addr), tag, None)
    }

    pub fn tsend_connected<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64) -> Result<(), crate::error::Error> {
        self.tsend_impl::<T,()>(buf, desc, None, tag, None)
    }

    pub fn tsend_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddressBase<EQ>, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> {
        self.tsend_impl(buf, desc, Some(mapped_addr), tag, Some(context))
    }

    pub fn tsend_connected_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> {
        self.tsend_impl(buf, desc, None, tag, Some(context))
    }
    
    fn tsendv_impl<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: Option<&MappedAddressBase<EQ>>, tag:u64, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_mapped_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, tag, ctx) };
        check_error(err) 
    }

	pub fn tsendv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &MappedAddressBase<EQ>, tag:u64) -> Result<(), crate::error::Error> { // [TODO]
        self.tsendv_impl::<T,()>(iov, desc, Some(dest_mapped_addr), tag, None)
    }
    
	pub fn tsendv_connected<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], tag:u64) -> Result<(), crate::error::Error> { // [TODO]
        self.tsendv_impl::<T,()>(iov, desc, None, tag, None)
    }

	pub fn tsendv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &MappedAddressBase<EQ>, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        self.tsendv_impl(iov, desc, Some(dest_mapped_addr), tag, Some(context))
    }

	pub fn tsendv_connected_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        self.tsendv_impl(iov, desc, None, tag, Some(context))
    }

    pub fn tsendmsg(&self, msg: &crate::msg::MsgTagged, options: TaggedSendMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsendmsg(self.handle(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, options.get_value()) };
        check_error(err)
    }

    fn tsenddata_impl<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: Option<&MappedAddressBase<EQ>>, tag: u64, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
        let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, raw_addr, tag, ctx) };
        check_error(err) 
    }

    pub fn tsenddata<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddressBase<EQ>, tag: u64) -> Result<(), crate::error::Error> {
        self.tsenddata_impl::<T,()>(buf, desc, data, Some(mapped_addr), tag, None)
    }

    pub fn tsenddata_connected<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64) -> Result<(), crate::error::Error> {
        self.tsenddata_impl::<T,()>(buf, desc, data, None, tag, None)
    }

    pub fn tsenddata_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddressBase<EQ>, tag: u64, context : &mut T0) -> Result<(), crate::error::Error> {
        self.tsenddata_impl(buf, desc, data, Some(mapped_addr), tag, Some(context))
    }

    pub fn tsenddata_connected_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64, context : &mut T0) -> Result<(), crate::error::Error> {
        self.tsenddata_impl(buf, desc, data, None, tag, Some(context))
    }

    pub fn tinject<T>(&self, buf: &[T], mapped_addr: &MappedAddressBase<EQ>, tag:u64 ) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tinject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), mapped_addr.raw_addr(), tag) };
        check_error(err)
    }

    pub fn tinject_connected<T>(&self, buf: &[T], tag:u64 ) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tinject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), FI_ADDR_UNSPEC, tag) };
        check_error(err)
    }

    pub fn tinjectdata<T>(&self, buf: &[T], data: u64, mapped_addr: &MappedAddressBase<EQ>, tag: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tinjectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, mapped_addr.raw_addr(), tag) };
        check_error(err)
    }

    pub fn tinjectdata_connected<T>(&self, buf: &[T], data: u64, tag: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tinjectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, FI_ADDR_UNSPEC, tag) };
        check_error(err)
    }
}


// impl TransmitContext {

//     fn tsend_impl<T, T0, EQ>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddressBase<EQ>>, tag:u64, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
//         let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
//         let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), raw_addr, tag, ctx) };
//         check_error(err) 
//     }

//     pub fn tsend<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddressBase<EQ>, tag:u64) -> Result<(), crate::error::Error> {
//         self.tsend_impl::<T,()>(buf, desc, Some(mapped_addr), tag, None)
//     }

//     pub fn tsend_connected<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64) -> Result<(), crate::error::Error> {
//         self.tsend_impl::<T,()>(buf, desc, None, tag, None)
//     }

//     pub fn tsend_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddressBase<EQ>, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> {
//         self.tsend_impl(buf, desc, Some(mapped_addr), tag, Some(context))
//     }

//     pub fn tsend_connected_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> {
//         self.tsend_impl(buf, desc, None, tag, Some(context))
//     }
    
//     fn tsendv_impl<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: Option<&MappedAddressBase<EQ>>, tag:u64, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
//         let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_mapped_addr, context);
//         let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), raw_addr, tag, ctx) };
//         check_error(err) 
//     }

// 	pub fn tsendv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &MappedAddressBase<EQ>, tag:u64) -> Result<(), crate::error::Error> { // [TODO]
//         self.tsendv_impl::<T,()>(iov, desc, Some(dest_mapped_addr), tag, None)
//     }
    
// 	pub fn tsendv_connected<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], tag:u64) -> Result<(), crate::error::Error> { // [TODO]
//         self.tsendv_impl::<T,()>(iov, desc, None, tag, None)
//     }

// 	pub fn tsendv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], dest_mapped_addr: &MappedAddressBase<EQ>, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
//         self.tsendv_impl(iov, desc, Some(dest_mapped_addr), tag, Some(context))
//     }

// 	pub fn tsendv_connected_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
//         self.tsendv_impl(iov, desc, None, tag, Some(context))
//     }

//     pub fn tsendmsg(&self, msg: &crate::msg::MsgTagged, options: TaggedSendMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tsendmsg(self.handle(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, options.get_value()) };
//         check_error(err)
//     }

//     fn tsenddata_impl<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: Option<&MappedAddressBase<EQ>>, tag: u64, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
//         let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
//         let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, raw_addr, tag, ctx) };
//         check_error(err) 
//     }

//     pub fn tsenddata<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddressBase<EQ>, tag: u64) -> Result<(), crate::error::Error> {
//         self.tsenddata_impl::<T,()>(buf, desc, data, Some(mapped_addr), tag, None)
//     }

//     pub fn tsenddata_connected<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64) -> Result<(), crate::error::Error> {
//         self.tsenddata_impl::<T,()>(buf, desc, data, None, tag, None)
//     }

//     pub fn tsenddata_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &MappedAddressBase<EQ>, tag: u64, context : &mut T0) -> Result<(), crate::error::Error> {
//         self.tsenddata_impl(buf, desc, data, Some(mapped_addr), tag, Some(context))
//     }

//     pub fn tsenddata_connected_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, tag: u64, context : &mut T0) -> Result<(), crate::error::Error> {
//         self.tsenddata_impl(buf, desc, data, None, tag, Some(context))
//     }

//     pub fn tinject<T>(&self, buf: &[T], mapped_addr: &MappedAddressBase<EQ>, tag:u64 ) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tinject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), mapped_addr.raw_addr(), tag) };
//         check_error(err)
//     }

//     pub fn tinject_connected<T>(&self, buf: &[T], tag:u64 ) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tinject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), FI_ADDR_UNSPEC, tag) };
//         check_error(err)
//     }

//     pub fn tinjectdata<T>(&self, buf: &[T], data: u64, mapped_addr: &MappedAddressBase<EQ>, tag: u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tinjectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, mapped_addr.raw_addr(), tag) };
//         check_error(err)
//     }

//     pub fn tinjectdata_connected<T>(&self, buf: &[T], data: u64, tag: u64) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_tinjectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, FI_ADDR_UNSPEC, tag) };
//         check_error(err)
//     }
// }

// impl ReceiveContext {

//     fn trecv_impl<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: Option<&MappedAddressBase<EQ>>, tag: u64, ignore:u64, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
//         let (raw_addr, ctx) = extract_raw_addr_and_ctx(mapped_addr, context);
//         let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), raw_addr, tag, ignore, ctx) };
//         check_error(err)
//     }

//     pub fn trecv<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddressBase<EQ>, tag: u64, ignore:u64) -> Result<(), crate::error::Error> {
//         self.trecv_impl::<T,()>(buf, desc, Some(mapped_addr), tag, ignore, None)
//     }

//     pub fn trecv_connected<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64) -> Result<(), crate::error::Error> {
//         self.trecv_impl::<T,()>(buf, desc, None, tag, ignore, None)
//     }
    
//     pub fn trecv_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &MappedAddressBase<EQ>, tag: u64, ignore:u64, context: &mut T0) -> Result<(), crate::error::Error> {
//         self.trecv_impl(buf, desc, Some(mapped_addr), tag, ignore, Some(context))
//     }
    
//     pub fn trecv_connected_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, tag: u64, ignore:u64, context: &mut T0) -> Result<(), crate::error::Error> {
//         self.trecv_impl(buf, desc, None, tag, ignore, Some(context))
//     }

//     fn trecvv_impl<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], src_mapped_addr: Option<&MappedAddressBase<EQ>>, tag: u64, ignore:u64, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
//         let (raw_addr, ctx) = extract_raw_addr_and_ctx(src_mapped_addr, context);
//         let err = unsafe{ libfabric_sys::inlined_fi_trecvv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), raw_addr, tag, ignore, ctx) };
//         check_error(err) 
//     }

// 	pub fn trecvv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], src_mapped_addr: &MappedAddressBase<EQ>, tag: u64, ignore:u64) -> Result<(), crate::error::Error> {
//         self.trecvv_impl::<T, ()>(iov, desc, Some(src_mapped_addr), tag, ignore, None)
//     }

// 	pub fn trecvv_connected<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64) -> Result<(), crate::error::Error> { //[TODO]
//         self.trecvv_impl::<T, ()>(iov, desc, None, tag, ignore, None)
//     }

// 	pub fn trecvv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], src_mapped_addr: &MappedAddressBase<EQ>, tag: u64, ignore:u64, context : &mut T0) -> Result<(), crate::error::Error> { //[TODO]
//         self.trecvv_impl(iov, desc, Some(src_mapped_addr), tag, ignore, Some(context))
//     }

// 	pub fn trecvv_connected_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], tag: u64, ignore:u64, context : &mut T0) -> Result<(), crate::error::Error> { //[TODO]
//         self.trecvv_impl(iov, desc, None, tag, ignore, Some(context))
//     }

//     pub fn trecvmsg(&self, msg: &crate::msg::MsgTagged, options: TaggedRecvMsgOptions) -> Result<(), crate::error::Error> {
//         let err = unsafe{ libfabric_sys::inlined_fi_trecvmsg(self.handle(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, options.get_value()) };
//         check_error(err)
//     }
// }
