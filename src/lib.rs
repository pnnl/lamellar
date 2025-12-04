#![allow(warnings)]

#[cfg(all(feature = "use-tokio", feature = "use-async-std"))]
compile_error!("Features \"use-tokio\", \"use-async-std\" are mutually exclusive");

#[cfg(not(feature = "thread-safe"))]
use std::cell::OnceCell;
use std::hash::Hash;
use std::ops::Range;
use std::ops::RangeFrom;
use std::ops::RangeFull;
use std::ops::RangeTo;
#[cfg(not(feature = "thread-safe"))]
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
#[cfg(feature = "thread-safe")]
use std::sync::Arc;
#[cfg(feature = "thread-safe")]
use std::sync::OnceLock;

use cq::SingleCompletion;
use domain::DomainBase;
use enums::AddressVectorType;
use eq::Event;
use info::InfoEntry;
use mr::MappedMemoryRegionKey;
use mr::MemoryRegionKey;
#[cfg(feature = "thread-safe")]
use parking_lot::RwLock;

#[cfg(not(feature = "thread-safe"))]
use std::cell::RefCell;

#[cfg(feature = "thread-safe")]
pub type MyRefCell<T> = RwLock<T>;
#[cfg(not(feature = "thread-safe"))]
pub type MyRefCell<T> = RefCell<T>;

#[cfg(feature = "thread-safe")]
pub type MyRc<T> = Arc<T>;
#[cfg(feature = "thread-safe")]
pub type MyOnceCell<T> = OnceLock<T>;

#[cfg(not(feature = "thread-safe"))]
pub type MyRc<T> = Rc<T>;
#[cfg(not(feature = "thread-safe"))]
pub type MyOnceCell<T> = OnceCell<T>;

use std::sync::atomic;

use av::{AddressVectorImplT, AddressVectorSetImpl};

pub mod av;
pub mod cntr;
pub mod cntroptions;
pub mod comm;
pub mod conn_ep;
pub mod cq;
pub mod cqoptions;
pub mod domain;
pub mod enums;
pub mod ep;
pub mod eq;
pub mod eqoptions;
pub mod error;
pub mod fabric;
mod fid;
pub mod info;
pub mod infocapsoptions;
pub mod iovec;
pub mod mr;
pub mod msg;
pub mod nic;
pub mod profile;
pub mod sync;
pub mod trigger;
mod utils;
pub mod xcontext;

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
pub mod async_;
pub mod connless_ep;

#[derive(Clone)]
pub struct TableMappedAddress {
    raw_mapped_addr: libfabric_sys::fi_addr_t,
    av: AddressSource,
}
impl std::fmt::Debug for TableMappedAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableMappedAddress")
            .field("raw_mapped_addr", &self.raw_mapped_addr)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct UnspecMappedAddress {
    raw_mapped_addr: libfabric_sys::fi_addr_t,
}

#[derive(Clone)]
pub struct MapMappedAddress {
    raw_mapped_addr: u64,
    av: AddressSource,
}

impl std::fmt::Debug for MapMappedAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapMappedAddress")
            .field("raw_mapped_addr", &self.raw_mapped_addr)
            .finish()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RemoteMemoryAddress<T = u8> {
    raw_mem_addr: *const T,
}

unsafe impl Send for RemoteMemoryAddress {}
unsafe impl Sync for RemoteMemoryAddress {}

impl<T: Copy> RemoteMemoryAddress<T> {
    pub(crate) fn new(raw_mem_addr: *const T) -> Self {
        Self { raw_mem_addr }
    }

    pub fn from_raw(raw_mem_addr: *const T) -> Self {
        Self { raw_mem_addr }
    }

    pub unsafe fn add(&self, offset: usize) -> Self {
        Self {
            raw_mem_addr: unsafe { self.raw_mem_addr.add(offset) },
        }
    }

    pub unsafe fn sub(&self, offset: usize) -> Self {
        Self {
            raw_mem_addr: unsafe { self.raw_mem_addr.sub(offset) },
        }
    }

    pub unsafe fn offset(&self, offset: isize) -> Self {
        Self {
            raw_mem_addr: unsafe { self.raw_mem_addr.offset(offset) },
        }
    }

    pub unsafe fn offset_from(&self, origin: &RemoteMemoryAddress<T>) -> isize {
        unsafe { self.raw_mem_addr.offset_from(origin.raw_mem_addr) }
    }

    pub fn as_ptr(&self) -> *const T {
        self.raw_mem_addr
    }

    /// Convert to a different type. This is unsafe because the caller must ensure that size and alignment are correct.
    pub unsafe fn as_type<U: Copy>(&self) -> RemoteMemoryAddress<U> {
        RemoteMemoryAddress::new(self.raw_mem_addr as *const U)
    }
}

impl<T> Eq for RemoteMemoryAddress<T> {}

impl<T> PartialEq for RemoteMemoryAddress<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw_mem_addr == other.raw_mem_addr
    }
}

impl<T> PartialOrd for RemoteMemoryAddress<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for RemoteMemoryAddress<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.raw_mem_addr.cmp(&other.raw_mem_addr)
    }
}

#[cfg(target_pointer_width = "64")]
impl<T: Copy> From<RemoteMemoryAddress<T>> for u64 {
    fn from(addr: RemoteMemoryAddress<T>) -> Self {
        addr.raw_mem_addr as u64
    }
}

#[cfg(target_pointer_width = "32")]
impl<T: Copy> From<RemoteMemoryAddress<T>> for u32 {
    fn from(addr: RemoteMemoryAddress<T>) -> Self {
        addr.raw_mem_addr as u32
    }
}

pub struct MemAddressInfo {
    bytes: Vec<u8>,
}

impl MemAddressInfo {
    pub fn from_slice<T: Copy, I>(
        slice_base: &[T],
        offset: usize,
        key: &MemoryRegionKey,
        info: &InfoEntry<I>,
    ) -> MemAddressInfo {
        let addr = if info.domain_attr().mr_mode().is_basic()
            || info.domain_attr().mr_mode().is_virt_addr()
        {
            let base_addr = slice_base.as_ptr() as u64;
            base_addr + (std::mem::size_of::<T>() * offset) as u64
        } else {
            (std::mem::size_of::<T>() * offset) as u64
        };

        let key_raw = match key.key {
            mr::OwnedMemoryRegionKey::Key(_) => false,
            mr::OwnedMemoryRegionKey::RawKey(_) => true,
        };

        let mut key_bytes = key.to_bytes();
        let mut bytes = if key_raw {
            key_bytes
        } else {
            key_bytes.extend(unsafe {
                std::slice::from_raw_parts(
                    &addr as *const u64 as *const u8,
                    std::mem::size_of::<u64>(),
                )
            });
            key_bytes
        };

        let addr_size =
            std::mem::size_of_val(slice_base) - offset as usize * std::mem::size_of::<T>();
        bytes.extend(unsafe {
            std::slice::from_raw_parts(
                &addr_size as *const usize as *const u8,
                std::mem::size_of::<usize>(),
            )
        });
        Self { bytes }
    }

    pub fn to_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn to_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    pub unsafe fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
        }
    }

    pub fn into_remote_info<EQ: ?Sized + SyncSend + 'static>(
        self,
        domain: &DomainBase<EQ>,
    ) -> Result<RemoteMemAddressInfo, crate::error::Error> {
        let mapped_key = self.mapped_mr_key(domain)?;
        let (addr, addr_size) = self.addr();
        Ok(RemoteMemAddressInfo::new(
            RemoteMemoryAddress::new(addr as *const u8),
            addr_size,
            mapped_key,
        ))
    }

    fn mapped_mr_key<EQ: ?Sized + SyncSend + 'static>(
        &self,
        domain: &DomainBase<EQ>,
    ) -> Result<MappedMemoryRegionKey, crate::error::Error> {
        unsafe {
            MappedMemoryRegionKey::from_raw(
                &self.bytes[..self.bytes.len()
                    - (std::mem::size_of::<u64>() + std::mem::size_of::<usize>())],
                domain,
            )
        }
    }

    fn addr(&self) -> (u64, usize) {
        let mut addr = 0u64;
        unsafe {
            std::slice::from_raw_parts_mut(&mut addr as *mut u64 as *mut u8, 8).copy_from_slice(
                &self.bytes[self.bytes.len()
                    - (std::mem::size_of::<u64>() + std::mem::size_of::<usize>())
                    ..self.bytes.len() - std::mem::size_of::<usize>()],
            )
        };

        let mut addr_size = 0usize;
        unsafe {
            std::slice::from_raw_parts_mut(&mut addr_size as *mut usize as *mut u8, 8)
                .copy_from_slice(&self.bytes[self.bytes.len() - std::mem::size_of::<usize>()..])
        };

        (addr, addr_size)
    }
}

pub trait RemoteMemRange {
    fn bounds(&self, len: usize) -> (usize, usize);
}

impl RemoteMemRange for Range<usize> {
    fn bounds(&self, _len: usize) -> (usize, usize) {
        (self.start, self.end)
    }
}

//rdfriese: should this be self.start + len?
impl RemoteMemRange for RangeFrom<usize> {
    fn bounds(&self, len: usize) -> (usize, usize) {
        (self.start, len)
    }
}

impl RemoteMemRange for RangeFull {
    fn bounds(&self, len: usize) -> (usize, usize) {
        (0, len)
    }
}

impl RemoteMemRange for RangeTo<usize> {
    fn bounds(&self, _len: usize) -> (usize, usize) {
        (0, self.end)
    }
}

pub struct RemoteMemAddrSlice<'a, T> {
    mem_address: RemoteMemoryAddress<T>,
    mem_size: usize,
    len: usize,
    key: MappedMemoryRegionKey,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a, T: std::marker::Copy> RemoteMemAddrSlice<'a, T> {
    pub(crate) fn new<TT: Copy>(
        mem_address: RemoteMemoryAddress<TT>,
        len: usize,
        key: MappedMemoryRegionKey,
    ) -> Self {
        Self {
            mem_address: RemoteMemoryAddress::new(mem_address.raw_mem_addr as *const T),
            mem_size: len * std::mem::size_of::<T>(),
            len,
            key,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn mem_address(&self) -> RemoteMemoryAddress<T> {
        self.mem_address
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn mem_size(&self) -> usize {
        self.mem_size
    }

    pub fn key(&self) -> &MappedMemoryRegionKey {
        &self.key
    }

    pub fn split_at(&self, mid: usize) -> (Self, Self) {
        let first_start = self.mem_address;
        assert!(mid <= self.len, "Split index out of bounds");
        let second_start = unsafe { self.mem_address.add(mid * std::mem::size_of::<T>()) };

        let first = Self::new(first_start, mid, self.key.clone());

        let second = Self::new(second_start, self.len - mid, self.key.clone());

        (first, second)
    }

    pub unsafe fn split_at_unchecked(&self, mid: usize) -> (Self, Self) {
        let first_start = self.mem_address;
        let second_start = unsafe { self.mem_address.add(mid) };

        let first = Self::new(first_start, mid, self.key.clone());

        let second = Self::new(second_start, self.mem_size() - mid, self.key.clone());

        (first, second)
    }
}

pub struct RemoteMemAddrSliceMut<'a, T> {
    mem_address: RemoteMemoryAddress<T>,
    mem_size: usize,
    len: usize,
    key: MappedMemoryRegionKey,
    phantom: std::marker::PhantomData<&'a mut ()>,
}

impl<'a, T: Copy> RemoteMemAddrSliceMut<'a, T> {
    pub(crate) fn new<TT: Copy>(
        mem_address: RemoteMemoryAddress<TT>,
        len: usize,
        key: MappedMemoryRegionKey,
    ) -> Self {
        Self {
            mem_address: RemoteMemoryAddress::new(mem_address.raw_mem_addr as *const T),
            mem_size: len * std::mem::size_of::<T>(),
            len,
            key,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn mem_address(&self) -> RemoteMemoryAddress<T> {
        self.mem_address
    }

    pub fn mem_size(&self) -> usize {
        self.mem_size
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn key(&self) -> &MappedMemoryRegionKey {
        &self.key
    }

    pub fn split_at_mut(&mut self, mid: usize) -> (Self, Self) {
        let first_start = self.mem_address;
        assert!(mid <= self.len, "Split index out of bounds");
        let second_start = unsafe { self.mem_address.add(mid * std::mem::size_of::<T>()) };

        let first = Self::new(first_start, mid, self.key.clone());

        let second = Self::new(second_start, self.len - mid, self.key.clone());

        (first, second)
    }

    pub fn split_at_mut_unchecked(&mut self, mid: usize) -> (Self, Self) {
        let first_start = self.mem_address;
        let second_start = unsafe { self.mem_address.add(mid * std::mem::size_of::<T>()) };

        let first = Self::new(first_start, mid, self.key.clone());

        let second = Self::new(second_start, self.len - mid, self.key.clone());

        (first, second)
    }
}

#[derive(Clone)]
pub struct RemoteMemAddressInfo {
    mem_address: RemoteMemoryAddress,
    len: usize,
    key: MappedMemoryRegionKey,
}

impl RemoteMemAddressInfo {
    pub(crate) fn new(
        mem_address: RemoteMemoryAddress,
        len: usize,
        key: MappedMemoryRegionKey,
    ) -> Self {
        Self {
            mem_address,
            len,
            key,
        }
    }

    pub fn mem_address(&self) -> RemoteMemoryAddress {
        self.mem_address
    }

    pub fn mem_len(&self) -> usize {
        self.len
    }

    pub fn key(&self) -> MappedMemoryRegionKey {
        self.key.clone()
    }

    pub fn slice<T: Copy>(&self, range: impl RemoteMemRange) -> RemoteMemAddrSlice<'_, T> {
        let (start, end) = range.bounds(self.len);
        assert!(
            start < end,
            "Invalid range for remote memory slice: start: {}, end: {}",
            start,
            end
        );
        let len = end - start;
        let (start, end) = (
            start * std::mem::size_of::<T>(),
            end * std::mem::size_of::<T>(),
        );
        // Ensure that the slice is within bounds
        assert!(
            start < self.len,
            "Out of bounds access to remote memory slice"
        );
        assert!(
            end - 1 < self.len,
            "Out of bounds access to remote memory slice"
        );
        RemoteMemAddrSlice::new(
            unsafe { self.mem_address.add(start) },
            len,
            self.key.clone(),
        )
    }

    pub unsafe fn slice_unchecked<T: Copy>(
        &self,
        range: impl RemoteMemRange,
    ) -> RemoteMemAddrSlice<'_, T> {
        let (start, end) = range.bounds(self.len);
        let len = end - start;
        let start = start * std::mem::size_of::<T>();
        RemoteMemAddrSlice::new(self.mem_address.add(start), len, self.key.clone())
    }

    pub fn slice_mut<T: Copy>(
        &mut self,
        range: impl RemoteMemRange,
    ) -> RemoteMemAddrSliceMut<'_, T> {
        let (start, end) = range.bounds(self.len);
        assert!(
            start < end,
            "Invalid range for remote memory slice: start: {}, end: {}",
            start,
            end
        );
        let len = end - start;
        let (start, end) = (
            start * std::mem::size_of::<T>(),
            end * std::mem::size_of::<T>(),
        );
        assert!(
            start < self.len,
            "Out of bounds access to remote memory slice"
        );
        assert!(
            end - 1 < self.len,
            "Out of bounds access to remote memory slice"
        );
        RemoteMemAddrSliceMut::new(
            unsafe { self.mem_address.add(start) },
            len,
            self.key.clone(),
        )
    }

    pub unsafe fn slice_mut_unchecked<T: Copy>(
        &mut self,
        range: impl RemoteMemRange,
    ) -> RemoteMemAddrSliceMut<'_, T> {
        let (start, end) = range.bounds(self.len);
        let len = end - start;
        let start = start * std::mem::size_of::<T>();
        RemoteMemAddrSliceMut::new(
            unsafe { self.mem_address.add(start) },
            len,
            self.key.clone(),
        )
    }

    // SAFETY: using this function essentially creates mutable pointers to the same memory region.
    // TODO: see if lamellar can manage this some other way...
    pub unsafe fn sub_region(&self, range: impl RemoteMemRange) -> RemoteMemAddressInfo {
        let (start, end) = range.bounds(self.len);
        // We can create a zero-length sub-region.
        assert!(
            start <= end,
            "Invalid range for remote memory sub-region: start: {}, end: {}",
            start,
            end
        );
        let len = end - start;
        // println!("sub_region start: {} end: {} len: {} full_len: {}", start, end, len, self.len);
        // Ensure that the sub-region is within bounds

        // We can create a zero-length sub-region.
        assert!(
            start <= self.len,
            "Out of bounds access to remote memory sub-region start:{}  len: {}",
            start,
            self.len
        );
        assert!(
            end - 1 < self.len,
            "Out of bounds access to remote memory sub-region start:{}  len: {}",
            start,
            self.len
        );
        RemoteMemAddressInfo::new(
            unsafe { self.mem_address.add(start) },
            len,
            self.key.clone(),
        )
    }

    pub fn contains(&self, addr: &usize) -> bool {
        // let addr = addr.raw_mem_addr as usize;
        let start = self.mem_address.raw_mem_addr as usize;
        let end = start + self.len;
        addr >= &start && addr < &end
    }
}

#[derive(Clone, Debug)]
pub enum RawMappedAddress {
    Map(libfabric_sys::fi_addr_t),
    Table(libfabric_sys::fi_addr_t),
    Unspec(libfabric_sys::fi_addr_t),
}

impl RawMappedAddress {
    pub(crate) fn get(&self) -> libfabric_sys::fi_addr_t {
        match self {
            RawMappedAddress::Map(addr) => *addr,
            RawMappedAddress::Table(addr) => *addr,
            RawMappedAddress::Unspec(addr) => *addr,
        }
    }

    pub(crate) fn from_raw(
        av_type: AddressVectorType,
        raw_addr: libfabric_sys::fi_addr_t,
    ) -> RawMappedAddress {
        match av_type {
            AddressVectorType::Map => RawMappedAddress::Map(raw_addr),
            AddressVectorType::Table => RawMappedAddress::Table(raw_addr),
            AddressVectorType::Unspec => panic!("Unspecified address type"),
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)] // We only keep these Rcs to prevent the AddressVector from being deallocated while the respective address is still in use.
pub(crate) enum AddressSource {
    Av(MyRc<dyn AddressVectorImplT>),
    AvSet(MyRc<AddressVectorSetImpl>),
}

/// Owned wrapper around a libfabric `fi_addr_t`.
///
/// This type wraps an instance of a `fi_addr_t`, in order to prevent modification and to monitor its lifetime.
/// It is usually generated by an [crate::av::AddressVector] after inserting an [crate::ep::Address].
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.22.0/man/fi_av.3.html).
///
/// Note that other objects that it will extend the respective [`crate::av::AddressVector`]'s (if any) lifetime until they
/// it is dropped.
#[derive(Debug)]
pub enum MappedAddress {
    Unspec(UnspecMappedAddress),
    Map(MapMappedAddress),
    Table(TableMappedAddress),
}

impl MappedAddress {
    // pub(crate) fn from_raw_addr_trait(addr: RawMappedAddress, av: &MyRc<dyn AddressVectorImplT>) -> Self {
    //     let avcell = OnceCell::new();

    //     if avcell.set(AddressSource::Av(av.clone())).is_err() {
    //         panic!("MappedAddress is already set");
    //     }

    //     Self {
    //         addr,
    //         av: avcell,
    //     }
    // }

    pub(crate) fn from_raw_addr(addr: RawMappedAddress, av: AddressSource) -> Self {
        match addr {
            RawMappedAddress::Map(addr) => Self::Map(MapMappedAddress {
                raw_mapped_addr: addr,
                av,
            }),
            RawMappedAddress::Table(addr) => Self::Table(TableMappedAddress {
                raw_mapped_addr: addr,
                av,
            }),
            RawMappedAddress::Unspec(addr) => Self::Unspec(UnspecMappedAddress {
                raw_mapped_addr: addr,
            }),
        }
    }

    pub(crate) fn from_raw_addr_no_av(addr: RawMappedAddress) -> Self {
        match addr {
            RawMappedAddress::Unspec(addr) => Self::Unspec(UnspecMappedAddress {
                raw_mapped_addr: addr,
            }),
            _ => panic!("Addresses mapped by an AV"),
        }
    }

    pub(crate) fn raw_addr(&self) -> libfabric_sys::fi_addr_t {
        match self {
            Self::Map(t) => t.raw_mapped_addr,
            Self::Table(m) => m.raw_mapped_addr,
            Self::Unspec(u) => u.raw_mapped_addr,
        }
    }

    pub fn rx_addr(
        &self,
        rx_index: i32,
        rx_ctx_bits: i32,
    ) -> Result<MappedAddress, crate::error::Error> {
        let ret =
            unsafe { libfabric_sys::inlined_fi_rx_addr(self.raw_addr(), rx_index, rx_ctx_bits) };
        if ret == FI_ADDR_NOTAVAIL || ret == FI_ADDR_UNSPEC {
            return Err(crate::error::Error::from_err_code(
                libfabric_sys::FI_EADDRNOTAVAIL,
            ));
        }

        Ok(match self {
            Self::Map(m) => Self::Map(MapMappedAddress {
                raw_mapped_addr: ret,
                av: m.av.clone(),
            }),
            Self::Table(t) => Self::Table(TableMappedAddress {
                raw_mapped_addr: ret,
                av: t.av.clone(),
            }),
            Self::Unspec(_) => Self::Unspec(UnspecMappedAddress {
                raw_mapped_addr: ret,
            }),
        })
    }
}

pub type DataType = libfabric_sys::fi_datatype;
pub type CollectiveOp = libfabric_sys::fi_collective_op;
const FI_ADDR_NOTAVAIL: u64 = u64::MAX;
const FI_KEY_NOTAVAIL: u64 = u64::MAX;
const FI_ADDR_UNSPEC: u64 = u64::MAX;

// pub struct Stx {

//     #[allow(dead_code)]
//     c_stx: *mut libfabric_sys::fid_stx,
// }

// impl Stx {
//     pub(crate) fn new<T0>(domain: &crate::domain::Domain, mut attr: crate::TxAttr, context: &mut T0) -> Result<Stx, error::Error> {
//         let mut c_stx: *mut libfabric_sys::fid_stx = std::ptr::null_mut();
//         let c_stx_ptr: *mut *mut libfabric_sys::fid_stx = &mut c_stx;
//         let err = unsafe { libfabric_sys::inlined_fi_stx_context(domain.c_domain, attr.get_mut(), c_stx_ptr, context as *mut T0 as *mut std::ffi::c_void) };

//         if err != 0 {
//             Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
//         }
//         else {
//             Ok(
//                 Self { c_stx }
//             )
//         }

//     }
// }

// pub struct SrxAttr {
//     c_attr: libfabric_sys::fi_srx_attr,
// }

// impl SrxAttr {
//     pub(crate) fn get(&self) -> *const libfabric_sys::fi_srx_attr {
//         &self.c_attr
//     }

//     pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_srx_attr {
//         &mut self.c_attr
//     }
// }

// struct fi_param {
// 	const char *name;
// 	enum fi_param_type type;
// 	const char *help_string;
// 	const char *value;
// };

// int fi_getparams(struct fi_param **params, int *count);
// void fi_freeparams(struct fi_param *params);

// pub struct Param {
//     c_param : libfabric_sys::fi_param,
// }

// pub fn get_params() -> Vec<Param> {
//     let mut len = 0 as i32;
//     let len_ptr : *mut i32 = &mut len;
//     let mut c_params: *mut libfabric_sys::fi_param = std::ptr::null_mut();
//     let mut c_params_ptr: *mut *mut libfabric_sys::fi_param = &mut c_params;

//     let err = libfabric_sys::fi_getparams(c_params_ptr, len_ptr);
//     if err != 0 {
//         panic!("fi_getparams failed {}", err);
//     }

//     let mut params = Vec::<Param>::new();
//     for i  in 0..len {
//         params.push(Param { c_param: unsafe{ c_params.add(i as usize) } });
//     }

//     params
// }

// pub struct Param {
//     c_param: libfabric_sys::fi_param,
// }
// #[cfg(feature = "thread-safe")]
// pub type CtxState = AtomicI8;
// #[cfg(not(feature = "thread-safe"))]
// pub type CtxState = i8;

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

pub(crate) enum ContextState {
    Cq(Result<SingleCompletion, crate::error::Error>),
    Eq(Result<Event, crate::error::Error>),
}

#[repr(C)]
struct Context1 {
    #[allow(dead_code)]
    c_val: libfabric_sys::fi_context,
    pub(crate) id: usize,
    pub(crate) ready: AtomicBool,
    pub(crate) state: MyOnceCell<ContextState>,
    pub(crate) waker: MyOnceCell<std::task::Waker>,
}

impl Context1 {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            c_val: {
                libfabric_sys::fi_context {
                    internal: [std::ptr::null_mut(); 4],
                }
            },
            ready: AtomicBool::new(false),
            state: MyOnceCell::new(),
            waker: MyOnceCell::new(),
        }
    }

    // pub(crate) fn reset(&mut self) {
    //     self.state = MyOnceCell::new();
    // }

    pub(crate) fn set_completion_done(
        &mut self,
        completion: Result<SingleCompletion, crate::error::Error>,
    ) {
        if self.state.set(ContextState::Cq(completion)).is_err() {
            panic!("Already initialized")
        }
        self.ready.store(true, atomic::Ordering::Relaxed)
    }

    pub(crate) fn set_event_done(&mut self, event: Result<Event, crate::error::Error>) {
        if self.state.set(ContextState::Eq(event)).is_err() {
            panic!("Already initialized")
        }
        self.ready.store(true, atomic::Ordering::Relaxed)
    }
}

// impl Default for Context1 {
//     fn default() -> Self {
//         Self::new()
//     }
// }

#[repr(C)]
struct Context2 {
    c_val: libfabric_sys::fi_context2,
    pub(crate) id: usize,
    state: MyOnceCell<ContextState>,
    pub(crate) ready: AtomicBool,
    pub(crate) waker: MyOnceCell<std::task::Waker>,
}

impl Context2 {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            c_val: {
                libfabric_sys::fi_context2 {
                    internal: [std::ptr::null_mut(); 8],
                }
            },
            ready: AtomicBool::new(false),
            state: MyOnceCell::new(),
            waker: MyOnceCell::new(),
        }
    }

    // pub(crate) fn reset(&mut self) {
    //     self.state = None
    // }

    pub(crate) fn set_completion_done(
        &mut self,
        completion: Result<SingleCompletion, crate::error::Error>,
    ) {
        if self.state.set(ContextState::Cq(completion)).is_err() {
            panic!("Already initialized")
        }
        self.ready.store(true, atomic::Ordering::Relaxed)
    }

    pub(crate) fn set_event_done(&mut self, event: Result<Event, crate::error::Error>) {
        if self.state.set(ContextState::Eq(event)).is_err() {
            panic!("Already initialized")
        }
        self.ready.store(true, atomic::Ordering::Relaxed)
    }
}

// impl Default for Context2 {
//     fn default() -> Self {
//         Self::new()
//     }
// }

enum ContextType {
    Context1(Box<Context1>),
    Context2(Box<Context2>),
}

// We use heap allocated data to allow moving the wrapper field
// without affecting the pointer used by libfabric

pub struct Context(ContextType);

impl ContextType {
    fn inner_mut(&mut self) -> *mut std::ffi::c_void {
        match self {
            ContextType::Context1(ctx) => &mut *(*(ctx)) as *mut Context1 as *mut std::ffi::c_void,
            ContextType::Context2(ctx) => &mut *(*(ctx)) as *mut Context2 as *mut std::ffi::c_void,
        }
    }

    #[allow(dead_code)]
    fn id(&self) -> usize {
        match self {
            ContextType::Context1(ctx) => ctx.id,
            ContextType::Context2(ctx) => ctx.id,
        }
    }

    fn inner(&self) -> *const std::ffi::c_void {
        match self {
            ContextType::Context1(ctx) => &*(*(ctx)) as *const Context1 as *const std::ffi::c_void,
            ContextType::Context2(ctx) => &*(*(ctx)) as *const Context2 as *const std::ffi::c_void,
        }
    }

    pub(crate) fn set_completion_done(
        &mut self,
        comp: Result<SingleCompletion, crate::error::Error>,
    ) {
        match self {
            ContextType::Context1(ctx) => ctx.set_completion_done(comp),
            ContextType::Context2(ctx) => ctx.set_completion_done(comp),
        }
    }

    pub(crate) fn set_event_done(&mut self, event: Result<Event, crate::error::Error>) {
        match self {
            ContextType::Context1(ctx) => ctx.set_event_done(event),
            ContextType::Context2(ctx) => ctx.set_event_done(event),
        }
    }

    pub(crate) fn ready(&self) -> bool {
        match self {
            ContextType::Context1(ctx) => ctx.ready.load(atomic::Ordering::Relaxed) == true,
            ContextType::Context2(ctx) => ctx.ready.load(atomic::Ordering::Relaxed) == true,
        }
    }

    pub(crate) fn reset(&mut self) {
        match self {
            ContextType::Context1(ctx) => {
                ctx.ready.store(false, atomic::Ordering::Relaxed);
                ctx.state.take();
            }
            ContextType::Context2(ctx) => {
                ctx.ready.store(false, atomic::Ordering::Relaxed);
                ctx.state.take();
            }
        }
    }

    pub(crate) fn state(&mut self) -> &mut MyOnceCell<ContextState> {
        match self {
            ContextType::Context1(ctx) => &mut ctx.state,
            ContextType::Context2(ctx) => &mut ctx.state,
        }
    }

    pub(crate) fn set_waker(&mut self, waker: std::task::Waker) {
        match self {
            ContextType::Context1(ctx) => {
                ctx.waker.take(); // Clear any previous waker
                ctx.waker.set(waker).unwrap(); // Set the new waker
            }
            ContextType::Context2(ctx) => {
                ctx.waker.take(); // Clear any previous waker
                ctx.waker.set(waker).unwrap(); // Set the new waker
            }
        }
    }

    pub(crate) fn get_waker(&mut self) -> MyOnceCell<std::task::Waker> {
        match self {
            ContextType::Context1(ctx) => ctx.waker.clone(),
            ContextType::Context2(ctx) => ctx.waker.clone(),
        }
    }

    pub(crate) fn get_type(&self) -> usize {
        match self {
            ContextType::Context1(_) => 1,
            ContextType::Context2(_) => 2,
        }
    }
}

impl Context {
    fn inner_mut(&mut self) -> *mut std::ffi::c_void {
        self.0.inner_mut()
    }

    fn inner(&self) -> *const std::ffi::c_void {
        self.0.inner()
    }

    pub(crate) fn state(&mut self) -> &mut MyOnceCell<ContextState> {
        self.0.state()
    }

    #[allow(dead_code)]
    pub(crate) fn set_completion_done(
        &mut self,
        comp: Result<SingleCompletion, crate::error::Error>,
    ) {
        self.0.set_completion_done(comp)
    }

    #[allow(dead_code)]
    pub(crate) fn set_event_done(&mut self, comp: Result<Event, crate::error::Error>) {
        self.0.set_event_done(comp)
    }

    pub(crate) fn reset(&mut self) {
        self.0.reset()
    }

    pub(crate) fn ready(&self) -> bool {
        self.0.ready()
    }
}

// pub trait BindImpl: AsRawFid {}
// pub trait Bind {
//     fn inner(&self) -> MyRc<dyn AsRawFid>;
// }

pub trait FdRetrievable {}
pub trait Waitable {}
pub trait Writable {}
pub trait WaitRetrievable {}

pub enum FabInfoCaps {
    MSG = 0,
    RMA = 1,
    TAG = 2,
    ATOMIC = 3,
    MCAST = 4,
    NAMEDRXCTX = 5,
    DRECV = 6,
    VMSG = 7,
    HMEM = 8,
    AVUSERID = 9,
    COLL = 10,
    XPU = 11,
    SEND = 12,
    RECV = 13,
    WRITE = 14,
    READ = 15,
    RWRITE = 16,
    RREAD = 17,
}

impl FabInfoCaps {
    pub const fn value(&self) -> usize {
        match self {
            FabInfoCaps::MSG => FabInfoCaps::MSG as usize,
            FabInfoCaps::RMA => FabInfoCaps::RMA as usize,
            FabInfoCaps::TAG => FabInfoCaps::TAG as usize,
            FabInfoCaps::ATOMIC => FabInfoCaps::ATOMIC as usize,
            FabInfoCaps::MCAST => FabInfoCaps::MCAST as usize,
            FabInfoCaps::NAMEDRXCTX => FabInfoCaps::NAMEDRXCTX as usize,
            FabInfoCaps::DRECV => FabInfoCaps::DRECV as usize,
            FabInfoCaps::VMSG => FabInfoCaps::VMSG as usize,
            FabInfoCaps::HMEM => FabInfoCaps::HMEM as usize,
            FabInfoCaps::AVUSERID => FabInfoCaps::AVUSERID as usize,
            FabInfoCaps::COLL => FabInfoCaps::COLL as usize,
            FabInfoCaps::XPU => FabInfoCaps::XPU as usize,
            FabInfoCaps::SEND => FabInfoCaps::SEND as usize,
            FabInfoCaps::RECV => FabInfoCaps::RECV as usize,
            FabInfoCaps::WRITE => FabInfoCaps::WRITE as usize,
            FabInfoCaps::READ => FabInfoCaps::READ as usize,
            FabInfoCaps::RWRITE => FabInfoCaps::RWRITE as usize,
            FabInfoCaps::RREAD => FabInfoCaps::RREAD as usize,
        }
    }
}
pub enum SyncCaps {
    WAIT = 0,
    RETRIEVE = 1,
    FD = 2,
}

pub use SyncCaps as CntrCaps;
pub use SyncCaps as CqCaps;

pub enum EqCaps {
    WAIT = 0,
    RETRIEVE = 1,
    FD = 2,
    WRITE = 3,
}

impl SyncCaps {
    pub const fn value(&self) -> usize {
        match self {
            SyncCaps::WAIT => SyncCaps::WAIT as usize,
            SyncCaps::RETRIEVE => SyncCaps::RETRIEVE as usize,
            SyncCaps::FD => SyncCaps::FD as usize,
        }
    }
}

impl EqCaps {
    pub const fn value(&self) -> usize {
        match self {
            EqCaps::WAIT => EqCaps::WAIT as usize,
            EqCaps::RETRIEVE => EqCaps::RETRIEVE as usize,
            EqCaps::FD => EqCaps::FD as usize,
            EqCaps::WRITE => EqCaps::WRITE as usize,
        }
    }
}

pub const fn get_eq<const N: usize>(index: EqCaps, asked: &[EqCaps]) -> bool {
    let mut i = 0;
    while i < N {
        if index.value() == asked[i].value() {
            return true;
        }
        i += 1;
    }
    false
}

pub const fn get_sync<const N: usize>(index: SyncCaps, asked: &[SyncCaps]) -> bool {
    let mut i = 0;
    while i < N {
        if index.value() == asked[i].value() {
            return true;
        }
        i += 1;
    }
    false
}

pub const fn get_info<const N: usize>(index: FabInfoCaps, asked: &[FabInfoCaps]) -> bool {
    let mut i = 0;
    while i < N {
        if index.value() == asked[i].value() {
            return true;
        }
        i += 1;
    }
    false
}

#[macro_export] // MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD
macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + libfabric::count!($($xs)*));
}
// macro_rules! set {
//     ($N: stmt, $opt: ident $opts: expr, ) => {
//         {get::<{$N}>($opt, &[$($opts),*])}
//     };
// }
#[macro_export] // MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD
macro_rules! info_caps_type_N {
    ($N: stmt, $($opt: expr),*) => {
        libfabric::infocapsoptions::InfoCaps<
        // set($N, MSG, $($opt),*),
        {libfabric::get_info::<{$N}>(FabInfoCaps::MSG, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::RMA, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::TAG, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::ATOMIC, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::MCAST, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::NAMEDRXCTX, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::DRECV, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::VMSG, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::HMEM, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::COLL, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::XPU, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::AVUSERID, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::SEND, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::RECV, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::WRITE, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::READ, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::RWRITE, &[$($opt),*])},
        {libfabric::get_info::<{$N}>(FabInfoCaps::RREAD, &[$($opt),*])}>

    };
}

#[macro_export] // CQ: WAIT, RETRIEVE, FD
macro_rules! cq_caps_type_N {
    ($N: stmt, $($opt: expr),*) => {
        libfabric::cq::CompletionQueueImpl<
        // set($N, MSG, $($opt),*),
        {libfabric::get_sync::<{$N}>(libfabric::SyncCaps::WAIT, &[$($opt),*])},
        {libfabric::get_sync::<{$N}>(libfabric::SyncCaps::RETRIEVE, &[$($opt),*])},
        {libfabric::get_sync::<{$N}>(libfabric::SyncCaps::FD, &[$($opt),*])}, >
    };
}

#[macro_export] // EQ: WRITE, WAIT, RETRIEVE, FD
macro_rules! eq_caps_type_N {
    ($N: stmt, $($opt: expr),*) => {
        libfabric::eq::EventQueueImpl<
        // set($N, MSG, $($opt),*),
        {libfabric::get_eq::<{$N}>(libfabric::EqCaps::WRITE, &[$($opt),*])},
        {libfabric::get_eq::<{$N}>(libfabric::EqCaps::WAIT, &[$($opt),*])},
        {libfabric::get_eq::<{$N}>(libfabric::EqCaps::RETRIEVE, &[$($opt),*])},
        {libfabric::get_eq::<{$N}>(libfabric::EqCaps::FD, &[$($opt),*])},>
    };
}

#[macro_export] // CNTR: WAIT, RETRIEVE, FD
macro_rules! cntr_caps_type_N {
    ($N: stmt, $($opt: expr),*) => {
        libfabric::cntr::CounterImpl<
        // set($N, MSG, $($opt),*),
        {libfabric::get_sync::<{$N}>(libfabric::SyncCaps::WAIT, &[$($opt),*])},
        {libfabric::get_sync::<{$N}>(libfabric::SyncCaps::RETRIEVE, &[$($opt),*])},
        {libfabric::get_sync::<{$N}>(libfabric::SyncCaps::FD, &[$($opt),*])}, >
    };
}

#[macro_export] // MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD
macro_rules!  info_caps_type{
    ($($opt: expr),*) => {
        libfabric::info_caps_type_N!(libfabric::count!($($opt)*), $($opt),*)
    };
}

#[macro_export] // MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD
macro_rules!  cq_caps_type{
    ($($opt: expr),*) => {
        libfabric::cq_caps_type_N!(libfabric::count!($($opt)*), $($opt),*)
    };
}

#[macro_export] // MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD
macro_rules!  eq_caps_type{
    ($($opt: expr),*) => {
        libfabric::eq_caps_type_N!(libfabric::count!($($opt)*), $($opt),*)
    };
}

#[macro_export] // MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD
macro_rules!  cntr_caps_type{
    ($($opt: expr),*) => {
        libfabric::cntr_caps_type_N!(libfabric::count!($($opt)*), $($opt),*)
    };
}

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
#[macro_export] // MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD
macro_rules! async_cq_caps_type_N {
    ($N: stmt, $($opt: expr),*) => {
        libfabric::async_::cq::AsyncCompletionQueueImpl
    };
}

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
#[macro_export] // MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD
macro_rules! async_eq_caps_type_N {
    ($N: stmt, $($opt: expr),*) => {
        libfabric::async_::eq::AsyncEventQueueImpl<
        // set($N, MSG, $($opt),*),
        {libfabric::get_eq::<{$N}>(libfabric::EqCaps::WRITE, &[$($opt),*])},>
    };
}

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
#[macro_export] // MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD
macro_rules!  async_cq_caps_type{
    ($($opt: expr),*) => {
        libfabric::async_cq_caps_type_N!(libfabric::count!($($opt)*), $($opt),*)
    };
}

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
#[macro_export] // MSG, RMA, TAG, ATOMIC, MCAST, NAMEDRXCTX, DRECV, VMSG, HMEM, COLL, XPU, SEND, RECV, WRITE, READ, RWRITE, RREAD
macro_rules!  async_eq_caps_type{
    ($($opt: expr),*) => {
        libfabric::async_eq_caps_type_N!(libfabric::count!($($opt)*), $($opt),*)
    };
}

pub trait AsFiType: Copy {
    fn as_fi_datatype() -> libfabric_sys::fi_datatype;
}

macro_rules! impl_as_fi_type {
    ($(($rtype: ty, $fitype: path)),*) => {
        $(impl AsFiType for $rtype {
            fn as_fi_datatype() -> libfabric_sys::fi_datatype {
                $fitype
            }
        })*
    };
}

impl_as_fi_type!(
    ((), libfabric_sys::fi_datatype_FI_VOID),
    (i8, libfabric_sys::fi_datatype_FI_INT8),
    (i16, libfabric_sys::fi_datatype_FI_INT16),
    (i32, libfabric_sys::fi_datatype_FI_INT32),
    (i64, libfabric_sys::fi_datatype_FI_INT64),
    (i128, libfabric_sys::fi_datatype_FI_INT128),
    (u8, libfabric_sys::fi_datatype_FI_UINT8),
    (u16, libfabric_sys::fi_datatype_FI_UINT16),
    (u32, libfabric_sys::fi_datatype_FI_UINT32),
    (u64, libfabric_sys::fi_datatype_FI_UINT64),
    (u128, libfabric_sys::fi_datatype_FI_UINT128),
    (f32, libfabric_sys::fi_datatype_FI_FLOAT),
    (f64, libfabric_sys::fi_datatype_FI_DOUBLE)
);

impl AsFiType for usize {
    fn as_fi_datatype() -> libfabric_sys::fi_datatype {
        if std::mem::size_of::<usize>() == 8 {
            libfabric_sys::fi_datatype_FI_UINT64
        } else if std::mem::size_of::<usize>() == 4 {
            libfabric_sys::fi_datatype_FI_UINT32
        } else if std::mem::size_of::<usize>() == 2 {
            libfabric_sys::fi_datatype_FI_UINT16
        } else if std::mem::size_of::<usize>() == 1 {
            libfabric_sys::fi_datatype_FI_UINT8
        } else {
            panic!("Unhandled usize datatype size")
        }
    }
}

impl AsFiType for isize {
    fn as_fi_datatype() -> libfabric_sys::fi_datatype {
        if std::mem::size_of::<isize>() == 8 {
            libfabric_sys::fi_datatype_FI_INT64
        } else if std::mem::size_of::<isize>() == 4 {
            libfabric_sys::fi_datatype_FI_INT32
        } else if std::mem::size_of::<isize>() == 2 {
            libfabric_sys::fi_datatype_FI_INT16
        } else if std::mem::size_of::<isize>() == 1 {
            libfabric_sys::fi_datatype_FI_INT8
        } else {
            panic!("Unhandled isize datatype size")
        }
    }
}
#[cfg(feature = "thread-safe")]
pub trait SyncSend: Sync + Send {}

#[cfg(not(feature = "thread-safe"))]
pub trait SyncSend {}
