use std::{marker::PhantomData, os::fd::{AsFd, BorrowedFd}, rc::Rc, cell::RefCell, ops::Deref, collections::HashMap};

use async_io::Async;

#[allow(unused_imports)]
use crate::fid::AsFid;
use crate::{cqoptions::{self, CqConfig, Options}, domain::{Domain, DomainImpl}, enums::{WaitObjType, CompletionFlags}, MappedAddress, FdRetrievable, WaitRetrievable, Waitable, fid::{OwnedFid, AsRawFid}, RawMappedAddress, error::Error};

//================== CompletionQueue (fi_cq) ==================//

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
macro_rules! read_cq_entry_ {
    ($read_fn: expr, $cq: expr, $count: expr, $entries: expr, $( $x:ident),*) => {
        {
            let capacity = $entries.capacity();
            if $count > capacity {
                $entries.reserve($count - capacity);
            }
            let err = unsafe{ $read_fn($cq, $entries.as_mut_ptr().cast(), $count, $($x,)*)};
            if err >= 0 {
                unsafe {$entries.set_len(err as usize)};
                $entries.shrink_to_fit();
            }
            err
        }
    }
}

macro_rules! read_cq_entry {
    ($read_fn: expr, $handle: expr, $count: expr, $entries: expr, $( $x:ident),*) => {
        match $entries {
            CompletionFormat::Unspec(ref mut cv) => {read_cq_entry_!($read_fn,  $handle, $count, cv, $( $x),*)},  
            CompletionFormat::Ctx(ref mut cv) => read_cq_entry_!($read_fn,  $handle, $count, cv, $( $x),*),  
            CompletionFormat::Msg(ref mut mv) => read_cq_entry_!($read_fn,  $handle, $count, mv, $( $x),*), 
            CompletionFormat::Data(ref mut dv) => read_cq_entry_!($read_fn,  $handle, $count, dv, $( $x),*), 
            CompletionFormat::Tagged(ref mut tv) => read_cq_entry_!($read_fn,  $handle, $count, tv, $( $x),*),
        }
    }
}
// macro_rules! read_cq_entry_in_slice {
//     ($read_fn: expr, $format: expr, $cq: expr, $count: expr,  $t: ty, $entries: expr, $( $x:ident),*) => {
//         {
//             let err = unsafe{ $read_fn($cq, $entries.as_mut_ptr().cast(), $count, $($x,)*)};
//             err
//         }
//     }
// }

// #[derive(Clone)]
pub trait EntryFormat: Clone{}

impl EntryFormat for libfabric_sys::fi_cq_entry{}
impl EntryFormat for libfabric_sys::fi_cq_msg_entry{}
impl EntryFormat for libfabric_sys::fi_cq_data_entry{}
impl EntryFormat for libfabric_sys::fi_cq_tagged_entry{}
impl EntryFormat for (){}

pub type CtxEntry = libfabric_sys::fi_cq_entry;
pub type MsgEntry = libfabric_sys::fi_cq_msg_entry;
pub type DataEntry = libfabric_sys::fi_cq_data_entry;
pub type TaggedEntry = libfabric_sys::fi_cq_tagged_entry;
pub type UnspecEntry = ();

#[derive(Clone)]
pub enum CompletionFormat {
    Unspec(Vec<Completion<CtxEntry>>), // fi_cq_entry seems to be the bare minimum needed
    Ctx(Vec<Completion<CtxEntry>>),
    Msg(Vec<Completion<MsgEntry>>),
    Data(Vec<Completion<DataEntry>>),
    Tagged(Vec<Completion<TaggedEntry>>),
}

pub enum SingleCompletionFormat {
    Unspec(Completion<CtxEntry>),
    Ctx(Completion<CtxEntry>),
    Msg(Completion<MsgEntry>),
    Data(Completion<DataEntry>),
    Tagged(Completion<TaggedEntry>),
}

macro_rules! read_cq_entry_into {
    ($read_fn: expr, $cq: expr, $count: expr, $buff: expr, $( $x:ident),*) => {
        unsafe{ $read_fn($cq, $buff, $count, $($x,)*)}
    }
}

struct CqAsyncRead<'a>{
    num_entries: usize,
    buf: *mut std::ffi::c_void,
    cq: &'a AsyncCompletionQueueImpl,
}


impl<'a> async_std::future::Future for CqAsyncRead<'a>{
    type Output=Result<usize, Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        // let mut buff = vec![1u8];
        // self.poll_read(cx, &mut buff[..])
        loop {
            let err = read_cq_entry_into!(libfabric_sys::inlined_fi_cq_read, self.cq.0.as_ref().handle(), self.num_entries, self.buf,);
            if err < 0 {
                let err = Error::from_err_code(-err as u32);
                if !matches!(err.kind, crate::error::ErrorKind::TryAgain) 
                {
                    return std::task::Poll::Ready(Err(err));
                }
                else if self.cq.0.poll_readable(cx).is_ready() {
                    continue;
                 }
                else {
                    return std::task::Poll::Pending;
                }
            }
            else {
                return std::task::Poll::Ready(Ok(err as usize));
            }
        }
    }
}

pub(crate) struct AsyncCompletionQueueImpl(pub(crate) Async<CompletionQueueImpl>);
impl Deref for  AsyncCompletionQueueImpl {
    type Target = CompletionQueueImpl;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl AsyncCompletionQueueImpl {

    pub(crate) fn new<T0>(domain: &Rc<crate::domain::DomainImpl>, attr: CompletionQueueAttr, context: Option<&mut T0>, default_buff_size: usize) -> Result<Self, crate::error::Error> {
        Ok(Self (Async::new(CompletionQueueImpl::new(domain, attr, context, default_buff_size)?).unwrap()))
    }

    pub(crate) async fn read_async(&self, count: usize) -> Result<CompletionFormat, crate::error::Error> {

        let mut buf = alloc_cq_entry!(*self.0.as_ref().entry_buff.borrow(), count);
        let fut = match &mut buf {
            CompletionFormat::Unspec(data) => {CqAsyncRead{num_entries: count, buf: data.as_mut_ptr().cast(), cq: self}},
            CompletionFormat::Ctx(data) => {CqAsyncRead{num_entries: count, buf: data.as_mut_ptr().cast(), cq: self}},
            CompletionFormat::Msg(data) => {CqAsyncRead{num_entries: count, buf: data.as_mut_ptr().cast(), cq: self}},
            CompletionFormat::Data(data) => {CqAsyncRead{num_entries: count, buf: data.as_mut_ptr().cast(), cq: self}},
            CompletionFormat::Tagged(data) => {CqAsyncRead{num_entries: count, buf: data.as_mut_ptr().cast(), cq: self}},
        };
        let ret = fut.await?;

        // *self.0.as_ref().completions.borrow_mut() += ret;
        // println!("Complete: {}/{}", self.0.as_ref().completions.borrow(), self.0.as_ref().requests.borrow());

        match &mut buf {
            CompletionFormat::Unspec(data) => unsafe{data.set_len(ret)},
            CompletionFormat::Ctx(data) => unsafe{data.set_len(ret)},
            CompletionFormat::Msg(data) => unsafe{data.set_len(ret)},
            CompletionFormat::Data(data) => unsafe{data.set_len(ret)},
            CompletionFormat::Tagged(data) => unsafe{data.set_len(ret)},
        }
        Ok(buf)
    }
}

pub(crate) struct CompletionQueueImpl {
    c_cq: *mut libfabric_sys::fid_cq,
    entry_buff: RefCell<CompletionFormat>,
    error_buff: RefCell<CompletionError>,
    fid: OwnedFid,
    requests: RefCell<usize>,
    completions: RefCell<usize>,
    pending_entries: RefCell<HashMap<usize, SingleCompletionFormat>>,
    wait_obj: Option<libfabric_sys::fi_wait_obj>,
    _domain_rc: Rc<DomainImpl>,
}


/// Owned wrapper around a libfabric `fid_cq`.
/// 
/// This type wraps an instance of a `fid_cq`, monitoring its lifetime and closing it when it goes out of scope.
/// To be able to check its configuration at compile this object is extended with a `T:`[`CqConfig`] (e.g. [Options]) that provides this information.
/// 
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.19.0/man/fi_cq.3.html).
/// 
/// Note that other objects that rely on a CompletQueue (e.g., an [crate::ep::Endpoint] bound to it) will extend its lifetime until they
/// are also dropped.
pub struct CompletionQueue<T: CqConfig> {
    pub(crate) inner: Rc<AsyncCompletionQueueImpl>,
    phantom: PhantomData<T>,
}

impl<'a> CompletionQueueImpl {

    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_cq {
        self.c_cq
    }

    pub(crate) fn new<T0>(domain: &Rc<crate::domain::DomainImpl>, mut attr: CompletionQueueAttr, context: Option<&mut T0>, default_buff_size: usize) -> Result<Self, crate::error::Error> {
        
        let mut c_cq: *mut libfabric_sys::fid_cq  = std::ptr::null_mut();
        let c_cq_ptr: *mut *mut libfabric_sys::fid_cq = &mut c_cq;
        // let mut entries: Vec<$t> = Vec::with_capacity(std::mem::size_of::<$t>() * $count);

        let err = 
            if let Some(ctx) = context {
                unsafe {libfabric_sys::inlined_fi_cq_open(domain.handle(), attr.get_mut(), c_cq_ptr, (ctx as *mut T0).cast())}
            }
            else {
                unsafe {libfabric_sys::inlined_fi_cq_open(domain.handle(), attr.get_mut(), c_cq_ptr, std::ptr::null_mut())}
            };
        
        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(
                Self {
                    c_cq, 
                    fid: OwnedFid::from(unsafe{&mut (*c_cq).fid }), 
                    wait_obj: Some(attr.c_attr.wait_obj),
                    completions: RefCell::new(0),
                    pending_entries: RefCell::new(HashMap::new()),
                    requests: RefCell::new(0),
                    _domain_rc: domain.clone(),
                    entry_buff: 
                        if attr.c_attr.format == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_UNSPEC {
                            RefCell::new(CompletionFormat::Unspec(Vec::with_capacity(default_buff_size)))
                        }
                        else if attr.c_attr.format == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_CONTEXT {
                            RefCell::new(CompletionFormat::Ctx(Vec::with_capacity(default_buff_size)))
                        }
                        else if attr.c_attr.format == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_MSG {
                            RefCell::new(CompletionFormat::Msg(Vec::with_capacity(default_buff_size)))
                        }
                        else if attr.c_attr.format == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_TAGGED {
                            RefCell::new(CompletionFormat::Tagged(Vec::with_capacity(default_buff_size)))
                        }
                        else if attr.c_attr.format == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_DATA {
                            RefCell::new(CompletionFormat::Data(Vec::with_capacity(default_buff_size)))
                        }
                        else {
                            panic!("Unexpected CompletionQueue type");
                        },
                    
                    error_buff: RefCell::new(CompletionError::new()),
                })
        }
    }

    //[TODO] Maybe avoid making the extra copies for the final vector
    pub(crate) fn read(&self, count: usize) -> Result<CompletionFormat, crate::error::Error> {
        let err = {
            let mut borrowed_entries = self.entry_buff.borrow_mut();
            read_cq_entry!(libfabric_sys::inlined_fi_cq_read, self.handle(), count,  *borrowed_entries, )
        };
        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            Ok(self.entry_buff.borrow().clone())
        }
    }

    pub(crate) fn read_in(&self, count: usize, buffer: &mut CompletionFormat) -> Result<(), crate::error::Error> {
        let err = read_cq_entry!(libfabric_sys::inlined_fi_cq_read, self.handle(), count, buffer, );
        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            // *self.completions.borrow_mut() += err as usize;
            // println!("Complete: {}/{}", self.completions.borrow(), self.requests.borrow());
            Ok(())
        }
    }


    pub(crate) fn readfrom(&self, count: usize) -> Result<(CompletionFormat, Option<MappedAddress>), crate::error::Error> {
       
        let mut address = 0;
        let p_address = &mut address as *mut libfabric_sys::fi_addr_t;    
        let err = {
            let mut borrowed_entries = self.entry_buff.borrow_mut();
            read_cq_entry!(libfabric_sys::inlined_fi_cq_readfrom, self.handle(), count, *borrowed_entries, p_address)
        };
        let address = if address == crate::FI_ADDR_NOTAVAIL {
            None
        }
        else {
            Some(MappedAddress::from_raw_addr_no_av(address))
        };
        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            *self.completions.borrow_mut() += err as usize;
            Ok((self.entry_buff.borrow().clone(), address))
        }
    }

    pub(crate) fn readfrom_in(&self, count: usize, buffer: &mut CompletionFormat) -> Result<Option<MappedAddress>, crate::error::Error> {
       
        let mut address = 0;
        let p_address = &mut address as *mut libfabric_sys::fi_addr_t;    
        let err = read_cq_entry!(libfabric_sys::inlined_fi_cq_readfrom, self.handle(), count, buffer, p_address);
        let address = if address == crate::FI_ADDR_NOTAVAIL {
            None
        }
        else {
            Some(MappedAddress::from_raw_addr_no_av(address))
        };
        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            Ok(address)
        }
    }

    pub(crate) fn readerr(&self, flags: u64) -> Result<CompletionError, crate::error::Error> {
        
        let ret = {
            let mut entry = self.error_buff.borrow_mut();
            unsafe { libfabric_sys::inlined_fi_cq_readerr(self.handle(), entry.get_mut(), flags) }
        };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()))
        }
        else {
            Ok(self.error_buff.borrow().clone())
        }
    }

    pub(crate) fn readerr_in(&self, err_buff: &mut CompletionError, flags: u64) -> Result<(), crate::error::Error> {
        
        let ret = {
            unsafe { libfabric_sys::inlined_fi_cq_readerr(self.handle(), err_buff.get_mut(), flags) }
        };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub(crate) fn print_error(&self, err_entry: &crate::cq::CompletionError) {
        println!("{}", unsafe{self.strerror(err_entry.prov_errno(), err_entry.err_data(), err_entry.err_data_size())} );
    }

    unsafe fn strerror(&self, prov_errno: i32, err_data: *const std::ffi::c_void, err_data_size: usize) -> &str {

        let ret = unsafe { libfabric_sys::inlined_fi_cq_strerror(self.handle(), prov_errno, err_data, std::ptr::null_mut() , err_data_size) };
    
            unsafe{ std::ffi::CStr::from_ptr(ret).to_str().unwrap() }
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
                let mut mutex_cond = libfabric_sys::fi_mutex_cond{
                    mutex: std::ptr::null_mut(),
                    cond: std::ptr::null_mut(),
                };

                let err = unsafe { libfabric_sys::inlined_fi_control(self.as_raw_fid(), libfabric_sys::FI_GETWAIT as i32, (&mut mutex_cond as *mut libfabric_sys::fi_mutex_cond).cast()) };
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


    // pub(crate) fn sread_with_cond(&self, count: usize, cond: usize, timeout: i32) -> Result<Completion, crate::error::Error> {
    //     let p_cond = cond as *const usize as *const std::ffi::c_void;
    //     let (err, ret) = read_cq_entry!(libfabric_sys::inlined_fi_cq_sread, self.format, self.handle(), count, p_cond, timeout);
    //     if err < 0 {
    //         Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
    //     }
    //     else {
    //         Ok(ret)
    //     }
    // }

    pub(crate) fn sread(&self, count: usize, cond: usize, timeout: i32) -> Result<CompletionFormat, crate::error::Error> {
        let p_cond = cond as *const usize as *const std::ffi::c_void;
        let err =  {
            let mut borrowed_entries = self.entry_buff.borrow_mut();
            read_cq_entry!(libfabric_sys::inlined_fi_cq_sread, self.handle(), count, *borrowed_entries, p_cond, timeout)
        };
        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            Ok(self.entry_buff.borrow().clone())
        }
    }

    pub(crate) fn sread_in(&self, count: usize, buffer: &mut CompletionFormat, cond: usize, timeout: i32) -> Result<(), crate::error::Error> {
        let p_cond = cond as *const usize as *const std::ffi::c_void;
        let err = read_cq_entry!(libfabric_sys::inlined_fi_cq_sread, self.handle(), count, buffer, p_cond, timeout);

        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            // *self.completions.borrow_mut() += err as usize;
            // println!("Complete: {}/{}", self.completions.borrow(), self.requests.borrow());

            Ok(())
        }
    }

    // pub(crate) fn sreadfrom_with_cond(&self, count: usize, timeout: i32) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> {
        
    //     let mut address = 0;
    //     let p_address = &mut address as *mut RawMappedAddress;   
    //     let p_cond = std::ptr::null_mut();
    //     let (err, ret) = read_cq_entry!(libfabric_sys::inlined_fi_cq_sreadfrom, self.format, self.handle(), count, p_address, p_cond, timeout);
    //     let address = if address == crate::FI_ADDR_NOTAVAIL {
    //         None
    //     }
    //     else {
    //         Some(MappedAddress::from_raw_addr_no_av(address))
    //     };

    //     if err < 0 {
    //         Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
    //     }
    //     else {
    //         Ok((ret, address))
    //     }
    // }

    pub(crate) fn sreadfrom(&self, count: usize, cond: usize, timeout: i32) -> Result<(CompletionFormat, Option<MappedAddress>), crate::error::Error> {
        
        let p_cond = cond as *const usize as *const std::ffi::c_void;
        let mut address = 0;
        let p_address = &mut address as *mut RawMappedAddress;   
        let err = {
            let mut borrowed_entries = self.entry_buff.borrow_mut();
            read_cq_entry!(libfabric_sys::inlined_fi_cq_sreadfrom, self.handle(), count, *borrowed_entries, p_address, p_cond, timeout)
        };
        let address = if address == crate::FI_ADDR_NOTAVAIL {
            None
        }
        else {
            Some(MappedAddress::from_raw_addr_no_av(address))
        };

        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            Ok((self.entry_buff.borrow().clone(), address))
        }
    }

    pub(crate) fn sreadfrom_in(&self, count: usize, buffer: &mut CompletionFormat, cond: usize, timeout: i32) -> Result< Option<MappedAddress>, crate::error::Error> {
        
        let p_cond = cond as *const usize as *const std::ffi::c_void;
        let mut address = 0;
        let p_address = &mut address as *mut RawMappedAddress;   
        let err = read_cq_entry!(libfabric_sys::inlined_fi_cq_sreadfrom, self.handle(), count, buffer, p_address, p_cond, timeout);

        let address = if address == crate::FI_ADDR_NOTAVAIL {
            None
        }
        else {
            Some(MappedAddress::from_raw_addr_no_av(address))
        };

        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            Ok(address)
        }
    }

    pub(crate) fn signal(&self) -> Result<(), crate::error::Error>{
        
        let err = unsafe { libfabric_sys::inlined_fi_cq_signal(self.handle()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    // pub(crate) fn completions(&self) -> usize {
    //     *self.completions.borrow()
    // }

    // pub(crate) fn request(&self) -> usize {
    //     *self.requests.borrow_mut() += 1;
    //     println!("Requests: {}", self.requests.borrow());
    //     *self.requests.borrow()
    // }
}


impl<T: CqConfig> CompletionQueue<T> {
    #[allow(dead_code)]
    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_cq {
        self.inner.handle()
    }

    pub(crate) fn new<T0>(_options: T, domain: &crate::domain::Domain, attr: CompletionQueueAttr, context: Option<&mut T0>, default_buff_size: usize) -> Result<Self, crate::error::Error> {
        Ok(
            Self {
                inner: Rc::new(AsyncCompletionQueueImpl::new(&domain.inner, attr, context, default_buff_size)?),
                phantom: PhantomData,
            }
        )
    }


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
    pub unsafe fn read_in_unchecked(&self, count: usize, buff: &mut CompletionFormat) -> Result<(), crate::error::Error> {
        self.inner.read_in(count, buff)
    }

    /// Similar to [Self::read] with the exception that it allows the CQ to return source address information to the user for any received data
    /// 
    /// If there is no source address to return it will return None as the second parameter
    /// 
    /// Corresponds to `fi_cq_readfrom`
    pub fn readfrom(&self, count: usize) -> Result<(CompletionFormat, Option<MappedAddress>), crate::error::Error> {
        self.inner.readfrom(count)
    }

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

    pub async fn sreadfrom_with_cond_async(&self, count: usize, cond: usize, timeout: i32) -> Result<(CompletionFormat, Option<MappedAddress>), crate::error::Error> {
        self.inner.sreadfrom(count, cond, timeout)
    }

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

impl<'a, T: CqConfig + FdRetrievable> CompletionQueue<T> {
    pub async fn read_async(&self, count: usize) -> Result<CompletionFormat, crate::error::Error> {
        self.inner.read_async(count).await
    }
}

impl crate::BindImpl for CompletionQueueImpl {}
impl crate::BindImpl for AsyncCompletionQueueImpl {}
impl<T: CqConfig + 'static> crate::Bind for CompletionQueue<T> {
    fn inner(&self) -> Rc<dyn crate::BindImpl> {
        self.inner.clone()
    }
}

impl AsFid for AsyncCompletionQueueImpl {
    fn as_fid(&self) -> crate::fid::BorrowedFid<'_> {
        self.fid.as_fid()
    }
}
impl AsFid for CompletionQueueImpl {
    fn as_fid(&self) -> crate::fid::BorrowedFid<'_> {
        self.fid.as_fid()
    }
}

impl<T: CqConfig> AsFid for CompletionQueue<T> {
    fn as_fid(&self) -> crate::fid::BorrowedFid<'_> {
        self.inner.as_fid()
    }
}

impl AsFd for CompletionQueueImpl {
    fn as_fd(&self) -> BorrowedFd<'_> {
        if let WaitObjType::Fd(fd) = self.wait_object().unwrap() {
            fd
        }
        else {
            panic!("Fabric object object type is not Fd")
        }
    }
}

impl<T: CqConfig + WaitRetrievable + FdRetrievable> AsFd for CompletionQueue<T> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

//================== CompletionQueue Builder ==================//


/// Builder for the [`CompletionQueue`] type.
/// 
/// `CompletionQueueBuilder` is used to configure and build a new `CompletionQueue`.
/// It encapsulates an incremental configuration of the address vector, as provided by a `fi_cq_attr`,
/// followed by a call to `fi_cq_open`  
pub struct CompletionQueueBuilder<'a, T, WAIT, WAITFD> {
    cq_attr: CompletionQueueAttr,
    domain: &'a Domain,
    ctx: Option<&'a mut T>,
    options: Options<WAIT, WAITFD>,
    default_buff_size: usize,
}

    
impl<'a> CompletionQueueBuilder<'a, (), cqoptions::WaitNoRetrieve, cqoptions::Off> {
    
    /// Initiates the creation of a new [CompletionQueue] on `domain`.
    /// 
    /// The initial configuration is what would be set if no `fi_cq_attr` or `context` was provided to 
    /// the `fi_cq_open` call. 
    pub fn new(domain: &'a Domain) -> CompletionQueueBuilder<(), cqoptions::WaitNoRetrieve, cqoptions::Off> {
        Self  {
            cq_attr: CompletionQueueAttr::new(),
            domain,
            ctx: None,
            options: Options::new(),
            default_buff_size: 10,
        }
    }
}

impl<'a, T, WAIT, WAITFD> CompletionQueueBuilder<'a, T, WAIT,  WAITFD> {

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

    /// Sets the underlying low-level waiting object to none.
    /// 
    /// Corresponds to setting `fi_cq_attr::wait_obj` to `FI_WAIT_NONE`.
    pub fn wait_none(mut self) -> CompletionQueueBuilder<'a, T, cqoptions::WaitNone, cqoptions::Off> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::None);

        CompletionQueueBuilder {
            options: self.options.no_wait(),
            cq_attr: self.cq_attr,
            domain: self.domain,
            ctx: self.ctx,
            default_buff_size: self.default_buff_size,
        }
    }
    
    /// Sets the underlying low-level waiting object to FD.
    /// 
    /// Corresponds to setting `fi_cq_attr::wait_obj` to `FI_WAIT_FD`.
    pub fn wait_fd(mut self) -> CompletionQueueBuilder<'a, T, cqoptions::WaitRetrieve, cqoptions::On> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::Fd);

        CompletionQueueBuilder {
            options: self.options.wait_fd(),
            cq_attr: self.cq_attr,
            domain: self.domain,
            ctx: self.ctx,
            default_buff_size: self.default_buff_size,
        }
    }


    /// Sets the underlying low-level waiting object to [crate::sync::WaitSet] `set`.
    /// 
    /// Corresponds to setting `fi_cq_attr::wait_obj` to `FI_WAIT_SET` and `fi_cq_attr::wait_set` to `set`.
    pub fn wait_set(mut self, set: &crate::sync::WaitSet) -> CompletionQueueBuilder<'a, T, cqoptions::WaitNoRetrieve, cqoptions::Off> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::Set(set));

        
        CompletionQueueBuilder {
            options: self.options.wait_no_retrieve(),
            cq_attr: self.cq_attr,
            domain: self.domain,
            ctx: self.ctx,
            default_buff_size: self.default_buff_size,
        }
    }

    /// Sets the underlying low-level waiting object to Mutex+Conditional.
    /// 
    /// Corresponds to setting `fi_cq_attr::wait_obj` to `FI_WAIT_MUTEX_COND`.
    pub fn wait_mutex(mut self) -> CompletionQueueBuilder<'a, T, cqoptions::WaitRetrieve, cqoptions::Off> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::MutexCond);

        
        CompletionQueueBuilder {
            options: self.options.wait_retrievable(),
            cq_attr: self.cq_attr,
            domain: self.domain,
            ctx: self.ctx,
            default_buff_size: self.default_buff_size,
        }
    }

    /// Indicates that the counter will wait without a wait object but instead yield on every wait.
    /// 
    /// Corresponds to setting `fi_cq_attr::wait_obj` to `FI_WAIT_YIELD`.
    pub fn wait_yield(mut self) -> CompletionQueueBuilder<'a, T, cqoptions::WaitNoRetrieve, cqoptions::Off> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::Yield);

        CompletionQueueBuilder {
            options: self.options.wait_no_retrieve(),
            cq_attr: self.cq_attr,
            domain: self.domain,
            ctx: self.ctx,
            default_buff_size: self.default_buff_size,
        }
    }
    
    // pub fn format_ctx(mut self) -> CompletionQueueBuilder<'a, T, WAIT,  WAITFD, libfabric_sys::fi_cq_entry> {
    
    //     self.cq_attr.format(CqFormat::CONTEXT);

    //     CompletionQueueBuilder {
    //         options: self.options,
    //         cq_attr: self.cq_attr,
    //         domain: self.domain,
    //         ctx: self.ctx,
    //         default_buff_size: self.default_buff_size,
    //     }
    // }
    
    // pub fn format_msg(mut self) -> CompletionQueueBuilder<'a, T, WAIT,  WAITFD, libfabric_sys::fi_cq_msg_entry> {
    
    //     self.cq_attr.format(CqFormat::MSG);

    //     CompletionQueueBuilder {
    //         options: self.options,
    //         cq_attr: self.cq_attr,
    //         domain: self.domain,
    //         ctx: self.ctx,
    //         phantom: PhantomData,
    //         default_buff_size: self.default_buff_size,
    //     }
    // }
    
    // pub fn format_data(mut self) -> CompletionQueueBuilder<'a, T, WAIT,  WAITFD, libfabric_sys::fi_cq_data_entry> {
    
    //     self.cq_attr.format(CqFormat::DATA);

    //     CompletionQueueBuilder {
    //         options: self.options,
    //         cq_attr: self.cq_attr,
    //         domain: self.domain,
    //         ctx: self.ctx,
    //         phantom: PhantomData,
    //         default_buff_size: self.default_buff_size,
    //     }
    // }
    
    // pub fn format_tagged(mut self) -> CompletionQueueBuilder<'a, T, WAIT,  WAITFD, libfabric_sys::fi_cq_tagged_entry> {
    
    //     self.cq_attr.format(CqFormat::TAGGED);

    //     CompletionQueueBuilder {
    //         options: self.options,
    //         cq_attr: self.cq_attr,
    //         domain: self.domain,
    //         ctx: self.ctx,
    //         phantom: PhantomData,
    //         default_buff_size: self.default_buff_size,
    //     }
    // }

    /// Enables blocking calls like [CompletionQueue::sread_with_cond] to only 'wake up' after
    /// the number of completions in their field `cond` is satisfied.
    /// 
    /// Corresponds to setting `fi_cq_attr::wait_cond` to `FI_CQ_COND_THRESHOLD`.
    pub fn enable_wait_cond(mut self) -> Self {
        self.cq_attr.wait_cond(crate::enums::WaitCond::Threshold);
        self
    }

    /// Sets the context to be passed to the `CompletionQueue`.
    /// 
    /// Corresponds to passing a non-NULL `context` value to `fi_cq_open`.
    pub fn context(self, ctx: &'a mut T) -> CompletionQueueBuilder<'a, T, WAIT, WAITFD> {
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
    pub fn build(self) ->  Result<CompletionQueue<Options<WAIT, WAITFD>>, crate::error::Error> {
        CompletionQueue::new(self.options, self.domain, self.cq_attr, self.ctx, self.default_buff_size)   
    }
}

//================== CompletionQueue Attribute (fi_cq_attr) ==================//

pub(crate) struct CompletionQueueAttr {
    pub(crate) c_attr: libfabric_sys::fi_cq_attr,
}

impl CompletionQueueAttr {

    pub(crate) fn new() -> Self {
        let c_attr = libfabric_sys::fi_cq_attr{
            size: 0, 
            flags: 0, 
            format: crate::enums::CqFormat::UNSPEC.get_value(), 
            wait_obj: crate::enums::WaitObj::Unspec.get_value(),
            signaling_vector: 0,
            wait_cond: crate::enums::WaitCond::None.get_value(),
            wait_set: std::ptr::null_mut()
        };

        Self {c_attr}
    }

    pub(crate) fn size(&mut self, size: usize) -> &mut Self {
        self.c_attr.size = size;
        self
    }

    pub(crate) fn format(&mut self, format: crate::enums::CqFormat) -> &mut Self {
        self.c_attr.format = format.get_value();
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
        self.c_attr.flags |= libfabric_sys::FI_AFFINITY as u64;
        self.c_attr.signaling_vector = signaling_vector;
        self
    }

    pub(crate) fn wait_cond(&mut self, wait_cond: crate::enums::WaitCond) -> &mut Self {
        self.c_attr.wait_cond = wait_cond.get_value();
        self
    }


    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_cq_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_cq_attr {
        &mut self.c_attr
    }
}

impl Default for CompletionQueueAttr {
    fn default() -> Self {
        Self::new()
    }
}

// //================== CompletionQueue Entry (fi_cq_entry) ==================//

// /// A completion format that provides only user specified context that was associated with the completion.
// /// 
// /// Corresponds to `fi_cq_entry`
// #[repr(C)]
// pub struct CqCtxEntry {
//     pub(crate) c_entry: libfabric_sys::fi_cq_entry,
// }

// impl CqCtxEntry {

//     pub(crate) fn new() -> Self {
//         Self {
//             c_entry: libfabric_sys::fi_cq_entry { op_context: std::ptr::null_mut() }
//         }
//     }

//     // pub fn op_context(&self) -> &Context { 

//     //     let p_ctx = self.c_entry.op_context as *const Context;
//     //     unsafe {& *p_ctx}
//     // }

//     /// Checks if the context held by the completion entry is equal to `ctx`
//     /// 
//     /// Corresponds to `fi_cq_entry::op_context == ctx`
//     pub fn is_op_context_equal<T>(&self, ctx: &T) -> bool {
//         std::ptr::eq(self.c_entry.op_context, (ctx as *const T).cast() )
//     }
// }

// impl Default for CqCtxEntry {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// enum EntryFormat {
//     Unspec,
//     Ctx(libfabric_sys::fi_cq_entry),
//     Msg(libfabric_sys::fi_cq_msg_entry),
//     Data(libfabric_sys::fi_cq_data_entry),
//     Tagged(libfabric_sys::fi_cq_tagged_entry),
// }


#[derive(Clone)]
pub struct Completion<Format> {
    pub(crate) c_entry: Format,
}


impl Completion<()> {
    fn new() -> Self {
        Self {
            c_entry: () 
        }
    }
}

impl Completion<libfabric_sys::fi_cq_entry> {
    fn new() -> Self {
        Self {
            c_entry: libfabric_sys::fi_cq_entry { op_context: std::ptr::null_mut() },
        }
    }
}

impl Completion<libfabric_sys::fi_cq_msg_entry> {
    fn new() -> Self {
        Self {
            c_entry: libfabric_sys::fi_cq_msg_entry { op_context: std::ptr::null_mut(), flags: 0, len: 0 },
        }
    }
}

impl Completion<libfabric_sys::fi_cq_data_entry> {
    fn new() -> Self {
        Self {
            c_entry: libfabric_sys::fi_cq_data_entry { op_context: std::ptr::null_mut(), flags: 0, len: 0, buf: std::ptr::null_mut(), data: 0 },
        }
    }
}

impl Completion<libfabric_sys::fi_cq_tagged_entry> {
    fn new() -> Self {
        Self {
            c_entry: libfabric_sys::fi_cq_tagged_entry { op_context: std::ptr::null_mut(), flags: 0, len: 0, buf: std::ptr::null_mut(), data: 0 , tag: 0},
        }
    }
}

impl Completion<libfabric_sys::fi_cq_entry> {
    
    pub fn is_op_context_equal<T>(&self, ctx: &T) -> bool {
        std::ptr::eq(self.c_entry.op_context, (ctx as *const T).cast() )
    }
} 

impl Completion<libfabric_sys::fi_cq_msg_entry> {
    
    pub fn is_op_context_equal<T>(&self, ctx: &T) -> bool {
        std::ptr::eq(self.c_entry.op_context, (ctx as *const T).cast() )
    }

    /// Returns the completion flags related to this completion entry
    /// 
    /// Corresponds to accessing the `fi_cq_msg_entry::flags` field.
    pub fn flags(&self) -> CompletionFlags {
        CompletionFlags::from_value(self.c_entry.flags)
    }
} 

impl Completion<libfabric_sys::fi_cq_data_entry> {
    
    pub fn is_op_context_equal<T>(&self, ctx: &T) -> bool {
        std::ptr::eq(self.c_entry.op_context, (ctx as *const T).cast() )
    }

    /// Returns the completion flags related to this completion entry
    /// 
    /// Corresponds to accessing the `fi_cq_data_entry::flags` field.
    pub fn flags(&self) -> CompletionFlags {
        CompletionFlags::from_value(self.c_entry.flags)
    }

    /// Returns the receive data buffer.
    /// 
    /// # Safety
    /// This is an unsafe method because the user needs to specify the datatype of the received data.
    /// 
    /// Corresponds to accessing the `fi_cq_data_entry::buf` field.
    pub unsafe fn buffer<T>(&self) -> &[T] {

        unsafe {std::slice::from_raw_parts(self.c_entry.buf as *const T, self.c_entry.len/std::mem::size_of::<T>())}
    }

    /// Returns the remote completion data.
    /// 
    /// Corresponds to accessing the `fi_cq_data_entry::data` field.
    pub fn data(&self) -> u64 {
        self.c_entry.data
    }
} 

impl Completion<libfabric_sys::fi_cq_tagged_entry> {
    
    pub fn is_op_context_equal<T>(&self, ctx: &T) -> bool {
        std::ptr::eq(self.c_entry.op_context, (ctx as *const T).cast() )
    }

    pub fn flags(&self) -> CompletionFlags {
        CompletionFlags::from_value(self.c_entry.flags)
    }


    /// Returns the receive data buffer.
    /// 
    /// # Safety
    /// This is an unsafe method because the user needs to specify the datatype of the received data.
    /// 
    /// Corresponds to accessing the `fi_cq_tagged_entry::buf` field.
    pub unsafe fn buffer<T>(&self) -> &[T] {

        unsafe {std::slice::from_raw_parts(self.c_entry.buf as *const T, self.c_entry.len/std::mem::size_of::<T>())}
    }

    /// Returns the remote completion data.
    /// 
    /// Corresponds to accessing the `fi_cq_tagged_entry::data` field.
    pub fn data(&self) -> u64 {
        self.c_entry.data
    }

    /// Returns the tag of the message associated with the completion.
    /// 
    /// Corresponds to accessing the `fi_cq_tagged_entry::tag` field.
    pub fn tag(&self) -> u64 {
        self.c_entry.tag
    }
} 

impl Default for Completion<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Completion<libfabric_sys::fi_cq_entry> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Completion<libfabric_sys::fi_cq_msg_entry> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Completion<libfabric_sys::fi_cq_data_entry> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Completion<libfabric_sys::fi_cq_tagged_entry> {
    fn default() -> Self {
        Self::new()
    }
}

// //================== CompletionQueue Message Entry (fi_cq_msg_entry) ==================//

// /// A completion format that provides minimal data for processing completions, with expanded support for reporting information about received messages.
// /// 
// /// Corresponds to `fi_cq_msg_entry`
// #[repr(C)]
// pub struct CqMsgEntry {
//     pub(crate) c_entry: libfabric_sys::fi_cq_msg_entry,
// }

// impl CqMsgEntry {

//     pub(crate) fn new() -> Self {
//         Self {
//             c_entry: libfabric_sys::fi_cq_msg_entry { op_context: std::ptr::null_mut(), flags: 0, len: 0 }
//         }
//     }

//     pub fn is_op_context_equal<T>(&self, ctx: &T) -> bool {
//         std::ptr::eq(self.c_entry.op_context, (ctx as *const T).cast() )
//     }

//     /// Returns the completion flags related to this completion entry
//     /// 
//     /// Corresponds to accessing the `fi_cq_msg_entry::flags` field.
//     pub fn flags(&self) -> CompletionFlags {
//         CompletionFlags::from_value(self.c_entry.flags)
//     }
// }

// impl Default for CqMsgEntry {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// //================== CompletionQueue Data Entry (fi_cq_data_entry) ==================//

// /// A completion format that provides data associated with a completion. Includes support for received message length, remote CQ data, and multi-receive buffers.
// /// 
// /// Corresponds to `fi_cq_data_entry`
// #[repr(C)]
// pub struct CqDataEntry {
//     pub(crate) c_entry: libfabric_sys::fi_cq_data_entry,
// }

// impl CqDataEntry {

//     pub(crate) fn new() -> Self {
//         Self {
//             c_entry: libfabric_sys::fi_cq_data_entry { op_context: std::ptr::null_mut(), flags: 0, len: 0, buf: std::ptr::null_mut(), data: 0}
//         }
//     }

//     pub fn is_op_context_equal<T>(&self, ctx: &T) -> bool {
//         std::ptr::eq(self.c_entry.op_context, (ctx as *const T).cast() )
//     }

//     /// Returns the completion flags related to this completion entry
//     /// 
//     /// Corresponds to accessing the `fi_cq_data_entry::flags` field.
//     pub fn flags(&self) -> CompletionFlags {
//         CompletionFlags::from_value(self.c_entry.flags)
//     }

//     /// Returns the receive data buffer.
//     /// 
//     /// # Safety
//     /// This is an unsafe method because the user needs to specify the datatype of the received data.
//     /// 
//     /// Corresponds to accessing the `fi_cq_data_entry::buf` field.
//     pub unsafe fn buffer<T>(&self) -> &[T] {

//         unsafe {std::slice::from_raw_parts(self.c_entry.buf as *const T, self.c_entry.len/std::mem::size_of::<T>())}
//     }

//     /// Returns the remote completion data.
//     /// 
//     /// Corresponds to accessing the `fi_cq_data_entry::data` field.
//     pub fn data(&self) -> u64 {
//         self.c_entry.data
//     }
// }

// impl Default for CqDataEntry {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// //================== CompletionQueue Tagged Entry (fi_cq_tagged_entry) ==================//

// /// A completion format that expands completion data to include support for the tagged message interfaces.
// /// 
// /// Corresponds to `fi_cq_tagged_entry`
// #[repr(C)]
// pub struct CqTaggedEntry {
//     pub(crate) c_entry: libfabric_sys::fi_cq_tagged_entry,
// }

// impl CqTaggedEntry {

//     pub(crate) fn new() -> Self {
//         Self {
//             c_entry: libfabric_sys::fi_cq_tagged_entry { op_context: std::ptr::null_mut(), flags: 0, len: 0, buf: std::ptr::null_mut(), data: 0, tag: 0}
//         }
//     }

//     pub fn op_context(&self) -> &Context { 

//         let p_ctx = self.c_entry.op_context as *const Context;
//         unsafe {& *p_ctx}
//     }

//     /// Returns the completion flags related to this completion entry
//     /// 
//     /// Corresponds to accessing the `fi_cq_tagged_entry::flags` field.
//     pub fn flags(&self) -> CompletionFlags {
//         CompletionFlags::from_value(self.c_entry.flags)
//     }

//     /// Returns the receive data buffer.
//     /// 
//     /// # Safety
//     /// This is an unsafe method because the user needs to specify the datatype of the received data.
//     /// 
//     /// Corresponds to accessing the `fi_cq_tagged_entry::buf` field.
//     pub unsafe fn buffer<T>(&self) -> &[T] {

//         unsafe {std::slice::from_raw_parts(self.c_entry.buf as *const T, self.c_entry.len/std::mem::size_of::<T>())}
//     }

//     /// Returns the remote completion data.
//     /// 
//     /// Corresponds to accessing the `fi_cq_tagged_entry::data` field.
//     pub fn data(&self) -> u64 {
//         self.c_entry.data
//     }

//     /// Returns the tag of the message associated with the completion.
//     /// 
//     /// Corresponds to accessing the `fi_cq_tagged_entry::tag` field.
//     pub fn tag(&self) -> u64 {
//         self.c_entry.tag
//     }

// }

// impl Default for CqTaggedEntry {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// /// A `Completion` represents one or more entries read from a [CompletionQueue] in one of the supported formats
// pub enum Completion {
//     Unspec(usize),
//     Context(Vec<CqCtxEntry>),
//     Message(Vec<CqMsgEntry>),
//     Data(Vec<CqDataEntry>),
//     Tagged(Vec<CqTaggedEntry>),
// }


//================== CompletionQueue Error Entry (fi_cq_err_entry) ==================//

/// A `CompletionError` represents a an error associated with a completion entry
#[repr(C)]
#[derive(Clone)]
pub struct CompletionError {
    pub(crate) c_err: libfabric_sys::fi_cq_err_entry,
}

impl CompletionError {

    fn new() -> Self {
        Self {
            c_err: libfabric_sys::fi_cq_err_entry {
                op_context: std::ptr::null_mut(),
                flags: 0,
                len: 0,
                buf: std::ptr::null_mut(),
                data: 0,
                tag: 0,
                olen: 0,
                err: 0,
                prov_errno: 0,
                err_data: std::ptr::null_mut(),
                err_data_size: 0,
            }
        }
    }
    
    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_cq_err_entry {
        &self.c_err
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_cq_err_entry {
        &mut self.c_err
    }       

    /// Returns the provider error code
    /// 
    /// Corresponds to access the `fi_cq_err_entry::prov_errno` field.
    pub fn prov_errno(&self) -> i32 {
        self.c_err.prov_errno
    }

    /// Returns the completion flags related to this completion error entry
    /// 
    /// Corresponds to accessing the `fi_cq_err_entry::flags` field.
    pub fn flags(&self) -> CompletionFlags {
        CompletionFlags::from_value(self.c_err.flags)
    }

    /// Returns the receive data buffer.
    /// 
    /// # Safety
    /// This is an unsafe method because the user needs to specify the datatype of the received data.
    /// 
    /// Corresponds to accessing the `fi_cq_err_entry::buf` field.
    pub unsafe fn buffer<T>(&self) -> &[T] {

        unsafe {std::slice::from_raw_parts(self.c_err.buf as *const T, self.c_err.len/std::mem::size_of::<T>())}
    }

    /// Returns the remote completion error data.
    /// 
    /// Corresponds to accessing the `fi_cq_err_entry::data` field.
    pub fn data(&self) -> u64 {
        self.c_err.data
    }

    /// Returns the tag of the message associated with the completion error.
    /// 
    /// Corresponds to accessing the `fi_cq_err_entry::tag` field.
    pub fn tag(&self) -> u64 {
        self.c_err.tag
    }

    /// Returns the overflow length of the completion error entry.
    /// 
    /// Corresponds to accessing the `fi_cq_err_entry::olen` field.
    pub fn overflow_length(&self) -> usize {
        self.c_err.olen
    }

    pub fn error(&self) -> Error {
        Error::from_err_code(self.c_err.err as u32)
    }

    pub fn is_op_context_equal(&self, ctx: &crate::Context) -> bool {
        std::ptr::eq(self.c_err.op_context, ctx as *const crate::Context as *const std::ffi::c_void)
    }

    pub(crate) fn err_data(&self) -> *const std::ffi::c_void {
        self.c_err.err_data
    }

    pub(crate) fn err_data_size(&self) -> usize {
        self.c_err.err_data_size
    }
}

impl Default for CompletionError {
    fn default() -> Self {
        Self::new()
    }
}


//================== Async Stuff ============================//
pub(crate) struct AsyncCtx {
    pub(crate) user_ctx: Option<*mut std::ffi::c_void>,
}

pub(crate) struct AsyncTransferCq{
    // pub(crate) req: usize,
    pub(crate) cq: Rc<AsyncCompletionQueueImpl>,
    pub(crate) ctx: usize
}



impl async_std::future::Future for AsyncTransferCq {
    type Output=Result<SingleCompletionFormat, Error>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        // let mut buff = vec![1u8];
        // self.poll_read(cx, &mut buff[..])
        // let async_ctx_as_usize= &self.ctx as *const AsyncCtx as usize;
        
        loop {
            if let Some(mut entry) = self.cq.pending_entries.borrow_mut().remove(&self.ctx) {
                match entry {
                    SingleCompletionFormat::Unspec(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                    SingleCompletionFormat::Ctx(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                    SingleCompletionFormat::Msg(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                    SingleCompletionFormat::Data(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                    SingleCompletionFormat::Tagged(ref mut e) => {e.c_entry.op_context = unsafe{ ( *(e.c_entry.op_context as *mut AsyncCtx)).user_ctx.unwrap_or(std::ptr::null_mut())} },
                }
                return std::task::Poll::Ready(Ok(entry));
            }

            let fut = self.cq.read_async(1);
            let mut pinned = Box::pin(fut) ;
            let res = match pinned.as_mut().poll(cx) {
                std::task::Poll::Ready(res) => res,
                std::task::Poll::Pending => return std::task::Poll::Pending,
            }?;
            match res {
                CompletionFormat::Unspec(entries) => for e in entries.iter() {self.cq.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Unspec(e.clone()));},
                CompletionFormat::Ctx(entries) => for e in entries.iter() {self.cq.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Ctx(e.clone()));},
                CompletionFormat::Msg(entries) => for e in entries.iter() {self.cq.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Msg(e.clone()));},
                CompletionFormat::Data(entries) => for e in entries.iter() {self.cq.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Data(e.clone()));},
                CompletionFormat::Tagged(entries) => for e in entries.iter() {self.cq.pending_entries.borrow_mut().insert(e.c_entry.op_context as usize, SingleCompletionFormat::Tagged(e.clone()));},
            }
        }
    }
}

//================== CompletionQueue Tests ==================//

#[cfg(test)]
mod tests {

    use crate::{cq::*, domain::DomainBuilder, info::Info};

    #[test]
    fn cq_open_close_simultaneous() {
        let info = Info::new().request().unwrap();
        let entries = info.get();
        
        let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
        let count = 10;
        let domain = DomainBuilder::new(&fab, &entries[0]).build().unwrap();
        // let mut cqs = Vec::new();
        for _ in 0..count {
            let _cq = CompletionQueueBuilder::new(&domain).wait_fd().build().unwrap();
        }
    }

    #[test]
    fn cq_signal() {
        let info = Info::new().request().unwrap();
        let entries = info.get();
        
        let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
        let domain = DomainBuilder::new(&fab, &entries[0]).build().unwrap();
        let cq = CompletionQueueBuilder::new(&domain)
            .size(1)
            .wait_fd()
            .build()
            .unwrap();

        cq.signal().unwrap();
        let ret = cq.sread(1, 2000);
        if let Err(ref err) = ret {
            if ! (matches!(err.kind, crate::error::ErrorKind::TryAgain) || matches!(err.kind, crate::error::ErrorKind::Canceled)) {
                ret.unwrap();
            }
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
                .wait_fd()
                .build()
                .unwrap();
        }
    }
}

#[cfg(test)]
mod libfabric_lifetime_tests {
    use crate::{cq::*, domain::DomainBuilder, info::Info};

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
                .wait_fd()
                .build()
                .unwrap();
            println!("Count = {}", std::rc::Rc::strong_count(&domain.inner));
            cqs.push(cq);
        }
        drop(domain);
        println!("Count = {} After dropping domain\n", std::rc::Rc::strong_count(&cqs[0].inner._domain_rc));

    }
}