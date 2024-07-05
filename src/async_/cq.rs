use std::{ops::Deref, rc::Rc, marker::PhantomData, future::Future, task::ready};

#[cfg(feature="use-async-std")]
use async_io::Async as Async;
#[cfg(feature="use-tokio")]
use tokio::io::unix::AsyncFd as Async;

use crate::{cq::{CompletionQueueImpl, SingleCompletionFormat, CompletionFormat, CompletionQueueAttr, Completion, CtxEntry, DataEntry, TaggedEntry, MsgEntry, CompletionError, CompletionQueueBase}, error::Error, fid::{AsFid, RawFid, AsRawFid}, cqoptions::{CqConfig, self, Options}, FdRetrievable, MappedAddress, Waitable, WaitRetrievable, enums::WaitObjType};

use super::{AsyncCtx, domain::{AsyncDomainImpl, Domain}};
macro_rules! alloc_cq_entry {
    ($format: expr, $count: expr) => {
        match $format {
            CompletionFormat::Ctx(_) => {
                let entries: Vec<Completion<CtxEntry>> = Vec::with_capacity($count);
                // for _ in 0..$count {
                //     entries.push(CqEntry::new())
                // }
                CompletionFormat::Ctx(entries)
            }
            CompletionFormat::Data(_) => {
                let entries: Vec<Completion<DataEntry>> = Vec::with_capacity($count);
                // for _ in 0..$count {
                //     entries.push(CqDataEntry::new())
                // }
                CompletionFormat::Data(entries)
            }
            CompletionFormat::Tagged(_) => {
                let entries: Vec<Completion<TaggedEntry>> = Vec::with_capacity($count);
                // for _ in 0..$count {
                //     entries.push(CqTaggedEntry::new())
                // }
                CompletionFormat::Tagged(entries)
            }
            CompletionFormat::Msg(_) => {
                let entries: Vec<Completion<MsgEntry>> = Vec::with_capacity($count);
                // for _ in 0..$count {
                //     entries.push(CqMsgEntry::new())
                // }
                CompletionFormat::Msg(entries)
            }
            CompletionFormat::Unspec(_) => {
                let entries: Vec<Completion<CtxEntry>> = Vec::with_capacity($count);

                CompletionFormat::Unspec(entries)
            }
        }
    };
}

// macro_rules! read_cq_entry_into {
//     ($read_fn: expr, $cq: expr, $count: expr, $buff: expr, $( $x:ident),*) => {
//         unsafe{ $read_fn($cq, $buff, $count, $($x,)*)}
//     }
// }

pub type CompletionQueue<T>  = CompletionQueueBase<T, AsyncCompletionQueueImpl>;

// pub struct CompletionQueue<T: CqConfig> {
//     pub(crate) inner: Rc<AsyncCompletionQueueImpl>,
//     phantom: PhantomData<T>,
// }


impl<T: CqConfig + FdRetrievable> CompletionQueue<T> {
    pub(crate) fn new<T0>(_options: T, domain: &Domain, attr: CompletionQueueAttr, context: Option<&mut T0>, default_buff_size: usize) -> Result<Self, crate::error::Error> {
        Ok(
            Self {
                inner: Rc::new(AsyncCompletionQueueImpl::new(&domain.inner, attr, context, default_buff_size)?),
                phantom: PhantomData,
            }
        )
    }

    pub async fn read_async(&self, count: usize) -> Result<CompletionFormat, crate::error::Error> {
        let mut buf = alloc_cq_entry!(*self.inner.entry_buff.borrow(), count);
        self.inner.read_async(&mut buf, count).await?;
        Ok(buf)
    }



}

impl<T: CqConfig > CompletionQueue<T> {

    /// Reads one or more completions from a completion queue
    /// 
    /// The call will read up to `count` completion entries which will be stored in a [Completion]
    /// 
    /// Corresponds to `fi_cq_read` with the `buf` maintained and casted automatically
    pub fn read(&self, count: usize) -> Result<CompletionFormat, crate::error::Error> {
        self.inner.read(count)
    }




    // /// Reads one or more completions from a completion queue
    // /// 
    // /// The call will read up to `count` completion entries which will be stored in a [Completion]
    // /// 
    // /// Corresponds to `fi_cq_read` with the `buf` maintained and casted automatically
    // pub fn read_in(&self, count: usize, buff: &mut CompletionFormat) -> Result<(), crate::error::Error> {
    //     // assert!(count <= buff.len());
    //     self.inner.read_in(count, buff)
    // }

    /// Reads one or more completions from a completion queue
    /// 
    /// The call will read up to `count` completion entries which will be stored in a [Completion]
    /// 
    /// Corresponds to `fi_cq_read` with the `buf` maintained and casted automatically
    pub unsafe fn read_in_unchecked(&self, count: usize, buff: &mut CompletionFormat) -> Result<usize, crate::error::Error> {
        self.inner.read_in(count, buff)
    }

    // /// Similar to [Self::read] with the exception that it allows the CQ to return source address information to the user for any received data
    // /// 
    // /// If there is no source address to return it will return None as the second parameter
    // /// 
    // /// Corresponds to `fi_cq_readfrom`
    // pub fn readfrom(&self, count: usize) -> Result<(CompletionFormat, Option<AsyncMappedAddress>), crate::error::Error> {
    //     self.inner.readfrom(count)
    // }

    // /// Similar to [Self::read] with the exception that it allows the CQ to return source address information to the user for any received data
    // /// 
    // /// If there is no source address to return it will return None as the second parameter
    // /// 
    // /// Corresponds to `fi_cq_readfrom`
    // pub fn readfrom_in(&self, count: usize, buff: &mut CompletionFormat) -> Result<Option<MappedAddress>, crate::error::Error> {
    //     // assert!(count <= buff.len());
    //     self.inner.readfrom_in(count, buff)
    // }

    /// Similar to [Self::read] with the exception that it allows the CQ to return source address information to the user for any received data
    /// 
    /// If there is no source address to return it will return None as the second parameter
    /// 
    /// Corresponds to `fi_cq_readfrom`
    pub unsafe fn readfrom_in_unchecked(&self, count: usize, buff: &mut CompletionFormat) -> Result<Option<MappedAddress>, crate::error::Error> {
        self.inner.readfrom_in(count, buff)
    }
    
    /// Reads one error completion from the queue
    /// 
    /// Corresponds to `fi_cq_readerr`
    pub fn readerr(&self, flags: u64) -> Result<CompletionError, crate::error::Error> {
        self.inner.readerr(flags)
    }
    
    /// Reads one error completion from the queue
    /// 
    /// Corresponds to `fi_cq_readerr`
    pub fn readerr_in(&self, err_buff: &mut CompletionError, flags: u64) -> Result<(), crate::error::Error> {
        self.inner.readerr_in(err_buff, flags)
    }

    pub fn print_error(&self, err_entry: &crate::cq::CompletionError) { //[TODO] Return a string
        self.inner.print_error(err_entry)
    }

    // pub(crate) fn request(&self) -> usize {
    //     self.inner.request()
    // }

}

impl<T: CqConfig + Waitable> CompletionQueue<T> {


    /// Blocking version of [Self::read]
    /// 
    /// This call will block the calling thread until either `count` completions have been read, or a timeout occurs.
    /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    /// 
    /// Corresponds to `fi_cq_sread` with `cond` set to `NULL`.
    pub fn sread(&self, count: usize, timeout: i32) -> Result<CompletionFormat, crate::error::Error> {
        self.inner.sread(count, 0, timeout)
    }

    // /// Blocking version of [Self::read]
    // /// 
    // /// This call will block the calling thread until either `count` completions have been read, or a timeout occurs.
    // /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    // /// 
    // /// Corresponds to `fi_cq_sread` with `cond` set to `NULL`.
    // pub fn sread_in(&self, count: usize, buff: &mut CompletionFormat, timeout: i32) -> Result<(), crate::error::Error> {
    //     // assert!(count <= buff.len());
    //     self.inner.sread_in(count, buff, 0, timeout)
    // }

    /// Blocking version of [Self::read]
    /// 
    /// This call will block the calling thread until either `count` completions have been read, or a timeout occurs.
    /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    /// 
    /// Corresponds to `fi_cq_sread` with `cond` set to `NULL`.
    pub unsafe fn sread_in_unchecked(&self, count: usize, buff: &mut CompletionFormat, timeout: i32) -> Result<(), crate::error::Error> {
        self.inner.sread_in(count, buff, 0, timeout)
    }

    /// Similar to  [Self::sread] with the ability to set a condition to unblock
    /// 
    /// This call will block the calling thread until `count` completions have been read, a timeout occurs or condition `cond` is met.
    /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    /// 
    /// Corresponds to `fi_cq_sread`
    pub fn sread_with_cond(&self, count: usize, cond: usize, timeout: i32) -> Result<CompletionFormat, crate::error::Error> {
        self.inner.sread(count, cond, timeout)
    }

    /// Similar to  [Self::sread] with the ability to set a condition to unblock
    /// 
    /// This call will block the calling thread until `count` completions have been read, a timeout occurs or condition `cond` is met.
    /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    /// 
    /// Corresponds to `fi_cq_sread`
    pub fn sread_with_cond_in_unchecked(&self, count: usize, buff: &mut CompletionFormat, cond: usize, timeout: i32) -> Result<(), crate::error::Error> {
        // assert!(count <= buff.len());
        self.inner.sread_in(count, buff, cond, timeout)
    }

    /// Blocking version of [Self::readfrom]
    /// 
    /// Operates the same as [`Self::sread`] with the exception that the call will also return the source address when it unblocks
    /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    /// 
    /// Corresponds to `fi_cq_sreadfrom` with `cond` set to `NULL`.
    pub fn sreadfrom(&self, count: usize, timeout: i32) -> Result<(CompletionFormat, Option<MappedAddress>), crate::error::Error> {
        self.inner.sreadfrom(count, 0, timeout)
    }

    // /// Blocking version of [Self::readfrom]
    // /// 
    // /// Operates the same as [`Self::sread`] with the exception that the call will also return the source address when it unblocks
    // /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    // /// 
    // /// Corresponds to `fi_cq_sreadfrom` with `cond` set to `NULL`.
    // pub fn sreadfrom_in(&self, count: usize, buff: &mut CompletionFormat, timeout: i32) -> Result<Option<MappedAddress>, crate::error::Error> {
    //     // assert!(count <= buff.len());
    //     self.inner.sreadfrom_in(count, buff, 0, timeout)
    // }

    /// Blocking version of [Self::readfrom]
    /// 
    /// Operates the same as [`Self::sread`] with the exception that the call will also return the source address when it unblocks
    /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    /// 
    /// Corresponds to `fi_cq_sreadfrom` with `cond` set to `NULL`.
    pub unsafe fn sreadfrom_in_unchecked(&self, count: usize, buff: &mut CompletionFormat, timeout: i32) -> Result<Option<MappedAddress>, crate::error::Error> {
        self.inner.sreadfrom_in(count, buff, 0, timeout)
    }

    /// Similar to  [Self::sreadfrom] with the ability to set a condition to unblock
    /// 
    /// This call will block the calling thread until `count` completions have been read, a timeout occurs or condition `cond` is met.
    /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    /// 
    /// Corresponds to `fi_cq_sreadfrom`
    pub fn sreadfrom_with_cond(&self, count: usize, cond: usize, timeout: i32) -> Result<(CompletionFormat, Option<MappedAddress>), crate::error::Error> {
        self.inner.sreadfrom(count, cond, timeout)
    }

    // pub async fn sreadfrom_with_cond_async(&self, count: usize, cond: usize, timeout: i32) -> Result<(CompletionFormat, Option<MappedAddress>), crate::error::Error> {
    //     self.inner.sreadfrom(count, cond, timeout)
    // }

    // /// Similar to  [Self::sreadfrom] with the ability to set a condition to unblock
    // /// 
    // /// This call will block the calling thread until `count` completions have been read, a timeout occurs or condition `cond` is met.
    // /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    // /// 
    // /// Corresponds to `fi_cq_sreadfrom`
    // pub fn sreadfrom_with_cond_in(&self, count: usize, buff: &mut CompletionFormat, cond: usize, timeout: i32) -> Result<Option<MappedAddress>, crate::error::Error> {
    //     // assert!(count <= buff.len());
    //     self.inner.sreadfrom_in(count, buff, cond, timeout)
    // }

    /// Similar to  [Self::sreadfrom] with the ability to set a condition to unblock
    /// 
    /// This call will block the calling thread until `count` completions have been read, a timeout occurs or condition `cond` is met.
    /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    /// 
    /// Corresponds to `fi_cq_sreadfrom`
    pub unsafe fn sreadfrom_with_cond_in_unchecked(&self, count: usize, buff: &mut CompletionFormat, cond: usize, timeout: i32) -> Result<Option<MappedAddress>, crate::error::Error> {
        self.inner.sreadfrom_in(count, buff, cond, timeout)
    }

    /// Unblock any thread waiting in [Self::sread], [Self::sreadfrom], [Self::sread_with_cond]
    /// 
    /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    /// 
    /// Corresponds to `fi_cq_signal`
    pub fn signal(&self) -> Result<(), crate::error::Error>{
        self.inner.signal()
    }
}

impl<'a, T: CqConfig + WaitRetrievable> CompletionQueue<T> { //[TODO] Make this a method of the trait ?

    /// Retreives the low-level wait object associated with the counter.
    /// 
    /// This method is available only if the counter has been configured with a retrievable
    /// underlying wait object.
    /// 
    /// Corresponds to `fi_cntr_control` with command `FI_GETWAIT`.
    pub fn wait_object(&self) -> Result<WaitObjType<'a>, crate::error::Error> {
        self.inner.wait_object()
    }
}

pub struct AsyncCompletionQueueImpl {
    base: Async<CompletionQueueImpl>,
}

impl Deref for  AsyncCompletionQueueImpl {
    type Target = CompletionQueueImpl;

    fn deref(&self) -> &Self::Target {
        self.get_inner()
    }
}
impl AsyncCompletionQueueImpl {

    pub(crate) fn get_inner(&self) -> &CompletionQueueImpl {
        self.base.get_ref()
    }

    pub(crate) fn new<T0>(domain: &Rc<AsyncDomainImpl>, attr: CompletionQueueAttr, context: Option<&mut T0>, default_buff_size: usize) -> Result<Self, crate::error::Error> {
        Ok(Self {base:Async::new(CompletionQueueImpl::new(domain, attr, context, default_buff_size)?).unwrap()})
    }

    pub(crate) async fn read_async<'a>(&'a self, buf: &'a mut CompletionFormat, count: usize) -> Result<(), crate::error::Error>  {
        // println!("Calling read async");
        // CqAsyncRead{num_entries: count, buf, cq: self}
        loop {

            let (err, _guard) = if self._domain_rc.get_fabric_impl().trywait(&[&self]).is_err() {
                // println!("Cannot block");
                (self.read_in(count, buf), None)
            }
            else {
                let _guard = self.base.readable().await.unwrap();
                (self.read_in(count, buf), Some(_guard))
            };
            
            match err {
                Err(error) => {
                    if !matches!(error.kind, crate::error::ErrorKind::TryAgain) {
                        return Err(error);
                    }
                    else {
                        // println!("Will continue");
                        #[cfg(feature = "use-tokio")]
                        match _guard {
                            Some(mut guard) => {if self.pending_entries.borrow().is_empty() {guard.clear_ready()}},
                            None => {}
                        }
                        continue;
                    }
                },
                Ok(len) => {      
                    // println!("Actually read something {}", len);          
                    match buf {
                        CompletionFormat::Unspec(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Ctx(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Msg(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Data(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Tagged(data) => unsafe{data.set_len(len)},
                    }
                    return Ok(())
                },
            }
        }
    }

    pub(crate) async fn async_transfer_wait(&self, async_ctx: &mut AsyncCtx) -> Result<SingleCompletionFormat, crate::error::Error> {
        let mut buf = alloc_cq_entry!(*self.entry_buff.borrow(), 1);
        loop {
            if let Some(mut entry) = self.pending_entries.borrow_mut().remove(&((async_ctx as *mut AsyncCtx) as usize)) {
                match entry {
                    SingleCompletionFormat::Unspec(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                    SingleCompletionFormat::Ctx(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                    SingleCompletionFormat::Msg(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                    SingleCompletionFormat::Data(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                    SingleCompletionFormat::Tagged(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                }
                // println!("Returning!");
                return Ok(entry);
            }
            
            // println!("About to block, waiting for transfer");
            
            let (err, mut _guard) = if self._domain_rc.get_fabric_impl().trywait(&[&self]).is_err() {
                // println!("Cannot block");
                (self.read_in(1, &mut buf), None)
            }
            else {
                let _guard = self.base.readable().await.unwrap();
                (self.read_in(1, &mut buf), Some(_guard))
            };
                
            match err {
                Err(error) => {
                    if !matches!(error.kind, crate::error::ErrorKind::TryAgain) {
                        return Err(error);
                    }
                    else {
                        // println!("Will continue");
                        #[cfg(feature = "use-tokio")]
                        match _guard {
                            Some(mut _guard) => {if self.pending_entries.borrow().is_empty() {_guard.clear_ready()}},
                            None => {}
                        }
                        continue;
                    }
                },
                Ok(len) => {      
                    // println!("Actually read something {}", len);          
                    match &mut buf {
                        CompletionFormat::Unspec(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Ctx(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Msg(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Data(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Tagged(data) => unsafe{data.set_len(len)},
                    }
                },
            }
            match &buf {
                CompletionFormat::Unspec(entries) => for e in entries.iter() {self.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Unspec(e.clone()));},
                CompletionFormat::Ctx(entries) => for e in entries.iter() {self.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Ctx(e.clone()));},
                CompletionFormat::Msg(entries) => for e in entries.iter() {self.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Msg(e.clone()));},
                CompletionFormat::Data(entries) => for e in entries.iter() {self.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Data(e.clone()));},
                CompletionFormat::Tagged(entries) => for e in entries.iter() {self.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Tagged(e.clone()));},
            }
        }
    }
}


pub(crate) struct AsyncTransferCq{
    // pub(crate) req: usize,
    pub(crate) buf: CompletionFormat,
    pub(crate) cq: Rc<AsyncCompletionQueueImpl>,
    pub(crate) ctx: usize,
}

impl AsyncTransferCq {
    #[allow(dead_code)]
    pub(crate) fn new(cq: Rc<AsyncCompletionQueueImpl>, ctx: usize) -> Self {
        let buf = alloc_cq_entry!(*cq.entry_buff.borrow(), 1); 
        Self {
            buf,
            cq,
            ctx, 
        }
    }
}


impl Future for AsyncTransferCq {
    type Output=Result<SingleCompletionFormat, Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let mut_self = self.get_mut();
        loop {
            if let Some(mut entry) = mut_self.cq.pending_entries.borrow_mut().remove(&mut_self.ctx) {
                match entry {
                    SingleCompletionFormat::Unspec(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                    SingleCompletionFormat::Ctx(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                    SingleCompletionFormat::Msg(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                    SingleCompletionFormat::Data(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                    SingleCompletionFormat::Tagged(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                }
                // println!("Returning!");
                return std::task::Poll::Ready(Ok(entry));
            }
            
            // println!("About to block, waiting for transfer");
            
            let (err, _guard) = if mut_self.cq._domain_rc.get_fabric_impl().trywait(&[&mut_self.cq]).is_err() {
                // println!("Cannot block");
                (mut_self.cq.read_in(1, &mut mut_self.buf), None)
            }
            else {

                #[cfg(feature = "use-tokio")]
                let _guard = ready!(mut_self.cq.base.poll_read_ready(cx)).unwrap();
                #[cfg(feature = "use-async-std")]
                let _guard = ready!(mut_self.cq.base.poll_readable(cx)).unwrap();
                // println!("Did not block");
                (mut_self.cq.read_in(1, &mut mut_self.buf), Some(_guard))
            };
                
            match err {
                Err(error) => {
                    if !matches!(error.kind, crate::error::ErrorKind::TryAgain) {
                        return std::task::Poll::Ready(Err(error));
                    }
                    else {
                        // println!("Will continue");
                        #[cfg(feature = "use-tokio")]
                        match _guard {
                            Some(mut guard) => {if mut_self.cq.pending_entries.borrow().is_empty() {guard.clear_ready()}},
                            None => {}
                        }
                        continue;
                    }
                },
                Ok(len) => {      
                    // println!("Actually read something {}", len);          
                    match &mut mut_self.buf {
                        CompletionFormat::Unspec(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Ctx(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Msg(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Data(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Tagged(data) => unsafe{data.set_len(len)},
                    }
                },
            }
            match &mut_self.buf {
                CompletionFormat::Unspec(entries) => for e in entries.iter() {mut_self.cq.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Unspec(e.clone()));},
                CompletionFormat::Ctx(entries) => for e in entries.iter() {mut_self.cq.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Ctx(e.clone()));},
                CompletionFormat::Msg(entries) => for e in entries.iter() {mut_self.cq.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Msg(e.clone()));},
                CompletionFormat::Data(entries) => for e in entries.iter() {mut_self.cq.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Data(e.clone()));},
                CompletionFormat::Tagged(entries) => for e in entries.iter() {mut_self.cq.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Tagged(e.clone()));},
            }
        }
    }
}

pub struct CqAsyncRead<'a>{
    num_entries: usize,
    buf: &'a mut CompletionFormat,
    cq: &'a AsyncCompletionQueueImpl,
}


impl<'a> Future for CqAsyncRead<'a>{
    type Output=Result<(), Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let mut_self = self.get_mut();
        loop {
            let (err, _guard) = if mut_self.cq._domain_rc.get_fabric_impl().trywait(&[&mut_self.cq]).is_err() {
                // println!("Cannot block");
                (mut_self.cq.read_in(1, &mut mut_self.buf), None)
            }
            else {

                #[cfg(feature = "use-tokio")]
                let _guard = ready!(mut_self.cq.base.poll_read_ready(cx)).unwrap();
                #[cfg(feature = "use-async-std")]
                let _guard = ready!(mut_self.cq.base.poll_readable(cx)).unwrap();
                // println!("Did not block");
                (mut_self.cq.read_in(1, &mut mut_self.buf), Some(_guard))
            };
                
            match err {
                Err(error) => {
                    if !matches!(error.kind, crate::error::ErrorKind::TryAgain) {
                        return std::task::Poll::Ready(Err(error));
                    }
                    else {
                        // println!("Will continue");
                        #[cfg(feature = "use-tokio")]
                        match _guard {
                            Some(mut guard) => {if mut_self.cq.pending_entries.borrow().is_empty() {guard.clear_ready()}},
                            None => {}
                        }
                        continue;
                    }
                },
                Ok(len) => {      
                    // println!("Actually read something {}", len);          
                    match &mut mut_self.buf {
                        CompletionFormat::Unspec(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Ctx(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Msg(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Data(data) => unsafe{data.set_len(len)},
                        CompletionFormat::Tagged(data) => unsafe{data.set_len(len)},
                    }
                },
            }
            //         CompletionFormat::Data(data) => unsafe{data.set_len(ret.try_into().unwrap())},
            //         CompletionFormat::Tagged(data) => unsafe{data.set_len(ret.try_into().unwrap())},
            //     }
            //     // Ok(buf)
            //     return std::task::Poll::Ready(Ok(()));
            // }
        }
    }
}



impl AsFid for AsyncCompletionQueueImpl {
    fn as_fid(&self) -> crate::fid::BorrowedFid<'_> {
        self.c_cq.as_fid()
    }
}
impl AsFid for &AsyncCompletionQueueImpl {
    fn as_fid(&self) -> crate::fid::BorrowedFid<'_> {
        self.c_cq.as_fid()
    }
}
impl AsFid for Rc<AsyncCompletionQueueImpl> {
    fn as_fid(&self) -> crate::fid::BorrowedFid<'_> {
        self.c_cq.as_fid()
    }
}


impl AsRawFid for AsyncCompletionQueueImpl {
    fn as_raw_fid(&self) -> RawFid {
        self.c_cq.as_raw_fid()
    }
}

impl crate::BindImpl for AsyncCompletionQueueImpl {}
impl<T: CqConfig + 'static> crate::Bind for CompletionQueue<T> {
    fn inner(&self) -> Rc<dyn crate::BindImpl> {
        self.inner.clone()
    }
}


pub struct CompletionQueueBuilder<'a, T> {
    cq_attr: CompletionQueueAttr,
    domain: &'a Domain,
    ctx: Option<&'a mut T>,
    options: cqoptions::Options<cqoptions::WaitRetrieve, cqoptions::On>,
    default_buff_size: usize,
}

    
impl<'a> CompletionQueueBuilder<'a, ()> {
    
    /// Initiates the creation of a new [CompletionQueue] on `domain`.
    /// 
    /// The initial configuration is what would be set if no `fi_cq_attr` or `context` was provided to 
    /// the `fi_cq_open` call. 
    pub fn new(domain: &'a Domain) -> CompletionQueueBuilder<()> {
        Self  {
            cq_attr: CompletionQueueAttr::new(),
            domain,
            ctx: None,
            options: Options::new().wait_fd(),
            default_buff_size: 10,
        }
    }
}

impl<'a, T> CompletionQueueBuilder<'a, T> {

    /// Specifies the minimum size of a completion queue.
    /// 
    /// Corresponds to setting the field `fi_cq_attr::size` to `size`.
    pub fn size(mut self, size: usize) -> Self {
        self.cq_attr.size(size);
        self
    }


    pub fn signaling_vector(mut self, signaling_vector: i32) -> Self { // [TODO]
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
    pub fn context(self, ctx: &'a mut T) -> CompletionQueueBuilder<'a, T> {
        CompletionQueueBuilder {
            ctx: Some(ctx),
            cq_attr: self.cq_attr,
            domain: self.domain,
            options: self.options,
            default_buff_size: self.default_buff_size,
        }
    }

    /// Constructs a new [CompletionQueue] with the configurations requested so far.
    /// 
    /// Corresponds to creating a `fi_cq_attr`, setting its fields to the requested ones,
    /// and passing it to the `fi_cq_open` call with an optional `context`.
    pub fn build(mut self) ->  Result<CompletionQueue<Options<cqoptions::WaitRetrieve, cqoptions::On>>, crate::error::Error> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::Fd);
        CompletionQueue::new(self.options, self.domain, self.cq_attr, self.ctx, self.default_buff_size)   
    }
}

#[cfg(test)]
mod tests {

    use crate::async_::{cq::*, domain::DomainBuilder};
    use crate::info::Info;

    #[test]
    fn cq_open_close_simultaneous() {
        let info = Info::new().request().unwrap();
        let entries = info.get();
        
        let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
        let count = 10;
        let domain = DomainBuilder::new(&fab, &entries[0]).build().unwrap();
        // let mut cqs = Vec::new();
        for _ in 0..count {
            let _cq = CompletionQueueBuilder::new(&domain).build().unwrap();
        }
    }

    #[test]
    fn cq_open_close_sizes() {
        let info = Info::new().request().unwrap();
        let entries = info.get();
        
        let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
        let domain = DomainBuilder::new(&fab, &entries[0]).build().unwrap();
        for i in -1..17 {
            let size = if i == -1 { 0 } else { 1 << i };
            let _cq = CompletionQueueBuilder::new(&domain).size(size)
                .build()
                .unwrap();
        }
    }
}

#[cfg(test)]
mod libfabric_lifetime_tests {
    use crate::async_::{cq::*, domain::DomainBuilder};
    use crate::info::Info;

    #[test]
    fn cq_drops_before_domain() {
        let info = Info::new().request().unwrap();
        let entries = info.get();
        
        let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
        let count = 10;
        let domain = DomainBuilder::new(&fab, &entries[0]).build().unwrap();
        let mut cqs = Vec::new();
        for _ in 0..count {
            let cq = CompletionQueueBuilder::new(&domain)
                .build()
                .unwrap();
            println!("Count = {}", std::rc::Rc::strong_count(&domain.inner));
            cqs.push(cq);
        }
        drop(domain);
        println!("Count = {} After dropping domain\n", std::rc::Rc::strong_count(&cqs[0].inner._domain_rc));

    }
}