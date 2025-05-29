use crate::RemoteMemoryAddress;
use crate::RemoteMemAddrSliceMut;
use crate::RemoteMemAddrSlice;
use crate::conn_ep::ConnectedEp;
use crate::connless_ep::ConnlessEp;
use crate::cq::ReadCq;
use crate::enums::ReadMsgOptions;
use crate::enums::WriteMsgOptions;
use crate::ep::Connected;
use crate::ep::Connectionless;
use crate::ep::EndpointBase;
use crate::ep::EndpointImplBase;
use crate::ep::EpState;
use crate::eq::ReadEq;
use crate::fid::AsRawTypedFid;
use crate::fid::AsTypedFid;
use crate::fid::EpRawFid;
use crate::infocapsoptions::ReadMod;
use crate::infocapsoptions::RmaCap;
use crate::infocapsoptions::WriteMod;
use crate::mr::MappedMemoryRegionKey;
use crate::mr::MemoryRegionDesc;
use crate::trigger::TriggeredContext;
use crate::utils::check_error;
use crate::utils::Either;
use crate::xcontext::TxContextBase;
use crate::xcontext::TxContextImplBase;
use crate::Context;
use crate::FI_ADDR_UNSPEC;

use super::message::extract_raw_addr_and_ctx;

pub(crate) trait ReadEpImpl: AsTypedFid<EpRawFid> {
    unsafe fn read_impl<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        src_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(src_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_read(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_mut_ptr().cast(),
                std::mem::size_of_val(buf),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                raw_addr,
                mem_addr.into(),
                mapped_key.key(),
                ctx,
            )
        };
        check_error(err)
    }

    unsafe fn readv_impl(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(src_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_readv(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                iov.as_ptr().cast(),
                desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                iov.len(),
                raw_addr,
                mem_addr.into(),
                mapped_key.key(),
                ctx,
            )
        };
        check_error(err)
    }

    unsafe fn readmsg_impl(
        &self,
        msg: Either<&crate::msg::MsgRmaMut, &crate::msg::MsgRmaConnectedMut>,
        options: ReadMsgOptions,
    ) -> Result<(), crate::error::Error> {
        let c_rma_msg = match msg {
            Either::Left(msg) => msg.inner(),
            Either::Right(msg) => msg.inner(),
        };

        let err = unsafe {
            libfabric_sys::inlined_fi_readmsg(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                c_rma_msg,
                options.as_raw(),
            )
        };
        check_error(err)
    }
}

pub trait ReadEp {
    /// Read data from a remote memory region into local buffer `buf`
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_read` without a provided context
    unsafe fn read_from<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        src_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [ReadEp::read_from] but with a context argument provided
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_read`
    unsafe fn read_from_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        src_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    unsafe fn read_from_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        src_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    /// Similar to [ReadEp::read_from] with a list of buffers `iov` instead of a single buffer to store the data ranges read
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readv` with no context`
    unsafe fn readv_from(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [ReadEp::readv_from] but providing a context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readv`
    unsafe fn readv_from_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    unsafe fn readv_from_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;
    unsafe fn readmsg_from(
        &self,
        msg: &crate::msg::MsgRmaMut,
        options: ReadMsgOptions,
    ) -> Result<(), crate::error::Error>;
}

pub trait ReadRemoteMemAddrSliceEp: ReadEp {
    /// Read data from a remote memory region into local buffer `buf`
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_read` without a provided context
    unsafe fn read_slice_from<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        src_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSlice,
    ) -> Result<(), crate::error::Error> {
        assert!(std::mem::size_of_val(buf) == rma_iov.mem_len(), "Source and destination slice sizes do not match");
        ReadEp::read_from(self, buf, desc, src_addr, rma_iov.mem_address(), &rma_iov.key())
    }

    /// Similar to [ReadEp::read_from] but with a context argument provided
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_read`
    unsafe fn read_slice_from_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        src_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSlice,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        assert!(std::mem::size_of_val(buf) == rma_iov.mem_len(), "Source and destination slice sizes do not match");
        ReadEp::read_from_with_context(self, buf, desc, src_addr, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    unsafe fn read_slice_from_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        src_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSlice,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        assert!(std::mem::size_of_val(buf) == rma_iov.mem_len(), "Source and destination slice sizes do not match");
        ReadEp::read_from_triggered(self, buf, desc, src_addr, rma_iov.mem_address(), &rma_iov.key(), context)
    }


    /// Similar to [ReadEp::read_from] with a list of buffers `iov` instead of a single buffer to store the data ranges read
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readv` with no context`
    unsafe fn readv_slice_from(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSlice,
    ) -> Result<(), crate::error::Error> {
        ReadEp::readv_from(self, iov, desc, src_addr, rma_iov.mem_address(), &rma_iov.key())
    }

    /// Similar to [ReadEp::readv_from] but providing a context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readv`
    unsafe fn readv_slice_from_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSlice,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        ReadEp::readv_from_with_context(self, iov, desc, src_addr, rma_iov.mem_address(), &rma_iov.key(), context)
    }
    
    unsafe fn readv_slice_from_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSlice,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        ReadEp::readv_from_triggered(self, iov, desc, src_addr, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    unsafe fn readmsg_slice_from(
        &self,
        msg: &crate::msg::MsgRmaMut,
        options: ReadMsgOptions,
    ) -> Result<(), crate::error::Error> {
        ReadEp::readmsg_from(self, msg, options)
    }
}

pub trait ConnectedReadEp {
    /// Similar to [ReadEp::read_from] but without specifying a src network address
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_read` with no context and `src_addr` = `FI_ADDR_UNSPEC`
    unsafe fn read<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [ReadEp::read] but providing a context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_read` with `src_addr` = `FI_ADDR_UNSPEC`
    unsafe fn read_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    unsafe fn read_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [ReadEp::readv_from] but without specifying a network address
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readv` with no context and `src_addr` set to `FI_ADDR_UNSPEC`
    unsafe fn readv(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [ReadEp::readv] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readv` with `src_addr` set to `FI_ADDR_UNSPEC`
    unsafe fn readv_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;
    unsafe fn readv_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;

    /// Read from remote node with the specifications provided by the `msg` argument
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readmsg`
    unsafe fn readmsg(
        &self,
        msg: &crate::msg::MsgRmaConnectedMut,
        options: ReadMsgOptions,
    ) -> Result<(), crate::error::Error>;
}

pub trait ConnectedReadRemoteMemAddrSliceEp: ConnectedReadEp {
    /// Similar to [ReadEp::read_from] but without specifying a src network address
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_read` with no context and `src_addr` = `FI_ADDR_UNSPEC`
    unsafe fn read_slice<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        rma_iov: &RemoteMemAddrSlice,
    ) -> Result<(), crate::error::Error> {
        ConnectedReadEp::read(self, buf, desc, rma_iov.mem_address(), &rma_iov.key())
    }

    /// Similar to [ReadEp::read] but providing a context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_read` with `src_addr` = `FI_ADDR_UNSPEC`
    unsafe fn read_slice_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        rma_iov: &RemoteMemAddrSlice,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        ConnectedReadEp::read_with_context(self, buf, desc, rma_iov.mem_address(), &rma_iov.key(), context)
    }


    unsafe fn read_slice_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        rma_iov: &RemoteMemAddrSlice,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        ConnectedReadEp::read_triggered(self, buf, desc, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    /// Similar to [ReadEp::readv_from] but without specifying a network address
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readv` with no context and `src_addr` set to `FI_ADDR_UNSPEC`
    unsafe fn readv_slice(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        rma_iov: &RemoteMemAddrSlice,
    ) -> Result<(), crate::error::Error> {
        ConnectedReadEp::readv(self, iov, desc, rma_iov.mem_address(), &rma_iov.key())
    }

    /// Similar to [ReadEp::readv] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readv` with `src_addr` set to `FI_ADDR_UNSPEC`
    unsafe fn readv_slice_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        rma_iov: &RemoteMemAddrSlice,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        ConnectedReadEp::readv_with_context(self, iov, desc, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    unsafe fn readv_slice_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        rma_iov: &RemoteMemAddrSlice,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        ConnectedReadEp::readv_triggered(self, iov, desc, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    /// Read from remote node with the specifications provided by the `msg` argument
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's reading from cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_readmsg`
    unsafe fn readmsg_slice(
        &self,
        msg: &crate::msg::MsgRmaConnectedMut,
        options: ReadMsgOptions,
    ) -> Result<(), crate::error::Error> {
        ConnectedReadEp::readmsg(self, msg, options)
    }
}


impl<EP: ReadEpImpl + ConnlessEp> ReadEp for EP {
    unsafe fn read_from<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        src_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.read_impl(buf, desc, Some(src_addr), mem_addr, mapped_key, None)
    }
    unsafe fn read_from_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        src_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.read_impl(
            buf,
            desc,
            Some(src_addr),
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }
    unsafe fn read_from_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        src_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.read_impl(
            buf,
            desc,
            Some(src_addr),
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }
    unsafe fn readv_from(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.readv_impl(iov, desc, Some(src_addr), mem_addr, mapped_key, None)
    }
    unsafe fn readv_from_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.readv_impl(
            iov,
            desc,
            Some(src_addr),
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }
    unsafe fn readv_from_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        src_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.readv_impl(
            iov,
            desc,
            Some(src_addr),
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }
    unsafe fn readmsg_from(
        &self,
        msg: &crate::msg::MsgRmaMut,
        options: ReadMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.readmsg_impl(Either::Left(msg), options)
    }
}

impl<EP: ReadEp> ReadRemoteMemAddrSliceEp for EP {}

impl<EP: ReadEpImpl + ConnectedEp> ConnectedReadEp for EP {
    unsafe fn read<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.read_impl(buf, desc, None, mem_addr, mapped_key, None)
    }

    unsafe fn read_with_context<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.read_impl(
            buf,
            desc,
            None,
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }

    unsafe fn read_triggered<T>(
        &self,
        buf: &mut [T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.read_impl(
            buf,
            desc,
            None,
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }

    unsafe fn readv(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.readv_impl(iov, desc, None, mem_addr, mapped_key, None)
    }

    unsafe fn readv_with_context(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.readv_impl(
            iov,
            desc,
            None,
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }

    unsafe fn readv_triggered(
        &self,
        iov: &[crate::iovec::IoVecMut],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.readv_impl(
            iov,
            desc,
            None,
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }

    unsafe fn readmsg(
        &self,
        msg: &crate::msg::MsgRmaConnectedMut,
        options: ReadMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.readmsg_impl(Either::Right(msg), options)
    }
}

impl<EP: ConnectedReadEp> ConnectedReadRemoteMemAddrSliceEp for EP {}

impl<EP: RmaCap + ReadMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> ReadEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: RmaCap + ReadMod, STATE: EpState, CQ: ?Sized + ReadCq> ReadEpImpl
    for TxContextImplBase<I, STATE, CQ>
{
}

impl<I: RmaCap + ReadMod, STATE: EpState, CQ: ?Sized + ReadCq> ReadEpImpl
    for TxContextBase<I, STATE, CQ>
{
}

impl<E: ReadEpImpl> ReadEpImpl for EndpointBase<E, Connected> {}
impl<E: ReadEpImpl> ReadEpImpl for EndpointBase<E, Connectionless> {}

pub(crate) trait WriteEpImpl: AsTypedFid<EpRawFid> {
    unsafe fn write_impl<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_write(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                std::mem::size_of_val(buf),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                raw_addr,
                mem_addr.into(),
                mapped_key.key(),
                ctx,
            )
        };
        check_error(err)
    }

    unsafe fn inject_write_impl<T>(
        &self,
        buf: &[T],
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        let raw_addr = if let Some(addr) = dest_addr {
            addr.raw_addr()
        } else {
            FI_ADDR_UNSPEC
        };

        let err = unsafe {
            libfabric_sys::inlined_fi_inject_write(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                std::mem::size_of_val(buf),
                raw_addr,
                mem_addr.into(),
                mapped_key.key(),
            )
        };
        check_error(err)
    }

    unsafe fn writev_impl(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_writev(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                iov.as_ptr().cast(),
                desc.map_or(std::ptr::null_mut(), |d| std::mem::transmute(d.as_ptr())),
                iov.len(),
                raw_addr,
                mem_addr.into(),
                mapped_key.key(),
                ctx,
            )
        };
        check_error(err)
    }

    #[allow(clippy::too_many_arguments)]
    unsafe fn writedata_impl<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: Option<*mut std::ffi::c_void>,
    ) -> Result<(), crate::error::Error> {
        let (raw_addr, ctx) = extract_raw_addr_and_ctx(dest_addr, context);
        let err = unsafe {
            libfabric_sys::inlined_fi_writedata(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                std::mem::size_of_val(buf),
                desc.map_or(std::ptr::null_mut(), |d| d.as_raw()),
                data,
                raw_addr,
                mem_addr.into(),
                mapped_key.key(),
                ctx,
            )
        };
        check_error(err)
    }

    unsafe fn inject_writedata_impl<T>(
        &self,
        buf: &[T],
        data: u64,
        dest_addr: Option<&crate::MappedAddress>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        let raw_addr = if let Some(addr) = dest_addr {
            addr.raw_addr()
        } else {
            FI_ADDR_UNSPEC
        };
        let err = unsafe {
            libfabric_sys::inlined_fi_inject_writedata(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                std::mem::size_of_val(buf),
                data,
                raw_addr,
                mem_addr.into(),
                mapped_key.key(),
            )
        };
        check_error(err)
    }

    unsafe fn writemsg_impl(
        &self,
        msg: Either<&crate::msg::MsgRma, &crate::msg::MsgRmaConnected>,
        options: WriteMsgOptions,
    ) -> Result<(), crate::error::Error> {
        let c_msg_rma = match msg {
            Either::Left(msg) => msg.inner(),
            Either::Right(msg) => msg.inner(),
        };
        let err = unsafe {
            libfabric_sys::inlined_fi_writemsg(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                c_msg_rma,
                options.as_raw(),
            )
        };
        check_error(err)
    }
}

pub trait WriteRemoteMemAddrSliceEp: WriteEp {
    /// Write data to a remote memory region from local buffer `buf` to remote address mem_addr
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write` without a provided context
    unsafe fn write_slice_to<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSliceMut,
    ) -> Result<(), crate::error::Error> {
        self.write_to(buf, desc, dest_addr, rma_iov.mem_address(), &rma_iov.key())
    }

    /// Similar to [WriteEp::write_to] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write`.
    unsafe fn write_slice_to_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSliceMut,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.write_to_with_context(buf, desc, dest_addr, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    /// Similar to [WriteEp::write_to] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write`.
    unsafe fn write_slice_to_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSliceMut,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.write_to_triggered(buf, desc, dest_addr, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_inject`.
    unsafe fn inject_write_slice_to<T>(
        &self,
        buf: &[T],
        dest_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSliceMut,
    ) -> Result<(), crate::error::Error> {
        self.inject_write_to(buf, dest_addr, rma_iov.mem_address(), &rma_iov.key())
    }

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_inject_writedata`.
    unsafe fn inject_writedata_slice_to<T>(
        &self,
        buf: &[T],
        data: u64,
        dest_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSliceMut,
    ) -> Result<(), crate::error::Error> {
        self.inject_writedata_to(buf, data, dest_addr, rma_iov.mem_address(), &rma_iov.key())
    }

    /// Similar to [WriteEp::write_to] but with a list of buffers `iov` instead of a single buffer to transfer
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev` without a provided context
    unsafe fn writev_slice_to(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSliceMut,
    ) -> Result<(), crate::error::Error> {
        self.writev_to(iov, desc, dest_addr, rma_iov.mem_address(), &rma_iov.key())
    }

    /// Similar to [WriteEp::writev_to] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev`.
    unsafe fn writev_slice_to_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSliceMut,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.writev_to_with_context(iov, desc, dest_addr, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    /// Similar to [WriteEp::writev_to] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev`.
    unsafe fn writev_slice_to_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSliceMut,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.writev_to_triggered(iov, desc, dest_addr, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` without providing a context.
    unsafe fn writedata_slice_to<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSliceMut,
    ) -> Result<(), crate::error::Error> {
        self.writedata_to(buf, desc, data, dest_addr, rma_iov.mem_address(), &rma_iov.key())
    }

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` .
    #[allow(clippy::too_many_arguments)]
    unsafe fn writedata_slice_to_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSliceMut,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.writedata_to_with_context(buf, desc, data, dest_addr, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` .
    #[allow(clippy::too_many_arguments)]
    unsafe fn writedata_slice_to_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: &crate::MappedAddress,
        rma_iov: &RemoteMemAddrSliceMut,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.writedata_to_triggered(buf, desc, data, dest_addr, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    unsafe fn writemsg_slice_to(
        &self,
        msg: &crate::msg::MsgRma,
        options: WriteMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.writemsg_to(msg, options)
    }
}

pub trait WriteEp {
    /// Write data to a remote memory region from local buffer `buf` to remote address mem_addr
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write` without a provided context
    unsafe fn write_to<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [WriteEp::write_to] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write`.
    unsafe fn write_to_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [WriteEp::write_to] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write`.
    unsafe fn write_to_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_inject`.
    unsafe fn inject_write_to<T>(
        &self,
        buf: &[T],
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error>;

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_inject_writedata`.
    unsafe fn inject_writedata_to<T>(
        &self,
        buf: &[T],
        data: u64,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [WriteEp::write_to] but with a list of buffers `iov` instead of a single buffer to transfer
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev` without a provided context
    unsafe fn writev_to(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [WriteEp::writev_to] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev`.
    unsafe fn writev_to_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [WriteEp::writev_to] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev`.
    unsafe fn writev_to_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` without providing a context.
    unsafe fn writedata_to<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error>;

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` .
    #[allow(clippy::too_many_arguments)]
    unsafe fn writedata_to_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` .
    #[allow(clippy::too_many_arguments)]
    unsafe fn writedata_to_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;

    unsafe fn writemsg_to(
        &self,
        msg: &crate::msg::MsgRma,
        options: WriteMsgOptions,
    ) -> Result<(), crate::error::Error>;
}

pub trait ConnectedWriteEp {
    /// Similar to [WriteEp::write_to] but without specifying a destination network address (e.g., for connected endpoints)
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write` with dest_addr = FI_ADDR_UNSPEC and no context.
    unsafe fn write<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [WriteEp::write] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn write_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [WriteEp::write] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn write_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_inject` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn inject_write<T>(
        &self,
        buf: &[T],
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error>;

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_inject_writedata` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn inject_writedata<T>(
        &self,
        buf: &[T],
        data: u64,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [WriteEp::writev_to] but without specifying a network address
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev` with `dest_addr` = `FI_ADDR_UNSPEC` and no contex.
    unsafe fn writev(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [WriteEp::writev] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn writev_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;

    /// Similar to [WriteEp::writev] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn writev_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` with no context and `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn writedata<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error>;

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn writedata_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error>;

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn writedata_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error>;

    /// Transfer data base on the specifications provided by the `msg`
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writemsg`
    unsafe fn writemsg(
        &self,
        msg: &crate::msg::MsgRmaConnected,
        options: WriteMsgOptions,
    ) -> Result<(), crate::error::Error>;
}

pub trait ConnectedWriteRemoteMemAddrSliceEp: ConnectedWriteEp {
    /// Similar to [WriteEp::write_to] but without specifying a destination network address (e.g., for connected endpoints)
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write` with dest_addr = FI_ADDR_UNSPEC and no context.
    unsafe fn write_slice<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        rma_iov: &RemoteMemAddrSliceMut,
    ) -> Result<(), crate::error::Error> {
        self.write(buf, desc, rma_iov.mem_address(), &rma_iov.key())
    }

    /// Similar to [WriteEp::write] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn write_slice_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        rma_iov: &RemoteMemAddrSliceMut,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.write_with_context(buf, desc, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    /// Similar to [WriteEp::write] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_write` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn write_slice_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        rma_iov: &RemoteMemAddrSliceMut,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.write_triggered(buf, desc, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_inject` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn inject_write_slice<T>(
        &self,
        buf: &[T],
        rma_iov: &RemoteMemAddrSliceMut,
    ) -> Result<(), crate::error::Error> {
        self.inject_write(buf, rma_iov.mem_address(), &rma_iov.key())
    }

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_inject_writedata` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn inject_writedata_slice<T>(
        &self,
        buf: &[T],
        data: u64,
        rma_iov: &RemoteMemAddrSliceMut,
    ) -> Result<(), crate::error::Error> {
        self.inject_writedata(buf, data, rma_iov.mem_address(), &rma_iov.key())
    }

    /// Similar to [WriteEp::writev_to] but without specifying a network address
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev` with `dest_addr` = `FI_ADDR_UNSPEC` and no contex.
    unsafe fn writev_slice(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        rma_iov: &RemoteMemAddrSliceMut,
    ) -> Result<(), crate::error::Error> {
        self.writev(iov, desc, rma_iov.mem_address(), &rma_iov.key())
    }

    /// Similar to [WriteEp::writev] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn writev_slice_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        rma_iov: &RemoteMemAddrSliceMut,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.writev_with_context(iov, desc, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    /// Similar to [WriteEp::writev] but with a provided context
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writev` with `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn writev_slice_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        rma_iov: &RemoteMemAddrSliceMut,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.writev_triggered(iov, desc, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` with no context and `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn writedata_slice<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        rma_iov: &RemoteMemAddrSliceMut,
    ) -> Result<(), crate::error::Error> {
        self.writedata(buf, desc, data, rma_iov.mem_address(), &rma_iov.key())
    }

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn writedata_slice_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        rma_iov: &RemoteMemAddrSliceMut,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.writedata_with_context(buf, desc, data, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writedata` `dest_addr` = `FI_ADDR_UNSPEC`.
    unsafe fn writedata_slice_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        rma_iov: &RemoteMemAddrSliceMut,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.writedata_triggered(buf, desc, data, rma_iov.mem_address(), &rma_iov.key(), context)
    }

    /// Transfer data base on the specifications provided by the `msg`
    ///
    /// The operation is only expected to have completed when a respective Completion has been generated
    ///
    /// # Safety
    /// This function is unsafe because the remote memory address that it's writing to cannot be guaranteed
    /// to be valid
    ///  
    /// Equivalent to `fi_writemsg`
    unsafe fn writemsg_slice(
        &self,
        msg: &crate::msg::MsgRmaConnected,
        options: WriteMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.writemsg(msg, options)
    }
}

impl<EP: WriteEpImpl + ConnlessEp> WriteEp for EP {
    #[inline]
    unsafe fn write_to<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.write_impl(buf, desc, Some(dest_addr), mem_addr, mapped_key, None)
    }
    #[inline]
    unsafe fn write_to_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.write_impl(
            buf,
            desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }
    #[inline]
    unsafe fn write_to_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.write_impl(
            buf,
            desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    unsafe fn inject_write_to<T>(
        &self,
        buf: &[T],
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.inject_write_impl(buf, Some(dest_addr), mem_addr, mapped_key)
    }

    #[inline]
    unsafe fn writev_to(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.writev_impl(iov, desc, Some(dest_addr), mem_addr, mapped_key, None)
    }
    #[inline]
    unsafe fn writev_to_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.writev_impl(
            iov,
            desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }
    #[inline]
    unsafe fn writev_to_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.writev_impl(
            iov,
            desc,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    unsafe fn writedata_to<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.writedata_impl(buf, desc, data, Some(dest_addr), mem_addr, mapped_key, None)
    }

    #[inline]
    unsafe fn writedata_to_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.writedata_impl(
            buf,
            desc,
            data,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    unsafe fn writedata_to_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.writedata_impl(
            buf,
            desc,
            data,
            Some(dest_addr),
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    unsafe fn inject_writedata_to<T>(
        &self,
        buf: &[T],
        data: u64,
        dest_addr: &crate::MappedAddress,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.inject_writedata_impl(buf, data, Some(dest_addr), mem_addr, mapped_key)
    }

    #[inline]
    unsafe fn writemsg_to(
        &self,
        msg: &crate::msg::MsgRma,
        options: WriteMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.writemsg_impl(Either::Left(msg), options)
    }
}

impl<EP: WriteEp> WriteRemoteMemAddrSliceEp for EP {}

impl<EP: WriteEpImpl + ConnectedEp> ConnectedWriteEp for EP {
    #[inline]
    unsafe fn write<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.write_impl(buf, desc, None, mem_addr, mapped_key, None)
    }

    #[inline]
    unsafe fn write_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.write_impl(
            buf,
            desc,
            None,
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    unsafe fn write_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.write_impl(
            buf,
            desc,
            None,
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    unsafe fn inject_write<T>(
        &self,
        buf: &[T],
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.inject_write_impl(buf, None, mem_addr, mapped_key)
    }

    #[inline]
    unsafe fn writev(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.writev_impl(iov, desc, None, mem_addr, mapped_key, None)
    }

    #[inline]
    unsafe fn writev_with_context(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.writev_impl(
            iov,
            desc,
            None,
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    unsafe fn writev_triggered(
        &self,
        iov: &[crate::iovec::IoVec],
        desc: Option<&[MemoryRegionDesc<'_>]>,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.writev_impl(
            iov,
            desc,
            None,
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    unsafe fn writedata<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.writedata_impl(buf, desc, data, None, mem_addr, mapped_key, None)
    }

    #[inline]
    unsafe fn writedata_with_context<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut Context,
    ) -> Result<(), crate::error::Error> {
        self.writedata_impl(
            buf,
            desc,
            data,
            None,
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    unsafe fn writedata_triggered<T>(
        &self,
        buf: &[T],
        desc: Option<&MemoryRegionDesc<'_>>,
        data: u64,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
        context: &mut TriggeredContext,
    ) -> Result<(), crate::error::Error> {
        self.writedata_impl(
            buf,
            desc,
            data,
            None,
            mem_addr,
            mapped_key,
            Some(context.inner_mut()),
        )
    }

    #[inline]
    unsafe fn inject_writedata<T>(
        &self,
        buf: &[T],
        data: u64,
        mem_addr: RemoteMemoryAddress,
        mapped_key: &MappedMemoryRegionKey,
    ) -> Result<(), crate::error::Error> {
        self.inject_writedata_impl(buf, data, None, mem_addr, mapped_key)
    }

    #[inline]
    unsafe fn writemsg(
        &self,
        msg: &crate::msg::MsgRmaConnected,
        options: WriteMsgOptions,
    ) -> Result<(), crate::error::Error> {
        self.writemsg_impl(Either::Right(msg), options)
    }
}

impl<EP: ConnectedWriteEp> ConnectedWriteRemoteMemAddrSliceEp for EP {}

impl<EP: RmaCap + WriteMod, EQ: ?Sized + ReadEq, CQ: ?Sized + ReadCq> WriteEpImpl
    for EndpointImplBase<EP, EQ, CQ>
{
}

impl<I: RmaCap + WriteMod, STATE: EpState, CQ: ?Sized + ReadCq> WriteEpImpl
    for TxContextImplBase<I, STATE, CQ>
{
}

impl<I: RmaCap + WriteMod, STATE: EpState, CQ: ?Sized + ReadCq> WriteEpImpl
    for TxContextBase<I, STATE, CQ>
{
}

impl<E: WriteEpImpl> WriteEpImpl for EndpointBase<E, Connected> {}
impl<E: WriteEpImpl> WriteEpImpl for EndpointBase<E, Connectionless> {}

pub trait ReadWriteEp: ReadEp + WriteEp {}
impl<EP: ReadEp + WriteEp> ReadWriteEp for EP {}
