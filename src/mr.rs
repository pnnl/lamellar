
//================== Memory Region (fi_mr) ==================//
pub struct MemoryRegion {
    pub(crate) c_mr: *mut libfabric_sys::fid_mr,
}

impl MemoryRegion {
    pub(crate) fn from_buffer<T0>(domain: &crate::domain::Domain, buf: &[T0], acs: u64, offset: u64, requested_key: u64, flags: u64) -> Self {
        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let c_mr_ptr: *mut *mut libfabric_sys::fid_mr = &mut c_mr;
        let err = unsafe { libfabric_sys::inlined_fi_mr_reg(domain.c_domain, buf.as_ptr() as *const std::ffi::c_void, buf.len() * std::mem::size_of::<T0>(), acs, offset, requested_key, flags, c_mr_ptr, std::ptr::null_mut()) };
        if err != 0 {
            panic!("fi_mr_reg failed {}", err);
        }
    
        Self { c_mr }        
    }

    pub(crate) fn from_attr(domain: &crate::domain::Domain, attr: MemoryRegionAttr, flags: u64) -> Self {
        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let c_mr_ptr: *mut *mut libfabric_sys::fid_mr = &mut c_mr;
        let err = unsafe { libfabric_sys::inlined_fi_mr_regattr(domain.c_domain, attr.get(), flags, c_mr_ptr) };
    
        if err != 0 {
            panic!("fi_mr_regattr failed {}", err);
        }
    
        Self { c_mr }           
    }
    
    pub(crate) fn from_iovec(domain: &crate::domain::Domain,  iov : &crate::IoVec, count: usize, acs: u64, offset: u64, requested_key: u64, flags: u64) -> Self {
        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let c_mr_ptr: *mut *mut libfabric_sys::fid_mr = &mut c_mr;
        let err = unsafe { libfabric_sys::inlined_fi_mr_regv(domain.c_domain, iov.get(), count, acs, offset, requested_key, flags, c_mr_ptr, std::ptr::null_mut()) };
    
        if err != 0 {
            panic!("fi_mr_regv failed {}", err);
        }
    
        Self { c_mr }    
    }


    pub fn desc<T0>(&mut self) -> &mut T0 {
        let ret: *mut T0 = (unsafe { libfabric_sys::inlined_fi_mr_desc(self.c_mr) }) as *mut T0;
        unsafe { &mut *ret }
    }


    pub fn key(&mut self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_mr_key(self.c_mr) }
    }

    pub fn bind(&self, fid: &impl crate::FID, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_mr_bind(self.c_mr, fid.fid(), flags) } ;
        
        if err != 0 {
            panic!("fi_mr_bind failed {}", err);
        }
    }

    pub fn refresh(&self, iov: &crate::IoVec, count: usize, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_mr_refresh(self.c_mr, iov.get(), count, flags) };

        if err != 0 {
            panic!("fi_mr_refresh failed {}", err);
        }
    }

    pub fn enable(&self) {
        let err = unsafe { libfabric_sys::inlined_fi_mr_enable(self.c_mr) };

        if err != 0 {
            panic!("fi_mr_enable failed {}", err);
        }
    }

    pub fn raw_attr(&self, base_addr: &mut u64, raw_key: &mut u8, key_size: &mut usize, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_mr_raw_attr(self.c_mr, base_addr as *mut u64, raw_key as *mut u8, key_size as *mut usize, flags) };

        if err != 0 {
            panic!("fi_mr_raw_attr failed {}", err);
        }        
    }
}

impl crate::FID for MemoryRegion{
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_mr).fid }
    }
}

//================== Memory Region attribute ==================//

pub struct MemoryRegionAttr {
    pub(crate) c_attr: libfabric_sys::fi_mr_attr,
}

impl MemoryRegionAttr {
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_mr_attr {
        &self.c_attr
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_mr_attr {
        &mut self.c_attr
    }
}


//================== Memory Region tests ==================//
#[allow(unused_imports)]
use crate::FID;

fn ft_alloc_bit_combo(fixed: u64, opt: u64) -> Vec<u64> {
    let bits_set = |mut val: u64 | -> u64 { let mut cnt = 0; while val > 0 {  cnt += 1 ; val &= val-1; } cnt };
    let num_flags = bits_set(opt) + 1;
    let len = 1 << (num_flags - 1) ;
    let mut flags = vec![0 as u64 ; num_flags as usize];
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
        for i in 0..8*std::mem::size_of::<u64>(){
            if index >> i & 1 == 1 {
                combos[index] |= flags[i];
            }
        }
    }

    combos
}
struct TestSizeParam(u64, u64);
const DEF_TEST_SIZES: [TestSizeParam; 6] = [TestSizeParam(1 << 0,0), TestSizeParam(1 << 1,0), TestSizeParam(1 << 2,0), TestSizeParam(1 << 3,0), TestSizeParam(1 << 4,0), TestSizeParam(1 << 5,0) ];

#[test]
fn mr_reg() {
    let ep_attr = crate::ep::EndpointAttr::new();
    let dom_attr = crate::domain::DomainAttr::new()
        .mode(!0)
        .mr_mode(!(crate::enums::MrMode::BASIC.get_value() | crate::enums::MrMode::SCALABLE.get_value() | crate::enums::MrType::LOCAL.get_value() ) as i32 );
    
    let hints = crate::InfoHints::new()
        .caps(crate::InfoCaps::new().msg().rma())
        .ep_attr(ep_attr)
        .domain_attr(dom_attr);

    let info = crate::Info::new().hints(hints).request();
    let entries: Vec<crate::InfoEntry> = info.get();
    
    if entries.len() > 0 {

        let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
        let mut domain = fab.domain(&entries[0]);

        let mut mr_access: u64 = 0;
        if entries[0].get_mode() & libfabric_sys::FI_LOCAL_MR == libfabric_sys::FI_LOCAL_MR || entries[0].get_domain_attr().get_mr_mode() as u32 & libfabric_sys::FI_MR_LOCAL == libfabric_sys::FI_MR_LOCAL {

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
        else {
            if entries[0].caps.is_rma() || entries[0].caps.is_atomic() {
                if entries[0].caps.is_remote_read() || !(entries[0].caps.is_read() || entries[0].caps.is_write() || entries[0].caps.is_remote_write()) {
                    mr_access |= libfabric_sys::FI_REMOTE_READ as u64 ;
                }
                else {
                    mr_access |= libfabric_sys::FI_REMOTE_WRITE as u64 ;
                }
            }
        }

        let combos = ft_alloc_bit_combo(0, mr_access);
        
        for i in 0..DEF_TEST_SIZES.len() {
            let buff_size = DEF_TEST_SIZES[i].0;
            let buf = vec![0 as u64; buff_size as usize ];
            for j in 0..combos.len() {
                let mut mr = domain.mr_reg(&buf, combos[j], 0, 0xC0DE, 0);
                mr.close();
            }
        }
        
        domain.close();
        fab.close();
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
//         .mr_mode(!(crate::enums::MrMode::BASIC.get_value() | crate::enums::MrMode::SCALABLE.get_value() | crate::enums::MrType::LOCAL.get_value() ) as i32 );
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
//             domain.close();
//             eq.close();
//             fab.close();
//             panic!("mr access == 0");            
//         }

//         domain.close();
//         eq.close();
//         fab.close();
//     }
//     else {
//         panic!("No capable fabric found!");
//     }
// }
