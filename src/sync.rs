use std::{os::fd::BorrowedFd, rc::Rc};

use crate::{enums::{self, WaitObjType2}, fabric::FabricImpl, utils::check_error, fid::{self, AsRawFid}};
use crate::fid::{OwnedFid, AsFid};
// impl Drop for WaitSet {
//     fn drop(&mut self) {
//        println!("Dropping WaitSet\n");
//     }
// }

// impl Drop for PollSet {
//     fn drop(&mut self) {
//        println!("Dropping PollSet\n");
//     }
// }


//================== Wait (fi_wait) ==================//

pub struct WaitSetBuilder<'a> {
    wait_attr : WaitSetAttr,
    fabric: &'a crate::fabric::Fabric,
}

impl<'a> WaitSetBuilder<'a> {
    pub fn new(fabric: &'a crate::fabric::Fabric) -> Self {
        WaitSetBuilder {
            wait_attr: WaitSetAttr::new(),
            fabric,
        }
    }

    pub fn wait_obj(mut self, wait_obj: enums::WaitObj2) -> Self {
        self.wait_attr.wait_obj(wait_obj);
        self
    }

    pub fn build(self) -> Result<WaitSet, crate::error::Error> {
        WaitSet::new(self.fabric, self.wait_attr)
    }
}

pub(crate) struct WaitSetImpl {
    pub(crate) c_wait: *mut libfabric_sys::fid_wait,
    fid: OwnedFid,
    _fabric_rc: Rc<FabricImpl>,
}

pub struct WaitSet {
    inner: Rc<WaitSetImpl>,
}

impl WaitSetImpl {
    
    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_wait {
        self.c_wait
    }

    pub(crate) fn new(fabric: &crate::fabric::Fabric, mut attr: WaitSetAttr) -> Result<Self, crate::error::Error> {
        let mut c_wait: *mut libfabric_sys::fid_wait  = std::ptr::null_mut();
        let c_wait_ptr: *mut *mut libfabric_sys::fid_wait = &mut c_wait;

        let err = unsafe {libfabric_sys::inlined_fi_wait_open(fabric.handle(), attr.get_mut(), c_wait_ptr)};
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    c_wait, 
                    fid: OwnedFid::from(unsafe{ &mut (*c_wait).fid}),
                    _fabric_rc: fabric.inner.clone(), 
                })
        }
    }

    pub(crate) fn wait(&self, timeout: i32) -> Result<(), crate::error::Error> { 
        let err = unsafe { libfabric_sys::inlined_fi_wait(self.handle(), timeout) };

        check_error(err.try_into().unwrap())
    }

    pub(crate) fn wait_object(&self) -> Result<WaitObjType2, crate::error::Error> {
        let mut res: libfabric_sys::fi_wait_obj = 0;
        let err = unsafe{libfabric_sys::inlined_fi_control(self.as_raw_fid(), libfabric_sys::FI_GETWAITOBJ as i32, (&mut res as *mut libfabric_sys::fi_wait_obj).cast() )};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            let ret = 
            if res == libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC {
                WaitObjType2::Unspec
            }
            else if res == libfabric_sys::fi_wait_obj_FI_WAIT_FD {
                let mut fd = 0; 
                let err = unsafe{libfabric_sys::inlined_fi_control(self.as_raw_fid(), libfabric_sys::FI_GETWAIT as i32, (&mut fd as *mut i32).cast() )};
                if err != 0 {
                    return Err(crate::error::Error::from_err_code((-err).try_into().unwrap()));
                }
                WaitObjType2::Fd(unsafe{BorrowedFd::borrow_raw(fd)})
            }
            else if res == libfabric_sys::fi_wait_obj_FI_WAIT_MUTEX_COND {
                let mut cond: libfabric_sys::fi_mutex_cond = libfabric_sys::fi_mutex_cond {
                    mutex: std::ptr::null_mut(),
                    cond: std::ptr::null_mut(),
                }; 
                let err = unsafe{libfabric_sys::inlined_fi_control(self.as_raw_fid(), libfabric_sys::FI_GETWAIT as i32, (&mut cond as *mut libfabric_sys::fi_mutex_cond).cast() )};
                if err != 0 {
                    return Err(crate::error::Error::from_err_code((-err).try_into().unwrap()));
                }
                WaitObjType2::MutexCond(cond)
            }
            else if res == libfabric_sys::fi_wait_obj_FI_WAIT_YIELD {
                WaitObjType2::Yield
            }
            else if res == libfabric_sys::fi_wait_obj_FI_WAIT_POLLFD {
                let mut wait: libfabric_sys::fi_wait_pollfd = libfabric_sys::fi_wait_pollfd {
                    change_index: 0,
                    nfds: 0,
                    fd: std::ptr::null_mut(),
                }; 
                let err = unsafe{libfabric_sys::inlined_fi_control(self.as_raw_fid(), libfabric_sys::FI_GETWAIT as i32, (&mut wait as *mut libfabric_sys::fi_wait_pollfd).cast() )};
                if err != 0 {
                    return Err(crate::error::Error::from_err_code((-err).try_into().unwrap()));
                }
                WaitObjType2::PollFd(wait)
            }
            else {
                panic!("Unexpected waitobject type")
            };
            Ok(ret)
        }
    }
}


impl WaitSet {
        
    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_wait {
        self.inner.handle()
    }

    pub(crate) fn new(fabric: &crate::fabric::Fabric, attr: WaitSetAttr) -> Result<Self, crate::error::Error> {
        Ok (
            Self {
                inner: 
                    Rc::new(WaitSetImpl::new(fabric, attr)?)
            }
        )
    }

    pub fn wait(&self, timeout: i32) -> Result<(), crate::error::Error> { 
        self.inner.wait(timeout)
    }

    pub fn wait_object(&self) -> Result<WaitObjType2, crate::error::Error> {
        self.inner.wait_object()
    }
}


impl AsFid for WaitSetImpl {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.fid.as_fid()
    }
}

impl AsFid for WaitSet {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.inner.as_fid()
    }
}

//================== Wait attribute ==================//

pub(crate) struct WaitSetAttr {
    pub(crate) c_attr: libfabric_sys::fi_wait_attr,
}

impl WaitSetAttr {

    pub(crate) fn new () -> Self {
        Self {
            c_attr: libfabric_sys::fi_wait_attr {
                wait_obj: libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC,
                flags: 0,
            }
        }
    }
    
    pub(crate) fn wait_obj(&mut self, wait_obj: enums::WaitObj2) -> &mut Self {
        self.c_attr.wait_obj = wait_obj.get_value();
        self
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_wait_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_wait_attr {
        &mut self.c_attr
    }   
}

//================== Poll (fi_poll) ==================//


pub struct PollSetBuilder<'a> {
    poll_attr : PollSetAttr,
    domain: &'a crate::domain::Domain,
}

impl<'a> PollSetBuilder<'a> {
    pub fn new(domain: &'a crate::domain::Domain) -> Self {
        PollSetBuilder {
            poll_attr: PollSetAttr::new(),
            domain,
        }
    }

    pub fn build(self) -> Result<PollSet, crate::error::Error> {
        PollSet::new(self.domain, self.poll_attr)
    }
}

pub(crate) struct PollSetImpl {
    pub(crate) c_poll: *mut libfabric_sys::fid_poll,
    fid: OwnedFid,
}

pub struct PollSet {
    inner: Rc<PollSetImpl>
}

impl PollSetImpl {

    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_poll {
        self.c_poll
    }

    pub(crate) fn new(domain: &crate::domain::Domain, mut attr: crate::sync::PollSetAttr) -> Result<Self, crate::error::Error> {
        let mut c_poll: *mut libfabric_sys::fid_poll = std::ptr::null_mut();
        let c_poll_ptr: *mut *mut libfabric_sys::fid_poll = &mut c_poll;
        let err = unsafe { libfabric_sys::inlined_fi_poll_open(domain.handle(), attr.get_mut(), c_poll_ptr) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    c_poll, 
                    fid: OwnedFid::from(unsafe{ &mut (*c_poll).fid }), 
                })
        }
    }

    pub(crate) fn poll<T0>(&self, contexts: &mut [T0]) -> Result<usize, crate::error::Error> {
        let ret = unsafe { libfabric_sys::inlined_fi_poll(self.handle(), contexts.as_mut_ptr().cast(),  contexts.len() as i32) };
        
        if ret < 0{
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()))
        }
        else {
            Ok(ret as usize)
        }
    }

    pub(crate) fn add(&self, fid: &impl AsFid, flags:u64) -> Result<(), crate::error::Error> { //[TODO] fid should implement Waitable trait
        let err = unsafe { libfabric_sys::inlined_fi_poll_add(self.handle(), fid.as_fid().as_raw_fid(), flags) };

        check_error(err.try_into().unwrap())
    }

    pub(crate) fn del(&self, fid: &impl AsFid, flags:u64) -> Result<(), crate::error::Error> { //[TODO] fid should implement Waitable trait
        let err = unsafe { libfabric_sys::inlined_fi_poll_del(self.handle(), fid.as_fid().as_raw_fid(), flags) };

        check_error(err.try_into().unwrap())
    }

    pub(crate) fn wait_object(&self) -> Result<WaitObjType2, crate::error::Error> {
        let mut res: libfabric_sys::fi_wait_obj = 0;
        let err = unsafe{libfabric_sys::inlined_fi_control(self.as_fid().as_raw_fid(), libfabric_sys::FI_GETWAITOBJ as i32, (&mut res as *mut libfabric_sys::fi_wait_obj).cast())};
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            let ret = 
            if res == libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC {
                WaitObjType2::Unspec
            }
            else if res == libfabric_sys::fi_wait_obj_FI_WAIT_FD {
                let mut fd = 0; 
                let err = unsafe{libfabric_sys::inlined_fi_control(self.as_fid().as_raw_fid(), libfabric_sys::FI_GETWAIT as i32, (&mut fd as *mut i32).cast())};
                if err != 0 {
                    return Err(crate::error::Error::from_err_code((-err).try_into().unwrap()));
                }
                WaitObjType2::Fd(unsafe{BorrowedFd::borrow_raw(fd)})
            }
            else if res == libfabric_sys::fi_wait_obj_FI_WAIT_MUTEX_COND {
                let mut cond: libfabric_sys::fi_mutex_cond = libfabric_sys::fi_mutex_cond {
                    mutex: std::ptr::null_mut(),
                    cond: std::ptr::null_mut(),
                }; 
                let err = unsafe{libfabric_sys::inlined_fi_control(self.as_fid().as_raw_fid(), libfabric_sys::FI_GETWAIT as i32, (&mut cond as *mut libfabric_sys::fi_mutex_cond).cast())};
                if err != 0 {
                    return Err(crate::error::Error::from_err_code((-err).try_into().unwrap()));
                }
                WaitObjType2::MutexCond(cond)
            }
            else if res == libfabric_sys::fi_wait_obj_FI_WAIT_YIELD {
                WaitObjType2::Yield
            }
            else if res == libfabric_sys::fi_wait_obj_FI_WAIT_POLLFD {
                let mut wait: libfabric_sys::fi_wait_pollfd = libfabric_sys::fi_wait_pollfd {
                    change_index: 0,
                    nfds: 0,
                    fd: std::ptr::null_mut(),
                }; 
                let err = unsafe{libfabric_sys::inlined_fi_control(self.as_fid().as_raw_fid(), libfabric_sys::FI_GETWAIT as i32, (&mut wait as *mut libfabric_sys::fi_wait_pollfd).cast())};
                if err != 0 {
                    return Err(crate::error::Error::from_err_code((-err).try_into().unwrap()));
                }
                WaitObjType2::PollFd(wait)
            }
            else {
                panic!("Unexpected waitobject type")
            };
            Ok(ret)
        }
    }
}

impl PollSet {
    
    #[allow(dead_code)]
    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_poll {
        self.inner.handle()
    }

    pub(crate) fn new(domain: &crate::domain::Domain, attr: crate::sync::PollSetAttr) -> Result<Self, crate::error::Error> {
        Ok(
            Self {
                inner: 
                    Rc::new(PollSetImpl::new(domain, attr)?)
            }
        )
    }

    pub fn poll<T0>(&self, contexts: &mut [T0]) -> Result<usize, crate::error::Error> {
        self.inner.poll(contexts)
    }


    pub fn add(&self, fid: &impl AsFid, flags:u64) -> Result<(), crate::error::Error> { //[TODO] fid should implement Waitable trait
        self.inner.add(fid, flags)
    }

    pub fn del(&self, fid: &impl AsFid, flags:u64) -> Result<(), crate::error::Error> { //[TODO] fid should implement Waitable trait
        self.inner.del(fid, flags)
    }

    pub fn wait_object(&self) -> Result<WaitObjType2, crate::error::Error> {
        self.inner.wait_object()
    }
}

impl AsFid for PollSetImpl {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.fid.as_fid()
    }
}

impl AsFid for PollSet {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.inner.as_fid()
    }
}

//================== Poll attribute ==================//

pub struct PollSetAttr {
    pub(crate) c_attr: libfabric_sys::fi_poll_attr,
}

impl PollSetAttr {
    pub(crate) fn new() -> Self {
        Self {
            c_attr: libfabric_sys::fi_poll_attr {
                flags: 0,
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_poll_attr {
        &self.c_attr
    }   

    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_poll_attr {
        &mut self.c_attr
    }      
}