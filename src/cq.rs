use std::{marker::PhantomData, os::fd::{AsFd, BorrowedFd}, rc::Rc};

#[allow(unused_imports)]
use crate::fid::AsFid;
use crate::{cqoptions::{self, CqConfig, Options}, domain::{Domain, DomainImpl}, enums::{CqFormat, WaitObjType, CompletionFlags}, MappedAddress, Context, FdRetrievable, WaitRetrievable, Waitable, fid::{OwnedFid, AsRawFid}, RawMappedAddress};

//================== CompletionQueue (fi_cq) ==================//

macro_rules! read_cq_entry {
    ($read_fn: expr, $format: expr, $cq: expr, $count: expr,  $( $x:ident),*) => {
        match $format {
            CqFormat::CONTEXT => {
                let mut entries: Vec<CqEntry> = Vec::new();
                entries.resize_with($count, Default::default);
                let err = unsafe{ $read_fn($cq, entries.as_mut_ptr().cast(), $count, $($x,)*)};
                (err, Completion::Context(entries))
            }
            CqFormat::DATA => {
                let mut entries: Vec<CqDataEntry>= Vec::new();
                entries.resize_with($count, Default::default);
                let err = unsafe{ $read_fn($cq, entries.as_mut_ptr().cast(), $count, $($x,)*)};
                (err, Completion::Data(entries))
            }
            CqFormat::TAGGED => {
                let mut entries: Vec<CqTaggedEntry> = Vec::new();
                entries.resize_with($count, Default::default);
                let err = unsafe{ $read_fn($cq, entries.as_mut_ptr().cast(), $count, $($x,)*)};

                (err, Completion::Tagged(entries))
            }
            CqFormat::MSG => {
                let mut entries: Vec<CqMsgEntry> = Vec::new();
                entries.resize_with($count, Default::default);
                let err = unsafe{ $read_fn($cq, entries.as_mut_ptr().cast(), $count, $($x,)*)};

                (err, Completion::Message(entries))
            }
            CqFormat::UNSPEC => {
                let res = unsafe{ $read_fn($cq, std::ptr::null_mut(), $count, $($x,)*)};
                if res >= 0 {
                    (res, Completion::Unspec(-res as usize))
                }
                else {
                    (res, Completion::Unspec(0))
                }
            }
        }
    }
}

pub(crate) struct CompletionQueueImpl {
    c_cq: *mut libfabric_sys::fid_cq,
    fid: OwnedFid,
    format: CqFormat,
    wait_obj: Option<libfabric_sys::fi_wait_obj>,
    _domain_rc: Rc<DomainImpl>
}

pub struct CompletionQueue<T: CqConfig> {
    pub(crate) inner: Rc<CompletionQueueImpl>,
    phantom: PhantomData<T>,
}

impl<'a> CompletionQueueImpl {

    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_cq {
        self.c_cq
    }

    pub(crate) fn new<T0>(domain: &Rc<crate::domain::DomainImpl>, mut attr: CompletionQueueAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        
        let mut c_cq: *mut libfabric_sys::fid_cq  = std::ptr::null_mut();
        let c_cq_ptr: *mut *mut libfabric_sys::fid_cq = &mut c_cq;

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
                    format: CqFormat::from_value(attr.c_attr.format), 
                    wait_obj: Some(attr.c_attr.wait_obj),
                    _domain_rc: domain.clone(),
                })
        }
    }

    //[TODO] Maybe avoid making the extra copies for the final vector
    pub(crate) fn read(&self, count: usize) -> Result<Completion, crate::error::Error> {
       
        let (err, ret) = read_cq_entry!(libfabric_sys::inlined_fi_cq_read, self.format, self.handle(),count,);
        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            Ok(ret)
        }
    }

    pub(crate) fn readfrom(&self, count: usize) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> {
       
        let mut address = 0;
        let p_address = &mut address as *mut libfabric_sys::fi_addr_t;    
        let (err, ret) = read_cq_entry!(libfabric_sys::inlined_fi_cq_readfrom, self.format, self.handle(),count, p_address);
        let address = if address == crate::FI_ADDR_NOTAVAIL {
            None
        }
        else {
            Some(MappedAddress::from_raw_addr(address))
        };
        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            Ok((ret, address))
        }
    }

    // // [TODO]  Condition is not taken into account
    pub(crate) fn readerr(&self, flags: u64) -> Result<CompletionError, crate::error::Error> {
        
        let mut entry = CompletionError::new();
        let ret = unsafe { libfabric_sys::inlined_fi_cq_readerr(self.handle(), entry.get_mut(), flags) };

        if ret < 0 {
            Err(crate::error::Error::from_err_code((-ret).try_into().unwrap()))
        }
        else {
            Ok(entry)
        }
    }

    pub(crate) fn print_error(&self, err_entry: &crate::cq::CompletionError) {
        println!("{}", unsafe{self.strerror(err_entry.get_prov_errno(), err_entry.get_err_data(), err_entry.get_err_data_size())} );
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


    pub(crate) fn sread_with_cond<T0>(&self, count: usize, cond: &T0, timeout: i32) -> Result<Completion, crate::error::Error> {
        let p_cond = cond as *const T0 as *const std::ffi::c_void;
        let (err, ret) = read_cq_entry!(libfabric_sys::inlined_fi_cq_sread, self.format, self.handle(), count, p_cond, timeout);
        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            Ok(ret)
        }
    }

    pub(crate) fn sread(&self, count: usize, timeout: i32) -> Result<Completion, crate::error::Error> {
        
        let p_cond = std::ptr::null_mut();
        let (err, ret) = read_cq_entry!(libfabric_sys::inlined_fi_cq_sread, self.format, self.handle(), count, p_cond, timeout);
        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            Ok(ret)
        }
    }

    pub(crate) fn sreadfrom(&self, count: usize, timeout: i32) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> {
        
        let mut address = 0;
        let p_address = &mut address as *mut RawMappedAddress;   
        let p_cond = std::ptr::null_mut();
        let (err, ret) = read_cq_entry!(libfabric_sys::inlined_fi_cq_sreadfrom, self.format, self.handle(), count, p_address, p_cond, timeout);
        let address = if address == crate::FI_ADDR_NOTAVAIL {
            None
        }
        else {
            Some(MappedAddress::from_raw_addr(address))
        };

        if err < 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()) ) 
        }
        else {
            Ok((ret, address))
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
}


impl<T: CqConfig> CompletionQueue<T> {
    #[allow(dead_code)]
    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_cq {
        self.inner.handle()
    }

    pub(crate) fn new<T0>(_options: T, domain: &crate::domain::Domain, attr: CompletionQueueAttr, context: Option<&mut T0>) -> Result<Self, crate::error::Error> {
        Ok(
            Self {
                inner: Rc::new(CompletionQueueImpl::new(&domain.inner, attr, context)?),
                phantom: PhantomData,
            }
        )
    }

    pub fn read(&self, count: usize) -> Result<Completion, crate::error::Error> {
        self.inner.read(count)
    }

    pub fn readfrom(&self, count: usize) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> {
        self.inner.readfrom(count)
    }
    
    pub fn readerr(&self, flags: u64) -> Result<CompletionError, crate::error::Error> {
        self.inner.readerr(flags)
    }

    pub fn print_error(&self, err_entry: &crate::cq::CompletionError) {
        self.inner.print_error(err_entry)
    }

}

impl<T: CqConfig + Waitable> CompletionQueue<T> {
    pub fn sread_with_cond<T0>(&self, count: usize, cond: &T0, timeout: i32) -> Result<Completion, crate::error::Error> {
        self.inner.sread_with_cond(count, cond, timeout)
    }

    pub fn sread(&self, count: usize, timeout: i32) -> Result<Completion, crate::error::Error> {
        self.inner.sread(count, timeout)
    }

    pub fn sreadfrom(&self, count: usize, timeout: i32) -> Result<(Completion, Option<MappedAddress>), crate::error::Error> {
        self.inner.sreadfrom(count, timeout)
    }

    pub fn signal(&self) -> Result<(), crate::error::Error>{
        self.inner.signal()
    }
}

impl<'a, T: CqConfig + WaitRetrievable> CompletionQueue<T> {
    pub fn wait_object(&self) -> Result<WaitObjType<'a>, crate::error::Error> {
        self.inner.wait_object()
    }
}

impl crate::BindImpl for CompletionQueueImpl {}
impl<T: CqConfig + 'static> crate::Bind for CompletionQueue<T> {
    fn inner(&self) -> Rc<dyn crate::BindImpl> {
        self.inner.clone()
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

//================== CompletionQueue Attribute (fi_cq_attr) ==================//

pub struct CompletionQueueBuilder<'a, T, WAIT, WAITFD> {
    cq_attr: CompletionQueueAttr,
    domain: &'a Domain,
    ctx: Option<&'a mut T>,
    options: Options<WAIT, WAITFD>,
}

impl<'a> CompletionQueueBuilder<'a, (), cqoptions::WaitNoRetrieve, cqoptions::Off> {
    pub fn new(domain: &'a Domain) -> CompletionQueueBuilder<(), cqoptions::WaitNoRetrieve, cqoptions::Off> {
        Self  {
            cq_attr: CompletionQueueAttr::new(),
            domain,
            ctx: None,
            options: Options::new()
        }
    }
}

impl<'a, T, WAIT, WAITFD> CompletionQueueBuilder<'a, T, WAIT,  WAITFD> {

    pub fn size(mut self, size: usize) -> Self {
        self.cq_attr.size(size);
        self
    }

    pub fn signaling_vector(mut self, signaling_vector: i32) -> Self {
        self.cq_attr.signaling_vector(signaling_vector);
        self
    }


    pub fn format(mut self, format: crate::enums::CqFormat) -> Self {
        self.cq_attr.format(format);
        self
    }

    pub fn wait_none(mut self) -> CompletionQueueBuilder<'a, T, cqoptions::WaitNone, cqoptions::Off> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::None);

        CompletionQueueBuilder {
            options: self.options.no_wait(),
            cq_attr: self.cq_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }
    
    pub fn wait_fd(mut self) -> CompletionQueueBuilder<'a, T, cqoptions::WaitRetrieve, cqoptions::On> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::Fd);

        CompletionQueueBuilder {
            options: self.options.wait_fd(),
            cq_attr: self.cq_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }

    pub fn wait_set(mut self, set: &crate::sync::WaitSet) -> CompletionQueueBuilder<'a, T, cqoptions::WaitNoRetrieve, cqoptions::Off> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::Set(set));

        
        CompletionQueueBuilder {
            options: self.options.wait_no_retrieve(),
            cq_attr: self.cq_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }

    pub fn wait_mutex(mut self) -> CompletionQueueBuilder<'a, T, cqoptions::WaitRetrieve, cqoptions::Off> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::MutexCond);

        
        CompletionQueueBuilder {
            options: self.options.wait_retrievable(),
            cq_attr: self.cq_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }

    pub fn wait_yield(mut self) -> CompletionQueueBuilder<'a, T, cqoptions::WaitNoRetrieve, cqoptions::Off> {
        self.cq_attr.wait_obj(crate::enums::WaitObj::Yield);

        CompletionQueueBuilder {
            options: self.options.wait_no_retrieve(),
            cq_attr: self.cq_attr,
            domain: self.domain,
            ctx: self.ctx,
        }
    }
    
    pub fn wait_cond(mut self, wait_cond: crate::enums::WaitCond) -> Self {
        self.cq_attr.wait_cond(wait_cond);
        self
    }

    pub fn context(self, ctx: &'a mut T) -> CompletionQueueBuilder<'a, T, WAIT, WAITFD> {
        CompletionQueueBuilder {
            ctx: Some(ctx),
            cq_attr: self.cq_attr,
            domain: self.domain,
            options: self.options,
        }
    }

    pub fn build(self) ->  Result<CompletionQueue<Options<WAIT, WAITFD>>, crate::error::Error> {
        CompletionQueue::new(self.options, self.domain, self.cq_attr, self.ctx)   
    }
}

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

//================== CompletionQueue Entry (fi_cq_entry) ==================//

#[repr(C)]
pub struct CqEntry {
    pub(crate) c_entry: libfabric_sys::fi_cq_entry,
}

impl CqEntry {

    pub fn new() -> Self {
        Self {
            c_entry: libfabric_sys::fi_cq_entry { op_context: std::ptr::null_mut() }
        }
    }

    pub fn op_context(&self) -> &Context { 

        let p_ctx = self.c_entry.op_context as *const Context;
        unsafe {& *p_ctx}
    }

    pub fn is_op_context_equal<T>(&self, ctx: &T) -> bool {
        std::ptr::eq(self.c_entry.op_context, (ctx as *const T).cast() )
    }
}

impl Default for CqEntry {
    fn default() -> Self {
        Self::new()
    }
}

//================== CompletionQueue Message Entry (fi_cq_msg_entry) ==================//

#[repr(C)]
pub struct CqMsgEntry {
    pub(crate) c_entry: libfabric_sys::fi_cq_msg_entry,
}

impl CqMsgEntry {

    pub fn new() -> Self {
        Self {
            c_entry: libfabric_sys::fi_cq_msg_entry { op_context: std::ptr::null_mut(), flags: 0, len: 0 }
        }
    }

    pub fn op_context(&self) -> &Context { 

        let p_ctx = self.c_entry.op_context as *const Context;
        unsafe {& *p_ctx}
    }

    pub fn flags(&self) -> CompletionFlags {
        CompletionFlags::from_value(self.c_entry.flags)
    }

    pub fn size(&self) -> usize {
        self.c_entry.len
    }
}

impl Default for CqMsgEntry {
    fn default() -> Self {
        Self::new()
    }
}

//================== CompletionQueue Data Entry (fi_cq_data_entry) ==================//

#[repr(C)]
pub struct CqDataEntry {
    pub(crate) c_entry: libfabric_sys::fi_cq_data_entry,
}

impl CqDataEntry {

    pub fn new() -> Self {
        Self {
            c_entry: libfabric_sys::fi_cq_data_entry { op_context: std::ptr::null_mut(), flags: 0, len: 0, buf: std::ptr::null_mut(), data: 0}
        }
    }

    pub fn op_context(&self) -> &Context { 

        let p_ctx = self.c_entry.op_context as *const Context;
        unsafe {& *p_ctx}
    }

    pub fn flags(&self) -> CompletionFlags {
        CompletionFlags::from_value(self.c_entry.flags)
    }

    pub fn buffer<T>(&self) -> &[T] {

        unsafe {std::slice::from_raw_parts(self.c_entry.buf as *const T, self.c_entry.len/std::mem::size_of::<T>())}
    }

    pub fn data(&self) -> u64 {
        self.c_entry.data
    }
}

impl Default for CqDataEntry {
    fn default() -> Self {
        Self::new()
    }
}

//================== CompletionQueue Tagged Entry (fi_cq_tagged_entry) ==================//

#[repr(C)]
pub struct CqTaggedEntry {
    pub(crate) c_entry: libfabric_sys::fi_cq_tagged_entry,
}

impl CqTaggedEntry {

    pub fn new() -> Self {
        Self {
            c_entry: libfabric_sys::fi_cq_tagged_entry { op_context: std::ptr::null_mut(), flags: 0, len: 0, buf: std::ptr::null_mut(), data: 0, tag: 0}
        }
    }

    pub fn op_context(&self) -> &Context { 

        let p_ctx = self.c_entry.op_context as *const Context;
        unsafe {& *p_ctx}
    }

    pub fn flags(&self) -> CompletionFlags {
        CompletionFlags::from_value(self.c_entry.flags)
    }

    pub fn buffer<T>(&self) -> &[T] {

        unsafe {std::slice::from_raw_parts(self.c_entry.buf as *const T, self.c_entry.len/std::mem::size_of::<T>())}
    }

    pub fn data(&self) -> u64 {
        self.c_entry.data
    }

    pub fn tag(&self) -> u64 {
        self.c_entry.tag
    }

}

impl Default for CqTaggedEntry {
    fn default() -> Self {
        Self::new()
    }
}

pub enum Completion {
    Unspec(usize),
    Context(Vec<CqEntry>),
    Message(Vec<CqMsgEntry>),
    Data(Vec<CqDataEntry>),
    Tagged(Vec<CqTaggedEntry>),
}


//================== CompletionQueue Error Entry (fi_cq_err_entry) ==================//

#[repr(C)]
pub struct CompletionError {
    pub(crate) c_err: libfabric_sys::fi_cq_err_entry,
}

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

    pub fn get_prov_errno(&self) -> i32 {
        self.c_err.prov_errno
    }

    pub fn flags(&self) -> CompletionFlags {
        CompletionFlags::from_value(self.c_err.flags)
    }

    pub fn is_op_context_equal(&self, ctx: &crate::Context) -> bool {
        std::ptr::eq(self.c_err.op_context, ctx as *const crate::Context as *const std::ffi::c_void)
    }

    pub(crate) fn get_err_data(&self) -> *const std::ffi::c_void {
        self.c_err.err_data
    }

    pub(crate) fn get_err_data_size(&self) -> usize {
        self.c_err.err_data_size
    }
}

impl Default for CompletionError {
    fn default() -> Self {
        Self::new()
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
        let mut cqs = Vec::new();
        for _ in 0..count {
            let cq = CompletionQueueBuilder::new(&domain).build().unwrap();
            cqs.push(cq);
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
            let _cq = CompletionQueueBuilder::new(&domain).size(size).build().unwrap();
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
            let cq = CompletionQueueBuilder::new(&domain).build().unwrap();
            println!("Count = {}", std::rc::Rc::strong_count(&domain.inner));
            cqs.push(cq);
        }
        drop(domain);
        println!("Count = {} After dropping domain\n", std::rc::Rc::strong_count(&cqs[0].inner._domain_rc));

    }
}