use std::marker::PhantomData;
use std::rc::Rc;

use crate::enums;
use crate::ep::ActiveEndpoint;
use crate::ep::Address;
use crate::ep::Endpoint;
use crate::ep::EndpointImpl;
use crate::error::Error;
use crate::MappedAddress;
use crate::fid;
use crate::fid::AsFid;
use crate::fid::OwnedFid;
use crate::infocapsoptions::CollCap;
use crate::mr::DataDescriptor;
use crate::utils::check_error;
use crate::utils::to_fi_datatype;

impl<E: CollCap> Endpoint<E> {

    pub fn join(&self, addr: &Address, flags: u64) -> Result<MulticastGroupCollective, crate::error::Error> { // [TODO]
        MulticastGroupCollective::new::<E, ()>(self, addr, flags, None)
    }

    pub fn join_with_context<T>(&self, addr: &Address, flags: u64, context: &mut T) -> Result<MulticastGroupCollective, crate::error::Error> {
        MulticastGroupCollective::new::<E, T>(self, addr, flags, Some(context))
    }

    pub fn join_collective(&self, coll_mapped_addr: &crate::MappedAddress, set: &crate::av::AddressVectorSet, flags: u64) -> Result<MulticastGroupCollective, crate::error::Error> {
        MulticastGroupCollective::new_collective::<E, ()>(self, coll_mapped_addr, set, flags, None)
    }

    pub fn join_collective_with_context<T>(&self, coll_mapped_addr: &crate::MappedAddress, set: &crate::av::AddressVectorSet, flags: u64, context : &mut T) -> Result<MulticastGroupCollective, crate::error::Error> {
        MulticastGroupCollective::new_collective::<E, T>(self, coll_mapped_addr, set, flags, Some(context))
    }
}

pub struct MulticastGroupCollective {
    inner: Rc<MulticastGroupCollectiveImpl>,
}

pub struct MulticastGroupCollectiveImpl  {
    c_mc: *mut libfabric_sys::fid_mc,
    fid: OwnedFid,
    ep: Rc<EndpointImpl>,
}

impl MulticastGroupCollective {

    pub(crate) fn handle(&self) -> *mut libfabric_sys::fid_mc {
        self.inner.c_mc
    }

    pub(crate) fn new<E: CollCap, T>(ep: &Endpoint<E>, addr: &Address, flags: u64, context: Option<&mut T>) -> Result<MulticastGroupCollective, Error> {
        let mut c_mc: *mut libfabric_sys::fid_mc = std::ptr::null_mut();
        let c_mc_ptr: *mut *mut libfabric_sys::fid_mc = &mut c_mc;
        let err = 
            if let Some(ctx) = context {
                unsafe { libfabric_sys::inlined_fi_join(ep.handle(), addr.as_bytes().as_ptr().cast(), flags, c_mc_ptr, (ctx as *mut T).cast()) }
            }
            else {
                unsafe { libfabric_sys::inlined_fi_join(ep.handle(), addr.as_bytes().as_ptr().cast(), flags, c_mc_ptr, std::ptr::null_mut()) }
            };

        if err != 0 {
            Err(Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    inner: Rc::new(MulticastGroupCollectiveImpl {
                        c_mc, 
                        fid: OwnedFid::from(unsafe { &mut (*c_mc).fid }), 
                        ep: ep.inner.clone()
                    }) 
                })
        }

    }

    pub(crate) fn new_collective<E: CollCap, T0>(ep: &Endpoint<E>, mapped_addr: &MappedAddress, set: &crate::av::AddressVectorSet, flags: u64, context: Option<&mut T0>) -> Result<MulticastGroupCollective, Error> {
        let mut c_mc: *mut libfabric_sys::fid_mc = std::ptr::null_mut();
        let c_mc_ptr: *mut *mut libfabric_sys::fid_mc = &mut c_mc;
        let err = 
            if let Some(ctx) = context {
                unsafe { libfabric_sys::inlined_fi_join_collective(ep.handle(), mapped_addr.raw_addr(), set.handle(), flags, c_mc_ptr, (ctx as *mut T0).cast()) }
            }
            else {
                unsafe { libfabric_sys::inlined_fi_join_collective(ep.handle(), mapped_addr.raw_addr(), set.handle(), flags, c_mc_ptr, std::ptr::null_mut()) }
            };

        if err != 0 {
            Err(Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self {
                    inner: Rc::new(MulticastGroupCollectiveImpl {
                        c_mc, 
                        fid: OwnedFid::from(unsafe { &mut (*c_mc).fid }), 
                        ep: ep.inner.clone()
                    }) 
                })
        }
    }

    pub fn get_addr(&self) -> MappedAddress {
        let ret = unsafe { libfabric_sys::inlined_fi_mc_addr(self.handle()) };
        MappedAddress::from_raw_addr(ret)
    }

    pub fn barrier(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_barrier(self.inner.ep.c_ep, self.get_addr().raw_addr() , std::ptr::null_mut()) };

        check_error(err)
    }

    pub fn barrier_with_context<T0>(&self, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_barrier(self.inner.ep.c_ep, self.get_addr().raw_addr() , (context as *mut T0).cast()) };

        check_error(err)
    }

    pub fn barrier2(&self, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_barrier2(self.inner.ep.c_ep, self.get_addr().raw_addr() , flags, std::ptr::null_mut()) };

        check_error(err)
    }

    pub fn barrier2_with_context<T0>(&self, flags: u64, context : &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_barrier2(self.inner.ep.c_ep, self.get_addr().raw_addr() , flags, (context as *mut T0).cast()) };

        check_error(err)
    }

    pub fn broadcas<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_broadcast(self.inner.ep.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), self.get_addr().raw_addr() , root_mapped_addr.raw_addr(), to_fi_datatype::<T>(), flags, std::ptr::null_mut()) };

        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn broadcast_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress,flags: u64, context : &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_broadcast(self.inner.ep.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), self.get_addr().raw_addr() , root_mapped_addr.raw_addr(), to_fi_datatype::<T>(), flags, (context as *mut T0).cast()) };

        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn alltoall<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_alltoall(self.inner.ep.c_ep, buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr().raw_addr() , to_fi_datatype::<T>(), flags, std::ptr::null_mut()) };

        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn alltoall_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,flags: u64, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_alltoall(self.inner.ep.c_ep, buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr().raw_addr() , to_fi_datatype::<T>(), flags, (context as *mut T0).cast()) };

        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn allreduce<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_allreduce(self.inner.ep.c_ep, buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr().raw_addr() , to_fi_datatype::<T>(), op.get_value(), flags, std::ptr::null_mut()) };

        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn allreduce_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  flags: u64, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_allreduce(self.inner.ep.c_ep, buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr().raw_addr() , to_fi_datatype::<T>(), op.get_value(), flags, (context as *mut T0).cast()) };

        check_error(err)
    }
    
    pub fn allgather<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_allgather(self.inner.ep.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result.as_mut_ptr() as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr().raw_addr() , libfabric_sys::fi_datatype_FI_UINT8, flags, std::ptr::null_mut()) };

        check_error(err)
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn allgather_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, flags: u64, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_allgather(self.inner.ep.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result.as_mut_ptr() as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr().raw_addr() , libfabric_sys::fi_datatype_FI_UINT8, flags, (context as *mut T0).cast()) };

        check_error(err)
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn reduce_scatter<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_reduce_scatter(self.inner.ep.c_ep, buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr().raw_addr() , to_fi_datatype::<T>(), op.get_value(), flags, std::ptr::null_mut()) };

        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reduce_scatter_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor,op: crate::enums::Op,  flags: u64, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_reduce_scatter(self.inner.ep.c_ep, buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr().raw_addr() , to_fi_datatype::<T>(), op.get_value(), flags, (context as *mut T0).cast()) };

        check_error(err)
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn reduce<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress,op: crate::enums::Op,  flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_reduce(self.inner.ep.c_ep, buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr().raw_addr() , root_mapped_addr.raw_addr(), to_fi_datatype::<T>(), op.get_value(), flags, std::ptr::null_mut()) };

        check_error(err)
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn reduce_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress,op: crate::enums::Op,  flags: u64, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_reduce(self.inner.ep.c_ep, buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr().raw_addr() , root_mapped_addr.raw_addr(), to_fi_datatype::<T>(), op.get_value(), flags, (context as *mut T0).cast()) };

        check_error(err)
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn scatter<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_scatter(self.inner.ep.c_ep, buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr().raw_addr() , root_mapped_addr.raw_addr(), to_fi_datatype::<T>(), flags, std::ptr::null_mut()) };

        check_error(err)
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn scatter_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, flags: u64, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_scatter(self.inner.ep.c_ep, buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr().raw_addr() , root_mapped_addr.raw_addr(), to_fi_datatype::<T>(), flags, (context as *mut T0).cast()) };

        check_error(err)
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn gather<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_gather(self.inner.ep.c_ep, buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr().raw_addr() , root_mapped_addr.raw_addr(), to_fi_datatype::<T>(), flags, std::ptr::null_mut()) };

        check_error(err)
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn gather_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, root_mapped_addr: &crate::MappedAddress, flags: u64, context: &mut T0) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_gather(self.inner.ep.c_ep, buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr().raw_addr() , root_mapped_addr.raw_addr(), to_fi_datatype::<T>(), flags, (context as *mut T0).cast()) };

        check_error(err)
    }
}

impl AsFid for MulticastGroupCollective {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.inner.fid.as_fid()
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
