use std::{marker::PhantomData, os::fd::{AsFd, BorrowedFd}, rc::{Rc, Weak}, cell::RefCell, collections::{HashMap, btree_map::Entry}, ops::Deref, borrow::BorrowMut};

use async_io::Async;
use libfabric_sys::{fi_mutex_cond, FI_AFFINITY, FI_WRITE};
#[allow(unused_imports)]
use crate::fid::AsFid;
use crate::{fid::AsRawFid, mr::MemoryRegion, comm::collective::MulticastGroupCollective, av::AddressVector, error::Error, cq::AsyncCtx};
use crate::{enums::WaitObjType, eqoptions::{self, EqConfig,  EqWritable, Off, On, Options, WaitNoRetrieve, WaitNone, WaitRetrieve}, FdRetrievable, WaitRetrievable, fabric::FabricImpl, infocapsoptions::Caps, info::{InfoHints, InfoEntry}, fid::{OwnedFid, self, Fid}, mr::MemoryRegionImpl, av::AddressVectorImpl, comm::collective::MulticastGroupCollectiveImpl};

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
    MrComplete(EventQueueEntry<T, *mut libfabric_sys::fid>),
    AVComplete(EventQueueEntry<T, AddressVector>),
    JoinComplete(EventQueueEntry<T, MulticastGroupCollective>),
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

struct EqAsyncRead<'a>{
    buf: *mut std::ffi::c_void,
    event: &'a mut u32,
    eq: &'a AsyncEventQueueImpl,
}

impl<'a> Unpin for EqAsyncRead<'a>{}

impl<'a> async_std::future::Future for EqAsyncRead<'a>{
    type Output=Result<isize, Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        // let mut buff = vec![1u8];
        // self.poll_read(cx, &mut buff[..])
        // let ev = self.event;
        let fid = self.eq.handle();
        let buf = self.buf;
        let ev = self.get_mut();
        loop {
            let err = unsafe { libfabric_sys::inlined_fi_eq_read(fid, ev.event , buf, std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), 0) };

            // let err = read_cq_entry_into!(libfabric_sys::inlined_fi_cq_read, self.cq.0.as_ref().handle(), self.num_entries, self.buf,);
            if err < 0 {
                let err = Error::from_err_code(-err as u32);
                if !matches!(err.kind, crate::error::ErrorKind::TryAgain) 
                {
                    return std::task::Poll::Ready(Err(err));
                }
                else if ev.eq.0.poll_readable(cx).is_ready() {
                    continue;
                 }
                else {
                    return std::task::Poll::Pending;
                }
            }
            else {
                return std::task::Poll::Ready(Ok(err));
            }
        }
    }
}

//================== EventQueue (fi_eq) ==================//
pub(crate) struct EventQueueImpl {
    c_eq: *mut libfabric_sys::fid_eq,
    fid: OwnedFid,
    wait_obj: Option<libfabric_sys::fi_wait_obj>,
    mrs: RefCell<std::collections::HashMap<Fid, Weak<MemoryRegionImpl>>>,   // We neep maps Fid -> MemoryRegionImpl/AddressVectorImpl/MulticastGroupCollectiveImpl, however, we don't want to extend 
                                                                            // the lifetime of any of these objects just because of the maps.
                                                                            // Moreover, these objects will keep references to the EQ to keep it from dropping while
                                                                            // they are still bound, thus, we would have cyclic references that wouldn't let any 
                                                                            // of the two sides drop. 
    avs: RefCell<std::collections::HashMap<Fid, Weak<AddressVectorImpl>>>,
    mcs: RefCell<std::collections::HashMap<Fid, Weak<MulticastGroupCollectiveImpl>>>,
    pending_entries: RefCell<HashMap<(u32, usize), Event<usize>>>,
    pending_cm_entries: RefCell<HashMap<(u32,libfabric_sys::fid_t), Vec::<Event<usize>> >>,
    event_buffer: RefCell<Vec<u8>>,
    _fabric_rc: Rc<FabricImpl>,
}

pub(crate) struct AsyncEventQueueImpl(pub(crate) Async<EventQueueImpl>);

impl AsyncEventQueueImpl {
    pub(crate) fn new<T0>(fabric: &Rc<crate::fabric::FabricImpl>, attr: EventQueueAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        Ok(Self(Async::new(EventQueueImpl::new(fabric, attr, context)?).unwrap()))
    }
}

impl Deref for  AsyncEventQueueImpl {
    type Target = EventQueueImpl;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

pub struct EventQueue<T: EqConfig> {
    pub(crate) inner: Rc<AsyncEventQueueImpl>,
    phantom: PhantomData<T>,
}

impl AsyncEventQueueImpl {
    pub(crate) async fn read_async(&self) -> Result<Event<usize>, crate::error::Error>{
        let mut event = 0 ;
        let mut buffer: Vec<u8> = vec![0; std::mem::size_of::<libfabric_sys::fi_eq_err_entry>()];
        let bytes = EqAsyncRead{buf: buffer.as_mut_ptr().cast(), event: &mut event, eq: self}.await?;

        // println!("Done!");
        Ok(self.read_eq_entry(bytes, &buffer, &event))
    }
}

impl<'a> EventQueueImpl {

    pub(crate) fn new<T0>(fabric: &Rc<crate::fabric::FabricImpl>, mut attr: EventQueueAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        let mut c_eq: *mut libfabric_sys::fid_eq  = std::ptr::null_mut();
        let c_eq_ptr: *mut *mut libfabric_sys::fid_eq = &mut c_eq;

        let err = 
            if let Some(ctx) = context {
                unsafe {libfabric_sys::inlined_fi_eq_open(fabric.c_fabric, attr.get_mut(), c_eq_ptr, (ctx as *mut T0).cast())}
            }
            else {
                unsafe {libfabric_sys::inlined_fi_eq_open(fabric.c_fabric, attr.get_mut(), c_eq_ptr, std::ptr::null_mut())}
            };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    c_eq, 
                    fid: OwnedFid::from(unsafe{ &mut (*c_eq).fid }), 
                    wait_obj:  Some(attr.c_attr.wait_obj),
                    mrs: RefCell::new(HashMap::new()),
                    avs: RefCell::new(HashMap::new()),
                    mcs: RefCell::new(HashMap::new()),
                    pending_entries: RefCell::new(HashMap::new()),
                    pending_cm_entries: RefCell::new(HashMap::new()),
                    event_buffer: RefCell::new(vec![0; std::mem::size_of::<libfabric_sys::fi_eq_err_entry>()]),
                    _fabric_rc: fabric.clone(),
                })
        }
    }

    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_eq {
        self.c_eq
    }

    pub(crate) fn bind_mr(&self, mr: &Rc<MemoryRegionImpl>) {
        self.mrs.borrow_mut().insert(mr.as_fid().as_raw_fid(), Rc::downgrade(mr));
    }

    pub(crate) fn bind_av(&self, av: &Rc<AddressVectorImpl>) {
        self.avs.borrow_mut().insert(av.as_fid().as_raw_fid(), Rc::downgrade(av));
    }

    pub(crate) fn bind_mc(&self, mc: &Rc<MulticastGroupCollectiveImpl>) {
        self.mcs.borrow_mut().insert(mc.as_fid().as_raw_fid(), Rc::downgrade(mc));
    }

    pub(crate) fn read(&self) -> Result<Event<usize>, crate::error::Error>{ //[TODO] Use an "owned" buffer instead of allocating a new one for each attempt and copy it out
        let mut event = 0 ;
        let mut buffer = self.event_buffer.borrow_mut();
        let ret = unsafe { libfabric_sys::inlined_fi_eq_read(self.handle(), &mut event, buffer.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), 0) };
        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(self.read_eq_entry(ret, &buffer, &event))
        }
    }



    pub(crate) fn peek(&self) -> Result<Event<usize>, crate::error::Error>{
        let mut event = 0 ;
        let mut buffer = self.event_buffer.borrow_mut();
        let ret = unsafe { libfabric_sys::inlined_fi_eq_read(self.handle(), &mut event, buffer.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), libfabric_sys::FI_PEEK.into()) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(self.read_eq_entry(ret, &buffer, &event))
        }
    }

    pub(crate) fn readerr(&self) -> Result<EventError, crate::error::Error> {

        let mut buffer = self.event_buffer.borrow_mut();

        let ret = unsafe { libfabric_sys::inlined_fi_eq_readerr(self.handle(), buffer.as_mut_ptr().cast(), 0) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            let mut err_event = EventError::new();
            err_event.c_err = unsafe { std::ptr::read(buffer.as_ptr().cast()) };
            Ok(err_event)
        }
    }

    pub(crate) fn peekerr(&self) -> Result<EventError, crate::error::Error> {
        
        let mut buffer = self.event_buffer.borrow_mut();
        let ret = unsafe { libfabric_sys::inlined_fi_eq_readerr(self.handle(), buffer.as_mut_ptr().cast(), libfabric_sys::FI_PEEK.into()) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            let mut err_event = EventError::new();
            err_event.c_err = unsafe { std::ptr::read(buffer.as_ptr().cast()) };
            Ok(err_event)
        }
    }

    pub(crate) fn strerror(&self, entry: &EventError) -> &str {
        let ret = unsafe { libfabric_sys::inlined_fi_eq_strerror(self.handle(), -entry.c_err.prov_errno, entry.c_err.err_data, std::ptr::null_mut(), 0) };
    
            unsafe{ std::ffi::CStr::from_ptr(ret).to_str().unwrap() }
    }

    pub(crate) fn write(&self, event: Event<usize>, flags: u64) -> Result<(), crate::error::Error>{
        let event_val = event.get_value();
        let (event_entry, event_entry_size) = event.get_entry();

        let ret = unsafe { libfabric_sys::inlined_fi_eq_write(self.handle(), event_val, event_entry, event_entry_size, flags) };
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

        let ret = unsafe { libfabric_sys::inlined_fi_eq_sread(self.handle(), &mut event,  buffer.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), timeout, flags) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(self.read_eq_entry(ret, &buffer, &event))
        }
    }

    pub(crate) fn speek(&self, timeout: i32) -> Result<Event<usize>, crate::error::Error> { 
        let mut event = 0;
        let mut buffer = self.event_buffer.borrow_mut();


        let ret = unsafe { libfabric_sys::inlined_fi_eq_sread(self.handle(), &mut event,  buffer.as_mut_ptr().cast(), std::mem::size_of::<libfabric_sys::fi_eq_err_entry>(), timeout, libfabric_sys::FI_PEEK.into()) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()) )
        }
        else {
            Ok(self.read_eq_entry(ret, &buffer, &event))
        }
    }

    fn read_eq_entry(&self, bytes_read: isize, buffer: &[u8], event: &u32) -> Event<usize> {
        if event == &libfabric_sys::FI_CONNREQ || event == &libfabric_sys::FI_CONNECTED || event == &libfabric_sys::FI_SHUTDOWN {
            debug_assert_eq!(bytes_read as usize, std::mem::size_of::<libfabric_sys::fi_eq_cm_entry>());
            Event::from_connect_value(*event, EventQueueCmEntry {
                c_entry: unsafe { std::ptr::read(buffer.as_ptr().cast()) }
            })          
        }
        else {
            debug_assert_eq!(bytes_read as usize, std::mem::size_of::<libfabric_sys::fi_eq_entry>());
            let c_entry: libfabric_sys::fi_eq_entry = unsafe { std::ptr::read(buffer.as_ptr().cast()) };
            
            if event == &libfabric_sys::FI_NOTIFY {
                panic!("Unexpected event");
            //     self.
            //     Event::Notify(entry)
            }
            
            if event == &libfabric_sys::FI_MR_COMPLETE {
                // let event_fid = self.mrs.borrow().get(&c_entry.fid).unwrap().clone();
                Event::MrComplete(
                    EventQueueEntry::<usize, *mut libfabric_sys::fid> {
                        c_entry,
                        event_fid: c_entry.fid, //MemoryRegion::from_impl(&event_fid.upgrade().unwrap()),
                        phantom: PhantomData,
                })
            }
            else if event == &libfabric_sys::FI_AV_COMPLETE {
                let event_fid = self.avs.borrow().get(&c_entry.fid).unwrap().clone();
                Event::AVComplete(
                    EventQueueEntry::<usize, AddressVector> {
                        c_entry,
                        event_fid: AddressVector::from_impl(&event_fid.upgrade().unwrap()),
                        phantom: PhantomData,
                })
            }
            else if event == &libfabric_sys::FI_JOIN_COMPLETE {
                let event_fid = self.mcs.borrow().get(&c_entry.fid).unwrap().clone();
                Event::JoinComplete(
                    EventQueueEntry::<usize, MulticastGroupCollective> {
                        c_entry,
                        event_fid: MulticastGroupCollective::from_impl(&event_fid.upgrade().unwrap()),
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
    
    #[allow(dead_code)]
    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_eq {
        self.inner.handle()
    }

    pub(crate) fn new<T0>(_options: T,fabric: &crate::fabric::Fabric, attr: EventQueueAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {

        Ok(
            Self {
                inner: Rc::new(
                    AsyncEventQueueImpl::new(&fabric.inner, attr, context)?
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

impl<T: EqConfig + WaitRetrievable + FdRetrievable> EventQueue<T> {
    pub async fn read_async(&self) -> Result<Event<usize>, crate::error::Error>{
        self.inner.read_async().await
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



impl<T: EqConfig> AsFid for EventQueue<T> {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
       self.inner.as_fid()
    }
}

impl AsFid for EventQueueImpl {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
       self.fid.as_fid()
    }
}
impl AsFid for AsyncEventQueueImpl {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
       self.fid.as_fid()
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


impl crate::BindImpl for EventQueueImpl {}
impl crate::BindImpl for AsyncEventQueueImpl {}
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
            self.c_attr.wait_set = wait_set.handle();
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
    c_entry: libfabric_sys::fi_eq_entry,
    event_fid: F, 
    phantom: std::marker::PhantomData<T>,
}

impl<T, F: AsFid> EventQueueEntry<T, F> {
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

pub struct EventQueueFut<const E: libfabric_sys::_bindgen_ty_18>{
    pub(crate) req_fid: libfabric_sys::fid_t,
    pub(crate) eq: Rc<AsyncEventQueueImpl>,
    pub(crate) ctx: usize,
}

impl<const E: libfabric_sys::_bindgen_ty_18> async_std::future::Future for EventQueueFut<E> {
    type Output=Result<Event<usize>, Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        loop {
            if E == libfabric_sys::FI_CONNREQ || E == libfabric_sys::FI_CONNECTED || E == libfabric_sys::FI_SHUTDOWN {
                if let Some(mut entries) = self.eq.pending_cm_entries.borrow_mut().remove(&(E, self.req_fid)) {
                    if !entries.is_empty() {
                        let entry = entries.pop().unwrap();
                        return std::task::Poll::Ready(Ok(entry))
                    }
                }
            }
            else if let Some(mut entry) = self.eq.pending_entries.borrow_mut().remove(&(E, self.ctx)) {
                if E == libfabric_sys::FI_MR_COMPLETE {
                    println!("Got MrComplete: {}", self.ctx);
                }
                if self.ctx != 0 {
                    match entry {
                        crate::eq::Event::MrComplete(ref mut e) => {println!("Got MrComplete!"); e.c_entry.context = unsafe{ ( *(e.c_entry.context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())}},
                        crate::eq::Event::AVComplete(ref mut e) => {println!("Got AvComplete!"); e.c_entry.context = unsafe{ ( *(e.c_entry.context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())}},
                        crate::eq::Event::JoinComplete(ref mut e) => {e.c_entry.context = unsafe{ ( *(e.c_entry.context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())}},
                        _ => panic!("Unexpected!"),
                    }
                    return std::task::Poll::Ready(Ok(entry))
                }
                else {
                    return std::task::Poll::Ready(Ok(entry))
                }
            }
            println!("About to block, waiting for connreq");
            let fut = self.eq.read_async();
            let mut pinned = Box::pin(fut) ;
            let res = match pinned.as_mut().poll(cx) {
                std::task::Poll::Ready(res) => {println!("Continuing!"); res},
                std::task::Poll::Pending => {println!("Returng pending"); return std::task::Poll::Pending},
            }?;
            
            match &res {
                    // crate::eq::Event::Notify(entry) | 
                crate::eq::Event::MrComplete(e) => {println!("Inserting MrComplete: {}", e.c_entry.context as usize); self.eq.pending_entries.borrow_mut().insert((libfabric_sys::FI_MR_COMPLETE, e.c_entry.context as usize),res);},
                crate::eq::Event::AVComplete(e) => {println!("Inserting AVComplete");self.eq.pending_entries.borrow_mut().insert((libfabric_sys::FI_AV_COMPLETE, e.c_entry.context as usize),res);},
                crate::eq::Event::JoinComplete(e) => {self.eq.pending_entries.borrow_mut().insert((libfabric_sys::FI_JOIN_COMPLETE, e.c_entry.context as usize),res);},
                crate::eq::Event::ConnReq(_) => {
                    println!("Got ConnReq!");
                    let mut map = self.eq.pending_cm_entries.borrow_mut();
                    if let Some(entries) = map.get_mut(&(libfabric_sys::FI_CONNREQ, self.req_fid)){
                        entries.push(res);
                    }
                    else {
                        map.insert((libfabric_sys::FI_CONNREQ, self.req_fid), vec![res]);
                    }
                },
                crate::eq::Event::Connected(_) => {
                    println!("Got Connected!");
                    let mut map = self.eq.pending_cm_entries.borrow_mut();
                    if let Some(entries) = map.get_mut(&(libfabric_sys::FI_CONNECTED, self.req_fid)){
                        entries.push(res);
                    }
                    else {
                        map.insert((libfabric_sys::FI_CONNECTED, self.req_fid), vec![res]);
                    }
                },
                crate::eq::Event::Shutdown(_) => { // [TODO] No one will explcitly look for shutdown requests, should probably store it somewhere else
                    println!("Got Shutdown!");
                    let mut map = self.eq.pending_cm_entries.borrow_mut();
                    if let Some(entries) = map.get_mut(&(libfabric_sys::FI_SHUTDOWN, self.req_fid)){
                        entries.push(res);
                    }
                    else {
                        map.insert((libfabric_sys::FI_SHUTDOWN, self.req_fid), vec![res]);
                    }
                },
            }

        }
    }
}


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