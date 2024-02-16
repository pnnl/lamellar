use debug_print::debug_println;

#[allow(unused_imports)]
use crate::FID;
use crate::{enums::Event, InfoEntry};


//================== EventQueue (fi_eq) ==================//

pub struct EventQueue {
    c_eq: *mut libfabric_sys::fid_eq,
}

impl EventQueue {
    pub(crate) fn new(fabric: &crate::fabric::Fabric, mut attr: EventQueueAttr) -> Result<Self, crate::error::Error> {
        let mut c_eq: *mut libfabric_sys::fid_eq  = std::ptr::null_mut();
        let c_eq_ptr: *mut *mut libfabric_sys::fid_eq = &mut c_eq;

        let err = unsafe {libfabric_sys::inlined_fi_eq_open(fabric.c_fabric, attr.get_mut(), c_eq_ptr, std::ptr::null_mut())};
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_eq }
            )
        }
    }

    pub fn read<T0>(&self, buf: &mut [T0]) -> Result<(usize, Event), crate::error::Error>{
        let mut event = 0 ;
        
        let ret = unsafe { libfabric_sys::inlined_fi_eq_read(self.c_eq, &mut event as *mut u32, buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), 0) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok((ret as usize, Event::from_value(event)))
        }
    }

    pub fn peek<T0>(&self, buf: &mut [T0]) -> Result<(usize, Event), crate::error::Error>{
        let mut event = 0 ;
        
        let ret = unsafe { libfabric_sys::inlined_fi_eq_read(self.c_eq, &mut event as *mut u32, buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), libfabric_sys::FI_PEEK.into()) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok((ret as usize, Event::from_value(event)))
        }
    }

    pub fn write<T0>(&self, event: Event, buf: & [T0], flags: u64) -> Result<usize, crate::error::Error>{
        // println!("{:?}", buf);
        let ret = unsafe { libfabric_sys::inlined_fi_eq_write(self.c_eq, event.get_value(), buf.as_ptr() as *const std::ffi::c_void, std::mem::size_of_val(buf), flags) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(ret as usize)
        }
    }

    pub fn sread<T0>(&self, buf: &mut [T0], timeout: i32, flags: u64) -> Result<(usize, Event), crate::error::Error> { 
        let mut event = 0;
        let ret = unsafe { libfabric_sys::inlined_fi_eq_sread(self.c_eq, &mut event as *mut u32,  buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), timeout, flags) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok((ret as usize, Event::from_value(event)))
        }
    }

    pub fn speek<T0>(&self, buf: &mut [T0], timeout: i32) -> Result<(usize, Event), crate::error::Error> { 
        let mut event = 0;
        let ret = unsafe { libfabric_sys::inlined_fi_eq_sread(self.c_eq, &mut event as *mut u32,  buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), timeout, libfabric_sys::FI_PEEK.into()) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok((ret as usize, Event::from_value(event)))
        }
    }

    pub fn readerr(&self, err: &mut EqErrEntry) -> Result<usize, crate::error::Error> {
        let ret = unsafe { libfabric_sys::inlined_fi_eq_readerr(self.c_eq, err.get_mut(), 0) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(ret as usize)
        }
    }

    pub fn peekerr(&self, err: &mut EqErrEntry) -> Result<usize, crate::error::Error> {
        let ret = unsafe { libfabric_sys::inlined_fi_eq_readerr(self.c_eq, err.get_mut(), libfabric_sys::FI_PEEK.into()) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(ret as usize)
        }
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

impl crate::Bind for EventQueue {
    
}

impl Drop for EventQueue {
    fn drop(&mut self) {
        debug_println!("Dropping eq");

        self.close().unwrap();
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

impl Default for EventQueueAttr {
    fn default() -> Self {
        Self::new()
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

impl Default for EqErrEntry {
    fn default() -> Self {
        Self::new()
    }
}

//================== EventQueueEntry (fi_eq_entry) ==================//

pub struct EventQueueEntry<T> {
    c_entry: libfabric_sys::fi_eq_entry,
    phantom: std::marker::PhantomData<T>,
}

impl<T> EventQueueEntry<T> {
    // const SIZE_OK: () = assert!(std::mem::size_of::<T>() == std::mem::size_of::<usize>(), 
    // "The context of an EventQueueEntry must always be of size equal to usize datatype.");

    pub fn new() -> Self {
        // let _ = Self::SIZE_OK;
        let c_entry = libfabric_sys::fi_eq_entry { 
            fid: std::ptr::null_mut(), 
            context: std::ptr::null_mut(), 
            data: 0 
        };

        Self { c_entry, phantom: std::marker::PhantomData }
    }

    pub fn fid(&mut self, fid: &impl crate::FID) -> &mut Self { //[TODO] Should this be pub(crate)?
        self.c_entry.fid = fid.fid();
        self
    }

    #[allow(dead_code)]
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
        unsafe { std::mem::transmute_copy::<T,T>(&*(context_ptr as *const T)) }
    }

    pub fn is_context_equal(&self, ctx: &crate::Context) -> bool {

        std::ptr::eq(self.c_entry.context, ctx as *const crate::Context as *const std::ffi::c_void)
    }

}

impl<T> Default for EventQueueEntry<T> {
    fn default() -> Self {
        Self::new()
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

impl Default for EventQueueCmEntry {
    fn default() -> Self {
        Self::new()
    }
}

//================== EventQueue related tests ==================//

#[cfg(test)]
mod tests {
    use crate::FID;

    #[test]
    fn eq_write_read_self() {
        let info = crate::Info::new().request().unwrap();
        let entries: Vec<crate::InfoEntry> = info.get();
        let mut eq_attr = crate::eq::EventQueueAttr::new();
        eq_attr.size(32)
            .flags(libfabric_sys::FI_WRITE.into())
            .wait_obj(crate::enums::WaitObj::NONE);
        let fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone()).unwrap();
        let eq = fab.eq_open(eq_attr).unwrap();
        for mut i in 0_usize ..5 {
            let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
            if i & 1 == 1 {
                entry.fid(&fab);
            }
            else {
                entry.fid(&eq);
            }

            entry.context(&mut i);
            let ret = eq.write(crate::enums::Event::NOTIFY, std::slice::from_ref(&entry), 0).unwrap();
            if ret != std::mem::size_of::<crate::eq::EventQueueEntry<usize>>() {
                panic!("eq.write failed {}", ret);
            }
        }
        for i in 0..10 {

            let mut entry = crate::eq::EventQueueEntry::<usize>::new();
            let (ret, event) = if i & 1 == 1 {
                eq.read(std::slice::from_mut(&mut entry)).unwrap()
            }
            else {
                eq.peek(std::slice::from_mut(&mut entry)).unwrap()

            };

            if ret != std::mem::size_of::<crate::eq::EventQueueEntry<usize>>() {
                panic!("eq.read failed {}", ret);
            }
            if !matches!(event, crate::enums::Event::NOTIFY) {
                panic!("Unexpected event {}", event.get_value());
            }

            if entry.get_context() != i /2 {
                panic!("Unexpected context {} vs {}", entry.get_context(), i/2);
            }

            if entry.get_fid() != if i & 2 == 2 {fab.fid()} else {eq.fid()} {
                panic!("Unexpected fid {:?}", entry.get_fid());
            }
        }
        let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
        let ret = eq.read( std::slice::from_mut(&mut entry));
        if let Err(ref err) = ret {
            if !matches!(err.kind, crate::error::ErrorKind::TryAgain) {
                ret.unwrap();
            }
        }

        // eq.close().unwrap();
        // fab.close().unwrap();
    }

    #[test]
    fn eq_size_verify() {
        let info = crate::Info::new().request().unwrap();
        let entries: Vec<crate::InfoEntry> = info.get();
        let mut eq_attr = crate::eq::EventQueueAttr::new();
        eq_attr.size(32)
            .flags(libfabric_sys::FI_WRITE.into())
            .wait_obj(crate::enums::WaitObj::NONE);
        let fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone()).unwrap();
        let eq = fab.eq_open(eq_attr).unwrap();

        for mut i in 0_usize .. 32 {
            let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
            entry
                .fid(&fab)
                .context(&mut i);
            let ret = eq.write(crate::enums::Event::NOTIFY, std::slice::from_mut(&mut entry), 0).unwrap();
            if ret != std::mem::size_of::<crate::eq::EventQueueEntry<usize>>() {
                panic!("eq.write write size != eventqueueentry {}", ret);
            }
        }

        // eq.close().unwrap();
        // fab.close().unwrap();
    }

    #[test]
    fn eq_write_sread_self() {
        let info = crate::Info::new().request().unwrap();
        let entries: Vec<crate::InfoEntry> = info.get();
        let mut eq_attr = crate::eq::EventQueueAttr::new();
        eq_attr.size(32)
            .flags(libfabric_sys::FI_WRITE.into()) // [TODO]
            .wait_obj(crate::enums::WaitObj::FD);
        let fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone()).unwrap();
        let eq = fab.eq_open(eq_attr).unwrap();
        for mut i in 0_usize ..5 {
            let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
            if i & 1 == 1 {
                entry.fid(&fab);
            }
            else {
                entry.fid(&eq);
            }

            entry.context(&mut i);
            let ret = eq.write(crate::enums::Event::NOTIFY, std::slice::from_ref(&entry), 0).unwrap();
            if ret != std::mem::size_of::<crate::eq::EventQueueEntry<usize>>() {
                panic!("eq.write failed {}", ret);
            }
        }
        for i in 0..10 {
            let mut entry = crate::eq::EventQueueEntry::<usize>::new();
            let (ret, event) = eq.sread(std::slice::from_mut(&mut entry), 2000 ,if (i & 1) == 1 { 0 } else { libfabric_sys::FI_PEEK.into()}).unwrap();
            if ret != std::mem::size_of::<crate::eq::EventQueueEntry<usize>>() {
                panic!("sread failed {}", ret);
            }
            if !matches!(event, crate::enums::Event::NOTIFY) {
                panic!("Unexpected event {}", event.get_value());
            }

            if entry.get_context() != i /2 {
                panic!("Unexpected context {} vs {}", entry.get_context(), i/2);
            }

            if entry.get_fid() != if i & 2 == 2 {fab.fid()} else {eq.fid()} {
                panic!("Unexpected fid {:?}", entry.get_fid());
            }
        }
        let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();

        let ret = eq.read(std::slice::from_mut(&mut entry));
        if let Err(ref err) = ret {
            if !matches!(err.kind, crate::error::ErrorKind::TryAgain) {
                ret.unwrap();
            }
        }

        // eq.close().unwrap();
        // fab.close().unwrap();
    }

    #[test]
    fn eq_readerr() {
        let info = crate::Info::new().request().unwrap();
        let entries: Vec<crate::InfoEntry> = info.get();
        let mut eq_attr = crate::eq::EventQueueAttr::new();
        eq_attr.size(32)
            .flags(libfabric_sys::FI_WRITE.into())
            .wait_obj(crate::enums::WaitObj::FD);
        let fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone()).unwrap();
        let eq = fab.eq_open(eq_attr).unwrap();
        for mut i in 0_usize ..5 {
            let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
            entry.fid(&fab);

            entry.context(&mut i);
            let ret = eq.write(crate::enums::Event::NOTIFY, std::slice::from_ref(&entry), 0).unwrap();
            if ret != std::mem::size_of::<crate::eq::EventQueueEntry<usize>>() {
                panic!("eq.write failed {}", ret);
            }
        }
        for i in 0..5 {
            let mut entry = crate::eq::EventQueueEntry::<usize>::new();
            let (ret,event) = eq.read(std::slice::from_mut(&mut entry)).unwrap();
            if ret != std::mem::size_of::<crate::eq::EventQueueEntry<usize>>() {
                panic!("Eq.read failed {}", ret);
            }
            if !matches!(event, crate::enums::Event::NOTIFY) {
                panic!("Unexpected event {:?}", event.get_value());
            }

            if entry.get_context() != i  {
                panic!("Unexpected context {} vs {}", entry.get_context(), i/2);
            }

            if entry.get_fid() != fab.fid() {
                panic!("Unexpected fid {:?}", entry.get_fid());
            }
        }
        let mut err_entry = crate::eq::EqErrEntry::new();
        let ret = eq.readerr(&mut err_entry);
        if let Err(ref err) = ret {
            if !matches!(err.kind, crate::error::ErrorKind::TryAgain) {
                ret.unwrap();
            }
        }
        // eq.close().unwrap();
        // fab.close().unwrap();
    }


    #[test]
    fn eq_open_close_sizes() {
        let info = crate::Info::new().request().unwrap();
        let entries = info.get();
        
        let fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone()).unwrap();
        for i in -1..17 {
            let size = if i == -1 { 0 } else { 1 << i };
            let mut eq_attr = crate::eq::EventQueueAttr::new();
                eq_attr.size(size);
            let _eq = fab.eq_open(eq_attr).unwrap();
        }
    }
}