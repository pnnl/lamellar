use core::panic;
#[allow(unused_imports)]
use crate::FID;
use crate::InfoEntry;


//================== EventQueue (fi_eq) ==================//

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

    pub fn sread<T0>(&self, event: &mut u32, buf: &mut T0, timeout: i32, flags: u64) -> isize { // [TODO] Check return
        let ret = unsafe { libfabric_sys::inlined_fi_eq_sread(self.c_eq, event as *mut u32, buf as *mut T0 as *mut std::ffi::c_void, std::mem::size_of::<T0>(), timeout, flags) };
        if ret < 0 {
            let mut err_entry = EqErrEntry::new();
            let ret2 = self.readerr(&mut err_entry, 0);


            println!("sread error: {} {}", ret2, self.strerror(&err_entry));
        }
        ret
        // if err != 0 {
        //     panic!("fi_eq_sread failed {}", err);
        // }
    }

    pub fn readerr(&self, err: &mut EqErrEntry, flags: u64) -> isize {
        unsafe { libfabric_sys::inlined_fi_eq_readerr(self.c_eq, err.get_mut(), flags) }
    }

    pub fn strerror(&self, entry: &EqErrEntry) -> &str {
        let ret = unsafe { libfabric_sys::inlined_fi_eq_strerror(self.c_eq, entry.c_err.prov_errno, entry.c_err.err_data, std::ptr::null_mut(), 0) };
    
            unsafe{ std::ffi::CStr::from_ptr(ret).to_str().unwrap() }
    }

}

impl crate::FID for EventQueue {
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_eq).fid }
    }
}


//================== EventQueue Attribute(fi_eq_attr) ==================//

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

//================== EqErrEntry (fi_eq_err_entry) ==================//

pub struct EqErrEntry {
    pub(crate) c_err: libfabric_sys::fi_eq_err_entry,
}

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

//================== EventQueueEntry (fi_eq_entry) ==================//

pub struct EventQueueEntry<T> {
    c_entry: libfabric_sys::fi_eq_entry,
    phantom: std::marker::PhantomData<T>,
}

impl<T> EventQueueEntry<T> {
    const SIZE_OK: () = assert!(std::mem::size_of::<T>() == std::mem::size_of::<usize>(), 
    "The context of an EventQueueEntry must always be of size equal to usize datatype.");

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

//================== EventQueueCmEntry (fi_eq_cm_entry) ==================//

pub struct EventQueueCmEntry {
    c_entry: libfabric_sys::fi_eq_cm_entry,
}

impl EventQueueCmEntry {
    pub fn new() -> EventQueueCmEntry {


        let c_entry = libfabric_sys::fi_eq_cm_entry {
            fid: std::ptr::null_mut(),
            info: std::ptr::null_mut(),
            data: libfabric_sys::__IncompleteArrayField::<u8>::new(),
        };

        Self { c_entry }
    }

    pub fn get_info(&self) -> InfoEntry {
        InfoEntry::new(self.c_entry.info)
    }

}

//================== EventQueue related tests ==================//


#[test]
fn eq_write_read_self() {
    let info = crate::Info::new().request();
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
    let info = crate::Info::new().request();
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
    let info = crate::Info::new().request();
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
    let info = crate::Info::new().request();
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


#[test]
fn eq_open_close_sizes() {
    let info = crate::Info::new().request();
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