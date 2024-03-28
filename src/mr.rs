use std::rc::Rc;

use crate::{enums::MrMode, OwnedFid, domain::DomainImpl, check_error};
#[allow(unused_imports)]
use crate::AsFid;

// impl Drop for MemoryRegion {
//     fn drop(&mut self) {
//        println!("Dropping MemoryRegion\n");
//     }
// }

//================== Memory Region (fi_mr) ==================//
pub(crate) struct MemoryRegionImpl {
    pub(crate) c_mr: *mut libfabric_sys::fid_mr,
    fid: OwnedFid,
    _domain_rc: Rc<DomainImpl>,
}

pub struct MemoryRegion {
    inner: Rc<MemoryRegionImpl>
}

impl MemoryRegion {
    
    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_mr {
        self.inner.c_mr
    }

    #[allow(dead_code)]
    pub(crate) fn from_buffer<T0>(domain: &crate::domain::Domain, buf: &[T0], acs: u64, offset: u64, requested_key: u64, flags: MrMode) -> Result<MemoryRegion, crate::error::Error> {
        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let c_mr_ptr: *mut *mut libfabric_sys::fid_mr = &mut c_mr;
        let err = unsafe { libfabric_sys::inlined_fi_mr_reg(domain.handle(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), acs, offset, requested_key, flags.get_value() as u64, c_mr_ptr, std::ptr::null_mut()) };
        
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    inner: Rc::new(
                        MemoryRegionImpl { 
                            c_mr, 
                            fid: crate::OwnedFid{fid: unsafe {&mut (*c_mr).fid } },
                            _domain_rc: domain.inner.clone(),
                        })        

                })
        }
    }

    pub(crate) fn from_attr(domain: &crate::domain::Domain, attr: MemoryRegionAttr, flags: MrMode) -> Result<MemoryRegion, crate::error::Error> {
        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let c_mr_ptr: *mut *mut libfabric_sys::fid_mr = &mut c_mr;
        let err = unsafe { libfabric_sys::inlined_fi_mr_regattr(domain.handle(), attr.get(), flags.get_value() as u64, c_mr_ptr) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    inner: Rc::new(
                        MemoryRegionImpl { 
                            c_mr, 
                            fid: crate::OwnedFid{fid: unsafe {&mut (*c_mr).fid } },
                            _domain_rc: domain.inner.clone(),
                        })        

                })
        }
    
    }

    #[allow(dead_code)]
    pub(crate) fn from_iovec<T>(domain: &crate::domain::Domain,  iov : &crate::IoVec<T>, count: usize, acs: u64, offset: u64, requested_key: u64, flags: MrMode) -> Result<MemoryRegion, crate::error::Error> {
        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let c_mr_ptr: *mut *mut libfabric_sys::fid_mr = &mut c_mr;
        let err = unsafe { libfabric_sys::inlined_fi_mr_regv(domain.handle(), iov.get(), count, acs, offset, requested_key, flags.get_value() as u64, c_mr_ptr, std::ptr::null_mut()) };
    
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    inner: Rc::new(
                        MemoryRegionImpl { 
                            c_mr, 
                            fid: crate::OwnedFid{fid: unsafe {&mut (*c_mr).fid } },
                            _domain_rc: domain.inner.clone(),
                        })        

                })
        }
    
    }

    pub fn get_key(&mut self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_mr_key(self.handle()) }
    }

    pub fn bind_cntr<T: crate::cntroptions::CntrConfig>(&self, cntr: &crate::cntr::Counter<T>, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_mr_bind(self.handle(), cntr.as_fid(), flags) } ;
        
        check_error(err.try_into().unwrap())
    }

    pub fn bind_ep(&self, ep: &crate::ep::Endpoint) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_mr_bind(self.handle(), ep.as_fid(), 0) } ;
        
        check_error(err.try_into().unwrap())
    }

    pub fn refresh<T>(&self, iov: &[crate::IoVec<T>], flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_mr_refresh(self.handle(), iov.as_ptr().cast(), iov.len(), flags) };

        check_error(err.try_into().unwrap())
    }

    pub fn enable(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_mr_enable(self.handle()) };

        check_error(err.try_into().unwrap())
    }

    pub fn raw_attr(&self, base_addr: &mut u64, key_size: &mut usize, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_mr_raw_attr(self.handle(), base_addr as *mut u64, std::ptr::null_mut(), key_size as *mut usize, flags) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        } 
        else {
            Ok(())
        }       
    }

    pub fn raw_attr_with_key(&self, base_addr: &mut u64, raw_key: &mut u8, key_size: &mut usize, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_mr_raw_attr(self.handle(), base_addr as *mut u64, raw_key as *mut u8, key_size as *mut usize, flags) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }        
        else {
            Ok(())
        }
    }

    pub fn description(&self) -> MemoryRegionDesc {
        let c_desc = unsafe { libfabric_sys::inlined_fi_mr_desc(self.handle())};
        if c_desc.is_null() {
            panic!("fi_mr_desc returned NULL");
        }

        MemoryRegionDesc { c_desc }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct MemoryRegionDesc {
    c_desc: *mut std::ffi::c_void,
}

impl crate::DataDescriptor for MemoryRegionDesc {
    
    fn get_desc(&mut self) -> *mut std::ffi::c_void {
        self.c_desc
    }

    fn get_desc_ptr(&mut self) -> *mut *mut std::ffi::c_void {
        &mut self.c_desc
    }
}

impl crate::AsFid for MemoryRegion{
    fn as_fid(&self) -> *mut libfabric_sys::fid {
        self.inner.fid.as_fid()
    }
}

//================== Memory Region attribute ==================//

pub struct MemoryRegionAttr {
    pub(crate) c_attr: libfabric_sys::fi_mr_attr,
}

impl MemoryRegionAttr {

    pub fn new() -> Self {
        Self {
            c_attr: libfabric_sys::fi_mr_attr {
                mr_iov: std::ptr::null(),
                iov_count: 0,
                access: 0,
                offset: 0,
                requested_key: 0,
                context: std::ptr::null_mut(),
                auth_key_size: 0,
                auth_key: std::ptr::null_mut(),
                iface: libfabric_sys::fi_hmem_iface_FI_HMEM_SYSTEM,
                device: libfabric_sys::fi_mr_attr__bindgen_ty_1 {reserved: 0},
                hmem_data: std::ptr::null_mut(),
            }
        }
    }

    pub fn iov<T>(&mut self, iov: &[crate::IoVec<T>] ) -> &mut Self {
        self.c_attr.mr_iov = iov.as_ptr() as *const libfabric_sys::iovec;
        self.c_attr.iov_count = iov.len();
        
        self
    }

    pub fn access(&mut self, access: u64) -> &mut Self {
        self.c_attr.access = access;
        self
    }

    pub fn access_send(&mut self) -> &mut Self { 
        self.c_attr.access |= libfabric_sys::FI_SEND as u64;
        self
    }

    pub fn access_recv(&mut self) -> &mut Self { 
        self.c_attr.access |= libfabric_sys::FI_RECV as u64;
        self
    }

    pub fn access_read(&mut self) -> &mut Self { 
        self.c_attr.access |= libfabric_sys::FI_READ as u64;
        self
    }

    pub fn access_write(&mut self) -> &mut Self { 
        self.c_attr.access |= libfabric_sys::FI_WRITE as u64;
        self
    }

    pub fn access_remote_write(&mut self) -> &mut Self { 
        self.c_attr.access |= libfabric_sys::FI_REMOTE_WRITE as u64;
        self
    }

    pub fn access_remote_read(&mut self) -> &mut Self { 
        self.c_attr.access |= libfabric_sys::FI_REMOTE_READ as u64;
        self
    }

    pub fn offset(&mut self, offset: u64) -> &mut Self {
        self.c_attr.offset = offset;
        self
    }

    pub fn context<T0>(&mut self, ctx: &mut T0) -> &mut Self {
        self.c_attr.context = ctx as * mut T0 as *mut std::ffi::c_void;
        self
    }
    
    pub fn requested_key(&mut self, key: u64) -> &mut Self {
        self.c_attr.requested_key = key;
        self
    }

    pub fn auth_key(&mut self, key: &mut [u8]) -> &mut Self {
        self.c_attr.auth_key_size = key.len();
        self.c_attr.auth_key = key.as_mut_ptr();
        self
    }

    pub fn iface(&mut self, iface: crate::enums::HmemIface) -> &mut Self {
        self.c_attr.iface = iface.get_value();
        self
    }

    // pub fn device(&mut self, device: libfabric_sys::devi) // [TODO]

    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_mr_attr {
        &self.c_attr
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_mr_attr {
        &mut self.c_attr
    }
}

impl Default for MemoryRegionAttr {
    fn default() -> Self {
        Self::new()
    }
}

pub struct MemoryRegionBuilder<'a> {
    mr_attr: MemoryRegionAttr,
    domain: &'a crate::domain::Domain,
    flags: MrMode,
}

impl<'a> MemoryRegionBuilder<'a> {

    pub fn new(domain: &'a crate::domain::Domain) -> Self {
        Self {
            mr_attr: MemoryRegionAttr::new(),
            domain,
            flags: MrMode::new(),
        }
    }

    pub fn iov<T>(mut self, iov: &[crate::IoVec<T>] ) -> Self {
        self.mr_attr.iov(iov);
        self
    }

    
    pub fn access_send(mut self) -> Self { 
        self.mr_attr.access_send();
        self
    }

    pub fn access_recv(mut self) -> Self { 
        self.mr_attr.access_recv();
        self
    }

    pub fn access_read(mut self) -> Self { 
        self.mr_attr.access_read();
        self
    }

    pub fn access_write(mut self) -> Self { 
        self.mr_attr.access_write();
        self
    }

    pub fn access_remote_write(mut self) -> Self { 
        self.mr_attr.access_remote_write();
        self
    }

    pub fn access_remote_read(mut self) -> Self { 
        self.mr_attr.access_remote_read();
        self
    }

    pub fn access(mut self, access: u64) -> Self {
        self.mr_attr.access(access);
        self
    }

    pub fn offset(mut self, offset: u64) -> Self {
        self.mr_attr.offset(offset);
        self
    }

    pub fn context<T0>(mut self, ctx: &mut T0) -> Self {
        self.mr_attr.context(ctx);
        self
    }
    
    pub fn requested_key(mut self, key: u64) -> Self {
        self.mr_attr.requested_key(key);
        self
    }

    pub fn auth_key(mut self, key: &mut [u8]) -> Self {
        self.mr_attr.auth_key(key);
        self
    }

    pub fn iface(mut self, iface: crate::enums::HmemIface) -> Self {
        self.mr_attr.iface(iface);
        self
    }

    pub fn flags(mut self, flags: MrMode) -> Self {
        self.flags = flags;
        self
    }

    pub fn build(self) -> Result<MemoryRegion, crate::error::Error> {
        MemoryRegion::from_attr(self.domain, self.mr_attr, self.flags)
    }
}

//================== Memory Region tests ==================//
#[cfg(test)]
mod tests {
    use crate::IoVec;

    use super::MemoryRegionBuilder;


    pub fn ft_alloc_bit_combo(fixed: u64, opt: u64) -> Vec<u64> {
        let bits_set = |mut val: u64 | -> u64 { let mut cnt = 0; while val > 0 {  cnt += 1 ; val &= val-1; } cnt };
        let num_flags = bits_set(opt) + 1;
        let len = 1 << (num_flags - 1) ;
        let mut flags = vec![0_u64 ; num_flags as usize];
        let mut num_flags = 0;
        for i in 0..8*std::mem::size_of::<u64>(){
            if opt >> i & 1 == 1 {
                flags[num_flags] = 1 << i; 
                num_flags += 1;
            }
        }
        let mut combos = Vec::new();

        for index in 0..len {
            combos.push(fixed);
            for (i, val) in flags.iter().enumerate().take(8*std::mem::size_of::<u64>()){
                if index >> i & 1 == 1 {
                    combos[index] |= val;
                }
            }
        }

        combos
    }
    pub struct TestSizeParam(pub u64, pub u64);
    pub const DEF_TEST_SIZES: [TestSizeParam; 6] = [TestSizeParam(1 << 0,0), TestSizeParam(1 << 1,0), TestSizeParam(1 << 2,0), TestSizeParam(1 << 3,0), TestSizeParam(1 << 4,0), TestSizeParam(1 << 5,0) ];

    #[test]
    fn mr_reg() {
        let ep_attr = crate::ep::EndpointAttr::new();
        let mut dom_attr = crate::domain::DomainAttr::new();
            dom_attr
            .mode(crate::enums::Mode::all())
            .mr_mode(crate::enums::MrMode::new().basic().scalable().local().inverse());
        
        let hints = crate::InfoHints::new()
            .caps(crate::InfoCaps::new().msg().rma())
            .ep_attr(ep_attr)
            .domain_attr(dom_attr);

        let info = crate::Info::new().hints(&hints).request().unwrap();
        let entries = info.get();
        
        if !entries.is_empty() {

            let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
            let domain = crate::domain::DomainBuilder::new(&fab, &entries[0]).build().unwrap();

            let mut mr_access: u64 = 0;
            if entries[0].get_mode().is_local_mr() || entries[0].get_domain_attr().get_mr_mode().is_local() {

                if entries[0].caps.is_msg() || entries[0].caps.is_tagged() {
                    let mut on = false;
                    if entries[0].caps.is_send() {
                        mr_access |= libfabric_sys::FI_SEND as u64;
                        on = true;
                    }
                    if entries[0].caps.is_recv() {
                        mr_access |= libfabric_sys::FI_RECV  as u64 ;
                        on = true;
                    }
                    if !on {
                        mr_access |= libfabric_sys::FI_SEND as u64 & libfabric_sys::FI_RECV as u64;
                    }
                }
            }
            else if entries[0].caps.is_rma() || entries[0].caps.is_atomic() {
                if entries[0].caps.is_remote_read() || !(entries[0].caps.is_read() || entries[0].caps.is_write() || entries[0].caps.is_remote_write()) {
                    mr_access |= libfabric_sys::FI_REMOTE_READ as u64 ;
                }
                else {
                    mr_access |= libfabric_sys::FI_REMOTE_WRITE as u64 ;
                }
            }

            let combos = ft_alloc_bit_combo(0, mr_access);
            
            for test in &DEF_TEST_SIZES {
                let buff_size = test.0;
                let mut buf = vec![0_u64; buff_size as usize ];
                for combo in &combos {
                    let _mr = MemoryRegionBuilder::new(&domain)
                        .iov(std::slice::from_mut(&mut IoVec::from_slice_mut(&mut buf)))
                        .access(*combo)
                        .offset(0)
                        .requested_key(0xC0DE)
                        .build()
                        .unwrap();
                    // mr.close().unwrap();
                }
            }
            
            // domain.close().unwrap();
            // fab.close().unwrap();
        }
        else {
            panic!("No capable fabric found!");
        }
    }

    // fn mr_regv() {
    //     let ep_attr = crate::ep::EndpointAttr::new();
    //     let mut dom_attr = crate::domain::DomainAttr::new();
    //         dom_attr
    //         .mode(!0)
    //         .mr_mode(!(crate::enums::MrMode::BASIC.get_value() | crate::enums::MrMode::SCALABLE.get_value() | crate::enums::MrMode::LOCAL.get_value() ) as i32 );
    //     let mut hints = crate::InfoHints::new();
    //         hints
    //         .caps(crate::InfoCaps::new().msg().rma())
    //         .ep_attr(ep_attr)
    //         .domain_attr(dom_attr);
    //     let info = crate::Info::with_hints(hints);
    //     let entries: Vec<crate::InfoEntry> = info.get();
    //     if entries.len() > 0 {

    //         let mut eq_attr = crate::eq::EventQueueAttr::new();
    //             eq_attr
    //             .size(32)
    //             .flags(libfabric_sys::FI_WRITE.into())
    //             .wait_obj(crate::enums::WaitObj::FD);
    //         let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
    //         let mut eq = fab.eq_open(eq_attr);
    //         let mut domain = fab.domain(&entries[0]);

    //         let mut mr_access: u64 = 0;
    //         if entries[0].get_mode() & libfabric_sys::FI_LOCAL_MR == libfabric_sys::FI_LOCAL_MR || entries[0].get_domain_attr().get_mr_mode() as u32 & libfabric_sys::FI_MR_LOCAL == libfabric_sys::FI_MR_LOCAL {

    //             if entries[0].caps.is_msg() || entries[0].caps.is_tagged() {
    //                 let mut on = false;
    //                 if entries[0].caps.is_send() {
    //                     mr_access |= libfabric_sys::FI_SEND as u64;
    //                     on = true;
    //                 }
    //                 if entries[0].caps.is_recv() {
    //                     mr_access |= libfabric_sys::FI_RECV  as u64 ;
    //                     on = true;
    //                 }
    //                 if !on {
    //                     mr_access |= libfabric_sys::FI_SEND as u64 & libfabric_sys::FI_RECV as u64;
    //                 }
    //             }
    //         }
    //         else {
    //             if entries[0].caps.is_rma() || entries[0].caps.is_atomic() {
    //                 if entries[0].caps.is_remote_read() || !(entries[0].caps.is_read() || entries[0].caps.is_write() || entries[0].caps.is_remote_write()) {
    //                     mr_access |= libfabric_sys::FI_REMOTE_READ as u64 ;
    //                 }
    //                 else {
    //                     mr_access |= libfabric_sys::FI_REMOTE_WRITE as u64 ;
    //                 }
    //             }
    //         }

    //         let iovec = IoVec::new();
    //         if mr_access != 0 {
    //             let i = 0;
    //             let buf = vec![0; DEF_TEST_SIZES[DEF_TEST_SIZES.len()-1].0 as usize];
    //             while i < DEF_TEST_SIZES.len() && entries[0].get_domain_attr().get_mr_iov_limit() < DEF_TEST_SIZES[i].0 {
    //                 let n = DEF_TEST_SIZES[i].0;
    //                 let base = &buf[0..];
                    
    //             }
    //         }
    //         else {
    //             domain.close().unwrap();
    //             eq.close().unwrap();
    //             fab.close().unwrap();
    //             panic!("mr access == 0");            
    //         }

    //         domain.close().unwrap();
    //         eq.close().unwrap();
    //         fab.close().unwrap();
    //     }
    //     else {
    //         panic!("No capable fabric found!");
    //     }
    // }
}

#[cfg(test)]
mod libfabric_lifetime_tests {
    use crate::IoVec;

    use super::MemoryRegionBuilder;
    
    #[test]
    fn mr_drops_before_domain() {
        let ep_attr = crate::ep::EndpointAttr::new();
        let mut dom_attr = crate::domain::DomainAttr::new();
            dom_attr
            .mode(crate::enums::Mode::all())
            .mr_mode(crate::enums::MrMode::new().basic().scalable().local().inverse());
        
        let hints = crate::InfoHints::new()
            .caps(crate::InfoCaps::new().msg().rma())
            .ep_attr(ep_attr)
            .domain_attr(dom_attr);

        let info = crate::Info::new().hints(&hints).request().unwrap();
        let entries = info.get();
        
        if !entries.is_empty() {

            let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
            let domain = crate::domain::DomainBuilder::new(&fab, &entries[0]).build().unwrap();

            let mut mr_access: u64 = 0;
            if entries[0].get_mode().is_local_mr() || entries[0].get_domain_attr().get_mr_mode().is_local() {

                if entries[0].caps.is_msg() || entries[0].caps.is_tagged() {
                    let mut on = false;
                    if entries[0].caps.is_send() {
                        mr_access |= libfabric_sys::FI_SEND as u64;
                        on = true;
                    }
                    if entries[0].caps.is_recv() {
                        mr_access |= libfabric_sys::FI_RECV  as u64 ;
                        on = true;
                    }
                    if !on {
                        mr_access |= libfabric_sys::FI_SEND as u64 & libfabric_sys::FI_RECV as u64;
                    }
                }
            }
            else if entries[0].caps.is_rma() || entries[0].caps.is_atomic() {
                if entries[0].caps.is_remote_read() || !(entries[0].caps.is_read() || entries[0].caps.is_write() || entries[0].caps.is_remote_write()) {
                    mr_access |= libfabric_sys::FI_REMOTE_READ as u64 ;
                }
                else {
                    mr_access |= libfabric_sys::FI_REMOTE_WRITE as u64 ;
                }
            }

            let combos = super::tests::ft_alloc_bit_combo(0, mr_access);
            
            let mut mrs = Vec::new();
            for test in &super::tests::DEF_TEST_SIZES {
                let buff_size = test.0;
                let mut buf = vec![0_u64; buff_size as usize ];
                for combo in &combos {
                    let mr = MemoryRegionBuilder::new(&domain)
                        .iov(std::slice::from_mut(&mut IoVec::from_slice_mut(&mut buf)))
                        .access(*combo)
                        .offset(0)
                        .requested_key(0xC0DE)
                        .build()
                        .unwrap();
                    mrs.push(mr);
                    println!("Count = {} \n", std::rc::Rc::strong_count(&domain.inner));
                }
            }
            drop(domain);
            println!("Count = {} After dropping domain\n", std::rc::Rc::strong_count(&mrs[0].inner._domain_rc));
            
            // domain.close().unwrap();
            // fab.close().unwrap();
        }
        else {
            panic!("No capable fabric found!");
        }
    }
}