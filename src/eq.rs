use core::panic;
#[allow(unused_imports)]
use crate::FID;

pub struct EventQueue {
    c_eq: *mut libfabric_sys::fid_eq,
}

impl EventQueue {
    pub fn new(fabric: &crate::fabric::Fabric, mut attr: EventQueueAttr) -> Self {
        let mut c_eq: *mut libfabric_sys::fid_eq  = std::ptr::null_mut();
        let c_eq_ptr: *mut *mut libfabric_sys::fid_eq = &mut c_eq;

        let err = unsafe {libfabric_sys::inlined_fi_eq_open(fabric.c_fabric, attr.get_mut(), c_eq_ptr, std::ptr::null_mut())};
        if err != 0 {
            panic!("fi_eq_open failed {}", err);
        }

        Self { c_eq }
    }

    pub fn read<T0>(&self, event: &mut u32, buf: &mut [T0], flags: u64) -> isize{
        let err = unsafe { libfabric_sys::inlined_fi_eq_read(self.c_eq, event as *mut u32, buf.as_mut_ptr() as *mut std::ffi::c_void, buf.len() * std::mem::size_of::<T0>(), flags) };

        // if err < 0 {
        //     panic!("fi_eq_read failed {}", err);
        // }
        err
    }

    pub fn write<T0>(&self, event: u32, buf: & [T0], flags: u64) -> isize{
        // println!("{:?}", buf);
        let err = unsafe { libfabric_sys::inlined_fi_eq_write(self.c_eq, event, buf.as_ptr() as *const std::ffi::c_void, buf.len() * std::mem::size_of::<T0>(), flags) };

        // if err < 0 {
        //     panic!("fi_eq_read write {}", err);
        // }
        err
    }


    pub fn sread<T0>(&self, event: &mut u32, buf: &mut [T0], timeout: i32, flags: u64) -> isize { // [TODO] Check return
        unsafe { libfabric_sys::inlined_fi_eq_sread(self.c_eq, event as *mut u32, buf.as_mut_ptr() as *mut std::ffi::c_void, buf.len() * std::mem::size_of::<T0>(), timeout, flags) }

        // if err != 0 {
        //     panic!("fi_eq_sread failed {}", err);
        // }
    }

    pub fn readerr(&self, err: &mut EqErrEntry, flags: u64) -> isize {
        unsafe { libfabric_sys::inlined_fi_eq_readerr(self.c_eq, err.get_mut(), flags) }
    }

    pub fn strerror<T0>(&self, prov_errno: i32, err_data: &T0, buf: String) -> &str {
        let len = buf.len();
        let c_str = std::ffi::CString::new(buf).unwrap();
        let raw = c_str.into_raw();
        let ret = unsafe { libfabric_sys::inlined_fi_eq_strerror(self.c_eq, prov_errno, err_data as *const T0 as *const std::ffi::c_void, raw, len) };
    
            unsafe{ std::ffi::CStr::from_ptr(ret).to_str().unwrap() }
    }

}

impl crate::FID for EventQueue {
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_eq).fid }
    }
}

pub struct CommandQueue {
    pub(crate) c_cq: *mut libfabric_sys::fid_cq,
}

impl crate::FID for CommandQueue {
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_cq).fid }
    }
}

pub struct CommandQueueAttr {
    pub(crate) c_attr: libfabric_sys::fi_cq_attr,
}

impl CommandQueueAttr {
    // pub size: usize,
    // pub flags: u64,
    // pub format: fi_cq_format,
    // pub wait_obj: fi_wait_obj,
    // pub signaling_vector: ::std::os::raw::c_int,
    // pub wait_cond: fi_cq_wait_cond,
    // pub wait_set: *mut fid_wait,
    // pub fn new() -> Self {
    //     let c_attr = libfabric_sys::fi_cq_attr{};
    // }
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

impl CommandQueue {
    pub(crate) fn new(domain: &crate::domain::Domain, mut attr: CommandQueueAttr) -> Self {
        let mut c_cq: *mut libfabric_sys::fid_cq  = std::ptr::null_mut();
        let c_cq_ptr: *mut *mut libfabric_sys::fid_cq = &mut c_cq;

        let err = unsafe {libfabric_sys::inlined_fi_cq_open(domain.c_domain, attr.get_mut(), c_cq_ptr, std::ptr::null_mut())};
        if err != 0 {
            panic!("fi_cq_open failed {}", err);
        }

        Self { c_cq } 
    }

    pub(crate) fn new_with_context<T0>(domain: &crate::domain::Domain, mut attr: CommandQueueAttr, context: &mut T0) -> Self {
        let mut c_cq: *mut libfabric_sys::fid_cq  = std::ptr::null_mut();
        let c_cq_ptr: *mut *mut libfabric_sys::fid_cq = &mut c_cq;

        let err = unsafe {libfabric_sys::inlined_fi_cq_open(domain.c_domain, attr.get_mut(), c_cq_ptr, context as *mut T0 as *mut std::ffi::c_void)};
        if err != 0 {
            panic!("fi_cq_open failed {}", err);
        }

        Self { c_cq } 
    }

    pub fn read<T0>(&self, buf: &mut [T0], count: usize) -> isize {
        unsafe { libfabric_sys::inlined_fi_cq_read(self.c_cq, buf.as_mut_ptr() as *mut std::ffi::c_void, count) }
    }

    pub fn readfrom<T0>(&self, buf: &mut [T0], count: usize, address: &mut crate::Address) -> isize {
        unsafe { libfabric_sys::inlined_fi_cq_readfrom(self.c_cq, buf.as_mut_ptr() as *mut std::ffi::c_void, count, address as *mut crate::Address) }
    }

    pub fn sread_with_cond<T0, T1>(&self, buf: &mut [T0], count: usize, cond: &T1, timeout: i32) -> isize {
        unsafe { libfabric_sys::inlined_fi_cq_sread(self.c_cq, buf.as_mut_ptr() as *mut std::ffi::c_void, count, cond as *const T1 as *const std::ffi::c_void, timeout) }
    }

    pub fn sread<T0>(&self, buf: &mut [T0], count: usize, timeout: i32) -> isize {
        unsafe { libfabric_sys::inlined_fi_cq_sread(self.c_cq, buf.as_mut_ptr() as *mut std::ffi::c_void, count, std::ptr::null_mut(), timeout) }
    }

    pub fn sreadfrom<T0, T1>(&self, buf: &mut [T0], count: usize, address: &mut crate::Address, cond: &T1, timeout: i32) -> isize {
        unsafe { libfabric_sys::inlined_fi_cq_sreadfrom(self.c_cq, buf.as_mut_ptr() as *mut std::ffi::c_void, count, address as *mut crate::Address, cond as *const T1 as *const std::ffi::c_void, timeout) }
    }

    pub fn signal(&self) {
        let err = unsafe { libfabric_sys::inlined_fi_cq_signal(self.c_cq) };

        if err != 0 {
            panic!("fi_cq_signal failed {}", err);
        }
    }

    pub fn readerr(&self, err: &mut CqErrEntry, flags: u64) -> isize {
        unsafe { libfabric_sys::inlined_fi_cq_readerr(self.c_cq, err.get_mut(), flags) }
    }

    pub fn strerror<T0, T1>(&self, prov_errno: i32, err_data: &T0, buf: String) -> &str {
        let len = buf.len();
        let c_str = std::ffi::CString::new(buf).unwrap();
        let raw = c_str.into_raw();
        let ret = unsafe { libfabric_sys::inlined_fi_cq_strerror(self.c_cq, prov_errno, err_data as *const T0 as *const std::ffi::c_void, raw , len) };
    
            unsafe{ std::ffi::CStr::from_ptr(ret).to_str().unwrap() }
    }
}


pub struct EventQueueAttr {
    c_attr: libfabric_sys::fi_eq_attr,
}

impl EventQueueAttr {

    pub fn new() -> Self {
        let c_attr = libfabric_sys::fi_eq_attr{ 
            size: 0, 
            flags: 0, 
            wait_obj: libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC,
            signaling_vector: 0, 
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
    
    pub fn wait_obj(&mut self, wait_obj: crate::enums::WaitObj) -> &mut Self {
        self.c_attr.wait_obj = wait_obj.get_value();
        self
    }

    pub fn signaling_vector(&mut self, signaling_vector: i32) -> &mut Self {
        self.c_attr.signaling_vector = signaling_vector;
        self
    }

    pub fn wait_set(&mut self, wait_set: &crate::sync::Wait) -> &mut Self {
        self.c_attr.wait_set = wait_set.c_wait;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_eq_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_eq_attr {
        &mut self.c_attr
    }    
}

pub struct EqErrEntry {
    pub(crate) c_err: libfabric_sys::fi_eq_err_entry,
}
// pub fid: fid_t,
// pub context: *mut ::std::os::raw::c_void,
// pub data: u64,
// pub err: ::std::os::raw::c_int,
// pub prov_errno: ::std::os::raw::c_int,
// pub err_data: *mut ::std::os::raw::c_void,
// pub err_data_size: usize,

impl EqErrEntry {
    pub fn new() -> Self {
        let c_err = libfabric_sys::fi_eq_err_entry{
            fid: std::ptr::null_mut(),
            context: std::ptr::null_mut(),
            data: 0,
            err: 0,
            prov_errno: 0,
            err_data: std::ptr::null_mut(),
            err_data_size: 0,
        };

        Self { c_err }
    }
    
    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_eq_err_entry {
        &self.c_err
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_eq_err_entry {
        &mut self.c_err
    }       
}

pub struct CqErrEntry {
    pub(crate) c_err: libfabric_sys::fi_cq_err_entry,
}


impl CqErrEntry {
    
    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_cq_err_entry {
        &self.c_err
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_cq_err_entry {
        &mut self.c_err
    }       
}

#[test]
fn cq_open_close_sizes() {
    let info = crate::Info::all();
    let entries = info.get();
    
    let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
    let mut eq = fab.eq_open(crate::eq::EventQueueAttr::new());
    let mut domain = fab.domain(&entries[0]);
    for i in -1..17 {
        let size ;
        if i == -1 {
            size = 0;
        }
        else {
            size = 1 << i;
        }
        let mut cq_attr = CommandQueueAttr::new();
        cq_attr.size(size); 
        let mut cq = domain.cq_open(cq_attr);
        cq.close();
    }
    domain.close();
    eq.close();
    fab.close();
}

#[test]
fn cq_open_close_simultaneous() {
    let info = crate::Info::all();
    let entries = info.get();
    
    let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
    let count = 10;
    let mut eq = fab.eq_open(crate::eq::EventQueueAttr::new());
    let mut domain = fab.domain(&entries[0]);
    let mut cqs = Vec::new();
    for _ in 0..count {
        let cq_attr = CommandQueueAttr::new();
        let cq = domain.cq_open(cq_attr);
        cqs.push(cq);
    }

    for mut cq in cqs {
        cq.close();
    }
    domain.close();
    eq.close();
    fab.close();
}

#[test]
fn cq_signal() {
    let info = crate::Info::all();
    let entries = info.get();
    let mut buf = vec![0,0,0];
    
    let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
    let mut eq = fab.eq_open(crate::eq::EventQueueAttr::new());
    let mut domain = fab.domain(&entries[0]);
    let mut cq_attr = CommandQueueAttr::new();
    cq_attr.size(1);
    let mut cq = domain.cq_open(cq_attr);
    cq.signal();
    let ret = cq.sread(&mut buf[..], 1, 2000);
    if ret != -(libfabric_sys::FI_EAGAIN as isize)  && ret != -(libfabric_sys::FI_ECANCELED as isize) {
        panic!("sread {}", ret);
    }
    cq.close();

    domain.close();
    eq.close();
    fab.close();
}

#[test]
fn eq_open_close_sizes() {
    let info = crate::Info::all();
    let entries = info.get();
    
    let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
    for i in -1..17 {
        let size ;
        if i == -1 {
            size = 0;
        }
        else {
            size = 1 << i;
        }
        let mut eq_attr = crate::eq::EventQueueAttr::new();
        eq_attr.size(size);
        let mut eq = fab.eq_open(eq_attr);
        eq.close();
    }
    fab.close();
}

#[repr(C)]
pub struct EventQueueEntry<T> {
    c_entry: libfabric_sys::fi_eq_entry,
    phantom: std::marker::PhantomData<T>,
}

impl<T> EventQueueEntry<T> {
    const SIZE_OK: () = assert!(std::mem::size_of::<T>() == std::mem::size_of::<usize>(), "The context of an EventQueueEntry must always be of size equal to usize datatype.");

    pub fn new() -> Self {
        let _ = Self::SIZE_OK;
        let c_entry = libfabric_sys::fi_eq_entry { 
            fid: std::ptr::null_mut(), 
            context: std::ptr::null_mut(), 
            data: 0 
        };

        Self { c_entry, phantom: std::marker::PhantomData }
    }

    pub fn fid(&mut self, fid: &impl crate::FID) -> &mut Self {
        self.c_entry.fid = fid.fid();
        self
    }

    pub(crate) fn get_fid(&self) -> *mut libfabric_sys::fid {
        self.c_entry.fid
    }

    pub fn context(&mut self, context: &mut T) -> &mut Self {
        let context_writable: *mut *mut std::ffi::c_void =  &mut (self.c_entry.context);
        let context_writable_usize: *mut  usize = context_writable as  *mut usize;
        unsafe { *context_writable_usize = *(context as *mut T as *mut usize) };
        self
    }

    pub fn data(&mut self, data: u64) -> &mut Self {
        self.c_entry.data = data;
        self
    }

    pub fn get_context(&self) -> T {
        let context_ptr:*mut *mut T = &mut (self.c_entry.context as *mut T);
        let res = unsafe { std::mem::transmute_copy::<T,T>(&*(context_ptr as *const T)) } ;

        res
    }

}

#[test]
fn eq_write_read_self() {
    let info = crate::Info::all();
    let entries: Vec<crate::InfoEntry> = info.get();
    let mut eq_attr = EventQueueAttr::new();
    eq_attr.size(32)
        .flags(libfabric_sys::FI_WRITE.into())
        .wait_obj(crate::enums::WaitObj::NONE);
    let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
    let mut eq = fab.eq_open(eq_attr);
    for mut i in 0 as usize ..5 {
        let mut entry: EventQueueEntry<usize> = EventQueueEntry::new();
        if i & 1 == 1 {
            entry.fid(&fab);
        }
        else {
            entry.fid(&eq);
        }

        entry.context(&mut i);
        let ret = eq.write(libfabric_sys::FI_NOTIFY, &[entry], 0);
        if ret != std::mem::size_of::<EventQueueEntry<usize>>().try_into().unwrap() {
            panic!("eq.write failed {}", ret);
        }
    }
    for i in 0..10 {
        let mut event = 0;
        let mut entry: [EventQueueEntry<usize>; 1] = [EventQueueEntry::new()];
        let ret = eq.read(&mut event, &mut entry, if (i & 1) == 1 { 0 } else { libfabric_sys::FI_PEEK.into()});
        if ret != std::mem::size_of::<EventQueueEntry<usize>>().try_into().unwrap() {
            panic!("eq.read failed {}", ret);
        }
        if event != libfabric_sys::FI_NOTIFY {
            panic!("Unexpected event {}", event);
        }

        if entry[0].get_context() != i /2 {
            panic!("Unexpected context {} vs {}", entry[0].get_context(), i/2);
        }

        if entry[0].get_fid() != if i & 2 == 2 {fab.fid()} else {eq.fid()} {
            panic!("Unexpected fid {:?}", entry[0].get_fid());
        }
    }
    let entry: EventQueueEntry<usize> = EventQueueEntry::new();
    let mut event = 0;
    let ret = eq.read(&mut event, &mut [entry], 0);
    if ret != - (libfabric_sys::FI_EAGAIN as isize) {
        panic!("fi_eq_read of empty EQ returned {}", ret);
    }
    eq.close();
    fab.close();
}

#[test]
fn eq_size_verify() {
    let info = crate::Info::all();
    let entries: Vec<crate::InfoEntry> = info.get();
    let mut eq_attr = EventQueueAttr::new();
    eq_attr.size(32)
        .flags(libfabric_sys::FI_WRITE.into())
        .wait_obj(crate::enums::WaitObj::NONE);
    let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
    let mut eq = fab.eq_open(eq_attr);

    for mut i in 0 as usize .. 32 {
        let mut entry: EventQueueEntry<usize> = EventQueueEntry::new();
        entry
            .fid(&fab)
            .context(&mut i);
        let ret = eq.write(libfabric_sys::FI_NOTIFY, &mut [entry], 0);
        if ret != std::mem::size_of::<EventQueueEntry<usize>>().try_into().unwrap() {
            panic!("eq.write write size != eventqueueentry {}", ret);
        }
    }

    eq.close();
    fab.close();
}

#[test]
fn eq_write_sread_self() {
    let info = crate::Info::all();
    let entries: Vec<crate::InfoEntry> = info.get();
    let mut eq_attr = EventQueueAttr::new();
    eq_attr.size(32)
        .flags(libfabric_sys::FI_WRITE.into())
        .wait_obj(crate::enums::WaitObj::FD);
    let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
    let mut eq = fab.eq_open(eq_attr);
    for mut i in 0 as usize ..5 {
        let mut entry: EventQueueEntry<usize> = EventQueueEntry::new();
        if i & 1 == 1 {
            entry.fid(&fab);
        }
        else {
            entry.fid(&eq);
        }

        entry.context(&mut i);
        let ret = eq.write(libfabric_sys::FI_NOTIFY, &[entry], 0);
        if ret != std::mem::size_of::<EventQueueEntry<usize>>().try_into().unwrap() {
            panic!("eq.write failed {}", ret);
        }
    }
    for i in 0..10 {
        let mut event = 0;
        let mut entry: [EventQueueEntry<usize>; 1] = [EventQueueEntry::new()];
        let ret = eq.sread(&mut event, &mut entry, 2000 ,if (i & 1) == 1 { 0 } else { libfabric_sys::FI_PEEK.into()});
        if ret != std::mem::size_of::<EventQueueEntry<usize>>().try_into().unwrap() {
            panic!("sread failed {}", ret);
        }
        if event != libfabric_sys::FI_NOTIFY {
            panic!("Unexpected event {}", event);
        }

        if entry[0].get_context() != i /2 {
            panic!("Unexpected context {} vs {}", entry[0].get_context(), i/2);
        }

        if entry[0].get_fid() != if i & 2 == 2 {fab.fid()} else {eq.fid()} {
            panic!("Unexpected fid {:?}", entry[0].get_fid());
        }
    }
    let entry: EventQueueEntry<usize> = EventQueueEntry::new();
    let mut event = 0;
    let ret = eq.read(&mut event, &mut [entry], 0);
    if ret != - (libfabric_sys::FI_EAGAIN as isize) {
        panic!("fi_eq_read of empty EQ returned {}", ret);
    }
    eq.close();
    fab.close();
}

#[test]
fn eq_readerr() {
    let info = crate::Info::all();
    let entries: Vec<crate::InfoEntry> = info.get();
    let mut eq_attr = EventQueueAttr::new();
    eq_attr.size(32)
        .flags(libfabric_sys::FI_WRITE.into())
        .wait_obj(crate::enums::WaitObj::FD);
    let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
    let mut eq = fab.eq_open(eq_attr);
    for mut i in 0 as usize ..5 {
        let mut entry: EventQueueEntry<usize> = EventQueueEntry::new();
        entry.fid(&fab);

        entry.context(&mut i);
        let ret = eq.write(libfabric_sys::FI_NOTIFY, &[entry], 0);
        if ret != std::mem::size_of::<EventQueueEntry<usize>>().try_into().unwrap() {
            panic!("eq.write failed {}", ret);
        }
    }
    for i in 0..5 {
        let mut event = 0;
        let mut entry: [EventQueueEntry<usize>; 1] = [EventQueueEntry::new()];
        let ret = eq.read(&mut event, &mut entry , 0);
        if ret != std::mem::size_of::<EventQueueEntry<usize>>().try_into().unwrap() {
            panic!("Eq.read failed {}", ret);
        }
        if event != libfabric_sys::FI_NOTIFY {
            panic!("Unexpected event {}", event);
        }

        if entry[0].get_context() != i  {
            panic!("Unexpected context {} vs {}", entry[0].get_context(), i/2);
        }

        if entry[0].get_fid() != fab.fid() {
            panic!("Unexpected fid {:?}", entry[0].get_fid());
        }
    }
    let mut err_entry = EqErrEntry::new();
    let err = eq.readerr(&mut err_entry, 0);
    if err != - (libfabric_sys::FI_EAGAIN as isize) {
        panic!("eq.readerr failed {}", err);
    }
    eq.close();
    fab.close();
}

