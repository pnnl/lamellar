use crate::cq::ReadCq;
use crate::cq::WaitCq;
use crate::cq::SingleCompletion;
use crate::domain::{DomainBase, DomainImplBase};
use crate::fid::AsTypedFid;
use crate::fid::BorrowedTypedFid;
use crate::fid::CqRawFid;
use crate::ContextType;
use crate::SyncSend;
use crate::{
    cq::{
        Completion, CompletionError, CompletionQueueAttr, CompletionQueueBase, CompletionQueueImpl,
    },
    error::Error,
    MappedAddress,
};
use crate::{Context, MyRc};
#[cfg(feature = "use-async-std")]
use async_io::{Async, Readable};
use std::pin::Pin;
use std::sync::atomic::AtomicUsize;
use std::{future::Future, task::ready};
#[cfg(feature = "use-tokio")]
use tokio::io::unix::AsyncFd as Async;

use super::AsyncFid;
// macro_rules! alloc_cq_entry {
//     ($format: expr, $count: expr) => {
//         match $format {
//             Completion::Ctx(_) => {
//                 let entries: Vec<CompletionEntry<CtxEntry>> = Vec::with_capacity($count);
//                 Completion::Ctx(entries)
//             }
//             Completion::Data(_) => {
//                 let entries: Vec<CompletionEntry<DataEntry>> = Vec::with_capacity($count);
//                 Completion::Data(entries)
//             }
//             Completion::Tagged(_) => {
//                 let entries: Vec<CompletionEntry<TaggedEntry>> = Vec::with_capacity($count);
//                 Completion::Tagged(entries)
//             }
//             Completion::Msg(_) => {
//                 let entries: Vec<CompletionEntry<MsgEntry>> = Vec::with_capacity($count);
//                 Completion::Msg(entries)
//             }
//             Completion::Unspec(_) => {
//                 let entries: Vec<CompletionEntry<CtxEntry>> = Vec::with_capacity($count);

//                 Completion::Unspec(entries)
//             }
//         }
//     };
// }

pub type CompletionQueue<T> = CompletionQueueBase<T>;

pub trait AsyncReadCq: ReadCq {
    // fn read_in_async<'a>(&'a self, buf: &'a mut Completion, count: usize) -> CqAsyncRead<'a>;
    // fn read_async(&self, count: usize,  ctx: &mut Context) -> CqAsyncReadOwned;
    fn wait_for_ctx_async<'a>(&'a self, ctx: &'a mut Context) -> AsyncTransferCq<'a>;
}

impl CompletionQueue<AsyncCompletionQueueImpl> {
    pub(crate) fn new_spinning<EQ: ?Sized + 'static + SyncSend>(
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
            inner: MyRc::new(AsyncCompletionQueueImpl::new_spinning(
                &domain.inner,
                attr,
                c_void,
                default_buff_size,
            )?),
        })
    }
}

impl CompletionQueue<AsyncCompletionQueueImpl> {
    pub(crate) fn new_blocking<EQ: ?Sized + 'static + SyncSend>(
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
            inner: MyRc::new(AsyncCompletionQueueImpl::new_blocking(
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
        let mut borrowed_entries = match &self.base {
            AsyncCompletionQueueImplBase::BlockingCq(async_cq) => {
                #[cfg(not(feature = "thread-safe"))]
                {
                    async_cq.get_ref().entry_buff.borrow_mut()
                }
                #[cfg(feature = "thread-safe")]
                {
                    async_cq.get_ref().entry_buff.write()
                }
            }
            AsyncCompletionQueueImplBase::SpinningCQ(cq) => {
                #[cfg(not(feature = "thread-safe"))]
                {
                    cq.entry_buff.borrow_mut()
                }
                #[cfg(feature = "thread-safe")]
                {
                    cq.entry_buff.write()
                }
            }
        };
        self.read_in(count, &mut borrowed_entries)?;
        Ok(borrowed_entries.clone())
    }

    fn readfrom(
        &self,
        count: usize,
    ) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> {
        let mut borrowed_entries = match &self.base {
            AsyncCompletionQueueImplBase::BlockingCq(async_cq) => {
                #[cfg(not(feature = "thread-safe"))]
                {
                    async_cq.get_ref().entry_buff.borrow_mut()
                }
                #[cfg(feature = "thread-safe")]
                {
                    async_cq.get_ref().entry_buff.write()
                }
            }
            AsyncCompletionQueueImplBase::SpinningCQ(cq) => {
                #[cfg(not(feature = "thread-safe"))]
                {
                    cq.entry_buff.borrow_mut()
                }
                #[cfg(feature = "thread-safe")]
                {
                    cq.entry_buff.write()
                }
            }
        };
        let address = self.readfrom_in(count, &mut borrowed_entries)?;
        Ok((borrowed_entries.clone(), address))
    }

    fn readerr(&self, flags: u64) -> Result<CompletionError, crate::error::Error> {
        let mut borrowed_entries = match &self.base {
            AsyncCompletionQueueImplBase::BlockingCq(async_cq) => {
                #[cfg(not(feature = "thread-safe"))]
                {
                    async_cq.get_ref().error_buff.borrow_mut()
                }
                #[cfg(feature = "thread-safe")]
                {
                    async_cq.get_ref().error_buff.write()
                }
            }
            AsyncCompletionQueueImplBase::SpinningCQ(cq) => {
                #[cfg(not(feature = "thread-safe"))]
                {
                    cq.error_buff.borrow_mut()
                }
                #[cfg(feature = "thread-safe")]
                {
                    cq.error_buff.write()
                }
            }
        };
        self.readerr_in(&mut borrowed_entries, flags)?;
        Ok(borrowed_entries.clone())
    }

    fn fid(&self) -> &crate::fid::OwnedCqFid {
        match &self.base {
            AsyncCompletionQueueImplBase::BlockingCq(async_cq) => &async_cq.get_ref().c_cq,
            AsyncCompletionQueueImplBase::SpinningCQ(cq) => &cq.c_cq,
        }
    }
}

// impl<'a, const WAIT: bool, const FD: bool> WaitObjectRetrieve<'a> for AsyncCompletionQueueImpl<WAIT, true, FD> {
//     fn wait_object(&self) -> Result<WaitObjType<'a>, crate::error::Error> {
//         let wait_obj = match &self.base {
//             AsyncCompletionQueueImplBase::BlockingCq(async_cq) => async_cq.get_ref().wait_obj,
//             AsyncCompletionQueueImplBase::SpinningCQ(cq) => cq.wait_obj,
//         };
//         if let Some(wait) = wait_obj {
//             if wait == libfabric_sys::fi_wait_obj_FI_WAIT_FD {
//                 let mut fd: i32 = 0;
//                 let err = unsafe {
//                     libfabric_sys::inlined_fi_control(
//                         self.as_typed_fid_mut().as_raw_fid(),
//                         libfabric_sys::FI_GETWAIT as i32,
//                         (&mut fd as *mut i32).cast(),
//                     )
//                 };
//                 if err < 0 {
//                     Err(crate::error::Error::from_err_code(
//                         (-err).try_into().unwrap(),
//                     ))
//                 } else {
//                     Ok(WaitObjType::Fd(unsafe { BorrowedFd::borrow_raw(fd) }))
//                 }
//             } else {
//                 panic!("Unexpected value for wait object in AsyncCompletionQueue");
//             }
//         } else {
//             panic!("Unexpected value for wait object in AsyncCompletionQueue");
//         }
//     }
// }

enum AsyncCompletionQueueImplBase {
    BlockingCq(Async<CompletionQueueImpl<true, true, true>>),
    SpinningCQ(CompletionQueueImpl<true, false, false>),
}

pub struct AsyncCompletionQueueImpl {
    base: AsyncCompletionQueueImplBase,
    pub(crate) pending_entries: AtomicUsize,
}
impl SyncSend for AsyncCompletionQueueImpl {}

impl WaitCq for AsyncCompletionQueueImpl {
    fn sread_with_cond(
        &self,
        count: usize,
        cond: usize,
        timeout: i32,
    ) -> Result<Completion, crate::error::Error> {
        match &self.base {
            AsyncCompletionQueueImplBase::BlockingCq(asyn_cq) => {
                asyn_cq.get_ref().sread_with_cond(count, cond, timeout)
            }
            AsyncCompletionQueueImplBase::SpinningCQ(cq) => {
                cq.sread_with_cond(count, cond, timeout)
            }
        }
    }

    fn sreadfrom_with_cond(
        &self,
        count: usize,
        cond: usize,
        timeout: i32,
    ) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> {
        match &self.base {
            AsyncCompletionQueueImplBase::BlockingCq(asyn_cq) => {
                asyn_cq.get_ref().sreadfrom_with_cond(count, cond, timeout)
            }
            AsyncCompletionQueueImplBase::SpinningCQ(cq) => {
                cq.sreadfrom_with_cond(count, cond, timeout)
            }
        }
    }
}

impl AsyncReadCq for AsyncCompletionQueueImpl {
    // fn read_in_async<'a>(&'a self, buf: &'a mut Completion, count: usize) -> CqAsyncRead<'a> {
    //     CqAsyncRead {
    //         num_entries: count,
    //         buf,
    //         cq: self,
    //         fut: None,
    //     }
    // }

    // fn read_async<'a>(&'a self, context: &'a mut Context) -> CqAsyncReadOwned<'a> {
    //     CqAsyncReadOwned::new(self, context)
    // }

    fn wait_for_ctx_async<'a>(&'a self, ctx: &'a mut Context) -> AsyncTransferCq<'a> {
        AsyncTransferCq::new(self, ctx)
    }
}

impl AsyncFid for AsyncCompletionQueueImpl {
    fn trywait(&self) -> Result<(), Error> {
        match &self.base {
            AsyncCompletionQueueImplBase::BlockingCq(async_cq) => async_cq
                .get_ref()
                ._domain_rc
                .fabric_impl()
                .trywait(async_cq.get_ref()),
            AsyncCompletionQueueImplBase::SpinningCQ(ref cq) => {
                cq._domain_rc.fabric_impl().trywait(cq)
            }
        }
    }
}

impl AsyncReadCq for CompletionQueue<AsyncCompletionQueueImpl> {
    // fn read_in_async<'a>(&'a self, buf: &'a mut Completion, count: usize) -> CqAsyncRead<'a> {
    //     self.inner.read_in_async(buf, count)
    // }

    // fn read_async(&self, count: usize, context: &mut Context) -> CqAsyncReadOwned {
    //     self.inner.read_async(count, context)
    // }

    fn wait_for_ctx_async<'a>(&'a self, ctx: &'a mut Context) -> AsyncTransferCq<'a> {
        self.inner.wait_for_ctx_async(ctx)
    }

    // pub async fn read_async(&self, count: usize) -> Result<Completion, crate::error::Error>  {
    //     self.inner.read_async(count).await
    // }
}

impl AsyncCompletionQueueImpl {
    pub(crate) fn new_blocking<EQ: ?Sized + 'static + SyncSend>(
        domain: &MyRc<DomainImplBase<EQ>>,
        attr: CompletionQueueAttr,
        context: *mut std::ffi::c_void,
        default_buff_size: usize,
    ) -> Result<Self, crate::error::Error> {
        Ok(Self {
            base: AsyncCompletionQueueImplBase::BlockingCq(
                Async::new(CompletionQueueImpl::new(
                    domain.clone(),
                    attr,
                    context,
                    default_buff_size,
                )?)
                .unwrap(),
            ),
            pending_entries: AtomicUsize::new(0),
        })
    }
}

impl AsyncCompletionQueueImpl {
    pub(crate) fn new_spinning<EQ: ?Sized + 'static + SyncSend>(
        domain: &MyRc<DomainImplBase<EQ>>,
        attr: CompletionQueueAttr,
        context: *mut std::ffi::c_void,
        default_buff_size: usize,
    ) -> Result<Self, crate::error::Error> {
        Ok(Self {
            base: AsyncCompletionQueueImplBase::SpinningCQ(CompletionQueueImpl::new(
                domain.clone(),
                attr,
                context,
                default_buff_size,
            )?),

            pending_entries: AtomicUsize::new(0),
        })
    }
}

pub struct AsyncTransferCq<'a> {
    fut: Pin<Box<CqAsyncReadOwned<'a>>>,
    waiting: bool,
    last_poll: Option<std::time::Instant>,
}

impl<'a> AsyncTransferCq<'a> {
    #[allow(dead_code)]
    pub(crate) fn new(cq: &'a AsyncCompletionQueueImpl, ctx: &'a mut Context) -> Self {
        // println!("Issued : {} {:x}", ctx.0.id(), ctx.inner() as usize);
        Self {
            fut: Box::pin(CqAsyncReadOwned::new(cq, ctx)),
            waiting: false,
            last_poll: None,
        }
    }
}

impl<'a> Future for AsyncTransferCq<'a> {
    type Output = Result<SingleCompletion, Error>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        // println!("Polling transfer cq for ctx {} {:x}", self.ctx.0.id(), self.ctx.inner() as usize);
        let mut_self = self.get_mut();
        ready!(mut_self.fut.as_mut().poll(cx))?;
        let state = mut_self.fut.context.state().take();
            // mut_self
            //     .fut
            //     .cq
            //     .pending_entries
            //     .fetch_sub(1, Ordering::SeqCst);
            mut_self.fut.context.reset();
            match state {
                Some(state) => match state {
                    crate::ContextState::Eq(_) => panic!("Should never find events here"),
                    crate::ContextState::Cq(comp) => return std::task::Poll::Ready(comp),
                },
                None => {
                    panic!("Should always have something to read when ready");
                }
            }
        // if mut_self.last_poll.is_none() {
        //     mut_self.last_poll = Some(Instant::now());
        //     // println!(
        //     //     "{} {:x} is being polled",
        //     //     mut_self.ctx.0.id(),
        //     //     mut_self.ctx.inner() as usize
        //     // );
        // } else {
        //     if mut_self.last_poll.unwrap().elapsed().as_secs() > 5 {
        //         // println!(
        //         //     "{} {:x} is being polled",
        //         //     mut_self.ctx.0.id(),
        //         //     mut_self.ctx.inner() as usize
        //         // );
        //         mut_self.last_poll = Some(Instant::now());
        //     }
        // }
        // loop {
        //     if mut_self.waiting && mut_self.ctx.ready() {
        //         // println!(
        //         //     "{} {:x} Completed YEAH",
        //         //     mut_self.ctx.0.id(),
        //         //     mut_self.ctx.inner() as usize
        //         // );
        //         let state = mut_self.ctx.state().take();
        //         mut_self
        //             .fut
        //             .cq
        //             .pending_entries
        //             .fetch_sub(1, Ordering::SeqCst);
        //         mut_self.ctx.reset();
        //         match state {
        //             Some(state) => match state {
        //                 crate::ContextState::Eq(_) => panic!("Should never find events here"),
        //                 crate::ContextState::Cq(comp) => return std::task::Poll::Ready(comp),
        //             },
        //             None => {
        //                 panic!("Should always have something to read when ready");
        //             }
        //         }
        //     } else if mut_self.waiting {
        //         // TODO: Reconsider if we should poll the cq here
        //         // let ret = mut_self.fut.cq.read(0);
        //         // match ret {
        //         //     Err(error) => {
        //         //         if !matches!(error.kind, crate::error::ErrorKind::TryAgain) {
        //         //             panic!("Other error");
        //         //         }
        //         //     }
        //         //     Ok(_) => {}
        //         // }

        //         // async_std::task::yield_now();
        //         cx.waker().wake_by_ref();
        //         return std::task::Poll::Pending;
        //     } else {
        //         // println!(
        //         //     "{} {:x} Might block",
        //         //     mut_self.ctx.0.id(),
        //         //     mut_self.ctx.inner() as usize
        //         // );
        //         #[allow(clippy::let_unit_value)]
        //         match ready!(mut_self.fut.as_mut().poll(cx)) {
        //             Ok(_) => {
        //                 // println!(
        //                 //     "{} {:x} Didn't block",
        //                 //     mut_self.ctx.0.id(),
        //                 //     mut_self.ctx.inner() as usize
        //                 // );
        //             }
        //             Err(error) => {
        //                 // println!(
        //                 //     "{} {:x} Didn't block",
        //                 //     mut_self.ctx.0.id(),
        //                 //     mut_self.ctx.inner() as usize
        //                 // );
        //                 if let ErrorKind::ErrorInCompletionQueue(ref q_error) = error.kind {
        //                     if q_error.c_err.op_context as usize == mut_self.ctx.inner() as usize {
        //                         return std::task::Poll::Ready(Err(error));
        //                     } else {
        //                         mut_self
        //                             .fut
        //                             .cq
        //                             .pending_entries
        //                             .fetch_add(1, Ordering::SeqCst);
        //                         match &mut_self.ctx.0 {
        //                             ContextType::Context1(_) => unsafe {
        //                                 (q_error.c_err.op_context as *mut std::ffi::c_void
        //                                     as *mut crate::Context1)
        //                                     .as_mut()
        //                                     .unwrap()
        //                             }
        //                             .set_completion_done(Err(error)),
        //                             ContextType::Context2(_) => unsafe {
        //                                 (q_error.c_err.op_context as *mut std::ffi::c_void
        //                                     as *mut crate::Context2)
        //                                     .as_mut()
        //                                     .unwrap()
        //                             }
        //                             .set_completion_done(Err(error)),
        //                         }
        //                         // mut_self.fut =
        //                         //     Box::pin(CqAsyncReadOwned::new(1, mut_self.fut.cq));
        //                         continue;
        //                     }
        //                 } else {
        //                     return std::task::Poll::Ready(Err(error));
        //                 }
        //             }
        //         }
        //     }

        //     // println!("Can read");
        //     let mut found = None;
        //     // let mut ctx_val = 0;
        //     // let mut ctx_addr = 0usize;
        //     match &mut_self.fut.buf {
        //         Completion::Unspec(entries) => {
        //             for e in entries {
        //                 if e.c_entry.op_context as usize == mut_self.ctx.inner() as usize {
        //                     // ctx_val = mut_self.ctx.0.id();
        //                     // ctx_addr = mut_self.ctx.inner() as usize;
        //                     found = Some(SingleCompletion::Unspec(e.clone()));
        //                 } else {
        //                     mut_self
        //                         .fut
        //                         .cq
        //                         .pending_entries
        //                         .fetch_add(1, Ordering::SeqCst);
        //                     match &mut_self.ctx.0 {
        //                         ContextType::Context1(_) => {
        //                             let ctx = unsafe {
        //                                 (e.c_entry.op_context as *mut std::ffi::c_void
        //                                     as *mut crate::Context1)
        //                                     .as_mut()
        //                                     .unwrap()
        //                             };
        //                             // ctx_addr = ctx as *mut crate::Context1 as usize;
        //                             // ctx_val = ctx.id;
        //                             ctx.set_completion_done(Ok(SingleCompletion::Unspec(e.clone())))
        //                         }
        //                         ContextType::Context2(_) => {
        //                             let ctx = unsafe {
        //                                 (e.c_entry.op_context as *mut std::ffi::c_void
        //                                     as *mut crate::Context2)
        //                                     .as_mut()
        //                                     .unwrap()
        //                             };
        //                             // ctx_addr = ctx as *mut crate::Context2 as usize;
        //                             // ctx_val = ctx.id;
        //                             ctx.set_completion_done(Ok(SingleCompletion::Unspec(e.clone())))
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //         Completion::Ctx(entries) => {
        //             for e in entries {
        //                 if e.c_entry.op_context as usize == mut_self.ctx.inner() as usize {
        //                     // ctx_val = mut_self.ctx.0.id();
        //                     // ctx_addr = e.c_entry.op_context as usize;
        //                     found = Some(SingleCompletion::Ctx(e.clone()));
        //                 } else {
        //                     mut_self
        //                         .fut
        //                         .cq
        //                         .pending_entries
        //                         .fetch_add(1, Ordering::SeqCst);
        //                     match &mut_self.ctx.0 {
        //                         ContextType::Context1(_) => {
        //                             let ctx = unsafe {
        //                                 (e.c_entry.op_context as *mut std::ffi::c_void
        //                                     as *mut crate::Context1)
        //                                     .as_mut()
        //                                     .unwrap()
        //                             };
        //                             // ctx_addr = ctx as *mut crate::Context1 as usize;
        //                             // ctx_val = ctx.id;
        //                             ctx.set_completion_done(Ok(SingleCompletion::Ctx(e.clone())))
        //                         }
        //                         ContextType::Context2(_) => {
        //                             let ctx = unsafe {
        //                                 (e.c_entry.op_context as *mut std::ffi::c_void
        //                                     as *mut crate::Context2)
        //                                     .as_mut()
        //                                     .unwrap()
        //                             };
        //                             // ctx_addr = ctx as *mut crate::Context2 as usize;
        //                             // ctx_val = ctx.id;
        //                             ctx.set_completion_done(Ok(SingleCompletion::Ctx(e.clone())))
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //         Completion::Msg(entries) => {
        //             for e in entries {
        //                 if e.c_entry.op_context as usize == mut_self.ctx.inner() as usize {
        //                     // ctx_val = mut_self.ctx.0.id();
        //                     // ctx_addr = e.c_entry.op_context as usize;
        //                     found = Some(SingleCompletion::Msg(e.clone()));
        //                 } else {
        //                     mut_self
        //                         .fut
        //                         .cq
        //                         .pending_entries
        //                         .fetch_add(1, Ordering::SeqCst);
        //                     match &mut_self.ctx.0 {
        //                         ContextType::Context1(_) => {
        //                             let ctx = unsafe {
        //                                 (e.c_entry.op_context as *mut std::ffi::c_void
        //                                     as *mut crate::Context1)
        //                                     .as_mut()
        //                                     .unwrap()
        //                             };
        //                             // ctx_addr = ctx as *mut crate::Context1 as usize;
        //                             // ctx_val = ctx.id;
        //                             ctx.set_completion_done(Ok(SingleCompletion::Msg(e.clone())))
        //                         }
        //                         ContextType::Context2(_) => {
        //                             let ctx = unsafe {
        //                                 (e.c_entry.op_context as *mut std::ffi::c_void
        //                                     as *mut crate::Context2)
        //                                     .as_mut()
        //                                     .unwrap()
        //                             };
        //                             // ctx_addr = ctx as *mut crate::Context2 as usize;
        //                             // ctx_val = ctx.id;
        //                             ctx.set_completion_done(Ok(SingleCompletion::Msg(e.clone())))
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //         Completion::Data(entries) => {
        //             for e in entries {
        //                 if e.c_entry.op_context as usize == mut_self.ctx.inner() as usize {
        //                     // ctx_val = mut_self.ctx.0.id();
        //                     // ctx_addr = e.c_entry.op_context as usize;
        //                     found = Some(SingleCompletion::Data(e.clone()));
        //                 } else {
        //                     mut_self
        //                         .fut
        //                         .cq
        //                         .pending_entries
        //                         .fetch_add(1, Ordering::SeqCst);
        //                     match &mut_self.ctx.0 {
        //                         ContextType::Context1(_) => {
        //                             let ctx = unsafe {
        //                                 (e.c_entry.op_context as *mut std::ffi::c_void
        //                                     as *mut crate::Context1)
        //                                     .as_mut()
        //                                     .unwrap()
        //                             };
        //                             // ctx_addr = ctx as *mut crate::Context1 as usize;
        //                             // ctx_val = ctx.id;
        //                             ctx.set_completion_done(Ok(SingleCompletion::Data(e.clone())))
        //                         }
        //                         ContextType::Context2(_) => {
        //                             let ctx = unsafe {
        //                                 (e.c_entry.op_context as *mut std::ffi::c_void
        //                                     as *mut crate::Context2)
        //                                     .as_mut()
        //                                     .unwrap()
        //                             };
        //                             // ctx_addr = ctx as *mut crate::Context2 as usize;
        //                             // ctx_val = ctx.id;
        //                             ctx.set_completion_done(Ok(SingleCompletion::Data(e.clone())))
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //         Completion::Tagged(entries) => {
        //             for e in entries {
        //                 if e.c_entry.op_context as usize == mut_self.ctx.inner() as usize {
        //                     // ctx_val = mut_self.ctx.0.id();
        //                     // ctx_addr = e.c_entry.op_context as usize;
        //                     found = Some(SingleCompletion::Tagged(e.clone()));
        //                 } else {
        //                     mut_self
        //                         .fut
        //                         .cq
        //                         .pending_entries
        //                         .fetch_add(1, Ordering::SeqCst);
        //                     match &mut_self.ctx.0 {
        //                         ContextType::Context1(_) => {
        //                             let ctx = unsafe {
        //                                 (e.c_entry.op_context as *mut std::ffi::c_void
        //                                     as *mut crate::Context1)
        //                                     .as_mut()
        //                                     .unwrap()
        //                             };
        //                             // ctx_addr = ctx as *mut crate::Context1 as usize;
        //                             // ctx_val = ctx.id;
        //                             ctx.set_completion_done(Ok(SingleCompletion::Tagged(e.clone())))
        //                         }
        //                         ContextType::Context2(_) => {
        //                             let ctx = unsafe {
        //                                 (e.c_entry.op_context as *mut std::ffi::c_void
        //                                     as *mut crate::Context2)
        //                                     .as_mut()
        //                                     .unwrap()
        //                             };
        //                             // ctx_addr = ctx as *mut crate::Context2 as usize;
        //                             // ctx_val = ctx.id;
        //                             ctx.set_completion_done(Ok(SingleCompletion::Tagged(e.clone())))
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //     }
        //     match found {
        //         Some(v) => {
        //             // println!("{} {:x} Finished right away", mut_self.ctx.0.id(), ctx_addr);
        //             return std::task::Poll::Ready(Ok(v));
        //         }
        //         None => {
        //             mut_self.waiting = true;

        //             // println!(
        //             //     "{} {:x} != {} {:x}!!!!Not for me!!!!!!!",
        //             //     ctx_val,
        //             //     ctx_addr,
        //             //     mut_self.ctx.0.id(),
        //             //     mut_self.ctx.inner() as usize
        //             // );
        //             // mut_self.fut = Box::pin(CqAsyncReadOwned::new(1, mut_self.fut.cq));
        //         }
            // }
        // }
    }
}

// pub struct CqAsyncRead<'a> {
//     num_entries: usize,
//     buf: &'a mut Completion,
//     cq: &'a AsyncCompletionQueueImpl,
//     #[cfg(feature = "use-async-std")]
//     fut: Option<Pin<Box<Readable<'a, CompletionQueueImpl<true, true, true>>>>>,
//     #[cfg(feature = "use-tokio")]
//     fut: Option<
//         Pin<
//             Box<
//                 dyn Future<
//                         Output = Result<
//                             tokio::io::unix::AsyncFdReadyGuard<
//                                 'a,
//                                 CompletionQueueImpl<true, true, true>,
//                             >,
//                             std::io::Error,
//                         >,
//                     > + 'a,
//             >,
//         >,
//     >,
// }

// impl<'a> Future for CqAsyncRead<'a> {
//     type Output = Result<(), Error>;

//     fn poll(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Self::Output> {
//         let mut_self = self.get_mut();
//         loop {
//             let (err, _guard) = if mut_self.cq.trywait().is_err() {
//                 (
//                     mut_self.cq.read_in(mut_self.num_entries, mut_self.buf),
//                     None,
//                 )
//             } else {
//                 if mut_self.fut.is_none() {
//                     mut_self.fut = Some(Box::pin(mut_self.cq.base.readable()))
//                 }
//                 // Tokio returns something we need, async_std returns ()
//                 #[allow(clippy::let_unit_value)]
//                 let _guard = ready!(mut_self.fut.as_mut().unwrap().as_mut().poll(cx)).unwrap();

//                 // We only need to reset the option to none
//                 #[allow(clippy::let_underscore_future)]
//                 let _ = mut_self.fut.take().unwrap();
//                 (mut_self.cq.read_in(1, mut_self.buf), Some(_guard))
//             };

//             match err {
//                 Err(error) => {
//                     if !matches!(error.kind, crate::error::ErrorKind::TryAgain) {
//                         if matches!(error.kind, crate::error::ErrorKind::ErrorAvailable) {
//                             let mut err = CompletionError::new();
//                             mut_self.cq.readerr_in(&mut err, 0)?;
//                             return std::task::Poll::Ready(Err(Error::from_completion_queue_err(
//                                 err,
//                             )));
//                         }
//                     } else {
//                         #[cfg(feature = "use-tokio")]
//                         if let Some(mut guard) = _guard {
//                             if mut_self.cq.pending_entries.load(Ordering::SeqCst) == 0 {
//                                 guard.clear_ready()
//                             }
//                         }
//                         continue;
//                     }
//                 }
//                 Ok(len) => {
//                     match &mut mut_self.buf {
//                         Completion::Unspec(data) => unsafe { data.set_len(len) },
//                         Completion::Ctx(data) => unsafe { data.set_len(len) },
//                         Completion::Msg(data) => unsafe { data.set_len(len) },
//                         Completion::Data(data) => unsafe { data.set_len(len) },
//                         Completion::Tagged(data) => unsafe { data.set_len(len) },
//                     }
//                     return std::task::Poll::Ready(Ok(()));
//                 }
//             }
//         }
//     }
// }

impl<'a> CqAsyncReadOwned<'a> {
    pub(crate) fn new(cq: &'a AsyncCompletionQueueImpl, context: &'a mut Context) -> Self {
    
        Self {
            buf: None,
            cq,
            fut: None,
            context,
        }
    }
}

pub struct CqAsyncReadOwned<'a> {
    buf: Option<SingleCompletion>,
    cq: &'a AsyncCompletionQueueImpl,
    context: &'a mut Context,
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
        // println!("Polling cq for {} entries", mut_self.num_entries);
        // let mut first = true;
        loop {
            // let cq_guard = mut_self.cq.base.get_ref().entry_buff.write();
            // let mut blocked = mut_self.cq.base.get_ref().block.write();
            // if *blocked != 0 {
            //     println!("BLOCKED {}", *blocked);
            //     assert!(*blocked == 0);
            // }
            // mut_self.context.0.set_waker(cx.waker().clone());
            if mut_self.context.ready() {
                return std::task::Poll::Ready(Ok(()));
            }
            // println!("About to block in CQ");

            // TODO: Reenable blocking on the file descriptor becoming readable
            // Note that we probably need to make sure no one polls the cq before we
            // actually create the future
            if mut_self.buf.is_none() {

                let (err, _guard) = match &mut_self.cq.base {
                    AsyncCompletionQueueImplBase::BlockingCq(async_cq) => {
                        let can_wait = mut_self.cq.trywait();
                        if can_wait.is_err() {
                            // println!("Cannot block");
                            let to_ret = (
                                async_cq
                                    .get_ref()
                                    .read(1),
                                None);
                            to_ret
                        } else {
                            // TODO: Do we need to skip calling readable if we were awaken?
                            // Does being awaken means that readable has returned true?
                            if mut_self.fut.is_none() {
                                // println!("Can wait in CQ {}", can_wait.is_ok());
                                // mut_self.cq.trywait().unwrap();
                                mut_self.fut = Some(Box::pin(async_cq.readable()))
                            }
                            #[allow(clippy::let_unit_value)]
                            // drop(blocked);
                            let _guard =
                                ready!(mut_self.fut.as_mut().unwrap().as_mut().poll(cx)).unwrap();

                            #[allow(clippy::let_underscore_future)]
                            let _ = mut_self.fut.take().unwrap();
                            // println!("Did not block");
                            // let cq_guard = mut_self.cq.base.get_ref().entry_buff.write();

                            (
                                async_cq.get_ref().read(1),
                                Some(_guard),
                            )
                        }
                    }
                    AsyncCompletionQueueImplBase::SpinningCQ(cq) => {
                        let to_ret = ( cq.read(1), None);
                        to_ret
                    }
                };


                match err {
                    Err(error) => {
                        if !matches!(error.kind, crate::error::ErrorKind::TryAgain) {
                            // println!("Found error!");
                            if matches!(error.kind, crate::error::ErrorKind::ErrorAvailable) {
                                let mut err = CompletionError::new();
                                mut_self.cq.readerr_in(&mut err, 0)?;
                                if err.c_err.op_context as usize == mut_self.context.inner() as usize {
                                    // println!("Found error in context!");
                                    // panic!("Error {:?}", err.error());
                                    return std::task::Poll::Ready(Err(Error::from_completion_queue_err(
                                        err,
                                    )));
                                }
                                else
                                {
                                    // println!("Found error in context but not for me!");
                                    // We need to return the error to the context
                                    match mut_self.context.0 {
                                        ContextType::Context1(_) => unsafe {
                                            (err.c_err.op_context as *mut std::ffi::c_void
                                                as *mut crate::Context1)
                                                .as_mut()
                                                .unwrap()
                                        }
                                        .set_completion_done(Err(Error::from_completion_queue_err(
                                            err,
                                        ))),
                                        ContextType::Context2(_) => unsafe {
                                            (err.c_err.op_context as *mut std::ffi::c_void
                                                as *mut crate::Context2)
                                                .as_mut()
                                                .unwrap()
                                        }
                                        .set_completion_done(Err(Error::from_completion_queue_err(
                                            err,
                                        ))),
                                    }
                                    cx.waker().wake_by_ref();
                                    return std::task::Poll::Pending;
                                }
                            }
                        } else {
                            // println!("Will continue");
                            #[cfg(feature = "use-tokio")]
                            if let Some(mut guard) = _guard {
                                if mut_self.cq.pending_entries.load(Ordering::SeqCst) == 0 {
                                    guard.clear_ready()
                                }
                            }
                            cx.waker().wake_by_ref();
                            return std::task::Poll::Pending;
                        }
                    }
                    Ok(mut completion) => {
                        mut_self.buf = completion.pop();
                    }
                }
            }
            let done = match &mut mut_self.buf {
                Some(d) => unsafe {
                    // println!("Signaling {:?}", data[0].c_entry.op_context);
                    let mut done = true;
                    match mut_self.context.0 {
                        ContextType::Context1(_) => {
                            let context = (d.op_context() as *mut std::ffi::c_void
                                as *mut crate::Context1)
                                .as_mut()
                                .unwrap();
                            if d.op_context() as usize != mut_self.context.inner() as usize {
                                context.set_completion_done(Ok(d.clone()));
                                mut_self.buf = None;
                                done = false;
                            }
                            else {
                                context.set_completion_done(Ok(d.clone()));
                            }
                        }
                        ContextType::Context2(_) => {
                            let context = (d.op_context() as *mut std::ffi::c_void
                                as *mut crate::Context2)
                                .as_mut()
                                .unwrap();
                            if d.op_context() as usize != mut_self.context.inner() as usize {
                                context.set_completion_done(Ok(d.clone()));
                                mut_self.buf = None;
                                done = false;
                            }
                            else {
                                context.set_completion_done(Ok(d.clone()));
                                mut_self.buf = None;
                            }
                        }
                    }
                    
                    done
                },
                None => {
                    panic!("Should always have something to read when ready");
                }
            };
            if done {
                // If we handled all contexts, we can return Ready
                // Otherwise we need to poll again to handle the remaining ones
                // println!("All done");
                return std::task::Poll::Ready(Ok(()));
            }
            else {
                cx.waker().wake_by_ref();
                return std::task::Poll::Pending;
            }
        }
    }
}

impl AsTypedFid<CqRawFid> for AsyncCompletionQueueImpl {
    fn as_typed_fid(&self) -> BorrowedTypedFid<CqRawFid> {
        match &self.base {
            AsyncCompletionQueueImplBase::BlockingCq(async_cq) => async_cq.get_ref().as_typed_fid(),
            AsyncCompletionQueueImplBase::SpinningCQ(cq) => cq.as_typed_fid(),
        }
    }
    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<CqRawFid> {
        match &self.base {
            AsyncCompletionQueueImplBase::BlockingCq(async_cq) => {
                async_cq.get_ref().as_typed_fid_mut()
            }
            AsyncCompletionQueueImplBase::SpinningCQ(cq) => cq.as_typed_fid_mut(),
        }
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

    pub fn build<EQ: ?Sized + 'static + SyncSend>(
        self,
        domain: &'a DomainBase<EQ>,
    ) -> Result<CompletionQueue<AsyncCompletionQueueImpl>, crate::error::Error> {
        #[cfg(feature = "async-cqs-spin")]
        {
            self.build_spinning_cq(domain)
        }
        #[cfg(not(feature = "async-cqs-spin"))]
        {
            self.build_blocking_cq(domain)
        }
    }

    /// Constructs a new [CompletionQueue] with the configurations requested so far.
    ///
    /// Corresponds to creating a `fi_cq_attr`, setting its fields to the requested ones,
    /// and passing it to the `fi_cq_open` call with an optional `context`.
    pub fn build_blocking_cq<EQ: ?Sized + 'static + SyncSend>(
        mut self,
        domain: &'a DomainBase<EQ>,
    ) -> Result<CompletionQueue<AsyncCompletionQueueImpl>, crate::error::Error> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::Fd);
        CompletionQueue::<AsyncCompletionQueueImpl>::new_blocking(
            domain,
            self.cq_attr,
            self.ctx,
            self.default_buff_size,
        )
    }

    /// Constructs a new [CompletionQueue] with the configurations requested so far.
    ///
    /// Corresponds to creating a `fi_cq_attr`, setting its fields to the requested ones,
    /// and passing it to the `fi_cq_open` call with an optional `context`.
    pub fn build_spinning_cq<EQ: ?Sized + 'static + SyncSend>(
        self,
        domain: &'a DomainBase<EQ>,
    ) -> Result<CompletionQueue<AsyncCompletionQueueImpl>, crate::error::Error> {
        CompletionQueue::<AsyncCompletionQueueImpl>::new_spinning(
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
    use crate::info::Info;

    use super::CompletionQueueBuilder;

    #[test]
    fn cq_open_close_simultaneous() {
        let info = Info::new(&crate::info::libfabric_version()).get().unwrap();
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
        let info = Info::new(&crate::info::libfabric_version()).get().unwrap();
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
    use crate::info::Info;

    #[test]
    fn cq_drops_before_domain() {
        let info = Info::new(&crate::info::libfabric_version()).get().unwrap();
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
