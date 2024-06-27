use std::{rc::{Rc, Weak}, collections::HashMap, cell::RefCell, ops::Deref, marker::PhantomData};

use async_io::Async;

use crate::{eq::{Event, EventQueueImpl, EventQueueAttr, EventQueueBase, BindEqImpl, EventQueueImplT}, error::Error, fid::{AsFid, self, RawFid, AsRawFid, AsRawTypedFid}, eqoptions::{self, Options}, FdRetrievable, WaitRetrievable, mr::MemoryRegionImplBase, av::AddressVectorImplBase, comm::collective::MulticastGroupCollectiveImplBase, async_::AsyncCtx};
use super::{mr::AsyncMemoryRegionImpl, av::AsyncAddressVectorImpl, comm::collective::AsyncMulticastGroupCollectiveImpl, cq::AsyncCompletionQueueImpl};

pub type EventQueue<T> = EventQueueBase<T, AsyncEventQueueImpl>;

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
        let fid = self.eq.as_raw_typed_fid();
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
                else if ev.eq.base.poll_readable(cx).is_ready() {
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

impl<T: eqoptions::EqConfig + WaitRetrievable + FdRetrievable> EventQueue<T> {
    pub async fn read_async(&self) -> Result<Event<usize>, crate::error::Error>{
        self.inner.read_async().await
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

pub struct AsyncEventQueueImpl {
    pub(crate) base: Async<EventQueueImpl>,
    pending_entries: RefCell<HashMap<(u32, usize), Event<usize>>>,
    pending_cm_entries: RefCell<HashMap<(u32,libfabric_sys::fid_t), Vec::<Event<usize>> >>,
    pub(crate) mrs: RefCell<std::collections::HashMap<RawFid, Weak<AsyncMemoryRegionImpl>>>,   // We neep maps Fid -> MemoryRegionImpl/AddressVectorImpl/MulticastGroupCollectiveImpl, however, we don't want to extend 
    // the lifetime of any of these objects just because of the maps.
    // Moreover, these objects will keep references to the EQ to keep it from dropping while
    // they are still bound, thus, we would have cyclic references that wouldn't let any 
    // of the two sides drop. 
    pub(crate) avs: RefCell<std::collections::HashMap<RawFid, Weak<AsyncAddressVectorImpl>>>,
    pub(crate) mcs: RefCell<std::collections::HashMap<RawFid, Weak<AsyncMulticastGroupCollectiveImpl>>>,
}
impl AsyncEventQueueImpl {
    pub(crate) fn new<T0>(fabric: &Rc<crate::fabric::FabricImpl>, attr: EventQueueAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        Ok(Self {
            base: Async::new(EventQueueImpl::new(fabric, attr, context)?).unwrap(),
            pending_entries: RefCell::new(HashMap::new()),
            pending_cm_entries: RefCell::new(HashMap::new()),
            mrs: RefCell::new(HashMap::new()),
            avs: RefCell::new(HashMap::new()),
            mcs: RefCell::new(HashMap::new()),
        })
    }

    pub(crate) fn bind_mr(&self, mr: &Rc<AsyncMemoryRegionImpl>) {
        self.mrs.borrow_mut().insert(mr.as_fid().as_raw_fid(), Rc::downgrade(mr));
    }

    pub(crate) fn bind_av(&self, av: &Rc<AsyncAddressVectorImpl>) {
        self.avs.borrow_mut().insert(av.as_fid().as_raw_fid(), Rc::downgrade(av));
    }

    pub(crate) fn bind_mc(&self, mc: &Rc<AsyncMulticastGroupCollectiveImpl>) {
        self.mcs.borrow_mut().insert(mc.as_fid().as_raw_fid(), Rc::downgrade(mc));
    }
}

impl Deref for  AsyncEventQueueImpl {
    type Target = EventQueueImpl;

    fn deref(&self) -> &Self::Target {
        self.base.as_ref()
    }
}


impl EventQueueImplT for AsyncEventQueueImpl {
    fn read(&self) -> Result<Event<usize>, crate::error::Error> {
        self.base.as_ref().read()
    }

    fn peek(&self) -> Result<Event<usize>, crate::error::Error> {
        self.base.as_ref().peek()
    }

    fn readerr(&self) -> Result<crate::eq::EventError, crate::error::Error> {
        self.base.as_ref().readerr()
    }

    fn peekerr(&self) -> Result<crate::eq::EventError, crate::error::Error> {
        self.base.as_ref().peekerr()
    }

    fn strerror(&self, entry: &crate::eq::EventError) -> &str {
        self.base.as_ref().strerror(entry)
    }
}

pub(crate) trait AsyncEventQueueImplT {
    async fn read_async(&self) -> Result<Event<usize>, crate::error::Error>;
}

impl AsyncEventQueueImplT for AsyncEventQueueImpl {
    
    async fn read_async(&self) -> Result<Event<usize>, crate::error::Error> {
        self.read_async().await
    }
}


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

impl AsFid for AsyncEventQueueImpl {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
       self.c_eq.as_fid()
    }
}

impl AsRawFid for AsyncEventQueueImpl {
    fn as_raw_fid(&self) -> RawFid {
       self.c_eq.as_raw_fid()
    }
}

impl crate::BindImpl for AsyncEventQueueImpl {}

impl BindEqImpl<AsyncEventQueueImpl, AsyncCompletionQueueImpl> for AsyncEventQueueImpl {
    fn bind_mr(&self, mr: &Rc<MemoryRegionImplBase<AsyncEventQueueImpl>>) {
        self.bind_mr(mr);
    }

    fn bind_av(&self, av: &Rc<AddressVectorImplBase<AsyncEventQueueImpl>>) {
        self.bind_av(av);
    }

    fn bind_mc(&self, mc: &Rc<MulticastGroupCollectiveImplBase<AsyncEventQueueImpl, AsyncCompletionQueueImpl>>) {
        self.bind_mc(mc);
    }
}

pub struct EventQueueBuilder<'a, T, WRITE> {
    eq_attr: EventQueueAttr,
    fabric: &'a crate::fabric::Fabric,
    ctx: Option<&'a mut T>,
    options: eqoptions::Options<WRITE, eqoptions::WaitRetrieve, eqoptions::On>,
}

impl<'a> EventQueueBuilder<'a, (), eqoptions::Off> {
    pub fn new(fabric: &'a crate::fabric::Fabric) -> Self {
       Self {
            eq_attr: EventQueueAttr::new(),
            fabric,
            ctx: None,
            options: Options::new().wait_fd(),
        }
    }
}

impl <'a, T, WRITE> EventQueueBuilder<'a, T, WRITE> {
    
    pub fn size(mut self, size: usize) -> Self {
        self.eq_attr.size(size);
        self
    }

    pub fn write(mut self) -> EventQueueBuilder<'a, T, eqoptions::On> {
        self.eq_attr.write();

        EventQueueBuilder {
            options: self.options.writable(),
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
            options: self.options,
        }
    }

    pub fn build(mut self) ->  Result<EventQueue<Options<WRITE, eqoptions::WaitRetrieve, eqoptions::On>>, crate::error::Error> {
        self.eq_attr.wait_obj(crate::enums::WaitObj::Fd);

        EventQueue::new(self.options, self.fabric, self.eq_attr, self.ctx)   
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
        println!("Count = {} After dropping fab\n", std::rc::Rc::strong_count(&eqs[0].inner._fabric_rc));
    }
}