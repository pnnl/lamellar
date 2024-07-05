use std::{marker::PhantomData, os::fd::{AsFd, BorrowedFd, AsRawFd, RawFd}, rc::Rc, cell::RefCell};

use libfabric_sys::{fi_mutex_cond, FI_AFFINITY, FI_WRITE};
#[allow(unused_imports)]
use crate::fid::AsFid;
use crate::{fid::{AsRawFid, AsRawTypedFid, OwnedEqFid, AsTypedFid, EqRawFid}};
use crate::{enums::WaitObjType, eqoptions::{self, EqConfig,  EqWritable, Off, On, Options, WaitNoRetrieve, WaitNone, WaitRetrieve}, FdRetrievable, WaitRetrievable, fabric::FabricImpl, infocapsoptions::Caps, info::{InfoHints, InfoEntry}, fid::{self, RawFid}};

// impl<T: EqConfig> Drop for EventQueue<T> {
//     fn drop(&mut self) {
//        println!("Dropping EventQueue\n");
//     }
// }

// enum NotifyEventFid {
//     Fabric(Fabric),
//     Domain(Domain),
//     Mr(MemoryRegion),
// }


// pub struct 

pub enum Event<T> {
    // Notify(EventQueueEntry<T, NotifyEventFid>),
    ConnReq(EventQueueCmEntry),
    Connected(EventQueueCmEntry),
    Shutdown(EventQueueCmEntry),
    MrComplete(EventQueueEntry<T, RawFid>),
    AVComplete(EventQueueEntry<T, RawFid>),
    JoinComplete(EventQueueEntry<T, RawFid>),
}

impl<T> Event<T>{

    #[allow(dead_code)]
    pub(crate) fn get_value(&self) -> libfabric_sys::_bindgen_ty_18 {

        match self {
            // Event::Notify(_) => libfabric_sys::FI_NOTIFY,
            Event::ConnReq(_) => libfabric_sys::FI_CONNREQ,
            Event::Connected(_) => libfabric_sys::FI_CONNECTED,
            Event::Shutdown(_) => libfabric_sys::FI_SHUTDOWN,
            Event::MrComplete(_) => libfabric_sys::FI_MR_COMPLETE,
            Event::AVComplete(_) => libfabric_sys::FI_AV_COMPLETE,
            Event::JoinComplete(_) => libfabric_sys::FI_JOIN_COMPLETE,
        }
    }

    pub(crate) fn get_entry(&self) -> (*const std::ffi::c_void, usize) {
        match self {
            // Event::Notify(entry)| 
            Event::MrComplete(entry) => {((&entry.c_entry as *const libfabric_sys::fi_eq_entry).cast(), std::mem::size_of::<libfabric_sys::fi_eq_entry>())},
            Event::AVComplete(entry) => {((&entry.c_entry as *const libfabric_sys::fi_eq_entry).cast(), std::mem::size_of::<libfabric_sys::fi_eq_entry>())},
            Event::JoinComplete(entry) => {((&entry.c_entry as *const libfabric_sys::fi_eq_entry).cast(), std::mem::size_of::<libfabric_sys::fi_eq_entry>())},
                // ( (&entry.c_entry as *const libfabric_sys::fi_eq_entry).cast(), std::mem::size_of::<libfabric_sys::fi_eq_entry>()),  
            
            Event::ConnReq(entry) | Event::Connected(entry) | Event::Shutdown(entry) => 
                ( (&entry.c_entry as *const libfabric_sys::fi_eq_cm_entry).cast(), std::mem::size_of::<libfabric_sys::fi_eq_cm_entry>()),
        } 
    } 

    // pub(crate) fn from_control_value(event: u32, entry: EventQueueEntry<usize>) -> Event<usize> {
    //     if event == libfabric_sys::FI_NOTIFY {
    //         Event::Notify(entry)
    //     }
    //     else if event == libfabric_sys::FI_MR_COMPLETE {
    //         Event::MrComplete(entry)
    //     }
    //     else if event == libfabric_sys::FI_AV_COMPLETE {
    //         Event::AVComplete(entry)
    //     }
    //     else if event == libfabric_sys::FI_JOIN_COMPLETE {
    //         Event::JoinComplete(entry)
    //     }
    //     else {
    //         panic!("Unexpected value for Event")
    //     }
    // }

    pub(crate) fn from_connect_value(val: u32, entry: EventQueueCmEntry) -> Self {
    
        if  val == libfabric_sys::FI_CONNREQ {
            Event::ConnReq(entry)
        }
        else if val == libfabric_sys::FI_CONNECTED {
            Event::Connected(entry)
        }
        else if val == libfabric_sys::FI_SHUTDOWN {
            Event::Shutdown(entry)
        }
        else {
            panic!("Unexpected value for Event")
        }
    }

}


//================== EventQueue (fi_eq) ==================//
pub struct EventQueueImpl {
    pub(crate) c_eq: OwnedEqFid,
    wait_obj: Option<libfabric_sys::fi_wait_obj>,
    // pub(crate) mrs: RefCell<std::collections::HashMap<RawFid, Weak<MemoryRegionImpl>>>,   // We neep maps Fid -> MemoryRegionImpl/AddressVectorImpl/MulticastGroupCollectiveImpl, however, we don't want to extend 
                                                                            // the lifetime of any of these objects just because of the maps.
                                                                            // Moreover, these objects will keep references to the EQ to keep it from dropping while
                                                                            // they are still bound, thus, we would have cyclic references that wouldn't let any 
                                                                            // of the two sides drop. 
    // pub(crate) avs: RefCell<std::collections::HashMap<RawFid, Weak<AddressVectorImpl>>>,
    // pub(crate) mcs: RefCell<std::collections::HashMap<RawFid, Weak<MulticastGroupCollectiveImpl>>>,

    event_buffer: RefCell<Vec<u8>>,
    pub(crate) _fabric_rc: Rc<FabricImpl>,
}


pub trait EventQueueImplT {
    fn read(&self) -> Result<Event<usize>, crate::error::Error>;

    fn read_in(&self, buff: &mut [u8], event: &mut u32) -> Result<usize, crate::error::Error> ;


    fn peek(&self) -> Result<Event<usize>, crate::error::Error>;
        
    fn readerr(&self) -> Result<EventError, crate::error::Error>;

    fn peekerr(&self) -> Result<EventError, crate::error::Error>;

    fn strerror(&self, entry: &EventError) -> &str;
}

// trait WaitableEventQueueImpl : EventQueueImplT {
//     fn sread(&self, timeout: i32, flags: u64) -> Result<Event<usize>, crate::error::Error>;
//     fn speek(&self, timeout: i32) -> Result<Event<usize>, crate::error::Error>;
// }

// pub struct WaitableEventQueueImpl {

// }

// pub struct EventQueueImplStrict<const WRITE: bool, const WAIT: bool, const RETRIEVE: bool, const FD : bool> {
    
// }

impl EventQueueImplT for EventQueueImpl {
    
    fn read(&self) -> Result<Event<usize>, crate::error::Error>{ //[TODO] Use an "owned" buffer instead of allocating a new one for each attempt and copy it out
        let mut event = 0 ;
        let mut buffer = self.event_buffer.borrow_mut();
        let ret = unsafe { libfabric_sys::inlined_fi_eq_read(self.as_raw_typed_fid(), &mut event, buffer.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), 0) };
        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(self.read_eq_entry(ret as usize, &buffer, &event))
        }
    }

    fn read_in(&self, buff: &mut [u8], event: &mut u32) -> Result<usize, crate::error::Error> {
        // let mut event = 0 ;
        let ret = unsafe { libfabric_sys::inlined_fi_eq_read(self.as_raw_typed_fid(), event, buff.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), 0) };
        if ret < 0 {
            return Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) );
        }
        Ok(ret as usize)
    } 

    fn peek(&self) -> Result<Event<usize>, crate::error::Error>{
        let mut event = 0 ;
        let mut buffer = self.event_buffer.borrow_mut();
        let ret = unsafe { libfabric_sys::inlined_fi_eq_read(self.as_raw_typed_fid(), &mut event, buffer.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), libfabric_sys::FI_PEEK.into()) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(self.read_eq_entry(ret as usize, &buffer, &event))
        }
    }

    fn readerr(&self) -> Result<EventError, crate::error::Error> {

        let mut buffer = self.event_buffer.borrow_mut();

        let ret = unsafe { libfabric_sys::inlined_fi_eq_readerr(self.as_raw_typed_fid(), buffer.as_mut_ptr().cast(), 0) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            let mut err_event = EventError::new();
            err_event.c_err = unsafe { std::ptr::read(buffer.as_ptr().cast()) };
            Ok(err_event)
        }
    }

    fn peekerr(&self) -> Result<EventError, crate::error::Error> {
        
        let mut buffer = self.event_buffer.borrow_mut();
        let ret = unsafe { libfabric_sys::inlined_fi_eq_readerr(self.as_raw_typed_fid(), buffer.as_mut_ptr().cast(), libfabric_sys::FI_PEEK.into()) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            let mut err_event = EventError::new();
            err_event.c_err = unsafe { std::ptr::read(buffer.as_ptr().cast()) };
            Ok(err_event)
        }
    }
    
    fn strerror(&self, entry: &EventError) -> &str {
        let ret = unsafe { libfabric_sys::inlined_fi_eq_strerror(self.as_raw_typed_fid(), -entry.c_err.prov_errno, entry.c_err.err_data, std::ptr::null_mut(), 0) };
    
        unsafe{ std::ffi::CStr::from_ptr(ret).to_str().unwrap() }
    }
}


pub type EventQueue<T> = EventQueueBase<T, EventQueueImpl>;

pub struct EventQueueBase<T: EqConfig, EQ> {
    pub(crate) inner: Rc<EQ>,
    pub(crate) phantom: PhantomData<T>,
}

// pub(crate) trait BindEqImpl<EQ, CQ> {
//     fn bind_mr(&self, mr: &Rc<MemoryRegionImplBase<EQ>>);

//     fn bind_av(&self, av: &Rc<AddressVectorImplBase<EQ>>);

//     fn bind_mc(&self, mc: &Rc<MulticastGroupCollectiveImplBase<EQ, CQ>>);
// }

// impl BindEqImpl<EventQueueImpl, CompletionQueueImpl> for EventQueueImpl {
//     fn bind_mr(&self, mr: &Rc<MemoryRegionImplBase<EventQueueImpl>>) {
//         self.bind_mr(mr);
//     }

//     fn bind_av(&self, av: &Rc<AddressVectorImplBase<EventQueueImpl>>) {
//         self.bind_av(av);
//     }

//     fn bind_mc(&self, mc: &Rc<MulticastGroupCollectiveImplBase<EventQueueImpl, CompletionQueueImpl>>) {
//         self.bind_mc(mc);
//     }
// }

impl<'a> EventQueueImpl {

    pub(crate) fn new<T0>(fabric: &Rc<crate::fabric::FabricImpl>, mut attr: EventQueueAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        let mut c_eq: *mut libfabric_sys::fid_eq  = std::ptr::null_mut();
        let c_eq_ptr: *mut *mut libfabric_sys::fid_eq = &mut c_eq;

        let err = 
            if let Some(ctx) = context {
                unsafe {libfabric_sys::inlined_fi_eq_open(fabric.as_raw_typed_fid(), attr.get_mut(), c_eq_ptr, (ctx as *mut T0).cast())}
            }
            else {
                unsafe {libfabric_sys::inlined_fi_eq_open(fabric.as_raw_typed_fid(), attr.get_mut(), c_eq_ptr, std::ptr::null_mut())}
            };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    c_eq: OwnedEqFid::from(c_eq), 
                    wait_obj:  Some(attr.c_attr.wait_obj),
                    // mrs: RefCell::new(HashMap::new()),
                    // avs: RefCell::new(HashMap::new()),
                    // mcs: RefCell::new(HashMap::new()),

                    event_buffer: RefCell::new(vec![0; std::mem::size_of::<libfabric_sys::fi_eq_err_entry>()]),
                    _fabric_rc: fabric.clone(),
                })
        }
    }

    // pub(crate) fn bind_mr(&self, mr: &Rc<MemoryRegionImpl>) {
    //     self.mrs.borrow_mut().insert(mr.as_raw_fid(), Rc::downgrade(mr));
    // }

    // pub(crate) fn bind_av(&self, av: &Rc<AddressVectorImpl>) {
    //     self.avs.borrow_mut().insert(av.as_raw_fid(), Rc::downgrade(av));
    // }

    // pub(crate) fn bind_mc(&self, mc: &Rc<MulticastGroupCollectiveImpl>) {
    //     self.mcs.borrow_mut().insert(mc.as_raw_fid(), Rc::downgrade(mc));
    // }

    // pub(crate) fn read(&self) -> Result<Event<usize>, crate::error::Error>{ //[TODO] Use an "owned" buffer instead of allocating a new one for each attempt and copy it out
    //     let mut event = 0 ;
    //     let mut buffer = self.event_buffer.borrow_mut();
    //     let ret = unsafe { libfabric_sys::inlined_fi_eq_read(self.as_raw_typed_fid(), &mut event, buffer.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), 0) };
    //     if ret < 0 {
    //         Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
    //     }
    //     else {
    //         Ok(self.read_eq_entry(ret, &buffer, &event))
    //     }
    // }

    // pub(crate) fn peek(&self) -> Result<Event<usize>, crate::error::Error>{
    //     let mut event = 0 ;
    //     let mut buffer = self.event_buffer.borrow_mut();
    //     let ret = unsafe { libfabric_sys::inlined_fi_eq_read(self.as_raw_typed_fid(), &mut event, buffer.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), libfabric_sys::FI_PEEK.into()) };

    //     if ret < 0 {
    //         Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
    //     }
    //     else {
    //         Ok(self.read_eq_entry(ret, &buffer, &event))
    //     }
    // }

    // pub(crate) fn readerr(&self) -> Result<EventError, crate::error::Error> {

    //     let mut buffer = self.event_buffer.borrow_mut();

    //     let ret = unsafe { libfabric_sys::inlined_fi_eq_readerr(self.as_raw_typed_fid(), buffer.as_mut_ptr().cast(), 0) };

    //     if ret < 0 {
    //         Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
    //     }
    //     else {
    //         let mut err_event = EventError::new();
    //         err_event.c_err = unsafe { std::ptr::read(buffer.as_ptr().cast()) };
    //         Ok(err_event)
    //     }
    // }

    // pub(crate) fn peekerr(&self) -> Result<EventError, crate::error::Error> {
        
    //     let mut buffer = self.event_buffer.borrow_mut();
    //     let ret = unsafe { libfabric_sys::inlined_fi_eq_readerr(self.as_raw_typed_fid(), buffer.as_mut_ptr().cast(), libfabric_sys::FI_PEEK.into()) };

    //     if ret < 0 {
    //         Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
    //     }
    //     else {
    //         let mut err_event = EventError::new();
    //         err_event.c_err = unsafe { std::ptr::read(buffer.as_ptr().cast()) };
    //         Ok(err_event)
    //     }
    // }

    // pub(crate) fn strerror(&self, entry: &EventError) -> &str {
    //     let ret = unsafe { libfabric_sys::inlined_fi_eq_strerror(self.as_raw_typed_fid(), -entry.c_err.prov_errno, entry.c_err.err_data, std::ptr::null_mut(), 0) };
    
    //         unsafe{ std::ffi::CStr::from_ptr(ret).to_str().unwrap() }
    // }

    pub(crate) fn write(&self, event: Event<usize>, flags: u64) -> Result<(), crate::error::Error>{
        let event_val = event.get_value();
        let (event_entry, event_entry_size) = event.get_entry();

        let ret = unsafe { libfabric_sys::inlined_fi_eq_write(self.as_raw_typed_fid(), event_val, event_entry, event_entry_size, flags) };
        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            debug_assert_eq!(ret as usize, event_entry_size);
            Ok(())
        }
    }

    pub(crate) fn sread(&self, timeout: i32, flags: u64) -> Result<Event<usize>, crate::error::Error> { 
        let mut event = 0;
        let mut buffer = self.event_buffer.borrow_mut();

        let ret = unsafe { libfabric_sys::inlined_fi_eq_sread(self.as_raw_typed_fid(), &mut event,  buffer.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), timeout, flags) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(self.read_eq_entry(ret as usize, &buffer, &event))
        }
    }

    pub(crate) fn speek(&self, timeout: i32) -> Result<Event<usize>, crate::error::Error> { 
        let mut event = 0;
        let mut buffer = self.event_buffer.borrow_mut();


        let ret = unsafe { libfabric_sys::inlined_fi_eq_sread(self.as_raw_typed_fid(), &mut event,  buffer.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), timeout, libfabric_sys::FI_PEEK.into()) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(self.read_eq_entry(ret as usize, &buffer, &event))
        }
    }

    pub(crate) fn read_eq_entry(&self, bytes_read: usize, buffer: &[u8], event: &u32) -> Event<usize> {
        if event == &libfabric_sys::FI_CONNREQ || event == &libfabric_sys::FI_CONNECTED || event == &libfabric_sys::FI_SHUTDOWN {
            debug_assert_eq!(bytes_read, std::mem::size_of::<libfabric_sys::fi_eq_cm_entry>());
            Event::from_connect_value(*event, EventQueueCmEntry {
                c_entry: unsafe { std::ptr::read(buffer.as_ptr().cast()) }
            })          
        }
        else {
            debug_assert_eq!(bytes_read, std::mem::size_of::<libfabric_sys::fi_eq_entry>());
            let c_entry: libfabric_sys::fi_eq_entry = unsafe { std::ptr::read(buffer.as_ptr().cast()) };
            
            if event == &libfabric_sys::FI_NOTIFY {
                panic!("Unexpected event");
            //     self.
            //     Event::Notify(entry)
            }
            
            if event == &libfabric_sys::FI_MR_COMPLETE {
                // let event_fid = self.mrs.borrow().get(&c_entry.fid).unwrap().clone();
                Event::MrComplete(
                    EventQueueEntry::<usize, RawFid> {
                        c_entry,
                        event_fid:c_entry.fid, //MemoryRegion::from_impl(&event_fid.upgrade().unwrap()),
                        phantom: PhantomData,
                })
            }
            else if event == &libfabric_sys::FI_AV_COMPLETE {
                // let event_fid = self.avs.borrow().get(&c_entry.fid).unwrap().clone();
                Event::AVComplete(
                    EventQueueEntry::<usize, RawFid> {
                        c_entry,
                        event_fid:c_entry.fid, // AddressVector::from_impl(&event_fid.upgrade().unwrap()),
                        phantom: PhantomData,
                })
            }
            else if event == &libfabric_sys::FI_JOIN_COMPLETE {
                // let event_fid = self.mcs.borrow().get(&c_entry.fid).unwrap().clone();
                Event::JoinComplete(
                    EventQueueEntry::<usize, RawFid> {
                        c_entry,
                        event_fid: c_entry.fid,
                        phantom: PhantomData,
                })
            }
            else {
                panic!("Unexpected value for Event")
            }
        }    
    }

    pub(crate) fn wait_object(&self) -> Result<WaitObjType<'a>, crate::error::Error> {

        if let Some(wait) = self.wait_obj {
            if wait == libfabric_sys::fi_wait_obj_FI_WAIT_FD {
                let mut fd: i32 = 0;
                let err = unsafe { libfabric_sys::inlined_fi_control(self.as_raw_fid(), libfabric_sys::FI_GETWAIT as i32, (&mut fd as *mut i32).cast()) };
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

                let err = unsafe { libfabric_sys::inlined_fi_control(self.as_raw_fid(), libfabric_sys::FI_GETWAIT as i32, (&mut mutex_cond as *mut fi_mutex_cond).cast()) };
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

impl<T: EqConfig> EventQueue<T> {

    pub(crate) fn new<T0>(_options: T,fabric: &crate::fabric::Fabric, attr: EventQueueAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {

        Ok(
            Self {
                inner: Rc::new(
                    EventQueueImpl::new(&fabric.inner, attr, context)?
                ),
                phantom: PhantomData, 
            })
    }

    pub fn read(&self) -> Result<Event<usize>, crate::error::Error>{
        self.inner.read()
    }

    pub fn peek(&self) -> Result<Event<usize>, crate::error::Error>{
        self.inner.peek()
    }
        
    pub fn readerr(&self) -> Result<EventError, crate::error::Error> {
        self.inner.readerr()
    }

    pub fn peekerr(&self) -> Result<EventError, crate::error::Error> {
        self.inner.peekerr()
    }

    pub fn strerror(&self, entry: &EventError) -> &str {
        self.inner.strerror(entry)
    }
    
}



impl<T: EqWritable + EqConfig> EventQueue<T> {

    pub fn write(&self, event: Event<usize>) -> Result<(), crate::error::Error>{
        self.inner.write(event, 0)
    }
}

impl<T: crate::Waitable + EqConfig> EventQueue<T> {

    pub fn sread(&self, timeout: i32) -> Result<Event<usize>, crate::error::Error> { 
        self.inner.sread(timeout, 0)
    }

    pub fn speek(&self, timeout: i32) -> Result<Event<usize>, crate::error::Error> { 
        self.inner.speek(timeout)
    }
}


impl<'a, T: crate::WaitRetrievable + EqConfig> EventQueue<T> {

    pub fn wait_object(&self) -> Result<WaitObjType<'a>, crate::error::Error> {
        self.inner.wait_object()
    }
}

impl<T: EqConfig, EQ: AsFid> AsFid for EventQueueBase<T, EQ> {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
       self.inner.as_fid()
    }
}

impl<T: EqConfig, EQ: AsRawFid> AsRawFid for EventQueueBase<T, EQ> {
    fn as_raw_fid(&self) -> RawFid {
       self.inner.as_raw_fid()
    }
}

impl AsFid for EventQueueImpl {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
       self.c_eq.as_fid()
    }
}

impl AsRawFid for EventQueueImpl {
    
    fn as_raw_fid(&self) -> RawFid {
        self.c_eq.as_raw_fid()
    }
}

impl<T: EqConfig, EQ: AsTypedFid<EqRawFid>> AsTypedFid<EqRawFid> for EventQueueBase<T, EQ> {
    
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<EqRawFid> {
        self.inner.as_typed_fid()
    }
}

impl<T: EqConfig, EQ: AsRawTypedFid<Output = EqRawFid>> AsRawTypedFid for EventQueueBase<T, EQ> {
    type Output = EqRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.inner.as_raw_typed_fid()
    }
}

impl AsTypedFid<EqRawFid> for EventQueueImpl {
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<EqRawFid> {
       self.c_eq.as_typed_fid()
    }
}

impl AsRawTypedFid for EventQueueImpl {
    type Output = EqRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.c_eq.as_raw_typed_fid()
    }
}


impl<T: EqConfig + WaitRetrievable + FdRetrievable> AsFd for EventQueue<T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        if let WaitObjType::Fd(fd) = self.wait_object().unwrap() {
            fd
        }
        else {
            panic!("Fabric object object type is not Fd")
        }
    }
}

impl AsFd for EventQueueImpl {
    fn as_fd(&self) -> BorrowedFd<'_> {
        if let WaitObjType::Fd(fd) = self.wait_object().unwrap() {
            fd
        }
        else {
            panic!("Fabric object object type is not Fd")
        }
    }
}

impl AsRawFd for EventQueueImpl {
    fn as_raw_fd(&self) -> RawFd {
        if let WaitObjType::Fd(fd) = self.wait_object().unwrap() {
            fd.as_raw_fd()
        }
        else {
            panic!("Fabric object object type is not Fd")
        }
    }
}


impl crate::BindImpl for EventQueueImpl {}
impl<T: EqConfig + 'static> crate::Bind for EventQueue<T> {
    
    fn inner(&self) -> Rc<dyn crate::BindImpl> {
        self.inner.clone()
    }
}


//================== EventQueue Attribute(fi_eq_attr) ==================//

pub struct EventQueueBuilder<'a, T, WRITE, WAIT, WAITFD> {
    eq_attr: EventQueueAttr,
    fabric: &'a crate::fabric::Fabric,
    ctx: Option<&'a mut T>,
    options: eqoptions::Options<WRITE, WAIT, WAITFD>,
}

impl<'a> EventQueueBuilder<'a, (), eqoptions::Off, WaitNoRetrieve, eqoptions::Off> {
    pub fn new(fabric: &'a crate::fabric::Fabric) -> Self {
       Self {
            eq_attr: EventQueueAttr::new(),
            fabric,
            ctx: None,
            options: Options::new(),
        }
    }
}

impl <'a, T, WRITE, WAIT, WAITFD> EventQueueBuilder<'a, T, WRITE, WAIT, WAITFD> {
    
    pub fn size(mut self, size: usize) -> Self {
        self.eq_attr.size(size);
        self
    }

    pub fn write(mut self) -> EventQueueBuilder<'a, T, eqoptions::On, WAIT, WAITFD> {
        self.eq_attr.write();

        EventQueueBuilder {
            options: self.options.writable(),
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: self.ctx,
        }
    }
    
    pub fn wait_none(mut self) -> EventQueueBuilder<'a, T, WRITE, WaitNone, Off> {
        self.eq_attr.wait_obj(crate::enums::WaitObj::None);

        EventQueueBuilder {
            options: self.options.no_wait(),
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: self.ctx,
        }
    }
    
    pub fn wait_fd(mut self) -> EventQueueBuilder<'a, T, WRITE, WaitRetrieve, On> {
        self.eq_attr.wait_obj(crate::enums::WaitObj::Fd);

        EventQueueBuilder {
            options: self.options.wait_fd(),
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: self.ctx,
        }
    }

    pub fn wait_set(mut self, set: &crate::sync::WaitSet) -> EventQueueBuilder<'a, T, WRITE, WaitNoRetrieve, Off> {
        self.eq_attr.wait_obj(crate::enums::WaitObj::Set(set));

        
        EventQueueBuilder {
            options: self.options.wait_no_retrieve(),
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: self.ctx,
        }
    }

    pub fn wait_mutex(mut self) -> EventQueueBuilder<'a, T, WRITE, WaitRetrieve, Off> {
        self.eq_attr.wait_obj(crate::enums::WaitObj::MutexCond);

        
        EventQueueBuilder {
            options: self.options.wait_retrievable(),
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: self.ctx,
        }
    }

    pub fn wait_yield(mut self) -> EventQueueBuilder<'a, T, WRITE, WaitNoRetrieve, Off> {
        self.eq_attr.wait_obj(crate::enums::WaitObj::Yield);

        EventQueueBuilder {
            options: self.options.wait_no_retrieve(),
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: self.ctx,
        }
    }

    pub fn signaling_vector(mut self, signaling_vector: i32) -> Self {
        self.eq_attr.signaling_vector(signaling_vector);
        self
    }

    pub fn context(self, ctx: &'a mut T) -> EventQueueBuilder<'a, T, WRITE, WAIT, WAITFD> {
        EventQueueBuilder {
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: Some(ctx),
            options: self.options,
        }
    }

    pub fn build(self) ->  Result<EventQueue<Options<WRITE, WAIT, WAITFD>>, crate::error::Error> {
        EventQueue::new(self.options, self.fabric, self.eq_attr, self.ctx)   
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

    pub(crate) fn write(&mut self) -> &mut Self {
        self.c_attr.flags |= FI_WRITE as u64;
        self
    }

    pub(crate) fn wait_obj(&mut self, wait_obj: crate::enums::WaitObj) -> &mut Self {
        
        if let crate::enums::WaitObj::Set(wait_set) = wait_obj {
            self.c_attr.wait_set = wait_set.as_raw_typed_fid();
        }
        self.c_attr.wait_obj = wait_obj.get_value();
        self
    }

    pub(crate) fn signaling_vector(&mut self, signaling_vector: i32) -> &mut Self {
        self.c_attr.flags |= FI_AFFINITY as u64;
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
pub struct EventError {
    pub(crate) c_err: libfabric_sys::fi_eq_err_entry,
}

impl EventError {
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

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_eq_err_entry {
        &mut self.c_err
    }       
}

impl Default for EventError {
    fn default() -> Self {
        Self::new()
    }
}

//================== EventQueueEntry (fi_eq_entry) ==================//

#[repr(C)]
#[derive(Clone)]
pub struct EventQueueEntry<T, F> {
    pub(crate) c_entry: libfabric_sys::fi_eq_entry,
    event_fid: F, 
    phantom: std::marker::PhantomData<T>,
}

impl<T, F: AsRawFid> EventQueueEntry<T, F> {
    // const SIZE_OK: () = assert!(std::mem::size_of::<T>() == std::mem::size_of::<usize>(), 
    // "The context of an EventQueueEntry must always be of size equal to usize datatype.");

    pub fn new(event_fid: F) -> Self {
        // let _ = Self::SIZE_OK;
        let c_entry = libfabric_sys::fi_eq_entry { 
            fid: event_fid.as_raw_fid(), 
            context: std::ptr::null_mut(), 
            data: 0 
        };

        Self { c_entry, event_fid, phantom: std::marker::PhantomData }
    }

    pub fn fid(&mut self) -> &F {
        &self.event_fid
    }

    pub fn set_context(&mut self, context: &mut T) -> &mut Self {
        let context_writable: *mut *mut std::ffi::c_void =  &mut (self.c_entry.context);
        let context_writable_usize: *mut  usize = context_writable as  *mut usize;
        unsafe { *context_writable_usize = *(context as *mut T as *mut usize) };
        self
    }

    pub fn set_data(&mut self, data: u64) -> &mut Self {
        self.c_entry.data = data;
        self
    }

    pub fn data(&self) -> u64 {
        self.c_entry.data
    }

    pub fn context(&self) -> T {
        let context_ptr:*mut *mut T = &mut (self.c_entry.context as *mut T);
        unsafe { std::mem::transmute_copy::<T,T>(&*(context_ptr as *const T)) }
    }

    pub fn is_context_equal(&self, ctx: &crate::Context) -> bool {

        std::ptr::eq(self.c_entry.context, ctx as *const crate::Context as *const std::ffi::c_void)
    }

}

// impl<T> Default for EventQueueEntry<T> {
//     fn default() -> Self {
//         Self::new()
//     }
// }

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

    pub fn get_fid(&self) -> libfabric_sys::fid_t {
        self.c_entry.fid
    }

    pub fn get_info<E: Caps>(&self) -> Result<InfoEntry<E>, crate::error::Error> { //[TODO] Should returen the proper type of info entry
        let caps = E::bitfield();
        if caps & unsafe{(*self.c_entry.info).caps} == caps {
            Ok(InfoEntry::<E>::new(self.c_entry.info))
        }
        else {
            Err(crate::error::Error::caps_error())
        }
    }
    pub fn get_info_from_caps<E: Caps>(&self, _caps: &InfoHints<E>) -> Result<InfoEntry<E>, crate::error::Error> { //[TODO] Should returen the proper type of info entry
        let caps = E::bitfield();
        if caps & unsafe{(*self.c_entry.info).caps} == caps {
            Ok(InfoEntry::<E>::new(self.c_entry.info))
        }
        else {
            Err(crate::error::Error::caps_error())
        }
    }
}

impl Default for EventQueueCmEntry {
    fn default() -> Self {
        Self::new()
    }
}

//================== Async Stuff ==============================//


//================== EventQueue related tests ==================//

#[cfg(test)]
mod tests {

    use crate::info::Info;

    use super::EventQueueBuilder;

    // #[test]
    // fn eq_write_read_self() {
    //     let info = Info::new().request().unwrap();
    //     let entries = info.get();
    //     let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
    //     let eq = EventQueueBuilder::new(&fab)
    //         .size(32)
    //         .write()
    //         .wait_fd()
    //         .build().unwrap();

    //     for mut i in 0_usize ..5 {
    //         let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
    //         if i & 1 == 1 {
    //             entry.fid(&fab);
    //         }
    //         else {
    //             entry.fid(&eq);
    //         }

    //         entry.context(&mut i);
    //         eq.write(Event::Notify(entry)).unwrap();
    //     }
    //     for i in 0..10 {

    //         let ret = if i & 1 == 1 {
    //             eq.read().unwrap()
    //         }
    //         else {
    //             eq.peek().unwrap()
    //         };

    //         if let crate::eq::Event::Notify(entry) = ret {
                
    //             if entry.get_context() != i /2 {
    //                 panic!("Unexpected context {} vs {}", entry.get_context(), i/2);
    //             }
                
    //             if entry.get_fid() != if i & 2 == 2 {fab.as_raw_fid()} else {eq.as_raw_fid()} {
    //                 panic!("Unexpected fid {:?}", entry.get_fid());
    //             }
    //         }
    //         else {
    //             panic!("Unexpected EventType");
    //         } 
    //     }

    //     let ret = eq.read();
    //     if let Err(ref err) = ret {
    //         if !matches!(err.kind, crate::error::ErrorKind::TryAgain) {
    //             ret.unwrap();
    //         }
    //     }

    // }

    // #[test]
    // fn eq_size_verify() {
    //     let info = Info::new().request().unwrap();
    //     let entries = info.get();
    //     let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
    //     let eq = EventQueueBuilder::new(&fab)
    //         .size(32)
    //         .write()
    //         .wait_fd()
    //         .build().unwrap();

    //     for mut i in 0_usize .. 32 {
    //         let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
    //         entry
    //             .fid(&fab)
    //             .context(&mut i);
    //         eq.write(Event::Notify(entry)).unwrap();
    //     }
    // }

    // #[test]
    // fn eq_write_sread_self() {
    //     let info = Info::new().request().unwrap();
    //     let entries = info.get();
    //     let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
    //     let eq = EventQueueBuilder::new(&fab)
    //         .size(32)
    //         .write()
    //         .wait_fd()
    //         .build().unwrap();

    //     for mut i in 0_usize ..5 {
    //         let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
    //         if i & 1 == 1 {
    //             entry.fid(&fab);
    //         }
    //         else {
    //             entry.fid(&eq);
    //         }

    //         entry.context(&mut i);
    //         eq.write(Event::Notify(entry)).unwrap();
    //     }
    //     for i in 0..10 {
    //         let event = if (i & 1) == 1 { eq.sread(2000) } else { eq.speek(2000) }.unwrap();

    //         if let crate::eq::Event::Notify(entry) = event {

    //             if entry.get_context() != i /2 {
    //                 panic!("Unexpected context {} vs {}", entry.get_context(), i/2);
    //             }
                
    //             if entry.get_fid() != if i & 2 == 2 {fab.as_raw_fid()} else {eq.as_raw_fid()} {
    //                 panic!("Unexpected fid {:?}", entry.get_fid());
    //             }
    //         }
    //         else {
    //             panic!("Unexpected EventType");
    //         }
    //     }
            
    //     let ret = eq.read();
    //     if let Err(ref err) = ret {
    //         if !matches!(err.kind, crate::error::ErrorKind::TryAgain) {
    //             ret.unwrap();
    //         }
    //     }

    // }

    // #[test]
    // fn eq_readerr() {
    //     let info = Info::new().request().unwrap();
    //     let entries = info.get();
    //     let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
    //     let eq = EventQueueBuilder::new(&fab)
    //         .size(32)
    //         .write()
    //         .wait_fd()
    //         .build().unwrap();

    //     for mut i in 0_usize ..5 {
    //         let mut entry: crate::eq::EventQueueEntry<usize> = crate::eq::EventQueueEntry::new();
    //         entry.fid(&fab);

    //         entry.context(&mut i);
    //         eq.write(Event::Notify(entry)).unwrap();
    //     }

    //     for i in 0..5 {
    //         let event = eq.read().unwrap();

    //         if let Event::Notify(entry) = event {

    //             if entry.get_context() != i  {
    //                 panic!("Unexpected context {} vs {}", entry.get_context(), i/2);
    //             }
                
    //             if entry.get_fid() != fab.as_raw_fid() {
    //                 panic!("Unexpected fid {:?}", entry.get_fid());
    //             }
    //         }
    //         else {
    //             panic!("Unexpected EventQueueEntryFormat");
    //         }
    //     }
    //     let ret = eq.readerr();
    //     if let Err(ref err) = ret {
    //         if !matches!(err.kind, crate::error::ErrorKind::TryAgain) {
    //             ret.unwrap();
    //         }
    //     }
    // }


    #[test]
    fn eq_open_close_sizes() {
        let info = Info::new().request().unwrap();
        let entries = info.get();
        
        let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
        for i in -1..17 {
            let size = if i == -1 { 0 } else { 1 << i };
            let _eq = EventQueueBuilder::new(&fab)
                .wait_fd()
                .size(size)
                .build().unwrap();
            
        }
    }
}



#[cfg(test)]
mod libfabric_lifetime_tests {

    use crate::info::Info;

    use super::EventQueueBuilder;


    #[test]
    fn eq_drops_before_fabric() {
        let info = Info::new().request().unwrap();
        let entries = info.get();
        
        let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
        let mut eqs = Vec::new();
        for i in -1..17 {
            let size = if i == -1 { 0 } else { 1 << i };
            let eq = EventQueueBuilder::new(&fab)
                .wait_fd()
                .size(size)
                .build().unwrap();
            eqs.push(eq);
            println!("Count = {} \n", std::rc::Rc::strong_count(&fab.inner));
        }

        drop(fab);
        println!("Count = {} After dropping fab\n", std::rc::Rc::strong_count(&eqs[0].inner._fabric_rc));
    }
}