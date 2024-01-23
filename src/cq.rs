use debug_print::debug_println;

#[allow(unused_imports)]
use crate::FID;

//================== CompletionQueue (fi_cq) ==================//


pub struct CompletionQueue {
    pub(crate) c_cq: *mut libfabric_sys::fid_cq,
}

impl CompletionQueue {
    pub(crate) fn new(domain: &crate::domain::Domain, mut attr: CompletionQueueAttr) -> Result<CompletionQueue, crate::error::Error> {
        let mut c_cq: *mut libfabric_sys::fid_cq  = std::ptr::null_mut();
        let c_cq_ptr: *mut *mut libfabric_sys::fid_cq = &mut c_cq;

        let err = unsafe {libfabric_sys::inlined_fi_cq_open(domain.c_domain, attr.get_mut(), c_cq_ptr, std::ptr::null_mut())};
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(
                Self { c_cq } 
            )
        }
    }

    pub(crate) fn new_with_context<T0>(domain: &crate::domain::Domain, mut attr: CompletionQueueAttr, context: &mut T0) -> Result<CompletionQueue, crate::error::Error> {
        let mut c_cq: *mut libfabric_sys::fid_cq  = std::ptr::null_mut();
        let c_cq_ptr: *mut *mut libfabric_sys::fid_cq = &mut c_cq;

        let err = unsafe {libfabric_sys::inlined_fi_cq_open(domain.c_domain, attr.get_mut(), c_cq_ptr, context as *mut T0 as *mut std::ffi::c_void)};
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(
                Self { c_cq } 
            )
        }
    }

    pub fn read<T0>(&self, buf: &mut [T0], count: usize) -> Result<usize, crate::error::Error> {
        let ret = unsafe { libfabric_sys::inlined_fi_cq_read(self.c_cq, buf.as_mut_ptr() as *mut std::ffi::c_void, count) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(ret as usize)
        }
    }

    pub fn readfrom<T0>(&self, buf: &mut [T0], count: usize, address: &mut crate::Address) -> Result<usize, crate::error::Error> {
        let ret = unsafe { libfabric_sys::inlined_fi_cq_readfrom(self.c_cq, buf.as_mut_ptr() as *mut std::ffi::c_void, count, address as *mut crate::Address) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(ret as usize)
        }
    }

    // [TODO]  Condition is not taken into account
    pub fn sread_with_cond<T0, T1>(&self, buf: &mut [T0], count: usize, cond: &T1, timeout: i32) -> Result<usize, crate::error::Error> {
        let ret = unsafe { libfabric_sys::inlined_fi_cq_sread(self.c_cq, buf.as_mut_ptr() as *mut std::ffi::c_void, count, cond as *const T1 as *const std::ffi::c_void, timeout) };
    
        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(ret as usize)
        }
    }

    pub fn sread<T0>(&self, buf: &mut [T0], count: usize, timeout: i32) -> Result<usize, crate::error::Error> {
        let ret  = unsafe { libfabric_sys::inlined_fi_cq_sread(self.c_cq, buf.as_mut_ptr() as *mut std::ffi::c_void, count, std::ptr::null_mut(), timeout) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()))
        }
        else {
            Ok(ret as usize)
        }
        // if ret < 0 {
        //     let mut err_entry = CqErrEntry::new();
        //     let ret2 = self.readerr(&mut err_entry, 0);


        //     println!("sread error: {} {}", ret2, unsafe{ self.strerror(err_entry.get_prov_errno(), err_entry.get_err_data(), err_entry.get_err_data_size()) } );
        // }
    }

    pub fn sreadfrom<T0, T1>(&self, buf: &mut [T0], count: usize, address: &mut crate::Address, cond: &T1, timeout: i32) -> Result<usize, crate::error::Error> {
        let ret = unsafe { libfabric_sys::inlined_fi_cq_sreadfrom(self.c_cq, buf.as_mut_ptr() as *mut std::ffi::c_void, count, address as *mut crate::Address, cond as *const T1 as *const std::ffi::c_void, timeout) };
    
        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()))
        }
        else {
            Ok(ret as usize)
        }
    }

    pub fn signal(&self) -> Result<(), crate::error::Error>{
        let err = unsafe { libfabric_sys::inlined_fi_cq_signal(self.c_cq) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn readerr(&self, err: &mut CqErrEntry, flags: u64) -> Result<usize, crate::error::Error> {
        let ret = unsafe { libfabric_sys::inlined_fi_cq_readerr(self.c_cq, err.get_mut(), flags) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()))
        }
        else {
            Ok(ret as usize)
        }
    }
    pub fn print_error(&self, err_entry: &crate::cq::CqErrEntry) {
        println!("{}", unsafe{self.strerror(err_entry.get_prov_errno(), err_entry.get_err_data(), err_entry.get_err_data_size())} );
    }

    unsafe fn strerror(&self, prov_errno: i32, err_data: *const std::ffi::c_void, err_data_size: usize) -> &str {
        // let len = buf.len();
        // let c_str = std::ffi::CString::new(buf).unwrap();
        // let raw = c_str.into_raw();
        let ret = unsafe { libfabric_sys::inlined_fi_cq_strerror(self.c_cq, prov_errno, err_data, std::ptr::null_mut() , err_data_size) };
    
            unsafe{ std::ffi::CStr::from_ptr(ret).to_str().unwrap() }
    }
}

impl crate::FID for CompletionQueue {
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_cq).fid }
    }
}

impl crate::Bind for CompletionQueue {
    
}

impl Drop for CompletionQueue {
    fn drop(&mut self) {
        debug_println!("Dropping cq");

        self.close().unwrap()
    }
}

//================== CompletionQueue Attribute (fi_cq_attr) ==================//


pub struct CompletionQueueAttr {
    pub(crate) c_attr: libfabric_sys::fi_cq_attr,
}

impl CompletionQueueAttr {

    pub fn new() -> Self {
        let c_attr = libfabric_sys::fi_cq_attr{
            size: 0, 
            flags: 0, 
            format: crate::enums::CqFormat::UNSPEC.get_value(), 
            wait_obj: crate::enums::WaitObj::UNSPEC.get_value(),
            signaling_vector: 0,
            wait_cond: crate::enums::WaitCond::NONE.get_value(),
            wait_set: std::ptr::null_mut()
        };

        Self {c_attr}
    }

    pub fn size(&mut self, size: usize) -> &mut Self {
        self.c_attr.size = size;
        self
    }

    pub fn flags(&mut self, flags: u64) -> &mut Self {
        self.c_attr.flags = flags;
        self
    }

    pub fn format(&mut self, format: crate::enums::CqFormat) -> &mut Self {
        self.c_attr.format = format.get_value();
        self
    }

    pub fn wait_obj(&mut self, wait_obj: crate::enums::WaitObj) -> &mut Self {
        self.c_attr.wait_obj = wait_obj.get_value();
        self
    }
    
    pub fn signaling_vector(&mut self, signaling_vector: i32) -> &mut Self {
        self.c_attr.signaling_vector = signaling_vector;
        self
    }

    pub fn wait_cond(&mut self, wait_cond: crate::enums::WaitCond) -> &mut Self {
        self.c_attr.wait_cond = wait_cond.get_value();
        self
    }

    pub fn wait_set(&mut self, wait_set: &crate::sync::Wait) -> &mut Self {
        self.c_attr.wait_set = wait_set.c_wait;
        self
    }


    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_cq_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_cq_attr {
        &mut self.c_attr
    }
}

//================== CompletionQueue Error Entry (fi_cq_err_entry) ==================//


pub struct CqErrEntry {
    pub(crate) c_err: libfabric_sys::fi_cq_err_entry,
}

impl CqErrEntry {

    pub fn new() -> Self {
        Self {
            c_err: libfabric_sys::fi_cq_err_entry {
                op_context: std::ptr::null_mut(),
                flags: 0,
                len: 0,
                buf: std::ptr::null_mut(),
                data: 0,
                tag: 0,
                olen: 0,
                err: 0,
                prov_errno: 0,
                err_data: std::ptr::null_mut(),
                err_data_size: 0,
            }
        }
    }
    
    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_cq_err_entry {
        &self.c_err
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_cq_err_entry {
        &mut self.c_err
    }       

    pub fn get_prov_errno(&self) -> i32 {
        self.c_err.prov_errno
    }

    pub(crate) fn get_err_data(&self) -> *const std::ffi::c_void {
        self.c_err.err_data
    }

    pub(crate) fn get_err_data_size(&self) -> usize {
        self.c_err.err_data_size
    }
}
//================== CompletionQueue Tests ==================//

#[cfg(test)]
mod tests {
    use crate::cq::*;

    #[test]
    fn cq_open_close_simultaneous() {
        let info = crate::Info::new().request().unwrap();
        let entries = info.get();
        
        let fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone()).unwrap();
        let count = 10;
        let _eq = fab.eq_open(crate::eq::EventQueueAttr::new()).unwrap();
        let domain = fab.domain(&entries[0]).unwrap();
        let mut cqs = Vec::new();
        for _ in 0..count {
            let cq_attr = crate::cq::CompletionQueueAttr::new();
            let cq = domain.cq_open(cq_attr).unwrap();
            cqs.push(cq);
        }
    }

    #[test]
    fn cq_signal() {
        let info = crate::Info::new().request().unwrap();
        let entries = info.get();
        let mut buf = vec![0,0,0];
        
        let fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone()).unwrap();
        let _eq = fab.eq_open(crate::eq::EventQueueAttr::new()).unwrap();
        let domain = fab.domain(&entries[0]).unwrap();
        let mut cq_attr = CompletionQueueAttr::new();
        cq_attr.size(1);
        let cq = domain.cq_open(cq_attr).unwrap();
        cq.signal().unwrap();
        let ret = cq.sread(&mut buf, 1, 2000);
        if let Err(ref err) = ret {
            if ! (matches!(err.kind, crate::error::ErrorKind::TryAgain) || matches!(err.kind, crate::error::ErrorKind::Canceled)) {
                ret.unwrap();
            }
        }
    }

    #[test]
    fn cq_open_close_sizes() {
        let info = crate::Info::new().request().unwrap();
        let entries = info.get();
        
        let fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone()).unwrap();
        let domain = fab.domain(&entries[0]).unwrap();
        for i in -1..17 {
            let size ;
            if i == -1 {
                size = 0;
            }
            else {
                size = 1 << i;
            }
            let mut cq_attr = CompletionQueueAttr::new();
            cq_attr.size(size); 
            let _cq = domain.cq_open(cq_attr).unwrap();
        }
    }
}