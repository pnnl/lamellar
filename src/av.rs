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

        if err < 0 {
            panic!("fi_av_insert failed {} : {}", err, crate::error_to_string(err.into()));
        }

        if (err as usize) != buf.len() {
            panic!("fi_av_insert failed. Inserted {} vs {}", err, buf.len());

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

// impl Drop for AddressVector {
//     fn drop(&mut self) {
//         println!("Dropping av");

//         self.close()
//     }
// }

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

    pub fn type_(self, av_type: crate::enums::AddressVectorType) -> Self{
        let mut c_attr = self.c_attr;
        c_attr.type_ = av_type.get_value();
        
        Self { c_attr }
    }

    pub fn count(self, count: usize) -> Self {
        let mut c_attr = self.c_attr;
        c_attr.count = count;

        Self { c_attr }
    }

    pub fn flags(self, flags: u64) -> Self {
        let mut c_attr = self.c_attr;
        c_attr.flags = flags;

        Self { c_attr }
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

#[cfg(test)]
mod tests {
    use crate::FID;
    
    #[test]
    fn av_open_close() {
        let ep_attr = crate::ep::EndpointAttr::new()
        .ep_type(crate::enums::EndpointType::RDM);
    
        let dom_attr = crate::domain::DomainAttr::new()
            .mode(!0)
            .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse());

        let hints = crate::InfoHints::new()
            .ep_attr(ep_attr)
            .domain_attr(dom_attr);

        let info = crate::Info::new().hints(&hints).request();
        let entries: Vec<crate::InfoEntry> = info.get();
        if entries.len() > 0 {
        
            let fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
            let domain = fab.domain(&entries[0]);
        
            for i in 0..17 {
                let count = 1 << i;
                let attr = crate::av::AddressVectorAttr::new()
                    .type_(crate::enums::AddressVectorType::MAP)
                    .count(count)
                    .flags(0);
                let av = domain.av_open(attr);
                    av.close();
            }

            domain.close();
            fab.close();
        }
        else {
            panic!("No capable fabric found!");
        }
    }

    #[test]
    fn av_good_sync() {
        
        let ep_attr = crate::ep::EndpointAttr::new().ep_type(crate::enums::EndpointType::RDM);

        let dom_attr = crate::domain::DomainAttr::new()
            .mode(!0)
            .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse());

        let hints = crate::InfoHints::new()
            .ep_attr(ep_attr)
            .domain_attr(dom_attr);

        let info = crate::Info::new()
            .hints(&hints).request();

        let entries: Vec<crate::InfoEntry> = info.get();
        if entries.len() > 0 {
            let attr = crate::av::AddressVectorAttr::new()
                .type_(crate::enums::AddressVectorType::MAP)
                .count(32);
            let fab: crate::fabric::Fabric = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
            let domain = fab.domain(&entries[0]);
            let av = domain.av_open(attr);

            av.close();
            domain.close();
            fab.close();
        }
        else {
            panic!("No capable fabric found!");
        }
    }
}