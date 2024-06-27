//================== Domain (fi_domain) ==================//

use std::{marker::PhantomData, os::fd::{AsFd, BorrowedFd}, rc::Rc};

#[allow(unused_imports)]
use crate::fid::AsFid;
use crate::{cntroptions::{self, CntrConfig, Options}, enums::WaitObjType, FdRetrievable, WaitRetrievable, domain::DomainImpl, BindImpl, utils::check_error, fid::{self, AsRawFid, OwnedCntrFid, CntrRawFid, AsRawTypedFid, RawFid, AsTypedFid}};

/// Owned wrapper around a libfabric `fi_cntr`.
/// 
/// This type wraps an instance of a `fi_cntr`, monitoring its lifetime and closing it when it goes out of scope.
/// To be able to check its configuration at compile this object is extended with a `T:`[`CntrConfig`] (e.g. [Options]) that provides this information.
/// 
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.19.0/man/fi_cntr.3.html).
/// 
/// Note that other objects that rely on a Counter (e.g., an [crate::ep::Endpoint] bound to it) will extend its lifetime until they
/// are also dropped.
pub struct Counter<T: CntrConfig> {
    pub(crate) inner: Rc<CounterImpl>,
    phantom: PhantomData<T>,
}

pub(crate) struct CounterImpl {
    pub(crate) c_cntr: OwnedCntrFid,
    wait_obj: Option<libfabric_sys::fi_wait_obj>,
    _domain_rc: Rc<DomainImpl>,
}



impl CounterImpl {

    pub(crate) fn new<T0>(domain: &Rc<crate::domain::DomainImpl>, mut attr: CounterAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        let mut c_cntr: CntrRawFid = std::ptr::null_mut();

        let err = 
            if let Some(ctx) = context {
                unsafe { libfabric_sys::inlined_fi_cntr_open(domain.as_raw_typed_fid(), attr.get_mut(), &mut c_cntr, (ctx as *mut T0).cast()) }
            }
            else {
                unsafe { libfabric_sys::inlined_fi_cntr_open(domain.as_raw_typed_fid(), attr.get_mut(), &mut c_cntr, std::ptr::null_mut()) }
            };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok (
                Self {
                    c_cntr: OwnedCntrFid::from(c_cntr), 
                    wait_obj: Some(attr.c_attr.wait_obj),
                    _domain_rc: domain.clone(),
                })
        }
    }

    pub(crate) fn read(&self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_cntr_read(self.as_raw_typed_fid()) }
    }

    pub(crate) fn readerr(&self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_cntr_readerr(self.as_raw_typed_fid()) }
    }

    pub(crate) fn add(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_add(self.as_raw_typed_fid(), val) };
    
        check_error(err.try_into().unwrap())
    }

    pub(crate) fn adderr(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_adderr(self.as_raw_typed_fid(), val) };
            
        check_error(err.try_into().unwrap())
    }

    pub(crate) fn set(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_set(self.as_raw_typed_fid(), val) };
            
        check_error(err.try_into().unwrap())
    }

    pub(crate) fn seterr(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_seterr(self.as_raw_typed_fid(), val) };
            
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
        let err = unsafe { libfabric_sys::inlined_fi_cntr_wait(self.as_raw_typed_fid(), threshold, timeout) };

        check_error(err.try_into().unwrap())
    }
}

impl<T: CntrConfig> Counter<T> {
    #[allow(dead_code)]

    pub(crate) fn new<T0>(domain: &crate::domain::Domain, attr: CounterAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        Ok(
            Self {
                inner: Rc::new(CounterImpl::new(&domain.inner, attr, context)?),
                phantom: PhantomData,
            }
        )
    }
    
    /// Returns the current value of the counter
    /// 
    /// Corresponds to `fi_cntr_read`
    pub fn read(&self) -> u64 {
        self.inner.read()
    }
    
    /// Returns the number of operations that completed in error and were unable to update the counter.
    /// 
    /// Corresponds to `fi_cntr_readerr`
    pub fn readerr(&self) -> u64 {
        self.inner.readerr()
    }

    /// Adds value `val` to the Counter
    /// 
    /// Corresponds to `fi_cntr_add`
    pub fn add(&self, val: u64) -> Result<(), crate::error::Error> {
        self.inner.add(val)
    }

    /// Adds value `val` to the error value of the Counter
    /// 
    /// Corresponds to `fi_cntr_adderr`
    pub fn adderr(&self, val: u64) -> Result<(), crate::error::Error> {
        self.inner.adderr(val)
    }

    /// Sets the Counter value to `val`
    /// 
    /// Corresponds to `fi_cntr_set`
    pub fn set(&self, val: u64) -> Result<(), crate::error::Error> {
        self.inner.set(val)
    }

    /// Sets the Counter error value to `val`
    /// 
    /// Corresponds to `fi_cntr_seterr`
    pub fn seterr(&self, val: u64) -> Result<(), crate::error::Error> {
        self.inner.seterr(val)
    }
}

impl<T: CntrConfig + crate::WaitRetrievable> Counter<T> {
    
    /// Retreives the low-level wait object associated with the counter.
    /// 
    /// This method is available only if the counter has been configured with a retrievable
    /// underlying wait object.
    /// 
    /// Corresponds to `fi_cntr_control` with command `FI_GETWAIT`.
    pub fn wait_object(&self) -> Result<WaitObjType<'_>, crate::error::Error> {
        self.inner.wait_object()
    }
}

impl<T: CntrConfig + crate::Waitable> Counter<T> {

    /// Wait until the counter reaches the specified `threshold``, or until an error or `timeout` occurs.
    /// 
    /// This method is not available if the counter is configured with no wait object ([CounterBuilder::wait_none])
    /// 
    /// Corresponds to `fi_cntr_wait`
    pub fn wait(&self, threshold: u64, timeout: i32) -> Result<(), crate::error::Error> { // [TODO]
        self.inner.wait(threshold, timeout)
    }
}


//================== Counter Builder ==================//

/// Builder for the [`Counter`] type.
/// 
/// `CounterBuilder` is used to configure and build a new `Counter`.
/// It encapsulates an incremental configuration of the counter, as provided by a `fi_cntr_attr`,
/// followed by a call to `fi_cntr_open`  
pub struct CounterBuilder<'a, T, WAIT, WAITFD> {
    cntr_attr: CounterAttr,
    domain: &'a crate::domain::Domain,
    ctx: Option<&'a mut T>,
    options: cntroptions::Options<WAIT, WAITFD>,
}

impl<'a> CounterBuilder<'a, (), cntroptions::WaitNoRetrieve, cntroptions::Off> {

    /// Initiates the creation of new [Counter] on `domain`.
    /// 
    /// The initial configuration is what would be set if no `fi_cntr_attr` or `context` was provided to 
    /// the `fi_cntr_open` call. 
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
    
    /// Sets the type of events to count
    /// 
    /// Corresponds to setting the `fi_cntr_attr::events` field.
    pub fn events(mut self, events: crate::enums::CounterEvents) -> Self {
        self.cntr_attr.events(events);

        self
    }

    /// Sets the underlying low-level waiting object to none.
    /// 
    /// Corresponds to setting `fi_cntr_attr::wait_obj` to `FI_WAIT_NONE`.
    pub fn wait_none(mut self) -> CounterBuilder<'a, T, cntroptions::WaitNone, cntroptions::Off> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::None);

        CounterBuilder {
            options: self.options.no_wait(),
            cntr_attr: self.cntr_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }        

    /// Sets the underlying low-level waiting object to FD.
    /// 
    /// Corresponds to setting `fi_cntr_attr::wait_obj` to `FI_WAIT_FD`.
    pub fn wait_fd(mut self) -> CounterBuilder<'a, T, cntroptions::WaitRetrieve, cntroptions::On> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::Fd);

        CounterBuilder {
            options: self.options.wait_fd(),
            cntr_attr: self.cntr_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }

    /// Sets the underlying low-level waiting object to [crate::sync::WaitSet] `set`.
    /// 
    /// Corresponds to setting `fi_cntr_attr::wait_obj` to `FI_WAIT_SET` and `fi_cntr_attr::wait_set` to `set`.
    pub fn wait_set(mut self, set: &crate::sync::WaitSet) -> CounterBuilder<'a, T, cntroptions::WaitNoRetrieve, cntroptions::Off> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::Set(set));

        CounterBuilder {
            options: self.options.wait_no_retrieve(),
            cntr_attr: self.cntr_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }

    /// Sets the underlying low-level waiting object to Mutex+Conditional.
    /// 
    /// Corresponds to setting `fi_cntr_attr::wait_obj` to `FI_WAIT_MUTEX_COND`.
    pub fn wait_mutex(mut self) -> CounterBuilder<'a, T, cntroptions::WaitRetrieve, cntroptions::Off> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::MutexCond);

        CounterBuilder {
            options: self.options.wait_retrievable(),
            cntr_attr: self.cntr_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }

    /// Indicates that the counter will wait without a wait object but instead yield on every wait.
    /// 
    /// Corresponds to setting `fi_cntr_attr::wait_obj` to `FI_WAIT_YIELD`.
    pub fn wait_yield(mut self) -> CounterBuilder<'a, T, cntroptions::WaitNoRetrieve, cntroptions::Off> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::Yield);

        CounterBuilder {
            options: self.options.wait_no_retrieve(),
            cntr_attr: self.cntr_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }

    /// Sets the context to be passed to the `Counter`.
    /// 
    /// Corresponds to passing a non-NULL `context` value to `fi_cntr_open`.
    pub fn context(self, ctx: &'a mut T) -> CounterBuilder::<'a, T, WAIT, WAITFD> {
        CounterBuilder {
            cntr_attr: self.cntr_attr,
            domain: self.domain,
            ctx: Some(ctx),
            options: self.options,
        }
    }

    /// Constructs a new [Counter] with the configurations requested so far.
    /// 
    /// Corresponds to creating a `fi_cntr_attr`, setting its fields to the requested ones,
    /// and passing it to the `fi_cntr_open` call with an optional `context`.
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

impl<T: CntrConfig> AsRawFid for Counter<T> {
    fn as_raw_fid(&self) -> RawFid {
        self.inner.as_raw_fid()
    }
}

impl AsFid for CounterImpl {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.c_cntr.as_fid()
    }
}

impl AsRawFid for CounterImpl {
    fn as_raw_fid(&self) -> RawFid {
        self.c_cntr.as_raw_fid()
    }
}

impl AsTypedFid<CntrRawFid> for CounterImpl {
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<'_, CntrRawFid> {
        self.c_cntr.as_typed_fid()
    }
}

impl AsRawTypedFid for CounterImpl {
    type Output = CntrRawFid;
    
    fn as_raw_typed_fid(&self) -> Self::Output {
        self.c_cntr.as_raw_typed_fid()
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
            self.c_attr.wait_set = wait_set.as_raw_typed_fid();
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