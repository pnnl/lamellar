use crate::check_error;
use crate::ep::Endpoint;
use crate::ep::ActiveEndpoint;
use crate::infocapsoptions::MsgCap;
use crate::xcontext::ReceiveContext;
use crate::xcontext::TransmitContext;



impl<E: MsgCap> Endpoint<E> {

    pub fn recv<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, std::ptr::null_mut()) };

        check_error(err)
    }

    pub fn recv_with_context<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, context.get_mut() as *mut  std::ffi::c_void ) };
    
        check_error(err)
    }

    #[allow(unused_variables)]
	pub fn recvv<T>(&self, iov: &[crate::IoVec<T>], desc: &mut [impl crate::DataDescriptor], count: usize, addr: crate::Address) -> Result<(), crate::error::Error> { //[TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.iter_mut().map(|v| v.get_desc()).collect::<Vec<_>>().as_mut_ptr().cast()  , count, addr, std::ptr::null_mut()) };
        check_error(err)
    }
    
    #[allow(unused_variables)]
	pub fn recvv_with_context<T>(&self, iov: &[crate::IoVec<T>], desc: &mut [impl crate::DataDescriptor], count: usize, addr: crate::Address,  context: &mut crate::Context) -> Result<(), crate::error::Error> { //[TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.iter_mut().map(|v| v.get_desc()).collect::<Vec<_>>().as_mut_ptr().cast()  , count, addr, context.get_mut().cast()) };
        check_error(err)
    }
    
    pub fn recvmsg(&self, msg: &crate::Msg, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, flags) };
        
        check_error(err)
    }

	pub fn sendv<T, T0>(&self, iov: &[crate::IoVec<T>], desc: &mut [impl crate::DataDescriptor], addr: crate::Address) -> Result<(), crate::error::Error> { 
        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), addr, std::ptr::null_mut()) };
        
        check_error(err)
    }
    
	pub fn sendv_with_context<T,T0>(&self, iov: &[crate::IoVec<T>], desc: &mut [impl crate::DataDescriptor], addr: crate::Address, context : &mut crate::Context) -> Result<(), crate::error::Error> { // [TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast() , desc.as_mut_ptr().cast(), iov.len(), addr, context.get_mut().cast()) };
        
        check_error(err)
    }

    pub fn send<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, std::ptr::null_mut()) };
    
        check_error(err)
    }

    pub fn send_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, context.get_mut().cast()) };
    
        check_error(err)
    }

    pub fn sendmsg(&self, msg: &crate::Msg, flags: crate::enums::TransferOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, flags.get_value().into()) };
    
        check_error(err)
    }


    pub fn senddata<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, std::ptr::null_mut()) };
    
        check_error(err)
    }

    pub fn senddata_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, context.get_mut() as *mut std::ffi::c_void) };
    
        check_error(err)
    }

    pub fn inject<T0>(&self, buf: &[T0], addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), addr) };
    
        check_error(err)
    }

    pub fn injectdata<T0>(&self, buf: &[T0], data: u64, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, addr) };
    
        check_error(err)
    }
}

impl TransmitContext {
    
    #[allow(unused_variables)]
	pub fn sendv<T, T0>(&self, iov: &[crate::IoVec<T>], desc: &mut [impl crate::DataDescriptor], count: usize, addr: crate::Address) -> Result<(), crate::error::Error> { // [TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), addr, std::ptr::null_mut()) };

        check_error(err)
    }
    
    #[allow(unused_variables)]
	pub fn sendv_with_context<T, T0>(&self, iov: &[crate::IoVec<T>], desc: &mut [impl crate::DataDescriptor], count: usize, addr: crate::Address, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), addr, (context as *mut T0).cast()) };

        check_error(err)
    }

    pub fn send<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, std::ptr::null_mut()) };
    
        check_error(err)
    }

    pub fn send_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, context.get_mut() as *mut  std::ffi::c_void) };
    
        check_error(err)
    }

    pub fn sendmsg(&self, msg: &crate::Msg, flags: crate::enums::TransferOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, flags.get_value().into()) };
    
        check_error(err)
    }


    pub fn senddata<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, std::ptr::null_mut()) };
    
        check_error(err)
    }

    pub fn senddata_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, context.get_mut() as *mut std::ffi::c_void) };
    
        check_error(err)
    }

    pub fn inject<T0>(&self, buf: &[T0], addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), addr) };
    
        check_error(err)
    }

    pub fn injectdata<T0>(&self, buf: &[T0], data: u64, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, addr) };
    
        check_error(err)
    }
}

impl ReceiveContext {

    pub fn recv<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, std::ptr::null_mut()) };

        check_error(err)
    }

    pub fn recv_with_context<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, context.get_mut() as *mut  std::ffi::c_void ) };
    
        check_error(err)
    }
    
    // #[allow(unused_variables)]
	pub fn recvv<T, T0>(&self, iov: &[crate::IoVec<T>], desc: &mut [impl crate::DataDescriptor], addr: crate::Address) -> Result<(), crate::error::Error> { //[TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), addr, std::ptr::null_mut()) };

        check_error(err)
    }
    
    #[allow(unused_variables)]
	pub fn recvv_with_context<T, T0>(&self, iov: &[crate::IoVec<T>], desc: &mut [impl crate::DataDescriptor], count: usize, addr: crate::Address, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        let err = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.as_ptr().cast(), desc.as_mut_ptr().cast(), iov.len(), addr, (context as *mut T0).cast()) };

        check_error(err)
    }
    
    pub fn recvmsg(&self, msg: &crate::Msg, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, flags) };
        
        check_error(err)
    }
}