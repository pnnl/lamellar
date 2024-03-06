use crate::ep::Endpoint;
use crate::ep::ActiveEndpoint;
use crate::xcontext::ReceiveContext;
use crate::xcontext::TransmitContext;


impl Endpoint {

    pub fn trecv<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, tag: u64, ignore:u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, tag, ignore, std::ptr::null_mut()) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub fn trecv_with_context<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, tag: u64, ignore:u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, tag, ignore, context.get_mut() as *mut  std::ffi::c_void) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(unused_variables)]
    #[allow(clippy::too_many_arguments)]
	pub fn trecvv<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, src_addr: crate::Address, tag: u64, ignore:u64, context : &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        todo!();
        // let ret = unsafe{ libfabric_sys::inlined_fi_trecvv(self.handle(), iov.get(), desc.get_desc(), count, src_addr, tag, ignore, context as *mut T1 as *mut std::ffi::c_void) };
        // ret   
    }

    pub fn trecvmsg(&self, msg: &crate::MsgTagged, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_trecvmsg(self.handle(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, flags) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tsend<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, tag:u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, tag, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tsend_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, tag:u64, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, tag, context.get_mut() as *mut std::ffi::c_void) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(unused_variables)]
	pub fn tsendv<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, dest_addr: crate::Address, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        todo!()
        // let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.get(), desc.get_desc(), count, dest_addr, tag, std::ptr::null_mut()) };
    }

    #[allow(unused_variables)]
	pub fn tsendv_with_context<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, dest_addr: crate::Address, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        todo!()
        // let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.get(), desc.get_desc(), count, dest_addr, tag, context.get_mut() as *mut std::ffi::c_void) };
    }

    pub fn tsendmsg(&self, msg: &crate::MsgTagged, flags: crate::enums::TransferOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsendmsg(self.handle(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, flags.get_value().into()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tsenddata<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address, tag: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, tag, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tsenddata_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address, tag: u64, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, tag, context.get_mut() as *mut std::ffi::c_void) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tinject<T0>(&self, buf: &[T0], addr: crate::Address, tag:u64 ) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tinject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), addr, tag) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tinjectdata<T0>(&self, buf: &[T0], data: u64, addr: crate::Address, tag: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tinjectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, addr, tag) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
}


impl TransmitContext {

    pub fn tsend<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, tag:u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, tag, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tsend_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, tag:u64, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, tag, context.get_mut() as *mut std::ffi::c_void) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(unused_variables)]
	pub fn tsendv<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, dest_addr: crate::Address, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        todo!()
        // let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.get(), desc.get_desc(), count, dest_addr, tag, std::ptr::null_mut()) };
    }

    #[allow(unused_variables)]
	pub fn tsendv_with_context<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, dest_addr: crate::Address, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        todo!()
        // let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.handle(), iov.get(), desc.get_desc(), count, dest_addr, tag, context.get_mut() as *mut std::ffi::c_void) };
    }

    pub fn tsendmsg(&self, msg: &crate::MsgTagged, flags: crate::enums::TransferOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsendmsg(self.handle(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, flags.get_value().into()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tsenddata<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address, tag: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, tag, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tsenddata_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address, tag: u64, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, tag, context.get_mut() as *mut std::ffi::c_void) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tinject<T0>(&self, buf: &[T0], addr: crate::Address, tag:u64 ) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tinject(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), addr, tag) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tinjectdata<T0>(&self, buf: &[T0], data: u64, addr: crate::Address, tag: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tinjectdata(self.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, addr, tag) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
}

impl ReceiveContext {

    pub fn trecv<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, tag: u64, ignore:u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, tag, ignore, std::ptr::null_mut()) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub fn trecv_with_context<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, tag: u64, ignore:u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, tag, ignore, context.get_mut() as *mut  std::ffi::c_void) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(unused_variables)]
    #[allow(clippy::too_many_arguments)]
	pub fn trecvv<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, src_addr: crate::Address, tag: u64, ignore:u64, context : &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        todo!();
        // let ret = unsafe{ libfabric_sys::inlined_fi_trecvv(self.handle(), iov.get(), desc.get_desc(), count, src_addr, tag, ignore, context as *mut T1 as *mut std::ffi::c_void) };
        // ret   
    }

    pub fn trecvmsg(&self, msg: &crate::MsgTagged, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_trecvmsg(self.handle(), &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, flags) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
}
