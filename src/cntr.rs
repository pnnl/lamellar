//================== Domain (fi_domain) ==================//

use std::{marker::PhantomData, os::fd::BorrowedFd};

#[allow(unused_imports)]
use crate::AsFid;
use crate::{enums::WaitObjType, OwnedFid};

pub struct Waitable;
pub struct NonWaitable;

pub type CounterWaitable = CounterBase<Waitable>;
pub type CounterNonWaitable = CounterBase<NonWaitable>;

pub enum Counter {
    Waitable(CounterBase<Waitable>),
    NonWaitable(CounterBase<NonWaitable>),
}

pub struct CounterBase<T> {
    pub(crate) c_cntr: *mut libfabric_sys::fid_cntr,
    fid: OwnedFid,
    phantom: PhantomData<T>,
    wait_obj: Option<libfabric_sys::fi_wait_obj>,
}

impl Counter {
    pub fn read(&self) -> u64 {
        match self {
            Counter::Waitable(cntr) => cntr.read(),
            Counter::NonWaitable(cntr) => cntr.read(),
        }
    }

    pub fn readerr(&self) -> u64 {
        match self {
            Counter::Waitable(cntr) => cntr.readerr(),
            Counter::NonWaitable(cntr) => cntr.readerr(),
        }
    }

    pub fn add(&self, val: u64) -> Result<(), crate::error::Error> {
        match self {
            Counter::Waitable(cntr) => cntr.add(val),
            Counter::NonWaitable(cntr) => cntr.add(val),
        }
    }

    pub fn adderr(&self, val: u64) -> Result<(), crate::error::Error> {
        match self {
            Counter::Waitable(cntr) => cntr.adderr(val),
            Counter::NonWaitable(cntr) => cntr.adderr(val),
        }
    }

    pub fn set(&self, val: u64) -> Result<(), crate::error::Error> {
        match self {
            Counter::Waitable(cntr) => cntr.set(val),
            Counter::NonWaitable(cntr) => cntr.set(val),
        } 
    }

    pub fn seterr(&self, val: u64) -> Result<(), crate::error::Error> {
        match self {
            Counter::Waitable(cntr) => cntr.seterr(val),
            Counter::NonWaitable(cntr) => cntr.seterr(val),
        } 
    }
}

impl<T> CounterBase<T> {
    pub(crate) fn new(domain: &crate::domain::Domain, mut attr: CounterAttr) -> Result<CounterBase<T>, crate::error::Error> {
        let mut c_cntr: *mut libfabric_sys::fid_cntr = std::ptr::null_mut();
        let c_cntr_ptr: *mut *mut libfabric_sys::fid_cntr = &mut c_cntr;
        let err = unsafe { libfabric_sys::inlined_fi_cntr_open(domain.c_domain, attr.get_mut(), c_cntr_ptr, std::ptr::null_mut()) };
        

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok (
                Self { c_cntr, fid: OwnedFid { fid: unsafe { &mut (*c_cntr).fid }}, phantom: PhantomData, wait_obj: None }
            )
        }

    }

    pub(crate) fn new_with_context<T0>(domain: &crate::domain::Domain, mut attr: CounterAttr, ctx: &mut T0) -> Result<CounterBase<T>, crate::error::Error> {
        let mut c_cntr: *mut libfabric_sys::fid_cntr = std::ptr::null_mut();
        let c_cntr_ptr: *mut *mut libfabric_sys::fid_cntr = &mut c_cntr;
        let err = unsafe { libfabric_sys::inlined_fi_cntr_open(domain.c_domain, attr.get_mut(), c_cntr_ptr, ctx as *mut T0 as *mut std::ffi::c_void) };


        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok (
                Self { c_cntr, fid: OwnedFid { fid: unsafe { &mut (*c_cntr).fid }}, phantom: PhantomData, wait_obj: None }
            )
        }

    }

    pub fn read(&self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_cntr_read(self.c_cntr) }
    }

    pub fn readerr(&self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_cntr_readerr(self.c_cntr) }
    }

    pub fn add(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_add(self.c_cntr, val) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(())
        }
    }

    pub fn adderr(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_adderr(self.c_cntr, val) };
            
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(())
        }
    }

    pub fn set(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_set(self.c_cntr, val) };
            
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(())
        }
    }

    pub fn seterr(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_seterr(self.c_cntr, val) };
            
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(())
        }
    }
}


impl CounterBase<Waitable> {

    pub fn wait(&self, threshold: u64, timeout: i32) -> Result<(), crate::error::Error> { // [TODO]
        let err = unsafe { libfabric_sys::inlined_fi_cntr_wait(self.c_cntr, threshold, timeout) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(())
        }
    }

    pub fn wait_obj(&self) -> Result<WaitObjType<'_>, crate::error::Error> {

        if let Some(wait) = self.wait_obj {
            if wait == libfabric_sys::fi_wait_obj_FI_WAIT_FD {
                let mut fd: i32 = 0;
                let err = unsafe { libfabric_sys::inlined_fi_control(self.as_fid(), libfabric_sys::FI_GETWAIT as i32, &mut fd as *mut i32 as *mut std::ffi::c_void) };
                if err < 0 {
                    Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
                }
                else {
                    Ok(WaitObjType::Fd(unsafe{ BorrowedFd::borrow_raw(fd) }))
                }
            }
            else if wait == libfabric_sys::fi_wait_obj_FI_WAIT_MUTEX_COND {
                let mut mutex_cond = libfabric_sys::fi_mutex_cond{
                    mutex: std::ptr::null_mut(),
                    cond: std::ptr::null_mut(),
                };

                let err = unsafe { libfabric_sys::inlined_fi_control(self.as_fid(), libfabric_sys::FI_GETWAIT as i32, &mut mutex_cond as *mut libfabric_sys::fi_mutex_cond as *mut std::ffi::c_void) };
                if err < 0 {
                    Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
                }
                else {
                    Ok(WaitObjType::MutexCond(mutex_cond))
                }
            }
            else if wait == libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC{
                Ok(WaitObjType::Unspec)
            }
            else {
                panic!("Could not retrieve wait object")
            }
        }
        else { 
            panic!("Should not be reachable! Could not retrieve wait object")
        }
    }
}

impl crate::AsFid for CounterBase<Waitable> {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
        self.fid.as_fid()
    }
}

impl crate::AsFid for CounterBase<NonWaitable> {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
        self.fid.as_fid()
    }
}

impl AsFid for Counter {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
        match self {
            Counter::Waitable(queue) => queue.as_fid(), 
            Counter::NonWaitable(queue) => queue.as_fid(), 
        }
    }
}

impl crate::Bind for CounterBase<Waitable> {}
impl crate::Bind for CounterBase<NonWaitable> {}
impl crate::Bind for Counter {}

//================== Counter Builder ==================//

pub struct CounterBuilder<'a, T> {
    cntr_attr: CounterAttr,
    domain: &'a crate::domain::Domain,
    ctx: Option<&'a mut T>,
}

impl<'a, T> CounterBuilder<'a, T> {
    
    pub fn events(mut self, events: crate::enums::CounterEvents) -> Self {
        self.cntr_attr.events(events);

        self
    }

    pub fn wait_obj(mut self, wait_obj: crate::enums::WaitObj) -> Self {
        self.cntr_attr.wait_obj(wait_obj);

        self
    }

    pub fn flags(mut self, flags: u64) -> Self {
        self.cntr_attr.flags(flags);

        self
    }

    pub fn context(self, ctx: &'a mut T) -> CounterBuilder::<'a, T> {
        CounterBuilder {
            cntr_attr: self.cntr_attr,
            domain: self.domain,
            ctx: Some(ctx),
        }
    }

    pub fn build(self) -> Result<Counter, crate::error::Error> {
        if self.cntr_attr.c_attr.wait_obj == crate::enums::WaitObj::NONE.get_value() {
            let cntr = if let Some(ctx) = self.ctx{
                CounterWaitable::new_with_context(self.domain, self.cntr_attr, ctx)?
            }
            else {
                CounterWaitable::new(self.domain, self.cntr_attr)?
            };

            Ok(Counter::Waitable(cntr))
        }
        else {
            let cntr = if let Some(ctx) = self.ctx{
                CounterNonWaitable::new_with_context(self.domain, self.cntr_attr, ctx)?
            }
            else {
                CounterNonWaitable::new(self.domain, self.cntr_attr)?
            };
            Ok(Counter::NonWaitable(cntr))
        }
    }
}

impl<'a> CounterBuilder<'a, ()> {
    pub fn new(domain: &crate::domain::Domain) -> CounterBuilder<()> {
        CounterBuilder::<()> {
            cntr_attr: CounterAttr::new(),
            domain,
            ctx: None,
        }
    }
}

//================== Counter attribute ==================//

#[derive(Clone, Copy)]
pub(crate) struct CounterAttr {
    pub(crate) c_attr: libfabric_sys::fi_cntr_attr,
}

impl CounterAttr {

    pub(crate) fn new() -> Self {
        let c_attr = libfabric_sys::fi_cntr_attr {
            events: libfabric_sys::fi_cntr_events_FI_CNTR_EVENTS_COMP,
            wait_obj: libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC,
            wait_set: std::ptr::null_mut(),
            flags: 0,
        };

        Self { c_attr }
    }

    pub(crate) fn events(&mut self, events: crate::enums::CounterEvents) -> &mut Self {
        self.c_attr.events = events.get_value();

        self
    }

    pub(crate) fn wait_obj(&mut self, wait_obj: crate::enums::WaitObj) -> &mut Self {
        if let crate::enums::WaitObj::SET(wait_set) = wait_obj {
            self.c_attr.wait_set = wait_set.c_wait;
        }
        self.c_attr.wait_obj = wait_obj.get_value();
        self
    }

    pub(crate) fn flags(&mut self, flags: u64) -> &mut Self {
        self.c_attr.flags = flags;

        self
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_cntr_attr {
        &self.c_attr
    }   

    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_cntr_attr {
        &mut self.c_attr
    }   
}

impl Default for CounterAttr {
    fn default() -> Self {
        Self::new()
    }
}    

//================== Counter tests ==================//

#[cfg(test)]
mod tests {
    use super::CounterBuilder;

    #[test]
    fn cntr_loop() {

        let mut dom_attr = crate::domain::DomainAttr::new();
            dom_attr
            .mode(crate::enums::Mode::all())
            .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse());
        
        let hints = crate::InfoHints::new()
            .domain_attr(dom_attr)
            .mode(crate::enums::Mode::all());
        

        let info = crate::Info::new().hints(&hints).request().unwrap();
        let entries: Vec<crate::InfoEntry> = info.get();
        
        if !entries.is_empty() {
            for e in entries {
                if e.get_domain_attr().get_cntr_cnt() != 0 {
                    let fab = crate::fabric::FabricBuilder::new(&e).build().unwrap();
                    let domain = crate::domain::DomainBuilder::new(&fab, &e).build().unwrap();
                    let cntr_cnt = std::cmp::min(e.get_domain_attr().get_cntr_cnt(), 100);
                    let cntrs: Vec<crate::cntr::Counter> = (0..cntr_cnt).map(|_| CounterBuilder::new(&domain).build().unwrap() ).collect();

                    for (i,cntr) in cntrs.iter().enumerate() {
                        cntr.set(i as u64).unwrap();
                        cntr.seterr((i << 1) as u64).unwrap();
                    }
                    
                    for (i,cntr) in cntrs.iter().enumerate() {
                        cntr.add(i as u64).unwrap();
                        cntr.adderr(i as u64).unwrap();
                    }

                    for (i,cntr) in cntrs.iter().enumerate() {
                        let expected = i + i;
                        let value = cntr.read() as usize;
                        assert_eq!(expected, value);
                    }
                    
                    for (i,cntr) in cntrs.iter().enumerate() {
                        let expected = (i << 1) + i;
                        let value = cntr.readerr() as usize;
                        assert_eq!(expected, value);
                    }
                    break;
                }

            }

        }
        else {
            panic!("Could not find suitable fabric");
        }
    }
}