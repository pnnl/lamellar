use debug_print::debug_println;

#[allow(unused_imports)]
use crate::AsFid;
use crate::{av::AddressVector, cntr::Counter, comm::collective::MulticastGroupCollective, cq::CompletionQueue, eq::EventQueue, OwnedFid};


pub trait BaseEndpoint : AsFid {

    fn handle(&self) -> *mut libfabric_sys::fid_ep;

    fn getname<T0>(&self, addr: &mut[T0]) -> Result<usize, crate::error::Error> {
        
        let mut len: usize = std::mem::size_of_val(addr);
        let len_ptr: *mut usize = &mut len;
        let err: i32 = unsafe { libfabric_sys::inlined_fi_getname(self.as_fid(), addr.as_mut_ptr() as *mut std::ffi::c_void, len_ptr) };

        if -err as u32  == libfabric_sys::FI_ETOOSMALL {
            Err(crate::error::Error{ c_err: -err  as u32, kind: crate::error::ErrorKind::TooSmall(len)} )
        }
        else if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(len)
        }
    }

    fn cancel(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cancel(self.as_fid(), std::ptr::null_mut()) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    fn cancel_with_context<T0>(&self, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cancel(self.as_fid(), context as *mut T0 as *mut std::ffi::c_void) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    fn setopt<T0>(&mut self, level: i32, optname: i32, opt: &[T0]) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_setopt(self.as_fid(), level, optname, opt.as_ptr() as *const std::ffi::c_void, opt.len())};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    fn getopt<T0>(&self, level: i32, optname: i32, opt: &mut [T0]) -> Result<usize, crate::error::Error> {
        let mut len = 0_usize;
        let len_ptr : *mut usize = &mut len;
        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.as_fid(), level, optname, opt.as_mut_ptr() as *mut std::ffi::c_void, len_ptr)};
        
        if -err as u32  == libfabric_sys::FI_ETOOSMALL {
            Err(crate::error::Error{ c_err: -err  as u32, kind: crate::error::ErrorKind::TooSmall(len)} )
        }
        else if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(len)
        }
    }
}

pub trait ActiveEndpoint: BaseEndpoint {

    fn enable(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_enable(self.handle()) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    fn rx_size_left(&self) -> Result<usize, crate::error::Error> {
        let ret = unsafe {libfabric_sys::inlined_fi_rx_size_left(self.handle())};

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()))
        }
        else {
            Ok(ret as usize)
        }
    }

    fn tx_size_left(&self) -> Result<usize, crate::error::Error> {
        let ret = unsafe {libfabric_sys::inlined_fi_tx_size_left(self.handle())};

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()))
        }
        else {
            Ok(ret as usize)
        }
    }

    fn getpeer<T0>(&self, addr: &mut [T0]) -> Result<usize, crate::error::Error> {
        let mut len = addr.len();
        let len_ptr: *mut usize = &mut len;
        let err = unsafe { libfabric_sys::inlined_fi_getpeer(self.handle(), addr.as_mut_ptr() as *mut std::ffi::c_void, len_ptr)};
        
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

    fn connect_with<T0,T1>(&self, addr: &T0, param: &[T1]) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_connect(self.handle(), addr as *const T0 as *const std::ffi::c_void, param.as_ptr() as *const std::ffi::c_void, param.len()) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    fn connect<T0>(&self, addr: &T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_connect(self.handle(), addr as *const T0 as *const std::ffi::c_void, std::ptr::null_mut(), 0) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    fn accept_with<T0>(&self, param: &[T0]) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_accept(self.handle(), param.as_ptr() as *const std::ffi::c_void, param.len()) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    fn accept(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_accept(self.handle(), std::ptr::null_mut(), 0) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    fn shutdown(&self, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_shutdown(self.handle(), flags) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

}


//================== Scalable Endpoint (fi_scalable_ep) ==================//
pub struct ScalableEndpoint {
    pub(crate) c_sep: *mut libfabric_sys::fid_ep,
}

impl ScalableEndpoint {
    pub fn new(domain: &crate::domain::Domain, info: &crate::InfoEntry) -> Result<Self, crate::error::Error> {
        let mut c_sep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_sep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_sep;
        let err = unsafe { libfabric_sys::inlined_fi_scalable_ep(domain.c_domain, info.c_info, c_sep_ptr, std::ptr::null_mut()) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_sep }
            )
        }
    }

    pub fn new_with_context<T0>(domain: &crate::domain::Domain, info: &crate::InfoEntry, context: &mut T0) -> Result<Self, crate::error::Error> {
        let mut c_sep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_sep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_sep;
        let err = unsafe { libfabric_sys::inlined_fi_scalable_ep(domain.c_domain, info.c_info, c_sep_ptr, context as *mut T0 as *mut std::ffi::c_void) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_sep }
            )
        }
    }

    pub fn bind<T: crate::Bind + crate::AsFid>(&self, res: &T, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_scalable_ep_bind(self.c_sep, res.as_fid(), flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn tx_context(&self, idx: i32, mut txattr: crate::TxAttr) -> Result<ScalableEndpoint, crate::error::Error> {
        let mut c_sep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_sep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_sep;

        let err = unsafe {libfabric_sys::inlined_fi_tx_context(self.handle(), idx, txattr.get_mut(), c_sep_ptr, std::ptr::null_mut())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_sep }
            )
        }
    }

    pub fn tx_context_with_context<T0>(&self, idx: i32, mut txattr: crate::TxAttr, context : &mut T0) -> Result<ScalableEndpoint, crate::error::Error> {
        let mut c_sep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_sep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_sep;

        let err = unsafe {libfabric_sys::inlined_fi_tx_context(self.handle(), idx, txattr.get_mut(), c_sep_ptr, context as *mut T0 as *mut std::ffi::c_void)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_sep }
            )
        }
    }

    pub fn rx_context(&self, idx: i32, mut rxattr: crate::RxAttr) -> Result<ScalableEndpoint, crate::error::Error> {
        let mut c_sep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_sep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_sep;

        let err = unsafe {libfabric_sys::inlined_fi_rx_context(self.handle(), idx, rxattr.get_mut(), c_sep_ptr, std::ptr::null_mut())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_sep }
            )
        }
    }

    pub fn rx_context_with_context<T0>(&self, idx: i32, mut rxattr: crate::RxAttr, context : &mut T0) -> Result<ScalableEndpoint, crate::error::Error> {
        let mut c_sep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_sep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_sep;

        let err = unsafe {libfabric_sys::inlined_fi_rx_context(self.handle(), idx, rxattr.get_mut(), c_sep_ptr, context as *mut T0 as *mut std::ffi::c_void)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_sep }
            )
        }
    }

    pub fn alias(&self, flags: u64) -> Result<ScalableEndpoint, crate::error::Error> {
        let mut c_sep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_sep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_sep;
        let err = unsafe { libfabric_sys::inlined_fi_ep_alias(self.handle(), c_sep_ptr, flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_sep }
            )
        }
    }
}

impl crate::AsFid for ScalableEndpoint {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_sep).fid }
    }
}

impl ActiveEndpoint for ScalableEndpoint {

}

impl BaseEndpoint for ScalableEndpoint {
    fn handle(&self) -> *mut libfabric_sys::fid_ep {
       self.c_sep
    }
}

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
    
    pub fn bind(&self, res: &EventQueue, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_pep_bind(self.c_pep, res.as_fid(), flags) };
        
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

    pub fn reject<T0>(&self, fid: &impl crate::AsFid, params: &[T0]) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_reject(self.c_pep, fid.as_fid(), params.as_ptr() as *const std::ffi::c_void, params.len())};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }

    }
}

impl crate::AsFid for PassiveEndpoint {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_pep).fid }
    }    
}


//================== Endpoint (fi_endpoint) ==================//

pub struct Endpoint {
    pub(crate) c_ep: *mut libfabric_sys::fid_ep,
    fid: OwnedFid,
}

pub struct IncompleteBindCq<'a> {
    ep: &'a  Endpoint,
    flags: u64,
}

impl<'a> IncompleteBindCq<'a> {
    pub fn recv(&mut self, selective: bool) -> &mut Self {
        if selective {
            self.flags |= libfabric_sys::FI_SELECTIVE_COMPLETION | libfabric_sys::FI_RECV  as u64 ;
        
            self
        }
        else {
            self.flags |= libfabric_sys::FI_RECV as u64;

            self
        }
    }

    pub fn transmit(&mut self, selective: bool) -> &mut Self {
        if selective {
            self.flags |= libfabric_sys::FI_SELECTIVE_COMPLETION | libfabric_sys::FI_TRANSMIT as u64;

            self
        }
        else {
            self.flags |= libfabric_sys::FI_TRANSMIT as u64;

            self
        }
    }

    pub fn cq(&mut self, cq: &CompletionQueue) -> Result<(), crate::error::Error> {
        self.ep.bind(cq, self.flags)
    }
}

pub struct IncompleteBindCntr<'a> {
    ep: &'a  Endpoint,
    flags: u64,
}

impl<'a> IncompleteBindCntr<'a> {

    
    pub fn read(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_READ as u64;

        self
    }

    pub fn recv(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_RECV as u64;

        self
    }

    pub fn remote_read(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_REMOTE_READ as u64;

        self
    }

    pub fn remote_write(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_REMOTE_WRITE as u64;

        self
    }

    pub fn send(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_SEND as u64;

        self
    }

    pub fn write(&mut self) -> &mut Self {
        self.flags |= libfabric_sys::FI_WRITE as u64;

        self
    }

    pub fn cntr(&mut self, cntr: &Counter) -> Result<(), crate::error::Error> {
        self.ep.bind(cntr, self.flags)
    }
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
                Self { c_ep, fid: OwnedFid { fid: unsafe{ &mut (*c_ep).fid } } }
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
                Self { c_ep, fid: OwnedFid { fid: unsafe{ &mut (*c_ep).fid } } }
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
                Self { c_ep, fid: OwnedFid { fid: unsafe{ &mut (*c_ep).fid } } }
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
                Self { c_ep, fid: OwnedFid { fid: unsafe{ &mut (*c_ep).fid } } }        
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
                Self { c_ep, fid: OwnedFid { fid: unsafe{ &mut (*c_ep).fid } } }        
            )
        }

    }

    pub(crate) fn bind<T: crate::Bind + crate::AsFid>(&self, res: &T, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_ep_bind(self.c_ep, res.as_fid(), flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    } 

    pub fn bind_cq(&self) -> IncompleteBindCq {
        IncompleteBindCq { ep: self, flags: 0}
    }

    pub fn bind_cntr(&self) -> IncompleteBindCntr {
        IncompleteBindCntr { ep: self, flags: 0}
    }

    pub fn bind_eq(&self, eq: &EventQueue) -> Result<(), crate::error::Error>  {
        
        self.bind(eq, 0)
    }

    pub fn bind_av(&self, av: &AddressVector) -> Result<(), crate::error::Error> {
    
        self.bind(av, 0)
    }

    pub fn tx_context(&self, idx: i32, mut txattr: crate::TxAttr) -> Result<Endpoint, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;

        let err = unsafe {libfabric_sys::inlined_fi_tx_context(self.handle(), idx, txattr.get_mut(), c_ep_ptr, std::ptr::null_mut())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep, fid: OwnedFid { fid: unsafe{ &mut (*c_ep).fid } } }
            )
        }
    }

    pub fn tx_context_with_context<T0>(&self, idx: i32, mut txattr: crate::TxAttr, context : &mut T0) -> Result<Endpoint, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;

        let err = unsafe {libfabric_sys::inlined_fi_tx_context(self.handle(), idx, txattr.get_mut(), c_ep_ptr, context as *mut T0 as *mut std::ffi::c_void)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep, fid: OwnedFid { fid: unsafe{ &mut (*c_ep).fid } } }
            )
        }
    }

    pub fn rx_context(&self, idx: i32, mut rxattr: crate::RxAttr) -> Result<Endpoint, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;

        let err = unsafe {libfabric_sys::inlined_fi_rx_context(self.handle(), idx, rxattr.get_mut(), c_ep_ptr, std::ptr::null_mut())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep, fid: OwnedFid { fid: unsafe{ &mut (*c_ep).fid } } }
            )
        }
    }

    pub fn rx_context_with_context<T0>(&self, idx: i32, mut rxattr: crate::RxAttr, context : &mut T0) -> Result<Endpoint, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;

        let err = unsafe {libfabric_sys::inlined_fi_rx_context(self.handle(), idx, rxattr.get_mut(), c_ep_ptr, context as *mut T0 as *mut std::ffi::c_void)};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep, fid: OwnedFid { fid: unsafe{ &mut (*c_ep).fid } } }
            )
        }
    }

    pub fn alias(&self, flags: u64) -> Result<Endpoint, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err = unsafe { libfabric_sys::inlined_fi_ep_alias(self.handle(), c_ep_ptr, flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_ep, fid: OwnedFid { fid: unsafe{ &mut (*c_ep).fid } } }
            )
        }
    }
}

impl crate::AsFid for Endpoint {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
        self.fid.as_fid()
    }
}


impl ActiveEndpoint for Endpoint {

}

impl BaseEndpoint for Endpoint {
    fn handle(&self) -> *mut libfabric_sys::fid_ep {
        self.c_ep
    }
}


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

impl Default for EndpointAttr {
    fn default() -> Self {
        Self::new()
    }
}
