use crate::ep;
use crate::ep::ActiveEndpoint;
use crate::ep::Endpoint;
use crate::OwnedFid;
use crate::error::Error;
use crate::Address;
use crate::AsFid;

impl Endpoint {

    pub fn join<T0>(&self, addr: &T0, flags: u64) -> Result<MulticastGroupCollective<Self>, crate::error::Error> where Self: Sized { // [TODO]
        MulticastGroupCollective::new(self, addr, flags)
    }

    pub fn join_with_context<T0,T1>(&self, addr: &T0, flags: u64, context: &mut crate::Context) -> Result<MulticastGroupCollective<Self>, crate::error::Error> where Self: Sized {
        MulticastGroupCollective::new_with_context(self, addr, flags, context)
    }

    pub fn join_collective(&self, coll_addr: crate::Address, set: &crate::av::AddressVectorSet, flags: u64) -> Result<MulticastGroupCollective<Self>, crate::error::Error> where Self: Sized {
        MulticastGroupCollective::new_collective(self, coll_addr, set, flags)
    }

    pub fn join_collective_with_context(&self, coll_addr: crate::Address, set: &crate::av::AddressVectorSet, flags: u64, context : &mut crate::Context) -> Result<MulticastGroupCollective<Self>, crate::error::Error> where Self: Sized {
        MulticastGroupCollective::new_collective_with_context(self, coll_addr, set, flags, context)
    }
}

pub struct MulticastGroupCollective<'a, T: crate::ep::ActiveEndpoint>  {
    c_mc: *mut libfabric_sys::fid_mc,
    fid: OwnedFid,
    ep: &'a T,
}

impl<'a, T0: ep::ActiveEndpoint> MulticastGroupCollective<'a, T0> {
    pub(crate) fn new<T>(ep: &'a T0, addr: &T, flags: u64) -> Result<MulticastGroupCollective<'a, T0>, Error> {
        let mut c_mc: *mut libfabric_sys::fid_mc = std::ptr::null_mut();
        let c_mc_ptr: *mut *mut libfabric_sys::fid_mc = &mut c_mc;
        let err = unsafe { libfabric_sys::inlined_fi_join(ep.handle(), addr as *const T as *const std::ffi::c_void, flags, c_mc_ptr, std::ptr::null_mut()) };

        if err != 0 {
            Err(Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_mc, fid: OwnedFid { fid: unsafe { &mut (*c_mc).fid }  }, ep  }
            )
        }

    }

    pub(crate) fn new_with_context<T>(ep: &'a T0, addr: &T, flags: u64, ctx: &mut crate::Context) -> Result<MulticastGroupCollective<'a, T0>, Error> {
        let mut c_mc: *mut libfabric_sys::fid_mc = std::ptr::null_mut();
        let c_mc_ptr: *mut *mut libfabric_sys::fid_mc = &mut c_mc;
        let err = unsafe { libfabric_sys::inlined_fi_join(ep.handle(), addr as *const T as *const std::ffi::c_void, flags, c_mc_ptr, ctx.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_mc, fid: OwnedFid { fid: unsafe { &mut (*c_mc).fid }  }, ep  }
            )
        }

    }

    pub(crate) fn new_collective(ep: &'a T0, addr: Address, set: &crate::av::AddressVectorSet, flags: u64) -> Result<MulticastGroupCollective<'a, T0>, Error> {
        let mut c_mc: *mut libfabric_sys::fid_mc = std::ptr::null_mut();
        let c_mc_ptr: *mut *mut libfabric_sys::fid_mc = &mut c_mc;
        let err = unsafe { libfabric_sys::inlined_fi_join_collective(ep.handle(), addr, set.c_set, flags, c_mc_ptr, std::ptr::null_mut()) };

        if err != 0 {
            Err(Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_mc, fid: OwnedFid { fid: unsafe { &mut (*c_mc).fid }  }, ep  }
            )
        }
    }

    pub(crate) fn new_collective_with_context(ep: &'a T0, addr: Address, set: &crate::av::AddressVectorSet, flags: u64, ctx: &mut crate::Context) -> Result<MulticastGroupCollective<'a, T0>, Error> {
        let mut c_mc: *mut libfabric_sys::fid_mc = std::ptr::null_mut();
        let c_mc_ptr: *mut *mut libfabric_sys::fid_mc = &mut c_mc;
        let err = unsafe { libfabric_sys::inlined_fi_join_collective(ep.handle(), addr, set.c_set, flags, c_mc_ptr, ctx.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(
                Self { c_mc, fid: OwnedFid { fid: unsafe { &mut (*c_mc).fid }  }, ep  }
            )
        }
    }

    pub fn get_addr(&self) -> Address {
        unsafe { libfabric_sys::inlined_fi_mc_addr(self.c_mc) }
    }

    pub fn barrier(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_barrier(self.ep.handle(), self.get_addr(), std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn barrier_with_context(&self, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_barrier(self.ep.handle(), self.get_addr(), context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn barrier2(&self, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_barrier2(self.ep.handle(), self.get_addr(), flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn barrier2_with_context(&self, flags: u64, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_barrier2(self.ep.handle(), self.get_addr(), flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    pub fn broadcas<T>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, root_addr: crate::Address, datatype: crate::DataType, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_broadcast(self.ep.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), self.get_addr(), root_addr, datatype, flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn broadcast_with_context<T>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, root_addr: crate::Address, datatype: crate::DataType, flags: u64, context : &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_broadcast(self.ep.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), self.get_addr(), root_addr, datatype, flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn alltoall<T>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, result: &mut T, result_desc: &mut impl crate::DataDescriptor, datatype: crate::DataType, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_alltoall(self.ep.handle(), buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr(), datatype, flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn alltoall_with_context<T>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, result: &mut T, result_desc: &mut impl crate::DataDescriptor, datatype: crate::DataType, flags: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_alltoall(self.ep.handle(), buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr(), datatype, flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn allreduce<T>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, result: &mut T, result_desc: &mut impl crate::DataDescriptor, datatype: crate::DataType, op: crate::enums::Op,  flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_allreduce(self.ep.handle(), buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr(), datatype, op.get_value(), flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn allreduce_with_context<T>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, result: &mut T, result_desc: &mut impl crate::DataDescriptor, datatype: crate::DataType, op: crate::enums::Op,  flags: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_allreduce(self.ep.handle(), buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr(), datatype, op.get_value(), flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    pub fn allgather<T>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, result: &mut [T], result_desc: &mut impl crate::DataDescriptor, flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_allgather(self.ep.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result.as_mut_ptr() as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr(), libfabric_sys::fi_datatype_FI_UINT8, flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn allgather_with_context<T>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, result: &mut [T], result_desc: &mut impl crate::DataDescriptor, flags: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_allgather(self.ep.handle(), buf.as_mut_ptr() as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result.as_mut_ptr() as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr(), libfabric_sys::fi_datatype_FI_UINT8, flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn reduce_scatter<T,T2>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, result: &mut T, result_desc: &mut impl crate::DataDescriptor, datatype: crate::DataType, op: crate::enums::Op,  flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_reduce_scatter(self.ep.handle(), buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr(), datatype, op.get_value(), flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn reduce_scatter_with_context<T,T1>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, result: &mut T, result_desc: &mut impl crate::DataDescriptor, datatype: crate::DataType, op: crate::enums::Op,  flags: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_reduce_scatter(self.ep.handle(), buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr(), datatype, op.get_value(), flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn reduce<T>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, result: &mut T, result_desc: &mut impl crate::DataDescriptor, root_addr: crate::Address, datatype: crate::DataType, op: crate::enums::Op,  flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_reduce(self.ep.handle(), buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr(), root_addr, datatype, op.get_value(), flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn reduce_with_context<T>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, result: &mut T, result_desc: &mut impl crate::DataDescriptor, root_addr: crate::Address, datatype: crate::DataType, op: crate::enums::Op,  flags: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_reduce(self.ep.handle(), buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr(), root_addr, datatype, op.get_value(), flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn scatter<T>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, result: &mut T, result_desc: &mut impl crate::DataDescriptor, root_addr: crate::Address, datatype: crate::DataType,  flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_scatter(self.ep.handle(), buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr(), root_addr, datatype, flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn scatter_with_context<T>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, result: &mut T, result_desc: &mut impl crate::DataDescriptor, root_addr: crate::Address, datatype: crate::DataType,  flags: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_scatter(self.ep.handle(), buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr(), root_addr, datatype, flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn gather<T>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, result: &mut T, result_desc: &mut impl crate::DataDescriptor, root_addr: crate::Address, datatype: crate::DataType,  flags: u64) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_gather(self.ep.handle(), buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr(), root_addr, datatype, flags, std::ptr::null_mut()) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
    
    #[allow(clippy::too_many_arguments)]
    pub fn gather_with_context<T>(&self, buf: &mut [T], desc: &mut impl crate::DataDescriptor, result: &mut T, result_desc: &mut impl crate::DataDescriptor, root_addr: crate::Address, datatype: crate::DataType,  flags: u64, context: &mut crate::Context) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_gather(self.ep.handle(), buf as *mut [T] as *mut std::ffi::c_void, std::mem::size_of_val(buf), desc.get_desc(), result as *mut T as *mut std::ffi::c_void, result_desc.get_desc(), self.get_addr(), root_addr, datatype, flags, context.get_mut() as *mut std::ffi::c_void) };

        if err != 0 {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
        else {
            Ok(())
        }
    }
}

impl<'a, T0: ActiveEndpoint> AsFid for MulticastGroupCollective<'a, T0> {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
        self.fid.as_fid()
    }
}
