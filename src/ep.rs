use std::{os::fd::{AsFd, BorrowedFd}, rc::Rc, cell::{RefCell, OnceCell}, marker::PhantomData};

use libfabric_sys::{fi_wait_obj_FI_WAIT_FD, inlined_fi_control, FI_BACKLOG, FI_GETOPSFLAG};

#[allow(unused_imports)]
use crate::fid::AsFid;
use crate::{av::{AddressVector}, cntr::Counter, cqoptions::CqConfig, enums::{HmemP2p, TransferOptions}, eq::{EventQueue, EventQueueImpl}, eqoptions::EqConfig, domain::DomainImpl, fabric::FabricImpl, utils::check_error, info::InfoEntry, fid::{OwnedFid, self, AsRawFid}};

#[repr(C)]
pub struct Address {
    address: Vec<u8>,
}

impl Address {

    pub unsafe fn from_raw_parts(raw: *const u8, len: usize) -> Self {
        let mut address = vec![0u8; len];
        address.copy_from_slice(std::slice::from_raw_parts(raw, len));
        Self{address}
    }

    pub unsafe fn from_bytes(raw: &[u8]) -> Self {
        Address { address: raw.to_vec() }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.address
    } 
}

pub(crate) struct EndpointImpl {
    pub(crate) c_ep: *mut libfabric_sys::fid_ep,
    fid: OwnedFid,
    _sync_rcs: RefCell<Vec<Rc<dyn crate::BindImpl>>>,
    pub(crate) bound_eq: OnceCell<Rc<EventQueueImpl>>,
    _domain_rc:  Rc<DomainImpl>,
}

pub struct Endpoint<T> {
    pub(crate) inner: Rc<EndpointImpl>,
    phantom: PhantomData<T>,
}


pub trait BaseEndpointImpl : AsFid {

    fn getname(&self) -> Result<Address, crate::error::Error> {
        let mut len = 0;
        let err: i32 = unsafe { libfabric_sys::inlined_fi_getname(self.as_fid().as_raw_fid(), std::ptr::null_mut(), &mut len) };
        if -err as u32  == libfabric_sys::FI_ETOOSMALL {
            let mut address = vec![0; len];
            let err: i32 = unsafe { libfabric_sys::inlined_fi_getname(self.as_fid().as_raw_fid(), address.as_mut_ptr().cast(), &mut len) };
            if err < 0
            {
                Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
            }
            else 
            {
                Ok(Address{address})
            }
        }
        else
        {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
    }

    fn buffered_limit(&self) -> Result<usize, crate::error::Error> {
        let mut res = 0_usize;
        let mut len = std::mem::size_of::<usize>();

        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.as_fid().as_raw_fid(), libfabric_sys::FI_OPT_ENDPOINT as i32, libfabric_sys::FI_OPT_BUFFERED_LIMIT as i32, (&mut res as *mut usize).cast(), &mut len)};
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(res)
        }
    }

    fn set_buffered_limit(&self, size: usize) -> Result<(), crate::error::Error> {
        let mut res = size;
        let mut len = std::mem::size_of::<usize>();

        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.as_fid().as_raw_fid(), libfabric_sys::FI_OPT_ENDPOINT as i32, libfabric_sys::FI_OPT_BUFFERED_LIMIT as i32, (&mut res as *mut usize).cast(), &mut len)};
    
        check_error(err.try_into().unwrap())
    }

    fn buffered_min(&self) -> Result<usize, crate::error::Error> {
        let mut res = 0_usize;
        let mut len = std::mem::size_of::<usize>();

        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.as_fid().as_raw_fid(), libfabric_sys::FI_OPT_ENDPOINT as i32, libfabric_sys::FI_OPT_BUFFERED_MIN as i32, (&mut res as *mut usize).cast(), &mut len)};
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(res)
        }
    }

    fn set_buffered_min(&self, size: usize) -> Result<(), crate::error::Error> {
        let mut res = size;
        let mut len = std::mem::size_of::<usize>();

        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.as_fid().as_raw_fid(), libfabric_sys::FI_OPT_ENDPOINT as i32, libfabric_sys::FI_OPT_BUFFERED_MIN as i32, (&mut res as *mut usize).cast(), &mut len)};
    
        check_error(err.try_into().unwrap())
    }

    fn cm_data_size(&self) -> Result<usize, crate::error::Error> {
        let mut res = 0_usize;
        let mut len = std::mem::size_of::<usize>();

        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.as_fid().as_raw_fid(), libfabric_sys::FI_OPT_ENDPOINT as i32, libfabric_sys::FI_OPT_CM_DATA_SIZE as i32, (&mut res as *mut usize).cast(), &mut len)};
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(res)
        }
    }

    fn set_cm_data_size(&self, size: usize) -> Result<(), crate::error::Error> {
        let mut res = size;
        let mut len = std::mem::size_of::<usize>();

        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.as_fid().as_raw_fid(), libfabric_sys::FI_OPT_ENDPOINT as i32, libfabric_sys::FI_OPT_CM_DATA_SIZE as i32, (&mut res as *mut usize).cast(), &mut len)};
    
        check_error(err.try_into().unwrap())
    }

    fn min_multi_recv(&self) -> Result<usize, crate::error::Error> {
        let mut res = 0_usize;
        let mut len = std::mem::size_of::<usize>();

        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.as_fid().as_raw_fid(), libfabric_sys::FI_OPT_ENDPOINT as i32, libfabric_sys::FI_OPT_MIN_MULTI_RECV as i32, (&mut res as *mut usize).cast(), &mut len)};
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(res)
        }
    }

    fn set_min_multi_recv(&self, size: usize) -> Result<(), crate::error::Error> {
        let mut res = size;
        let mut len = std::mem::size_of::<usize>();

        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.as_fid().as_raw_fid(), libfabric_sys::FI_OPT_ENDPOINT as i32, libfabric_sys::FI_OPT_MIN_MULTI_RECV as i32, (&mut res as *mut usize).cast(), &mut len)};
    
        check_error(err.try_into().unwrap())
    }

    fn hmem_p2p(&self) -> Result<HmemP2p, crate::error::Error> {
        let mut res = 0_u32;
        let mut len = std::mem::size_of::<u32>();

        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.as_fid().as_raw_fid(), libfabric_sys::FI_OPT_ENDPOINT as i32, libfabric_sys::FI_OPT_FI_HMEM_P2P as i32, (&mut res as *mut u32).cast(), &mut len)};
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(HmemP2p::from_value(res))
        }
    }

    fn xpu_trigger(&self) -> Result<libfabric_sys::fi_trigger_xpu, crate::error::Error> {
        let mut res = libfabric_sys::fi_trigger_xpu {
            count: 0,
            iface: 0,
            device: libfabric_sys::fi_trigger_xpu__bindgen_ty_1 {
                reserved: 0,
            },
            var: std::ptr::null_mut(),
        };
        let mut len = std::mem::size_of::<libfabric_sys::fi_trigger_xpu>();

        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.as_fid().as_raw_fid(), libfabric_sys::FI_OPT_ENDPOINT as i32, libfabric_sys::FI_OPT_XPU_TRIGGER as i32, (&mut res as *mut libfabric_sys::fi_trigger_xpu).cast(), &mut len)};
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(res)
        }
    }

    fn set_hmem_p2p(&self, hmem: HmemP2p) -> Result<(), crate::error::Error> {

        let mut len = std::mem::size_of::<u32>();

        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.as_fid().as_raw_fid(), libfabric_sys::FI_OPT_ENDPOINT as i32, libfabric_sys::FI_OPT_FI_HMEM_P2P as i32, (&mut hmem.get_value() as *mut u32).cast(), &mut len)};
    
        check_error(err.try_into().unwrap())
    }

    fn set_cuda_api_permitted(&self, permitted: bool) -> Result<(), crate::error::Error> {
    
        let mut val = if permitted {1_u32} else {0_u32}; 
        let mut len = std::mem::size_of::<u32>();

        let err = unsafe { libfabric_sys::inlined_fi_getopt(self.as_fid().as_raw_fid(), libfabric_sys::FI_OPT_ENDPOINT as i32, libfabric_sys::FI_OPT_FI_HMEM_P2P as i32, (&mut val as *mut u32).cast(), &mut len)};
    
        check_error(err.try_into().unwrap())
    }

    fn wait_fd(&self) -> Result<BorrowedFd, crate::error::Error> {
        let mut fd = 0;

        let err = unsafe{ libfabric_sys::inlined_fi_control(self.as_fid().as_raw_fid(), fi_wait_obj_FI_WAIT_FD as i32, (&mut fd as *mut i32).cast())};
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(unsafe{BorrowedFd::borrow_raw(fd)})
        }
    }
}


impl<T> BaseEndpointImpl for Endpoint<T> {}

impl BaseEndpointImpl for EndpointImpl {}

impl<T> ActiveEndpointImpl for Endpoint<T> {
    fn handle(&self) -> *mut libfabric_sys::fid_ep {
        self.inner.handle()
    }
}

// impl<'a, T> ActiveEndpoint<'a> for Endpoint<T> {
    
//     fn inner(&self) -> Rc<dyn BaseEndpointImpl> {
//         self.inner.clone()
//     }
// }

impl EndpointImpl {
    
    #[allow(dead_code)]
    pub fn getname(&self) -> Result<Address, crate::error::Error> {
        BaseEndpointImpl::getname(self)
    }
    
    #[allow(dead_code)]
    pub fn buffered_limit(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::buffered_limit(self)
    }
    
    #[allow(dead_code)]
    pub fn set_buffered_limit(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_buffered_limit(self, size)
    }
    
    #[allow(dead_code)]
    pub fn buffered_min(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::buffered_min(self)
    }
    
    #[allow(dead_code)]
    pub fn set_buffered_min(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_buffered_min(self, size)
    }
    
    #[allow(dead_code)]
    pub fn cm_data_size(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::cm_data_size(self)
    }
    
    #[allow(dead_code)]
    pub fn set_cm_data_size(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_cm_data_size(self, size)
    }
    
    #[allow(dead_code)]
    pub fn min_multi_recv(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::min_multi_recv(self)
    }
    
    #[allow(dead_code)]
    pub fn set_min_multi_recv(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_min_multi_recv(self, size)
    }
    
    #[allow(dead_code)]
    pub fn set_hmem_p2p(&self, hmem: HmemP2p) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_hmem_p2p(self, hmem)
    }
    
    #[allow(dead_code)]
    pub fn hmem_p2p(&self) -> Result<HmemP2p, crate::error::Error> {
        BaseEndpointImpl::hmem_p2p(self)
    }
    
    #[allow(dead_code)]
    pub fn xpu_trigger(&self) -> Result<libfabric_sys::fi_trigger_xpu, crate::error::Error> {
        BaseEndpointImpl::xpu_trigger(self)
    }
    
    #[allow(dead_code)]
    pub fn set_cuda_api_permitted(&self, permitted: bool) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_cuda_api_permitted(self, permitted)
    }
    
    #[allow(dead_code)]
    pub fn wait_fd(&self) -> Result<BorrowedFd, crate::error::Error> {
        BaseEndpointImpl::wait_fd(self)
    }
    
    #[allow(dead_code)]
    pub fn enable(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::enable(self)
    }
    
    #[allow(dead_code)]
    pub fn cancel(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::cancel(self)
    }
    
    #[allow(dead_code)]
    pub fn cancel_with_context<T0>(&self, context: &mut T0) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::cancel_with_context(self, context)
    }
    
    #[allow(dead_code)]
    pub fn rx_size_left(&self) -> Result<usize, crate::error::Error> {
        ActiveEndpointImpl::rx_size_left(self)
    }
    
    #[allow(dead_code)]
    pub fn tx_size_left(&self) -> Result<usize, crate::error::Error> {
        ActiveEndpointImpl::tx_size_left(self)
    }
    
    #[allow(dead_code)]
    pub fn getpeer(&self) -> Result<Address, crate::error::Error> {
        ActiveEndpointImpl::getpeer(self)
    }
    
    #[allow(dead_code)]
    pub fn connect_with<P>(&self, addr: &Address, param: &[P]) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::connect_with(self,addr, param)
    }
    
    #[allow(dead_code)]
    pub fn connect(&self, addr: &Address) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::connect(self, addr)
    }
    
    #[allow(dead_code)]
    pub fn accept_with<P>(&self, param: &[P]) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::accept_with(self, param)
    }
    
    #[allow(dead_code)]
    pub fn accept(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::accept(self)
    }
    
    #[allow(dead_code)]
    pub fn shutdown(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::shutdown(self, 0)
    } 
}

impl<T> Endpoint<T> {
    
    pub fn getname(&self) -> Result<Address, crate::error::Error> {
        BaseEndpointImpl::getname(self)
    }

    pub fn buffered_limit(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::buffered_limit(self)
    }

    pub fn set_buffered_limit(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_buffered_limit(self, size)
    }

    pub fn buffered_min(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::buffered_min(self)
    }

    pub fn set_buffered_min(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_buffered_min(self, size)
    }

    pub fn cm_data_size(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::cm_data_size(self)
    }

    pub fn set_cm_data_size(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_cm_data_size(self, size)
    }

    pub fn min_multi_recv(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::min_multi_recv(self)
    }

    pub fn set_min_multi_recv(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_min_multi_recv(self, size)
    }

    pub fn set_hmem_p2p(&self, hmem: HmemP2p) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_hmem_p2p(self, hmem)
    }

    pub fn hmem_p2p(&self) -> Result<HmemP2p, crate::error::Error> {
        BaseEndpointImpl::hmem_p2p(self)
    }

    pub fn xpu_trigger(&self) -> Result<libfabric_sys::fi_trigger_xpu, crate::error::Error> {
        BaseEndpointImpl::xpu_trigger(self)
    }

    pub fn set_cuda_api_permitted(&self, permitted: bool) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_cuda_api_permitted(self, permitted)
    }

    pub fn wait_fd(&self) -> Result<BorrowedFd, crate::error::Error> {
        BaseEndpointImpl::wait_fd(self)
    }

    pub fn enable(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::enable(self)
    }

    pub fn cancel(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::cancel(self)
    }

    pub fn cancel_with_context<T0>(&self, context: &mut T0) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::cancel_with_context(self, context)
    }

    pub fn rx_size_left(&self) -> Result<usize, crate::error::Error> {
        ActiveEndpointImpl::rx_size_left(self)
    }

    pub fn tx_size_left(&self) -> Result<usize, crate::error::Error> {
        ActiveEndpointImpl::tx_size_left(self)
    }

    pub fn getpeer(&self) -> Result<Address, crate::error::Error> {
        ActiveEndpointImpl::getpeer(self)
    }

    pub fn connect_with<P>(&self, addr: &Address, param: &[P]) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::connect_with(self,addr, param)
    }

    pub fn connect(&self, addr: &Address) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::connect(self, addr)
    }

    pub fn accept_with<P>(&self, param: &[P]) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::accept_with(self, param)
    }

    pub fn accept(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::accept(self)
    }

    pub fn shutdown(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::shutdown(self, 0)
    } 
}

impl<E> AsFd for Endpoint<E> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.wait_fd().unwrap()
    }
}

impl AsFd for EndpointImpl {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.wait_fd().unwrap()
    }
}

//================== Scalable Endpoint (fi_scalable_ep) ==================//
pub(crate) struct ScalableEndpointImpl {
    pub(crate) c_sep: *mut libfabric_sys::fid_ep,
    fid: OwnedFid,
    _domain_rc:  Rc<DomainImpl>
}

pub struct ScalableEndpoint<E> {
    inner: Rc<ScalableEndpointImpl>,
    phantom: PhantomData<E>,
}

impl ScalableEndpoint<()> {
    pub fn new<T0, E>(domain: &crate::domain::Domain, info: &InfoEntry<E>, context: Option<&mut T0>) -> Result<ScalableEndpoint<E>, crate::error::Error> {
        Ok(
            ScalableEndpoint::<E> { 
                inner: Rc::new( ScalableEndpointImpl::new(&domain.inner, info, context)?),
                phantom: PhantomData,
            })
    }
}

impl ScalableEndpointImpl {

    pub fn new<T0, E>(domain: &Rc<crate::domain::DomainImpl>, info: &InfoEntry<E>, context: Option<&mut T0>) -> Result<ScalableEndpointImpl, crate::error::Error> {
        let mut c_sep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_sep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_sep;
        let err = 
            if let Some(ctx) = context {
                unsafe { libfabric_sys::inlined_fi_scalable_ep(domain.handle(), info.c_info, c_sep_ptr, (ctx as *mut T0).cast()) }
            }
            else {
                unsafe { libfabric_sys::inlined_fi_scalable_ep(domain.handle(), info.c_info, c_sep_ptr, std::ptr::null_mut()) }
            };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            
            Ok(
                ScalableEndpointImpl { 
                    c_sep, 
                    fid: OwnedFid::from(unsafe{ &mut (*c_sep).fid }),
                    _domain_rc: domain.clone(), 
                })
        }
    }

    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_ep {
        self.c_sep
    }

    fn bind<T: crate::fid::AsFid>(&self, res: &T, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_scalable_ep_bind(self.handle(), res.as_fid().as_raw_fid(), flags) };
        
        check_error(err.try_into().unwrap())
    }

    // pub(crate) fn bind_av(&self, av: &Rc<AddressVectorImpl>) -> Result<(), crate::error::Error> {
    
    //     self.bind(&av, 0)
    // }

    pub(crate) fn alias(&self, flags: u64) -> Result<ScalableEndpointImpl, crate::error::Error> {
        let mut c_sep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_sep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_sep;
        let err = unsafe { libfabric_sys::inlined_fi_ep_alias(self.handle(), c_sep_ptr, flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                ScalableEndpointImpl { 
                    c_sep, 
                    fid: OwnedFid::from(unsafe{ &mut (*c_sep).fid }),
                    _domain_rc: self._domain_rc.clone(), 
                })
        }
    }
    
    #[allow(dead_code)]
    pub(crate) fn getname(&self) -> Result<Address, crate::error::Error> {
        BaseEndpointImpl::getname(self)
    }
    
    #[allow(dead_code)]
    pub(crate) fn buffered_limit(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::buffered_limit(self)
    }
    
    #[allow(dead_code)]
    pub(crate) fn set_buffered_limit(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_buffered_limit(self, size)
        
    }
    
    #[allow(dead_code)]
    pub(crate) fn buffered_min(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::buffered_min(self)
    }
    
    #[allow(dead_code)]
    pub(crate) fn set_buffered_min(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_buffered_min(self, size)
        
    }
    
    #[allow(dead_code)]
    pub(crate) fn cm_data_size(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::cm_data_size(self)
    }
    
    #[allow(dead_code)]
    pub(crate) fn set_cm_data_size(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_cm_data_size(self, size)
        
    }
    
    #[allow(dead_code)]
    pub(crate) fn min_multi_recv(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::min_multi_recv(self)
    }
    
    #[allow(dead_code)]
    pub(crate) fn set_min_multi_recv(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_min_multi_recv(self, size)
    }
    
    #[allow(dead_code)]
    pub(crate) fn set_hmem_p2p(&self, hmem: HmemP2p) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_hmem_p2p(self, hmem)
    }
    
    #[allow(dead_code)]
    pub(crate) fn hmem_p2p(&self) -> Result<HmemP2p, crate::error::Error> {
        BaseEndpointImpl::hmem_p2p(self)
    }
    
    #[allow(dead_code)]
    pub(crate) fn xpu_trigger(&self) -> Result<libfabric_sys::fi_trigger_xpu, crate::error::Error> {
        BaseEndpointImpl::xpu_trigger(self)
    }
    
    #[allow(dead_code)]
    pub(crate) fn set_cuda_api_permitted(&self, permitted: bool) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_cuda_api_permitted(self, permitted)
    }
    
    #[allow(dead_code)]
    pub(crate) fn wait_fd(&self) -> Result<BorrowedFd, crate::error::Error> {
        BaseEndpointImpl::wait_fd(self)
    }
    
    #[allow(dead_code)]
    pub(crate) fn enable(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::enable(self)
    }
    
    #[allow(dead_code)]
    pub(crate) fn cancel(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::cancel(self)
    }
    
    #[allow(dead_code)]
    pub(crate) fn cancel_with_context<T0>(&self, context: &mut T0) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::cancel_with_context(self, context)
    }
    
    #[allow(dead_code)]
    pub(crate) fn rx_size_left(&self) -> Result<usize, crate::error::Error> {
        ActiveEndpointImpl::rx_size_left(self)
    }
    
    #[allow(dead_code)]
    pub(crate) fn tx_size_left(&self) -> Result<usize, crate::error::Error> {
        ActiveEndpointImpl::tx_size_left(self)
    }
    
    #[allow(dead_code)]
    pub(crate) fn getpeer(&self) -> Result<Address, crate::error::Error> {
        ActiveEndpointImpl::getpeer(self)
    }
    
    #[allow(dead_code)]
    pub(crate) fn connect_with<T>(&self, addr: &Address, param: &[T]) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::connect_with(self,addr, param)
    }
    
    #[allow(dead_code)]
    pub(crate) fn connect(&self, addr: &Address) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::connect(self, addr)
    }
    
    #[allow(dead_code)]
    pub(crate) fn accept_with<T0>(&self, param: &[T0]) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::accept_with(self, param)
    }
    
    #[allow(dead_code)]
    pub(crate) fn accept(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::accept(self)
    }
    
    #[allow(dead_code)]
    pub(crate) fn shutdown(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::shutdown(self, 0)
    } 
}

impl<E> ScalableEndpoint<E> {

    pub fn handle(&self) -> *mut libfabric_sys::fid_ep {
        self.inner.handle()
    }

    pub fn bind_av(&self, av: &AddressVector) -> Result<(), crate::error::Error> {
        self.inner.bind(&av.inner, 0)
    }

    pub fn alias(&self, flags: u64) -> Result<ScalableEndpoint<E>, crate::error::Error> {
        Ok(
            Self {
                inner: Rc::new(self.inner.alias(flags)?),
                phantom: PhantomData,
            }
        )
    }

    pub fn getname(&self) -> Result<Address, crate::error::Error> {
        BaseEndpointImpl::getname(self)
    }

    pub fn buffered_limit(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::buffered_limit(self)
    }

    pub fn set_buffered_limit(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_buffered_limit(self, size)
    }

    pub fn buffered_min(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::buffered_min(self)
    }

    pub fn set_buffered_min(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_buffered_min(self, size)
    }

    pub fn cm_data_size(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::cm_data_size(self)
    }

    pub fn set_cm_data_size(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_cm_data_size(self, size)
    }

    pub fn min_multi_recv(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::min_multi_recv(self)
    }

    pub fn set_min_multi_recv(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_min_multi_recv(self, size)
    }

    pub fn set_hmem_p2p(&self, hmem: HmemP2p) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_hmem_p2p(self, hmem)
    }

    pub fn hmem_p2p(&self) -> Result<HmemP2p, crate::error::Error> {
        BaseEndpointImpl::hmem_p2p(self)
    }

    pub fn xpu_trigger(&self) -> Result<libfabric_sys::fi_trigger_xpu, crate::error::Error> {
        BaseEndpointImpl::xpu_trigger(self)
    }

    pub fn set_cuda_api_permitted(&self, permitted: bool) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_cuda_api_permitted(self, permitted)
    }

    pub fn wait_fd(&self) -> Result<BorrowedFd, crate::error::Error> {
        BaseEndpointImpl::wait_fd(self)
    }

    pub fn enable(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::enable(self)
    }

    pub fn cancel(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::cancel(self)
    }

    pub fn cancel_with_context<T0>(&self, context: &mut T0) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::cancel_with_context(self, context)
    }

    pub fn rx_size_left(&self) -> Result<usize, crate::error::Error> {
        ActiveEndpointImpl::rx_size_left(self)
    }

    pub fn tx_size_left(&self) -> Result<usize, crate::error::Error> {
        ActiveEndpointImpl::tx_size_left(self)
    }

    pub fn getpeer<T0>(&self) -> Result<Address, crate::error::Error> {
        ActiveEndpointImpl::getpeer(self)
    }

    pub fn connect_with<T>(&self, addr: &Address, param: &[T]) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::connect_with(self,addr, param)
    }

    pub fn connect(&self, addr: &Address) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::connect(self, addr)
    }

    pub fn accept_with<T0>(&self, param: &[T0]) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::accept_with(self, param)
    }

    pub fn accept(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::accept(self)
    }

    pub fn shutdown(&self) -> Result<(), crate::error::Error> {
        ActiveEndpointImpl::shutdown(self, 0)
    }     
}


impl AsFid for ScalableEndpointImpl {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.fid.as_fid()
    }
}
impl<E> AsFid for ScalableEndpoint<E> {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.inner.as_fid()
    }
}

impl<E> BaseEndpointImpl for ScalableEndpoint<E> {}
//     fn handle(&self) -> *mut libfabric_sys::fid_ep {
//         self.handle()
//     }
// }

impl BaseEndpointImpl for ScalableEndpointImpl {}
//     fn handle(&self) -> *mut libfabric_sys::fid_ep {
//         self.handle()
//     }
// }

impl ActiveEndpointImpl for ScalableEndpointImpl {
    fn handle(&self) -> *mut libfabric_sys::fid_ep {
        self.c_sep
    }
}

impl<E> ActiveEndpointImpl for ScalableEndpoint<E> {
    fn handle(&self) -> *mut libfabric_sys::fid_ep {
        self.inner.handle()
    }
}


// impl<'a, E> ActiveEndpoint<'a> for ScalableEndpointImpl<E> {
//     fn handle(&self) -> *mut libfabric_sys::fid_ep {
//         self.c_sep
//     }
    
//     fn inner(&self) -> Rc<dyn ActiveEndpointImpl> {
//         self.clone()
//     }
// }

impl<E> AsFd for ScalableEndpoint<E> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.wait_fd().unwrap()
    }
}

//================== Passive Endpoint (fi_passive_ep) ==================//

pub(crate) struct PassiveEndpointImpl<E> {
    pub(crate) c_pep: *mut libfabric_sys::fid_pep,
    fid: OwnedFid,
    _sync_rcs: RefCell<Vec<Rc<dyn crate::BindImpl>>>,
    phantom: PhantomData<E>,
    _fabric_rc: Rc<FabricImpl>,
}

pub struct PassiveEndpoint<E> {
    inner: Rc<PassiveEndpointImpl<E>>,
}

impl PassiveEndpoint<()> {
    pub fn new<T0, E>(fabric: &crate::fabric::Fabric, info: &InfoEntry<E>, context: Option<&mut T0>) -> Result<PassiveEndpoint<E>, crate::error::Error> {
        Ok(
            PassiveEndpoint::<E> {
                inner: 
                    Rc::new(PassiveEndpointImpl::new(&fabric.inner, info, context)?)
            }
        )
    }
}

impl PassiveEndpointImpl<()> {

    pub fn new<T0, E>(fabric: &Rc<crate::fabric::FabricImpl>, info: &InfoEntry<E>, context: Option<&mut T0>) -> Result<PassiveEndpointImpl<E>, crate::error::Error> {
        let mut c_pep: *mut libfabric_sys::fid_pep = std::ptr::null_mut();
        let c_pep_ptr: *mut *mut libfabric_sys::fid_pep = &mut c_pep;
        let err = 
            if let Some(ctx) = context {
                unsafe { libfabric_sys::inlined_fi_passive_ep(fabric.c_fabric, info.c_info, c_pep_ptr, (ctx as *mut T0).cast()) }
            }
            else {
                unsafe { libfabric_sys::inlined_fi_passive_ep(fabric.c_fabric, info.c_info, c_pep_ptr, std::ptr::null_mut()) }
            };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                PassiveEndpointImpl::<E> { 
                    c_pep, 
                    fid: OwnedFid::from(unsafe{ &mut (*c_pep).fid }),
                    _sync_rcs: RefCell::new(Vec::new()),
                    _fabric_rc: fabric.clone(),
                    phantom: PhantomData,
                })
        }
    }
}


impl<E> PassiveEndpointImpl<E> {
    
    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_pep {
        self.c_pep
    }

    pub fn bind(&self, res: &Rc<EventQueueImpl>, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_pep_bind(self.c_pep, res.as_fid().as_raw_fid(), flags) };
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            self._sync_rcs.borrow_mut().push(res.clone()); 
            Ok(())
        }
    }

    pub fn listen(&self) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_listen(self.handle())};
        
        check_error(err.try_into().unwrap())
    }

    pub fn reject<T0>(&self, fid: &impl AsFid, params: &[T0]) -> Result<(), crate::error::Error> {
        let err = unsafe {libfabric_sys::inlined_fi_reject(self.handle(), fid.as_fid().as_raw_fid(), params.as_ptr().cast(), params.len())};

        check_error(err.try_into().unwrap())

    }

    pub fn set_backlog_size(&self, size: i32) -> Result<(), crate::error::Error> {
        let err = unsafe{ libfabric_sys::inlined_fi_control(self.as_fid().as_raw_fid(), FI_BACKLOG as i32, (&mut size.clone() as *mut i32).cast())};
        check_error(err.try_into().unwrap())
    }

    #[allow(dead_code)]
    pub fn buffered_limit(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::buffered_limit(self)
    }
    
    #[allow(dead_code)]
    pub fn set_buffered_limit(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_buffered_limit(self, size)
        
    }
    
    #[allow(dead_code)]
    pub fn buffered_min(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::buffered_min(self)
    }
    
    #[allow(dead_code)]
    pub fn set_buffered_min(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_buffered_min(self, size)
        
    }
    
    #[allow(dead_code)]
    pub fn cm_data_size(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::cm_data_size(self)
    }
    
    #[allow(dead_code)]
    pub fn set_cm_data_size(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_cm_data_size(self, size)
        
    }
    
    #[allow(dead_code)]
    pub fn min_multi_recv(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::min_multi_recv(self)
    }
    
    #[allow(dead_code)]
    pub fn set_min_multi_recv(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_min_multi_recv(self, size)
    }
    
    #[allow(dead_code)]
    pub fn set_hmem_p2p(&self, hmem: HmemP2p) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_hmem_p2p(self, hmem)
    }
    
    #[allow(dead_code)]
    pub fn hmem_p2p(&self) -> Result<HmemP2p, crate::error::Error> {
        BaseEndpointImpl::hmem_p2p(self)
    }
    
    #[allow(dead_code)]
    pub fn xpu_trigger(&self) -> Result<libfabric_sys::fi_trigger_xpu, crate::error::Error> {
        BaseEndpointImpl::xpu_trigger(self)
    }
    
    #[allow(dead_code)]
    pub fn set_cuda_api_permitted(&self, permitted: bool) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_cuda_api_permitted(self, permitted)
    }
    
    pub fn wait_fd(&self) -> Result<BorrowedFd, crate::error::Error> {
        BaseEndpointImpl::wait_fd(self)
    }

}

impl<E> PassiveEndpoint<E> {
    
    #[allow(dead_code)]
    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_pep {
        self.inner.handle()
    }

    pub fn bind<T: EqConfig + 'static>(&self, res: &EventQueue<T>, flags: u64) -> Result<(), crate::error::Error> {
        self.inner.bind(&res.inner, flags)
    }

    pub fn listen(&self) -> Result<(), crate::error::Error> {
        self.inner.listen()
    }

    pub fn reject<T0>(&self, fid: &impl AsFid, params: &[T0]) -> Result<(), crate::error::Error> {
        self.inner.reject(fid, params)
    }

    pub fn set_backlog_size(&self, size: i32) -> Result<(), crate::error::Error> {
        self.inner.set_backlog_size(size)
    }

    pub fn buffered_limit(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::buffered_limit(self)
    }

    pub fn set_buffered_limit(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_buffered_limit(self, size)

    }

    pub fn buffered_min(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::buffered_min(self)
    }

    pub fn set_buffered_min(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_buffered_min(self, size)

    }

    pub fn cm_data_size(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::cm_data_size(self)
    }

    pub fn set_cm_data_size(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_cm_data_size(self, size)

    }

    pub fn min_multi_recv(&self) -> Result<usize, crate::error::Error> {
        BaseEndpointImpl::min_multi_recv(self)
    }

    pub fn set_min_multi_recv(&self, size: usize) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_min_multi_recv(self, size)
    }

    pub fn set_hmem_p2p(&self, hmem: HmemP2p) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_hmem_p2p(self, hmem)
    }

    pub fn hmem_p2p(&self) -> Result<HmemP2p, crate::error::Error> {
        BaseEndpointImpl::hmem_p2p(self)
    }

    pub fn xpu_trigger(&self) -> Result<libfabric_sys::fi_trigger_xpu, crate::error::Error> {
        BaseEndpointImpl::xpu_trigger(self)
    }

    pub fn set_cuda_api_permitted(&self, permitted: bool) -> Result<(), crate::error::Error> {
        BaseEndpointImpl::set_cuda_api_permitted(self, permitted)
    }
    
    pub fn wait_fd(&self) -> Result<BorrowedFd, crate::error::Error> {
        BaseEndpointImpl::wait_fd(self)
    }
}

impl<E> BaseEndpointImpl for PassiveEndpoint<E> {}
//     fn handle(&self) -> *mut libfabric_sys::fid_ep {
//         self.handle()
//     }
// }

impl<E> BaseEndpointImpl for PassiveEndpointImpl<E> {}
//     fn handle(&self) -> *mut libfabric_sys::fid_ep {
//         self.handle()
//     }
// }

impl<E> AsFid for PassiveEndpointImpl<E> {
    fn as_fid(&self) -> fid::BorrowedFid {
        self.fid.as_fid()
    }    
}

impl<E> AsFid for PassiveEndpoint<E> {
    fn as_fid(&self) -> fid::BorrowedFid {
        self.inner.as_fid()
    }    
}

impl<E> AsFd for PassiveEndpointImpl<E> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.wait_fd().unwrap()
    }
}

impl<E> AsFd for PassiveEndpoint<E> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.wait_fd().unwrap()
    }
}

//================== Endpoint (fi_endpoint) ==================//

pub struct IncompleteBindCq<'a> {
    pub(crate) ep: &'a EndpointImpl,
    pub(crate) flags: u64,
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

    pub fn cq<T: CqConfig + 'static, Format: crate::cq::EntryFormat + 'static>(&mut self, cq: &crate::cq::CompletionQueue<T, Format>) -> Result<(), crate::error::Error> {
        self.ep.bind(cq, self.flags)
    }
}

// impl Drop for PassiveEndpointImpl {
//     fn drop(&mut self) {
//        println!("Dropping PassiveEndpoint\n");
//     }
// }

// impl Drop for EndpointImpl {
//     fn drop(&mut self) {
//         println!("Dropping Endpoint\n");
//     }
// }

// impl Drop for ScalableEndpointImpl {
//     fn drop(&mut self) {
//         println!("Dropping ScalableEndpointImpl\n");
//     }
// }


pub struct IncompleteBindCntr<'a> {
    pub(crate) ep: &'a EndpointImpl,
    pub(crate) flags: u64,
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

    pub fn cntr<T: crate::cntroptions::CntrConfig + 'static>(&mut self, cntr: &Counter<T>) -> Result<(), crate::error::Error> {
        self.ep.bind(cntr, self.flags)
    }
}

impl EndpointImpl {

    pub fn new<T0, E>(domain: &Rc<crate::domain::DomainImpl>, info: &InfoEntry<E>, flags: u64, context: Option<&mut T0>) -> Result< EndpointImpl, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err =
            if let Some(ctx) = context {
                unsafe { libfabric_sys::inlined_fi_endpoint2(domain.handle(), info.c_info, c_ep_ptr, flags, (ctx as *mut T0).cast()) }
            } 
            else {
                unsafe { libfabric_sys::inlined_fi_endpoint2(domain.handle(), info.c_info, c_ep_ptr, flags, std::ptr::null_mut()) }
            };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { 
                    c_ep, 
                    fid: OwnedFid::from(unsafe{ &mut (*c_ep).fid }),
                    _sync_rcs: RefCell::new(Vec::new()),
                    bound_eq: OnceCell::new(),
                    _domain_rc: domain.clone(),
                })
        }
    }
}

impl Endpoint<()> {
    pub fn new<T0, E>(domain: &crate::domain::Domain, info: &InfoEntry<E>, flags: u64, context: Option<&mut T0>) -> Result< Endpoint<E>, crate::error::Error> {
        Ok(
            Endpoint::<E> {
                inner:Rc::new(EndpointImpl::new(&domain.inner, info, flags, context)?),
                phantom: PhantomData,
            }
        )
    }
}

// pub(crate) fn from_attr(domain: &crate::domain::Domain, mut rx_attr: crate::RxAttr) -> Result<Self, crate::error::Error> {
    //     let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
    //     let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
    //     let err = unsafe { libfabric_sys::inlined_fi_srx_context(domain.handle(), rx_attr.get_mut(), c_ep_ptr,  std::ptr::null_mut()) };

    //     if err != 0 {
    //         Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
    //     }
    //     else {
    //         Ok(
    //             Self { c_ep, fid: OwnedFid { fid: unsafe{ &mut (*c_ep).fid } } }        
    //         )
    //     }

    // }

    // pub(crate) fn from_attr_with_context<T0>(domain: &crate::domain::Domain, mut rx_attr: crate::RxAttr, context: &mut T0) -> Result<Self, crate::error::Error> {
    //     let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
    //     let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
    //     let err = unsafe { libfabric_sys::inlined_fi_srx_context(domain.handle(), rx_attr.get_mut(), c_ep_ptr, context as *mut T0 as *mut std::ffi::c_void) };

    //     if err != 0 {
    //         Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
    //     }
    //     else {
    //         Ok(
    //             Self { c_ep, fid: OwnedFid { fid: unsafe{ &mut (*c_ep).fid } } }        
    //         )
    //     }

    // }
impl EndpointImpl {

    pub(crate) fn bind<T: crate::Bind + AsFid>(&self, res: &T, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_ep_bind(self.handle(), res.as_raw_fid(), flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            self._sync_rcs.borrow_mut().push(res.inner().clone()); //  [TODO] Do this with inner mutability.
            Ok(())
        }
    } 

    pub(crate) fn bind_eq_(&self, res: &Rc<EventQueueImpl>, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_ep_bind(self.handle(), res.as_raw_fid(), flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            if self.bound_eq.set(res.clone()).is_err() {
                panic!("Endpoint is already bound to an EventQueue");
            }
            Ok(())
        }
    } 

    pub(crate) fn bind_cq(&self) -> IncompleteBindCq {
        IncompleteBindCq { ep: self, flags: 0}
    }

    pub(crate) fn bind_cntr(&self) -> IncompleteBindCntr {
        IncompleteBindCntr { ep: self, flags: 0}
    }

    pub(crate) fn bind_eq<T: EqConfig + 'static>(&self, eq: &EventQueue<T>) -> Result<(), crate::error::Error>  {
        
        self.bind_eq_(&eq.inner, 0)
    }

    pub(crate) fn bind_av(&self, av: &AddressVector) -> Result<(), crate::error::Error> {
    
        self.bind(av, 0)
    }

    pub(crate) fn alias(&self, flags: u64) -> Result<EndpointImpl, crate::error::Error> {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err = unsafe { libfabric_sys::inlined_fi_ep_alias(self.handle(), c_ep_ptr, flags) };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { 
                    c_ep, 
                    fid: OwnedFid::from(unsafe{ &mut (*c_ep).fid }),
                    _sync_rcs: RefCell::new(Vec::new()),
                    bound_eq: OnceCell::new(),
                    _domain_rc: self._domain_rc.clone(),
                })
        }
    }
}

impl<E> Endpoint<E> {
    pub fn bind_cq(&self) -> IncompleteBindCq {
        self.inner.bind_cq()
    }

    pub fn bind_cntr(&self) -> IncompleteBindCntr {
        self.inner.bind_cntr()
    }
        
    pub fn bind_eq<T: EqConfig + 'static>(&self, eq: &EventQueue<T>) -> Result<(), crate::error::Error>  {
        self.inner.bind_eq(eq)
    }

    pub fn bind_av(&self, av: &AddressVector) -> Result<(), crate::error::Error> {
        self.inner.bind_av(av)
    }

    pub fn alias(&self, flags: u64) -> Result<Endpoint<E>, crate::error::Error> {
        Ok(
            Self {
                inner: Rc::new (self.inner.alias(flags)?),
                phantom: PhantomData,
            }
        )
    }
}


impl<E> AsFid for Endpoint<E> {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.inner.as_fid().clone()
    }
}

impl AsFid for EndpointImpl {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.fid.as_fid().clone()
    }
}

impl ActiveEndpointImpl for EndpointImpl {
    fn handle(&self) -> *mut libfabric_sys::fid_ep {
        self.c_ep
    }
}

// impl<E> ActiveEndpointImpl for Endpoint<E> {
//     fn handle(&self) -> *mut libfabric_sys::fid_ep {
//         self.inner.handle()
//     }
// }
// impl<'a, E> ActiveEndpoint<'a> for EndpointImpl<E> {
//     fn inner(&'a self) -> Rc<dyn BaseEndpointImpl + 'a> {
//         self.inner()
//     }
// }

// pub trait ActiveEndpoint<'a>: ActiveEndpointImpl {
//     fn inner(&'a self) -> Rc<dyn BaseEndpointImpl + 'a>;
// }


pub trait ActiveEndpointImpl: AsFid{
    
    fn handle(&self) -> *mut libfabric_sys::fid_ep;

    fn enable(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_enable(self.handle()) };
        
        check_error(err.try_into().unwrap())
    }

    fn cancel(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cancel(self.as_fid().as_raw_fid(), std::ptr::null_mut()) };
        
        check_error(err)
    }

    fn cancel_with_context<T0>(&self, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cancel(self.as_fid().as_raw_fid(), (context as *mut T0).cast()) };
        
        check_error(err)
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

    fn getpeer(&self) -> Result<Address, crate::error::Error> {
        let mut len = 0;
        let err = unsafe { libfabric_sys::inlined_fi_getpeer(self.handle(), std::ptr::null_mut(), &mut len)};
        
        if -err as u32 ==  libfabric_sys::FI_ETOOSMALL{
            let mut address = vec![0; len];
            let err = unsafe { libfabric_sys::inlined_fi_getpeer(self.handle(), address.as_mut_ptr().cast(), &mut len)};
            if err != 0 {
                Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
            }
            else {
                Ok(Address{address})
            }
        }
        else {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
    }

    fn connect_with<T>(&self, addr: &Address, param: &[T]) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_connect(self.handle(), addr.as_bytes().as_ptr().cast(), param.as_ptr().cast(), param.len()) };
        
        check_error(err.try_into().unwrap())
    }

    fn connect(&self, addr: &Address) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_connect(self.handle(), addr.as_bytes().as_ptr().cast(), std::ptr::null_mut(), 0) };

        check_error(err.try_into().unwrap())
    }

    fn accept_with<T0>(&self, param: &[T0]) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_accept(self.handle(), param.as_ptr().cast(), param.len()) };
        
        check_error(err.try_into().unwrap())
    }

    fn accept(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_accept(self.handle(), std::ptr::null_mut(), 0) };
        
        check_error(err.try_into().unwrap())
    }

    fn shutdown(&self, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_shutdown(self.handle(), flags) };

        check_error(err.try_into().unwrap())
    }

    fn transmit_options(&self) -> Result<TransferOptions, crate::error::Error> {
        let mut ops = libfabric_sys::FI_TRANSMIT;
        let err = unsafe{ inlined_fi_control(self.as_fid().as_raw_fid(), FI_GETOPSFLAG as i32, (&mut ops as *mut u32).cast())}; 

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(TransferOptions::from_value(ops))
        }
    }

    fn receive_options(&self) -> Result<TransferOptions, crate::error::Error> {
        let mut ops = libfabric_sys::FI_RECV;
        let err = unsafe{ inlined_fi_control(self.as_fid().as_raw_fid(), FI_GETOPSFLAG as i32, (&mut ops as *mut u32).cast())}; 

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(TransferOptions::from_value(ops))
        }
    }

    fn set_transmit_options(&self, ops: TransferOptions) -> Result<(), crate::error::Error> {

        ops.transmit();
        let err = unsafe{ inlined_fi_control(self.as_fid().as_raw_fid(), libfabric_sys::FI_SETOPSFLAG as i32, (&mut ops.get_value() as *mut u32).cast())}; 

        check_error(err.try_into().unwrap())
    }

    fn set_receive_options(&self, ops: TransferOptions) -> Result<(), crate::error::Error> {
        
        ops.recv();
        let err = unsafe{ inlined_fi_control(self.as_fid().as_raw_fid(), libfabric_sys::FI_SETOPSFLAG as i32, (&mut ops.get_value() as *mut u32).cast())}; 

        check_error(err.try_into().unwrap())
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

    pub fn ep_type(&mut self, type_: crate::enums::EndpointType) -> &mut Self {

        self.c_attr.type_ = type_.get_value();
        self
    }

    pub fn protocol(&mut self, proto: crate::enums::Protocol) -> &mut Self {

        self.c_attr.protocol = proto.get_value();
        self
    }

    pub fn max_msg_size(&mut self, size: usize) -> &mut Self {

        self.c_attr.max_msg_size = size;
        self
    }

    pub fn msg_prefix_size(&mut self, size: usize) -> &mut Self {

        self.c_attr.msg_prefix_size = size;
        self
    }

    pub fn max_order_raw_size(&mut self, size: usize) -> &mut Self {

        self.c_attr.max_order_raw_size = size;
        self
    }

    pub fn max_order_war_size(&mut self, size: usize) -> &mut Self {

        self.c_attr.max_order_war_size = size;
        self
    }

    pub fn max_order_waw_size(&mut self, size: usize) -> &mut Self {

        self.c_attr.max_order_waw_size = size;
        self
    }

    pub fn mem_tag_format(&mut self, tag: u64) -> &mut Self {

        self.c_attr.mem_tag_format = tag;
        self
    }

    pub fn tx_ctx_cnt(&mut self, size: usize) -> &mut Self {

        self.c_attr.tx_ctx_cnt = size;
        self
    }

    pub fn rx_ctx_cnt(&mut self, size: usize) -> &mut Self {

        self.c_attr.rx_ctx_cnt = size;
        self
    }

    pub fn auth_key(&mut self, key: &mut [u8]) -> &mut Self {

        self.c_attr.auth_key_size = key.len();
        self.c_attr.auth_key = key.as_mut_ptr();
        self
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

pub struct EndpointBuilder<'a, T, E> {
    ep_attr: EndpointAttr,
    flags: u64,
    info: &'a InfoEntry<E>,
    ctx: Option<&'a mut T>,
}

impl<'a> EndpointBuilder<'a, (), ()> {

    pub fn new<E>(info: &'a InfoEntry<E>, ) -> EndpointBuilder<'a, (), E> {
        EndpointBuilder::<(), E> {
            ep_attr: EndpointAttr::new(),
            flags: 0,
            info,
            ctx: None,
        }
    }
}

impl<'a, E> EndpointBuilder<'a, (), E> {

    pub fn build(self, domain: &crate::domain::Domain) -> Result<Endpoint<E>, crate::error::Error> {
        Endpoint::new(domain, self.info, self.flags, self.ctx)
    }

    pub fn build_scalable(self, domain: &crate::domain::Domain) -> Result<ScalableEndpoint<E>, crate::error::Error> {
        ScalableEndpoint::new(domain, self.info, self.ctx)
    }

    pub fn build_passive(self, fabric: &crate::fabric::Fabric) -> Result<PassiveEndpoint<E>, crate::error::Error> {
        PassiveEndpoint::new(fabric, self.info, self.ctx)
    }

    // pub(crate) fn from(c_ep_attr: *mut libfabric_sys::fi_ep_attr) -> Self {
    //     let c_attr = unsafe { *c_ep_attr };

    //     Self { c_attr }
    // }

    pub fn flags(mut self, flags: u64) -> Self {
        self.flags = flags;
        self
    }

    pub fn ep_type(mut self, type_: crate::enums::EndpointType) -> Self {

        self.ep_attr.ep_type(type_);
        self
    }

    pub fn protocol(mut self, proto: crate::enums::Protocol) -> Self{
        
        self.ep_attr.protocol(proto);
        self
    }

    pub fn max_msg_size(mut self, size: usize) -> Self {

        self.ep_attr.max_msg_size(size);
        self
    }

    pub fn msg_prefix_size(mut self, size: usize) -> Self {

        self.ep_attr.msg_prefix_size(size);
        self
    }

    pub fn max_order_raw_size(mut self, size: usize) -> Self {

        self.ep_attr.max_order_raw_size(size);
        self
    }

    pub fn max_order_war_size(mut self, size: usize) -> Self {

        self.ep_attr.max_order_war_size(size);
        self
    }

    pub fn max_order_waw_size(mut self, size: usize) -> Self {

        self.ep_attr.max_order_waw_size(size);
        self
    }

    pub fn mem_tag_format(mut self, tag: u64) -> Self {

        self.ep_attr.mem_tag_format(tag);
        self
    }

    pub fn tx_ctx_cnt(mut self, size: usize) -> Self {

        self.ep_attr.tx_ctx_cnt(size);
        self
    }

    pub fn rx_ctx_cnt(mut self, size: usize) -> Self {

        self.ep_attr.rx_ctx_cnt(size);
        self
    }

    pub fn auth_key(mut self, key: &mut [u8]) -> Self {

        self.ep_attr.auth_key(key);
        self
    }
}