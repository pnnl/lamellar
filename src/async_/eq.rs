use std::{collections::HashMap, future::Future, pin::Pin, sync::atomic::Ordering, task::ready};

#[cfg(feature = "use-async-std")]
use async_io::Async;
#[cfg(feature = "use-tokio")]
use tokio::io::unix::AsyncFd as Async;
use std::sync::atomic::AtomicUsize;
use crate::{
    cq::WaitObjectRetrieve,
    eq::{Event, EventError, EventQueueAttr, EventQueueBase, EventQueueImpl, ReadEq, WriteEq},
    error::{Error, ErrorKind},
    fid::{AsTypedFid, BorrowedTypedFid, EqRawFid, Fid},
    Context, MyRc, MyRefCell, SyncSend,
};

use super::AsyncFid;

pub type EventQueue<T> = EventQueueBase<T>;

//pub(crate) trait  FdEq: ReadEq + AsRawFd{
//
//}

impl<const WRITE: bool> AsyncFid for AsyncEventQueueImpl<WRITE> {
    fn trywait(&self) -> Result<(), Error> {
        self.base.get_ref()._fabric_rc.trywait(self.base.get_ref())
    }
}

impl<EQ: AsyncFid> AsyncFid for EventQueue<EQ> {
    fn trywait(&self) -> Result<(), Error> {
        self.inner.trywait()
    }
}

#[derive(Clone)]
enum EqType<'a> {
    Write(&'a AsyncEventQueueImpl<true>),
    NoWrite(&'a AsyncEventQueueImpl<false>),
}

impl<'a> EqType<'a> {
    #[inline]
    pub(crate) fn trywait(&self) -> Result<(), Error> {
        match self {
            EqType::Write(e) => e.trywait(),
            EqType::NoWrite(e) => e.trywait(),
        }
    }

    #[inline]
    pub(crate) fn read_in(&self, buff: &mut [u8], event: &mut u32) -> Result<usize, Error> {
        match self {
            EqType::Write(e) => e.read_in(buff, event),
            EqType::NoWrite(e) => e.read_in(buff, event),
        }
    }

    #[inline]
    pub(crate) fn readerr_in(&self, buff: &mut [u8]) -> Result<usize, Error> {
        match self {
            EqType::Write(e) => e.readerr_in(buff),
            EqType::NoWrite(e) => e.readerr_in(buff),
        }
    }

    #[inline]
    pub(crate) fn remove_cm_entry(
        &self,
        event_type: libfabric_sys::_bindgen_ty_18,
        req_fid: &Fid,
    ) -> Option<Result<Event, Error>> {
        match self {
            EqType::Write(e) => e.remove_cm_entry(event_type, req_fid),
            EqType::NoWrite(e) => e.remove_cm_entry(event_type, req_fid),
        }
    }

    #[inline]
    pub(crate) fn insert_cm_entry(
        &self,
        event_type: libfabric_sys::_bindgen_ty_18,
        req_fid: &Fid,
        entry: Result<Event, Error>,
    ) {
        match self {
            EqType::Write(e) => e.insert_cm_entry(event_type, req_fid, entry),
            EqType::NoWrite(e) => e.insert_cm_entry(event_type, req_fid, entry),
        }
    }

    #[inline]
    pub(crate) fn remove_pending_entry(&self) {
        match self {
            EqType::Write(e) => e.remove_pending_entry(),
            EqType::NoWrite(e) => e.remove_pending_entry(),
        };
    }

    #[inline]
    pub(crate) fn insert_pending_entry(&self) {
        match self {
            EqType::Write(e) => e.insert_pending_entry(),
            EqType::NoWrite(e) => e.insert_pending_entry(),
        };
    }

    // #[inline]
    // pub(crate) fn remove_entry(&self, ctx: usize) -> Option<Result<Event, Error>> {
    //     match self {
    //         EqType::Write(e) => e.remove_entry(ctx),
    //         EqType::NoWrite(e) => e.remove_entry(ctx),
    //     }
    // }

    // #[inline]
    // pub(crate) fn insert_entry(&self, ctx: usize, entry: Result<Event, Error>) {
    //     match self {
    //         EqType::Write(e) => e.insert_entry(ctx, entry),
    //         EqType::NoWrite(e) => e.insert_entry(ctx, entry),
    //     }
    // }

    #[inline]
    pub(crate) fn read_eq_entry(&self, bytes_read: usize, buffer: &[u8], event: &u32) -> Event {
        EventQueueImpl::<false, true, true, true>::read_eq_entry(bytes_read, buffer, event)
    }

    #[inline]
    pub(crate) fn insert_err_entry(&self, req_fid: &Fid, entry: Result<Event, Error>) {
        match self {
            EqType::Write(e) => e.insert_err_entry(req_fid, entry),
            EqType::NoWrite(e) => e.insert_err_entry(req_fid, entry),
        }
    }

    #[inline]
    pub(crate) fn remove_err_entry(&self, req_fid: &Fid) -> Option<Result<Event, Error>> {
        match self {
            EqType::Write(e) => e.remove_err_entry(req_fid),
            EqType::NoWrite(e) => e.remove_err_entry(req_fid),
        }
    }

    #[inline]
    pub(crate) fn using_context2(&self) -> bool {
        match self {
            EqType::Write(e) => e.base.as_ref()._fabric_rc.using_context2,
            EqType::NoWrite(e) => e.base.as_ref()._fabric_rc.using_context2,
        }
    }
}

#[cfg(feature = "use-tokio")]
pub(crate) enum FutType<'a> {
    Write(
        Pin<
            Box<
                dyn Future<
                        Output = Result<
                            tokio::io::unix::AsyncFdReadyGuard<
                                'a,
                                EventQueueImpl<true, true, true, true>,
                            >,
                            std::io::Error,
                        >,
                    > + 'a,
            >,
        >,
    ),
    NoWrite(
        Pin<
            Box<
                dyn Future<
                        Output = Result<
                            tokio::io::unix::AsyncFdReadyGuard<
                                'a,
                                EventQueueImpl<false, true, true, true>,
                            >,
                            std::io::Error,
                        >,
                    > + 'a,
            >,
        >,
    ),
}

#[cfg(feature = "use-async-std")]
pub(crate) enum FutType<'a> {
    Write(Pin<Box<async_io::Readable<'a, EventQueueImpl<true, true, true, true>>>>),
    NoWrite(Pin<Box<async_io::Readable<'a, EventQueueImpl<false, true, true, true>>>>),
}

pub struct EqAsyncRead<'a> {
    buf: &'a mut [u8],
    event: &'a mut u32,
    eq: EqType<'a>,
    fut: Option<FutType<'a>>,
}

impl<'a> EqAsyncRead<'a> {
    fn new(buf: &'a mut [u8], event: &'a mut u32, eq: EqType<'a>) -> Self {
        Self {
            buf,
            event,
            eq,
            fut: None,
        }
    }
}
#[cfg(feature = "use-tokio")]
enum Guard<'a> {
    Write(tokio::io::unix::AsyncFdReadyGuard<'a, EventQueueImpl<true, true, true, true>>),
    NoWrite(tokio::io::unix::AsyncFdReadyGuard<'a, EventQueueImpl<false, true, true, true>>),
}

#[cfg(feature = "use-async-std")]
enum Guard {
    Write(()),
    NoWrite(()),
}

#[cfg(feature = "use-tokio")]
impl<'a> Guard<'a> {
    fn clear_ready(&mut self) {
        match self {
            Guard::Write(g) => g.clear_ready(),
            Guard::NoWrite(g) => g.clear_ready(),
        }
    }
}

impl<'a> Future for EqAsyncRead<'a> {
    type Output = Result<usize, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let ev = self.get_mut();
        loop {
            // println!("About to block waiting for event");
            let (err, _guard) = if ev.eq.trywait().is_err() {
                (ev.eq.read_in(ev.buf, ev.event), None)
            } else {
                if ev.fut.is_none() {
                    ev.fut = Some(match ev.eq {
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
                (ev.eq.read_in(ev.buf, ev.event), Some(_guard))
            };
            match err {
                Err(error) => {
                    if !matches!(error.kind, crate::error::ErrorKind::TryAgain) {
                        if matches!(error.kind, crate::error::ErrorKind::ErrorAvailable) {
                            let _len = ev.eq.readerr_in(ev.buf)?;
                            let mut err_event = EventError::new();
                            err_event.c_err = unsafe { std::ptr::read(ev.buf.as_ptr().cast()) };
                            return std::task::Poll::Ready(Err(Error::from_queue_err(
                                crate::error::QueueError::Event(err_event),
                            )));
                        }
                    } else {
                        // println!("Will continue");

                        #[cfg(feature = "use-tokio")]
                        if let Some(mut guard) = _guard {
                            match ev.eq {
                                #[cfg(feature = "thread-safe")]
                                EqType::Write(e) => {
                                    if e.pending_cm_entries.read().is_empty()
                                        && e.pending_entries.load(Ordering::SeqCst) == 0
                                    {
                                        guard.clear_ready()
                                    }
                                }
                                #[cfg(not(feature = "thread-safe"))]
                                EqType::Write(e) => {
                                    if e.pending_cm_entries.borrow().is_empty()
                                        && e.pending_entries.borrow().is_empty()
                                    {
                                        guard.clear_ready()
                                    }
                                }
                                #[cfg(feature = "thread-safe")]
                                EqType::NoWrite(e) => {
                                    if e.pending_cm_entries.read().is_empty()
                                        && e.pending_entries.load(Ordering::SeqCst) == 0
                                    {
                                        guard.clear_ready()
                                    }
                                }
                                #[cfg(not(feature = "thread-safe"))]
                                EqType::NoWrite(e) => {
                                    if e.pending_cm_entries.borrow().is_empty()
                                        && e.pending_entries.borrow().is_empty()
                                    {
                                        guard.clear_ready()
                                    }
                                }
                            }
                        }
                        continue;
                    }
                }
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

impl<'a> Future for EqAsyncReadOwned<'a> {
    type Output = Result<usize, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let ev = self.get_mut();
        loop {
            // println!("About to block waiting for event");
            let (err, _guard) = if ev.eq.trywait().is_err() {
                // println!("Could not block");
                (ev.eq.read_in(&mut ev.buf, &mut ev.event), None)
            } else {
                if ev.fut.is_none() {
                    ev.fut = Some(match ev.eq {
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

                // println!("Did not block");

                // We only need to reset the option to none
                #[allow(clippy::let_underscore_future)]
                let _ = ev.fut.take().unwrap();
                (ev.eq.read_in(&mut ev.buf, &mut ev.event), Some(_guard))
            };
            match err {
                Err(error) => {
                    if !matches!(error.kind, crate::error::ErrorKind::TryAgain) {
                        if matches!(error.kind, crate::error::ErrorKind::ErrorAvailable) {
                            let _len = ev.eq.readerr_in(&mut ev.buf)?;
                            let mut err_event = EventError::new();
                            err_event.c_err = unsafe { std::ptr::read(ev.buf.as_ptr().cast()) };
                            return std::task::Poll::Ready(Err(Error::from_queue_err(
                                crate::error::QueueError::Event(err_event),
                            )));
                        } else {
                            return std::task::Poll::Ready(Err(error));
                        }
                    } else {
                        // println!("Will continue");
                        #[cfg(feature = "use-tokio")]
                        match _guard {
                            Some(mut guard) => match ev.eq {
                                #[cfg(feature = "thread-safe")]
                                EqType::Write(e) => {
                                    if e.pending_cm_entries.read().is_empty()
                                        && e.pending_entries.load(Ordering::SeqCst) == 0
                                    {
                                        guard.clear_ready()
                                    }
                                }
                                #[cfg(not(feature = "thread-safe"))]
                                EqType::Write(e) => {
                                    if e.pending_cm_entries.borrow().is_empty()
                                        && e.pending_entries.borrow().is_empty()
                                    {
                                        guard.clear_ready()
                                    }
                                }
                                #[cfg(feature = "thread-safe")]
                                EqType::NoWrite(e) => {
                                    if e.pending_cm_entries.borrow().is_empty()
                                        && e.pending_entries.borrow().is_empty()
                                    {
                                        guard.clear_ready()
                                    }
                                }
                                #[cfg(not(feature = "thread-safe"))]
                                EqType::NoWrite(e) => {
                                    if e.pending_cm_entries.read().is_empty()
                                        && e.pending_entries.load(Ordering::SeqCst) == 0
                                    {
                                        guard.clear_ready()
                                    }
                                }
                            },
                            None => {}
                        }
                        continue;
                    }
                }
                Ok(len) => return std::task::Poll::Ready(Ok(len)),
            }
        }
    }
}

impl<const WRITE: bool> EventQueue<AsyncEventQueueImpl<WRITE>> {
    pub(crate) fn new(
        fabric: &crate::fabric::Fabric,
        attr: EventQueueAttr,
        context: Option<&mut Context>,
    ) -> Result<Self, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(Self {
            inner: MyRc::new(AsyncEventQueueImpl::new(&fabric.inner, attr, c_void)?),
        })
    }
}

pub trait AsyncReadEq: ReadEq + AsyncFid {
    fn read_in_async<'a>(&'a self, buf: &'a mut [u8], event: &'a mut u32) -> EqAsyncRead;
    fn async_event_wait<'a>(
        &'a self,
        event_type: libfabric_sys::_bindgen_ty_18,
        req_fid: Fid,
        ctx: Option<&'a mut crate::Context>,
    ) -> AsyncEventEq<'a>;
}

impl AsyncEventQueueImpl<true> {
    #[allow(dead_code)]
    pub(crate) async fn read_async(&self) -> Result<Event, crate::error::Error> {
        let mut buf = vec![0; std::mem::size_of::<libfabric_sys::fi_eq_err_entry>()];
        let mut event = 0;
        let len = self.read_in_async(&mut buf, &mut event).await?;
        Ok(EventQueueImpl::<true, true, true, true>::read_eq_entry(
            len, &buf, &event,
        ))
    }
}

impl AsyncEventQueueImpl<false> {
    #[allow(dead_code)]
    pub(crate) async fn read_async(&self) -> Result<Event, crate::error::Error> {
        let mut buf = vec![0; std::mem::size_of::<libfabric_sys::fi_eq_err_entry>()];
        let mut event = 0;
        let len = self.read_in_async(&mut buf, &mut event).await?;
        Ok(EventQueueImpl::<false, true, true, true>::read_eq_entry(
            len, &buf, &event,
        ))
    }
}

impl AsyncReadEq for AsyncEventQueueImpl<true> {
    fn read_in_async<'a>(&'a self, buf: &'a mut [u8], event: &'a mut u32) -> EqAsyncRead {
        EqAsyncRead::new(buf, event, EqType::Write(self))
    }

    fn async_event_wait<'a>(
        &'a self,
        event_type: libfabric_sys::_bindgen_ty_18,
        req_fid: Fid,
        ctx: Option<&'a mut crate::Context>,
    ) -> AsyncEventEq<'a> {
        AsyncEventEq::new(event_type, req_fid, EqType::Write(self), ctx)
    }
}

impl AsyncReadEq for AsyncEventQueueImpl<false> {
    fn read_in_async<'a>(&'a self, buf: &'a mut [u8], event: &'a mut u32) -> EqAsyncRead {
        EqAsyncRead::new(buf, event, EqType::NoWrite(self))
    }

    fn async_event_wait<'a>(
        &'a self,
        event_type: libfabric_sys::_bindgen_ty_18,
        req_fid: Fid,
        ctx: Option<&'a mut crate::Context>,
    ) -> AsyncEventEq<'a> {
        AsyncEventEq::new(event_type, req_fid, EqType::NoWrite(self), ctx)
    }
}

impl<EQ: AsyncReadEq> AsyncReadEq for EventQueue<EQ> {
    fn read_in_async<'a>(&'a self, buf: &'a mut [u8], event: &'a mut u32) -> EqAsyncRead {
        self.inner.read_in_async(buf, event)
    }

    fn async_event_wait<'a>(
        &'a self,
        event_type: libfabric_sys::_bindgen_ty_18,
        req_fid: Fid,
        ctx: Option<&'a mut crate::Context>,
    ) -> AsyncEventEq<'a> {
        self.inner.async_event_wait(event_type, req_fid, ctx)
    }
}

impl<EQ: AsyncReadEq> EventQueue<EQ> {
    pub async fn read_async(&self) -> Result<Event, crate::error::Error> {
        let mut buf = vec![0; std::mem::size_of::<libfabric_sys::fi_eq_err_entry>()];
        let mut event = 0;
        let len = self.inner.read_in_async(&mut buf, &mut event).await?;
        Ok(EventQueueImpl::<true, true, true, true>::read_eq_entry(
            len, &buf, &event,
        ))
    }
}

pub struct AsyncEventQueueImpl<const WRITE: bool> {
    pub(crate) base: Async<EventQueueImpl<WRITE, true, true, true>>,
    pending_entries: AtomicUsize,
    pending_err_entries: MyRefCell<HashMap<Fid, Vec<Result<Event, Error>>>>,
    #[allow(clippy::type_complexity)]
    pending_cm_entries: MyRefCell<HashMap<(u32, Fid), Vec<Result<Event, Error>>>>,
}

impl<const WRITE: bool> SyncSend for AsyncEventQueueImpl<WRITE> {}

impl<const WRITE: bool> AsyncEventQueueImpl<WRITE> {
    pub(crate) fn new(
        fabric: &MyRc<crate::fabric::FabricImpl>,
        attr: EventQueueAttr,
        context: *mut std::ffi::c_void,
    ) -> Result<Self, crate::error::Error> {
        Ok(Self {
            base: Async::new(EventQueueImpl::new(fabric, attr, context)?).unwrap(),
            pending_entries:AtomicUsize::new(0),
            pending_cm_entries: MyRefCell::new(HashMap::new()),
            pending_err_entries: MyRefCell::new(HashMap::new()),
        })
    }

    #[inline]
    pub(crate) fn insert_cm_entry(
        &self,
        event_type: libfabric_sys::_bindgen_ty_18,
        req_fid: &Fid,
        entry: Result<Event, Error>,
    ) {
        #[cfg(feature = "thread-safe")]
        let mut vec = self.pending_cm_entries.write();
        #[cfg(not(feature = "thread-safe"))]
        let mut vec = self.pending_cm_entries.borrow_mut();
        if let Some(vec) = vec.get_mut(&(event_type, *req_fid)) {
            vec.push(entry);
        } else {
            #[cfg(feature = "thread-safe")]
            self.pending_cm_entries
                .write()
                .insert((event_type, *req_fid), vec![entry]);
            #[cfg(not(feature = "thread-safe"))]
            self.pending_cm_entries
                .borrow_mut()
                .insert((event_type, *req_fid), vec![entry]);
        }
    }

    // #[inline]
    // pub(crate) fn insert_entry(&self, ctx: usize, entry: Result<Event, Error>) {
    //     #[cfg(feature = "thread-safe")]
    //     self.pending_entries.write().insert(ctx, entry);
    //     #[cfg(not(feature = "thread-safe"))]
    //     self.pending_entries.borrow_mut().insert(ctx, entry);
    // }

    #[inline]
    pub(crate) fn remove_cm_entry(
        &self,
        event_type: libfabric_sys::_bindgen_ty_18,
        req_fid: &Fid,
    ) -> Option<Result<Event, Error>> {
        #[cfg(feature = "thread-safe")]
        let mut res = self.pending_cm_entries.write();
        #[cfg(not(feature = "thread-safe"))]
        let mut res = self.pending_cm_entries.borrow_mut();
        match res.get_mut(&(event_type, *req_fid)) {
            None => None,
            Some(vec) => vec.pop(),
        }
    }

    #[inline]
    pub(crate) fn remove_pending_entry(&self) {
        self.pending_entries.fetch_sub(1, Ordering::SeqCst);
    }

    #[inline]
    pub(crate) fn insert_pending_entry(&self) {
        self.pending_entries.fetch_add(1, Ordering::SeqCst);
    }

    // #[inline]
    // pub(crate) fn remove_entry(&self, ctx: usize) -> Option<Result<Event, Error>> {
    //     #[cfg(feature = "thread-safe")]
    //     {
    //         self.pending_entries.write().remove(&ctx)
    //     }

    //     #[cfg(not(feature = "thread-safe"))]
    //     {
    //         self.pending_entries.borrow_mut().remove(&ctx)
    //     }
    // }

    #[inline]
    pub(crate) fn insert_err_entry(&self, req_fid: &Fid, entry: Result<Event, Error>) {
        #[cfg(feature = "thread-safe")]
        let mut res = self.pending_err_entries.write();
        #[cfg(not(feature = "thread-safe"))]
        let mut res = self.pending_err_entries.borrow_mut();
        if let Some(vec) = res.get_mut(req_fid) {
            vec.push(entry);
        } else {
            #[cfg(feature = "thread-safe")]
            self.pending_err_entries
                .write()
                .insert(*req_fid, vec![entry]);
            #[cfg(not(feature = "thread-safe"))]
            self.pending_err_entries
                .borrow_mut()
                .insert(*req_fid, vec![entry]);
        }
    }

    #[inline]
    pub(crate) fn remove_err_entry(&self, req_fid: &Fid) -> Option<Result<Event, Error>> {
        #[cfg(feature = "thread-safe")]
        let mut res = self.pending_err_entries.write();
        #[cfg(not(feature = "thread-safe"))]
        let mut res = self.pending_err_entries.borrow_mut();
        match res.get_mut(req_fid) {
            None => None,
            Some(vec) => vec.pop(),
        }
    }

    // pub(crate) fn bind_mr(&self, mr: &MyRc<AsyncMemoryRegionImpl>) {
    //     self.mrs.borrow_mut().insert(mr.as_raw_fid(), MyRc::downgrade(mr));
    // }

    // pub(crate) fn bind_av(&self, av: &MyRc<AsyncAddressVectorImpl>) {
    //     self.avs.borrow_mut().insert(av.as_raw_fid(), MyRc::downgrade(av));
    // }

    // pub(crate) fn bind_mc(&self, mc: &MyRc<AsyncMulticastGroupCollectiveImpl>) {
    //     self.mcs.borrow_mut().insert(mc.as_raw_fid(), MyRc::downgrade(mc));
    // }

    pub(crate) fn get_inner(&self) -> &EventQueueImpl<WRITE, true, true, true> {
        self.base.get_ref()
    }
}

impl<const WRITE: bool> ReadEq for AsyncEventQueueImpl<WRITE> {
    fn read(&self) -> Result<Event, crate::error::Error> {
        self.get_inner().read()
    }

    fn peek(&self) -> Result<Event, crate::error::Error> {
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

impl WriteEq for AsyncEventQueueImpl<true> {}
impl<'a, const WRITE: bool> WaitObjectRetrieve<'a> for AsyncEventQueueImpl<WRITE> {
    fn wait_object(&self) -> Result<crate::enums::WaitObjType<'a>, crate::error::Error> {
        self.get_inner().wait_object()
    }
}

// pub(crate) trait AsyncEventQueueImplT {
//     async fn read_async(&self) -> Result<Event, crate::error::Error>;
// }

// impl AsyncEventQueueImplT for AsyncEventQueueImpl {

//     async fn read_async(&self) -> Result<Event, crate::error::Error> {
//         self.read_async().await
//     }
// }

pub struct AsyncEventEq<'a> {
    pub(crate) req_fid: Fid,
    pub(crate) ctx: Option<&'a mut crate::Context>,
    event_type: libfabric_sys::_bindgen_ty_18,
    fut: Pin<Box<EqAsyncReadOwned<'a>>>,
}

impl<'a> AsyncEventEq<'a> {
    fn new(
        event_type: libfabric_sys::_bindgen_ty_18,
        req_fid: Fid,
        eq: EqType<'a>,
        ctx: Option<&'a mut crate::Context>,
    ) -> Self {
        Self {
            event_type,
            fut: Box::pin(EqAsyncReadOwned::new(eq)),
            req_fid,
            ctx,
        }
    }
}

impl<'a> Future for AsyncEventEq<'a> {
    type Output = Result<Event, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let ev = self.get_mut();
        let using_ctx2 = ev.fut.eq.using_context2();
        loop {
            let ctx_id = match &mut ev.ctx {
                None => {
                    if ev.event_type == libfabric_sys::FI_CONNREQ
                    || ev.event_type == libfabric_sys::FI_CONNECTED
                    || ev.event_type == libfabric_sys::FI_SHUTDOWN
                    {
                        if let Some(entry) = ev.fut.eq.remove_cm_entry(ev.event_type, &ev.req_fid) {
                            return std::task::Poll::Ready(entry);
                        }
                        else if let Some(err_entry) = ev.fut.eq.remove_err_entry(&ev.req_fid) {
                            return std::task::Poll::Ready(err_entry);
                        }
                    } 
                    0
                }
                Some(ctx) => {
                    if ctx.ready() {
                        let state = ctx.state().take();
                        ctx.reset();
                        ev.fut.eq.remove_pending_entry();
                        match state {
                            Some(state) => {
                                match state {
                                    crate::ContextState::Cq(_) => panic!("Should never find completions here"),
                                    crate::ContextState::Eq(event) => return std::task::Poll::Ready(event),
                                }
                            },
                            None => {panic!("Should always be set when context is ready")},
                        }
                    }
                    ctx.inner() as usize
                }
            };

            let mut res = match ready!(ev.fut.as_mut().poll(cx)) {
                // Ok(len) => len,
                Err(error) => {
                    if let ErrorKind::ErrorInQueue(ref q_err) = error.kind {
                        match q_err {
                            crate::error::QueueError::Event(_) => Err(error),
                            crate::error::QueueError::Completion(_) => todo!(), // Should never be the case
                        }
                    } else {
                        return std::task::Poll::Ready(Err(error));
                    }
                }
                Ok(len) => {
                    // println!("Read something {}", ev.fut.event);
                    Ok(ev.fut.eq.read_eq_entry(len, &ev.fut.buf, &ev.fut.event))
                }
            };

            match res {
                Ok(entry) => {
                    match entry {
                        // crate::eq::Event::Notify(entry) |
                        crate::eq::Event::MrComplete(ref e) => {
                            if e.c_entry.context as usize == ctx_id {
                                return std::task::Poll::Ready(Ok(entry));
                            } else {
                                ev.fut.eq.insert_pending_entry();

                                if using_ctx2 {
                                    unsafe{(e.c_entry.context as *mut std::ffi::c_void  as *mut crate::Context2).as_mut().unwrap()}.set_event_done(Ok(entry));
                                }
                                else {
                                    unsafe{(e.c_entry.context as *mut std::ffi::c_void  as *mut crate::Context1).as_mut().unwrap()}.set_event_done(Ok(entry));
                                }
                            }
                        }
                        crate::eq::Event::AVComplete(ref e) => {
                            if e.c_entry.context as usize == ctx_id {
                                return std::task::Poll::Ready(Ok(entry));
                            } else {
                                ev.fut.eq.insert_pending_entry();

                                if using_ctx2 {
                                    unsafe{(e.c_entry.context as *mut std::ffi::c_void  as *mut crate::Context2).as_mut().unwrap()}.set_event_done(Ok(entry));
                                }
                                else {
                                    unsafe{(e.c_entry.context as *mut std::ffi::c_void  as *mut crate::Context1).as_mut().unwrap()}.set_event_done(Ok(entry));
                                }
                            }
                        }
                        crate::eq::Event::JoinComplete(ref e) => {
                            if e.c_entry.context as usize == ctx_id {
                                return std::task::Poll::Ready(Ok(entry));
                            } else {
                                ev.fut.eq.insert_pending_entry();
                                if using_ctx2 {
                                    unsafe{(e.c_entry.context as *mut std::ffi::c_void  as *mut crate::Context2).as_mut().unwrap()}.set_event_done(Ok(entry));
                                }
                                else {
                                    unsafe{(e.c_entry.context as *mut std::ffi::c_void  as *mut crate::Context1).as_mut().unwrap()}.set_event_done(Ok(entry));
                                }
                            }
                        }
                        crate::eq::Event::ConnReq(ref e) => {
                            if ev.event_type == libfabric_sys::FI_CONNREQ
                                && ev.req_fid.0 == e.c_entry.fid as usize
                            {
                                return std::task::Poll::Ready(Ok(entry));
                            } else {
                                ev.fut.eq.insert_cm_entry(
                                    libfabric_sys::FI_CONNREQ,
                                    &ev.req_fid,
                                    Ok(entry),
                                );
                            }
                        }
                        crate::eq::Event::Connected(ref e) => {
                            if ev.event_type == libfabric_sys::FI_CONNECTED
                                && ev.req_fid.0 == e.c_entry.fid as usize
                            {
                                return std::task::Poll::Ready(Ok(entry));
                            } else {
                                ev.fut.eq.insert_cm_entry(
                                    libfabric_sys::FI_CONNECTED,
                                    &ev.req_fid,
                                    Ok(entry),
                                );
                            }
                        }
                        crate::eq::Event::Shutdown(ref e) => {
                            // [TODO] No one will explcitly look for shutdown requests, should probably store it somewhere else
                            if ev.event_type == libfabric_sys::FI_SHUTDOWN
                                && ev.req_fid.0 == e.c_entry.fid as usize
                            {
                                return std::task::Poll::Ready(Ok(entry));
                            } else {
                                ev.fut.eq.insert_cm_entry(
                                    libfabric_sys::FI_SHUTDOWN,
                                    &ev.req_fid,
                                    Ok(entry),
                                );
                            }
                        }
                    }
                }
                Err(err_entry) => match err_entry.kind {
                    ErrorKind::ErrorInQueue(ref err) => match err {
                        crate::error::QueueError::Completion(_) => todo!(),
                        crate::error::QueueError::Event(e) => {
                            if ctx_id != 0 {
                                if e.c_err.context as usize == ctx_id {
                                    return std::task::Poll::Ready(Err(err_entry));
                                } else if e.c_err.context as usize == 0 {
                                    ev.fut.eq.insert_err_entry(&Fid(e.c_err.fid as usize), Err(err_entry))
                                } else {
                                    ev.fut.eq.insert_pending_entry();

                                    if using_ctx2 {
                                        unsafe{(e.c_err.context as *mut std::ffi::c_void  as *mut crate::Context2).as_mut().unwrap()}.set_event_done(Err(err_entry));
                                    }
                                    else {
                                        unsafe{(e.c_err.context as *mut std::ffi::c_void  as *mut crate::Context1).as_mut().unwrap()}.set_event_done(Err(err_entry));
                                    }
                                }
                            } else if e.c_err.context as usize == 0 {
                                if e.c_err.fid as usize== ev.req_fid.0  {
                                    return std::task::Poll::Ready(Err(err_entry));
                                } else {
                                    ev.fut.eq.insert_err_entry(&Fid(e.c_err.fid as usize), Err(err_entry))
                                }
                            } else {
                                ev.fut.eq.insert_pending_entry();

                                if using_ctx2 {
                                    unsafe{(e.c_err.context as *mut std::ffi::c_void  as *mut crate::Context2).as_mut().unwrap()}.set_event_done(Err(err_entry));
                                }
                                else {
                                    unsafe{(e.c_err.context as *mut std::ffi::c_void  as *mut crate::Context1).as_mut().unwrap()}.set_event_done(Err(err_entry));
                                }
                            }
                        }
                    },
                    _ => panic!("Unexpected error"),
                },
            }

            ev.fut = Box::pin(EqAsyncReadOwned::new(ev.fut.eq.clone()));
        }
    }
}

// impl<const WRITE: bool> AsFid for AsyncEventQueueImpl<WRITE> {
//     fn as_fid(&self) -> fid::BorrowedFid<'_> {
//         self.base.get_ref().c_eq.as_fid()
//     }
// }

// impl<const WRITE: bool> AsFid for &AsyncEventQueueImpl<WRITE> {
//     fn as_fid(&self) -> fid::BorrowedFid<'_> {
//         self.base.get_ref().as_fid()
//     }
// }
// impl<const WRITE: bool> AsFid for MyRc<AsyncEventQueueImpl<WRITE>> {
//     fn as_fid(&self) -> fid::BorrowedFid<'_> {
//         self.base.get_ref().as_fid()
//     }
// }

// impl<const WRITE: bool> AsRawFid for AsyncEventQueueImpl<WRITE> {
//     fn as_raw_fid(&self) -> RawFid {
//         self.base.get_ref().as_raw_fid()
//     }
// }

impl<const WRITE: bool> AsTypedFid<EqRawFid> for AsyncEventQueueImpl<WRITE> {

    fn as_typed_fid(&self) -> BorrowedTypedFid<EqRawFid> {
        self.base.get_ref().as_typed_fid()
    }
    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<EqRawFid> {
        self.base.get_ref().as_typed_fid_mut()
    }
}

// impl<const WRITE: bool> crate::BindImpl for AsyncEventQueueImpl<WRITE> {}

// impl BindEqImpl<AsyncEventQueueImpl, AsyncCompletionQueueImpl> for AsyncEventQueueImpl {
//     fn bind_mr(&self, mr: &MyRc<MemoryRegionImplBase<AsyncEventQueueImpl>>) {
//         self.bind_mr(mr);
//     }

//     fn bind_av(&self, av: &MyRc<AddressVectorImplBase<AsyncEventQueueImpl>>) {
//         self.bind_av(av);
//     }

//     fn bind_mc(&self, mc: &MyRc<MulticastGroupCollectiveImplBase<AsyncEventQueueImpl, AsyncCompletionQueueImpl>>) {
//         self.bind_mc(mc);
//     }
// }

pub struct EventQueueBuilder<'a, const WRITE: bool> {
    eq_attr: EventQueueAttr,
    fabric: &'a crate::fabric::Fabric,
    ctx: Option<&'a mut Context>,
}

impl<'a> EventQueueBuilder<'a, false> {
    pub fn new(fabric: &'a crate::fabric::Fabric) -> Self {
        Self {
            eq_attr: EventQueueAttr::new(),
            fabric,
            ctx: None,
        }
    }
}

impl<'a, const WRITE: bool> EventQueueBuilder<'a, WRITE> {
    pub fn size(mut self, size: usize) -> Self {
        self.eq_attr.size(size);
        self
    }

    pub fn write(mut self) -> EventQueueBuilder<'a, true> {
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

    pub fn context(self, ctx: &'a mut Context) -> EventQueueBuilder<'a, WRITE> {
        EventQueueBuilder {
            eq_attr: self.eq_attr,
            fabric: self.fabric,
            ctx: Some(ctx),
        }
    }

    pub fn build(mut self) -> Result<EventQueue<AsyncEventQueueImpl<WRITE>>, crate::error::Error> {
        self.eq_attr.wait_obj(crate::enums::WaitObj::Fd);

        EventQueue::<AsyncEventQueueImpl<WRITE>>::new(self.fabric, self.eq_attr, self.ctx)
    }
}

#[cfg(test)]
mod tests {

    use crate::info::{Info, Version};

    use super::EventQueueBuilder;

    // #[test]
    // fn eq_write_read_self() {
    //     let info = Info::new().build().unwrap();
    // let entry = info.into_iter().next().unwrap();
    //     let fab = crate::fabric::FabricBuilder::new(&entry).build().unwrap();
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
    //     let info = Info::new().build().unwrap();
    //     let entries = info.get();
    //     let fab = crate::fabric::FabricBuilder::new(&entry).build().unwrap();
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
    //     let info = Info::new().build().unwrap();
    //     let entries = info.get();
    //     let fab = crate::fabric::FabricBuilder::new(&entry).build().unwrap();
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
    //     let info = Info::new().build().unwrap();
    //     let entries = info.get();
    //     let fab = crate::fabric::FabricBuilder::new(&entry).build().unwrap();
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
        let info = Info::new(&Version {
            major: 1,
            minor: 19,
        })
        .get()
        .unwrap();
        let entry = info.into_iter().next().unwrap();

        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        for i in -1..17 {
            let size = if i == -1 { 0 } else { 1 << i };
            let _eq = EventQueueBuilder::new(&fab).size(size).build().unwrap();
        }
    }
}

#[cfg(test)]
mod libfabric_lifetime_tests {

    use crate::info::{Info, Version};

    use super::EventQueueBuilder;

    #[test]
    fn eq_drops_before_fabric() {
        let info = Info::new(&Version {
            major: 1,
            minor: 19,
        })
        .get()
        .unwrap();
        let entry = info.into_iter().next().unwrap();

        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let mut eqs = Vec::new();
        for i in -1..17 {
            let size = if i == -1 { 0 } else { 1 << i };
            let eq = EventQueueBuilder::new(&fab).size(size).build().unwrap();
            eqs.push(eq);
        }
        drop(fab);
    }
}
