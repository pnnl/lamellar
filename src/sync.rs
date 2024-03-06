use std::os::fd::BorrowedFd;

use crate::{enums::{self, WaitObjType2}, AsFid};


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

pub struct WaitSet {
    pub(crate) c_wait: *mut libfabric_sys::fid_wait,
    fid: crate::OwnedFid,
}

impl WaitSet {
    
    pub(crate) fn new(fabric: &crate::fabric::Fabric, mut attr: WaitSetAttr) -> Result<Self, crate::error::Error> {
        let mut c_wait: *mut libfabric_sys::fid_wait  = std::ptr::null_mut();
        let c_wait_ptr: *mut *mut libfabric_sys::fid_wait = &mut c_wait;

        let err = unsafe {libfabric_sys::inlined_fi_wait_open(fabric.c_fabric, attr.get_mut(), c_wait_ptr)};
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_wait, fid: crate::OwnedFid{fid: unsafe{ &mut (*c_wait).fid} } }        
            )
        }
    }

    pub fn wait(&self, timeout: i32) -> Result<(), crate::error::Error> { 
        let err = unsafe { libfabric_sys::inlined_fi_wait(self.c_wait, timeout) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn wait_object(&self) -> Result<WaitObjType2, crate::error::Error> {
        let mut res: libfabric_sys::fi_wait_obj = 0;
        let err = unsafe{libfabric_sys::inlined_fi_control(self.as_fid(), libfabric_sys::FI_GETWAITOBJ as i32, &mut res as *mut libfabric_sys::fi_wait_obj as *mut std::ffi::c_void )};
        
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
                let err = unsafe{libfabric_sys::inlined_fi_control(self.as_fid(), libfabric_sys::FI_GETWAIT as i32, &mut fd as *mut i32 as *mut std::ffi::c_void )};
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
                let err = unsafe{libfabric_sys::inlined_fi_control(self.as_fid(), libfabric_sys::FI_GETWAIT as i32, &mut cond as *mut libfabric_sys::fi_mutex_cond as *mut std::ffi::c_void )};
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
                let err = unsafe{libfabric_sys::inlined_fi_control(self.as_fid(), libfabric_sys::FI_GETWAIT as i32, &mut wait as *mut libfabric_sys::fi_wait_pollfd as *mut std::ffi::c_void )};
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

impl AsFid for WaitSet {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
        self.fid.as_fid()
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

pub struct PollSet {
    pub(crate) c_poll: *mut libfabric_sys::fid_poll,
    fid: crate::OwnedFid,
}

impl PollSet {
    pub(crate) fn new(domain: &crate::domain::Domain, mut attr: crate::sync::PollSetAttr) -> Result<PollSet, crate::error::Error> {
        let mut c_poll: *mut libfabric_sys::fid_poll = std::ptr::null_mut();
        let c_poll_ptr: *mut *mut libfabric_sys::fid_poll = &mut c_poll;
        let err = unsafe { libfabric_sys::inlined_fi_poll_open(domain.c_domain, attr.get_mut(), c_poll_ptr) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_poll, fid: crate::OwnedFid {fid: unsafe{ &mut (*c_poll).fid } } }
            )
        }
    }

    pub fn poll<T0>(&self, contexts: &mut [T0]) -> Result<usize, crate::error::Error> {
        let ret = unsafe { libfabric_sys::inlined_fi_poll(self.c_poll, contexts.as_mut_ptr() as *mut *mut std::ffi::c_void,  contexts.len() as i32) };
        
        if ret < 0{
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()))
        }
        else {
            Ok(ret as usize)
        }
    }

    pub fn add(&self, fid: &impl crate::AsFid, flags:u64) -> Result<(), crate::error::Error> { //[TODO] fid should implement Waitable trait
        let err = unsafe { libfabric_sys::inlined_fi_poll_add(self.c_poll, fid.as_fid(), flags) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn del(&self, fid: &impl crate::AsFid, flags:u64) -> Result<(), crate::error::Error> { //[TODO] fid should implement Waitable trait
        let err = unsafe { libfabric_sys::inlined_fi_poll_del(self.c_poll, fid.as_fid(), flags) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn wait_object(&self) -> Result<WaitObjType2, crate::error::Error> {
        let mut res: libfabric_sys::fi_wait_obj = 0;
        let err = unsafe{libfabric_sys::inlined_fi_control(self.as_fid(), libfabric_sys::FI_GETWAITOBJ as i32, &mut res as *mut libfabric_sys::fi_wait_obj as *mut std::ffi::c_void )};
        
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
                let err = unsafe{libfabric_sys::inlined_fi_control(self.as_fid(), libfabric_sys::FI_GETWAIT as i32, &mut fd as *mut i32 as *mut std::ffi::c_void )};
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
                let err = unsafe{libfabric_sys::inlined_fi_control(self.as_fid(), libfabric_sys::FI_GETWAIT as i32, &mut cond as *mut libfabric_sys::fi_mutex_cond as *mut std::ffi::c_void )};
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
                let err = unsafe{libfabric_sys::inlined_fi_control(self.as_fid(), libfabric_sys::FI_GETWAIT as i32, &mut wait as *mut libfabric_sys::fi_wait_pollfd as *mut std::ffi::c_void )};
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

impl AsFid for PollSet {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
        self.fid.as_fid()
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