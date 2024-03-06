use crate::ep::Endpoint;
use crate::ep::ActiveEndpoint;
use crate::xcontext::ReceiveContext;
use crate::xcontext::TransmitContext;



impl Endpoint {

    pub fn recv<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn recv_with_context<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, context.get_mut() as *mut  std::ffi::c_void ) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(unused_variables)]
	pub fn recvv<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, addr: crate::Address) -> Result<(), crate::error::Error> { //[TODO]
        todo!();
        // let ret = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.get(), desc.get_desc(), count, addr, std::ptr::null_mut()) };
        // ret
    }
    
    #[allow(unused_variables)]
	pub fn recvv_with_context<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, addr: crate::Address, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        todo!();
        // let ret = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.get(), desc.get_desc(), count, addr, context as *mut T1 as *mut std::ffi::c_void) };
        // ret
    }
    
    pub fn recvmsg(&self, msg: &crate::Msg, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(unused_variables)]
	pub fn sendv<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, addr: crate::Address) -> Result<(), crate::error::Error> { // [TODO]
        todo!()
        // let ret = let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.get(), desc.get_desc(), count, addr, std::ptr::null_mut()) };;
        // ret
    }
    
    #[allow(unused_variables)]
	pub fn sendv_with_context<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, addr: crate::Address, context : &mut crate::Context) -> Result<(), crate::error::Error> { // [TODO]
        todo!()
        // let ret = let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.get(), desc.get_desc(), count, addr, context.get_mut() as *mut  std::ffi::c_void) };;
        // ret
    }

    pub fn send<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn send_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, context.get_mut() as *mut  std::ffi::c_void) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn sendmsg(&self, msg: &crate::Msg, flags: crate::enums::TransferOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, flags.get_value().into()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }


    pub fn senddata<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn senddata_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, context.get_mut() as *mut std::ffi::c_void) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn inject<T0>(&self, buf: &[T0], addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), addr) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn injectdata<T0>(&self, buf: &[T0], data: u64, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, addr) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
}

impl TransmitContext {

    #[allow(unused_variables)]
	pub fn sendv<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, addr: crate::Address) -> Result<(), crate::error::Error> { // [TODO]
        todo!()
        // let ret = let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.get(), desc.get_desc(), count, addr, std::ptr::null_mut()) };;
        // ret
    }
    
    #[allow(unused_variables)]
	pub fn sendv_with_context<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, addr: crate::Address, context : &mut crate::Context) -> Result<(), crate::error::Error> { // [TODO]
        todo!()
        // let ret = let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.handle(), iov.get(), desc.get_desc(), count, addr, context.get_mut() as *mut  std::ffi::c_void) };;
        // ret
    }

    pub fn send<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn send_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, context.get_mut() as *mut  std::ffi::c_void) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn sendmsg(&self, msg: &crate::Msg, flags: crate::enums::TransferOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, flags.get_value().into()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }


    pub fn senddata<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn senddata_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, context.get_mut() as *mut std::ffi::c_void) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn inject<T0>(&self, buf: &[T0], addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), addr) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn injectdata<T0>(&self, buf: &[T0], data: u64, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, addr) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
}

impl ReceiveContext {

    pub fn recv<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn recv_with_context<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, context.get_mut() as *mut  std::ffi::c_void ) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(unused_variables)]
	pub fn recvv<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, addr: crate::Address) -> Result<(), crate::error::Error> { //[TODO]
        todo!();
        // let ret = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.get(), desc.get_desc(), count, addr, std::ptr::null_mut()) };
        // ret
    }
    
    #[allow(unused_variables)]
	pub fn recvv_with_context<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, addr: crate::Address, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        todo!();
        // let ret = unsafe{ libfabric_sys::inlined_fi_recvv(self.handle(), iov.get(), desc.get_desc(), count, addr, context as *mut T1 as *mut std::ffi::c_void) };
        // ret
    }
    
    pub fn recvmsg(&self, msg: &crate::Msg, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.handle(), &msg.c_msg as *const libfabric_sys::fi_msg, flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
}