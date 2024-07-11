use std::marker::PhantomData;
use std::rc::Rc;

use crate::RawMappedAddress;
use crate::cq::CompletionQueueImplT;
use crate::enums;
use crate::enums::CollectiveOptions;
use crate::enums::JoinOptions;
use crate::ep::Address;
use crate::ep::EndpointBase;
use crate::ep::EndpointImplBase;
use crate::eq::EventQueueImplT;
use crate::error::Error;
use crate::MappedAddress;
use crate::fid;
use crate::fid::AsRawFid;
use crate::fid::AsTypedFid;
use crate::fid::McRawFid;
use crate::fid::OwnedMcFid;
use crate::fid::RawFid;
use crate::fid::{AsFid, AsRawTypedFid};
use crate::infocapsoptions::CollCap;
use crate::mr::DataDescriptor;
use crate::utils::check_error;
use crate::utils::to_fi_datatype;

use super::message::extract_raw_addr_and_ctx;

impl<E, EQ, CQ: ?Sized + CompletionQueueImplT> EndpointBase<E, EQ, CQ> where E: CollCap, EQ: ?Sized + EventQueueImplT {

    #[inline]
    pub(crate) fn join_impl<T>(&self, addr: &Address, options: JoinOptions, context: Option<&mut T>) -> Result<MulticastGroupCollectiveBase<EQ, CQ>, crate::error::Error> {
        let mc = MulticastGroupCollectiveBase::new(self, addr, options.get_value(), context)?;
        // self.inner.eq.get().expect("Endpoint is not bound to an Event Queue").bind_mc(&mc.inner); 
        Ok(mc)
    }

    pub fn join(&self, addr: &Address, options: JoinOptions) -> Result<MulticastGroupCollectiveBase<EQ, CQ>, crate::error::Error> { // [TODO]
        self.join_impl::<()>(addr, options, None)
    }

    pub fn join_with_context<T>(&self, addr: &Address, options: JoinOptions, context: &mut T) -> Result<MulticastGroupCollectiveBase<EQ, CQ>, crate::error::Error> {
        self.join_impl(addr, options, Some(context))        
    }

    #[inline]
    fn join_collective_impl<T>(&self, coll_mapped_addr: &crate::MappedAddress, set: &crate::av::AddressVectorSet, options: JoinOptions, context : Option<&mut T>) -> Result<MulticastGroupCollectiveBase<EQ, CQ>, crate::error::Error> {
        let mc = MulticastGroupCollectiveBase::new_collective::<E, T>(self, coll_mapped_addr, set, options.get_value(), context)?;
        // self.inner.eq.get().expect("Endpoint is not bound to an Event Queue").bind_mc(&mc.inner);
        Ok(mc)
    }

    pub fn join_collective(&self, coll_mapped_addr: &crate::MappedAddress, set: &crate::av::AddressVectorSet, options: JoinOptions) -> Result<MulticastGroupCollectiveBase<EQ, CQ>, crate::error::Error> {
        self.join_collective_impl::<()>(coll_mapped_addr, set, options, None)
    }

    pub fn join_collective_with_context<T>(&self, coll_mapped_addr: &crate::MappedAddress, set: &crate::av::AddressVectorSet, options: JoinOptions, context : &mut T) -> Result<MulticastGroupCollectiveBase<EQ, CQ>, crate::error::Error> {
        self.join_collective_impl(coll_mapped_addr, set, options, Some(context))
    }
}

pub type MulticastGroupCollective<EQ, CQ> = MulticastGroupCollectiveBase<EQ, CQ>;

pub struct MulticastGroupCollectiveBase<EQ: ?Sized + EventQueueImplT, CQ: ?Sized + CompletionQueueImplT> {
    pub(crate) inner: Rc<MulticastGroupCollectiveImplBase<EQ, CQ>>,
}

pub type MulticastGroupCollectiveImpl<EQ, CQ>  = MulticastGroupCollectiveImplBase<EQ, CQ>;

pub struct MulticastGroupCollectiveImplBase<EQ: ?Sized + EventQueueImplT, CQ: ?Sized + CompletionQueueImplT>  {
    c_mc: OwnedMcFid,
    addr: RawMappedAddress,
    pub(crate) ep: Rc<EndpointImplBase<EQ, CQ>>,
}

impl<EQ: ?Sized + EventQueueImplT, CQ: ?Sized + CompletionQueueImplT> MulticastGroupCollectiveBase<EQ, CQ> {

    #[allow(dead_code)]
    pub(crate) fn from_impl(mc_impl: &Rc<MulticastGroupCollectiveImplBase<EQ, CQ>>) -> Self {
        Self {
            inner: mc_impl.clone(),
        }
    }

    pub(crate) fn new<E: CollCap, T>(ep: &EndpointBase<E, EQ, CQ>, addr: &Address, flags: u64, context: Option<&mut T>) -> Result<MulticastGroupCollectiveBase<EQ, CQ>, Error> {
        let mut c_mc: McRawFid = std::ptr::null_mut();
        let err = 
            if let Some(ctx) = context {
                unsafe { libfabric_sys::inlined_fi_join(ep.as_raw_typed_fid(), addr.as_bytes().as_ptr().cast(), flags, &mut c_mc, (ctx as *mut T).cast()) }
            }
            else {
                unsafe { libfabric_sys::inlined_fi_join(ep.as_raw_typed_fid(), addr.as_bytes().as_ptr().cast(), flags, &mut c_mc, std::ptr::null_mut()) }
            };

        if err != 0 {
            Err(Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    inner: Rc::new(MulticastGroupCollectiveImplBase {
                        addr: unsafe { libfabric_sys::inlined_fi_mc_addr(c_mc)},
                        c_mc: OwnedMcFid::from(c_mc), 
                        ep: ep.inner.clone()
                    }) 
                })
        }

    }

    pub(crate) fn new_collective<E: CollCap,T0>(ep: &EndpointBase<E, EQ, CQ>, mapped_addr: &MappedAddress, set: &crate::av::AddressVectorSet, flags: u64, context: Option<&mut T0>) -> Result<MulticastGroupCollectiveBase<EQ, CQ>, Error> {
        let mut c_mc: McRawFid = std::ptr::null_mut();
        let err = 
            if let Some(ctx) = context {
                unsafe { libfabric_sys::inlined_fi_join_collective(ep.as_raw_typed_fid(), mapped_addr.raw_addr(), set.as_raw_typed_fid(), flags, &mut c_mc, (ctx as *mut T0).cast()) }
            }
            else {
                unsafe { libfabric_sys::inlined_fi_join_collective(ep.as_raw_typed_fid(), mapped_addr.raw_addr(), set.as_raw_typed_fid(), flags, &mut c_mc, std::ptr::null_mut()) }
            };

        if err != 0 {
            Err(Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    inner: Rc::new(MulticastGroupCollectiveImplBase {
                        addr: unsafe { libfabric_sys::inlined_fi_mc_addr(c_mc)},
                        c_mc: OwnedMcFid::from(c_mc), 
                        ep: ep.inner.clone()
                    }) 
                })
        }
    }

    pub(crate) fn get_raw_addr(&self) -> RawMappedAddress {
        self.inner.addr
    }

    #[inline]
    pub(crate) fn barrier_impl<T0>(&self, context: Option<*mut T0>, options: Option<CollectiveOptions>) -> Result<(), crate::error::Error> { 
        let ctx = if let Some(ctx) = context {
            ctx.cast()
        }
        else {
            std::ptr::null_mut()
        };
        
        let err = if let Some(opt) = options {
            unsafe { libfabric_sys::inlined_fi_barrier2(self.inner.ep.as_raw_typed_fid(), self.get_raw_addr() , opt.get_value(), ctx) }
        }
        else {
            unsafe { libfabric_sys::inlined_fi_barrier(self.inner.ep.as_raw_typed_fid(), self.get_raw_addr(), ctx) }
        };

        check_error(err)
    } 


    pub fn barrier(&self) -> Result<(), crate::error::Error> {
        self.barrier_impl::<()>(None, None)
    }

    pub fn barrier_with_context<T0>(&self, context: &mut T0) -> Result<(), crate::error::Error> {
        self.barrier_impl(Some(context), None)
    }

    pub fn barrier_with_options(&self, options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.barrier_impl::<()>(None, Some(options))
    }

    pub fn barrier_with_context_with_options<T0>(&self, options: CollectiveOptions, context : &mut T0) -> Result<(), crate::error::Error> {
        self.barrier_impl(Some(context), Some(options))
    }

    #[inline]
    pub(crate) fn broadcast_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe { libfabric_sys::inlined_fi_broadcast(self.inner.ep.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), self.get_raw_addr() , raw_addr, to_fi_datatype::<T>(), options.get_value(), ctx) };
        check_error(err)
    }

    pub fn broadcast<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.broadcast_impl::<T, ()>(buf, desc, Some(root_mapped_addr), options, None)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn broadcast_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context : &mut T0) -> Result<(), crate::error::Error> {
        self.broadcast_impl(buf, desc, Some(root_mapped_addr), options, Some(context))
    }

    #[inline]
    pub(crate) fn alltoall_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, options: CollectiveOptions, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let ctx = if let Some(ctx) = context {
            ctx.cast()
        }
        else {
            std::ptr::null_mut()
        };
        let err = unsafe { libfabric_sys::inlined_fi_alltoall(self.inner.ep.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_raw_addr() , to_fi_datatype::<T>(), options.get_value(), ctx) };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn alltoall<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.alltoall_impl::<T, ()>(buf, desc, result, result_desc, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn alltoall_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> {
        self.alltoall_impl(buf, desc, result, result_desc, options, Some(context))
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn allreduce_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  options: CollectiveOptions, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let ctx = if let Some(ctx) = context {
            ctx.cast()
        }
        else {
            std::ptr::null_mut()
        };

        let err = unsafe { libfabric_sys::inlined_fi_allreduce(self.inner.ep.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_raw_addr() , to_fi_datatype::<T>(), op.get_value(), options.get_value(), ctx) };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn allreduce<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.allreduce_impl::<T, ()>(buf, desc, result, result_desc, op, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn allreduce_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> {
        self.allreduce_impl(buf, desc, result, result_desc, op, options, Some(context))
    }
    
    #[inline]
    pub(crate) fn allgather_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, options: CollectiveOptions, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let ctx = if let Some(ctx) = context {
            ctx.cast()
        }
        else {
            std::ptr::null_mut()
        };

        let err = unsafe { libfabric_sys::inlined_fi_allgather(self.inner.ep.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), result.as_mut_ptr().cast(), result_desc.get_desc(), self.get_raw_addr() , libfabric_sys::fi_datatype_FI_UINT8, options.get_value(), ctx) };
        check_error(err)
    }

    pub fn allgather<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.allgather_impl::<T, ()>(buf, desc, result, result_desc, options, None)
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn allgather_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> {
        self.allgather_impl(buf, desc, result, result_desc, options, Some(context))
    }
    
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn reduce_scatter_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  options: CollectiveOptions, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let ctx = if let Some(ctx) = context {
            ctx.cast()
        }
        else {
            std::ptr::null_mut()
        };

        let err = unsafe { libfabric_sys::inlined_fi_reduce_scatter(self.inner.ep.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_raw_addr() , to_fi_datatype::<T>(), op.get_value(), options.get_value(), ctx) };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reduce_scatter<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.reduce_scatter_impl::<T, ()>(buf, desc, result, result_desc, op, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reduce_scatter_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> {
        self.reduce_scatter_impl(buf, desc, result, result_desc, op, options, Some(context))
    }
    
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn reduce_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: Option<&crate::MappedAddress>,op: crate::enums::Op,  options: CollectiveOptions, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe { libfabric_sys::inlined_fi_reduce(self.inner.ep.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_raw_addr() , raw_addr, to_fi_datatype::<T>(), op.get_value(), options.get_value(), ctx) };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reduce<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress,op: crate::enums::Op,  options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.reduce_impl::<T, ()>(buf, desc, result, result_desc, Some(root_mapped_addr), op, options, None)
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn reduce_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress,op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> {
        self.reduce_impl(buf, desc, result, result_desc, Some(root_mapped_addr), op, options, Some(context))
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn scatter_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe { libfabric_sys::inlined_fi_scatter(self.inner.ep.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_raw_addr() , raw_addr, to_fi_datatype::<T>(), options.get_value(), ctx) };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn scatter<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.scatter_impl::<T, ()>(buf, desc, result, result_desc, Some(root_mapped_addr), options, None)
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn scatter_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> {
        self.scatter_impl(buf, desc, result, result_desc, Some(root_mapped_addr), options, Some(context))
    }
    
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn gather_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe { libfabric_sys::inlined_fi_gather(self.inner.ep.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_raw_addr() , raw_addr, to_fi_datatype::<T>(), options.get_value(), ctx) };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn gather<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.gather_impl::<T, ()>(buf, desc, result, result_desc, Some(root_mapped_addr), options, None)
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn gather_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> {
        self.gather_impl(buf, desc, result, result_desc, Some(root_mapped_addr), options, Some(context))
    }
}

impl<EQ: ?Sized + EventQueueImplT, CQ: ?Sized + CompletionQueueImplT> AsFid for MulticastGroupCollectiveBase<EQ, CQ> {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.inner.as_fid()
    }
}

impl<EQ: ?Sized + EventQueueImplT, CQ: ?Sized + CompletionQueueImplT> AsFid for MulticastGroupCollectiveImplBase<EQ, CQ> {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.c_mc.as_fid()
    }
}

impl<EQ: ?Sized + EventQueueImplT, CQ: ?Sized + CompletionQueueImplT> AsRawFid for MulticastGroupCollectiveBase<EQ, CQ> {
    fn as_raw_fid(&self) -> RawFid {
        self.inner.as_raw_fid()
    }
}

impl<EQ: ?Sized + EventQueueImplT, CQ: ?Sized + CompletionQueueImplT> AsRawFid for MulticastGroupCollectiveImplBase<EQ, CQ> {
    fn as_raw_fid(&self) -> RawFid {
        self.c_mc.as_raw_fid()
    }
}
impl<EQ: ?Sized + EventQueueImplT, CQ: ?Sized + CompletionQueueImplT> AsTypedFid<McRawFid> for MulticastGroupCollectiveBase<EQ, CQ> {
    
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<McRawFid> {
        self.inner.as_typed_fid()
    }
}

impl<EQ: ?Sized + EventQueueImplT, CQ: ?Sized + CompletionQueueImplT> AsTypedFid<McRawFid> for MulticastGroupCollectiveImplBase<EQ, CQ> {
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<McRawFid> {
        self.c_mc.as_typed_fid()
    }
}

impl<EQ: ?Sized + EventQueueImplT, CQ: ?Sized + CompletionQueueImplT> AsRawTypedFid for MulticastGroupCollectiveBase<EQ, CQ> {
    type Output = McRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.inner.as_raw_typed_fid()
    }
}

impl<EQ: ?Sized + EventQueueImplT, CQ: ?Sized + CompletionQueueImplT> AsRawTypedFid for MulticastGroupCollectiveImplBase<EQ, CQ> {
    type Output = McRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.c_mc.as_raw_typed_fid()
    }
}


pub struct CollectiveAttr<T> {
    pub(crate) c_attr: libfabric_sys::fi_collective_attr,
    phantom: PhantomData<T>,
}

impl<T: 'static> CollectiveAttr<T> {

    //[TODO] CHECK INITIAL VALUES
    pub fn new() -> Self {

        Self {
            c_attr: libfabric_sys::fi_collective_attr {
                op: 0,
                datatype: to_fi_datatype::<T>(),
                datatype_attr: libfabric_sys::fi_atomic_attr{count: 0, size: 0},
                max_members: 0,
                mode: 0, // [TODO] What are the valid options?
            },
            phantom: PhantomData,
        }
    }


    pub fn op(mut self, op: &enums::Op) -> Self {
        self.c_attr.op = op.get_value();
        self
    }

    pub fn max_members(mut self, members: usize) -> Self {
        self.c_attr.max_members = members;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_collective_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_collective_attr {
        &mut self.c_attr
    }
}

impl<T: 'static> Default for CollectiveAttr<T> {
    fn default() -> Self {
        Self::new()
    }
}
