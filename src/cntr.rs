//================== Domain (fi_domain) ==================//

use std::os::fd::{AsFd, BorrowedFd};

#[allow(unused_imports)]
use crate::fid::AsFid;
use crate::{
    cq::WaitObjectRetrieve,
    domain::DomainImplT,
    enums::WaitObjType,
    fid::{self, AsRawFid, AsRawTypedFid, AsTypedFid, CntrRawFid, OwnedCntrFid, RawFid},
    utils::check_error,
    Context, MyRc,
};

/// Owned wrapper around a libfabric `fi_cntr`.
///
/// This type wraps an instance of a `fi_cntr`, monitoring its lifetime and closing it when it goes out of scope.
/// To be able to check its configuration at compile this object is extended with a `T:`[`CntrConfig`] (e.g. [Options]) that provides this information.
///
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.19.0/man/fi_cntr.3.html).
///
/// Note that other objects that rely on a Counter (e.g., an [crate::ep::Endpoint] bound to it) will extend its lifetime until they
/// are also dropped.
pub struct Counter<CNTR> {
    pub(crate) inner: MyRc<CNTR>,
}

pub struct CounterImpl<const WAIT: bool, const RETRIEVE: bool, const FD: bool> {
    pub(crate) c_cntr: OwnedCntrFid,
    wait_obj: Option<libfabric_sys::fi_wait_obj>,
    _domain_rc: MyRc<dyn DomainImplT>,
}

pub trait ReadCntr: AsRawTypedFid<Output = CntrRawFid> + AsRawFid {
    fn read(&self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_cntr_read(self.as_raw_typed_fid()) }
    }

    fn readerr(&self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_cntr_readerr(self.as_raw_typed_fid()) }
    }

    fn add(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_add(self.as_raw_typed_fid(), val) };

        check_error(err.try_into().unwrap())
    }

    fn adderr(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_adderr(self.as_raw_typed_fid(), val) };

        check_error(err.try_into().unwrap())
    }

    fn set(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_set(self.as_raw_typed_fid(), val) };

        check_error(err.try_into().unwrap())
    }

    fn seterr(&self, val: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_seterr(self.as_raw_typed_fid(), val) };

        check_error(err.try_into().unwrap())
    }
}

pub trait WaitCntr: AsRawTypedFid<Output = CntrRawFid> + ReadCntr {
    fn wait(&self, threshold: u64, timeout: i32) -> Result<(), crate::error::Error> {
        // [TODO]
        let err = unsafe {
            libfabric_sys::inlined_fi_cntr_wait(self.as_raw_typed_fid(), threshold, timeout)
        };

        check_error(err.try_into().unwrap())
    }
}

impl<const WAIT: bool, const RETRIEVE: bool, const FD: bool> ReadCntr
    for CounterImpl<WAIT, RETRIEVE, FD>
{
}
impl<const RETRIEVE: bool, const FD: bool> WaitCntr for CounterImpl<true, RETRIEVE, FD> {}

impl<'a, const FD: bool> WaitObjectRetrieve<'a> for CounterImpl<true, true, FD> {
    fn wait_object(&self) -> Result<WaitObjType<'a>, crate::error::Error> {
        if let Some(wait) = self.wait_obj {
            if wait == libfabric_sys::fi_wait_obj_FI_WAIT_FD {
                let mut fd: i32 = 0;
                let err = unsafe {
                    libfabric_sys::inlined_fi_control(
                        self.as_raw_fid(),
                        libfabric_sys::FI_GETWAIT as i32,
                        (&mut fd as *mut i32).cast(),
                    )
                };
                if err < 0 {
                    Err(crate::error::Error::from_err_code(
                        (-err).try_into().unwrap(),
                    ))
                } else {
                    Ok(WaitObjType::Fd(unsafe { BorrowedFd::borrow_raw(fd) }))
                }
            } else if wait == libfabric_sys::fi_wait_obj_FI_WAIT_MUTEX_COND {
                let mut mutex_cond = libfabric_sys::fi_mutex_cond {
                    mutex: std::ptr::null_mut(),
                    cond: std::ptr::null_mut(),
                };

                let err = unsafe {
                    libfabric_sys::inlined_fi_control(
                        self.as_raw_fid(),
                        libfabric_sys::FI_GETWAIT as i32,
                        (&mut mutex_cond as *mut libfabric_sys::fi_mutex_cond).cast(),
                    )
                };
                if err < 0 {
                    Err(crate::error::Error::from_err_code(
                        (-err).try_into().unwrap(),
                    ))
                } else {
                    Ok(WaitObjType::MutexCond(mutex_cond))
                }
            } else if wait == libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC {
                Ok(WaitObjType::Unspec)
            } else {
                panic!("Could not retrieve wait object")
            }
        } else {
            panic!("Should not be reachable! Could not retrieve wait object")
        }
    }
}

impl<CNTR: ReadCntr> ReadCntr for Counter<CNTR> {}
impl<CNTR: WaitCntr> WaitCntr for Counter<CNTR> {}

impl<const WAIT: bool, const RETRIEVE: bool, const FD: bool> CounterImpl<WAIT, RETRIEVE, FD> {
    pub(crate) fn new<EQ: ?Sized + 'static>(
        domain: &MyRc<crate::domain::DomainImplBase<EQ>>,
        mut attr: CounterAttr,
        context: *mut std::ffi::c_void,
    ) -> Result<Self, crate::error::Error> {
        let mut c_cntr: CntrRawFid = std::ptr::null_mut();

        let err = unsafe {
            libfabric_sys::inlined_fi_cntr_open(
                domain.as_raw_typed_fid(),
                attr.get_mut(),
                &mut c_cntr,
                context,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(Self {
                c_cntr: OwnedCntrFid::from(c_cntr),
                wait_obj: Some(attr.c_attr.wait_obj),
                _domain_rc: domain.clone() as MyRc<dyn DomainImplT>,
            })
        }
    }
}

impl<const WAIT: bool, const RETRIEVE: bool, const FD: bool>
    Counter<CounterImpl<WAIT, RETRIEVE, FD>>
{
    #[allow(dead_code)]

    pub(crate) fn new<EQ: ?Sized + 'static>(
        domain: &crate::domain::DomainBase<EQ>,
        attr: CounterAttr,
        context: Option<&mut crate::Context>,
    ) -> Result<Self, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };
        Ok(Self {
            inner: MyRc::new(CounterImpl::new(&domain.inner, attr, c_void)?),
        })
    }
}

//================== Counter Builder ==================//
/// Builder for the [`Counter`] type.
///
/// `CounterBuilder` is used to configure and build a new `Counter`.
/// It encapsulates an incremental configuration of the counter, as provided by a `fi_cntr_attr`,
/// followed by a call to `fi_cntr_open`  
pub struct CounterBuilder<'a, const WAIT: bool, const RETRIEVE: bool, const FD: bool> {
    cntr_attr: CounterAttr,
    ctx: Option<&'a mut Context>,
}

impl<'a> CounterBuilder<'a, true, false, false> {
    /// Initiates the creation of new [Counter] on `domain`.
    ///
    /// The initial configuration is what would be set if no `fi_cntr_attr` or `context` was provided to
    /// the `fi_cntr_open` call.
    pub fn new() -> Self {
        Self {
            cntr_attr: CounterAttr::new(),
            ctx: None,
        }
    }
}

impl<'a> Default for CounterBuilder<'a, true, false, false> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, const WAIT: bool, const RETRIEVE: bool, const FD: bool>
    CounterBuilder<'a, WAIT, RETRIEVE, FD>
{
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
    pub fn wait_none(mut self) -> CounterBuilder<'a, false, false, false> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::None);

        CounterBuilder {
            cntr_attr: self.cntr_attr,
            ctx: self.ctx,
        }
    }

    /// Sets the underlying low-level waiting object to FD.
    ///
    /// Corresponds to setting `fi_cntr_attr::wait_obj` to `FI_WAIT_FD`.
    pub fn wait_fd(mut self) -> CounterBuilder<'a, true, true, true> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::Fd);

        CounterBuilder {
            cntr_attr: self.cntr_attr,
            ctx: self.ctx,
        }
    }

    /// Sets the underlying low-level waiting object to [crate::sync::WaitSet] `set`.
    ///
    /// Corresponds to setting `fi_cntr_attr::wait_obj` to `FI_WAIT_SET` and `fi_cntr_attr::wait_set` to `set`.
    pub fn wait_set(
        mut self,
        set: &crate::sync::WaitSet,
    ) -> CounterBuilder<'a, true, false, false> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::Set(set));

        CounterBuilder {
            cntr_attr: self.cntr_attr,
            ctx: self.ctx,
        }
    }

    /// Sets the underlying low-level waiting object to Mutex+Conditional.
    ///
    /// Corresponds to setting `fi_cntr_attr::wait_obj` to `FI_WAIT_MUTEX_COND`.
    pub fn wait_mutex(mut self) -> CounterBuilder<'a, true, true, false> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::MutexCond);

        CounterBuilder {
            cntr_attr: self.cntr_attr,
            ctx: self.ctx,
        }
    }

    /// Indicates that the counter will wait without a wait object but instead yield on every wait.
    ///
    /// Corresponds to setting `fi_cntr_attr::wait_obj` to `FI_WAIT_YIELD`.
    pub fn wait_yield(mut self) -> CounterBuilder<'a, true, false, false> {
        self.cntr_attr.wait_obj(crate::enums::WaitObj::Yield);

        CounterBuilder {
            cntr_attr: self.cntr_attr,
            ctx: self.ctx,
        }
    }

    /// Sets the context to be passed to the `Counter`.
    ///
    /// Corresponds to passing a non-NULL `context` value to `fi_cntr_open`.
    pub fn context(self, ctx: &'a mut Context) -> CounterBuilder<'a, WAIT, RETRIEVE, FD> {
        CounterBuilder {
            cntr_attr: self.cntr_attr,
            ctx: Some(ctx),
        }
    }

    /// Constructs a new [Counter] with the configurations requested so far.
    ///
    /// Corresponds to creating a `fi_cntr_attr`, setting its fields to the requested ones,
    /// and passing it to the `fi_cntr_open` call with an optional `context`.
    pub fn build<EQ: ?Sized + 'static>(
        self,
        domain: &'a crate::domain::DomainBase<EQ>,
    ) -> Result<Counter<CounterImpl<WAIT, RETRIEVE, FD>>, crate::error::Error> {
        Counter::new(domain, self.cntr_attr, self.ctx)
    }
}

//================== Trait impls ==================//

impl<T: AsFid> AsFid for Counter<T> {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.inner.as_fid()
    }
}

impl<T: AsRawFid> AsRawFid for Counter<T> {
    fn as_raw_fid(&self) -> RawFid {
        self.inner.as_raw_fid()
    }
}

impl<T: AsRawTypedFid<Output = CntrRawFid>> AsRawTypedFid for Counter<T> {
    type Output = CntrRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.inner.as_raw_typed_fid()
    }
}

impl<const WAIT: bool, const RETRIEVE: bool, const FD: bool> AsFid
    for CounterImpl<WAIT, RETRIEVE, FD>
{
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.c_cntr.as_fid()
    }
}

impl<const WAIT: bool, const RETRIEVE: bool, const FD: bool> AsRawFid
    for CounterImpl<WAIT, RETRIEVE, FD>
{
    fn as_raw_fid(&self) -> RawFid {
        self.c_cntr.as_raw_fid()
    }
}

impl<const WAIT: bool, const RETRIEVE: bool, const FD: bool> AsTypedFid<CntrRawFid>
    for CounterImpl<WAIT, RETRIEVE, FD>
{
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<'_, CntrRawFid> {
        self.c_cntr.as_typed_fid()
    }
}

impl<const WAIT: bool, const RETRIEVE: bool, const FD: bool> AsRawTypedFid
    for CounterImpl<WAIT, RETRIEVE, FD>
{
    type Output = CntrRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.c_cntr.as_raw_typed_fid()
    }
}

impl AsFd for CounterImpl<true, true, true> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        if let WaitObjType::Fd(fd) = self.wait_object().unwrap() {
            fd
        } else {
            panic!("Fabric object object type is not Fd")
        }
    }
}

impl<T: AsFd> AsFd for Counter<T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

impl<T: ReadCntr + 'static> crate::Bind for Counter<T> {
    fn inner(&self) -> MyRc<dyn AsRawFid> {
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
        self.c_attr.events = events.as_raw();

        self
    }

    pub(crate) fn wait_obj(&mut self, wait_obj: crate::enums::WaitObj) -> &mut Self {
        if let crate::enums::WaitObj::Set(wait_set) = wait_obj {
            self.c_attr.wait_set = wait_set.as_raw_typed_fid();
        }
        self.c_attr.wait_obj = wait_obj.as_raw();
        self
    }

    // pub(crate) fn flags(&mut self, flags: u64) -> &mut Self {
    //     self.c_attr.flags = flags;

    //     self
    // }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_cntr_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_cntr_attr {
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
    use crate::info::{Info, Version};

    use super::{CounterBuilder, ReadCntr};

    #[test]
    fn cntr_loop() {
        let info = Info::new(&Version {
            major: 1,
            minor: 18,
        })
        .enter_hints()
            .enter_domain_attr()
                .mode(crate::enums::Mode::all())
                .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse())
            .leave_domain_attr()
        .leave_hints()
        .get()
        .unwrap();

        for e in info.iter() {
            if e.domain_attr().cntr_cnt() != 0 {
                let fab = crate::fabric::FabricBuilder::new().build(e).unwrap();
                let domain = crate::domain::DomainBuilder::new(&fab, e).build().unwrap();
                let cntr_cnt = std::cmp::min(e.domain_attr().cntr_cnt(), 100);
                let cntrs: Vec<_> = (0..cntr_cnt)
                    .map(|_| CounterBuilder::new().build(&domain).unwrap())
                    .collect();

                for (i, cntr) in cntrs.iter().enumerate() {
                    cntr.set(i as u64).unwrap();
                    cntr.seterr((i << 1) as u64).unwrap();
                }

                for (i, cntr) in cntrs.iter().enumerate() {
                    cntr.add(i as u64).unwrap();
                    cntr.adderr(i as u64).unwrap();
                }

                for (i, cntr) in cntrs.iter().enumerate() {
                    let expected = i + i;
                    let value = cntr.read() as usize;
                    assert_eq!(expected, value);
                }

                for (i, cntr) in cntrs.iter().enumerate() {
                    let expected = (i << 1) + i;
                    let value = cntr.readerr() as usize;
                    assert_eq!(expected, value);
                }
                break;
            }
        }
    }
}

#[cfg(test)]
mod libfabric_lifetime_tests {

    use crate::{
        cntr::ReadCntr,
        info::{Info, Version},
    };

    use super::CounterBuilder;

    #[test]
    fn cntr_drops_before_domain() {
        let info = Info::new(&Version {
            major: 1,
            minor: 18,
        })
        .enter_hints()
            .enter_domain_attr()
                .mode(crate::enums::Mode::all())
                .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse())
            .leave_domain_attr()
        .leave_hints()
        .get()
        .unwrap();

        for e in info.iter() {
            if e.domain_attr().cntr_cnt() != 0 {
                let fab = crate::fabric::FabricBuilder::new().build(e).unwrap();
                let domain = crate::domain::DomainBuilder::new(&fab, e).build().unwrap();
                let cntr_cnt = std::cmp::min(e.domain_attr().cntr_cnt(), 100);
                let cntrs: Vec<_> = (0..cntr_cnt)
                    .map(|_| CounterBuilder::new().build(&domain).unwrap())
                    .collect();
                for (i, cntr) in cntrs.iter().enumerate() {
                    cntr.set(i as u64).unwrap();
                    cntr.seterr((i << 1) as u64).unwrap();
                }

                for (i, cntr) in cntrs.iter().enumerate() {
                    cntr.add(i as u64).unwrap();
                    cntr.adderr(i as u64).unwrap();
                }

                for (i, cntr) in cntrs.iter().enumerate() {
                    let expected = i + i;
                    let value = cntr.read() as usize;
                    assert_eq!(expected, value);
                }

                for (i, cntr) in cntrs.iter().enumerate() {
                    let expected = (i << 1) + i;
                    let value = cntr.readerr() as usize;
                    assert_eq!(expected, value);
                }
                drop(domain);
                break;
            }
        }
    }
}
