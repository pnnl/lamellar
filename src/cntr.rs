//================== Domain (fi_domain) ==================//

#[allow(unused_imports)]
use crate::FID;

pub struct Counter {
    pub(crate) c_cntr: *mut libfabric_sys::fid_cntr,
}

impl Counter {
    pub(crate) fn new(domain: &crate::domain::Domain, mut attr: CounterAttr) -> Self {
        let mut c_cntr: *mut libfabric_sys::fid_cntr = std::ptr::null_mut();
        let c_cntr_ptr: *mut *mut libfabric_sys::fid_cntr = &mut c_cntr;
        let err = unsafe { libfabric_sys::inlined_fi_cntr_open(domain.c_domain, attr.get_mut(), c_cntr_ptr, std::ptr::null_mut()) };

        if err != 0 {
            panic!("fi_cntr_open failed {}: {}", err,  crate::error_to_string(err.into()));
        }

        Self { c_cntr }
    }

    pub fn read(&self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_cntr_read(self.c_cntr) }
    }

    pub fn readerr(&self) -> u64 {
        unsafe { libfabric_sys::inlined_fi_cntr_readerr(self.c_cntr) }
    }

    pub fn add(&self, val: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_add(self.c_cntr, val) };
    
        if err != 0 {
            panic!("fi_cntr_add failed {}: {}", err, crate::error_to_string(err.into()));
        }
    }

    pub fn adderr(&self, val: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_adderr(self.c_cntr, val) };
            
        if err != 0 {
            panic!("fi_cntr_adderr failed {}: {}", err, crate::error_to_string(err.into()));
        }
    }

    pub fn set(&self, val: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_set(self.c_cntr, val) };
            
        if err != 0 {
            panic!("fi_cntr_set failed {}: {}", err, crate::error_to_string(err.into()));
        }
    }

    pub fn seterr(&self, val: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_cntr_seterr(self.c_cntr, val) };
            
        if err != 0 {
            panic!("fi_cntr_seterr failed {}: {}", err, crate::error_to_string(err.into()));
        }
    }

    pub fn wait(&self, threshold: u64, timeout: i32) -> i32 { // [TODO]
        unsafe { libfabric_sys::inlined_fi_cntr_wait(self.c_cntr, threshold, timeout) }
    }
}


impl crate::FID for Counter {
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_cntr).fid }
    }
}

//================== Counter attribute ==================//

#[derive(Clone, Copy)]
pub struct CounterAttr {
    pub(crate) c_attr: libfabric_sys::fi_cntr_attr,
}
// pub struct fi_cntr_attr {
//     pub events: fi_cntr_events,
//     pub wait_obj: fi_wait_obj,
//     pub wait_set: *mut fid_wait,
//     pub flags: u64,
// }
impl CounterAttr {

    pub fn new() -> Self {
        let c_attr = libfabric_sys::fi_cntr_attr {
            events: 0,
            wait_obj: 0,
            wait_set: std::ptr::null_mut(),
            flags: 0,
        };

        Self { c_attr }
    }

    pub fn events(&mut self, events: crate::enums::CounterEvents) -> &mut Self {
        self.c_attr.events = events.get_value();

        self
    }

    pub fn wait_obj(&mut self, wait_obj: crate::enums::WaitObj) -> &mut Self {
        self.c_attr.wait_obj = wait_obj.get_value();

        self
    }

    pub fn wait_set(&mut self, wait_set: &crate::sync::Wait) -> &mut Self {
        self.c_attr.wait_set = wait_set.c_wait;

        self
    }

    pub fn flags(&mut self, flags: u64) -> &mut Self {
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

//================== Counter tests ==================//

#[cfg(test)]
mod tests {
    use crate::FID;

    #[test]
    fn cntr_loop() {

        let dom_attr = crate::domain::DomainAttr::new()
            .mode(!0)
            .mr_mode(!(crate::enums::MrMode::BASIC.get_value() | crate::enums::MrMode::SCALABLE.get_value()) as i32 );
        
        let hints = crate::InfoHints::new()
            .domain_attr(dom_attr)
            .mode(!0);
        

        let info = crate::Info::new().hints(hints).request();
        let entries: Vec<crate::InfoEntry> = info.get();
        
        if entries.len() > 0 {
            for e in entries {
                if e.get_domain_attr().get_cntr_cnt() != 0 {
                    let fab = crate::fabric::Fabric::new(e.fabric_attr.clone());
                    let domain = fab.domain(&e);
                    let cntr_cnt = std::cmp::min(e.get_domain_attr().get_cntr_cnt(), 100);
                    let cntrs: Vec<crate::cntr::Counter> = (0..cntr_cnt).map(|_| domain.cntr_open(crate::cntr::CounterAttr::new())).collect();

                    for (i,cntr) in cntrs.iter().enumerate() {
                        cntr.set(i as u64);
                        cntr.seterr((i << 1) as u64);
                    }
                    
                    for (i,cntr) in cntrs.iter().enumerate() {
                        cntr.add(i as u64);
                        cntr.adderr(i as u64);
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
                    
                    for cntr in cntrs {
                        cntr.close();
                    }

                    domain.close();
                    fab.close();
                    break;
                }

            }

        }
        else {
            panic!("Could not find suitable fabric");
        }
    }
}