use std::{rc::Rc, cell::OnceCell};

#[allow(unused_imports)] 
use crate::fid::AsFid;
use crate::{domain::{Domain, DomainImplT}, eqoptions::EqConfig, fid::{AsRawFid, self, AvRawFid, OwnedAVFid, AsRawTypedFid, AsTypedFid, OwnedAVSetFid, AVSetRawFid, RawFid}, FI_ADDR_NOTAVAIL, ep::Address, eq::{EventQueue, EventQueueImpl, EventQueueImplT}, enums::{AVOptions, AVSetOptions}, RawMappedAddress, MappedAddress, cq::CompletionQueueImplT, AddressSource};


// impl Drop for AddressVector {
//     fn drop(&mut self) {
//        println!("Dropping AddressVector\n");
//     }
// }
//================== AddressVector ==================//

pub(crate) trait AddressVectorImplT {}

impl<EQ: ?Sized> AddressVectorImplT for AddressVectorImplBase<EQ> {}
pub(crate) struct AddressVectorImplBase<EQ: ?Sized> {
    pub(crate) c_av: OwnedAVFid, 
    pub(crate) _eq_rc: OnceCell<Rc<EQ>>,
    pub(crate) _domain_rc: Rc<dyn DomainImplT>,
}
pub(crate) type AddressVectorImpl = AddressVectorImplBase<dyn EventQueueImplT>;



impl<EQ: ?Sized + EventQueueImplT> AddressVectorImplBase<EQ> {

    pub(crate) fn new<DEQ: ?Sized + 'static, T>(domain: &Rc<crate::domain::DomainImplBase<DEQ>>, mut attr: AddressVectorAttr, context: Option<&mut T>) -> Result<Self, crate::error::Error> {
        let mut c_av:   AvRawFid =  std::ptr::null_mut();

        let err = 
        if let Some(ctx) = context {
            unsafe { libfabric_sys::inlined_fi_av_open(domain.as_raw_typed_fid(), attr.get_mut(), &mut c_av, ctx as *mut T as *mut std::ffi::c_void) }
        }
        else {
            unsafe { libfabric_sys::inlined_fi_av_open(domain.as_raw_typed_fid(), attr.get_mut(), &mut c_av, std::ptr::null_mut()) }
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(
                Self {
                    c_av: OwnedAVFid::from(c_av),
                    _eq_rc: OnceCell::new(),
                    _domain_rc: domain.clone(),
                }
            )
        }
    }
}
impl<EQ: ?Sized + EventQueueImplT> AddressVectorImplBase<EQ> {

    /// Associates an [EventQueue](crate::eq::EventQueue) with the AddressVector.
    /// 
    /// This method directly corresponds to a call to `fi_av_bind(av, eq, 0)`.
    /// # Errors
    ///
    /// This function will return an error if the underlying library call fails.
    pub(crate) fn bind(&self, eq: &Rc<EQ>) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_av_bind(self.as_raw_typed_fid(), eq.as_raw_fid(), 0) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            if self._eq_rc.set(eq.clone()).is_err() {
                panic!("AddressVector is alread bound to an EventQueue");
            }
            Ok(())
        }
    }
}
impl<EQ: ?Sized> AddressVectorImplBase<EQ> {

    fn insert<T>(&self, addr: &[Address], flags: u64, ctx: Option<&mut T>) -> Result<Vec<RawMappedAddress>, crate::error::Error> { // [TODO] //[TODO] Handle flags, handle context, handle async

        let mut fi_addresses = vec![0u64; addr.len()];
        let total_size = addr.iter().fold(0, |acc, addr| acc + addr.as_bytes().len() );
        let mut serialized: Vec<u8> = Vec::with_capacity(total_size);
        for a in addr {
            serialized.extend(a.as_bytes().iter())
        }

        let err = if let Some(ctx) = ctx {
            unsafe { libfabric_sys::inlined_fi_av_insert(self.as_raw_typed_fid(), serialized.as_ptr().cast(), fi_addresses.len(), fi_addresses.as_mut_ptr().cast(), flags, (ctx as *mut T).cast()) }
        }
        else {
             unsafe { libfabric_sys::inlined_fi_av_insert(self.as_raw_typed_fid(), serialized.as_ptr().cast(), fi_addresses.len(), fi_addresses.as_mut_ptr().cast(), flags, std::ptr::null_mut()) }
        };

        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            // let mapped_addresses = fi_addresses.into_iter().map(|fi_addr| if fi_addr == FI_ADDR_NOTAVAIL {None} else {Some(MappedAddress::from_raw_addr(fi_addr, self))}).collect::<Vec<_>>();
            Ok(fi_addresses)
        }
    }

    pub(crate) fn insertsvc(&self, node: &str, service: &str, flags: u64) -> Result<RawMappedAddress, crate::error::Error> {
        let mut fi_addr = 0u64;
        let err = unsafe { libfabric_sys::inlined_fi_av_insertsvc(self.as_raw_typed_fid(), node.as_bytes().as_ptr().cast(), service.as_bytes().as_ptr().cast(), &mut fi_addr, flags, std::ptr::null_mut())  };


        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(fi_addr)
        }
    }

    pub(crate) fn insertsym(&self, node: &str, nodecnt :usize, service: &str, svccnt: usize, flags: u64) -> Result<Vec<RawMappedAddress>, crate::error::Error> { // [TODO] Handle case where operation partially failed
        let total_cnt = nodecnt * svccnt;
        let mut fi_addresses = vec![0u64; total_cnt];
        let err = unsafe { libfabric_sys::inlined_fi_av_insertsym(self.as_raw_typed_fid(), node.as_bytes().as_ptr().cast(), nodecnt, service.as_bytes().as_ptr().cast(), svccnt, fi_addresses.as_mut_ptr().cast(), flags, std::ptr::null_mut())  };

        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            // let mapped_addresses = fi_addresses.into_iter().map(|fi_addr| if fi_addr == FI_ADDR_NOTAVAIL {None} else {Some(MappedAddress::from_raw_addr(fi_addr))}).collect::<Vec<_>>();
            Ok(fi_addresses)
        }
    }

    pub(crate) fn remove(&self, addr: Vec<crate::MappedAddress>) -> Result<(), crate::error::Error> {
        let mut fi_addresses =  addr.into_iter().map(|mapped_addr| {mapped_addr.raw_addr()}).collect::<Vec<u64>>();
        
        let err = unsafe { libfabric_sys::inlined_fi_av_remove(self.as_raw_typed_fid(), fi_addresses.as_mut_ptr().cast(), fi_addresses.len(), 0) };

        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub(crate) fn lookup(&self, mapped_addr: crate::MappedAddress) -> Result<Address, crate::error::Error> {
        let mut addrlen : usize = 0;
        let err = unsafe { libfabric_sys::inlined_fi_av_lookup(self.as_raw_typed_fid(), mapped_addr.raw_addr(), std::ptr::null_mut(), &mut addrlen) };
        
        if -err as u32  == libfabric_sys::FI_ETOOSMALL {
            let mut addr = vec![0u8; addrlen];
            let err = unsafe { libfabric_sys::inlined_fi_av_lookup(self.as_raw_typed_fid(), mapped_addr.raw_addr(), addr.as_mut_ptr().cast(), &mut addrlen) };

            if err < 0 {
                Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
            }
            else {
                Ok(unsafe {Address::from_bytes(&addr)} )
            }
        }
        else {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
    }

    pub(crate) fn straddr(&self, addr: &Address) -> String {
        let mut addr_str: Vec<u8> = Vec::new();
        let mut strlen = addr_str.len();
        let strlen_ptr: *mut usize = &mut strlen;
        unsafe { libfabric_sys::inlined_fi_av_straddr(self.as_raw_typed_fid(), addr.as_bytes().as_ptr().cast(), addr_str.as_mut_ptr().cast(), strlen_ptr) };
        addr_str.resize(strlen, 1);
        
        let mut strlen = addr_str.len();
        let strlen_ptr: *mut usize = &mut strlen;
        unsafe { libfabric_sys::inlined_fi_av_straddr(self.as_raw_typed_fid(), addr.as_bytes().as_ptr().cast(), addr_str.as_mut_ptr().cast(), strlen_ptr) };
        std::ffi::CString::from_vec_with_nul(addr_str).unwrap().into_string().unwrap()
    }
}



/// Owned wrapper around a libfabric `fid_av`.
/// 
/// This type wraps an instance of a `fid_av`, monitoring its lifetime and closing it when it goes out of scope.
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.19.0/man/fi_av.3.html).
/// 
/// Note that other objects that rely on an AddressVector (e.g., [MappedAddress]) will extend its lifetime until they
/// are also dropped.
pub type AddressVector = AddressVectorBase<dyn EventQueueImplT>;
pub struct AddressVectorBase<EQ: ?Sized + EventQueueImplT> {
    pub(crate) inner: Rc<AddressVectorImplBase<EQ>>,
}

impl<EQ: EventQueueImplT + ?Sized + 'static> AddressVectorBase<EQ> {

    #[allow(dead_code)]
    pub(crate) fn from_impl(av_impl: &Rc<AddressVectorImplBase<EQ>>) -> Self {
        Self {
            inner: av_impl.clone(),
        }
    }

    pub(crate) fn new<DEQ: ?Sized + 'static, T>(domain: &crate::domain::DomainBase<DEQ>, attr: AddressVectorAttr, context: Option<&mut T>) -> Result<Self, crate::error::Error> {
        
        Ok(
            Self {
                inner: Rc::new (AddressVectorImplBase::new(&domain.inner, attr, context)?)
            }
        )
    }

    /// Insert one or more [Address]es into the [AddressVector] and return a [Vec] of [MappedAddress]es, one for each input address.
    /// 
    /// The operation can be modified using the requested `options` as defined in [AVOptions].
    /// For address(es) that could not be mapped a [None] value will be returned at the respective index.
    /// 
    /// This method directly corresponds to a call to `fi_av_insert`
    pub fn insert(&self, addr: &[Address], options: AVOptions) -> Result<Vec<Option<MappedAddress>>, crate::error::Error> { // [TODO] handle async
        let fi_addresses = self.inner.insert::<()>(addr, options.get_value(), None)?;
        Ok(fi_addresses.into_iter().map(|fi_addr| if fi_addr == FI_ADDR_NOTAVAIL {None} else {Some(MappedAddress::from_raw_addr(fi_addr, AddressSource::Av(self.inner.clone())))}).collect::<Vec<_>>())
    }
    
    /// Same as [Self::insert] but with an extra argument to provide a context
    ///
    pub fn insert_with_context<T>(&self, addr: &[Address], options: AVOptions, ctx: &mut T) -> Result<Vec<Option<MappedAddress>>, crate::error::Error> { // [TODO] handle async
        let fi_addresses = self.inner.insert(addr, options.get_value(), Some(ctx))?;
        Ok(fi_addresses.into_iter().map(|fi_addr| if fi_addr == FI_ADDR_NOTAVAIL {None} else {Some(MappedAddress::from_raw_addr(fi_addr, AddressSource::Av(self.inner.clone())))}).collect::<Vec<_>>())
    }

    /// Similar to [Self::insert] but with address formatted as node, service [String]s
    ///
    /// Directly corrsponds to `fi_av_insertsvc`
    pub fn insertsvc(&self, node: &str, service: &str, options: AVOptions) -> Result<Option<MappedAddress>, crate::error::Error> {
        let fi_addr = self.inner.insertsvc(node, service, options.get_value())?;
        if fi_addr != FI_ADDR_NOTAVAIL {
            Ok(Some(MappedAddress::from_raw_addr(fi_addr, AddressSource::Av(self.inner.clone()))))
        }
        else {
            Ok(None)
        }
        
    }

    /// Similar to [Self::insert] but with address(es) formatted as a base `node` + increments up to `nodecnt`, base `service`  + increments up to `svccnt`
    ///
    /// Directly corresponds to `fi_av_insertsym`
    pub fn insertsym(&self, node: &str, nodecnt :usize, service: &str, svccnt: usize, options: AVOptions) -> Result<Vec<Option<MappedAddress>>, crate::error::Error> { // [TODO] Handle case where operation partially failed
        let fi_addresses = self.inner.insertsym(node, nodecnt, service, svccnt, options.get_value())?;
        Ok(fi_addresses.into_iter().map(|fi_addr| if fi_addr == FI_ADDR_NOTAVAIL {None} else {Some(MappedAddress::from_raw_addr(fi_addr, AddressSource::Av(self.inner.clone())))}).collect::<Vec<_>>())
    }

    /// Removes the given [MappedAddress]es from the AddressVector. 
    /// 
    /// This method will consume the mapped addresses passed to it to prevent their reuse.
    /// 
    /// Directly corresponds to `fi_av_remove`
    pub fn remove(&self, addr: Vec<crate::MappedAddress>) -> Result<(), crate::error::Error> {
        self.inner.remove(addr)
    }
    
    /// Retrieves an address stored in the address vector.
    /// 
    /// Directly corresponds to `fi_av_lookup`
    pub fn lookup(&self, mapped_addr: crate::MappedAddress) -> Result<Address, crate::error::Error> {
        self.inner.lookup(mapped_addr)
    }
    
    /// Convert an [Address] into a printable string.
    ///
    /// Directly corresponds to `fi_av_straddr`
    pub fn straddr(&self, addr: &Address) -> String {
        self.inner.straddr(addr)
    }
    
}



/// Builder for the [`AddressVector`] type.
/// 
/// `AddressVectorBuilder` is used to configure and build a new `AddressVector`.
/// It encapsulates an incremental configuration of the address vector, as provided by a `fi_av_attr`,
/// followed by a call to `fi_av_open`  
pub struct AddressVectorBuilder<'a, T, EQ: ?Sized> {
    av_attr: AddressVectorAttr,
    eq: Option<&'a Rc<EQ>>,
    ctx: Option<&'a mut T>,
    domain: &'a Domain,
}


impl<'a> AddressVectorBuilder<'a, (), ()> {
    
    /// Initiates the creation of a new [AddressVector] on `domain`.
    /// 
    /// The initial configuration is what would be set if no `fi_av_attr` or `context` was provided to 
    /// the `fi_av_open` call. 
    pub fn new(domain: &'a Domain) -> AddressVectorBuilder<'a, (), ()> {
        AddressVectorBuilder {
            av_attr: AddressVectorAttr::new(),
            eq: None,
            ctx: None,
            domain,
        }
    }
}

impl<'a, T, EQ> AddressVectorBuilder<'a, T, EQ> {


    /// Sets the type of the [AddressVector].
    /// 
    /// Corresponds to setting field `fi_av_attr::type`
    pub fn type_(mut self, av_type: crate::enums::AddressVectorType) -> Self {
        self.av_attr.type_(av_type);
        self
    }

    /// Sets address bits to identify rx ctx of the [AddressVector].
    /// 
    /// Corresponds to setting field `fi_av_attr::rx_ctx_bits`
    pub fn rx_ctx_bits(mut self, rx_ctx_bits: i32) -> Self { //[TODO] Maybe wrap bitfield
        self.av_attr.rx_ctx_bits(rx_ctx_bits);
        self
    }

    /// Sets the number of [Address]es that will be inserted into the [AddressVector]
    /// 
    /// Corresponds to setting field `fi_av_attr::count`
    pub fn count(mut self, count: usize) -> Self {
        self.av_attr.count(count);
        self
    }

    /// Sets the number of [Endpoint][crate::ep::Endpoint]s that will be inserted into the [AddressVector]
    /// 
    /// Corresponds to setting field `fi_av_attr::ep_per_node`
    pub fn ep_per_node(mut self, count: usize) -> Self {
        self.av_attr.ep_per_node(count);
        self
    }


    /// Sets the system name of the [AddressVector] to `name`.
    /// 
    /// Corresponds to setting field `fi_av_attr::name`
    pub fn name(mut self, name: String) -> Self {
        self.av_attr.name(name);
        self 
    }

    /// Sets the base mmap address of the [AddressVector] to `addr`.
    /// 
    /// Corresponds to setting field `fi_av_attr::map_addr`
    pub fn map_addr(mut self, addr: usize) -> Self {
        self.av_attr.map_addr(addr);
        self
    }

    /// Sets the [AddressVector] to read-only mode.
    /// 
    /// Corresponds to setting the corresponding bit (`FI_READ`) of the field `fi_av_attr::flags`
    pub fn read_only(mut self) -> Self {
        self.av_attr.read_only();
        self
    }
}

impl<'a, T, EQ: EventQueueImplT> AddressVectorBuilder<'a, T, EQ> {

    /// Requests that insertions to [AddressVector] be done asynchronously.
    /// 
    /// An asynchronous address vector is required to be bound to an [EventQueue] before any insertions take place.
    /// Thus, setting this option requires the user to specify the queue that will be used to report the completion
    /// of address insertions.
    /// 
    /// Corresponds to setting the corresponding bit (`FI_EVENT`) of the field `fi_av_attr::flags` and calling
    /// `fi_av_bind(eq)`, once the address vector has been constructed.
    pub fn async_(mut self, eq: &'a EventQueue<EQ>) -> Self {
        self.av_attr.async_();
        self.eq = Some(&eq.inner);
        self
    }
}

impl<'a, T, EQ> AddressVectorBuilder<'a, T, EQ> {

    /// Indicates that each node will be associated with the same number of endpoints.
    /// 
    /// Corresponds to setting the corresponding bit (`FI_SYMMETRIC`) of the field `fi_av_attr::flags`.
    pub fn symmetric(mut self) -> Self {
        self.av_attr.symmetric();
        self
    }

    /// Sets the context to be passed to the [AddressVector].
    /// 
    /// Corresponds to passing a non-NULL `context` value to `fi_av_open`.
    pub fn context(self, ctx: &'a mut T) -> AddressVectorBuilder<'a, T, EQ> {
        AddressVectorBuilder {
            av_attr: self.av_attr,
            domain: self.domain,
            eq: self.eq,
            ctx: Some(ctx),
        }
    }
}
impl<'a, T> AddressVectorBuilder<'a, T, ()> {

    /// Constructs a new [AddressVector] with the configurations requested so far.
    /// 
    /// Corresponds to creating an `fi_av_attr`, setting its fields to the requested ones,
    /// calling `fi_av_open` with an optional `context`, and, if asynchronous, binding with
    /// the selected [EventQueue].
    pub fn build(self) -> Result<AddressVector, crate::error::Error> {
        let av = AddressVector::new(self.domain, self.av_attr, self.ctx)?;
        Ok(av)
        // match self.eq {
        //     None => Ok(av),
        //     Some(eq) => {av.inner.bind(eq)?; Ok(av)}
        // }
    }
    
}
impl<'a, T, EQ: ?Sized + EventQueueImplT + 'static> AddressVectorBuilder<'a, T, EQ> {

    /// Constructs a new [AddressVector] with the configurations requested so far.
    /// 
    /// Corresponds to creating an `fi_av_attr`, setting its fields to the requested ones,
    /// calling `fi_av_open` with an optional `context`, and, if asynchronous, binding with
    /// the selected [EventQueue].
    pub fn build(self) -> Result<AddressVectorBase<EQ>, crate::error::Error> {
        let av = AddressVectorBase::new(self.domain, self.av_attr, self.ctx)?;
        match self.eq {
            None => Ok(av),
            Some(eq) => {av.inner.bind(eq)?; Ok(av)}
        }
    }
    
}

//================== AddressVectorSet ==================//

pub(crate) struct AddressVectorSetImpl {
    pub(crate) c_set : OwnedAVSetFid,
    _av_rc: Rc<dyn AddressVectorImplT>,
}



impl AddressVectorSetImpl {

    fn new<EQ: AsRawFid + 'static + ?Sized + EventQueueImplT, T>(av: &AddressVectorBase<EQ>, mut attr: AddressVectorSetAttr, context: Option<&mut T>) -> Result<Self, crate::error::Error> {
        let mut c_set: AVSetRawFid = std::ptr::null_mut();

        let err = 
        if let Some(ctx) = context {
            unsafe { libfabric_sys::inlined_fi_av_set(av.as_raw_typed_fid(), attr.get_mut(), &mut c_set, (ctx as *mut T).cast()) }
        }
        else {
            unsafe { libfabric_sys::inlined_fi_av_set(av.as_raw_typed_fid(), attr.get_mut(), &mut c_set, std::ptr::null_mut()) }
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    c_set: OwnedAVSetFid::from(c_set ),
                    _av_rc: av.inner.clone(),
                })
        }
    }

    pub(crate) fn union(&self, other: &AddressVectorSetImpl) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_union(self.as_raw_typed_fid(), other.as_raw_typed_fid()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub(crate) fn intersect(&self, other: &AddressVectorSetImpl) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_intersect(self.as_raw_typed_fid(), other.as_raw_typed_fid()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub(crate) fn diff(&self, other: &AddressVectorSetImpl) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_diff(self.as_raw_typed_fid(), other.as_raw_typed_fid()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub(crate) fn insert(&self, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_insert(self.as_raw_typed_fid(), mapped_addr.raw_addr()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub(crate) fn remove(&self, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_remove(self.as_raw_typed_fid(), mapped_addr.raw_addr()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub(crate) fn get_addr(&self) -> Result<RawMappedAddress, crate::error::Error> {
        let mut addr = 0u64;
        // let addr_ptr: *mut crate::MappedAddress = &mut addr;
        let err = unsafe { libfabric_sys::inlined_fi_av_set_addr(self.as_raw_typed_fid(), &mut addr) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(addr)
        }
    }
}

/// Owned wrapper around a libfabric `fid_av_set`.
/// 
/// This type wraps an instance of a `fid_av_set`, monitoring its lifetime and closing it when it goes out of scope.
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.19.0/man/fi_av_set.3.html).
/// 
/// Note that other objects that rely on an AddressVectorSet (e.g., [crate::comm::collective::MulticastGroupCollective]) will extend its lifetime until they
/// are also dropped.

pub struct AddressVectorSet {
    inner: Rc<AddressVectorSetImpl>,
}

impl AddressVectorSet {

    pub(crate) fn new<EQ: AsRawFid + 'static + ?Sized + EventQueueImplT, T>(av: &AddressVectorBase<EQ>, attr: AddressVectorSetAttr, context: Option<&mut T>) -> Result<Self, crate::error::Error> {
        Ok(
            Self {
                inner: 
                    Rc::new(AddressVectorSetImpl::new(av, attr, context)?)
            }
        )
    }

    /// Perform a set union operation on two AV sets
    /// 
    /// The result is stored in `Self`, which is modified.
    /// 
    /// Corresponds to `fi_av_set_union`
    pub fn union(&mut self, other: &AddressVectorSet) -> Result<(), crate::error::Error> {
        self.inner.union(&other.inner)
    }

    /// Perform a set intersection operation on two AV sets
    /// 
    /// The result is stored in `Self`, which is modified.
    /// 
    /// Corresponds to `fi_av_set_intersect`
    pub fn intersect(&mut self, other: &AddressVectorSet) -> Result<(), crate::error::Error> {
        self.inner.intersect(&other.inner)
    }
    
    /// Perform a set difference operation on two AV sets
    /// 
    /// The result is stored in `Self`, which is modified.
    /// 
    /// Corresponds to `fi_av_set_diff`
    pub fn diff(&mut self, other: &AddressVectorSet) -> Result<(), crate::error::Error> {
        self.inner.diff(&other.inner)
    }
    
    /// Adds an address to the [AddressVectorSet].
    /// 
    /// `Self` is modified.
    /// 
    /// Corresponds to `fi_av_set_insert`
    pub fn insert(&mut self, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        self.inner.insert(mapped_addr)
    }
    
    /// Removes an address to the [AddressVectorSet].
    /// 
    /// `Self` is modified.
    /// 
    /// Corresponds to `fi_av_set_remove`
    pub fn remove(&mut self, mapped_addr: &crate::MappedAddress) -> Result<(), crate::error::Error> {
        self.inner.remove(mapped_addr)
    }
    
    pub fn get_addr(&self) -> Result<crate::MappedAddress, crate::error::Error> {
        let raw_addr = self.inner.get_addr()?;
        Ok(MappedAddress::from_raw_addr(raw_addr, AddressSource::AvSet(self.inner.clone())))
    }    
}

/// Builder for the AddressVectorSet type.
/// 
/// `AddressVectorSetBuilder` is used to configure and build a new [AddressVectorSet].
/// It encapsulates an incremental configuration of the address vector set, as provided by a `fi_av_set_attr`,
/// followed by a call to `fi_av_set`  
pub struct AddressVectorSetBuilder<'a, T> {
    avset_attr: AddressVectorSetAttr,
    ctx: Option<&'a mut T>,
    av: &'a AddressVector,
}

impl<'a> AddressVectorSetBuilder<'a, ()> {
    pub fn new(av: &'a AddressVector) -> AddressVectorSetBuilder<'a, ()> {
        AddressVectorSetBuilder {
            avset_attr: AddressVectorSetAttr::new(),
            ctx: None,
            av,
        }
    }
}

impl<'a, T> AddressVectorSetBuilder<'a, T> {

    /// Indicates the expected the number of members that will be a part of the AV set.
    /// 
    /// Corresponds to setting the `fi_av_set_attr::count` field.
    pub fn count(mut self, size: usize) -> Self {

        self.avset_attr.count(size);
        self
    }

    /// Indicates the start address to include to the the AV set.
    /// 
    /// Corresponds to setting the `fi_av_set_attr::start_addr` field.
    pub fn start_addr(mut self, mapped_addr: &crate::MappedAddress) -> Self { // [TODO] Merge with end_addr + stride
        
        self.avset_attr.start_addr(mapped_addr);
        self
    }

    /// Indicates the end address to include to the the AV set.
    /// 
    /// Corresponds to setting the `fi_av_set_attr::end_addr` field.
    pub fn end_addr(mut self, mapped_addr: &crate::MappedAddress) -> Self {
        
        self.avset_attr.end_addr(mapped_addr);
        self
    }

    /// The number of entries between successive addresses included in the AV set.
    /// 
    /// Corresponds to setting the `fi_av_set_attr::stride` field.
    pub fn stride(mut self, stride: usize) -> Self {

        self.avset_attr.stride(stride);
        self
    }

    /// If supported by the fabric, this represents a key associated with the AV set. 
    /// 
    /// Corresponds to setting the `fi_av_set_attr::comm_key` and `fi_av_set_attr::comm_key_size` fields. 
    pub fn comm_key(mut self, key: &mut [u8]) -> Self {
        
        self.avset_attr.comm_key(key);
        self
    }

    /// May be used to configure the AV set, including restricting which collective operations the AV set needs to support.
    /// 
    /// `options` captures the [flags](AVSetOptions) that can be possibly set for an AV set.
    /// 
    /// Corresponds to setting the `fi_av_set_attr::flags` field.
    pub fn options(mut self, options: AVSetOptions) -> Self { //[TODO] We should provide different function for each bitflag. 

        self.avset_attr.options(options);
        self
    }

    /// Sets the context to be passed to the AV set.
    /// 
    /// Corresponds to passing a non-NULL `context` value to `fi_av_set`.
    pub fn context(self, ctx: &'a mut T) -> AddressVectorSetBuilder<'a, T> {
        AddressVectorSetBuilder {
            avset_attr: self.avset_attr,
            av: self.av,
            ctx: Some(ctx),
        }
    }

    /// Constructs a new [AddressVectorSet] with the configurations requested so far.
    /// 
    /// Corresponds to creating an `fi_av_set_attr`, setting its fields to the requested ones,
    /// passing it to a `fi_av_set` call with an optional `context` (set by [Self::context]).
    pub fn build(self) -> Result<AddressVectorSet, crate::error::Error> {
        AddressVectorSet::new(self.av, self.avset_attr, self.ctx)
    }
}

//================== Attribute Structs ==================//

pub struct AddressVectorAttr {
    pub(crate) c_attr: libfabric_sys::fi_av_attr, 
}

impl AddressVectorAttr {
    pub fn new() -> Self {
        let c_attr = libfabric_sys::fi_av_attr{
            type_: crate::enums::AddressVectorType::Unspec.get_value(), 
            rx_ctx_bits: 0,
            count: 0,
            ep_per_node: 0,
            name: std::ptr::null(),
            map_addr: std::ptr::null_mut(),
            flags: 0
        };

        Self { c_attr }
    }

    pub fn type_(&mut self, av_type: crate::enums::AddressVectorType) -> &mut Self {
        self.c_attr.type_ = av_type.get_value();
        self
    }

    pub fn rx_ctx_bits(&mut self, rx_ctx_bits: i32) -> &mut Self {
        self.c_attr.rx_ctx_bits = rx_ctx_bits;
        self
    }

    pub fn count(&mut self, count: usize) -> &mut Self {
        self.c_attr.count = count;
        self
    }
    
    pub fn ep_per_node(&mut self, count: usize) -> &mut Self {
        self.c_attr.ep_per_node = count;
        self
    }

    pub fn name(&mut self, name: String) -> &mut Self {
        let c_str = std::ffi::CString::new(name).unwrap();
        self.c_attr.name = c_str.into_raw();
        self 
    }

    pub fn map_addr(&mut self, addr: usize) -> &mut Self { //[TODO] Datatype correct??
        self.c_attr.map_addr = addr as *mut std::ffi::c_void;
        self
    }

    pub fn read_only(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_READ as u64;
        self
    }

    pub fn symmetric(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_SYMMETRIC;
        self
    }

    pub fn async_(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_EVENT as u64;
        self
    }


    #[allow(dead_code)]
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_av_attr {
        &self.c_attr
    }   

    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_av_attr {
        &mut self.c_attr
    }  
}

impl Default for AddressVectorAttr {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct AddressVectorSetAttr {
    c_attr: libfabric_sys::fi_av_set_attr,
}


impl AddressVectorSetAttr {

    pub(crate) fn new() -> Self {
        Self {
            c_attr: libfabric_sys::fi_av_set_attr {
                count: 0,
                start_addr: 0,
                end_addr: 0,
                stride: 0,
                comm_key_size: 0,
                comm_key: std::ptr::null_mut(),
                flags: 0,
            }
        }
    }

    pub(crate) fn count(&mut self, size: usize) -> &mut Self {

        self.c_attr.count = size;
        self
    }

    pub(crate) fn start_addr(&mut self, mapped_addr: &crate::MappedAddress) -> &mut Self {
        
        self.c_attr.start_addr = mapped_addr.raw_addr();
        self
    }

    pub(crate) fn end_addr(&mut self, mapped_addr: &crate::MappedAddress) -> &mut Self {
        
        self.c_attr.end_addr = mapped_addr.raw_addr();
        self
    }

    pub(crate) fn stride(&mut self, stride: usize) -> &mut Self {

        self.c_attr.stride = stride as u64;
        self
    }

    pub(crate) fn comm_key(&mut self, key: &mut [u8]) -> &mut Self {
        
        self.c_attr.comm_key_size = key.len();
        self.c_attr.comm_key = key.as_mut_ptr();
        self
    }

    pub(crate) fn options(&mut self, options: AVSetOptions) -> &mut Self {

        self.c_attr.flags = options.get_value();
        self
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_av_set_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_av_set_attr {
        &mut self.c_attr
    }    
}

impl Default for AddressVectorSetAttr {
    fn default() -> Self {
        Self::new()
    }
}


//================== Trait Impls ==================//
impl<EQ: ?Sized + AsRawFid + EventQueueImplT> AsFid for AddressVectorBase<EQ> {
    fn as_fid(&self) -> fid::BorrowedFid {
        self.inner.as_fid()
    }
}

impl<EQ: ?Sized + AsRawFid + EventQueueImplT> AsRawFid for AddressVectorBase<EQ> {
    fn as_raw_fid(&self) -> RawFid {
        self.inner.as_raw_fid()
    }
}


impl AsFid for AddressVectorSet {
    fn as_fid(&self) -> fid::BorrowedFid {
        self.inner.c_set.as_fid()
    }
}

impl AsTypedFid<AVSetRawFid> for AddressVectorSet {
    
    #[inline]
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<AVSetRawFid> {
        self.inner.as_typed_fid()
    }
}

impl AsRawTypedFid for AddressVectorSet {
    type Output = AVSetRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.inner.as_raw_typed_fid()
    }
}


impl AsTypedFid<AVSetRawFid> for AddressVectorSetImpl {
    
    #[inline]
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<AVSetRawFid> {
        self.c_set.as_typed_fid()
    }
}

impl AsRawTypedFid for AddressVectorSetImpl {
    type Output = AVSetRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.c_set.as_raw_typed_fid()
    }
}

impl AsFid for AddressVectorSetImpl {
    fn as_fid(&self) -> fid::BorrowedFid {
        self.c_set.as_fid()
    }
}

impl AsFid for Rc<AddressVectorSetImpl> {
    fn as_fid(&self) -> fid::BorrowedFid {
        self.c_set.as_fid()
    }
}



impl<EQ: AsRawFid + EventQueueImplT> AsTypedFid<AvRawFid> for AddressVectorBase<EQ> {
    
    #[inline]
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<AvRawFid> {
        self.inner.as_typed_fid()
    }
}

impl<EQ: ?Sized + AsRawFid + EventQueueImplT> AsRawTypedFid for AddressVectorBase<EQ> {
    type Output = AvRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.inner.as_raw_typed_fid()
    }
}


impl<EQ> AsTypedFid<AvRawFid> for AddressVectorImplBase<EQ> {
    
    #[inline]
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<AvRawFid> {
        self.c_av.as_typed_fid()
    }
}

impl<EQ: ?Sized> AsRawTypedFid for AddressVectorImplBase<EQ> {
    type Output = AvRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.c_av.as_raw_typed_fid()
    }
}

impl<EQ: ?Sized> AsFid for AddressVectorImplBase<EQ> {
    fn as_fid(&self) -> fid::BorrowedFid {
        self.c_av.as_fid()
    }
}

impl<EQ: ?Sized> AsRawFid for AddressVectorImplBase<EQ> {
    fn as_raw_fid(&self) -> RawFid {
        self.c_av.as_raw_fid()
    }
}

impl<EQ: ?Sized> AsFid for Rc<AddressVectorImplBase<EQ>> {
    fn as_fid(&self) -> fid::BorrowedFid {
        self.c_av.as_fid()
    }
}

impl<EQ: ?Sized> crate::BindImpl for AddressVectorImplBase<EQ> {}

impl<EQ: ?Sized + 'static + AsRawFid + EventQueueImplT> crate::Bind for AddressVectorBase<EQ> {
    fn inner(&self) -> Rc<dyn crate::BindImpl> {
        self.inner.clone()
    }
}


//================== Tests ==================//

#[cfg(test)]
mod tests {
    use crate::info::{InfoHints, Info};

    use super::AddressVectorBuilder;

    #[test]
    fn av_open_close() {
        let mut ep_attr = crate::ep::EndpointAttr::new();
            ep_attr.ep_type(crate::enums::EndpointType::Rdm);
    
        let mut dom_attr = crate::domain::DomainAttr::new();
            dom_attr
            .mode(crate::enums::Mode::all())
            .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse());

        let hints = InfoHints::new()
            .ep_attr(ep_attr)
            .domain_attr(dom_attr);

        let info = Info::new().hints(&hints).request().unwrap();
        let entries = info.get();
        if !entries.is_empty() {
        
            let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
            let domain = crate::domain::DomainBuilder::new(&fab, &entries[0]).build().unwrap();
        
            for i in 0..17 {
                let count = 1 << i;
                let _av = AddressVectorBuilder::new(&domain)
                    .type_(crate::enums::AddressVectorType::Map)
                    .count(count)
                    .build()
                    .unwrap();
            }
        }
        else {
            panic!("No capable fabric found!");
        }
    }

    #[test]
    fn av_good_sync() {
        
        let mut ep_attr = crate::ep::EndpointAttr::new();
            ep_attr.ep_type(crate::enums::EndpointType::Rdm);

        let mut dom_attr = crate::domain::DomainAttr::new();
            dom_attr
            .mode(crate::enums::Mode::all())
            .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse());

        let hints = InfoHints::new()
            .ep_attr(ep_attr)
            .domain_attr(dom_attr);

        let info = Info::new()
            .hints(&hints).request().unwrap();

        let entries = info.get();
        if !entries.is_empty() {
            let fab: crate::fabric::Fabric = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
            let domain = crate::domain::DomainBuilder::new(&fab, &entries[0]).build().unwrap();
            let _av = AddressVectorBuilder::new(&domain)
                .type_(crate::enums::AddressVectorType::Map)
                .count(32)
                .build()
                .unwrap();
        }
        else {
            panic!("No capable fabric found!");
        }
    }
}

#[cfg(test)]
mod libfabric_lifetime_tests {
    use crate::info::{InfoHints, Info};

    use super::AddressVectorBuilder;

    #[test]
    fn av_drops_before_domain() {
        
        let mut ep_attr = crate::ep::EndpointAttr::new();
            ep_attr.ep_type(crate::enums::EndpointType::Rdm);
    
        let mut dom_attr = crate::domain::DomainAttr::new();
            dom_attr
            .mode(crate::enums::Mode::all())
            .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse());

        let hints = InfoHints::new()
            .ep_attr(ep_attr)
            .domain_attr(dom_attr);

        let info = Info::new().hints(&hints).request().unwrap();
        let entries = info.get();
        if !entries.is_empty() {
        
            let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
            let domain = crate::domain::DomainBuilder::new(&fab, &entries[0]).build().unwrap();
        
            let mut avs = Vec::new();
            for i in 0..17 {
                let count = 1 << i;
                let av = AddressVectorBuilder::new(&domain)
                    .type_(crate::enums::AddressVectorType::Map)
                    .count(count)
                    .build()
                    .unwrap();
                avs.push(av);
                println!("Count = {}", std::rc::Rc::strong_count(&domain.inner));
            }
            drop(domain);
            println!("Count = {} After dropping domain", std::rc::Rc::strong_count(&avs[0].inner._domain_rc));
        }
        else {
            panic!("No capable fabric found!");
        }
    }
}