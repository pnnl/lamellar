use crate::enums::RecvMsgOptions;
use crate::enums::SendMsgOptions;
use crate::ep::ActiveEndpointImpl;
use crate::ep::Endpoint;
use crate::infocapsoptions::MsgCap;
use crate::infocapsoptions::RecvMod;
use crate::infocapsoptions::SendMod;
use crate::mr::DataDescriptor;
use crate::utils::check_error;
use crate::xcontext::ReceiveContext;
use crate::xcontext::TransmitContext;



impl<E: MsgCap + RecvMod> Endpoint<E> {

    pub fn recv<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), std::ptr::null_mut()) };

        check_error(err)
    }

    pub fn recv_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), (context as *mut T0).cast() ) };
    
        check_error(err)
    }

	pub fn recvv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), std::ptr::null_mut()) };
        check_error(err)
    }
    
	pub fn recvv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress,  context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), (context as *mut T0).cast()) };
        check_error(err)
    }

    pub fn recvmsg(&self, msg: &crate::msg::Msg, options: RecvMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
        
        check_error(err)
    }
}

impl<E: MsgCap + SendMod> Endpoint<E> {

	pub fn sendv<T>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> { 
        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), std::ptr::null_mut()) };
        
        check_error(err)
    }
    
	pub fn sendv_with_context<T,T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), (context as *mut T0).cast()) };
        
        check_error(err)
    }

    pub fn send<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), std::ptr::null_mut()) };
    
        check_error(err)
    }

    pub fn send_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), (context as *mut T0).cast()) };
    
        check_error(err)
    }

    pub fn sendmsg(&self, msg: &crate::msg::Msg, options: SendMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
    
        check_error(err)
    }


    pub fn senddata<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), std::ptr::null_mut()) };
    
        check_error(err)
    }

    pub fn senddata_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), (context as *mut T0).cast()) };
    
        check_error(err)
    }

    pub fn inject<T>(&self, buf: &[T], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), mapped_addr.raw_addr()) };
    
        check_error(err)
    }

    pub fn injectdata<T>(&self, buf: &[T], data: u64, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, mapped_addr.raw_addr()) };
    
        check_error(err)
    }
}

impl TransmitContext {
    
	pub fn sendv<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> { // [TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), std::ptr::null_mut()) };

        check_error(err)
    }
    
	pub fn sendv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), (context as *mut T0).cast()) };

        check_error(err)
    }

    pub fn send<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), std::ptr::null_mut()) };
    
        check_error(err)
    }

    pub fn send_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), (context as *mut T0).cast()) };
    
        check_error(err)
    }

    pub fn sendmsg(&self, msg: &crate::msg::Msg, options: SendMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
    
        check_error(err)
    }


    pub fn senddata<T>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), std::ptr::null_mut()) };
    
        check_error(err)
    }

    pub fn senddata_with_context<T, T0>(&self, buf: &[T], desc: &mut impl DataDescriptor, data: u64, mapped_addr: &crate::MappedAddress, context : &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, mapped_addr.raw_addr(), (context as *mut T0).cast()) };
    
        check_error(err)
    }

    pub fn inject<T>(&self, buf: &[T], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), mapped_addr.raw_addr()) };
    
        check_error(err)
    }

    pub fn injectdata<T>(&self, buf: &[T], data: u64, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, mapped_addr.raw_addr()) };
    
        check_error(err)
    }
}

impl ReceiveContext {

    pub fn recv<T>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), std::ptr::null_mut()) };

        check_error(err)
    }

    pub fn recv_with_context<T, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mapped_addr: &crate::MappedAddress, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), mapped_addr.raw_addr(), (context as *mut T0).cast() ) };
    
        check_error(err)
    }
    
	pub fn recvv<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), std::ptr::null_mut()) };

        check_error(err)
    }
    
	pub fn recvv_with_context<T, T0>(&self, iov: &[crate::iovec::IoVec<T>], desc: &mut [impl DataDescriptor], mapped_addr: &crate::MappedAddress, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), mapped_addr.raw_addr(), (context as *mut T0).cast()) };

        check_error(err)
    }
    
    pub fn recvmsg(&self, msg: &crate::msg::Msg, options: RecvMsgOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, options.get_value()) };
        
        check_error(err)
    }
}