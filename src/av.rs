use std::rc::Rc;

#[allow(unused_imports)] 
use crate::fid::AsFid;
use crate::{domain::{Domain, DomainImpl}, eqoptions::EqConfig, fid::{OwnedFid, AsRawFid, self}, FI_ADDR_NOTAVAIL, MappedAddress, ep::Address};


// impl Drop for AddressVector {
//     fn drop(&mut self) {
//        println!("Dropping AddressVector\n");
//     }
// }
//================== AddressVector ==================//

/// Owned wrapper around a libfabric `fid_av`.
/// 
/// This type wraps an instance of a `fid_av`, monitoring its lifetime and closing it when it goes out of scope.
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.19.0/man/fi_av.3.html).
pub struct AddressVectorImpl {
    pub(crate) c_av: *mut libfabric_sys::fid_av, 
    fid: OwnedFid,
    _domain_rc: Rc<DomainImpl>,
}

pub struct AddressVector {
    inner: Rc<AddressVectorImpl>,
}

impl AddressVector {

    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_av {
        self.inner.c_av
    }

    pub(crate) fn new<T>(domain: &crate::domain::Domain, mut attr: AddressVectorAttr, context: Option<&mut T>) -> Result<Self, crate::error::Error> {
        let mut c_av:   *mut libfabric_sys::fid_av =  std::ptr::null_mut();
        let c_av_ptr: *mut *mut libfabric_sys::fid_av = &mut c_av;

        let err = 
        if let Some(ctx) = context {
            unsafe { libfabric_sys::inlined_fi_av_open(domain.handle(), attr.get_mut(), c_av_ptr, ctx as *mut T as *mut std::ffi::c_void) }
        }
        else {
            unsafe { libfabric_sys::inlined_fi_av_open(domain.handle(), attr.get_mut(), c_av_ptr, std::ptr::null_mut()) }
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(
                Self {
                    inner: Rc::new(
                        AddressVectorImpl {
                            c_av,
                            fid: OwnedFid::from(unsafe {&mut (*c_av).fid} ),
                            _domain_rc: domain.inner.clone(),
                    })
                })
        }
    }


    /// Associates an [EventQueue](crate::eq::EventQueue) with the AddressVector.
    /// 
    /// This method directly corresponds to a call to `fi_av_bind(av, eq, 0)`.
    /// # Errors
    ///
    /// This function will return an error if the underlying library call fails.
    pub fn bind<T: EqConfig>(&self, eq: &crate::eq::EventQueue<T>) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_av_bind(self.handle(), eq.as_raw_fid(), 0) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(())
        }
    }

    /// Inserts one or more addresses into an AV. 
    /// 
    /// This method directly corresponds to a call to `fi_insert(av,..)`, where the 
    ///
    /// # Errors
    ///
    /// This function will return an error if the underlying library call fails.
    pub fn insert(&self, addr: &[Address], flags: u64) -> Result<Vec<Option<MappedAddress>>, crate::error::Error> { // [TODO] //[TODO] Handle flags, handle context, handle async

        let mut fi_addresses = vec![0u64; addr.len()];
        let total_size = addr.iter().fold(0, |acc, addr| acc + addr.as_bytes().len() );
        let mut serialized: Vec<u8> = Vec::with_capacity(total_size);
        for a in addr {
            serialized.extend(a.as_bytes().iter())
        }

        let err = unsafe { libfabric_sys::inlined_fi_av_insert(self.handle(), serialized.as_ptr().cast(), fi_addresses.len(), fi_addresses.as_mut_ptr().cast(), flags, std::ptr::null_mut()) };

        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            let mapped_addresses = fi_addresses.into_iter().map(|fi_addr| if fi_addr == FI_ADDR_NOTAVAIL {None} else {Some(MappedAddress::from_raw_addr(fi_addr))}).collect::<Vec<_>>();
            Ok(mapped_addresses)
        }
    }

    pub fn insertsvc(&self, node: &str, service: &str, flags: u64) -> Result<Option<MappedAddress>, crate::error::Error> { // [TODO] Handle case where operation partially failed
        let mut fi_addr = 0u64;
        let err = unsafe { libfabric_sys::inlined_fi_av_insertsvc(self.handle(), node.as_bytes().as_ptr() as *const i8, service.as_bytes().as_ptr() as *const i8, &mut fi_addr, flags, std::ptr::null_mut())  };


        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else if fi_addr == FI_ADDR_NOTAVAIL {
            Ok(None)
        }
        else {
            Ok(Some(MappedAddress::from_raw_addr(fi_addr)))
        }
    }

    pub fn insertsym(&self, node: &str, nodecnt :usize, service: &str, svccnt: usize, flags: u64) -> Result<Vec<Option<MappedAddress>>, crate::error::Error> { // [TODO] Handle case where operation partially failed
        let total_cnt = nodecnt * svccnt;
        let mut fi_addresses = vec![0u64; total_cnt];
        let err = unsafe { libfabric_sys::inlined_fi_av_insertsym(self.handle(), node.as_bytes().as_ptr() as *const i8, nodecnt, service.as_bytes().as_ptr() as *const i8, svccnt, fi_addresses.as_mut_ptr().cast(), flags, std::ptr::null_mut())  };

        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            let mapped_addresses = fi_addresses.into_iter().map(|fi_addr| if fi_addr == FI_ADDR_NOTAVAIL {None} else {Some(MappedAddress::from_raw_addr(fi_addr))}).collect::<Vec<_>>();
            Ok(mapped_addresses)
        }
    }

    pub fn remove(&self, addr: Vec<crate::MappedAddress>) -> Result<(), crate::error::Error> {
        let mut fi_addresses =  addr.into_iter().map(|mapped_addr| {mapped_addr.raw_addr()}).collect::<Vec<u64>>();
        
        let err = unsafe { libfabric_sys::inlined_fi_av_remove(self.handle(), fi_addresses.as_mut_ptr().cast(), fi_addresses.len(), 0) };

        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn lookup(&self, mapped_addr: crate::MappedAddress) -> Result<Address, crate::error::Error> {
        let mut addrlen : usize = 0;
        let err = unsafe { libfabric_sys::inlined_fi_av_lookup(self.handle(), mapped_addr.raw_addr(), std::ptr::null_mut(), &mut addrlen) };
        
        if -err as u32  == libfabric_sys::FI_ETOOSMALL {
            let mut addr = vec![0u8; addrlen];
            let err = unsafe { libfabric_sys::inlined_fi_av_lookup(self.handle(), mapped_addr.raw_addr(), addr.as_mut_ptr().cast(), &mut addrlen) };

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

    pub fn straddr<T0>(&self, addr: &Address) -> String {
        let mut addr_str: Vec<u8> = Vec::new();
        let mut strlen = addr_str.len();
        let strlen_ptr: *mut usize = &mut strlen;
        unsafe { libfabric_sys::inlined_fi_av_straddr(self.handle(), addr.as_bytes().as_ptr().cast(), addr_str.as_mut_ptr() as *mut std::ffi::c_char, strlen_ptr) };
        addr_str.resize(strlen, 1);
        
        let mut strlen = addr_str.len();
        let strlen_ptr: *mut usize = &mut strlen;
        unsafe { libfabric_sys::inlined_fi_av_straddr(self.handle(), addr.as_bytes().as_ptr().cast(), addr_str.as_mut_ptr() as *mut std::ffi::c_char, strlen_ptr) };
        std::ffi::CString::from_vec_with_nul(addr_str).unwrap().into_string().unwrap()
    }
}

pub struct AddressVectorBuilder<'a, T> {
    av_attr: AddressVectorAttr,
    ctx: Option<&'a mut T>,
    domain: &'a Domain,
}

impl<'a> AddressVectorBuilder<'a, ()> {
    pub fn new(domain: &'a Domain) -> AddressVectorBuilder<'a, ()> {
        AddressVectorBuilder {
            av_attr: AddressVectorAttr::new(),
            ctx: None,
            domain,
        }
    }
}

impl<'a, T> AddressVectorBuilder<'a, T> {

    pub fn type_(mut self, av_type: crate::enums::AddressVectorType) -> Self {
        self.av_attr.type_(av_type);
        self
    }

    pub fn rx_ctx_bits(mut self, rx_ctx_bits: i32) -> Self {
        self.av_attr.rx_ctx_bits(rx_ctx_bits);
        self
    }

    pub fn count(mut self, count: usize) -> Self {
        self.av_attr.count(count);
        self
    }
    
    pub fn ep_per_node(mut self, count: usize) -> Self {
        self.av_attr.ep_per_node(count);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.av_attr.name(name);
        self 
    }

    pub fn map_addr(mut self, addr: usize) -> Self {
        self.av_attr.map_addr(addr);
        self
    }

    pub fn flags(mut self, flags: u64) -> Self {
        self.av_attr.flags(flags);
        self
    }

    pub fn context(self, ctx: &'a mut T) -> AddressVectorBuilder<'a, T> {
        AddressVectorBuilder {
            av_attr: self.av_attr,
            domain: self.domain,
            ctx: Some(ctx),
        }
    }

    pub fn build(self) -> Result<AddressVector, crate::error::Error> {
        AddressVector::new(self.domain, self.av_attr, self.ctx)
    }
    
}

//================== AddressVectorSet ==================//

pub struct AddressVectorSetImpl {
    pub(crate) c_set : *mut libfabric_sys::fid_av_set,
    fid: OwnedFid,
    _av_rc: Rc<AddressVectorImpl>,
}

pub struct AddressVectorSet {
    inner: Rc<AddressVectorSetImpl>,
}

impl AddressVectorSet {

    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_av_set {
        self.inner.c_set
    }

    // pub(crate) fn new(av: &AddressVector,attr: AddressVectorSetAttr) -> Result<AddressVectorSet, crate::error::Error> {
    //     Self::new_::<()>(av, attr, None)
    // }

    // pub(crate) fn new_with_context<T>(av: &AddressVector, attr: AddressVectorSetAttr, context: &mut T) -> Result<AddressVectorSet, crate::error::Error> {
    //     Self::new_(av, attr, Some(context))
    // }

    fn new<T>(av: &AddressVector, mut attr: AddressVectorSetAttr, context: Option<&mut T>) -> Result<AddressVectorSet, crate::error::Error> {
        let mut c_set: *mut libfabric_sys::fid_av_set = std::ptr::null_mut();
        let c_set_ptr: *mut *mut libfabric_sys::fid_av_set = &mut c_set;

        let err = 
        if let Some(ctx) = context {
            unsafe { libfabric_sys::inlined_fi_av_set(av.handle(), attr.get_mut(), c_set_ptr, (ctx as *mut T).cast()) }
        }
        else {
            unsafe { libfabric_sys::inlined_fi_av_set(av.handle(), attr.get_mut(), c_set_ptr, std::ptr::null_mut()) }
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    inner: Rc::new(
                        AddressVectorSetImpl { 
                            c_set, 
                            fid: OwnedFid::from(unsafe {&mut (*c_set).fid} ),
                            _av_rc: av.inner.clone(),
                    })
                })
        }
    }

    pub fn union(&mut self, other: &AddressVectorSet) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_union(self.handle(), other.handle()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn intersect(&mut self, other: &AddressVectorSet) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_intersect(self.handle(), other.handle()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub fn diff(&mut self, other: &AddressVectorSet) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_diff(self.handle(), other.handle()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub fn insert(&mut self, mapped_addr: crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_insert(self.handle(), mapped_addr.raw_addr()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn remove(&mut self, mapped_addr: crate::MappedAddress) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_remove(self.handle(), mapped_addr.raw_addr()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn get_addr(&mut self) -> Result<crate::MappedAddress, crate::error::Error> {
        let mut addr = 0u64;
        // let addr_ptr: *mut crate::MappedAddress = &mut addr;
        let err = unsafe { libfabric_sys::inlined_fi_av_set_addr(self.handle(), &mut addr) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(MappedAddress::from_raw_addr(addr))
        }
    }
}

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

    pub fn count(mut self, size: usize) -> Self {

        self.avset_attr.count(size);
        self
    }

    pub fn start_addr(mut self, mapped_addr: crate::MappedAddress) -> Self {
        
        self.avset_attr.start_addr(mapped_addr);
        self
    }

    pub fn end_addr(mut self, mapped_addr: crate::MappedAddress) -> Self {
        
        self.avset_attr.end_addr(mapped_addr);
        self
    }

    pub fn stride(mut self, stride: usize) -> Self {

        self.avset_attr.stride(stride);
        self
    }

    pub fn comm_key(mut self, key: &mut [u8]) -> Self {
        
        self.avset_attr.comm_key(key);
        self
    }

    pub fn flags(mut self, flags: u64) -> Self {

        self.avset_attr.flags(flags);
        self
    }

    pub fn context(self, ctx: &'a mut T) -> AddressVectorSetBuilder<'a, T> {
        AddressVectorSetBuilder {
            avset_attr: self.avset_attr,
            av: self.av,
            ctx: Some(ctx),
        }
    }

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

    pub fn flags(&mut self, flags: u64) -> &mut Self {
        self.c_attr.flags = flags;
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

    pub(crate) fn start_addr(&mut self, mapped_addr: crate::MappedAddress) -> &mut Self {
        
        self.c_attr.start_addr = mapped_addr.raw_addr();
        self
    }

    pub(crate) fn end_addr(&mut self, mapped_addr: crate::MappedAddress) -> &mut Self {
        
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

    pub(crate) fn flags(&mut self, flags: u64) -> &mut Self {

        self.c_attr.flags = flags;
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


impl AsFid for AddressVectorSet {
    fn as_fid(&self) -> fid::BorrowedFid {
        self.inner.fid.as_fid()
    }
}

impl AsFid for AddressVector {
    fn as_fid(&self) -> fid::BorrowedFid {
        self.inner.fid.as_fid()
    }
}

impl crate::BindImpl for AddressVectorImpl {}

impl crate::Bind for AddressVector {
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
                    .flags(0)
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
                    .flags(0)
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