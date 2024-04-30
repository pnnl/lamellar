//================== Domain (fi_domain) ==================//

use std::{marker::PhantomData, os::fd::{AsFd, BorrowedFd}, rc::Rc};

#[allow(unused_imports)]
use crate::fid::AsFid;
use crate::{cntroptions::{self, CntrConfig, Options}, enums::WaitObjType, FdRetrievable, WaitRetrievable, domain::DomainImpl, BindImpl, utils::check_error, fid::{OwnedFid, self, AsRawFid}};

pub struct Counter<T: CntrConfig> {
    pub(crate) inner: Rc<CounterImpl>,
    phantom: PhantomData<T>,
}

pub(crate) struct CounterImpl {
    pub(crate) c_cntr: *mut libfabric_sys::fid_cntr,
    fid: OwnedFid,
    wait_obj: Option<libfabric_sys::fi_wait_obj>,
    _domain_rc: Rc<DomainImpl>,
}



impl CounterImpl {

    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_cntr {
        self.c_cntr
    }

    pub(crate) fn new<T0>(domain: &Rc<crate::domain::DomainImpl>, mut attr: CounterAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        let mut c_cntr: *mut libfabric_sys::fid_cntr = std::ptr::null_mut();
        let c_cntr_ptr: *mut *mut libfabric_sys::fid_cntr = &mut c_cntr;
        let err = 
            if let Some(ctx) = context {
                unsafe { libfabric_sys::inlined_fi_cntr_open(domain.handle(), attr.get_mut(), c_cntr_ptr, (ctx as *mut T0).cast()) }
            }
            else {
                unsafe { libfabric_sys::inlined_fi_cntr_open(domain.handle(), attr.get_mut(), c_cntr_ptr, std::ptr::null_mut()) }
            };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok (
                Self {
                    c_cntr, 
                    fid: OwnedFid::from(unsafe { &mut (*c_cntr).fid }), 
                    wait_obj: Some(attr.c_attr.wait_obj),
                    _domain_rc: domain.clone(),
                })
        }
    }

    pub(crate) fn read(&self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_cntr_read(self.handle()) }
    }

    pub(crate) fn readerr(&self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_cntr_readerr(self.handle()) }
    }

    pub(crate) fn add(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_add(self.handle(), val) };
    
        check_error(err.try_into().unwrap())
    }

    pub(crate) fn adderr(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_adderr(self.handle(), val) };
            
        check_error(err.try_into().unwrap())
    }

    pub(crate) fn set(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_set(self.handle(), val) };
            
        check_error(err.try_into().unwrap())
    }

    pub(crate) fn seterr(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_seterr(self.handle(), val) };
            
        check_error(err.try_into().unwrap())
    }    

    pub(crate) fn wait_object(&self) -> Result<WaitObjType<'_>, crate::error::Error> {

        if let Some(wait) = self.wait_obj {
            if wait == libfabric_sys::fi_wait_obj_FI_WAIT_FD {
                let mut fd: i32 = 0;
                let err = unsafe { libfabric_sys::inlined_fi_control(self.as_raw_fid(), libfabric_sys::FI_GETWAIT as i32, (&mut fd as *mut i32).cast()) };
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

                let err = unsafe { libfabric_sys::inlined_fi_control(self.as_raw_fid(), libfabric_sys::FI_GETWAIT as i32, (&mut mutex_cond as *mut libfabric_sys::fi_mutex_cond).cast()) };
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

    pub(crate) fn wait(&self, threshold: u64, timeout: i32) -> Result<(), crate::error::Error> { // [TODO]
        let err = unsafe { libfabric_sys::inlined_fi_cntr_wait(self.handle(), threshold, timeout) };

        check_error(err.try_into().unwrap())
    }
}

impl<T: CntrConfig> Counter<T> {
    #[allow(dead_code)]
    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_cntr {
        self.inner.handle()
    }

    pub(crate) fn new<T0>(domain: &crate::domain::Domain, attr: CounterAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        Ok(
            Self {
                inner: Rc::new(CounterImpl::new(&domain.inner, attr, context)?),
                phantom: PhantomData,
            }
        )
    }
    
    pub fn read(&self) -> u64 {
        self.inner.read()
    }

    pub fn readerr(&self) -> u64 {
        self.inner.readerr()
    }

    pub fn add(&self, val: u64) -> Result<(), crate::error::Error> {
        self.inner.add(val)
    }

    pub fn adderr(&self, val: u64) -> Result<(), crate::error::Error> {
        self.inner.adderr(val)
    }

    pub fn set(&self, val: u64) -> Result<(), crate::error::Error> {
        self.inner.set(val)
    }

    pub fn seterr(&self, val: u64) -> Result<(), crate::error::Error> {
        self.inner.seterr(val)
    }
}

impl<T: CntrConfig + crate::WaitRetrievable> Counter<T> {
    
    pub fn wait_object(&self) -> Result<WaitObjType<'_>, crate::error::Error> {
        self.inner.wait_object()
    }
}

impl<T: CntrConfig + crate::Waitable> Counter<T> {

    pub fn wait(&self, threshold: u64, timeout: i32) -> Result<(), crate::error::Error> { // [TODO]
        self.inner.wait(threshold, timeout)
    }
}


//================== Counter Builder ==================//

pub struct CounterBuilder<'a, T, WAIT, WAITFD> {
    cntr_attr: CounterAttr,
    domain: &'a crate::domain::Domain,
    ctx: Option<&'a mut T>,
    options: cntroptions::Options<WAIT, WAITFD>,
}

impl<'a> CounterBuilder<'a, (), cntroptions::WaitNoRetrieve, cntroptions::Off> {
    pub fn new(domain: &'a crate::domain::Domain) -> Self {
        Self {
            cntr_attr: CounterAttr::new(),
            domain,
            ctx: None,
            options: Options::new(),
        }
    }
}

impl<'a, T, WAIT, WAITFD> CounterBuilder<'a, T, WAIT, WAITFD> {
    
    pub fn events(mut self, events: crate::enums::CounterEvents) -> Self {
        self.cntr_attr.events(events);

        self
    }

    pub fn wait_none(mut self) -> CounterBuilder<'a, T, cntroptions::WaitNone, cntroptions::Off> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::None);

        CounterBuilder {
            options: self.options.no_wait(),
            cntr_attr: self.cntr_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }        


    pub fn wait_fd(mut self) -> CounterBuilder<'a, T, cntroptions::WaitRetrieve, cntroptions::On> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::Fd);

        CounterBuilder {
            options: self.options.wait_fd(),
            cntr_attr: self.cntr_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }

    pub fn wait_set(mut self, set: &crate::sync::WaitSet) -> CounterBuilder<'a, T, cntroptions::WaitNoRetrieve, cntroptions::Off> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::Set(set));

        CounterBuilder {
            options: self.options.wait_no_retrieve(),
            cntr_attr: self.cntr_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }

    pub fn wait_mutex(mut self) -> CounterBuilder<'a, T, cntroptions::WaitRetrieve, cntroptions::Off> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::MutexCond);

        CounterBuilder {
            options: self.options.wait_retrievable(),
            cntr_attr: self.cntr_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }

    pub fn wait_yield(mut self) -> CounterBuilder<'a, T, cntroptions::WaitNoRetrieve, cntroptions::Off> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::Yield);

        CounterBuilder {
            options: self.options.wait_no_retrieve(),
            cntr_attr: self.cntr_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }

    pub fn context(self, ctx: &'a mut T) -> CounterBuilder::<'a, T, WAIT, WAITFD> {
        CounterBuilder {
            cntr_attr: self.cntr_attr,
            domain: self.domain,
            ctx: Some(ctx),
            options: self.options,
        }
    }

    pub fn build(self) -> Result<Counter<Options<WAIT, WAITFD>>, crate::error::Error> {
        Counter::new(self.domain, self.cntr_attr, self.ctx)
    }
}



//================== Trait impls ==================//

impl<T: CntrConfig> AsFid for Counter<T> {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.inner.as_fid()
    }
}

impl AsFid for CounterImpl {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.fid.as_fid()
    }
}

impl AsFd for CounterImpl {
    fn as_fd(&self) -> BorrowedFd<'_> {
        if let WaitObjType::Fd(fd) = self.wait_object().unwrap() {
            fd
        }
        else {
            panic!("Fabric object object type is not Fd")
        }
    }
}

impl<T: CntrConfig + WaitRetrievable + FdRetrievable> AsFd for Counter<T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

impl BindImpl for CounterImpl {}

impl<T: CntrConfig + 'static> crate::Bind for Counter<T> {
    fn inner(&self) -> Rc<dyn BindImpl> {
        self.inner.clone()
    }
}

//================== Attribute objects ==================//

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
        if let crate::enums::WaitObj::Set(wait_set) = wait_obj {
            self.c_attr.wait_set = wait_set.handle();
        }
        self.c_attr.wait_obj = wait_obj.get_value();
        self
    }

    // pub(crate) fn flags(&mut self, flags: u64) -> &mut Self {
    //     self.c_attr.flags = flags;

    //     self
    // }

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
    use crate::info::{InfoHints, Info};

    use super::CounterBuilder;

    #[test]
    fn cntr_loop() {

        let mut dom_attr = crate::domain::DomainAttr::new();
            dom_attr
            .mode(crate::enums::Mode::all())
            .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse());
        
        let hints = InfoHints::new()
            .domain_attr(dom_attr)
            .mode(crate::enums::Mode::all());
        

        let info = Info::new().hints(&hints).request().unwrap();
        let entries = info.get();
        
        if !entries.is_empty() {
            for e in entries {
                if e.get_domain_attr().get_cntr_cnt() != 0 {
                    let fab = crate::fabric::FabricBuilder::new(e).build().unwrap();
                    let domain = crate::domain::DomainBuilder::new(&fab, e).build().unwrap();
                    let cntr_cnt = std::cmp::min(e.get_domain_attr().get_cntr_cnt(), 100);
                    let cntrs: Vec<_> = (0..cntr_cnt).map(|_| CounterBuilder::new(&domain).build().unwrap() ).collect::<>();

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

#[cfg(test)]
mod libfabric_lifetime_tests {

    use crate::info::{InfoHints, Info};

    use super::CounterBuilder;


    #[test]
    fn cntr_drops_before_domain() {
        let mut dom_attr = crate::domain::DomainAttr::new();
            dom_attr
            .mode(crate::enums::Mode::all())
            .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse());
        
        let hints = InfoHints::new()
            .domain_attr(dom_attr)
            .mode(crate::enums::Mode::all());
        

        let info = Info::new().hints(&hints).request().unwrap();
        let entries = info.get();
        
        if !entries.is_empty() {
            for e in entries {
                if e.get_domain_attr().get_cntr_cnt() != 0 {
                    let fab = crate::fabric::FabricBuilder::new(e).build().unwrap();
                    let domain = crate::domain::DomainBuilder::new(&fab, e).build().unwrap();
                    let cntr_cnt = std::cmp::min(e.get_domain_attr().get_cntr_cnt(), 100);
                    let cntrs: Vec<_> = (0..cntr_cnt).map(|_| CounterBuilder::new(&domain).build().unwrap() ).collect::<>();
                    println!("Count = {}", std::rc::Rc::strong_count(&domain.inner));
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
                    drop(domain);
                    println!("Count = {} After dropping domain ", std::rc::Rc::strong_count(&cntrs[0].inner._domain_rc));
                    break;
                }
            }
        }
        else {
            panic!("Could not find suitable fabric");
        }
    }
}