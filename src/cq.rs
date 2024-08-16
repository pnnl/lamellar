use std::os::fd::{AsFd, BorrowedFd, RawFd, AsRawFd};

use crate::{domain::{DomainBase, DomainImplT}, fid::AsFid, Context, MyRc, MyRefCell};
use crate::{enums::{WaitObjType, CompletionFlags}, MappedAddress, fid::{AsRawFid, AsRawTypedFid, RawFid, CqRawFid, OwnedCqFid, AsTypedFid}, RawMappedAddress, error::Error};
//================== CompletionQueue (fi_cq) ==================//


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
            }
            err
        }
    }
}

macro_rules! read_cq_entry {
    ($read_fn: expr, $handle: expr, $count: expr, $entries: expr, $( $x:ident),*) => {
        match $entries {
            Completion::Unspec(ref mut cv) => {read_cq_entry_!($read_fn,  $handle, $count, cv, $( $x),*)},  
            Completion::Ctx(ref mut cv) => read_cq_entry_!($read_fn,  $handle, $count, cv, $( $x),*),  
            Completion::Msg(ref mut mv) => read_cq_entry_!($read_fn,  $handle, $count, mv, $( $x),*), 
            Completion::Data(ref mut dv) => read_cq_entry_!($read_fn,  $handle, $count, dv, $( $x),*), 
            Completion::Tagged(ref mut tv) => read_cq_entry_!($read_fn,  $handle, $count, tv, $( $x),*),
        }
    }
}

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
pub enum Completion {
    Unspec(Vec<CompletionEntry<CtxEntry>>), // fi_cq_entry seems to be the bare minimum needed
    Ctx(Vec<CompletionEntry<CtxEntry>>),
    Msg(Vec<CompletionEntry<MsgEntry>>),
    Data(Vec<CompletionEntry<DataEntry>>),
    Tagged(Vec<CompletionEntry<TaggedEntry>>),
}

pub enum SingleCompletion {
    Unspec(CompletionEntry<CtxEntry>),
    Ctx(CompletionEntry<CtxEntry>),
    Msg(CompletionEntry<MsgEntry>),
    Data(CompletionEntry<DataEntry>),
    Tagged(CompletionEntry<TaggedEntry>),
}

pub struct CompletionQueueImpl<const WAIT: bool, const RETRIEVE: bool, const FD: bool> {
    pub(crate) c_cq: OwnedCqFid,
    pub(crate) entry_buff: MyRefCell<Completion>,
    pub(crate) error_buff: MyRefCell<CompletionError>,
    #[allow(dead_code)]
    pub(crate) wait_obj: Option<libfabric_sys::fi_wait_obj>,
    pub(crate) _domain_rc: MyRc<dyn DomainImplT>,
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
pub type CompletionQueue<T> = CompletionQueueBase<T>;

pub struct CompletionQueueBase<CQ: ?Sized> {
    pub(crate) inner: MyRc<CQ>,
}

pub trait WaitCq: AsRawTypedFid<Output = CqRawFid> {

    /// Blocking version of [ReadCq::read_in]
    /// 
    /// This call will block the calling thread until either `count` completions have been read in `buffer`, or a timeout occurs.
    /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    /// 
    /// Corresponds to `fi_cq_sread` with `cond` set to `NULL`.
    fn sread_in(&self, count: usize, buffer: &mut Completion, cond: usize, timeout: i32) -> Result<(), crate::error::Error> {
        let p_cond = cond as *const usize as *const std::ffi::c_void;
        let err = read_cq_entry!(libfabric_sys::inlined_fi_cq_sread, self.as_raw_typed_fid(), count, buffer, p_cond, timeout);

        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            Ok(())
        }
    }

    
    /// Blocking version of [ReadCq::readfrom_in]
    /// 
    /// Operates the same as [`WaitCq::sread_in`] with the exception that the call will also return the source address when it unblocks
    /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    /// 
    /// Corresponds to `fi_cq_sreadfrom` with `cond` set to `NULL`.
    fn sreadfrom_in(&self, count: usize, buffer: &mut Completion, cond: usize, timeout: i32) -> Result<Option<MappedAddress>, crate::error::Error> {
        
        let p_cond = cond as *const usize as *const std::ffi::c_void;
        let mut address = 0;
        let p_address = &mut address;   
        let err = read_cq_entry!(libfabric_sys::inlined_fi_cq_sreadfrom, self.as_raw_typed_fid(), count, buffer, p_address, p_cond, timeout);

        let address = if address == crate::FI_ADDR_NOTAVAIL {
            None
        }
        else {
            Some(MappedAddress::from_raw_addr_no_av(RawMappedAddress::Unspec(address)))
        };

        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            Ok(address)
        }
    }

    /// Similar to  [Self::sread] with the ability to set a condition to unblock
    /// 
    /// This call will block the calling thread until `count` completions have been read, a timeout occurs or condition `cond` is met.
    /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    /// 
    /// Corresponds to `fi_cq_sread`
    fn sread_with_cond(&self, count: usize, cond: usize, timeout: i32) -> Result<Completion, crate::error::Error> ;
    

    /// Blocking version of [ReadCq::read]
    /// 
    /// Similar to [WaitCq::sread_in] but the result is returned instead of stored in an input argument
    fn sread(&self, count: usize, timeout: i32) -> Result<Completion, crate::error::Error> {
        self.sread_with_cond(count, 0, timeout)
    }

    fn sreadfrom_with_cond(&self, count: usize, cond: usize, timeout: i32) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> ;
    
        
    /// Blocking version of [ReadCq::readfrom]
    /// 
    /// Operates the same as [`WaitCq::sread`] with the exception that the call will also return the source address when it unblocks
    /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    /// 
    /// Corresponds to `fi_cq_sreadfrom` with `cond` set to `NULL`.
    fn sreadfrom(&self, count: usize, timeout: i32) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> {
        self.sreadfrom_with_cond(count, 0, timeout)
    }
    
    /// Unblock any thread waiting in [WaitCq::sread], [WaitCq::sreadfrom], [WaitCq::sread_with_cond]
    /// 
    /// This call is not available for completion queues configured with no wait object (i.e. [CompletionQueueBuilder::wait_none()]).
    /// 
    /// Corresponds to `fi_cq_signal`
    fn signal(&self) -> Result<(), crate::error::Error>{
        
        let err = unsafe { libfabric_sys::inlined_fi_cq_signal(self.as_raw_typed_fid()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
}
pub trait ReadCq: AsRawTypedFid<Output = CqRawFid> + AsRawFid{

    /// Reads one or more completions from a completion queue
    /// 
    /// The call will read up to `count` completion entries which will be stored in `buffer`
    /// 
    /// Corresponds to `fi_cq_read` with the `buf` maintained and casted automatically
    fn read_in(&self, count: usize, buffer: &mut Completion) -> Result<usize, crate::error::Error> {
        let err = read_cq_entry!(libfabric_sys::inlined_fi_cq_read, self.as_raw_typed_fid(), count, buffer, );
        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            Ok(err as usize)
        }  
    }

    /// Similar to [ReadCq::read_in] with the exception that it allows the CQ to return source address information to the user for any received data
    /// 
    /// If there is no source address to return it will return [None].
    /// 
    /// Corresponds to `fi_cq_readfrom`
    fn readfrom_in(&self, count: usize, buffer: &mut Completion) -> Result<Option<MappedAddress>, crate::error::Error> {
       
        let mut address = 0;
        let p_address = &mut address as *mut libfabric_sys::fi_addr_t;    
        let err = read_cq_entry!(libfabric_sys::inlined_fi_cq_readfrom, self.as_raw_typed_fid(), count, buffer, p_address);
        let address = if address == crate::FI_ADDR_NOTAVAIL {
            None
        }
        else {
            Some(MappedAddress::from_raw_addr_no_av(RawMappedAddress::Unspec(address)))
        };
        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            Ok(address)
        }
    }

    /// Reads one error completion from the queue in `err_buff`
    /// 
    /// Corresponds to `fi_cq_readerr`
    fn readerr_in(&self, err_buff: &mut CompletionError, flags: u64) -> Result<(), crate::error::Error> {
        
        let ret = {
            unsafe { libfabric_sys::inlined_fi_cq_readerr(self.as_raw_typed_fid(), err_buff.get_mut(), flags) }
        };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    fn print_error(&self, err_entry: &crate::cq::CompletionError) {
        let ret = unsafe { libfabric_sys::inlined_fi_cq_strerror(self.as_raw_typed_fid(), err_entry.prov_errno(), err_entry.err_data(), std::ptr::null_mut() , err_entry.err_data_size()) };
        let err_str = unsafe{ std::ffi::CStr::from_ptr(ret).to_str().unwrap() };
        println!("{}", err_str );
    }

    /// Reads one or more completions from a completion queue
    /// 
    /// The call will read up to `count` completion entries which will be stored in a [Completion]
    /// 
    /// Corresponds to `fi_cq_read` with the `buf` maintained and casted automatically
    fn read(&self, count: usize) -> Result<Completion, crate::error::Error> ;

    /// Similar to [ReadCq::read] with the exception that it allows the CQ to return source address information to the user for any received data
    /// 
    /// If there is no source address to return it will return None as the second parameter
    /// 
    /// Corresponds to `fi_cq_readfrom`
    fn readfrom(&self, count: usize) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> ;

    /// Reads one error completion from the queue
    /// 
    /// Corresponds to `fi_cq_readerr`
    fn readerr(&self, flags: u64) -> Result<CompletionError, crate::error::Error> ;
}

impl<const WAIT: bool , const RETRIEVE: bool, const FD: bool> ReadCq for CompletionQueueImpl<WAIT, RETRIEVE, FD> {
    fn read(&self, count: usize) -> Result<Completion, crate::error::Error> {
        #[cfg(feature="thread-safe")]
        let mut borrowed_entries = self.entry_buff.write();
        #[cfg(not(feature="thread-safe"))]
        let mut borrowed_entries = self.entry_buff.borrow_mut();
        self.read_in(count, &mut borrowed_entries)?;
        Ok(borrowed_entries.clone())
    }

    fn readfrom(&self, count: usize) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> {
        #[cfg(feature="thread-safe")]
        let mut borrowed_entries = self.entry_buff.write();
        #[cfg(not(feature="thread-safe"))]
        let mut borrowed_entries = self.entry_buff.borrow_mut();
        let address = self.readfrom_in(count, &mut borrowed_entries)?;
        Ok((borrowed_entries.clone(), address))
    }

    fn readerr(&self, flags: u64) -> Result<CompletionError, crate::error::Error> {
        #[cfg(feature="thread-safe")]
        let mut entry = self.error_buff.write();
        #[cfg(not(feature="thread-safe"))]
        let mut entry = self.error_buff.borrow_mut();
        self.readerr_in(&mut entry, flags)?;
        Ok(entry.clone())
    }
}

impl<const RETRIEVE: bool, const FD: bool> WaitCq for CompletionQueueImpl<true, RETRIEVE, FD> {
    fn sread_with_cond(&self, count: usize, cond: usize, timeout: i32) -> Result<Completion, crate::error::Error> {
        #[cfg(feature="thread-safe")]
        let mut borrowed_entries = self.entry_buff.write();
        #[cfg(not(feature="thread-safe"))]
        let mut borrowed_entries = self.entry_buff.borrow_mut();
        self.sread_in(count, &mut borrowed_entries, cond, timeout)?;
        Ok(borrowed_entries.clone())
    }

    fn sreadfrom_with_cond(&self, count: usize, cond: usize, timeout: i32) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> {
        
        #[cfg(feature="thread-safe")]
        let mut borrowed_entries = self.entry_buff.write();
        #[cfg(not(feature="thread-safe"))]
        let mut borrowed_entries = self.entry_buff.borrow_mut();
        let address = self.sreadfrom_in(count, &mut borrowed_entries, cond, timeout)?;
        Ok((borrowed_entries.clone(), address))
    }
}

impl<'a, const WAIT: bool, const FD: bool> WaitObjectRetrieve<'a> for CompletionQueueImpl<WAIT, true, FD> {
    fn wait_object(&self) -> Result<WaitObjType<'a>, crate::error::Error> {

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
}

pub trait WaitObjectRetrieve<'a> {

    /// Retreives the low-level wait object associated with the counter.
    /// 
    /// This method is available only if the counter has been configured with a retrievable
    /// underlying wait object.
    /// 
    /// Corresponds to `fi_control` with command `FI_GETWAIT`.
    fn wait_object(&self) -> Result<WaitObjType<'a>, crate::error::Error>;
}

impl<const WAIT: bool , const RETRIEVE: bool, const FD: bool> CompletionQueueImpl<WAIT, RETRIEVE, FD>  {

    pub(crate) fn new(domain: MyRc<dyn DomainImplT>, mut attr: CompletionQueueAttr, context: *mut std::ffi::c_void, default_buff_size: usize) -> Result<Self, crate::error::Error> {
        
        let mut c_cq: CqRawFid  = std::ptr::null_mut();

        let err = unsafe {libfabric_sys::inlined_fi_cq_open(domain.as_raw_typed_fid(), attr.get_mut(), &mut c_cq, context)};

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) )
        }
        else {
            Ok(
                Self {
                    c_cq: OwnedCqFid::from(c_cq), 
                    wait_obj: Some(attr.c_attr.wait_obj),

                    _domain_rc: domain,
                    entry_buff: 
                        if attr.c_attr.format == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_UNSPEC {
                            MyRefCell::new(Completion::Unspec(Vec::with_capacity(default_buff_size)))
                        }
                        else if attr.c_attr.format == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_CONTEXT {
                            MyRefCell::new(Completion::Ctx(Vec::with_capacity(default_buff_size)))
                        }
                        else if attr.c_attr.format == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_MSG {
                            MyRefCell::new(Completion::Msg(Vec::with_capacity(default_buff_size)))
                        }
                        else if attr.c_attr.format == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_TAGGED {
                            MyRefCell::new(Completion::Tagged(Vec::with_capacity(default_buff_size)))
                        }
                        else if attr.c_attr.format == libfabric_sys::fi_cq_format_FI_CQ_FORMAT_DATA {
                            MyRefCell::new(Completion::Data(Vec::with_capacity(default_buff_size)))
                        }
                        else {
                            panic!("Unexpected CompletionQueue type");
                        },
                    
                    error_buff: MyRefCell::new(CompletionError::new()),
                })
        }
    }
}


impl<const WAIT: bool , const RETRIEVE: bool, const FD: bool> CompletionQueue<CompletionQueueImpl<WAIT, RETRIEVE, FD>> {

    // pub(crate) fn new<EQ: AsFid + 'static, T0>(_options: T, domain: &crate::domain::DomainBase<EQ>, attr: CompletionQueueAttr, context: Option<&mut T0>, default_buff_size: usize) -> Result<Self, crate::error::Error> {
    pub(crate) fn new(domain: MyRc<dyn DomainImplT>, attr: CompletionQueueAttr, context: Option<&mut Context>, default_buff_size: usize) -> Result<Self, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(
            Self {
                inner: MyRc::new(CompletionQueueImpl::new(domain, attr, c_void, default_buff_size)?),
            }
        )
    }
}

impl<T: ReadCq>  ReadCq for CompletionQueue<T> {
    fn read(&self, count: usize) -> Result<Completion, crate::error::Error> {
        self.inner.read(count)
    }

    fn readfrom(&self, count: usize) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> {
        self.inner.readfrom(count)
    }

    fn readerr(&self, flags: u64) -> Result<CompletionError, crate::error::Error> {
        self.inner.readerr(flags)
    }
}

impl<T: WaitCq> WaitCq for CompletionQueue<T> {
    fn sread_with_cond(&self, count: usize, cond: usize, timeout: i32) -> Result<Completion, crate::error::Error>  {
        self.inner.sread_with_cond(count, cond, timeout)
    }

    fn sreadfrom_with_cond(&self, count: usize, cond: usize, timeout: i32) -> Result<(Completion, Option<MappedAddress>), crate::error::Error>  {
        self.inner.sreadfrom_with_cond(count, cond, timeout)
    }
}

impl<'a, T: WaitObjectRetrieve<'a>> CompletionQueue<T> { //[TODO] Make this a method of the trait ?

    pub fn wait_object(&self) -> Result<WaitObjType<'a>, crate::error::Error> {
        self.inner.wait_object()
    }
}

impl<T: ReadCq + 'static> crate::Bind for CompletionQueue<T> {
    fn inner(&self) -> MyRc<dyn AsRawFid> {
        self.inner.clone()
    }
}

impl<const WAIT: bool , const RETRIEVE: bool, const FD: bool> AsFid for CompletionQueueImpl<WAIT, RETRIEVE, FD> {
    fn as_fid(&self) -> crate::fid::BorrowedFid<'_> {
        self.c_cq.as_fid()
    }
}

impl<const WAIT: bool , const RETRIEVE: bool, const FD: bool> AsRawFid for CompletionQueueImpl<WAIT, RETRIEVE, FD> {
    fn as_raw_fid(&self) -> RawFid {
        self.c_cq.as_raw_fid()
    }
}


impl<const WAIT: bool , const RETRIEVE: bool, const FD: bool> AsTypedFid<CqRawFid> for CompletionQueueImpl<WAIT, RETRIEVE, FD> {
    fn as_typed_fid(&self) -> crate::fid::BorrowedTypedFid<CqRawFid> {
        self.c_cq.as_typed_fid()
    }
}

impl<const WAIT: bool , const RETRIEVE: bool, const FD: bool> AsRawTypedFid for CompletionQueueImpl<WAIT, RETRIEVE, FD> {
    type Output = CqRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.c_cq.as_raw_typed_fid()
    }
}


impl<T: AsFid> AsFid for CompletionQueue<T> {
    fn as_fid(&self) -> crate::fid::BorrowedFid<'_> {
        self.inner.as_fid()
    }
}


impl<T: AsRawFid> AsRawFid for CompletionQueue<T> {
    fn as_raw_fid(&self) -> RawFid {
        self.inner.as_raw_fid()
    }
}

impl<T: AsTypedFid<CqRawFid>> AsTypedFid<CqRawFid> for CompletionQueue<T> {
    fn as_typed_fid(&self) -> crate::fid::BorrowedTypedFid<CqRawFid> {
        self.inner.as_typed_fid()
    }
}

impl<T: AsRawTypedFid<Output = CqRawFid>> AsRawTypedFid for CompletionQueue<T> {
    type Output = CqRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.inner.as_raw_typed_fid()
    }
}

impl AsFd for CompletionQueueImpl<true, true, true> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        if let WaitObjType::Fd(fd) = self.wait_object().unwrap() {
            fd
        }
        else {
            panic!("Fabric object object type is not Fd")
        }
    }
}

impl AsRawFd for CompletionQueueImpl<true, true, true> {
    fn as_raw_fd(&self) -> RawFd {
        if let WaitObjType::Fd(fd) = self.wait_object().unwrap() {
            fd.as_raw_fd()
        }
        else {
            panic!("Fabric object object type is not Fd")
        }
    }
}

impl<T: AsFd> AsFd for CompletionQueue<T> {
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
pub struct CompletionQueueBuilder<'a, const WAIT: bool, const RETRIEVE: bool, const FD: bool> {
    cq_attr: CompletionQueueAttr,
    ctx: Option<&'a mut Context>,
    // options: Options<WAIT, WAITFD>,
    default_buff_size: usize,
}

    
impl<'a> CompletionQueueBuilder<'a, true, false, false> {
    
    /// Initiates the creation of a new [CompletionQueue] on `domain`.
    /// 
    /// The initial configuration is what would be set if no `fi_cq_attr` or `context` was provided to 
    /// the `fi_cq_open` call. 
    pub fn new() -> CompletionQueueBuilder<'a, true, false, false> {
        Self  {
            cq_attr: CompletionQueueAttr::new(),
            ctx: None,
            default_buff_size: 10,
        }
    }
}

impl<'a> Default for CompletionQueueBuilder<'a, true, false, false> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, const WAIT: bool, const RETRIEVE: bool, const FD: bool> CompletionQueueBuilder<'a, WAIT, RETRIEVE, FD> {

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
    pub fn wait_none(mut self) -> CompletionQueueBuilder<'a, false, false, false> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::None);

        CompletionQueueBuilder {
            cq_attr: self.cq_attr,
            ctx: self.ctx,
            default_buff_size: self.default_buff_size,
        }
    }
    
    /// Sets the underlying low-level waiting object to FD.
    /// 
    /// Corresponds to setting `fi_cq_attr::wait_obj` to `FI_WAIT_FD`.
    pub fn wait_fd(mut self) -> CompletionQueueBuilder<'a, true,  true, true> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::Fd);

        CompletionQueueBuilder {
            cq_attr: self.cq_attr,
            ctx: self.ctx,
            default_buff_size: self.default_buff_size,
        }
    }


    /// Sets the underlying low-level waiting object to [crate::sync::WaitSet] `set`.
    /// 
    /// Corresponds to setting `fi_cq_attr::wait_obj` to `FI_WAIT_SET` and `fi_cq_attr::wait_set` to `set`.
    pub fn wait_set(mut self, set: &crate::sync::WaitSet) -> CompletionQueueBuilder<'a, true, false, false> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::Set(set));

        
        CompletionQueueBuilder {
            cq_attr: self.cq_attr,
            ctx: self.ctx,
            default_buff_size: self.default_buff_size,
        }
    }

    /// Sets the underlying low-level waiting object to Mutex+Conditional.
    /// 
    /// Corresponds to setting `fi_cq_attr::wait_obj` to `FI_WAIT_MUTEX_COND`.
    pub fn wait_mutex(mut self) -> CompletionQueueBuilder<'a, true, true, false> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::MutexCond);

        
        CompletionQueueBuilder {
            cq_attr: self.cq_attr,
            ctx: self.ctx,
            default_buff_size: self.default_buff_size,
        }
    }

    /// Indicates that the counter will wait without a wait object but instead yield on every wait.
    /// 
    /// Corresponds to setting `fi_cq_attr::wait_obj` to `FI_WAIT_YIELD`.
    pub fn wait_yield(mut self) -> CompletionQueueBuilder<'a, true, false, false> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::Yield);

        CompletionQueueBuilder {
            cq_attr: self.cq_attr,
            ctx: self.ctx,
            default_buff_size: self.default_buff_size,
        }
    }
    
    // pub fn format_ctx(mut self) -> CompletionQueueBuilder<'a, T, WAIT,  WAITFD, libfabric_sys::fi_cq_entry> {
    
    //     self.cq_attr.format(CqFormat::CONTEXT);

    //     CompletionQueueBuilder {
    //         options: self.options,
    //         cq_attr: self.cq_attr,
    //         ctx: self.ctx,
    //         default_buff_size: self.default_buff_size,
    //     }
    // }
    
    // pub fn format_msg(mut self) -> CompletionQueueBuilder<'a, T, WAIT,  WAITFD, libfabric_sys::fi_cq_msg_entry> {
    
    //     self.cq_attr.format(CqFormat::MSG);

    //     CompletionQueueBuilder {
    //         options: self.options,
    //         cq_attr: self.cq_attr,
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
    pub fn context(self, ctx: &'a mut Context) -> CompletionQueueBuilder<'a, WAIT, RETRIEVE, FD> {
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
    pub fn build<EQ: ?Sized + 'static>(self, domain: &DomainBase<EQ>) ->  Result<CompletionQueue<CompletionQueueImpl<WAIT, RETRIEVE, FD>>, crate::error::Error> {
        // CompletionQueue::new(self.options, self.domain, self.cq_attr, self.ctx, self.default_buff_size)   
        CompletionQueue::<CompletionQueueImpl<WAIT, RETRIEVE, FD>>::new(domain.inner.clone(), self.cq_attr, self.ctx, self.default_buff_size)   
    }
}

//================== CompletionQueue Attribute (fi_cq_attr) ==================//

#[derive(Clone)]
pub(crate) struct CompletionQueueAttr {
    pub(crate) c_attr: libfabric_sys::fi_cq_attr,
}

impl CompletionQueueAttr {

    pub(crate) fn new() -> Self {
        let c_attr = libfabric_sys::fi_cq_attr{
            size: 0, 
            flags: 0, 
            format: crate::enums::CqFormat::Unspec.as_raw(), 
            wait_obj: crate::enums::WaitObj::Unspec.as_raw(),
            signaling_vector: 0,
            wait_cond: crate::enums::WaitCond::None.as_raw(),
            wait_set: std::ptr::null_mut()
        };

        Self {c_attr}
    }

    pub(crate) fn size(&mut self, size: usize) -> &mut Self {
        self.c_attr.size = size;
        self
    }

    pub(crate) fn format(&mut self, format: crate::enums::CqFormat) -> &mut Self {
        self.c_attr.format = format.as_raw();
        self
    }
    
    pub(crate) fn wait_obj(&mut self, wait_obj: crate::enums::WaitObj) -> &mut Self {
        if let crate::enums::WaitObj::Set(wait_set) = wait_obj {
            self.c_attr.wait_set = wait_set.as_raw_typed_fid();
        }
        self.c_attr.wait_obj = wait_obj.as_raw();
        self
    }
    
    pub(crate) fn signaling_vector(&mut self, signaling_vector: i32) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_AFFINITY as u64;
        self.c_attr.signaling_vector = signaling_vector;
        self
    }

    pub(crate) fn wait_cond(&mut self, wait_cond: crate::enums::WaitCond) -> &mut Self {
        self.c_attr.wait_cond = wait_cond.as_raw();
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
#[derive(Clone, Debug)]
pub struct CompletionEntry<Format> {
    pub(crate) c_entry: Format,
}

impl CompletionEntry<()> {
    fn new() -> Self {
        Self {
            c_entry: () 
        }
    }
}

impl CompletionEntry<CtxEntry> {
    fn new() -> Self {
        Self {
            c_entry: CtxEntry { op_context: std::ptr::null_mut() },
        }
    }
}

impl CompletionEntry<MsgEntry> {
    fn new() -> Self {
        Self {
            c_entry: MsgEntry { op_context: std::ptr::null_mut(), flags: 0, len: 0 },
        }
    }
}

impl CompletionEntry<DataEntry> {
    fn new() -> Self {
        Self {
            c_entry: DataEntry { op_context: std::ptr::null_mut(), flags: 0, len: 0, buf: std::ptr::null_mut(), data: 0 },
        }
    }
}

impl CompletionEntry<TaggedEntry> {
    fn new() -> Self {
        Self {
            c_entry: TaggedEntry { op_context: std::ptr::null_mut(), flags: 0, len: 0, buf: std::ptr::null_mut(), data: 0 , tag: 0},
        }
    }
}

impl CompletionEntry<CtxEntry> {
    
    pub fn is_op_context_equal<T>(&self, ctx: &T) -> bool {
        std::ptr::eq(self.c_entry.op_context, (ctx as *const T).cast() )
    }
} 

impl CompletionEntry<MsgEntry> {
    
    pub fn is_op_context_equal<T>(&self, ctx: &T) -> bool {
        std::ptr::eq(self.c_entry.op_context, (ctx as *const T).cast() )
    }

    /// Returns the completion flags related to this completion entry
    /// 
    /// Corresponds to accessing the `fi_cq_msg_entry::flags` field.
    pub fn flags(&self) -> CompletionFlags {
        CompletionFlags::from_raw(self.c_entry.flags)
    }
} 

impl CompletionEntry<DataEntry> {
    
    pub fn is_op_context_equal<T>(&self, ctx: &T) -> bool {
        std::ptr::eq(self.c_entry.op_context, (ctx as *const T).cast() )
    }

    /// Returns the completion flags related to this completion entry
    /// 
    /// Corresponds to accessing the `fi_cq_data_entry::flags` field.
    pub fn flags(&self) -> CompletionFlags {
        CompletionFlags::from_raw(self.c_entry.flags)
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

impl CompletionEntry<TaggedEntry> {
    
    pub fn is_op_context_equal<T>(&self, ctx: &T) -> bool {
        std::ptr::eq(self.c_entry.op_context, (ctx as *const T).cast() )
    }

    pub fn flags(&self) -> CompletionFlags {
        CompletionFlags::from_raw(self.c_entry.flags)
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

impl Default for CompletionEntry<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CompletionEntry<CtxEntry> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CompletionEntry<MsgEntry> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CompletionEntry<DataEntry> {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CompletionEntry<TaggedEntry> {
    fn default() -> Self {
        Self::new()
    }
}
//================== CompletionQueue Error Entry (fi_cq_err_entry) ==================//
/// A `CompletionError` represents a an error associated with a completion entry
#[repr(C)]
#[derive(Clone)]
pub struct CompletionError {
    pub(crate) c_err: libfabric_sys::fi_cq_err_entry,
}

//[TODO] Verify if true!


impl CompletionError {

    pub fn new() -> Self {
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
        CompletionFlags::from_raw(self.c_err.flags)
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

    /// Returns the generic error related to this CompletionError
    /// 
    /// Corresponds to accessing the `fi_cq_err_entry::err` field.
    pub fn error(&self) -> Error {
        Error::from_err_code(self.c_err.err as u32)
    }

    pub fn is_op_context_equal(&self, ctx: &crate::Context) -> bool {
        std::ptr::eq(self.c_err.op_context, ctx.inner())
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

//================== CompletionQueue Tests ==================//

#[cfg(test)]
mod tests {

    use crate::{cq::*, domain::DomainBuilder, info::{Info, Version}};

    #[test]
    fn cq_open_close_simultaneous() {
        let info = Info::new(&Version{major: 1, minor: 19})
            .get()
            .unwrap();
        
        let entry = info.into_iter().next().unwrap();
        
        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let count = 10;
        let domain = DomainBuilder::new(&fab, &entry).build().unwrap();
        // let mut cqs = Vec::new();
        for _ in 0..count {
            let _cq = CompletionQueueBuilder::new().wait_fd().build(&domain).unwrap();
        }
    }

    #[test]
    fn cq_signal() {
        let info = Info::new(&Version{major: 1, minor: 19})
            .get()
            .unwrap();
        let entry = info.into_iter().next().unwrap();
        
        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let domain = DomainBuilder::new(&fab, &entry).build().unwrap();
        let cq = CompletionQueueBuilder::new()
            .size(1)
            .wait_fd()
            .build(&domain)
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
        let info = Info::new(&Version{major: 1, minor: 19})
            .get()
            .unwrap();
        let entry = info.into_iter().next().unwrap();
        
        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let domain = DomainBuilder::new(&fab, &entry).build().unwrap();
        for i in -1..17 {
            let size = if i == -1 { 0 } else { 1 << i };
            let _cq = CompletionQueueBuilder::new().size(size)
                .wait_fd()
                .build(&domain)
                .unwrap();
        }
    }
}

#[cfg(test)]
mod libfabric_lifetime_tests {
    use crate::{cq::*, domain::DomainBuilder, info::{Info, Version}};

    #[test]
    fn cq_drops_before_domain() {
        let info = Info::new(&Version{major: 1, minor: 19})
            .get()
            .unwrap();
        let entry = info.into_iter().next().unwrap();
        
        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let count = 10;
        let domain = DomainBuilder::new(&fab, &entry).build().unwrap();
        let mut cqs = Vec::new();
        for _ in 0..count {
            let cq = CompletionQueueBuilder::new()
                .wait_fd()
                .build(&domain)
                .unwrap();
            cqs.push(cq);
        }
        drop(domain);

    }
}