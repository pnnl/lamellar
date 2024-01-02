#[allow(unused_imports)]
use crate::FID;

//================== Passive Endpoint (fi_passive_ep) ==================//

pub struct PassiveEndpoint {
    pub(crate) c_pep: *mut libfabric_sys::fid_pep,
}

impl PassiveEndpoint {
    pub fn new(fabric: &crate::fabric::Fabric, info: &crate::InfoEntry) -> Result<Self, crate::error::Error> {
        let mut c_pep: *mut libfabric_sys::fid_pep = std::ptr::null_mut();
        let c_pep_ptr: *mut *mut libfabric_sys::fid_pep = &mut c_pep;
        let err = unsafe { libfabric_sys::inlined_fi_passive_ep(fabric.c_fabric, info.c_info, c_pep_ptr, std::ptr::null_mut()) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_pep }
            )
        }
    }

    pub fn new_with_context<T0>(fabric: &crate::fabric::Fabric, info: &crate::InfoEntry, context: &mut T0) -> Result<Self, crate::error::Error> {
        let mut c_pep: *mut libfabric_sys::fid_pep = std::ptr::null_mut();
        let c_pep_ptr: *mut *mut libfabric_sys::fid_pep = &mut c_pep;
        let err = unsafe { libfabric_sys::inlined_fi_passive_ep(fabric.c_fabric, info.c_info, c_pep_ptr, context as *mut T0 as *mut std::ffi::c_void) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_pep }
            )
        }
    }
    
    pub fn bind(&self, fid: &impl crate::FID, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_pep_bind(self.c_pep,fid.fid(), flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn listen(&self) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_listen(self.c_pep)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn reject<T0>(&self, fid: &impl crate::FID, params: &[T0]) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_reject(self.c_pep, fid.fid(), params.as_ptr() as *const std::ffi::c_void, params.len())};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }

    }
}

impl crate::FID for PassiveEndpoint {
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_pep).fid }
    }    
}

//================== Endpoint (fi_endpoint) ==================//

pub struct Endpoint {
    pub(crate) c_ep: *mut libfabric_sys::fid_ep,
}


impl Endpoint {

    pub fn new(domain: &crate::domain::Domain, info: &crate::InfoEntry) -> Result<Self, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err = unsafe { libfabric_sys::inlined_fi_endpoint(domain.c_domain, info.c_info, c_ep_ptr, std::ptr::null_mut()) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep }
            )
        }

    }
    

    pub fn new_with_context<T0>(domain: &crate::domain::Domain, info: &crate::InfoEntry, context: &mut crate::Context) -> Result<Endpoint, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err = unsafe { libfabric_sys::inlined_fi_endpoint(domain.c_domain, info.c_info, c_ep_ptr, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok (
                Self { c_ep }
            )
        }

    }

    pub fn new2<T0>(domain: &crate::domain::Domain, info: &crate::InfoEntry, flags: u64, context: &mut T0) -> Result<Self, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err = unsafe { libfabric_sys::inlined_fi_endpoint2(domain.c_domain, info.c_info, c_ep_ptr, flags, context as *mut T0 as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep }
            )
        }

    }

    pub fn new_scalable(domain: &crate::domain::Domain, info: &crate::InfoEntry) -> Result<Self, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err = unsafe { libfabric_sys::inlined_fi_scalable_ep(domain.c_domain, info.c_info, c_ep_ptr, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep }
            )
        }
    }

    pub fn new_scalable_with_context<T0>(domain: &crate::domain::Domain, info: &crate::InfoEntry, context : &mut T0) -> Result<Self, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err = unsafe { libfabric_sys::inlined_fi_scalable_ep(domain.c_domain, info.c_info, c_ep_ptr, context as *mut T0 as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep }
            )
        }
    }

    pub(crate) fn from_attr(domain: &crate::domain::Domain, mut rx_attr: crate::RxAttr) -> Result<Self, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err = unsafe { libfabric_sys::inlined_fi_srx_context(domain.c_domain, rx_attr.get_mut(), c_ep_ptr,  std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep }        
            )
        }

    }

    pub(crate) fn from_attr_with_context<T0>(domain: &crate::domain::Domain, mut rx_attr: crate::RxAttr, context: &mut T0) -> Result<Self, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err = unsafe { libfabric_sys::inlined_fi_srx_context(domain.c_domain, rx_attr.get_mut(), c_ep_ptr, context as *mut T0 as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep }        
            )
        }

    }

    pub fn bind(&self, fid: &impl crate::FID, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_ep_bind(self.c_ep,fid.fid(), flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    } 

    pub fn enable(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_enable(self.c_ep) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn alias(&self, flags: u64) -> Result<Endpoint, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err = unsafe { libfabric_sys::inlined_fi_ep_alias(self.c_ep, c_ep_ptr, flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Endpoint { c_ep }
            )
        }
    }

    pub fn tx_context(&self, idx: i32, mut txattr: crate::TxAttr) -> Result<Endpoint, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;

        let err = unsafe {libfabric_sys::inlined_fi_tx_context(self.c_ep, idx, txattr.get_mut(), c_ep_ptr, std::ptr::null_mut())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep }
            )
        }
    }

    pub fn tx_context_with_context<T0>(&self, idx: i32, mut txattr: crate::TxAttr, context : &mut T0) -> Result<Endpoint, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;

        let err = unsafe {libfabric_sys::inlined_fi_tx_context(self.c_ep, idx, txattr.get_mut(), c_ep_ptr, context as *mut T0 as *mut std::ffi::c_void)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep }
            )
        }

    }

    pub fn rx_context(&self, idx: i32, mut rxattr: crate::RxAttr) -> Result<Endpoint, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;

        let err = unsafe {libfabric_sys::inlined_fi_rx_context(self.c_ep, idx, rxattr.get_mut(), c_ep_ptr, std::ptr::null_mut())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep }
            )
        }

    }

    pub fn rx_context_with_context<T0>(&self, idx: i32, mut rxattr: crate::RxAttr, context : &mut T0) -> Result<Endpoint, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;

        let err = unsafe {libfabric_sys::inlined_fi_rx_context(self.c_ep, idx, rxattr.get_mut(), c_ep_ptr, context as *mut T0 as *mut std::ffi::c_void)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep }
            )
        }
    }

    pub fn rx_size_left(&self) -> Result<usize, crate::error::Error> {
        let ret = unsafe {libfabric_sys::inlined_fi_rx_size_left(self.c_ep)};

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()))
        }
        else {
            Ok(ret as usize)
        }
    }

    pub fn tx_size_left(&self) -> Result<usize, crate::error::Error> {
        let ret = unsafe {libfabric_sys::inlined_fi_tx_size_left(self.c_ep)};

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()))
        }
        else {
            Ok(ret as usize)
        }
    }
    
    pub fn recv<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn recv_with_context<T0>(&self, buf: &mut [T0], len: usize, desc: &mut impl crate::DataDescriptor, addr: crate::Address, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recv(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, desc.get_desc(), addr, context.get_mut() as *mut  std::ffi::c_void ) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn trecv<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, tag: u64, ignore:u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, tag, ignore, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn trecv_with_context<T0>(&self, buf: &mut [T0], len: usize, desc: &mut impl crate::DataDescriptor, addr: crate::Address, tag: u64, ignore:u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_trecv(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, desc.get_desc(), addr, tag, ignore, context.get_mut() as *mut  std::ffi::c_void) };
    
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
        // let ret = unsafe{ libfabric_sys::inlined_fi_recvv(self.c_ep, iov.get(), desc.get_desc(), count, addr, std::ptr::null_mut()) };
        // ret
    }
    
    #[allow(unused_variables)]
	pub fn recvv_with_context<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, addr: crate::Address, context: &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        todo!();
        // let ret = unsafe{ libfabric_sys::inlined_fi_recvv(self.c_ep, iov.get(), desc.get_desc(), count, addr, context as *mut T1 as *mut std::ffi::c_void) };
        // ret
    }
    
    
    #[allow(unused_variables)]
	pub fn trecvv<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, src_addr: crate::Address, tag: u64, ignore:u64, context : &mut T0) -> Result<(), crate::error::Error> { //[TODO]
        todo!();
        // let ret = unsafe{ libfabric_sys::inlined_fi_trecvv(self.c_ep, iov.get(), desc.get_desc(), count, src_addr, tag, ignore, context as *mut T1 as *mut std::ffi::c_void) };
        // ret   
    }

    pub fn recvmsg(&self, msg: &crate::Msg, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.c_ep, &msg.c_msg as *const libfabric_sys::fi_msg, flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn trecvmsg(&self, msg: &crate::MsgTagged, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_trecvmsg(self.c_ep, &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, flags) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub unsafe fn read<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, src_addr: crate::Address, addr: u64,  key: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_read(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), src_addr, addr, key, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub unsafe fn read_with_context<T0>(&self, buf: &mut [T0], len: usize, desc: &mut impl crate::DataDescriptor, src_addr: crate::Address, addr: u64,  key: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_read(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, desc.get_desc(), src_addr, addr, key, context.get_mut() as *mut  std::ffi::c_void) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(unused_variables)]
	pub unsafe fn readv(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, src_addr: crate::Address, addr: u64, key: u64) -> Result<(), crate::error::Error> { //[TODO]
        todo!()
        // let ret = unsafe{ libfabric_sys::inlined_fi_readv(self.c_ep, iov.get(), desc.get_desc(), count, src_addr, addr, key, std::ptr::null_mut()) };
        // ret 
    }
    
    #[allow(unused_variables)]
	pub unsafe  fn readv_with_context<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, src_addr: crate::Address, addr: u64, key: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> { //[TODO]
        todo!()
        // let ret = unsafe{ libfabric_sys::inlined_fi_readv(self.c_ep, iov.get(), desc.get_desc(), count, src_addr, addr, key, context.get_mut() as *mut  std::ffi::c_void) };
        // ret 
    }


    pub unsafe fn readmsg(&self, msg: &crate::MsgRma, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_readmsg(self.c_ep, &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, flags) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }


    pub unsafe fn write<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key:u64) -> Result<(), crate::error::Error>  {
        let err = unsafe{ libfabric_sys::inlined_fi_write(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), dest_addr, addr, key, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub unsafe fn write_with_context<T0,T1>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key:u64, context: &mut crate::Context) -> Result<(), crate::error::Error>  {
        let err = unsafe{ libfabric_sys::inlined_fi_write(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), dest_addr, addr, key, context.get_mut() as *mut  std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(unused_variables)]
	pub unsafe fn writev<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize,  dest_addr: crate::Address, addr: u64, key:u64, context: &mut crate::Context) -> Result<(), crate::error::Error> { //[TODO]
        todo!()
        // let ret = unsafe{ libfabric_sys::inlined_fi_writev(self.c_ep, iov.get(), desc.get_desc(), count, dest_addr, addr, key, context.get_mut() as *mut  std::ffi::c_void) };
        // ret   
    }
    
    pub unsafe fn writemsg(&self, msg: &crate::MsgRma, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_writemsg(self.c_ep, &msg.c_msg_rma as *const libfabric_sys::fi_msg_rma, flags) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub unsafe fn writedata<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address, other_addr: u64, key: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, other_addr, key, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub unsafe fn writedata_with_context<T0,T1>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address, other_addr: u64, key: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_writedata(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, other_addr, key, context.get_mut() as *mut  std::ffi::c_void) };
    
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
        // let ret = let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.c_ep, iov.get(), desc.get_desc(), count, addr, std::ptr::null_mut()) };;
        // ret
    }
    
    #[allow(unused_variables)]
	pub fn sendv_with_context<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, addr: crate::Address, context : &mut crate::Context) -> Result<(), crate::error::Error> { // [TODO]
        todo!()
        // let ret = let err = unsafe{ libfabric_sys::inlined_fi_sendv(self.c_ep, iov.get(), desc.get_desc(), count, addr, context.get_mut() as *mut  std::ffi::c_void) };;
        // ret
    }

    pub fn send<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn send_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_send(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, context.get_mut() as *mut  std::ffi::c_void) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tsend<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, tag:u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, tag, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tsend_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, addr: crate::Address, tag:u64, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsend(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), addr, tag, context.get_mut() as *mut std::ffi::c_void) };
    
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
        // let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.c_ep, iov.get(), desc.get_desc(), count, dest_addr, tag, std::ptr::null_mut()) };
    }

    #[allow(unused_variables)]
	pub fn tsendv_with_context<T0>(&self, iov: &crate::IoVec, desc: &mut impl crate::DataDescriptor, count: usize, dest_addr: crate::Address, tag:u64, context : &mut T0) -> Result<(), crate::error::Error> { // [TODO]
        todo!()
        // let err = unsafe{ libfabric_sys::inlined_fi_tsendv(self.c_ep, iov.get(), desc.get_desc(), count, dest_addr, tag, context.get_mut() as *mut std::ffi::c_void) };
    }

    pub fn sendmsg(&self, msg: &crate::Msg, flags: crate::enums::TransferOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.c_ep, &msg.c_msg as *const libfabric_sys::fi_msg, flags.get_value().into()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tsendmsg(&self, msg: &crate::MsgTagged, flags: crate::enums::TransferOptions) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsendmsg(self.c_ep, &msg.c_msg_tagged as *const libfabric_sys::fi_msg_tagged, flags.get_value().into()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn senddata<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn senddata_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_senddata(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, context.get_mut() as *mut std::ffi::c_void) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tsenddata<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address, tag: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, tag, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tsenddata_with_context<T0>(&self, buf: &[T0], desc: &mut impl crate::DataDescriptor, data: u64, addr: crate::Address, tag: u64, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), data, addr, tag, context.get_mut() as *mut std::ffi::c_void) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }


    pub fn inject<T0>(&self, buf: &[T0], addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), addr) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tinject<T0>(&self, buf: &[T0], addr: crate::Address, tag:u64 ) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tinject(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), addr, tag) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub unsafe fn inject_write<T0>(&self, buf: &[T0], dest_addr: crate::Address, addr: u64, key:u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject_write(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), dest_addr, addr, key) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }     

    pub fn injectdata<T0>(&self, buf: &[T0], data: u64, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_injectdata(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, addr) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tinjectdata<T0>(&self, buf: &[T0], data: u64, addr: crate::Address, tag: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_tinjectdata(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, addr, tag) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub unsafe fn inject_writedata<T0>(&self, buf: &[T0], data: u64, dest_addr: crate::Address, addr: u64, key: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_inject_writedata(self.c_ep, buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), data, dest_addr, addr, key) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn getpeer<T0>(&self, addr: &mut [T0]) -> Result<usize, crate::error::Error> {
        let mut len = addr.len();
        let len_ptr: *mut usize = &mut len;
        let err = unsafe { libfabric_sys::inlined_fi_getpeer(self.c_ep, addr.as_mut_ptr() as *mut std::ffi::c_void, len_ptr)};
        
        if addr.len() < len {
            Err(crate::error::Error{c_err: libfabric_sys::FI_ETOOSMALL, kind: crate::error::ErrorKind::TooSmall(len)})
        }
        else if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(len)
        }
    }

    pub fn connect_with<T0,T1>(&self, addr: &T0, param: &[T1]) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_connect(self.c_ep, addr as *const T0 as *const std::ffi::c_void, param.as_ptr() as *const std::ffi::c_void, param.len()) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn connect<T0>(&self, addr: &T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_connect(self.c_ep, addr as *const T0 as *const std::ffi::c_void, std::ptr::null_mut(), 0) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn accept_with<T0>(&self, param: &[T0]) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_accept(self.c_ep, param.as_ptr() as *const std::ffi::c_void, param.len()) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn accept(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_accept(self.c_ep, std::ptr::null_mut(), 0) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn shutdown(&self, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_shutdown(self.c_ep, flags) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn atomic<T0,T1>(&self, buf: &[T0], count : usize, desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.c_ep, buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc(), dest_addr, addr, key, datatype, op.get_value(), std::ptr::null_mut())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn atomic_with_context<T0,T1>(&self, buf: &[T0], count : usize, desc: &mut impl crate::DataDescriptor, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomic(self.c_ep, buf.as_ptr()  as *const std::ffi::c_void, count, desc.get_desc(), dest_addr, addr, key, datatype, op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn atomicv<T0,T1>(&self, iov: &crate::Ioc, desc: &mut [impl crate::DataDescriptor], count : usize, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.c_ep, iov.get(), desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, count, dest_addr, addr, key, datatype, op.get_value(), std::ptr::null_mut())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn atomicv_with_context<T0,T1>(&self, iov: &crate::Ioc, desc: &mut [impl crate::DataDescriptor], count : usize, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_atomicv(self.c_ep, iov.get(), desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, count, dest_addr, addr, key, datatype, op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn atomicmsg(&self, msg: &crate::MsgAtomic, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_atomicmsg(self.c_ep, msg.c_msg_atomic, flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn inject_atomic<T0,T1>(&self, buf: &[T0], count : usize, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_inject_atomic(self.c_ep, buf.as_ptr()  as *const std::ffi::c_void, count, dest_addr, addr, key, datatype, op.get_value())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn fetch_atomic<T0,T1>(&self, buf: &[T0], count : usize, desc: &mut [T1], res: &mut [T0], res_desc: &mut [T1], dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.c_ep, buf.as_ptr()  as *const std::ffi::c_void, count, desc.as_mut_ptr()  as *mut std::ffi::c_void, res.as_mut_ptr()  as *mut std::ffi::c_void, res_desc.as_mut_ptr() as *mut std::ffi::c_void, dest_addr, addr, key, datatype, op.get_value(), std::ptr::null_mut())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn fetch_atomic_with_context<T0,T1>(&self, buf: &[T0], count : usize, desc: &mut [T1], res: &mut [T0], res_desc: &mut [T1], dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.c_ep, buf.as_ptr()  as *const std::ffi::c_void, count, desc.as_mut_ptr()  as *mut std::ffi::c_void, res.as_mut_ptr()  as *mut std::ffi::c_void, res_desc.as_mut_ptr() as *mut std::ffi::c_void, dest_addr, addr, key, datatype, op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn fetch_atomicv<T0,T1>(&self, iov: &crate::Ioc, desc: &mut [T1], count : usize, resultv: &mut crate::Ioc,  res_desc: &mut [T1], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.c_ep, iov.get(), desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, count, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, dest_addr, addr, key, datatype, op.get_value(), std::ptr::null_mut())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn fetch_atomicv_with_context<T0,T1>(&self, iov: &crate::Ioc, desc: &mut [T1], count : usize, resultv: &mut crate::Ioc,  res_desc: &mut [T1], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error>{
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.c_ep, iov.get(), desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, count, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, dest_addr, addr, key, datatype, op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn fetch_atomicmsg<T0>(&self, msg: &crate::MsgAtomic,  resultv: &mut crate::Ioc,  res_desc: &mut [T0], res_count : usize, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_fetch_atomicmsg(self.c_ep, msg.c_msg_atomic, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, flags) };
        
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn compare_atomic<T0, T1>(&self, buf: &[T0], count : usize, desc: &mut [T1], compare: &mut [T0], compare_desc: &mut [T1], 
            result: &mut [T0], result_desc: &mut [T1], dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.c_ep, buf.as_ptr()  as *const std::ffi::c_void, count, desc.as_mut_ptr()  as *mut std::ffi::c_void, compare.as_mut_ptr()  as *mut std::ffi::c_void, 
            compare_desc.as_mut_ptr()  as *mut std::ffi::c_void, result.as_mut_ptr()  as *mut std::ffi::c_void, result_desc.as_mut_ptr()  as *mut std::ffi::c_void, dest_addr, addr, key, datatype, op.get_value(), std::ptr::null_mut())};
        
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn compare_atomic_with_context<T0, T1>(&self, buf: &[T0], count : usize, desc: &mut [T1], compare: &mut [T0], compare_desc: &mut [T1], 
            result: &mut [T0], result_desc: &mut [T1], dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.c_ep, buf.as_ptr()  as *const std::ffi::c_void, count, desc.as_mut_ptr()  as *mut std::ffi::c_void, compare.as_mut_ptr()  as *mut std::ffi::c_void, 
            compare_desc.as_mut_ptr()  as *mut std::ffi::c_void, result.as_mut_ptr()  as *mut std::ffi::c_void, result_desc.as_mut_ptr()  as *mut std::ffi::c_void, dest_addr, addr, key, datatype, op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn compare_atomicv(&self, iov: &crate::Ioc, desc: &mut [impl crate::DataDescriptor], count : usize, comparetv: &mut crate::Ioc,  compare_desc: &mut [impl crate::DataDescriptor], compare_count : usize, 
        resultv: &mut crate::Ioc,  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.c_ep, iov.get(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, comparetv.get_mut(), compare_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, compare_count, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, dest_addr, addr, key, datatype, op.get_value(), std::ptr::null_mut())};
        
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn compare_atomicv_with_context(&self, iov: &crate::Ioc, desc: &mut [impl crate::DataDescriptor], count : usize, comparetv: &mut crate::Ioc,  compare_desc: &mut [impl crate::DataDescriptor], compare_count : usize, 
        resultv: &mut crate::Ioc,  res_desc: &mut [impl crate::DataDescriptor], res_count : usize, dest_addr: crate::Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::enums::Op, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.c_ep, iov.get(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, comparetv.get_mut(), compare_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, compare_count, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, dest_addr, addr, key, datatype, op.get_value(), context.get_mut() as *mut std::ffi::c_void)};
        
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn compare_atomicmsg<T0>(&self, msg: &crate::MsgAtomic, comparev: &crate::Ioc, compare_desc: &mut [T0], compare_count : usize, resultv: &mut crate::Ioc,  res_desc: &mut [T0], res_count : usize, flags: u64) -> Result<(), crate::error::Error> {
        let err: isize = unsafe { libfabric_sys::inlined_fi_compare_atomicmsg(self.c_ep, msg.c_msg_atomic, comparev.get(), compare_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, compare_count, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, flags) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn atomicvalid(&self, datatype: crate::DataType, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_atomicvalid(self.c_ep, datatype, op.get_value(), &mut count as *mut usize)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }

    pub fn fetch_atomicvalid(&self, datatype: crate::DataType, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_fetch_atomicvalid(self.c_ep, datatype, op.get_value(), &mut count as *mut usize)};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }

    pub fn compare_atomicvalid(&self, datatype: crate::DataType, op: crate::enums::Op) -> Result<usize, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_compare_atomicvalid(self.c_ep, datatype, op.get_value(), &mut count as *mut usize)};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(count)
        }
    }

    pub fn join<T0>(&self, addr: &T0, flags: u64) -> Result<crate::Mc, crate::error::Error> { // [TODO]
        crate::Mc::new(self, addr, flags)
    }

    pub fn join_with_context<T0,T1>(&self, addr: &T0, flags: u64, context: &mut crate::Context) -> Result<crate::Mc, crate::error::Error> {
        crate::Mc::new_with_context(self, addr, flags, context)
    }

    pub fn join_collective(&self, coll_addr: crate::Address, set: &crate::av::AddressVectorSet, flags: u64) -> Result<crate::Mc, crate::error::Error> {
        crate::Mc::new_collective(self, coll_addr, set, flags)
    }

    pub fn join_collective_with_context(&self, coll_addr: crate::Address, set: &crate::av::AddressVectorSet, flags: u64, context : &mut crate::Context) -> Result<crate::Mc, crate::error::Error> {
        crate::Mc::new_collective_with_context(self, coll_addr, set, flags, context)
    }

    pub fn barrier<T0>(&self, addr: crate::Address) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_barrier(self.c_ep, addr, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn barrier_with_context<T0>(&self, addr: crate::Address, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_barrier(self.c_ep, addr, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn barrier2<T0>(&self, addr: crate::Address, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_barrier2(self.c_ep, addr, flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn barrier2_with_context<T0>(&self, addr: crate::Address, flags: u64, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_barrier2(self.c_ep, addr, flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn broadcast<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, root_addr: crate::Address, datatype: crate::DataType, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_broadcast(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), coll_addr, root_addr, datatype, flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn broadcast_with_context<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, root_addr: crate::Address, datatype: crate::DataType, flags: u64, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_broadcast(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), coll_addr, root_addr, datatype, flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn alltoall<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, result: &mut T0, result_desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, datatype: crate::DataType, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_alltoall(self.c_ep, buf as *mut [T0] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T0 as *mut std::ffi::c_void, result_desc.get_desc(), coll_addr, datatype, flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn alltoall_with_context<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, result: &mut T0, result_desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, datatype: crate::DataType, flags: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_alltoall(self.c_ep, buf as *mut [T0] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T0 as *mut std::ffi::c_void, result_desc.get_desc(), coll_addr, datatype, flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn allreduce<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, result: &mut T0, result_desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, datatype: crate::DataType, op: crate::enums::Op,  flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_allreduce(self.c_ep, buf as *mut [T0] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T0 as *mut std::ffi::c_void, result_desc.get_desc(), coll_addr, datatype, op.get_value(), flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn allreduce_with_context<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, result: &mut T0, result_desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, datatype: crate::DataType, op: crate::enums::Op,  flags: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_allreduce(self.c_ep, buf as *mut [T0] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T0 as *mut std::ffi::c_void, result_desc.get_desc(), coll_addr, datatype, op.get_value(), flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub fn allgather<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, result: &mut T0, result_desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, datatype: crate::DataType, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_allgather(self.c_ep, buf as *mut [T0] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T0 as *mut std::ffi::c_void, result_desc.get_desc(), coll_addr, datatype, flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub fn allgather_with_context<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, result: &mut T0, result_desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, datatype: crate::DataType, flags: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_allgather(self.c_ep, buf as *mut [T0] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T0 as *mut std::ffi::c_void, result_desc.get_desc(), coll_addr, datatype, flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub fn reduce_scatter<T0,T2>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, result: &mut T0, result_desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, datatype: crate::DataType, op: crate::enums::Op,  flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_reduce_scatter(self.c_ep, buf as *mut [T0] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T0 as *mut std::ffi::c_void, result_desc.get_desc(), coll_addr, datatype, op.get_value(), flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn reduce_scatter_with_context<T0,T1>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, result: &mut T0, result_desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, datatype: crate::DataType, op: crate::enums::Op,  flags: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_reduce_scatter(self.c_ep, buf as *mut [T0] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T0 as *mut std::ffi::c_void, result_desc.get_desc(), coll_addr, datatype, op.get_value(), flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub fn reduce<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, result: &mut T0, result_desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, root_addr: crate::Address, datatype: crate::DataType, op: crate::enums::Op,  flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_reduce(self.c_ep, buf as *mut [T0] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T0 as *mut std::ffi::c_void, result_desc.get_desc(), coll_addr, root_addr, datatype, op.get_value(), flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub fn reduce_with_context<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, result: &mut T0, result_desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, root_addr: crate::Address, datatype: crate::DataType, op: crate::enums::Op,  flags: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_reduce(self.c_ep, buf as *mut [T0] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T0 as *mut std::ffi::c_void, result_desc.get_desc(), coll_addr, root_addr, datatype, op.get_value(), flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub fn scatter<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, result: &mut T0, result_desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, root_addr: crate::Address, datatype: crate::DataType,  flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_scatter(self.c_ep, buf as *mut [T0] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T0 as *mut std::ffi::c_void, result_desc.get_desc(), coll_addr, root_addr, datatype, flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub fn scatter_with_context<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, result: &mut T0, result_desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, root_addr: crate::Address, datatype: crate::DataType,  flags: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_scatter(self.c_ep, buf as *mut [T0] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T0 as *mut std::ffi::c_void, result_desc.get_desc(), coll_addr, root_addr, datatype, flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub fn gather<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, result: &mut T0, result_desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, root_addr: crate::Address, datatype: crate::DataType,  flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_gather(self.c_ep, buf as *mut [T0] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T0 as *mut std::ffi::c_void, result_desc.get_desc(), coll_addr, root_addr, datatype, flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub fn gather_with_context<T0>(&self, buf: &mut [T0], desc: &mut impl crate::DataDescriptor, result: &mut T0, result_desc: &mut impl crate::DataDescriptor, coll_addr: crate::Address, root_addr: crate::Address, datatype: crate::DataType,  flags: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_gather(self.c_ep, buf as *mut [T0] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T0 as *mut std::ffi::c_void, result_desc.get_desc(), coll_addr, root_addr, datatype, flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
}

impl crate::FID for Endpoint {
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_ep).fid }
    }
}

// impl Drop for Endpoint {
    
//     fn drop(&mut self) {
//         println!("Dropping ep");
//         self.close();
//     }
// }

// impl Drop for PassiveEndpoint {
//     fn drop(&mut self) {
//         self.close();
//     }
// }

//================== Endpoint attribute ==================//
#[derive(Clone)]
pub struct EndpointAttr {
    c_attr: libfabric_sys::fi_ep_attr,
}

impl EndpointAttr {
    pub fn new() -> Self {
        let c_attr = libfabric_sys::fi_ep_attr{
            type_: libfabric_sys::fi_ep_type_FI_EP_UNSPEC,
            protocol: libfabric_sys::FI_PROTO_UNSPEC,
            protocol_version: 0,
            max_msg_size: 0,
            msg_prefix_size: 0,
            max_order_raw_size: 0,
            max_order_war_size: 0,
            max_order_waw_size: 0,
            mem_tag_format: 0,
            tx_ctx_cnt: 0,
            rx_ctx_cnt: 0,
            auth_key_size: 0,
            auth_key: std::ptr::null_mut(),
        };

        Self { c_attr }
    }

    pub(crate) fn from(c_ep_attr: *mut libfabric_sys::fi_ep_attr) -> Self {
        let c_attr = unsafe { *c_ep_attr };

        Self { c_attr }
    }

    pub fn ep_type(self, type_: crate::enums::EndpointType) -> Self {

        let mut c_attr = self.c_attr;
        c_attr.type_ = type_.get_value();

        Self { c_attr }
    }

    pub fn get_type(&self) -> crate::enums::EndpointType {
        crate::enums::EndpointType::from(self.c_attr.type_)
    }

    pub fn get_max_msg_size(&self) -> usize {
        self.c_attr.max_msg_size 
    }

    pub fn get_msg_prefix_size(&self) -> usize {
        self.c_attr.msg_prefix_size
    }

    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_ep_attr {
        &self.c_attr
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_ep_attr {
        &mut self.c_attr
    }
}
