use crate::cq::ReadCq;
use crate::cq::WaitCq;
use crate::cq::WaitObjectRetrieve;
use crate::cq::{CompletionEntry, SingleCompletion};
use crate::domain::{DomainBase, DomainImplBase};
use crate::error::ErrorKind;
use crate::fid::AsTypedFid;
use crate::fid::BorrowedTypedFid;
use crate::fid::CqRawFid;
use crate::SyncSend;
use crate::{
    cq::{
        Completion, CompletionError, CompletionQueueAttr, CompletionQueueBase, CompletionQueueImpl,
        CtxEntry, DataEntry, MsgEntry, TaggedEntry,
    },
    enums::WaitObjType,
    error::Error,
    fid::{ AsRawFid},
    MappedAddress,
};
use crate::{Context, MyRc, MyRefCell};
#[cfg(feature = "use-async-std")]
use async_io::{Async, Readable};
use std::collections::HashMap;
use std::os::fd::BorrowedFd;
use std::pin::Pin;
use std::{future::Future, task::ready};
#[cfg(feature = "use-tokio")]
use tokio::io::unix::AsyncFd as Async;

use super::AsyncFid;
macro_rules! alloc_cq_entry {
    ($format: expr, $count: expr) => {
        match $format {
            Completion::Ctx(_) => {
                let entries: Vec<CompletionEntry<CtxEntry>> = Vec::with_capacity($count);
                Completion::Ctx(entries)
            }
            Completion::Data(_) => {
                let entries: Vec<CompletionEntry<DataEntry>> = Vec::with_capacity($count);
                Completion::Data(entries)
            }
            Completion::Tagged(_) => {
                let entries: Vec<CompletionEntry<TaggedEntry>> = Vec::with_capacity($count);
                Completion::Tagged(entries)
            }
            Completion::Msg(_) => {
                let entries: Vec<CompletionEntry<MsgEntry>> = Vec::with_capacity($count);
                Completion::Msg(entries)
            }
            Completion::Unspec(_) => {
                let entries: Vec<CompletionEntry<CtxEntry>> = Vec::with_capacity($count);

                Completion::Unspec(entries)
            }
        }
    };
}

pub type CompletionQueue<T> = CompletionQueueBase<T>;

pub trait AsyncReadCq: ReadCq {
    fn read_in_async<'a>(&'a self, buf: &'a mut Completion, count: usize) -> CqAsyncRead;
    fn read_async(&self, count: usize) -> CqAsyncReadOwned;
    fn wait_for_ctx_async(&self, ctx: &mut Context) -> AsyncTransferCq;
}

impl CompletionQueue<AsyncCompletionQueueImpl> {
    pub(crate) fn new<EQ: ?Sized + 'static + SyncSend>(
        domain: &DomainBase<EQ>,
        attr: CompletionQueueAttr,
        context: Option<&mut Context>,
        default_buff_size: usize,
    ) -> Result<Self, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(Self {
            inner: MyRc::new(AsyncCompletionQueueImpl::new(
                &domain.inner,
                attr,
                c_void,
                default_buff_size,
            )?),
        })
    }
}

impl ReadCq for AsyncCompletionQueueImpl {
    fn read(&self, count: usize) -> Result<Completion, crate::error::Error> {
        #[cfg(feature = "thread-safe")]
        let mut borrowed_entries = self.base.get_ref().entry_buff.write();
        #[cfg(not(feature = "thread-safe"))]
        let mut borrowed_entries = self.base.get_ref().entry_buff.borrow_mut();
        self.read_in(count, &mut borrowed_entries)?;
        Ok(borrowed_entries.clone())
    }

    fn readfrom(
        &self,
        count: usize,
    ) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> {
        #[cfg(feature = "thread-safe")]
        let mut borrowed_entries = self.base.get_ref().entry_buff.write();
        #[cfg(not(feature = "thread-safe"))]
        let mut borrowed_entries = self.base.get_ref().entry_buff.borrow_mut();
        let address = self.readfrom_in(count, &mut borrowed_entries)?;
        Ok((borrowed_entries.clone(), address))
    }

    fn readerr(&self, flags: u64) -> Result<CompletionError, crate::error::Error> {
        #[cfg(feature = "thread-safe")]
        let mut entry = self.base.get_ref().error_buff.write();
        #[cfg(not(feature = "thread-safe"))]
        let mut entry = self.base.get_ref().error_buff.borrow_mut();
        self.readerr_in(&mut entry, flags)?;
        Ok(entry.clone())
    }
    
    fn fid(&self) -> &crate::fid::OwnedCqFid {
        &self.base.as_ref().c_cq
    }
}

impl<'a> WaitObjectRetrieve<'a> for AsyncCompletionQueueImpl {
    fn wait_object(&self) -> Result<WaitObjType<'a>, crate::error::Error> {
        if let Some(wait) = self.base.get_ref().wait_obj {
            if wait == libfabric_sys::fi_wait_obj_FI_WAIT_FD {
                let mut fd: i32 = 0;
                let err = unsafe {
                    libfabric_sys::inlined_fi_control(
                        self.as_typed_fid_mut().as_raw_fid(),
                        libfabric_sys::FI_GETWAIT as i32,
                        (&mut fd as *mut i32).cast(),
                    )
                };
                if err < 0 {
                    Err(crate::error::Error::from_err_code(
                        (-err).try_into().unwrap(),
                    ))
                } else {
                    Ok(WaitObjType::Fd(unsafe { BorrowedFd::borrow_raw(fd) }))
                }
            } else {
                panic!("Unexpected value for wait object in AsyncCompletionQueue");
            }
        } else {
            panic!("Unexpected value for wait object in AsyncCompletionQueue");
        }
    }
}

// impl AsRawTypedFid for AsyncCompletionQueueImpl {
//     type Output = CqRawFid;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.base.get_ref().as_raw_typed_fid()
//     }
// }

pub struct AsyncCompletionQueueImpl {
    pub(crate) base: Async<CompletionQueueImpl<true, true, true>>,
    pub(crate) pending_entries: MyRefCell<HashMap<usize, Result<SingleCompletion, Error>>>,
}
impl SyncSend for AsyncCompletionQueueImpl {}

impl WaitCq for AsyncCompletionQueueImpl {
    fn sread_with_cond(
        &self,
        count: usize,
        cond: usize,
        timeout: i32,
    ) -> Result<Completion, crate::error::Error> {
        self.base.get_ref().sread_with_cond(count, cond, timeout)
    }

    fn sreadfrom_with_cond(
        &self,
        count: usize,
        cond: usize,
        timeout: i32,
    ) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> {
        self.base
            .get_ref()
            .sreadfrom_with_cond(count, cond, timeout)
    }
}

impl AsyncReadCq for AsyncCompletionQueueImpl {
    fn read_in_async<'a>(&'a self, buf: &'a mut Completion, count: usize) -> CqAsyncRead {
        CqAsyncRead {
            num_entries: count,
            buf,
            cq: self,
            fut: None,
        }
    }

    fn read_async(&self, count: usize) -> CqAsyncReadOwned {
        CqAsyncReadOwned::new(count, self)
    }

    fn wait_for_ctx_async(&self, ctx: &mut Context) -> AsyncTransferCq {
        AsyncTransferCq::new(self, ctx.inner_mut() as usize)
    }
}

impl AsyncFid for AsyncCompletionQueueImpl {
    fn trywait(&self) -> Result<(), Error> {
        self.base
            .get_ref()
            ._domain_rc
            .get_fabric_impl()
            .trywait(self.base.get_ref())
    }
}

impl<T: AsyncReadCq> AsyncReadCq for CompletionQueue<T> {
    fn read_in_async<'a>(&'a self, buf: &'a mut Completion, count: usize) -> CqAsyncRead {
        self.inner.read_in_async(buf, count)
    }

    fn read_async(&self, count: usize) -> CqAsyncReadOwned {
        self.inner.read_async(count)
    }

    fn wait_for_ctx_async(&self, ctx: &mut Context) -> AsyncTransferCq {
        self.inner.wait_for_ctx_async(ctx)
    }

    // pub async fn read_async(&self, count: usize) -> Result<Completion, crate::error::Error>  {
    //     self.inner.read_async(count).await
    // }
}

impl AsyncCompletionQueueImpl {
    pub(crate) fn new<EQ: ?Sized + 'static + SyncSend>(
        domain: &MyRc<DomainImplBase<EQ>>,
        attr: CompletionQueueAttr,
        context: *mut std::ffi::c_void,
        default_buff_size: usize,
    ) -> Result<Self, crate::error::Error> {
        Ok(Self {
            base: Async::new(CompletionQueueImpl::new(
                domain.clone(),
                attr,
                context,
                default_buff_size,
            )?)
            .unwrap(),
            pending_entries: MyRefCell::new(HashMap::new()),
        })
    }
}

pub struct AsyncTransferCq<'a> {
    pub(crate) ctx: usize,
    fut: Pin<Box<CqAsyncReadOwned<'a>>>,
}

impl<'a> AsyncTransferCq<'a> {
    #[allow(dead_code)]
    pub(crate) fn new(cq: &'a AsyncCompletionQueueImpl, ctx: usize) -> Self {
        Self {
            fut: Box::pin(CqAsyncReadOwned::new(1, cq)),
            ctx,
        }
    }
}

impl<'a> Future for AsyncTransferCq<'a> {
    type Output = Result<SingleCompletion, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut_self = self.get_mut();
        loop {
            #[cfg(feature = "thread-safe")]
            let queue_entry = mut_self
                .fut
                .cq
                .pending_entries
                .write()
                .remove(&mut_self.ctx);
            #[cfg(not(feature = "thread-safe"))]
            let queue_entry = mut_self
                .fut
                .cq
                .pending_entries
                .borrow_mut()
                .remove(&mut_self.ctx);
            if let Some(queue_entry) = queue_entry {
                match queue_entry {
                    Ok(entry) => {
                        // match entry {
                        //     SingleCompletion::Unspec(ref mut e) => {
                        //         // e.c_entry.op_context = unsafe {
                        //         //     (*(e.c_entry.op_context as *mut AsyncCtx))
                        //         //         .user_ctx
                        //         //         .unwrap_or(std::ptr::null_mut())
                        //         // }
                        //     }
                        //     SingleCompletion::Ctx(ref mut e) => {
                        //         // e.c_entry.op_context = unsafe {
                        //         //     (*(e.c_entry.op_context as *mut AsyncCtx))
                        //         //         .user_ctx
                        //         //         .unwrap_or(std::ptr::null_mut())
                        //         // }
                        //     }
                        //     SingleCompletion::Msg(ref mut e) => {
                        //         // e.c_entry.op_context = unsafe {
                        //         //     (*(e.c_entry.op_context as *mut AsyncCtx))
                        //         //         .user_ctx
                        //         //         .unwrap_or(std::ptr::null_mut())
                        //         // }
                        //     }
                        //     SingleCompletion::Data(ref mut e) => {
                        //         // e.c_entry.op_context = unsafe {
                        //         //     (*(e.c_entry.op_context as *mut AsyncCtx))
                        //         //         .user_ctx
                        //         //         .unwrap_or(std::ptr::null_mut())
                        //         // }
                        //     }
                        //     SingleCompletion::Tagged(ref mut e) => {
                        //         // e.c_entry.op_context = unsafe {
                        //         //     (*(e.c_entry.op_context as *mut AsyncCtx))
                        //         //         .user_ctx
                        //         //         .unwrap_or(std::ptr::null_mut())
                        //         // }
                        //     }
                        // }
                        // println!("Completion Found in map");
                        return std::task::Poll::Ready(Ok(entry));
                    }
                    Err(err_entry) => {
                        // println!("Completion Found in map as error");
                        return std::task::Poll::Ready(Err(err_entry));
                    }
                }
            }
            // println!("Waiting for readable CQ");

            #[allow(clippy::let_unit_value)]
            match ready!(mut_self.fut.as_mut().poll(cx)) {
                Ok(_) => {}
                Err(error) => {
                    if let ErrorKind::ErrorInQueue(ref q_error) = error.kind {
                        match q_error {
                            crate::error::QueueError::Event(_) => todo!(), // Should never be the case
                            crate::error::QueueError::Completion(q_err_entry) => {
                                if q_err_entry.c_err.op_context as usize == mut_self.ctx {
                                    return std::task::Poll::Ready(Err(error));
                                } else {
                                    #[cfg(feature = "thread-safe")]
                                    mut_self
                                        .fut
                                        .cq
                                        .pending_entries
                                        .write()
                                        .insert(q_err_entry.c_err.op_context as usize, Err(error));
                                    #[cfg(not(feature = "thread-safe"))]
                                    mut_self
                                        .fut
                                        .cq
                                        .pending_entries
                                        .borrow_mut()
                                        .insert(q_err_entry.c_err.op_context as usize, Err(error));
                                    mut_self.fut =
                                        Box::pin(CqAsyncReadOwned::new(1, mut_self.fut.cq));
                                    continue;
                                }
                            }
                        }
                    } else {
                        return std::task::Poll::Ready(Err(error));
                    }
                }
            }
            // println!("Can read");
            let mut found = None;
            match &mut_self.fut.buf {
                Completion::Unspec(entries) => {
                    for e in entries.iter() {
                        if e.c_entry.op_context as usize == mut_self.ctx {
                            found = Some(SingleCompletion::Unspec(e.clone()));
                        } else {
                            #[cfg(feature = "thread-safe")]
                            mut_self.fut.cq.pending_entries.write().insert(
                                e.c_entry.op_context as usize,
                                Ok(SingleCompletion::Unspec(e.clone())),
                            );
                            #[cfg(not(feature = "thread-safe"))]
                            mut_self.fut.cq.pending_entries.borrow_mut().insert(
                                e.c_entry.op_context as usize,
                                Ok(SingleCompletion::Unspec(e.clone())),
                            );
                        }
                    }
                }
                Completion::Ctx(entries) => {
                    for e in entries.iter() {
                        if e.c_entry.op_context as usize == mut_self.ctx {
                            found = Some(SingleCompletion::Ctx(e.clone()));
                        } else {
                            #[cfg(feature = "thread-safe")]
                            mut_self.fut.cq.pending_entries.write().insert(
                                e.c_entry.op_context as usize,
                                Ok(SingleCompletion::Ctx(e.clone())),
                            );
                            #[cfg(not(feature = "thread-safe"))]
                            mut_self.fut.cq.pending_entries.borrow_mut().insert(
                                e.c_entry.op_context as usize,
                                Ok(SingleCompletion::Ctx(e.clone())),
                            );
                        }
                    }
                }
                Completion::Msg(entries) => {
                    for e in entries.iter() {
                        if e.c_entry.op_context as usize == mut_self.ctx {
                            found = Some(SingleCompletion::Msg(e.clone()));
                        } else {
                            #[cfg(feature = "thread-safe")]
                            mut_self.fut.cq.pending_entries.write().insert(
                                e.c_entry.op_context as usize,
                                Ok(SingleCompletion::Msg(e.clone())),
                            );
                            #[cfg(not(feature = "thread-safe"))]
                            mut_self.fut.cq.pending_entries.borrow_mut().insert(
                                e.c_entry.op_context as usize,
                                Ok(SingleCompletion::Msg(e.clone())),
                            );
                        }
                    }
                }
                Completion::Data(entries) => {
                    for e in entries.iter() {
                        if e.c_entry.op_context as usize == mut_self.ctx {
                            found = Some(SingleCompletion::Data(e.clone()));
                        } else {
                            #[cfg(feature = "thread-safe")]
                            mut_self.fut.cq.pending_entries.write().insert(
                                e.c_entry.op_context as usize,
                                Ok(SingleCompletion::Data(e.clone())),
                            );
                            #[cfg(not(feature = "thread-safe"))]
                            mut_self.fut.cq.pending_entries.borrow_mut().insert(
                                e.c_entry.op_context as usize,
                                Ok(SingleCompletion::Data(e.clone())),
                            );
                        }
                    }
                }
                Completion::Tagged(entries) => {
                    for e in entries.iter() {
                        if e.c_entry.op_context as usize == mut_self.ctx {
                            found = Some(SingleCompletion::Tagged(e.clone()));
                        } else {
                            #[cfg(feature = "thread-safe")]
                            mut_self.fut.cq.pending_entries.write().insert(
                                e.c_entry.op_context as usize,
                                Ok(SingleCompletion::Tagged(e.clone())),
                            );
                            #[cfg(not(feature = "thread-safe"))]
                            mut_self.fut.cq.pending_entries.borrow_mut().insert(
                                e.c_entry.op_context as usize,
                                Ok(SingleCompletion::Tagged(e.clone())),
                            );
                        }
                    }
                }
            }
            match found {
                Some(v) => return std::task::Poll::Ready(Ok(v)),
                None => {
                    mut_self.fut = Box::pin(CqAsyncReadOwned::new(1, mut_self.fut.cq));
                }
            }
        }
    }
}

pub struct CqAsyncRead<'a> {
    num_entries: usize,
    buf: &'a mut Completion,
    cq: &'a AsyncCompletionQueueImpl,
    #[cfg(feature = "use-async-std")]
    fut: Option<Pin<Box<Readable<'a, CompletionQueueImpl<true, true, true>>>>>,
    #[cfg(feature = "use-tokio")]
    fut: Option<
        Pin<
            Box<
                dyn Future<
                        Output = Result<
                            tokio::io::unix::AsyncFdReadyGuard<
                                'a,
                                CompletionQueueImpl<true, true, true>,
                            >,
                            std::io::Error,
                        >,
                    > + 'a,
            >,
        >,
    >,
}

impl<'a> Future for CqAsyncRead<'a> {
    type Output = Result<(), Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut_self = self.get_mut();
        loop {
            let (err, _guard) = if mut_self.cq.trywait().is_err() {
                (
                    mut_self.cq.read_in(mut_self.num_entries, mut_self.buf),
                    None,
                )
            } else {
                if mut_self.fut.is_none() {
                    mut_self.fut = Some(Box::pin(mut_self.cq.base.readable()))
                }
                // Tokio returns something we need, async_std returns ()
                #[allow(clippy::let_unit_value)]
                let _guard = ready!(mut_self.fut.as_mut().unwrap().as_mut().poll(cx)).unwrap();

                // We only need to reset the option to none
                #[allow(clippy::let_underscore_future)]
                let _ = mut_self.fut.take().unwrap();
                (mut_self.cq.read_in(1, mut_self.buf), Some(_guard))
            };

            match err {
                Err(error) => {
                    if !matches!(error.kind, crate::error::ErrorKind::TryAgain) {
                        if matches!(error.kind, crate::error::ErrorKind::ErrorAvailable) {
                            let mut err = CompletionError::new();
                            mut_self.cq.readerr_in(&mut err, 0)?;
                            return std::task::Poll::Ready(Err(Error::from_queue_err(
                                crate::error::QueueError::Completion(err),
                            )));
                        }
                    } else {
                        #[cfg(feature = "use-tokio")]
                        if let Some(mut guard) = _guard {
                            if mut_self.cq.pending_entries.read().is_empty() {
                                guard.clear_ready()
                            }
                        }
                        continue;
                    }
                }
                Ok(len) => {
                    match &mut mut_self.buf {
                        Completion::Unspec(data) => unsafe { data.set_len(len) },
                        Completion::Ctx(data) => unsafe { data.set_len(len) },
                        Completion::Msg(data) => unsafe { data.set_len(len) },
                        Completion::Data(data) => unsafe { data.set_len(len) },
                        Completion::Tagged(data) => unsafe { data.set_len(len) },
                    }
                    return std::task::Poll::Ready(Ok(()));
                }
            }
        }
    }
}

impl<'a> CqAsyncReadOwned<'a> {
    pub(crate) fn new(num_entries: usize, cq: &'a AsyncCompletionQueueImpl) -> Self {
        Self {
            #[cfg(feature = "thread-safe")]
            buf: alloc_cq_entry!(*cq.base.get_ref().entry_buff.read(), num_entries),
            #[cfg(not(feature = "thread-safe"))]
            buf: alloc_cq_entry!(*cq.base.get_ref().entry_buff.borrow(), num_entries),
            num_entries,
            cq,
            fut: None,
        }
    }
}

pub struct CqAsyncReadOwned<'a> {
    num_entries: usize,
    buf: Completion,
    cq: &'a AsyncCompletionQueueImpl,
    #[cfg(feature = "use-async-std")]
    fut: Option<Pin<Box<Readable<'a, CompletionQueueImpl<true, true, true>>>>>,
    #[cfg(feature = "use-tokio")]
    fut: Option<
        Pin<
            Box<
                dyn Future<
                        Output = Result<
                            tokio::io::unix::AsyncFdReadyGuard<
                                'a,
                                CompletionQueueImpl<true, true, true>,
                            >,
                            std::io::Error,
                        >,
                    > + 'a,
            >,
        >,
    >,
}

impl<'a> Future for CqAsyncReadOwned<'a> {
    type Output = Result<(), Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut_self = self.get_mut();
        // let mut first = true;
        loop {
            // println!("About to block in CQ");
            let (err, _guard) = if mut_self.cq.trywait().is_err() {
                // println!("Cannot block");
                (
                    mut_self.cq.read_in(mut_self.num_entries, &mut mut_self.buf),
                    None,
                )
            } else {
                if mut_self.fut.is_none() {
                    mut_self.fut = Some(Box::pin(mut_self.cq.base.readable()))
                }
                #[allow(clippy::let_unit_value)]
                let _guard = ready!(mut_self.fut.as_mut().unwrap().as_mut().poll(cx)).unwrap();

                #[allow(clippy::let_underscore_future)]
                let _ = mut_self.fut.take().unwrap();
                // println!("Did not block");
                (mut_self.cq.read_in(1, &mut mut_self.buf), Some(_guard))
            };

            match err {
                Err(error) => {
                    if !matches!(error.kind, crate::error::ErrorKind::TryAgain) {
                        // println!("Found error!");
                        if matches!(error.kind, crate::error::ErrorKind::ErrorAvailable) {
                            // println!("Found error Avail!");

                            let mut err = CompletionError::new();
                            mut_self.cq.readerr_in(&mut err, 0)?;
                            println!("Returning actual error");
                            return std::task::Poll::Ready(Err(Error::from_queue_err(
                                crate::error::QueueError::Completion(err),
                            )));
                        }
                    } else {
                        // println!("Will continue");
                        #[cfg(feature = "use-tokio")]
                        if let Some(mut guard) = _guard {
                            if mut_self.cq.pending_entries.read().is_empty() {
                                guard.clear_ready()
                            }
                        }
                        continue;
                    }
                }
                Ok(len) => {
                    // println!("Actually read something {}", len);
                    match &mut mut_self.buf {
                        Completion::Unspec(data) => unsafe { data.set_len(len) },
                        Completion::Ctx(data) => unsafe { data.set_len(len) },
                        Completion::Msg(data) => unsafe { data.set_len(len) },
                        Completion::Data(data) => unsafe { data.set_len(len) },
                        Completion::Tagged(data) => unsafe { data.set_len(len) },
                    }
                    return std::task::Poll::Ready(Ok(()));
                }
            }
        }
    }
}

// impl AsFid for AsyncCompletionQueueImpl {
//     fn as_fid(&self) -> crate::fid::BorrowedFid<'_> {
//         self.base.get_ref().as_fid()
//     }
// }
// impl AsFid for &AsyncCompletionQueueImpl {
//     fn as_fid(&self) -> crate::fid::BorrowedFid<'_> {
//         self.base.get_ref().as_fid()
//     }
// }
// impl AsFid for MyRc<AsyncCompletionQueueImpl> {
//     fn as_fid(&self) -> crate::fid::BorrowedFid<'_> {
//         self.base.get_ref().as_fid()
//     }
// }

// impl AsRawFid for AsyncCompletionQueueImpl {
//     fn as_raw_fid(&self) -> RawFid {
//         self.base.get_ref().as_raw_fid()
//     }
// }

impl AsTypedFid<CqRawFid> for AsyncCompletionQueueImpl {
    fn as_typed_fid(&self) -> BorrowedTypedFid<CqRawFid> {
        self.base.get_ref().as_typed_fid()
    }
    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<CqRawFid> {
        self.base.get_ref().as_typed_fid_mut()
    }
}

pub struct CompletionQueueBuilder<'a> {
    cq_attr: CompletionQueueAttr,
    ctx: Option<&'a mut Context>,
    default_buff_size: usize,
}

impl<'a> CompletionQueueBuilder<'a> {
    /// Initiates the creation of a new [CompletionQueue] on `domain`.
    ///
    /// The initial configuration is what would be set if no `fi_cq_attr` or `context` was provided to
    /// the `fi_cq_open` call.
    pub fn new() -> CompletionQueueBuilder<'a> {
        Self {
            cq_attr: CompletionQueueAttr::new(),
            ctx: None,
            default_buff_size: 10,
        }
    }
}

impl<'a> Default for CompletionQueueBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> CompletionQueueBuilder<'a> {
    /// Specifies the minimum size of a completion queue.
    ///
    /// Corresponds to setting the field `fi_cq_attr::size` to `size`.
    pub fn size(mut self, size: usize) -> Self {
        self.cq_attr.size(size);
        self
    }

    pub fn signaling_vector(mut self, signaling_vector: i32) -> Self {
        // [TODO]
        self.cq_attr.signaling_vector(signaling_vector);
        self
    }

    /// Specificies the completion `format`
    ///
    /// Corresponds to setting the field `fi_cq_attr::format`.
    pub fn format(mut self, format: crate::enums::CqFormat) -> Self {
        self.cq_attr.format(format);
        self
    }

    pub fn default_buff_size(mut self, default_buff_size: usize) -> Self {
        self.default_buff_size = default_buff_size;
        self
    }

    /// Sets the context to be passed to the `CompletionQueue`.
    ///
    /// Corresponds to passing a non-NULL `context` value to `fi_cq_open`.
    pub fn context(self, ctx: &'a mut Context) -> CompletionQueueBuilder<'a> {
        CompletionQueueBuilder {
            ctx: Some(ctx),
            cq_attr: self.cq_attr,
            default_buff_size: self.default_buff_size,
        }
    }

    /// Constructs a new [CompletionQueue] with the configurations requested so far.
    ///
    /// Corresponds to creating a `fi_cq_attr`, setting its fields to the requested ones,
    /// and passing it to the `fi_cq_open` call with an optional `context`.
    pub fn build<EQ: ?Sized + 'static + SyncSend>(
        mut self,
        domain: &'a DomainBase<EQ>,
    ) -> Result<CompletionQueue<AsyncCompletionQueueImpl>, crate::error::Error> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::Fd);
        CompletionQueue::<AsyncCompletionQueueImpl>::new(
            domain,
            self.cq_attr,
            self.ctx,
            self.default_buff_size,
        )
    }
}

#[cfg(test)]
mod tests {

    use crate::domain::DomainBuilder;
    use crate::info::{Info, Version};

    use super::CompletionQueueBuilder;

    #[test]
    fn cq_open_close_simultaneous() {
        let info = Info::new(&Version {
            major: 1,
            minor: 19,
        })
        .get()
        .unwrap();
        let entry = info.into_iter().next().unwrap();

        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let count = 10;
        let domain = DomainBuilder::new(&fab, &entry).build().unwrap();
        // let mut cqs = Vec::new();
        for _ in 0..count {
            let _cq = CompletionQueueBuilder::new().build(&domain).unwrap();
        }
    }

    #[test]
    fn cq_open_close_sizes() {
        let info = Info::new(&Version {
            major: 1,
            minor: 19,
        })
        .get()
        .unwrap();
        let entry = info.into_iter().next().unwrap();

        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let domain = DomainBuilder::new(&fab, &entry).build().unwrap();
        for i in -1..17 {
            let size = if i == -1 { 0 } else { 1 << i };
            let _cq = CompletionQueueBuilder::new()
                .size(size)
                .build(&domain)
                .unwrap();
        }
    }
}

#[cfg(test)]
mod libfabric_lifetime_tests {
    use crate::async_::cq::CompletionQueueBuilder;
    use crate::domain::DomainBuilder;
    use crate::info::{Info, Version};

    #[test]
    fn cq_drops_before_domain() {
        let info = Info::new(&Version {
            major: 1,
            minor: 19,
        })
        .get()
        .unwrap();
        let entry = info.into_iter().next().unwrap();

        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let count = 10;
        let domain = DomainBuilder::new(&fab, &entry).build().unwrap();
        let mut cqs = Vec::new();
        for _ in 0..count {
            let cq = CompletionQueueBuilder::new().build(&domain).unwrap();
            cqs.push(cq);
        }
        drop(domain);
    }
}
