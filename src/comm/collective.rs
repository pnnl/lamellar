use std::marker::PhantomData;

use crate::MyOnceCell;
use crate::MyRc;
use crate::MyRefCell;
use crate::RawMappedAddress;
use crate::av::AddressVectorSet;
use crate::av::AddressVectorSetImpl;
use crate::cq::ReadCq;
use crate::enums;
use crate::enums::CollectiveOptions;
use crate::enums::JoinOptions;
use crate::ep::EndpointBase;
use crate::ep::EndpointImplBase;
use crate::eq::ReadEq;
use crate::error::Error;

use crate::fid;
use crate::fid::AsRawFid;
use crate::fid::AsTypedFid;
use crate::fid::EpRawFid;
use crate::fid::McRawFid;
use crate::fid::OwnedMcFid;
use crate::fid::RawFid;
use crate::fid::{AsFid, AsRawTypedFid};
use crate::infocapsoptions::CollCap;
use crate::mr::DataDescriptor;
use crate::utils::check_error;
use crate::utils::to_fi_datatype;
use super::message::extract_raw_addr_and_ctx;
use super::message::extract_raw_ctx;

pub struct MulticastGroupCollective {
    pub(crate) inner: MyRc<MulticastGroupCollectiveImpl>,
}


pub struct MulticastGroupCollectiveImpl  {
    c_mc: MyOnceCell<OwnedMcFid>,
    addr: MyOnceCell<RawMappedAddress>,
    eps: MyRefCell<Vec<MyRc<dyn CollectiveValidEp>>>,
    avset: MyRc<AddressVectorSetImpl>,
}

pub(crate) trait CollectiveValidEp {}
impl<EP: CollectiveEp> CollectiveValidEp for EP {}

impl MulticastGroupCollectiveImpl {
    pub(crate) fn new(avset: &MyRc<AddressVectorSetImpl>) -> Self  {
        Self {
            c_mc: MyOnceCell::new(),
            addr: MyOnceCell::new(),
            eps: MyRefCell::new(Vec::new()),
            avset: avset.clone(),
        }
    }

    // pub(crate) fn join_impl<T, EP: CollectiveValidEp + AsRawTypedFid<Output = EpRawFid> + 'static>(&self, ep: &MyRc<EP>, addr: &Address, options: JoinOptions, context: Option<&mut T>) -> Result<(), Error> {
    //     let mut c_mc: McRawFid = std::ptr::null_mut();
    //     let err = 
    //         if let Some(ctx) = context {
    //             unsafe { libfabric_sys::inlined_fi_join(ep.as_raw_typed_fid(), addr.as_bytes().as_ptr().cast(), options.get_value(), &mut c_mc, (ctx as *mut T).cast()) }
    //         }
    //         else {
    //             unsafe { libfabric_sys::inlined_fi_join(ep.as_raw_typed_fid(), addr.as_bytes().as_ptr().cast(), options.get_value(), &mut c_mc, std::ptr::null_mut()) }
    //         };

    //     if err != 0 {
    //         Err(Error::from_err_code((-err).try_into().unwrap()))
    //     }
    //     else {
    //         if let Err(old_mc)  = self.c_mc.set(OwnedMcFid::from(c_mc)) {
    //             assert!(old_mc.as_raw_typed_fid() == c_mc);
    //         }
    //         else {
    //             self.addr.set(unsafe { libfabric_sys::inlined_fi_mc_addr(c_mc)}).unwrap()
    //         }
    //         self.eps.write().push(ep.clone());
    //         Ok(())
    //     }
    // }

    pub(crate) fn join_collective_impl<T, EP: CollectiveEp + AsRawTypedFid<Output = EpRawFid> + 'static>(&self, ep: &MyRc<EP>, options: JoinOptions, context: Option<&mut T>) -> Result<(), Error> {
        let mut c_mc: McRawFid = std::ptr::null_mut();
        let addr = self.addr.get();
        let raw_addr = if let Some(addr) = addr {
            *addr
        } 
        else {
            self.avset.get_addr()?
        };
        let err = 
            if let Some(ctx) = context {
                unsafe { libfabric_sys::inlined_fi_join_collective(ep.as_raw_typed_fid(), raw_addr, self.avset.as_raw_typed_fid(), options.get_value(), &mut c_mc, (ctx as *mut T).cast()) }
            }
            else {
                unsafe { libfabric_sys::inlined_fi_join_collective(ep.as_raw_typed_fid(), raw_addr, self.avset.as_raw_typed_fid(), options.get_value(), &mut c_mc, std::ptr::null_mut()) }
            };

        if err != 0 {
            Err(Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            if let Err(old_mc)  = self.c_mc.set(OwnedMcFid::from(c_mc)) {
                assert!(old_mc.as_raw_typed_fid() == c_mc);
            }
            else {
                self.addr.set(unsafe { libfabric_sys::inlined_fi_mc_addr(c_mc)}).unwrap()
            }
            #[cfg(feature="thread-safe")]
            self.eps.write().push(ep.clone());
            #[cfg(not(feature="thread-safe"))]
            self.eps.borrow_mut().push(ep.clone());
            Ok(())
        }
    }
}


impl MulticastGroupCollective {

    #[allow(dead_code)]
    pub(crate) fn from_impl(mc_impl: &MyRc<MulticastGroupCollectiveImpl>) -> Self {
        Self {
            inner: mc_impl.clone(),
        }
    }

    pub fn new(avset: &AddressVectorSet) -> Self {
        Self { inner: MyRc::new(MulticastGroupCollectiveImpl::new(&avset.inner))} 
    }

    pub(crate) fn get_raw_addr(&self) -> RawMappedAddress {
        *self.inner.addr.get().unwrap()
    }

    // pub fn join<E: CollectiveEp + AsRawTypedFid<Output = EpRawFid> + 'static>(&self, ep: &EndpointBase<E>, addr: &Address, options: JoinOptions) -> Result<(), Error> {
    //     self.inner.join_impl::<(), E>(&ep.inner, addr, options, None)
    // }

    // pub fn join_with_context<E: CollectiveEp + AsRawTypedFid<Output = EpRawFid> + 'static,T>(&self, ep: &EndpointBase<E>, addr: &Address, options: JoinOptions, context: &mut T) -> Result<(), Error> {
    //     self.inner.join_impl(&ep.inner, addr, options, Some(context))
    // }

    pub fn join_collective_with_context<E: CollectiveEp + AsRawTypedFid<Output = EpRawFid> + 'static,T>(&self, ep: &EndpointBase<E>, options: JoinOptions, context: &mut T) -> Result<(), Error> {
        self.inner.join_collective_impl(&ep.inner, options, Some(context))
        
    }
    pub fn join_collective<E: CollectiveEp + AsRawTypedFid<Output = EpRawFid> + 'static>(&self, ep: &EndpointBase<E>, options: JoinOptions) -> Result<(), Error> {
        self.inner.join_collective_impl::<(), E>(&ep.inner, options, None)
    }
}

pub(crate) trait CollectiveEpImpl : CollectiveEp + AsRawTypedFid<Output = EpRawFid> {
    fn barrier_impl<T0>(&self,  mc_group: &MulticastGroupCollective, context: Option<*mut T0>, options: Option<CollectiveOptions>) -> Result<(), crate::error::Error> { 
        let ctx = extract_raw_ctx(context);
        
        let err = if let Some(opt) = options {
            unsafe { libfabric_sys::inlined_fi_barrier2(self.as_raw_typed_fid(), mc_group.get_raw_addr() , opt.get_value(), ctx) }
        }
        else {
            unsafe { libfabric_sys::inlined_fi_barrier(self.as_raw_typed_fid(), mc_group.get_raw_addr(), ctx) }
        };

        check_error(err)
    } 

    fn broadcast_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, context : Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (root_raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe { libfabric_sys::inlined_fi_broadcast(self.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), mc_group.get_raw_addr() , root_raw_addr, to_fi_datatype::<T>(), options.get_value(), ctx) };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn alltoall_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let ctx = extract_raw_ctx(context);
        let err = unsafe { libfabric_sys::inlined_fi_alltoall(self.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), mc_group.get_raw_addr(), to_fi_datatype::<T>(), options.get_value(), ctx) };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn allgather_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let ctx = extract_raw_ctx(context);
        let err = unsafe { libfabric_sys::inlined_fi_allgather(self.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), result.as_mut_ptr().cast(), result_desc.get_desc(), mc_group.get_raw_addr(), libfabric_sys::fi_datatype_FI_UINT8, options.get_value(), ctx) };
        check_error(err)
    }
    
    #[allow(clippy::too_many_arguments)]
    fn allreduce_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let ctx = extract_raw_ctx(context);
        let err = unsafe { libfabric_sys::inlined_fi_allreduce(self.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), mc_group.get_raw_addr() , to_fi_datatype::<T>(), op.get_value(), options.get_value(), ctx) };
        check_error(err)
    }
    
    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let ctx = extract_raw_ctx(context);
        let err = unsafe { libfabric_sys::inlined_fi_reduce_scatter(self.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), mc_group.get_raw_addr(), to_fi_datatype::<T>(), op.get_value(), options.get_value(), ctx) };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: Option<&crate::MappedAddress>,op: crate::enums::Op,  options: CollectiveOptions, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (root_raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe { libfabric_sys::inlined_fi_reduce(self.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), mc_group.get_raw_addr(), root_raw_addr, to_fi_datatype::<T>(), op.get_value(), options.get_value(), ctx) };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn scatter_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (root_raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe { libfabric_sys::inlined_fi_scatter(self.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), mc_group.get_raw_addr(), root_raw_addr, to_fi_datatype::<T>(), options.get_value(), ctx) };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    fn gather_impl<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: Option<&crate::MappedAddress>, options: CollectiveOptions, context: Option<*mut T0>) -> Result<(), crate::error::Error> {
        let (root_raw_addr, ctx) = extract_raw_addr_and_ctx(root_mapped_addr, context);
        let err = unsafe { libfabric_sys::inlined_fi_gather(self.as_raw_typed_fid(), buf.as_mut_ptr().cast(), std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), mc_group.get_raw_addr(), root_raw_addr, to_fi_datatype::<T>(), options.get_value(), ctx) };
        check_error(err)
    }
}

pub trait CollectiveEp {
    fn barrier(&self, mc_group: &MulticastGroupCollective) -> Result<(), crate::error::Error> ;
    fn barrier_with_context<T0>(&self, mc_group: &MulticastGroupCollective, context: &mut T0) -> Result<(), crate::error::Error> ;
    fn barrier_with_options(&self, mc_group: &MulticastGroupCollective, options: CollectiveOptions) -> Result<(), crate::error::Error> ;
    fn barrier_with_context_with_options<T0>(&self, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context : &mut T0) -> Result<(), crate::error::Error> ;
    fn broadcast<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor,  mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn broadcast_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor,  mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context : &mut T0) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn alltoall<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn alltoall_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn allreduce<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective,op: crate::enums::Op,  options: CollectiveOptions) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn allreduce_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective,op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> ;
    fn allgather<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn allgather_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective,op: crate::enums::Op,  options: CollectiveOptions) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective,op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn reduce<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress,op: crate::enums::Op,  options: CollectiveOptions) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn reduce_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress,op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn scatter<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn scatter_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn gather<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> Result<(), crate::error::Error> ;
    #[allow(clippy::too_many_arguments)]
    fn gather_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> ;
}

impl<EP: CollectiveEpImpl> CollectiveEp for EP {
    fn barrier(&self, mc_group: &MulticastGroupCollective) -> Result<(), crate::error::Error> {
        self.barrier_impl::<()>(mc_group, None, None)
    }

    fn barrier_with_context<T0>(&self, mc_group: &MulticastGroupCollective, context: &mut T0) -> Result<(), crate::error::Error> {
        self.barrier_impl(mc_group, Some(context), None)
    }

    fn barrier_with_options(&self, mc_group: &MulticastGroupCollective, options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.barrier_impl::<()>(mc_group, None, Some(options))
    }

    fn barrier_with_context_with_options<T0>(&self, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context : &mut T0) -> Result<(), crate::error::Error> {
        self.barrier_impl(mc_group, Some(context), Some(options))
    }

    fn broadcast<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.broadcast_impl::<T, ()>(buf, desc, mc_group, Some(root_mapped_addr), options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn broadcast_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor,  mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context : &mut T0) -> Result<(), crate::error::Error> {
        self.broadcast_impl(buf, desc, mc_group, Some(root_mapped_addr), options, Some(context))
    }

    #[allow(clippy::too_many_arguments)]
    fn alltoall<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.alltoall_impl::<T, ()>(buf, desc, result, result_desc, mc_group, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn alltoall_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> {
        self.alltoall_impl(buf, desc, result, result_desc, mc_group, options, Some(context))
    }

    #[allow(clippy::too_many_arguments)]
    fn allreduce<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.allreduce_impl::<T, ()>(buf, desc, result, result_desc, mc_group, op, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn allreduce_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> {
        self.allreduce_impl(buf, desc, result, result_desc, mc_group, op, options, Some(context))
    }

    fn allgather<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.allgather_impl::<T, ()>(buf, desc, result, result_desc, mc_group, options, None)
    }
    
    #[allow(clippy::too_many_arguments)]
    fn allgather_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut [T], result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> {
        self.allgather_impl(buf, desc, result, result_desc, mc_group, options, Some(context))
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.reduce_scatter_impl::<T, ()>(buf, desc, result, result_desc, mc_group, op, options, None)
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce_scatter_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> {
        self.reduce_scatter_impl(buf, desc, result, result_desc, mc_group, op, options, Some(context))
    }

    #[allow(clippy::too_many_arguments)]
    fn reduce<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress,op: crate::enums::Op,  options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.reduce_impl::<T, ()>(buf, desc, result, result_desc, mc_group, Some(root_mapped_addr), op, options, None)
    }
    
    #[allow(clippy::too_many_arguments)]
    fn reduce_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress,op: crate::enums::Op,  options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> {
        self.reduce_impl(buf, desc, result, result_desc, mc_group, Some(root_mapped_addr), op, options, Some(context))
    }

    #[allow(clippy::too_many_arguments)]
    fn scatter<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.scatter_impl::<T, ()>(buf, desc, result, result_desc, mc_group, Some(root_mapped_addr), options, None)
    }
    
    #[allow(clippy::too_many_arguments)]
    fn scatter_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> {
        self.scatter_impl(buf, desc, result, result_desc, mc_group, Some(root_mapped_addr), options, Some(context))
    }

    #[allow(clippy::too_many_arguments)]
    fn gather<T: 'static>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions) -> Result<(), crate::error::Error> {
        self.gather_impl::<T, ()>(buf, desc, result, result_desc, mc_group, Some(root_mapped_addr), options, None)
    }
    
    #[allow(clippy::too_many_arguments)]
    fn gather_with_context<T: 'static, T0>(&self, buf: &mut [T], desc: &mut impl DataDescriptor, result: &mut T, result_desc: &mut impl DataDescriptor, mc_group: &MulticastGroupCollective, root_mapped_addr: &crate::MappedAddress, options: CollectiveOptions, context: &mut T0) -> Result<(), crate::error::Error> {
        self.gather_impl(buf, desc, result, result_desc, mc_group, Some(root_mapped_addr), options, Some(context))
    }
}

impl<EP: CollCap, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> CollectiveEpImpl for EndpointImplBase<EP, EQ, CQ>  {}

impl<E: CollectiveEpImpl> CollectiveEpImpl for EndpointBase<E>  {}

impl AsFid for MulticastGroupCollective {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.inner.as_fid()
    }
}

impl AsFid for MulticastGroupCollectiveImpl {
    fn as_fid(&self) -> fid::BorrowedFid<'_> {
        self.c_mc.get().unwrap().as_fid()
    }
}

impl AsRawFid for MulticastGroupCollective {
    fn as_raw_fid(&self) -> RawFid {
        self.inner.as_raw_fid()
    }
}

impl AsRawFid for MulticastGroupCollectiveImpl {
    fn as_raw_fid(&self) -> RawFid {
        self.c_mc.get().unwrap().as_raw_fid()
    }
}
impl AsTypedFid<McRawFid> for MulticastGroupCollective {
    
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<McRawFid> {
        self.inner.as_typed_fid()
    }
}

impl AsTypedFid<McRawFid> for MulticastGroupCollectiveImpl {
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<McRawFid> {
        self.c_mc.get().unwrap().as_typed_fid()
    }
}

impl AsRawTypedFid for MulticastGroupCollective {
    type Output = McRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.inner.as_raw_typed_fid()
    }
}

impl AsRawTypedFid for MulticastGroupCollectiveImpl {
    type Output = McRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.c_mc.get().unwrap().as_raw_typed_fid()
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
