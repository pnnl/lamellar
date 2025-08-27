use std::{marker::PhantomData, ptr::null};

#[allow(unused_imports)]
// use crate::fid::AsFid;
use crate::{
    cntr::ReadCntr,
    domain::DomainImplT,
    enums::{MrAccess, MrRegOpt},
    ep::{BaseEndpoint, EpState},
    fid::{self, AsRawFid, AsRawTypedFid, MrRawFid, OwnedMrFid, RawFid},
    iovec::IoVec,
    utils::check_error,
    Context, MyOnceCell, MyRc, SyncSend,
};
use crate::{
    ep::ActiveEndpoint,
    error::Error,
    fid::{AsTypedFid, BorrowedTypedFid},
};

/// Represents a DMA buffer.
/// 
/// Corresponds to `libfabric_sys::fi_mr_dmabuf`.
pub struct DmaBuf {
    c_dmabuf: libfabric_sys::fi_mr_dmabuf,
}
pub(crate) enum OwnedMemoryRegionKey {
    Key(u64),
    RawKey((Vec<u8>, u64)),
}


/// Represents a key needed to access a remote [MemoryRegion].
pub struct MemoryRegionKey<'a> {
    pub(crate) key: &'a OwnedMemoryRegionKey,
}

impl<'a> MemoryRegionKey<'a> {
    /// Convert the memory region key to a byte representation.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.key.to_bytes()
    }
}

impl OwnedMemoryRegionKey {
    /// Construct a new [OwnedMemoryRegionKey] from a slice of bytes, usually received
    /// from remote node using raw keys.
    ///
    /// # Safety
    /// This function is unsafe since there is not guarantee that the bytes read indeed represent
    /// a key
    ///
    unsafe fn from_bytes<EQ: ?Sized + SyncSend>(
        raw: &[u8],
        domain: &crate::domain::DomainBase<EQ>,
    ) -> Self {
        OwnedMemoryRegionKey::from_bytes_impl(raw, &*domain.inner)
    }

    /// Convert the memory region key to a byte representation.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            OwnedMemoryRegionKey::Key(key) => {
                let mut bytes = vec![0; std::mem::size_of::<u64>()];
                unsafe {
                    bytes.copy_from_slice(std::slice::from_raw_parts(
                        key as *const u64 as *const u8,
                        std::mem::size_of::<u64>(),
                    ))
                };
                bytes
            }
            OwnedMemoryRegionKey::RawKey(key) => [&key.0[..], unsafe {
                std::slice::from_raw_parts(
                    &key.1 as *const u64 as *const u8,
                    std::mem::size_of::<u64>(),
                )
            }]
            .concat(),
        }
    }

    unsafe fn from_bytes_impl(raw: &[u8], domain: &(impl DomainImplT + ?Sized)) -> Self {
        if domain.mr_mode().is_raw() {
            assert!(raw.len() == domain.mr_key_size() + std::mem::size_of::<u64>());
            let base_addr = *(raw[raw.len() - std::mem::size_of::<u64>()..].as_ptr() as *const u64);
            Self::RawKey((
                raw[0..raw.len() - std::mem::size_of::<u64>()].to_vec(),
                base_addr,
            ))
        } else {
            let mut key = 0u64;
            unsafe {
                std::slice::from_raw_parts_mut(&mut key as *mut u64 as *mut u8, 8)
                    .copy_from_slice(raw)
            };
            Self::Key(key)
        }
    }

    fn as_borrowed(&self) -> MemoryRegionKey {
        MemoryRegionKey { key: self }
    }

    fn into_mapped<EQ: ?Sized + 'static + SyncSend>(
        mut self,
        domain: &crate::domain::DomainBase<EQ>,
    ) -> Result<MappedMemoryRegionKey, crate::error::Error> {
        match self {
            OwnedMemoryRegionKey::Key(mapped_key) => Ok(MappedMemoryRegionKey {
                inner: MyRc::new(MappedMemoryRegionKeyImpl::Key(mapped_key)),
            }),
            OwnedMemoryRegionKey::RawKey(_) => {
                let mapped_key = domain.map_raw(&mut self, 0)?;
                Ok(MappedMemoryRegionKey {
                    inner: MyRc::new(MappedMemoryRegionKeyImpl::MappedRawKey((
                        mapped_key,
                        domain.inner.clone(),
                    ))),
                })
            }
        }
    }
}

enum MappedMemoryRegionKeyImpl {
    Key(u64),
    MappedRawKey((u64, MyRc<dyn DomainImplT>)),
}

/// Uniformly represents a (mapped if raw) memory region  key that can be used to
/// access remote [MemoryRegion]s. This struct will automatically unmap the key
/// if needed when it is dropped.
#[derive(Clone)]
pub struct MappedMemoryRegionKey {
    inner: MyRc<MappedMemoryRegionKeyImpl>,
}

impl MappedMemoryRegionKey {
    /// Construct a new [MappedMemoryRegionKey] from a slice of bytes, usually received
    /// from a peer in the network
    pub unsafe fn from_raw<EQ: ?Sized + SyncSend + 'static>(
        raw: &[u8],
        domain: &crate::domain::DomainBase<EQ>,
    ) -> Result<Self, Error> {
        OwnedMemoryRegionKey::from_bytes(raw, domain)
            .into_mapped(domain)
            .map_err(|err| crate::error::Error::from_err_code(err.c_err))
    }

    pub(crate) fn key(&self) -> u64 {
        match *self.inner {
            MappedMemoryRegionKeyImpl::Key(key)
            | MappedMemoryRegionKeyImpl::MappedRawKey((key, _)) => key,
        }
    }
}

impl Drop for MappedMemoryRegionKeyImpl {
    fn drop(&mut self) {
        match self {
            MappedMemoryRegionKeyImpl::Key(_) => {}
            MappedMemoryRegionKeyImpl::MappedRawKey((key, ref domain_impl)) => {
                domain_impl.unmap_key(*key).unwrap();
            }
        }
    }
}

//================== Memory Region (fi_mr) ==================//

pub(crate) struct MemoryRegionImpl {
    pub(crate) c_mr: OwnedMrFid,
    pub(crate) key: Result<OwnedMemoryRegionKey, crate::error::Error>,
    pub(crate) mr_desc: OwnedMemoryRegionDesc,
    pub(crate) _domain_rc: MyRc<dyn DomainImplT>,
    pub(crate) bound_cntr: MyOnceCell<MyRc<dyn ReadCntr>>,
    pub(crate) bound_ep: MyOnceCell<MyRc<dyn ActiveEndpoint>>,
}

/// Owned wrapper around a libfabric `fid_mr`.
///
/// This type wraps an instance of a `fid_mr`, monitoring its lifetime and closing it when it goes out of scope.
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.22.0/man/fi_mr.3.html).
///
/// Note that other objects that rely on a MemoryRegion (e.g., [`MemoryRegionKey`]) will extend its lifetime until they
/// are also dropped.
pub struct MemoryRegion {
    pub(crate) inner: MyRc<MemoryRegionImpl>,
}

pub(crate) fn mr_key(
    c_mr: *mut libfabric_sys::fid_mr,
    domain: &impl DomainImplT,
) -> Result<OwnedMemoryRegionKey, crate::error::Error> {
    if domain.mr_mode().is_raw() {
        raw_key(c_mr, 0, domain)
    } else {
        let ret = unsafe { libfabric_sys::inlined_fi_mr_key(c_mr) };
        if ret == crate::FI_KEY_NOTAVAIL {
            Err(crate::error::Error::from_err_code(libfabric_sys::FI_ENOKEY))
        } else {
            Ok(OwnedMemoryRegionKey::Key(ret))
        }
    }
}

fn raw_key(
    c_mr: *mut libfabric_sys::fid_mr,
    flags: u64,
    domain: &impl DomainImplT,
) -> Result<OwnedMemoryRegionKey, crate::error::Error> {
    let mut base_addr = 0u64;
    let mut key_size = domain.mr_key_size();
    let mut raw_key = vec![0u8; key_size + std::mem::size_of::<u64>()];
    let err = unsafe {
        libfabric_sys::inlined_fi_mr_raw_attr(
            c_mr,
            &mut base_addr,
            raw_key.as_mut_ptr().cast(),
            &mut key_size,
            flags,
        )
    };

    if err != 0 {
        Err(crate::error::Error::from_err_code(
            (-err).try_into().unwrap(),
        ))
    } else {
        let raw_key_len = raw_key.len();
        raw_key[raw_key_len - std::mem::size_of::<u64>()..].copy_from_slice(unsafe {
            std::slice::from_raw_parts(&base_addr as *const u64 as *const u8, 8)
        });
        Ok(unsafe { OwnedMemoryRegionKey::from_bytes_impl(&raw_key, domain) })
    }
}

impl MemoryRegionImpl {
    #[allow(dead_code)]
    fn from_buffer<T, EQ: 'static + SyncSend>(
        domain: &MyRc<crate::domain::DomainImplBase<EQ>>,
        buf: &[T],
        access: &MrAccess,
        requested_key: u64,
        flags: MrRegOpt,
        context: *mut std::ffi::c_void,
    ) -> Result<Self, crate::error::Error> {
        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let err = unsafe {
            libfabric_sys::inlined_fi_mr_reg(
                domain.as_typed_fid_mut().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                std::mem::size_of_val(buf),
                access.as_raw().into(),
                0,
                requested_key,
                flags.as_raw(),
                &mut c_mr,
                context,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            let c_desc = unsafe { libfabric_sys::inlined_fi_mr_desc(c_mr) };
            Ok(Self {
                #[cfg(not(feature = "threading-domain"))]
                c_mr: OwnedMrFid::from(c_mr),
                #[cfg(feature = "threading-domain")]
                c_mr: OwnedMrFid::from(c_mr, domain.c_domain.domain.clone()),
                _domain_rc: domain.clone(),
                bound_cntr: MyOnceCell::new(),
                bound_ep: MyOnceCell::new(),
                mr_desc: OwnedMemoryRegionDesc { c_desc },
                key: mr_key(c_mr, domain.as_ref()),
            })
        }
    }

    pub(crate) fn from_attr<EQ: ?Sized + 'static + SyncSend>(
        domain: &MyRc<crate::domain::DomainImplBase<EQ>>,
        attr: MemoryRegionAttr,
        flags: MrRegOpt,
    ) -> Result<Self, crate::error::Error> {
        // [TODO] Add context version
        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let c_mr_ptr: *mut *mut libfabric_sys::fid_mr = &mut c_mr;
        let err = unsafe {
            libfabric_sys::inlined_fi_mr_regattr(
                domain.as_typed_fid_mut().as_raw_typed_fid(),
                attr.get(),
                flags.as_raw(),
                c_mr_ptr,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            let c_desc = unsafe { libfabric_sys::inlined_fi_mr_desc(c_mr) };

            Ok(Self {
                #[cfg(feature = "threading-domain")]
                c_mr: OwnedMrFid::from(c_mr, domain.c_domain.domain.clone()),
                #[cfg(not(feature = "threading-domain"))]
                c_mr: OwnedMrFid::from(c_mr),
                _domain_rc: domain.clone(),
                bound_cntr: MyOnceCell::new(),
                bound_ep: MyOnceCell::new(),
                mr_desc: OwnedMemoryRegionDesc { c_desc },
                key: mr_key(c_mr, domain.as_ref()),
            })
        }
    }

    #[allow(dead_code)]
    fn from_iovec<EQ: 'static + SyncSend>(
        domain: &MyRc<crate::domain::DomainImplBase<EQ>>,
        iov: &[crate::iovec::IoVec],
        access: &MrAccess,
        requested_key: u64,
        flags: MrRegOpt,
        context: *mut std::ffi::c_void,
    ) -> Result<Self, crate::error::Error> {
        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let c_mr_ptr: *mut *mut libfabric_sys::fid_mr = &mut c_mr;
        let err = unsafe {
            libfabric_sys::inlined_fi_mr_regv(
                domain.as_typed_fid_mut().as_raw_typed_fid(),
                iov.as_ptr().cast(),
                iov.len(),
                access.as_raw().into(),
                0,
                requested_key,
                flags.as_raw(),
                c_mr_ptr,
                context,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            let c_desc = unsafe { libfabric_sys::inlined_fi_mr_desc(c_mr) };
            Ok(Self {
                #[cfg(feature = "threading-domain")]
                c_mr: OwnedMrFid::from(c_mr, domain.c_domain.domain.clone()),
                #[cfg(not(feature = "threading-domain"))]
                c_mr: OwnedMrFid::from(c_mr),
                _domain_rc: domain.clone(),
                bound_cntr: MyOnceCell::new(),
                bound_ep: MyOnceCell::new(),
                mr_desc: OwnedMemoryRegionDesc { c_desc },
                key: mr_key(c_mr, domain.as_ref()),
            })
        }
    }

    pub(crate) fn key(&self) -> Result<MemoryRegionKey, crate::error::Error> {
        let key = self.key.as_ref();
        match key {
            Ok(key) => Ok(key.as_borrowed()),
            Err(err) => Err(crate::error::Error::from_err_code(err.c_err)),
        }
    }

    pub(crate) fn bind_cntr(
        &self,
        cntr: &MyRc<impl ReadCntr + 'static>,
        remote_write_event: bool,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_mr_bind(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                cntr.as_typed_fid().as_raw_fid(),
                if remote_write_event {
                    libfabric_sys::FI_REMOTE_WRITE as u64
                } else {
                    0
                },
            )
        };
        if err != 0 && self.bound_cntr.set(cntr.clone()).is_err() {
            panic!("Memory Region already bound to an Endpoint");
        }
        check_error(err.try_into().unwrap())
    }
}

impl MemoryRegionImpl {
    #[allow(dead_code)]
    pub(crate) fn bind_ep<EP: ActiveEndpoint + 'static>(
        &self,
        ep: &MyRc<EP>,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_mr_bind(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                ep.as_typed_fid().as_raw_fid(),
                0,
            )
        };
        if err != 0 && self.bound_ep.set(ep.clone()).is_err() {
            panic!("Memory Region already bound to an Endpoint");
        }
        check_error(err.try_into().unwrap())
    }
}

impl MemoryRegionImpl {
    pub(crate) fn refresh(
        &self,
        iov: &[crate::iovec::IoVec],
        flags: u64,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_mr_refresh(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                iov.as_ptr().cast(),
                iov.len(),
                flags,
            )
        };

        check_error(err.try_into().unwrap())
    }

    pub(crate) fn enable(&self) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_mr_enable(self.as_typed_fid_mut().as_raw_typed_fid())
        };

        check_error(err.try_into().unwrap())
    }

    pub(crate) fn address(&self, flags: u64) -> Result<u64, crate::error::Error> {
        let mut base_addr = 0u64;
        let mut key_size = 0usize;
        let err = unsafe {
            libfabric_sys::inlined_fi_mr_raw_attr(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                &mut base_addr,
                std::ptr::null_mut(),
                &mut key_size,
                flags,
            )
        };

        if err != 0 && -err as u32 != libfabric_sys::FI_ETOOSMALL {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(base_addr)
        }
    }

    // pub(crate) fn raw_attr(&self, base_addr: &mut u64, key_size: &mut usize, flags: u64) -> Result<(), crate::error::Error> { //[TODO] Return the key as it should be returned
    //     let err = unsafe { libfabric_sys::inlined_fi_mr_raw_attr(self.as_raw_typed_fid(), base_addr, std::ptr::null_mut(), key_size, flags) };

    //     if err != 0 {
    //         Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
    //     }
    //     else {
    //         Ok(())
    //     }
    // }

    // pub(crate) fn raw_attr_with_key(&self, base_addr: &mut u64, raw_key: &mut u8, key_size: &mut usize, flags: u64) -> Result<(), crate::error::Error> {
    //     let err = unsafe { libfabric_sys::inlined_fi_mr_raw_attr(self.as_raw_typed_fid(), base_addr, raw_key, key_size, flags) };

    //     if err != 0 {
    //         Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
    //     }
    //     else {
    //         Ok(())
    //     }
    // }

    pub(crate) fn descriptor(&self) -> MemoryRegionDesc {
        self.mr_desc.as_borrowed()
    }
}

impl MemoryRegion {
    #[allow(dead_code)]
    pub(crate) fn from_impl(mr_impl: &MyRc<MemoryRegionImpl>) -> Self {
        MemoryRegion {
            inner: mr_impl.clone(),
        }
    }

    #[allow(dead_code)]
    fn from_buffer<T, EQ: 'static + SyncSend>(
        domain: &crate::domain::DomainBase<EQ>,
        buf: &[T],
        access: &MrAccess,
        requested_key: u64,
        flags: MrRegOpt,
        context: Option<&mut Context>,
    ) -> Result<Self, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(Self {
            inner: MyRc::new(MemoryRegionImpl::from_buffer(
                &domain.inner,
                buf,
                access,
                requested_key,
                flags,
                c_void,
            )?),
        })
    }

    pub(crate) fn from_attr<EQ: ?Sized + 'static + SyncSend>(
        domain: &crate::domain::DomainBase<EQ>,
        attr: MemoryRegionAttr,
        flags: MrRegOpt,
    ) -> Result<Self, crate::error::Error> {
        // [TODO] Add context version
        Ok(Self {
            inner: MyRc::new(MemoryRegionImpl::from_attr(&domain.inner, attr, flags)?),
        })
    }

    #[allow(dead_code)]
    fn from_iovec<EQ: 'static + SyncSend>(
        domain: &crate::domain::DomainBase<EQ>,
        iov: &[crate::iovec::IoVec],
        access: &MrAccess,
        requested_key: u64,
        flags: MrRegOpt,
        context: Option<&mut Context>,
    ) -> Result<Self, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(Self {
            inner: MyRc::new(MemoryRegionImpl::from_iovec(
                &domain.inner,
                iov,
                access,
                requested_key,
                flags,
                c_void,
            )?),
        })
    }

    /// Returns the remote key needed to access a registered memory region.
    ///
    /// This call will automatically request a 'raw' key if the provider requires it.
    ///
    /// Corresponds to calling `fi_mr_raw_attr` or `fi_mr_key` depending on the requirements of the respective [Domain](crate::domain::Domain).
    pub fn key(&self) -> Result<MemoryRegionKey, crate::error::Error> {
        self.inner.key()
    }

    /// Associates the memory region with a counter
    ///
    /// Bind the memory region to `cntr` and request event generation for remote writes or atomics targeting this memory region.
    ///
    /// Corresponds to `fi_mr_bind` with a `fid_cntr`
    pub fn bind_cntr(
        &self,
        cntr: &crate::cntr::Counter<impl ReadCntr + 'static>,
        remote_write_event: bool,
    ) -> Result<(), crate::error::Error> {
        self.inner.bind_cntr(&cntr.inner, remote_write_event)
    }

    /// Notify the provider of any change to the physical pages backing a registered memory region.
    ///
    /// Corresponds to `fi_mr_refresh`
    pub fn refresh(&self, iov: &[crate::iovec::IoVec]) -> Result<(), crate::error::Error> {
        //[TODO]
        self.inner.refresh(iov, 0)
    }

    /// Retrieves the address of memory backing this memory region
    ///
    /// Corresponds to `fi_mr_raw_attr`
    pub fn address(&self) -> Result<u64, crate::error::Error> {
        self.inner.address(0)
    }

    /// Return a local descriptor associated with a registered memory region.
    ///
    /// Corresponds to `fi_mr_desc`
    pub fn descriptor(&self) -> MemoryRegionDesc<'_> {
        self.inner.descriptor()
    }
}

/// An opaque wrapper for the descriptor of a [MemoryRegion] as obtained from
/// `fi_mr_desc`.
#[repr(C)]
#[derive(Debug)]
pub(crate) struct OwnedMemoryRegionDesc {
    c_desc: *mut std::ffi::c_void,
}

impl OwnedMemoryRegionDesc {
    fn as_borrowed<'a>(&'a self) -> MemoryRegionDesc<'a> {
        MemoryRegionDesc {
            c_desc: self.c_desc,
            phantom: PhantomData,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
/// Represents a descriptor for a memory region.
///
/// Its lifetime is bound to a [MemoryRegion].
pub struct MemoryRegionDesc<'a> {
    c_desc: *mut std::ffi::c_void,
    phantom: PhantomData<&'a ()>,
}

impl MemoryRegionDesc<'_> {
    pub(crate) fn as_raw(&self) -> *mut std::ffi::c_void {
        self.c_desc
    }
}

unsafe impl Send for OwnedMemoryRegionDesc {}
// #[cfg(feature="threading-thread-safe")]
// TODO
#[cfg(feature = "thread-safe")]
unsafe impl Sync for OwnedMemoryRegionDesc {}

unsafe impl Send for MemoryRegionDesc<'_> {}
// #[cfg(feature="threading-thread-safe")]
// TODO
#[cfg(feature = "thread-safe")]
unsafe impl Sync for MemoryRegionDesc<'_> {}

impl OwnedMemoryRegionDesc {
    pub(crate) fn from_raw(c_desc: *mut std::ffi::c_void) -> Self {
        Self { c_desc }
    }
}

impl AsTypedFid<MrRawFid> for MemoryRegion {
    fn as_typed_fid(&self) -> BorrowedTypedFid<MrRawFid> {
        self.inner.as_typed_fid()
    }

    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<MrRawFid> {
        self.inner.as_typed_fid_mut()
    }
}

impl AsTypedFid<MrRawFid> for MemoryRegionImpl {
    fn as_typed_fid(&self) -> BorrowedTypedFid<MrRawFid> {
        self.c_mr.as_typed_fid()
    }

    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<MrRawFid> {
        self.c_mr.as_typed_fid_mut()
    }
}

// impl AsRawTypedFid for MemoryRegion {
//     type Output = MrRawFid;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.inner.as_raw_typed_fid()
//     }
// }

// impl AsRawTypedFid for MemoryRegionImpl {
//     type Output = MrRawFid;

//     fn as_raw_typed_fid(&self) -> Self::Output {
//         self.c_mr.as_raw_typed_fid()
//     }
// }

//================== Memory Region attribute ==================//

pub struct MemoryRegionAttr {
    pub(crate) c_attr: libfabric_sys::fi_mr_attr,
    // phantom: PhantomData<&'a ()>,
}

impl MemoryRegionAttr {
    pub fn new() -> Self {
        Self {
            c_attr: libfabric_sys::fi_mr_attr {
                __bindgen_anon_1: libfabric_sys::fi_mr_attr__bindgen_ty_1 { dmabuf: null() },
                iov_count: 0,
                access: 0,
                offset: 0,
                requested_key: 0,
                context: std::ptr::null_mut(),
                auth_key_size: 0,
                auth_key: std::ptr::null_mut(),
                iface: libfabric_sys::fi_hmem_iface_FI_HMEM_SYSTEM,
                device: libfabric_sys::fi_mr_attr__bindgen_ty_2 { reserved: 0 },
                hmem_data: std::ptr::null_mut(),
                page_size: 0,
            },
        }
    }

    pub fn iov(&mut self, iov: &[crate::iovec::IoVec]) -> &mut Self {
        self.c_attr.__bindgen_anon_1.mr_iov = iov.as_ptr().cast();
        self.c_attr.iov_count = iov.len();

        self
    }

    pub fn dmabuf(&mut self, dmabuf: &DmaBuf) -> &mut Self {
        self.c_attr.__bindgen_anon_1.dmabuf = &dmabuf.c_dmabuf;

        self
    }

    pub fn access(&mut self, access: &MrAccess) -> &mut Self {
        self.c_attr.access = access.as_raw() as u64;
        self
    }

    pub fn access_collective(&mut self) -> &mut Self {
        self.c_attr.access |= libfabric_sys::FI_COLLECTIVE as u64;
        self
    }

    pub fn access_send(&mut self) -> &mut Self {
        self.c_attr.access |= libfabric_sys::FI_SEND as u64;
        self
    }

    pub fn access_recv(&mut self) -> &mut Self {
        self.c_attr.access |= libfabric_sys::FI_RECV as u64;
        self
    }

    pub fn access_read(&mut self) -> &mut Self {
        self.c_attr.access |= libfabric_sys::FI_READ as u64;
        self
    }

    pub fn access_write(&mut self) -> &mut Self {
        self.c_attr.access |= libfabric_sys::FI_WRITE as u64;
        self
    }

    pub fn access_remote_write(&mut self) -> &mut Self {
        self.c_attr.access |= libfabric_sys::FI_REMOTE_WRITE as u64;
        self
    }

    pub fn access_remote_read(&mut self) -> &mut Self {
        self.c_attr.access |= libfabric_sys::FI_REMOTE_READ as u64;
        self
    }

    pub fn offset(&mut self, offset: u64) -> &mut Self {
        self.c_attr.offset = offset;
        self
    }

    pub fn context<T0>(&mut self, ctx: &mut T0) -> &mut Self {
        self.c_attr.context = (ctx as *mut T0).cast();
        self
    }

    pub fn requested_key(&mut self, key: u64) -> &mut Self {
        self.c_attr.requested_key = key;
        self
    }

    pub fn auth_key(&mut self, key: &mut [u8]) -> &mut Self {
        self.c_attr.auth_key_size = key.len();
        self.c_attr.auth_key = key.as_mut_ptr();
        self
    }

    pub fn iface(&mut self, iface: crate::enums::HmemIface) -> &mut Self {
        self.c_attr.iface = iface.as_raw();
        self.c_attr.device = match iface {
            crate::enums::HmemIface::Ze(drv_idx, dev_idx) => {
                let ze_id = unsafe { libfabric_sys::inlined_fi_hmem_ze_device(drv_idx, dev_idx) };
                libfabric_sys::fi_mr_attr__bindgen_ty_2 { ze: ze_id }
            }
            crate::enums::HmemIface::System => {
                libfabric_sys::fi_mr_attr__bindgen_ty_2 { reserved: 0 }
            }
            crate::enums::HmemIface::Cuda(id) => {
                libfabric_sys::fi_mr_attr__bindgen_ty_2 { cuda: id }
            }
            crate::enums::HmemIface::Rocr(id) => {
                libfabric_sys::fi_mr_attr__bindgen_ty_2 { cuda: id }
            }
            crate::enums::HmemIface::Neuron(id) => {
                libfabric_sys::fi_mr_attr__bindgen_ty_2 { neuron: id }
            }
            crate::enums::HmemIface::SynapseAi(id) => {
                libfabric_sys::fi_mr_attr__bindgen_ty_2 { synapseai: id }
            }
        };
        self
    }

    pub(crate) fn get(&self) -> *const libfabric_sys::fi_mr_attr {
        &self.c_attr
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_mr_attr {
        &mut self.c_attr
    }
}

impl Default for MemoryRegionAttr {
    fn default() -> Self {
        Self::new()
    }
}

/// A memory region bound to an [crate::ep::Endpoint]
pub struct EpBindingMemoryRegion {
    mr: MemoryRegion,
}

/// A memory region bound to a [MemoryRegion]
pub struct RmaEventMemoryRegion {
    mr: MemoryRegion,
}

impl EpBindingMemoryRegion {
    /// Associates the memory region with an endpoint
    ///
    /// Bind the memory region to `ep`.
    ///
    /// Corresponds to `fi_mr_bind` with a `fid_ep`
    pub(crate) fn bind_ep<EP: ActiveEndpoint + 'static, STATE: EpState>(
        &self,
        ep: &crate::ep::EndpointBase<EP, STATE>,
    ) -> Result<(), crate::error::Error> {
        self.mr.inner.bind_ep(&ep.inner)
    }

    /// Associates the memory region with a counter
    ///
    /// Bind the memory region to `cntr` and request event generation for remote writes or atomics targeting this memory region.
    ///
    /// Corresponds to `fi_mr_bind` with a `fid_cntr`
    pub fn bind_cntr(
        &self,
        cntr: &crate::cntr::Counter<impl ReadCntr + 'static>,
        remote_write_event: bool,
    ) -> Result<(), crate::error::Error> {
        self.mr.inner.bind_cntr(&cntr.inner, remote_write_event)
    }

    /// Enables a memory region for use.
    ///
    /// Corresponds to `fi_mr_enable`
    pub fn enable<EP: ActiveEndpoint + 'static, STATE: EpState>(self, ep: &crate::ep::EndpointBase<EP, STATE>) -> Result<MemoryRegion, crate::error::Error> {
        self.bind_ep(ep)?;
        self.mr.inner.enable()?;
        Ok(self.mr)
    }
}

impl RmaEventMemoryRegion {
    /// Associates the memory region with a counter
    ///
    /// Bind the memory region to `cntr` and request event generation for remote writes or atomics targeting this memory region.
    ///
    /// Corresponds to `fi_mr_bind` with a `fid_cntr`
    pub fn bind_cntr(
        &self,
        cntr: &crate::cntr::Counter<impl ReadCntr + 'static>,
        remote_write_event: bool,
    ) -> Result<(), crate::error::Error> {
        self.mr.inner.bind_cntr(&cntr.inner, remote_write_event)
    }

    /// Enables a memory region for use.
    ///
    /// Corresponds to `fi_mr_enable`
    pub fn enable(self) -> Result<MemoryRegion, crate::error::Error> {
        self.mr.inner.enable()?;
        Ok(self.mr)
    }
}

pub enum MaybeDisabledMemoryRegion {
    Enabled(MemoryRegion),
    Disabled(DisabledMemoryRegion),
}

/// A disabled memory region that needs to be bound to an [crate::ep::Endpoint] or a [MemoryRegion].
pub enum DisabledMemoryRegion {
    EpBind(EpBindingMemoryRegion),
    RmaEvent(RmaEventMemoryRegion)
}

pub(crate) enum MRBackingBuf<'a> {
    IoVs(Vec<IoVec<'a>>),
    DmaBuf(&'a DmaBuf),
}

/// Builder for the [MemoryRegion] type.
///
/// `MemoryRegionBuilder` is used to configure and build a new [MemoryRegion].
/// It encapsulates an incremental configuration of the address vector set, as provided by a `fi_mr_attr`,
/// followed by a call to `fi_mr_regattr.  
pub struct MemoryRegionBuilder<'a> {
    pub(crate) mr_attr: MemoryRegionAttr,
    pub(crate) backing_buf: MRBackingBuf<'a>,
    pub(crate) flags: MrRegOpt,
}

impl<'a> MemoryRegionBuilder<'a> {
    /// Initiates the creation of new [MemoryRegion] on `domain`, with backing memory `buff`.
    ///
    /// The initial configuration is only setting the fields `fi_mr_attr::mr_iov`, `fi_mr_attr::iface`.
    pub fn new<T>(buff: &'a [T], iface: crate::enums::HmemIface) -> Self {
        let mut mr_attr = MemoryRegionAttr::new();
        mr_attr.iface(iface);
        Self {
            mr_attr,
            flags: MrRegOpt::new(),
            backing_buf: MRBackingBuf::IoVs(vec![IoVec::from_slice(buff)]),
        }
    }

    /// Initiates the creation of new [MemoryRegion] on `domain`, with backing memory `buff`.
    ///
    /// The initial configuration is only setting the fields `fi_mr_attr::mr_iov`, `fi_mr_attr::iface`.
    pub fn new_from_dma_buf<T>(dmabuff: &'a DmaBuf, iface: crate::enums::HmemIface) -> Self {
        let mut mr_attr = MemoryRegionAttr::new();
        mr_attr.iface(iface);
        Self {
            mr_attr,
            flags: MrRegOpt::new(),
            backing_buf: MRBackingBuf::DmaBuf(dmabuff),
        }
    }

    /// Add another backing buffer to the memory region
    ///
    /// Corresponds to 'pushing' another value to the `fi_mr_attr::mr_iov` field.
    pub fn add_buffer<T>(mut self, buff: &'a [T]) -> Self {
        match &mut self.backing_buf {
            MRBackingBuf::IoVs(vec) => vec.push(IoVec::from_slice(buff)),
            MRBackingBuf::DmaBuf(_) => panic!("FI_MR_DMABUF is in use"),
        }

        self
    }

    /// Indicates that the MR may be used for collective operations.
    ///
    /// Corresponds to setting the respective bitflag of the `fi_mr_attr::access` field
    pub fn access_collective(mut self) -> Self {
        //[TODO] Required if the FI_MR_COLLECTIVE mr_mode bit has been set on the domain.
        //[TODO] Should be paired with FI_SEND/FI_RECV
        self.mr_attr.access_collective();
        self
    }

    /// Indicates that the MR may be used for send operations.
    ///
    /// Corresponds to setting the respective bitflag of the `fi_mr_attr::access` field
    pub fn access_send(mut self) -> Self {
        self.mr_attr.access_send();
        self
    }

    /// Indicates that the MR may be used for receive operations.
    ///
    /// Corresponds to setting the respective bitflag of the `fi_mr_attr::access` field
    pub fn access_recv(mut self) -> Self {
        self.mr_attr.access_recv();
        self
    }

    /// Indicates that the MR may be used as buffer to store the results of RMA read operations.
    ///
    /// Corresponds to setting the respective bitflag of the `fi_mr_attr::access` field
    pub fn access_read(mut self) -> Self {
        self.mr_attr.access_read();
        self
    }

    /// Indicates that the memory buffer may be used as the source buffer for RMA write and atomic operations on the initiator side
    ///
    /// Corresponds to setting the respective bitflag of the `fi_mr_attr::access` field
    pub fn access_write(mut self) -> Self {
        self.mr_attr.access_write();
        self
    }

    /// Indicates that the memory buffer may be used as the target buffer of an RMA write or atomic operation.
    ///
    /// Corresponds to setting the respective bitflag of the `fi_mr_attr::access` field
    pub fn access_remote_write(mut self) -> Self {
        self.mr_attr.access_remote_write();
        self
    }

    /// Indicates that the memory buffer may be used as the source buffer of an RMA read operation on the target side
    ///
    /// Corresponds to setting the respective bitflag of the `fi_mr_attr::access` field
    pub fn access_remote_read(mut self) -> Self {
        self.mr_attr.access_remote_read();
        self
    }

    /// Another method to provide the access permissions collectively
    ///
    /// Corresponds to setting the respective bitflags of the `fi_mr_attr::access` field
    pub fn access(mut self, access: &MrAccess) -> Self {
        self.mr_attr.access(access);
        self
    }

    // pub fn offset(mut self, offset: u64) -> Self {
    //     self.mr_attr.offset(offset);
    //     self
    // }

    /// Application context associated with asynchronous memory registration operations.
    ///
    /// Corresponds to setting the `fi_mr_attr::context` field to `ctx`
    pub fn context<T0>(mut self, ctx: &mut T0) -> Self {
        self.mr_attr.context(ctx);
        self
    }

    /// An application specified access key associated with the memory region.
    ///
    /// Corresponds to setting the `fi_mr_attr::requested_key` field
    pub fn requested_key(mut self, key: u64) -> Self {
        self.mr_attr.requested_key(key);
        self
    }

    /// Indicates the key to associate with this memory registration
    ///
    /// Corresponds to setting the fields `fi_mr_attr::auth_key` and `fi_mr_attr::auth_key_size`
    pub fn auth_key(mut self, key: &mut [u8]) -> Self {
        self.mr_attr.auth_key(key);
        self
    }

    /// Indicates that the memory region is only accessible from the device.
    ///
    /// Corresponds to setting the `FI_HMEM_DEVICE_ONLY` flag
    pub fn hmem_device_only(mut self) -> Self {
        self.flags = self.flags.hmem_device_only();
        self
    }

    /// Indicates that the memory region is allocated from host memory.
    ///
    /// Corresponds to setting the `FI_HMEM_HOST_ALLOC` flag
    pub fn hmem_host_alloc(mut self) -> Self {
        self.flags = self.flags.hmem_host_alloc();
        self
    }

    /// Indicates that the memory region is used for RMA events.
    ///
    /// Corresponds to setting the `FI_RMA_EVENT` flag
    pub fn rma_event(mut self) -> Self {
        self.flags = self.flags.rma_event();
        self
    }

    /// Indicates that the memory region is used for persistent memory.
    ///
    /// Corresponds to setting the `FI_RMA_PMEM` flag
    pub fn rma_pmem(mut self) -> Self {
        self.flags = self.flags.rma_pmem();
        self
    }

    /// Constructs a new [MemoryRegion] with the configurations requested so far.
    ///
    /// Corresponds to creating a `fi_mr_attr`, setting its fields to the requested ones,
    /// and passign it to `fi_mr_regattr`.
    pub fn build<EQ: ?Sized + 'static + SyncSend>(
        mut self,
        domain: &'a crate::domain::DomainBase<EQ>,
    ) -> Result<MaybeDisabledMemoryRegion, crate::error::Error> {
        if domain.inner._eq_rc.get().is_some() {
            let (_eq, async_reg) = domain.inner._eq_rc.get().unwrap();
            if *async_reg {
                panic!("Manual async memory registration is not supported. Use the ::async_::mr::MemoryRegionBuilder for that.")
            }
        }
        match &mut self.backing_buf {
            MRBackingBuf::IoVs(vec) => self.mr_attr.iov(vec),
            MRBackingBuf::DmaBuf(dmabuf) => self.mr_attr.dmabuf(dmabuf),
        };

        let mr = MemoryRegion::from_attr(domain, self.mr_attr, self.flags)?;
        
        if domain.mr_mode().is_endpoint() {
            Ok(MaybeDisabledMemoryRegion::Disabled(DisabledMemoryRegion::EpBind(EpBindingMemoryRegion {
                mr,
            })))
        } else {
            Ok(MaybeDisabledMemoryRegion::Enabled(mr))
        }
    }

    // /// Constructs a new [MemoryRegion] with the configurations requested so far.
    // ///
    // /// Corresponds to creating a `fi_mr_attr`, setting its fields to the requested ones,
    // /// and passign it to `fi_mr_regattr`.
    // pub async fn build_async(self) -> Result<(Event<usize>,MemoryRegion), crate::error::Error> {
    //     panic!("Async memory registration is currently not supported due to a potential bug in libfabric");
    //     self.mr_attr.iov(&self.iovs);
    //     MemoryRegion::from_attr_async(self.domain, self.mr_attr, self.flags).await
    // }
}

//=================== Async Stuff =========================//

//================== Memory Region tests ==================//
#[cfg(test)]
mod tests {
    use crate::{enums::MrAccess, info::Info};

    use super::{MaybeDisabledMemoryRegion, MemoryRegionBuilder};

    pub fn ft_alloc_bit_combo(fixed: u64, opt: u64) -> Vec<u64> {
        let bits_set = |mut val: u64| -> u64 {
            let mut cnt = 0;
            while val > 0 {
                cnt += 1;
                val &= val - 1;
            }
            cnt
        };
        let num_flags = bits_set(opt) + 1;
        let len = 1 << (num_flags - 1);
        let mut flags = vec![0_u64; num_flags as usize];
        let mut num_flags = 0;
        for i in 0..8 * std::mem::size_of::<u64>() {
            if opt >> i & 1 == 1 {
                flags[num_flags] = 1 << i;
                num_flags += 1;
            }
        }
        let mut combos = Vec::new();

        for index in 0..len {
            combos.push(fixed);
            for (i, val) in flags
                .iter()
                .enumerate()
                .take(8 * std::mem::size_of::<u64>())
            {
                if index >> i & 1 == 1 {
                    combos[index] |= val;
                }
            }
        }

        combos
    }
    pub struct TestSizeParam(pub u64);
    pub const DEF_TEST_SIZES: [TestSizeParam; 6] = [
        TestSizeParam(1 << 0),
        TestSizeParam(1 << 1),
        TestSizeParam(1 << 2),
        TestSizeParam(1 << 3),
        TestSizeParam(1 << 4),
        TestSizeParam(1 << 5),
    ];

    #[test]
    fn mr_reg() {
        // let ep_attr = crate::ep::EndpointAttr::new();
        // let mut dom_attr = crate::domain::DomainAttr::new();
        // dom_attr.mode = crate::enums::Mode::all();
        // dom_attr.mr_mode = crate::enums::MrMode::new().basic().scalable().local().inverse();

        // let hints = InfoHints::new()
        //     .caps(crate::infocapsoptions::InfoCaps::new().msg().rma())
        //     .ep_attr(ep_attr)
        //     .domain_attr(dom_attr);

        let info = Info::new(&crate::info::libfabric_version())
            .enter_hints()
            .caps(crate::infocapsoptions::InfoCaps::new().msg().rma())
            .enter_domain_attr()
            .mode(crate::enums::Mode::all())
            .mr_mode(
                crate::enums::MrMode::new()
                    .basic()
                    .scalable()
                    .local()
                    .inverse(),
            )
            .leave_domain_attr()
            .leave_hints()
            .get()
            .unwrap();

        let entry = info.into_iter().next();

        if let Some(entry) = entry {
            let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
            let domain = crate::domain::DomainBuilder::new(&fab, &entry)
                .build()
                .unwrap();

            let mut mr_access: u64 = 0;
            if entry.mode().is_local_mr() || entry.domain_attr().mr_mode().is_local() {
                if entry.caps().is_msg() || entry.caps().is_tagged() {
                    let mut on = false;
                    if entry.caps().is_send() {
                        mr_access |= libfabric_sys::FI_SEND as u64;
                        on = true;
                    }
                    if entry.caps().is_recv() {
                        mr_access |= libfabric_sys::FI_RECV as u64;
                        on = true;
                    }
                    if !on {
                        mr_access |= libfabric_sys::FI_SEND as u64 & libfabric_sys::FI_RECV as u64;
                    }
                }
            } else if entry.caps().is_rma() || entry.caps().is_atomic() {
                if entry.caps().is_remote_read()
                    || !(entry.caps().is_read()
                        || entry.caps().is_write()
                        || entry.caps().is_remote_write())
                {
                    mr_access |= libfabric_sys::FI_REMOTE_READ as u64;
                } else {
                    mr_access |= libfabric_sys::FI_REMOTE_WRITE as u64;
                }
            }

            let combos = ft_alloc_bit_combo(0, mr_access);

            for test in &DEF_TEST_SIZES {
                let buff_size = test.0;
                let buf = vec![0_u64; buff_size as usize];
                for combo in &combos {
                    let mr = MemoryRegionBuilder::new(&buf, crate::enums::HmemIface::System)
                        // .iov(std::slice::from_mut(&mut IoVec::from_slice_mut(&mut buf)))
                        .access(&MrAccess::from_raw(*combo as u32))
                        .requested_key(0xC0DE)
                        .build(&domain)
                        .unwrap();
                    let mr = match mr {
                        MaybeDisabledMemoryRegion::Enabled(mr) => mr,
                        MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                            super::DisabledMemoryRegion::EpBind(_disabled_mr) => todo!(), //disabled_mr.enable(ep),
                            super::DisabledMemoryRegion::RmaEvent(disabled_mr) => disabled_mr.enable().unwrap(),
                        }
                    };
                    let _desc = mr.descriptor();
                    // mr.close().unwrap();
                }
            }

            // domain.close().unwrap();
            // fab.close().unwrap();
        } else {
            panic!("No capable fabric found!");
        }
    }

    pub struct MemoryRegionSlice<'a, DATA: Copy> {
        pub slice: &'a mut [DATA],
    }

    impl<'a, DATA: Copy> MemoryRegionSlice<'a, DATA> {
        pub fn split_at(
            &'a mut self,
            mid: usize,
        ) -> (MemoryRegionSlice<'a, DATA>, MemoryRegionSlice<'a, DATA>) {
            let (s0, s1) = self.slice.split_at_mut(mid);
            (
                MemoryRegionSlice::<'a, DATA> { slice: s0 },
                MemoryRegionSlice::<'a, DATA> { slice: s1 },
            )
        }
    }

    impl<'a, DATA: Copy> MemoryRegionSlice<'a, DATA> {
        pub fn slice<Idx>(&'a mut self, index: Idx) -> MemoryRegionSlice<'a, DATA>
        where
            Idx: std::slice::SliceIndex<[DATA], Output = [DATA]>,
        {
            MemoryRegionSlice::<'a, DATA> {
                slice: &mut self.slice[index],
            }
        }
    }

    #[test]
    fn try_wrapper() {
        let mut vec = vec![0u8; 10];
        let mut wrapped_vec = MemoryRegionSlice {
            slice: &mut vec[..],
        };
        wrapped_vec.slice(0..4);
        // let (w0, w1) = wrapped_vec.split_at(5);
        // vec[0] = 1;
        // let new_wrapped = wrapped_vec.slice();
    }
}

#[cfg(test)]
mod libfabric_lifetime_tests {
    use crate::{enums::MrAccess, info::Info};

    use super::MemoryRegionBuilder;

    #[test]
    fn mr_drops_before_domain() {
        // let ep_attr = crate::ep::EndpointAttr::new();
        // let mut dom_attr = crate::domain::DomainAttr::new();
        //     dom_attr.mode = crate::enums::Mode::all();
        //     dom_attr.mr_mode = crate::enums::MrMode::new().basic().scalable().local().inverse();

        // let hints = InfoHints::new()
        //     .caps(crate::infocapsoptions::InfoCaps::new().msg().rma())
        //     .ep_attr(ep_attr)
        //     .domain_attr(dom_attr);

        let info = Info::new(&crate::info::libfabric_version())
            .enter_hints()
            .caps(crate::infocapsoptions::InfoCaps::new().msg().rma())
            .enter_domain_attr()
            .mode(crate::enums::Mode::all())
            .mr_mode(
                crate::enums::MrMode::new()
                    .basic()
                    .scalable()
                    .local()
                    .inverse(),
            )
            .leave_domain_attr()
            .leave_hints()
            .get()
            .unwrap();

        let entry = info.into_iter().next();

        if let Some(entry) = entry {
            let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
            let domain = crate::domain::DomainBuilder::new(&fab, &entry)
                .build()
                .unwrap();

            let mut mr_access: u64 = 0;

            if entry.mode().is_local_mr() || entry.domain_attr().mr_mode().is_local() {
                if entry.caps().is_msg() || entry.caps().is_tagged() {
                    let mut on = false;
                    if entry.caps().is_send() {
                        mr_access |= libfabric_sys::FI_SEND as u64;
                        on = true;
                    }
                    if entry.caps().is_recv() {
                        mr_access |= libfabric_sys::FI_RECV as u64;
                        on = true;
                    }
                    if !on {
                        mr_access |= libfabric_sys::FI_SEND as u64 & libfabric_sys::FI_RECV as u64;
                    }
                }
            } else if entry.caps().is_rma() || entry.caps().is_atomic() {
                if entry.caps().is_remote_read()
                    || !(entry.caps().is_read()
                        || entry.caps().is_write()
                        || entry.caps().is_remote_write())
                {
                    mr_access |= libfabric_sys::FI_REMOTE_READ as u64;
                } else {
                    mr_access |= libfabric_sys::FI_REMOTE_WRITE as u64;
                }
            }

            let combos = super::tests::ft_alloc_bit_combo(0, mr_access);

            let mut mrs = Vec::new();
            for test in &super::tests::DEF_TEST_SIZES {
                let buff_size = test.0;
                let buf = vec![0_u64; buff_size as usize];
                for combo in &combos {
                    let mr = MemoryRegionBuilder::new(&buf, crate::enums::HmemIface::System)
                        .access(&MrAccess::from_raw(*combo as u32))
                        .requested_key(0xC0DE)
                        .build(&domain)
                        .unwrap();
                    mrs.push(mr);
                }
            }
            drop(domain);
        } else {
            panic!("No capable fabric found!");
        }
    }
}
