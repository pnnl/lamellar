#[allow(unused_imports)] 
use crate::FID;
//================== Address Vector (fi_av) ==================//
pub struct AddressVector {
    pub(crate) c_av: *mut libfabric_sys::fid_av, 
}

impl AddressVector {
    pub fn new(domain: &crate::domain::Domain, mut attr: AddressVectorAttr) -> Self {
        let mut c_av:   *mut libfabric_sys::fid_av =  std::ptr::null_mut();
        let c_av_ptr: *mut *mut libfabric_sys::fid_av = &mut c_av;

        let err = unsafe { libfabric_sys::inlined_fi_av_open(domain.c_domain, attr.get_mut(), c_av_ptr, std::ptr::null_mut()) };

        if err != 0 {
            panic!("fi_av_open failed {}", err);
        }

        Self {
            c_av,
        }
    }

    pub fn bind(&self, fid: &impl crate::FID, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_av_bind(self.c_av, fid.fid(), flags) };

        if err != 0 {
            panic!("fi_av_bind failed {}", err);
        }
    }

    pub fn insert<T0>(&self, buf: &[T0], addr: &mut crate::Address, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_av_insert(self.c_av, buf.as_ptr() as *const std::ffi::c_void, buf.len(), addr as *mut crate::Address, flags, std::ptr::null_mut())  };

        if err != 0 {
            panic!("fi_av_insert failed {}", err);
        }
    }

    pub fn insertsvc(&self, node: &str, service: &str, addr: &mut crate::Address, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_av_insertsvc(self.c_av, node.as_bytes().as_ptr() as *const i8, service.as_bytes().as_ptr() as *const i8, addr as *mut crate::Address, flags, std::ptr::null_mut())  };

        if err != 0 {
            panic!("fi_av_insertvc failed {}", err);
        }
    }

    pub fn insertsym(&self, node: &str, nodecnt :usize, service: &str, svccnt: usize, addr: &mut crate::Address, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_av_insertsym(self.c_av, node.as_bytes().as_ptr() as *const i8, nodecnt, service.as_bytes().as_ptr() as *const i8, svccnt, addr as *mut crate::Address, flags, std::ptr::null_mut())  };

        if err != 0 {
            panic!("fi_av_insertsym failed {}", err);
        }
    }

    pub fn remove(&self, addr: &mut crate::Address, count: usize, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_av_remove(self.c_av, addr as *mut crate::Address, count, flags) };

        if err != 0 {
            panic!("fi_av_remove failed {}", err);
        }
    }

    pub fn lookup<T0>(&self, addr: crate::Address, address: &mut [T0] ) -> usize {
        let mut addrlen : usize = 0;
        let addrlen_ptr: *mut usize = &mut addrlen;
        let err = unsafe { libfabric_sys::inlined_fi_av_lookup(self.c_av, addr, address.as_mut_ptr() as *mut std::ffi::c_void, addrlen_ptr) };
        
        if err != 0 {
            panic!("fi_av_lookup failed {}", err);
        }

        addrlen 
    }

    //[TODO]
    pub fn straddr<T0,T1>(&self, addr: &[T0], buf: &mut [T1]) -> &str {
        let mut strlen = buf.len();
        let strlen_ptr: *mut usize = &mut strlen;
        let straddr: *const i8 = unsafe { libfabric_sys::inlined_fi_av_straddr(self.c_av, addr.as_ptr() as *const std::ffi::c_void, buf.as_mut_ptr() as *mut std::ffi::c_char, strlen_ptr) };
        let str_addr = unsafe {std::ffi::CStr::from_ptr(straddr)};
        str_addr.to_str().unwrap()
    }

    pub fn avset<T0>(&self, attr:AddressVectorSetAttr , context: &mut T0) -> AddressVectorSet {
        AddressVectorSet::new(self, attr, context)
    }

}

impl crate::FID for AddressVector {
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_av).fid }
    }
}

//================== Address Vector attribute ==================//

pub struct AddressVectorAttr {
    pub(crate) c_attr: libfabric_sys::fi_av_attr, 
}

impl AddressVectorAttr {
    pub fn new() -> Self {
        let c_attr = libfabric_sys::fi_av_attr{
            type_: crate::enums::AddressVectorType::UNSPEC.get_value(), 
            rx_ctx_bits: 0,
            count: 0,
            ep_per_node: 0,
            name: std::ptr::null(),
            map_addr: std::ptr::null_mut(),
            flags: 0
        };

        Self { c_attr }
    }

    pub fn avtype(&mut self, av_type: crate::enums::AddressVectorType) -> &mut Self{
        self.c_attr.type_ = av_type.get_value();

        self
    }

    pub fn count(&mut self, count: usize) -> &mut Self {
        self.c_attr.count = count;

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

//================== Address Set (fi_av_set) ==================//

pub struct AddressVectorSet {
    pub(crate) c_set : *mut libfabric_sys::fid_av_set,
}

impl AddressVectorSet {
    pub(crate) fn new<T0>(av: &AddressVector, mut attr: AddressVectorSetAttr, context: &mut T0) -> Self {
        let mut c_set: *mut libfabric_sys::fid_av_set = std::ptr::null_mut();
        let c_set_ptr: *mut *mut libfabric_sys::fid_av_set = &mut c_set;

        let err = unsafe { libfabric_sys::inlined_fi_av_set(av.c_av, attr.get_mut(), c_set_ptr, context as *mut T0 as *mut std::ffi::c_void) };
        if err != 0 {
            panic!("fi_av_set failed {}", err);
        }

        Self { c_set }
    }
    
    pub fn union(&mut self, other: &AddressVectorSet) {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_union(self.c_set, other.c_set) };

        if err != 0 {
            panic!("fi_av_set_union failed {}", err);
        }
    }
    pub fn intersect(&mut self, other: &AddressVectorSet) {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_intersect(self.c_set, other.c_set) };

        if err != 0 {
            panic!("fi_av_set_intersect failed {}", err);
        }
    }
    pub fn diff(&mut self, other: &AddressVectorSet) {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_diff(self.c_set, other.c_set) };

        if err != 0 {
            panic!("fi_av_set_diff failed {}", err);
        }
    }
    
    pub fn insert(&mut self, addr: crate::Address) {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_insert(self.c_set, addr) };

        if err != 0 {
            panic!("fi_av_set_insert failed {}", err);
        }
    }

    pub fn remove(&mut self, addr: crate::Address) {
        let err = unsafe { libfabric_sys::inlined_fi_av_set_remove(self.c_set, addr) };

        if err != 0 {
            panic!("fi_av_set_remove failed {}", err);
        }
    }

    pub fn addr(&mut self) -> crate::Address {
        let mut addr: crate::Address = 0;
        let addr_ptr: *mut crate::Address = &mut addr;
        let err = unsafe { libfabric_sys::inlined_fi_av_set_addr(self.c_set, addr_ptr) };

        if err != 0 {
            panic!("fi_av_set_addr failed {}", err);
        }

        addr
    }
}

//================== Address Vector Set attribute ==================//


pub struct AddressVectorSetAttr {
    c_attr: libfabric_sys::fi_av_set_attr,
}

impl AddressVectorSetAttr {

    #[allow(dead_code)]
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_av_set_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_av_set_attr {
        &mut self.c_attr
    }    
}


//================== Address Vector tests ==================//

#[test]
fn av_open_close() {
    let mut ep_attr = crate::ep::EndpointAttr::new();
        ep_attr
        .ep_type(crate::enums::EndpointType::RDM);
    let mut dom_attr = crate::domain::DomainAttr::new();
        dom_attr
        .mode(!(crate::enums::MrMode::BASIC.get_value() | crate::enums::MrMode::SCALABLE.get_value()) as u64 );
    let mut hints = crate::InfoHints::new();
        hints
        .ep_attr(ep_attr)
        .domain_attr(dom_attr);
    let info = crate::Info::with_hints(hints);
    let entries: Vec<crate::InfoEntry> = info.get();
    if entries.len() > 0 {

        let mut eq_attr = crate::eq::EventQueueAttr::new();
            eq_attr
            .size(32)
            .flags(libfabric_sys::FI_WRITE.into())
            .wait_obj(crate::enums::WaitObj::FD);
        let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
        let mut eq = fab.eq_open(eq_attr);
        let mut domain = fab.domain(&entries[0]);
        
        for i in 0..17 {
            let count = 1 << i;
            let mut attr = crate::av::AddressVectorAttr::new();
                attr
                .avtype(crate::enums::AddressVectorType::MAP)
                .count(count)
                .flags(0);
            let mut av = domain.av_open(attr);
            av.close();
        }
        domain.close();
        eq.close();
        fab.close();
    }

}