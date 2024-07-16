use std::{rc::Rc, collections::HashMap, cell::RefCell, future::Future, task::ready, pin::Pin, os::fd::AsRawFd};

#[cfg(feature="use-async-std")]
use async_io::Async as Async;
#[cfg(feature="use-tokio")]
use tokio::io::unix::AsyncFd as Async;

use crate::{eq::{Event, EventQueueImpl, EventQueueAttr, EventQueueBase, EventQueueImplT, WritableEventQueueImplT, EventError}, error::{Error, ErrorKind}, fid::{AsFid, self, RawFid, AsRawFid, AsRawTypedFid, EqRawFid}, async_::AsyncCtx, cq::WaitObjectRetrievable};

use super::AsyncFid;

pub type EventQueue<T> = EventQueueBase<T>;

pub(crate) trait  FdEq: EventQueueImplT + AsRawFd{
    
}

impl<const WRITE: bool> AsyncFid for AsyncEventQueueImpl<WRITE> {
    fn trywait(&self) -> Result<(), Error> {
        self.base.get_ref()._fabric_rc.trywait(self.base.get_ref())
    }
}

#[derive(Clone)]
enum EqType<'a>{
    Write(&'a AsyncEventQueueImpl<true>),
    NoWrite(&'a AsyncEventQueueImpl<false>),
}

impl<'a>  EqType<'a> {
    #[inline]
    pub(crate) fn trywait(&self) -> Result<(), Error> {
        match self {
            EqType::Write(e) => e.trywait(),
            EqType::NoWrite(e) => e.trywait(),
        }
    }

    #[inline]
    pub(crate) fn read_in(&self, buff: &mut [u8], event: &mut u32) -> Result<usize, Error>{
        match self {
            EqType::Write(e) => e.read_in(buff, event),
            EqType::NoWrite(e) => e.read_in(buff, event),
        }
    }

    #[inline]
    pub(crate) fn readerr_in(&self, buff: &mut [u8]) -> Result<usize, Error>{
        match self {
            EqType::Write(e) => e.readerr_in(buff),
            EqType::NoWrite(e) => e.readerr_in(buff),
        }
    }

    #[inline]
    pub(crate) fn remove_cm_entry(&self, event_type: libfabric_sys::_bindgen_ty_18, req_fid: RawFid) -> Option<Result<Event<usize>, Error>>{
        match self {
            EqType::Write(e) =>  e.remove_cm_entry(event_type, req_fid),
            EqType::NoWrite(e) =>  e.remove_cm_entry(event_type, req_fid),
        }
    }

    #[inline]
    pub(crate) fn insert_cm_entry(&self, event_type: libfabric_sys::_bindgen_ty_18, req_fid: RawFid, entry: Result<Event<usize>, Error>) {
        match self {
            EqType::Write(e) =>  e.insert_cm_entry(event_type, req_fid, entry),
            EqType::NoWrite(e) =>  e.insert_cm_entry(event_type, req_fid, entry),
        }
    }

    #[inline]
    pub(crate) fn remove_entry(&self, ctx: usize) -> Option<Result<Event<usize>, Error>>{
        match self {
            EqType::Write(e) => e.remove_entry(ctx),
            EqType::NoWrite(e) => e.remove_entry(ctx),
        }
    }

    #[inline]
    pub(crate) fn insert_entry(&self, ctx: usize, entry: Result<Event<usize>, Error>){
        match self {
            EqType::Write(e) => e.insert_entry(ctx, entry),
            EqType::NoWrite(e) => e.insert_entry(ctx, entry),
        }
    }

    #[inline]
    pub(crate) fn read_eq_entry(&self, bytes_read: usize, buffer: &[u8], event: &u32) -> Event<usize> {
        match self {
            EqType::Write(e) => e.base.get_ref().read_eq_entry(bytes_read, buffer, event),
            EqType::NoWrite(e) => e.base.get_ref().read_eq_entry(bytes_read, buffer, event),
        }
    }

    #[inline]
    pub(crate) fn insert_err_entry(&self, req_fid: RawFid, entry: Result<Event<usize>, Error>){
        match self {
            EqType::Write(e) => e.insert_err_entry(req_fid, entry),
            EqType::NoWrite(e) => e.insert_err_entry(req_fid, entry),
        }
    }

    #[inline]
    pub(crate) fn remove_err_entry(&self, req_fid: RawFid) -> Option<Result<Event<usize>, Error>>{
        match self {
            EqType::Write(e) => e.remove_err_entry(req_fid),
            EqType::NoWrite(e) => e.remove_err_entry(req_fid),
        }
    }


}


#[cfg(feature = "use-tokio")]
pub(crate) enum FutType<'a> {
    Write(Pin<Box<dyn Future<Output = Result<tokio::io::unix::AsyncFdReadyGuard<'a, EventQueueImpl<true,true,true,true>>, std::io::Error>> + 'a>>),
    NoWrite(Pin<Box<dyn Future<Output = Result<tokio::io::unix::AsyncFdReadyGuard<'a, EventQueueImpl<false,true,true,true>>, std::io::Error>> + 'a>>),
}

#[cfg(feature = "use-async-std")]
pub(crate) enum FutType<'a> {
    Write(Pin<Box<async_io::Readable<'a, EventQueueImpl<true,true,true,true>>>>),
    NoWrite(Pin<Box<async_io::Readable<'a, EventQueueImpl<false,true,true,true>>>>),
}

pub struct EqAsyncRead<'a>{
    buf: &'a mut [u8],
    event: &'a mut u32,
    eq: EqType<'a>,
    fut: Option<FutType<'a>>,
}

impl<'a> EqAsyncRead<'a> {
    fn new(buf: &'a mut [u8], event: &'a mut u32, eq: EqType<'a>,) -> Self {
        Self {
            buf,
            event,
            eq,
            fut: None,
        }
    }
} 
#[cfg(feature="use-tokio")]
enum Guard<'a> {
    Write(tokio::io::unix::AsyncFdReadyGuard<'a, EventQueueImpl<true, true, true, true>>),
    NoWrite(tokio::io::unix::AsyncFdReadyGuard<'a, EventQueueImpl<false, true, true, true>>),
}

#[cfg(feature="use-async-std")]
enum Guard {
    Write(()),
    NoWrite(()),
}


#[cfg(feature="use-tokio")]
impl<'a> Guard<'a> {
    fn clear_ready(&mut self) {
        match self{
            Guard::Write(g) => g.clear_ready(),
            Guard::NoWrite(g) => g.clear_ready(),
        }
    }
}

impl<'a> Future for EqAsyncRead<'a>{
    type Output=Result<usize, Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let ev = self.get_mut();
        loop {
            // println!("About to block waiting for event");
            let (err, _guard) = if ev.eq.trywait().is_err() {
                (ev.eq.read_in( ev.buf,  ev.event), None)
            }
            else {
                if ev.fut.is_none() {
                    ev.fut = Some(
                        match ev.eq {
                            EqType::Write(e) => FutType::Write(Box::pin(e.base.readable())),
                            EqType::NoWrite(e) => FutType::NoWrite(Box::pin(e.base.readable())),
                        })
                }

                // Tokio returns something we need, async_std returns ()
                #[allow(clippy::unit_arg)]
                let _guard = match ev.fut.as_mut().unwrap() {
                    FutType::Write(e) => Guard::Write(ready!(e.as_mut().poll(cx)).unwrap()),
                    FutType::NoWrite(e) => Guard::NoWrite(ready!(e.as_mut().poll(cx)).unwrap()),
                };
                
                // We only need to reset the option to none
                #[allow(clippy::let_underscore_future)]
                let _ = ev.fut.take().unwrap();
                (ev.eq.read_in( ev.buf,  ev.event), Some(_guard))
            };
            match err {
                Err(error) => {
                    if !matches!(error.kind, crate::error::ErrorKind::TryAgain) {
                        if matches!(error.kind, crate::error::ErrorKind::ErrorAvailable)
                        {
                            let _len = ev.eq.readerr_in(ev.buf)?;
                            let mut err_event = EventError::new();
                            err_event.c_err = unsafe { std::ptr::read(ev.buf.as_ptr().cast()) };
                            return std::task::Poll::Ready(Err(Error::from_queue_err(crate::error::QueueError::Event(err_event))));
                        }
                    }
                    else {
                        // println!("Will continue");
                        if let Some(mut guard) = _guard {
                            match ev.eq {
                                EqType::Write(e) => { if e.pending_cm_entries.borrow().is_empty() && e.pending_entries.borrow().is_empty() {guard.clear_ready()}},
                                EqType::NoWrite(e) => { if e.pending_cm_entries.borrow().is_empty() && e.pending_entries.borrow().is_empty() {guard.clear_ready()}},
                            }
                        }
                        continue;
                    }
                },
                Ok(len) => return std::task::Poll::Ready(Ok(len)),
            }
        }
    }
}

struct EqAsyncReadOwned<'a> {
    buf: Vec<u8>,
    event: u32,
    eq: EqType<'a>,
    fut: Option<FutType<'a>>,
}

impl<'a> EqAsyncReadOwned<'a> {
    pub(crate) fn new(eq: EqType<'a>) -> Self {
        Self {
            buf: vec![0; std::mem::size_of::<libfabric_sys::fi_eq_err_entry>()],
            event: 0,
            eq,
            fut: None,
        }
    }
}

impl<'a> Future for EqAsyncReadOwned<'a>{
    type Output=Result<usize, Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let ev = self.get_mut();
        loop {
            println!("About to block waiting for event");
            let (err, _guard) = if ev.eq.trywait().is_err() {
                println!("Could not block");
                (ev.eq.read_in( &mut ev.buf,  &mut ev.event), None)
            }
            else {
                if ev.fut.is_none() {
                    ev.fut = Some(
                        match ev.eq {
                            EqType::Write(e) => FutType::Write(Box::pin(e.base.readable())),
                            EqType::NoWrite(e) => FutType::NoWrite(Box::pin(e.base.readable())),
                        })
                }

                // Tokio returns something we need, async_std returns ()
                #[allow(clippy::unit_arg)]
                let _guard = match ev.fut.as_mut().unwrap() {
                    FutType::Write(e) => Guard::Write(ready!(e.as_mut().poll(cx)).unwrap()),
                    FutType::NoWrite(e) => Guard::NoWrite(ready!(e.as_mut().poll(cx)).unwrap()),
                };

                println!("Did not block");

                
                // We only need to reset the option to none
                #[allow(clippy::let_underscore_future)]
                let _ = ev.fut.take().unwrap();
                (ev.eq.read_in( &mut ev.buf,  &mut ev.event), Some(_guard))
            };
            match err {
                Err(error) => {
                    if !matches!(error.kind, crate::error::ErrorKind::TryAgain) {
                        if matches!(error.kind, crate::error::ErrorKind::ErrorAvailable)
                        {
                            let _len = ev.eq.readerr_in(&mut ev.buf)?;
                            let mut err_event = EventError::new();
                            err_event.c_err = unsafe { std::ptr::read(ev.buf.as_ptr().cast()) };
                            return std::task::Poll::Ready(Err(Error::from_queue_err(crate::error::QueueError::Event(err_event))));
                        }
                        else {
                            return std::task::Poll::Ready(Err(error));
                        }
                    }
                    else {
                        println!("Will continue");
                        #[cfg(feature = "use-tokio")]
                        match _guard {
                            Some(mut guard) => {
                                match ev.eq {
                                    EqType::Write(e) => { if e.pending_cm_entries.borrow().is_empty() && e.pending_entries.borrow().is_empty() {guard.clear_ready()}},
                                    EqType::NoWrite(e) => { if e.pending_cm_entries.borrow().is_empty() && e.pending_entries.borrow().is_empty() {guard.clear_ready()}},
                                }
                            },
                            None => {}
                        }
                        continue;
                    }
                },
                Ok(len) => {println!("Read {} bytes, event {}", len, ev.event); return std::task::Poll::Ready(Ok(len))},
            }
        }
    }
}

impl<const WRITE: bool> EventQueue<AsyncEventQueueImpl<WRITE>> {

    pub(crate) fn new<T0>(fabric: &crate::fabric::Fabric, attr: EventQueueAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {

        Ok(
            Self {
                inner: Rc::new(
                    AsyncEventQueueImpl::new(&fabric.inner, attr, context)?
                ),
            })
    }
}


pub trait AsyncEventQueueImplT: EventQueueImplT + AsyncFid{
    fn read_in_async<'a>(&'a self, buf: &'a mut [u8], event: &'a mut u32) -> EqAsyncRead ;
    fn async_event_wait(&self, event_type: libfabric_sys::_bindgen_ty_18, req_fid: RawFid, ctx: usize) -> AsyncEventEq ;
}

impl AsyncEventQueueImpl<true> {
    pub(crate) async fn read_async(&self) -> Result<Event<usize>, crate::error::Error>{
        let mut buf = vec![0; std::mem::size_of::<libfabric_sys::fi_eq_err_entry>()];
        let mut event = 0;
        let len = self.read_in_async(&mut buf, &mut event).await?;
        Ok(self.base.get_ref().read_eq_entry(len, &buf, &event))
    }
}

impl AsyncEventQueueImpl<false> {
    pub(crate) async fn read_async(&self) -> Result<Event<usize>, crate::error::Error>{
        let mut buf = vec![0; std::mem::size_of::<libfabric_sys::fi_eq_err_entry>()];
        let mut event = 0;
        let len = self.read_in_async(&mut buf, &mut event).await?;
        Ok(self.base.get_ref().read_eq_entry(len, &buf, &event))
    }
}

impl AsyncEventQueueImplT for AsyncEventQueueImpl<true> {

    fn read_in_async<'a>(&'a self, buf: &'a mut [u8], event: &'a mut u32) -> EqAsyncRead {
        EqAsyncRead::new(buf, event, EqType::Write(self) )
    }

    fn async_event_wait(&self, event_type: libfabric_sys::_bindgen_ty_18, req_fid: RawFid, ctx: usize) -> AsyncEventEq {
        AsyncEventEq::new(event_type, req_fid, EqType::Write(self), ctx)
    } 
}

impl AsyncEventQueueImplT for AsyncEventQueueImpl<false> {

    fn read_in_async<'a>(&'a self, buf: &'a mut [u8], event: &'a mut u32) -> EqAsyncRead {
        EqAsyncRead::new(buf, event, EqType::NoWrite(self) )
    }

    fn async_event_wait(&self, event_type: libfabric_sys::_bindgen_ty_18, req_fid: RawFid, ctx: usize) -> AsyncEventEq {
        AsyncEventEq::new(event_type, req_fid, EqType::NoWrite(self), ctx)
    } 
}

pub struct AsyncEventQueueImpl<const WRITE: bool> {
    pub(crate) base: Async<EventQueueImpl<WRITE, true, true, true>>,
    pending_entries: RefCell<HashMap<usize, Result<Event<usize>, Error>>>,
    pending_err_entries: RefCell<HashMap<libfabric_sys::fid_t, Vec::<Result<Event<usize>, Error>>>>,
    pending_cm_entries: RefCell<HashMap<(u32,libfabric_sys::fid_t), Vec::<Result<Event<usize>, Error>>>>,

    // pub(crate) mrs: RefCell<std::collections::HashMap<RawFid, Weak<AsyncMemoryRegionImpl>>>,   // We neep maps Fid -> MemoryRegionImpl/AddressVectorImpl/MulticastGroupCollectiveImpl, however, we don't want to extend 
    // the lifetime of any of these objects just because of the maps.
    // Moreover, these objects will keep references to the EQ to keep it from dropping while
    // they are still bound, thus, we would have cyclic references that wouldn't let any 
    // of the two sides drop. 
    // pub(crate) avs: RefCell<std::collections::HashMap<RawFid, Weak<AsyncAddressVectorImpl>>>,
    // pub(crate) mcs: RefCell<std::collections::HashMap<RawFid, Weak<AsyncMulticastGroupCollectiveImpl>>>,
}


impl<const WRITE: bool> AsyncEventQueueImpl<WRITE> {
    pub(crate) fn new<T0>(fabric: &Rc<crate::fabric::FabricImpl>, attr: EventQueueAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        
        Ok(Self {
            base: Async::new(EventQueueImpl::new(fabric, attr, context)?).unwrap(),
            pending_entries: RefCell::new(HashMap::new()),
            pending_cm_entries: RefCell::new(HashMap::new()),
            pending_err_entries: RefCell::new(HashMap::new()),
            // mrs: RefCell::new(HashMap::new()),
            // avs: RefCell::new(HashMap::new()),
            // mcs: RefCell::new(HashMap::new()),
        })
    }
    // if let Some(mut entries) = ev.fut.eq.pending_cm_entries.borrow_mut().remove(&(ev.event_type, ev.req_fid)) {

    #[inline]
    pub(crate) fn insert_cm_entry(&self, event_type: libfabric_sys::_bindgen_ty_18, req_fid: RawFid, entry: Result<Event<usize>, Error>) {
        if let Some(vec) =  self.pending_cm_entries.borrow_mut().get_mut(&(event_type, req_fid)) {
            vec.push(entry);
        }
        else {
            self.pending_cm_entries.borrow_mut().insert((event_type, req_fid), vec![entry]);
        }
    }

    #[inline]
    pub(crate) fn insert_entry(&self, ctx: usize, entry: Result<Event<usize>, Error>) {
        self.pending_entries.borrow_mut().insert( ctx, entry);
    }

    #[inline]
    pub(crate) fn remove_cm_entry(&self, event_type: libfabric_sys::_bindgen_ty_18, req_fid: RawFid) -> Option<Result<Event<usize>, Error>>{
        match self.pending_cm_entries.borrow_mut().get_mut(&(event_type, req_fid)) {
            None => None, Some(vec) => vec.pop()
        }
    }

    #[inline]
    pub(crate) fn remove_entry(&self, ctx: usize) -> Option<Result<Event<usize>, Error>>{
        self.pending_entries.borrow_mut().remove(&ctx)
    }



    #[inline]
    pub(crate) fn insert_err_entry(&self, req_fid: RawFid, entry: Result<Event<usize>, Error>) {
        if let Some(vec) =  self.pending_err_entries.borrow_mut().get_mut(&req_fid) {
            vec.push(entry);
        }
        else {
            self.pending_err_entries.borrow_mut().insert(req_fid, vec![entry]);
        }
    }

    #[inline]
    pub(crate) fn remove_err_entry(&self, req_fid: RawFid) -> Option<Result<Event<usize>, Error>>{
        match self.pending_err_entries.borrow_mut().get_mut(&req_fid) {
            None => None, Some(vec) => vec.pop()
        }
    }

    // pub(crate) fn bind_mr(&self, mr: &Rc<AsyncMemoryRegionImpl>) {
    //     self.mrs.borrow_mut().insert(mr.as_raw_fid(), Rc::downgrade(mr));
    // }

    // pub(crate) fn bind_av(&self, av: &Rc<AsyncAddressVectorImpl>) {
    //     self.avs.borrow_mut().insert(av.as_raw_fid(), Rc::downgrade(av));
    // }

    // pub(crate) fn bind_mc(&self, mc: &Rc<AsyncMulticastGroupCollectiveImpl>) {
    //     self.mcs.borrow_mut().insert(mc.as_raw_fid(), Rc::downgrade(mc));
    // }

    pub(crate) fn get_inner(&self) -> &EventQueueImpl<WRITE, true, true, true> {
        self.base.get_ref()
    }
}

impl<const WRITE: bool> EventQueueImplT for AsyncEventQueueImpl<WRITE> {
    fn read(&self) -> Result<Event<usize>, crate::error::Error> {
        self.get_inner().read()
    }

    fn peek(&self) -> Result<Event<usize>, crate::error::Error> {
        self.get_inner().peek()
    }

    fn readerr(&self) -> Result<crate::eq::EventError, crate::error::Error> {
        self.get_inner().readerr()
    }

    fn peekerr(&self) -> Result<crate::eq::EventError, crate::error::Error> {
        self.get_inner().peekerr()
    }

    fn strerror(&self, entry: &crate::eq::EventError) -> &str {
        self.get_inner().strerror(entry)
    }
}

impl WritableEventQueueImplT for AsyncEventQueueImpl<true> {}
impl<'a, const WRITE: bool> WaitObjectRetrievable<'a> for AsyncEventQueueImpl<WRITE> {
    fn wait_object(&self) -> Result<crate::enums::WaitObjType<'a>, crate::error::Error> {
        self.get_inner().wait_object()
    }
}

// pub(crate) trait AsyncEventQueueImplT {
//     async fn read_async(&self) -> Result<Event<usize>, crate::error::Error>;
// }

// impl AsyncEventQueueImplT for AsyncEventQueueImpl {
    
//     async fn read_async(&self) -> Result<Event<usize>, crate::error::Error> {
//         self.read_async().await
//     }
// }


pub struct AsyncEventEq<'a>{
    pub(crate) req_fid: libfabric_sys::fid_t,
    pub(crate) ctx: usize,
    event_type: libfabric_sys::_bindgen_ty_18,
    fut: Pin<Box<EqAsyncReadOwned<'a>>>,
}

impl<'a> AsyncEventEq<'a> {
    fn new(event_type: libfabric_sys::_bindgen_ty_18, req_fid: libfabric_sys::fid_t, eq: EqType<'a>, ctx: usize) -> Self {
        Self {
            event_type,
            fut: Box::pin(EqAsyncReadOwned::new(eq)),
            req_fid,
            ctx,
        }
    }
}

impl<'a> Future for AsyncEventEq<'a> {
    type Output=Result<Event<usize>, Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let ev = self.get_mut();

        loop{
            if ev.event_type == libfabric_sys::FI_CONNREQ || ev.event_type == libfabric_sys::FI_CONNECTED || ev.event_type == libfabric_sys::FI_SHUTDOWN {
                if let Some(entry) = ev.fut.eq.remove_cm_entry(ev.event_type, ev.req_fid) {
                    return std::task::Poll::Ready(entry);
                }
            }
            else if let Some(mut entry) = ev.fut.eq.remove_entry(ev.ctx) {
                if ev.ctx != 0 {
                    if let Ok(ref mut entry) = entry {
                        match entry {
                            crate::eq::Event::MrComplete(ref mut e) 
                            | crate::eq::Event::AVComplete(ref mut e) 
                            | crate::eq::Event::JoinComplete(ref mut e) => {e.c_entry.context = unsafe{ ( *(e.c_entry.context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())}},
                            _ => panic!("Unexpected!"),
                        }
                    }
                    return std::task::Poll::Ready(entry);
                }
                else {
                    return std::task::Poll::Ready(entry);
                }
            }
            if let Some(err) = ev.fut.eq.remove_err_entry(ev.req_fid) {
                return std::task::Poll::Ready(err);
            }

            let mut res = match ready!(ev.fut.as_mut().poll(cx)) {
                // Ok(len) => len,
                Err(error) => {
                    if let ErrorKind::ErrorInQueue(ref q_err) = error.kind {
                        match q_err {
                            crate::error::QueueError::Event(_) => {
                                Err(error)
                            },
                            crate::error::QueueError::Completion(_) => todo!(), // Should never be the case
                        }
                    }
                    else {
                        return std::task::Poll::Ready(Err(error));
                    }
                },
                Ok(len) => {
                    println!("Read something {}", ev.fut.event);
                    Ok(ev.fut.eq.read_eq_entry(len, &ev.fut.buf, &ev.fut.event))
                }
            };
            

            match &mut res {
                Ok(entry) => {
                    match entry {

                        // crate::eq::Event::Notify(entry) | 
                        crate::eq::Event::MrComplete(e) => { if e.c_entry.context as usize == ev.ctx  { if ev.ctx != 0 {e.c_entry.context = unsafe{ ( *(e.c_entry.context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())}}; return std::task::Poll::Ready(res);  } else {ev.fut.eq.insert_entry(e.c_entry.context as usize,res);}},
                        crate::eq::Event::AVComplete(e) => { if e.c_entry.context as usize == ev.ctx  { if ev.ctx != 0 {e.c_entry.context = unsafe{ ( *(e.c_entry.context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())}}; return std::task::Poll::Ready(res);  } else {ev.fut.eq.insert_entry(e.c_entry.context as usize,res);}},
                        crate::eq::Event::JoinComplete(e) => { println!("Found join event"); if e.c_entry.context as usize == ev.ctx  { if ev.ctx != 0 {e.c_entry.context = unsafe{ ( *(e.c_entry.context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())}}; return std::task::Poll::Ready(res);  } else {ev.fut.eq.insert_entry(e.c_entry.context as usize, res);}},
                        crate::eq::Event::ConnReq(e) => {
                            if ev.event_type == libfabric_sys::FI_CONNREQ && ev.req_fid == e.c_entry.fid{
                                return std::task::Poll::Ready(res);
                            }
                            else {
                                ev.fut.eq.insert_cm_entry(libfabric_sys::FI_CONNREQ, ev.req_fid, res);
                            }
                        },
                        crate::eq::Event::Connected(e) => {
                            if ev.event_type == libfabric_sys::FI_CONNECTED && ev.req_fid == e.c_entry.fid {
                                return std::task::Poll::Ready(res);
                            }
                            else {
                                ev.fut.eq.insert_cm_entry(libfabric_sys::FI_CONNECTED, ev.req_fid, res);
                            }
                        },
                        crate::eq::Event::Shutdown(e) => { // [TODO] No one will explcitly look for shutdown requests, should probably store it somewhere else
                            if ev.event_type == libfabric_sys::FI_SHUTDOWN && ev.req_fid == e.c_entry.fid {
                                return std::task::Poll::Ready(res);
                            }
                            else {
                                ev.fut.eq.insert_cm_entry(libfabric_sys::FI_SHUTDOWN, ev.req_fid, res);
                            }
                        },
                    }
                }
                Err(err_entry) => {
                    match err_entry.kind {
                        ErrorKind::ErrorInQueue(ref err) => {
                            match err {
                                crate::error::QueueError::Completion(_) => todo!(),
                                crate::error::QueueError::Event(e) => {
                                    if ev.ctx != 0 {
                                        if e.c_err.context as usize == ev.ctx {
                                            return std::task::Poll::Ready(res);
                                        }
                                        else if e.c_err.context as usize == 0 {
                                            ev.fut.eq.insert_err_entry(e.c_err.fid, res)
                                        }
                                        else {
                                            ev.fut.eq.insert_entry(e.c_err.context as usize, res)
                                        }
                                    }
                                    else if e.c_err.context as usize == 0 {
                                        if e.c_err.fid == ev.req_fid {
                                            return std::task::Poll::Ready(res);
                                        }
                                        else {
                                            ev.fut.eq.insert_err_entry(e.c_err.fid, res)
                                        }
                                    }
                                    else {
                                        ev.fut.eq.insert_entry(e.c_err.context as usize, res)
                                    }
                                }
                            }
                        }
                        _ => panic!("Unexpected error"),
                    }
                }
            }

            ev.fut = Box::pin(EqAsyncReadOwned::new(ev.fut.eq.clone()));
        }

    }
}

impl<const WRITE: bool> AsFid for AsyncEventQueueImpl<WRITE> {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
       self.base.get_ref().c_eq.as_fid()
    }
}

impl<const WRITE: bool> AsFid for &AsyncEventQueueImpl<WRITE> {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
       self.base.get_ref().as_fid()
    }
}
impl<const WRITE: bool> AsFid for Rc<AsyncEventQueueImpl<WRITE>> {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
       self.base.get_ref().as_fid()
    }
}

impl<const WRITE: bool> AsRawFid for AsyncEventQueueImpl<WRITE> {
    fn as_raw_fid(&self) -> RawFid {
       self.base.get_ref().as_raw_fid()
    }
}

impl<const WRITE: bool> AsRawTypedFid for AsyncEventQueueImpl<WRITE> {
    type Output = EqRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
       self.base.get_ref().as_raw_typed_fid()
    }
    
}

impl<const WRITE: bool> crate::BindImpl for AsyncEventQueueImpl<WRITE> {}

// impl BindEqImpl<AsyncEventQueueImpl, AsyncCompletionQueueImpl> for AsyncEventQueueImpl {
//     fn bind_mr(&self, mr: &Rc<MemoryRegionImplBase<AsyncEventQueueImpl>>) {
//         self.bind_mr(mr);
//     }

//     fn bind_av(&self, av: &Rc<AddressVectorImplBase<AsyncEventQueueImpl>>) {
//         self.bind_av(av);
//     }

//     fn bind_mc(&self, mc: &Rc<MulticastGroupCollectiveImplBase<AsyncEventQueueImpl, AsyncCompletionQueueImpl>>) {
//         self.bind_mc(mc);
//     }
// }

pub struct EventQueueBuilder<'a, T, const WRITE: bool> {
    eq_attr: EventQueueAttr,
    fabric: &'a crate::fabric::Fabric,
    ctx: Option<&'a mut T>,
}

impl<'a> EventQueueBuilder<'a, (), false> {
    pub fn new(fabric: &'a crate::fabric::Fabric) -> Self {
       Self {
            eq_attr: EventQueueAttr::new(),
            fabric,
            ctx: None,
        }
    }
}

impl <'a, T, const WRITE: bool> EventQueueBuilder<'a, T, WRITE> {
    
    pub fn size(mut self, size: usize) -> Self {
        self.eq_attr.size(size);
        self
    }

    pub fn write(mut self) -> EventQueueBuilder<'a, T, true> {
        self.eq_attr.write();

        EventQueueBuilder {
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: self.ctx,
        }
    }
    
    pub fn signaling_vector(mut self, signaling_vector: i32) -> Self {
        self.eq_attr.signaling_vector(signaling_vector);
        self
    }

    pub fn context(self, ctx: &'a mut T) -> EventQueueBuilder<'a, T, WRITE> {
        EventQueueBuilder {
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: Some(ctx),
        }
    }

    pub fn build(mut self) ->  Result<EventQueue<AsyncEventQueueImpl<WRITE>>, crate::error::Error> {
        self.eq_attr.wait_obj(crate::enums::WaitObj::Fd);

        EventQueue::<AsyncEventQueueImpl<WRITE>>::new(self.fabric, self.eq_attr, self.ctx)   
    }
}



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
                .size(size)
                .build().unwrap();
            eqs.push(eq);
            println!("Count = {} \n", std::rc::Rc::strong_count(&fab.inner));
        }

        drop(fab);
        println!("Count = {} After dropping fab\n", std::rc::Rc::strong_count(&eqs[0].inner.base.get_ref()._fabric_rc));
    }
}