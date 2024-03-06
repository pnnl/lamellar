use std::{marker::PhantomData, os::fd::BorrowedFd};

use libfabric_sys::fi_mutex_cond;

#[allow(unused_imports)]
use crate::AsFid;
use crate::{enums::WaitObjType, InfoEntry, OwnedFid};


#[allow(non_camel_case_types)]
pub enum Event<T> {
    NOTIFY(EventQueueEntry<T>),
    CONNREQ(EventQueueCmEntry),
    CONNECTED(EventQueueCmEntry),
    SHUTDOWN(EventQueueCmEntry),
    MR_COMPLETE(EventQueueEntry<T>),
    AV_COMPLETE(EventQueueEntry<T>),
    JOIN_COMPLETE(EventQueueEntry<T>),
}

impl<T> Event<T>{

    #[allow(dead_code)]
    pub(crate) fn get_value(&self) -> libfabric_sys::_bindgen_ty_18 {

        match self {
            Event::NOTIFY(_) => libfabric_sys::FI_NOTIFY,
            Event::CONNREQ(_) => libfabric_sys::FI_CONNREQ,
            Event::CONNECTED(_) => libfabric_sys::FI_CONNECTED,
            Event::SHUTDOWN(_) => libfabric_sys::FI_SHUTDOWN,
            Event::MR_COMPLETE(_) => libfabric_sys::FI_MR_COMPLETE,
            Event::AV_COMPLETE(_) => libfabric_sys::FI_AV_COMPLETE,
            Event::JOIN_COMPLETE(_) => libfabric_sys::FI_JOIN_COMPLETE,
        }
    }

    pub(crate) fn get_entry(&self) -> (*const std::ffi::c_void, usize) {
        match self {
            Event::NOTIFY(entry)| Event::MR_COMPLETE(entry) | Event::AV_COMPLETE(entry) | Event::JOIN_COMPLETE(entry) => 
                (&entry.c_entry as *const libfabric_sys::fi_eq_entry as *const std::ffi::c_void, std::mem::size_of::<libfabric_sys::fi_eq_entry>()),  
            
            Event::CONNREQ(entry) | Event::CONNECTED(entry) | Event::SHUTDOWN(entry) => 
                (&entry.c_entry as *const libfabric_sys::fi_eq_cm_entry as *const std::ffi::c_void, std::mem::size_of::<libfabric_sys::fi_eq_cm_entry>()),
        } 
    } 

    pub(crate) fn from_control_value(val: u32, entry: EventQueueEntry<usize>) -> Event<usize> {
        if val == libfabric_sys::FI_NOTIFY {
            Event::NOTIFY(entry)
        }
        else if val == libfabric_sys::FI_MR_COMPLETE {
            Event::MR_COMPLETE(entry)
        }
        else if val == libfabric_sys::FI_AV_COMPLETE {
            Event::AV_COMPLETE(entry)
        }
        else if val == libfabric_sys::FI_JOIN_COMPLETE {
            Event::JOIN_COMPLETE(entry)
        }
        else {
            panic!("Unexpected value for Event")
        }
    }

    pub(crate) fn from_connect_value(val: u32, entry: EventQueueCmEntry) -> Self {
    
        if  val == libfabric_sys::FI_CONNREQ {
            Event::CONNREQ(entry)
        }
        else if val == libfabric_sys::FI_CONNECTED {
            Event::CONNECTED(entry)
        }
        else if val == libfabric_sys::FI_SHUTDOWN {
            Event::SHUTDOWN(entry)
        }
        else {
            panic!("Unexpected value for Event")
        }
    }

}

//================== EventQueue (fi_eq) ==================//
pub struct Waitable;
pub struct NonWaitable;

pub struct EventQueueBase<T> {
    c_eq: *mut libfabric_sys::fid_eq,
    fid: OwnedFid,
    phantom: PhantomData<T>,
    wait_obj: Option<libfabric_sys::fi_wait_obj>,
}

pub type EventQueueNonWaitable = EventQueueBase<NonWaitable>;
pub type EventQueueWaitable = EventQueueBase<Waitable>;

pub enum EventQueue {
    Waitable(EventQueueWaitable),
    NonWaitable(EventQueueNonWaitable),
}

impl EventQueue {

    pub fn read(&self) -> Result<Event<usize>, crate::error::Error> {

        match self {
            EventQueue::Waitable(queue) => queue.read(), 
            EventQueue::NonWaitable(queue) => queue.read(), 
       }
    }

    pub fn peek(&self) -> Result<Event<usize>, crate::error::Error> {

        match self {
            EventQueue::Waitable(queue) => queue.peek(), 
            EventQueue::NonWaitable(queue) => queue.peek(), 
       }
    }

    pub fn write(&self, event: Event<usize>, flags: u64) -> Result<(), crate::error::Error> {

        match self {
            EventQueue::Waitable(queue) => queue.write(event, flags), 
            EventQueue::NonWaitable(queue) => queue.write(event, flags), 
       }
    }


    pub fn readerr(&self) -> Result<EventQueueErrEntry, crate::error::Error> {
       
       match self {
            EventQueue::Waitable(queue) => queue.readerr(), 
            EventQueue::NonWaitable(queue) => queue.readerr(), 
       }
    }

    pub fn peekerr(&self) -> Result<EventQueueErrEntry, crate::error::Error> {
        
        match self {
            EventQueue::Waitable(queue) => queue.peekerr(), 
            EventQueue::NonWaitable(queue) => queue.peekerr(), 
       }
    }

    pub fn strerror(&self, entry: &EventQueueErrEntry) -> &str {
        match self {
            EventQueue::Waitable(queue) => queue.strerror(entry), 
            EventQueue::NonWaitable(queue) => queue.strerror(entry), 
       }
    }
}

impl EventQueueNonWaitable {

    pub(crate) fn new(fabric: &crate::fabric::Fabric, mut attr: EventQueueAttr) -> Result<Self, crate::error::Error> {
        let mut c_eq: *mut libfabric_sys::fid_eq  = std::ptr::null_mut();
        let c_eq_ptr: *mut *mut libfabric_sys::fid_eq = &mut c_eq;

        let err = unsafe {libfabric_sys::inlined_fi_eq_open(fabric.c_fabric, attr.get_mut(), c_eq_ptr, std::ptr::null_mut())};
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_eq, fid: OwnedFid { fid: unsafe{ &mut (*c_eq).fid } }, phantom: PhantomData, wait_obj: None }
            )
        }
    }

    pub(crate) fn new_with_context<T0>(fabric: &crate::fabric::Fabric, mut attr: EventQueueAttr, ctx: &mut T0) -> Result<Self, crate::error::Error> {
        let mut c_eq: *mut libfabric_sys::fid_eq  = std::ptr::null_mut();
        let c_eq_ptr: *mut *mut libfabric_sys::fid_eq = &mut c_eq;

        let err = unsafe {libfabric_sys::inlined_fi_eq_open(fabric.c_fabric, attr.get_mut(), c_eq_ptr, ctx as *mut T0 as *mut std::ffi::c_void)};
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_eq, fid: OwnedFid { fid: unsafe{ &mut (*c_eq).fid } }, phantom: PhantomData,  wait_obj: None }
            )
        }
    }
}

impl<'a> EventQueueWaitable {

    pub(crate) fn new(fabric: &crate::fabric::Fabric, mut attr: EventQueueAttr) -> Result<Self, crate::error::Error> {
        let mut c_eq: *mut libfabric_sys::fid_eq  = std::ptr::null_mut();
        let c_eq_ptr: *mut *mut libfabric_sys::fid_eq = &mut c_eq;

        let err = unsafe {libfabric_sys::inlined_fi_eq_open(fabric.c_fabric, attr.get_mut(), c_eq_ptr, std::ptr::null_mut())};
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_eq, fid: OwnedFid { fid: unsafe{ &mut (*c_eq).fid } }, phantom: PhantomData, wait_obj:  Some(attr.c_attr.wait_obj) }
            )
        }
    }

    pub(crate) fn new_with_context<T0>(fabric: &crate::fabric::Fabric, mut attr: EventQueueAttr, ctx: &mut T0) -> Result<Self, crate::error::Error> {
        let mut c_eq: *mut libfabric_sys::fid_eq  = std::ptr::null_mut();
        let c_eq_ptr: *mut *mut libfabric_sys::fid_eq = &mut c_eq;

        let err = unsafe {libfabric_sys::inlined_fi_eq_open(fabric.c_fabric, attr.get_mut(), c_eq_ptr, ctx as *mut T0 as *mut std::ffi::c_void)};
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_eq, fid: OwnedFid { fid: unsafe{ &mut (*c_eq).fid } }, phantom: PhantomData, wait_obj:  Some(attr.c_attr.wait_obj) }
            )
        }
    }

    pub fn sread(&self, timeout: i32, flags: u64) -> Result<Event<usize>, crate::error::Error> { 
        let mut event = 0;
        let mut buffer: Vec<u8> = vec![0; std::mem::size_of::<libfabric_sys::fi_eq_err_entry>()];
        let ret = unsafe { libfabric_sys::inlined_fi_eq_sread(self.c_eq, &mut event as *mut u32,  buffer.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), timeout, flags) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(read_eq_entry(ret, &buffer, &event))
        }
    }

    pub fn speek(&self, timeout: i32) -> Result<Event<usize>, crate::error::Error> { 
        let mut event = 0;
        let mut buffer: Vec<u8> = vec![0; std::mem::size_of::<libfabric_sys::fi_eq_err_entry>()];

        let ret = unsafe { libfabric_sys::inlined_fi_eq_sread(self.c_eq, &mut event as *mut u32,  buffer.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), timeout, libfabric_sys::FI_PEEK.into()) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(read_eq_entry(ret, &buffer, &event))
        }
    }

    pub fn wait_object(&self) -> Result<WaitObjType<'a>, crate::error::Error> {

        if let Some(wait) = self.wait_obj {
            if wait == libfabric_sys::fi_wait_obj_FI_WAIT_FD {
                let mut fd: i32 = 0;
                let err = unsafe { libfabric_sys::inlined_fi_control(self.as_fid(), libfabric_sys::FI_GETWAIT as i32, &mut fd as *mut i32 as *mut std::ffi::c_void) };
                if err < 0 {
                    Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
                }
                else {
                    Ok(WaitObjType::Fd(unsafe{ BorrowedFd::borrow_raw(fd) }))
                }
            }
            else if wait == libfabric_sys::fi_wait_obj_FI_WAIT_MUTEX_COND {
                let mut mutex_cond = fi_mutex_cond{
                    mutex: std::ptr::null_mut(),
                    cond: std::ptr::null_mut(),
                };

                let err = unsafe { libfabric_sys::inlined_fi_control(self.as_fid(), libfabric_sys::FI_GETWAIT as i32, &mut mutex_cond as *mut fi_mutex_cond as *mut std::ffi::c_void) };
                if err < 0 {
                    Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
                }
                else {
                    Ok(WaitObjType::MutexCond(mutex_cond))
                }
            }
            else if wait == libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC{
                Ok(WaitObjType::Unspec)
            }
            else {
                panic!("Could not retrieve wait object")
            }
        }
        else { 
            panic!("Should not be reachable! Could not retrieve wait object")
        }
    }
}

fn read_eq_entry(bytes_read: isize, buffer: &Vec<u8>, event: &u32) -> Event<usize> {
    if event == &libfabric_sys::FI_CONNREQ || event == &libfabric_sys::FI_CONNECTED || event == &libfabric_sys::FI_SHUTDOWN {
        debug_assert_eq!(bytes_read as usize, std::mem::size_of::<libfabric_sys::fi_eq_cm_entry>());
        // let res = unsafe { std::slice::from_raw_parts(buffer.as_mut_ptr()  as *mut libfabric_sys::fi_eq_cm_entry, 1) };
        Event::from_connect_value(*event, EventQueueCmEntry {
            c_entry: unsafe { std::ptr::read(buffer.as_ptr() as *const libfabric_sys::fi_eq_cm_entry) }
        })          
    }
    else {
        debug_assert_eq!(bytes_read as usize, std::mem::size_of::<libfabric_sys::fi_eq_entry>());

        Event::<usize>::from_control_value(*event,
            EventQueueEntry::<usize> {
                c_entry: unsafe { std::ptr::read(buffer.as_ptr() as *const libfabric_sys::fi_eq_entry) },
                phantom: PhantomData,
            }
        )
    }    
}

impl<T> EventQueueBase<T> {

    pub fn read(&self) -> Result<Event<usize>, crate::error::Error>{
        let mut event = 0 ;
        let mut buffer: Vec<u8> = vec![0; std::mem::size_of::<libfabric_sys::fi_eq_err_entry>()];
        let ret = unsafe { libfabric_sys::inlined_fi_eq_read(self.c_eq, &mut event as *mut u32, buffer.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), 0) };
        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(read_eq_entry(ret, &buffer, &event))
        }
    }

    pub fn peek(&self) -> Result<Event<usize>, crate::error::Error>{
        let mut event = 0 ;
        let mut buffer: Vec<u8> = vec![0; std::mem::size_of::<libfabric_sys::fi_eq_err_entry>()];
        let ret = unsafe { libfabric_sys::inlined_fi_eq_read(self.c_eq, &mut event as *mut u32, buffer.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), libfabric_sys::FI_PEEK.into()) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(read_eq_entry(ret, &buffer, &event))
        }
    }

    pub fn write(&self, event: Event<usize>, flags: u64) -> Result<(), crate::error::Error>{
        // println!("{:?}", buf);
        let event_val = event.get_value();
        let (event_entry, event_entry_size) = event.get_entry();

        let ret = unsafe { libfabric_sys::inlined_fi_eq_write(self.c_eq, event_val, event_entry, event_entry_size, flags) };
        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            debug_assert_eq!(ret as usize, event_entry_size);
            Ok(())
        }
    }


    pub fn readerr(&self) -> Result<EventQueueErrEntry, crate::error::Error> {
        let mut err_q = EventQueueErrEntry::new();
        let ret = unsafe { libfabric_sys::inlined_fi_eq_readerr(self.c_eq, err_q.get_mut(), 0) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(err_q)
        }
    }

    pub fn peekerr(&self) -> Result<EventQueueErrEntry, crate::error::Error> {
        let mut err_q = EventQueueErrEntry::new();
        let ret = unsafe { libfabric_sys::inlined_fi_eq_readerr(self.c_eq, err_q.get_mut(), libfabric_sys::FI_PEEK.into()) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(err_q)
        }
    }

    pub fn strerror(&self, entry: &EventQueueErrEntry) -> &str {
        let ret = unsafe { libfabric_sys::inlined_fi_eq_strerror(self.c_eq, entry.c_err.prov_errno, entry.c_err.err_data, std::ptr::null_mut(), 0) };
    
            unsafe{ std::ffi::CStr::from_ptr(ret).to_str().unwrap() }
    }

}

impl crate::AsFid for EventQueueWaitable {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
       self.fid.as_fid()
    }
}

impl crate::AsFid for EventQueueNonWaitable {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
       self.fid.as_fid()
    }
}

impl crate::AsFid for EventQueue {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
        match self {
            EventQueue::Waitable(queue) => queue.as_fid(), 
            EventQueue::NonWaitable(queue) => queue.as_fid(), 
        }
    }
}

impl crate::Bind for EventQueue {}
impl crate::Bind for EventQueueWaitable {}
impl crate::Bind for EventQueueNonWaitable {}

//================== EventQueue Attribute(fi_eq_attr) ==================//

pub struct EventQueueBuilder<'a, T> {
    eq_attr: EventQueueAttr,
    fabric: &'a crate::fabric::Fabric,
    ctx: Option<&'a mut T>,
}

impl<'a> EventQueueBuilder<'a, ()> {
    pub fn new(fabric: &'a crate::fabric::Fabric) -> EventQueueBuilder<()> {
        EventQueueBuilder::<()> {
            eq_attr: EventQueueAttr::new(),
            fabric,
            ctx: None,
        }
    }
}

impl <'a, T> EventQueueBuilder<'a, T> {
    
    pub fn size(mut self, size: usize) -> Self {
        self.eq_attr.size(size);
        self
    }

    pub fn flags(mut self, flags: u64) -> Self {
        self.eq_attr.flags(flags);
        self
    }
    
    pub fn wait_obj(mut self, wait_obj: crate::enums::WaitObj) -> Self {
        self.eq_attr.wait_obj(wait_obj);
        self
    }

    pub fn signaling_vector(mut self, signaling_vector: i32) -> Self {
        self.eq_attr.signaling_vector(signaling_vector);
        self
    }

    pub fn context(self, ctx: &'a mut T) -> EventQueueBuilder<'a, T> {
        EventQueueBuilder {
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: Some(ctx),
        }
    }

    pub fn build(self) ->  Result<EventQueue, crate::error::Error> {
        if self.eq_attr.c_attr.wait_obj == crate::enums::WaitObj::NONE.get_value() {

            let cq = if let Some(ctx) = self.ctx {
                EventQueueNonWaitable::new_with_context(self.fabric, self.eq_attr, ctx)?
            }
            else {
                EventQueueNonWaitable::new(self.fabric, self.eq_attr)?   
            };

            Ok(EventQueue::NonWaitable(cq))
        }
        else {
            let cq = if let Some(ctx) = self.ctx {
                EventQueueWaitable::new_with_context(self.fabric, self.eq_attr, ctx)?
            }
            else {
                EventQueueWaitable::new(self.fabric, self.eq_attr)?   
            };

            Ok(EventQueue::Waitable(cq))
        }
    }
}


pub(crate) struct EventQueueAttr {
    c_attr: libfabric_sys::fi_eq_attr,
}

impl EventQueueAttr {

    pub(crate) fn new() -> Self {
        let c_attr = libfabric_sys::fi_eq_attr{ 
            size: 0, 
            flags: 0, 
            wait_obj: libfabric_sys::fi_wait_obj_FI_WAIT_UNSPEC,
            signaling_vector: 0, 
            wait_set: std::ptr::null_mut()
        };

        Self {c_attr}
    }

    pub(crate) fn size(&mut self, size: usize) -> &mut Self {
        self.c_attr.size = size;
        self
    }

    pub(crate) fn flags(&mut self, flags: u64) -> &mut Self {
        self.c_attr.flags = flags;
        self
    }
    
    pub(crate) fn wait_obj(&mut self, wait_obj: crate::enums::WaitObj) -> &mut Self {
        
        if let crate::enums::WaitObj::SET(wait_set) = wait_obj {
            self.c_attr.wait_set = wait_set.c_wait;
        }
        self.c_attr.wait_obj = wait_obj.get_value();
        self
    }

    pub(crate) fn signaling_vector(&mut self, signaling_vector: i32) -> &mut Self {
        self.c_attr.signaling_vector = signaling_vector;
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
#[repr(C)]
pub struct EventQueueErrEntry {
    pub(crate) c_err: libfabric_sys::fi_eq_err_entry,
}

impl EventQueueErrEntry {
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

impl Default for EventQueueErrEntry {
    fn default() -> Self {
        Self::new()
    }
}

//================== EventQueueEntry (fi_eq_entry) ==================//

#[repr(C)]
#[derive(Clone)]
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

    pub fn fid(&mut self, fid: &impl crate::AsFid) -> &mut Self { //[TODO] Should this be pub(crate)?
        self.c_entry.fid = fid.as_fid();
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
#[repr(C)]
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
    use crate::AsFid;

    use super::{Event, EventQueue, EventQueueBuilder};

    #[test]
    fn eq_write_read_self() {
        let info = crate::Info::new().request().unwrap();
        let entries: Vec<crate::InfoEntry> = info.get();
        let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
        let eq = EventQueueBuilder::new(&fab)
            .size(32)
            .flags(libfabric_sys::FI_WRITE.into())
            .wait_obj(crate::enums::WaitObj::NONE)
            .build().unwrap();

        for mut i in 0_usize ..5 {
            let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
            if i & 1 == 1 {
                entry.fid(&fab);
            }
            else {
                entry.fid(&eq);
            }

            entry.context(&mut i);
            eq.write(Event::NOTIFY(entry), 0).unwrap();
        }
        for i in 0..10 {

            let ret = if i & 1 == 1 {
                eq.read().unwrap()
            }
            else {
                eq.peek().unwrap()
            };

            if let crate::eq::Event::NOTIFY(entry) = ret {
                
                if entry.get_context() != i /2 {
                    panic!("Unexpected context {} vs {}", entry.get_context(), i/2);
                }
                
                if entry.get_fid() != if i & 2 == 2 {fab.as_fid()} else {eq.as_fid()} {
                    panic!("Unexpected fid {:?}", entry.get_fid());
                }
            }
            else {
                panic!("Unexpected EventType");
            } 
        }

        let ret = eq.read();
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
        let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
        let eq = EventQueueBuilder::new(&fab)
            .size(32)
            .flags(libfabric_sys::FI_WRITE.into())
            .wait_obj(crate::enums::WaitObj::NONE)
            .build().unwrap();

        for mut i in 0_usize .. 32 {
            let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
            entry
                .fid(&fab)
                .context(&mut i);
            eq.write(Event::NOTIFY(entry), 0).unwrap();
        }
    }

    #[test]
    fn eq_write_sread_self() {
        let info = crate::Info::new().request().unwrap();
        let entries: Vec<crate::InfoEntry> = info.get();
        let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
        let eq = EventQueueBuilder::new(&fab)
            .size(32)
            .flags(libfabric_sys::FI_WRITE.into())
            .wait_obj(crate::enums::WaitObj::FD)
            .build().unwrap();
        for mut i in 0_usize ..5 {
            let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
            if i & 1 == 1 {
                entry.fid(&fab);
            }
            else {
                entry.fid(&eq);
            }

            entry.context(&mut i);
            eq.write(Event::NOTIFY(entry), 0).unwrap();
        }
        if let EventQueue::Waitable(eq) = eq {

            for i in 0..10 {
                // let mut entry = crate::eq::EventQueueEntry::<usize>::new();
                let event = if (i & 1) == 1 { eq.sread(2000, 0) } else { eq.speek(2000) }.unwrap();
                // if ret != std::mem::size_of::<crate::eq::EventQueueEntry<usize>>() {
                //     panic!("sread failed {}", ret);
                // }
                if let crate::eq::Event::NOTIFY(entry) = event {

                    if entry.get_context() != i /2 {
                        panic!("Unexpected context {} vs {}", entry.get_context(), i/2);
                    }
                    
                    if entry.get_fid() != if i & 2 == 2 {fab.as_fid()} else {eq.as_fid()} {
                        panic!("Unexpected fid {:?}", entry.get_fid());
                    }
                }
                else {
                    panic!("Unexpected EventType");
                }
            }
            
            let ret = eq.read();
            if let Err(ref err) = ret {
                if !matches!(err.kind, crate::error::ErrorKind::TryAgain) {
                    ret.unwrap();
                }
            }
        }

        // eq.close().unwrap();
        // fab.close().unwrap();
    }

    #[test]
    fn eq_readerr() {
        let info = crate::Info::new().request().unwrap();
        let entries: Vec<crate::InfoEntry> = info.get();
        let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
        let eq = EventQueueBuilder::new(&fab)
            .size(32)
            .flags(libfabric_sys::FI_WRITE.into())
            .wait_obj(crate::enums::WaitObj::FD)
            .build().unwrap();
        for mut i in 0_usize ..5 {
            let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
            entry.fid(&fab);

            entry.context(&mut i);
            eq.write(Event::NOTIFY(entry), 0).unwrap();
        }
        for i in 0..5 {
            let event = eq.read().unwrap();

            if let Event::NOTIFY(entry) = event {

                if entry.get_context() != i  {
                    panic!("Unexpected context {} vs {}", entry.get_context(), i/2);
                }
                
                if entry.get_fid() != fab.as_fid() {
                    panic!("Unexpected fid {:?}", entry.get_fid());
                }
            }
            else {
                panic!("Unexpected EventQueueEntryFormat");
            }
        }
        let ret = eq.readerr();
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
        
        let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
        for i in -1..17 {
            let size = if i == -1 { 0 } else { 1 << i };
            let eq = EventQueueBuilder::new(&fab)
                .wait_obj(crate::enums::WaitObj::FD)
                .size(size)
                .build().unwrap();
            if let EventQueue::Waitable(eq) = eq {
                eq.wait_object().unwrap();
            }; 
        }
    }
}