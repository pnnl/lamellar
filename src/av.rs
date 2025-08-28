use std::ffi::CString;
use std::marker::PhantomData;

use crate::eq::{AVCompleteEvent, EventQueueBase};
use crate::fid::AsTypedFid;
use crate::utils::check_error;

#[allow(unused_imports)]
// use crate::fid::AsFid;
use crate::{
    domain::{DomainBase, DomainImplT},
    enums::{AVOptions, AVSetOptions, AddressVectorType},
    ep::Address,
    eq::ReadEq,
    fid::{
        self, AVSetRawFid, AsRawFid, AsRawTypedFid, AvRawFid, OwnedAVFid, OwnedAVSetFid, RawFid,
    },
    AddressSource, Context, MappedAddress, MyOnceCell, MyRc, RawMappedAddress, SyncSend,
    FI_ADDR_NOTAVAIL,
};

//================== AddressVector Public ==================//

/// Owned wrapper around a blocking libfabric `fid_av`.
///
/// This type wraps an instance of a `fid_av`, monitoring its lifetime and closing it when it goes out of scope.
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.22.0/man/fi_av.3.html).
///
/// Note that other objects that rely on an AddressVector (e.g., [MappedAddress]) will extend its lifetime until they
/// are also dropped.
pub type AddressVector = AddressVectorBase<Block, dyn ReadEq>;

/// Owned wrapper around a non-blocking libfabric `fid_av`.
pub type NoBlockAddressVector = AddressVectorBase<NoBlock, dyn ReadEq>;

pub struct Block {}
pub struct NoBlock {}

#[repr(C)]
/// Represents an authorization key as the one returned by the `fi_av_lookup_auth_key` function.
pub struct AuthKey {
    pub(crate) auth_key: Vec<u8>,
}
/// A trait that marks whether an AddressVector is blocking or non-blocking.
pub trait AVSyncMode {}

impl AVSyncMode for Block {}
impl AVSyncMode for NoBlock {}

/// Enum with the different ways to provide an address to be inserted into an AddressVector.
pub enum AvInAddress<'a> {
    String(&'a str),
    Encoded(&'a [Address]),
    Service((&'a str, &'a str)),
    Symmetric((&'a str, usize, &'a str, usize)),
}

impl<'a> From<&'a [Address]> for AvInAddress<'a> {
    fn from(value: &'a [Address]) -> Self {
        AvInAddress::Encoded(value)
    }
}

impl<'a> From<&'a str> for AvInAddress<'a> {
    fn from(value: &'a str) -> Self {
        AvInAddress::String(value)
    }
}

impl<'a> From<(&'a str, &'a str)> for AvInAddress<'a> {
    fn from(value: (&'a str, &'a str)) -> Self {
        AvInAddress::Service(value)
    }
}

impl<'a> From<(&'a str, usize, &'a str, usize)> for AvInAddress<'a> {
    fn from(value: (&'a str, usize, &'a str, usize)) -> Self {
        AvInAddress::Symmetric(value)
    }
}

pub struct AddressVectorBase<MODE, EQ: ?Sized + ReadEq> {
    pub(crate) inner: MyRc<AddressVectorImplBase<EQ>>,
    phantom: PhantomData<MODE>,
}

impl<Mode: AVSyncMode, EQ: ReadEq + ?Sized + 'static> AddressVectorBase<Mode, EQ> {
    #[allow(dead_code)]
    pub(crate) fn from_impl(av_impl: &MyRc<AddressVectorImplBase<EQ>>) -> Self {
        Self {
            inner: av_impl.clone(),
            phantom: PhantomData,
        }
    }

    pub(crate) fn new<DEQ: ?Sized + 'static + SyncSend>(
        domain: &crate::domain::DomainBase<DEQ>,
        attr: AddressVectorAttr,
        context: Option<&mut Context>,
    ) -> Result<Self, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(Self {
            inner: MyRc::new(AddressVectorImplBase::new(&domain.inner, attr, c_void)?),
            phantom: PhantomData,
        })
    }

    /// Removes the given [MappedAddress]es from the AddressVector.
    ///
    /// This method will consume the mapped addresses passed to it to prevent their reuse.
    ///
    /// Directly corresponds to `fi_av_remove`
    pub fn remove(&self, addr: Vec<crate::MappedAddress>) -> Result<(), crate::error::Error> {
        self.inner.remove(addr)
    }

    /// Retrieves an address stored in the address vector.
    ///
    /// Directly corresponds to `fi_av_lookup`
    pub fn lookup(
        &self,
        mapped_addr: crate::MappedAddress,
    ) -> Result<Address, crate::error::Error> {
        self.inner.lookup(mapped_addr)
    }

    /// Convert an [Address] into a printable string.
    ///
    /// Directly corresponds to `fi_av_straddr`
    pub fn straddr(&self, addr: &Address) -> String {
        self.inner.straddr(addr)
    }

    /// Inserts an authorization key into the AddressVector.
    ///
    /// Directly corresponds to `fi_av_insert_auth_key`
    pub fn insert_auth_key(&self, auth_key: &AuthKey) -> Result<(), crate::error::Error> {
        self.inner.insert_auth_key(auth_key)
    }

    /// Looks up the authorization key associated with a mapped address.
    ///
    /// Directly corresponds to `fi_av_lookup_auth_key`
    pub fn lookup_auth_key(
        &self,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<AuthKey, crate::error::Error> {
        self.inner.lookup_auth_key(mapped_addr)
    }

    /// Sets the user ID associated with a mapped address.
    ///
    /// Directly corresponds to `fi_av_set_user_id`
    pub fn set_user_id(
        &self,
        mapped_addr: &MappedAddress,
        user_id: &MappedAddress,
        flags: crate::enums::UserId,
    ) -> Result<(), crate::error::Error> {
        self.inner.set_user_id(mapped_addr, user_id, flags)
    }
}

impl<EQ: ReadEq + ?Sized + 'static> AddressVectorBase<Block, EQ> {
    /// Insert one or more addresses into the [AddressVector] and return a [Vec] of [MappedAddress]es, one for each input address and wait for the operation to complete.
    /// Addresses can be of types:
    /// - A single string ([AvInAddress::String]) that provides both a node and a service
    /// - A slice of [Address] ([AvInAddress::Encoded])
    /// - A node and a service as two separate strings ([AvInAddress::Service])
    /// - A node followed by a count of increments, a service followed by a count of increments ([AvInAddress::Symmetric])
    ///
    /// The operation can be modified using the requested `options` as defined in [AVOptions].
    /// For address(es) that could not be mapped a [None] value will be returned at the respective index.
    ///
    /// This method corresponds to a call to:
    /// - `fi_av_insert` if `addr` == [AvInAddress::Encoded]
    /// - `fi_av_insertsvc` if `addr` == [AvInAddress::String] or [AvInAddress::Service]
    /// - `fi_av_insertsym` if `addr` == [AvInAddress::Symmetric]
    pub fn insert(
        &self,
        addr: AvInAddress,
        options: AVOptions,
    ) -> Result<Vec<Option<MappedAddress>>, crate::error::Error> {
        let fi_addresses = match addr {
            AvInAddress::String(str_addr) => {
                self.inner.insertsvc_str(str_addr, options.as_raw(), None)?
            }
            AvInAddress::Encoded(addresses) => {
                self.inner.insert::<()>(addresses, options.as_raw(), None)?
            }
            AvInAddress::Service((node, svc)) => {
                self.inner
                    .insertsvc::<()>(node, svc, options.as_raw(), None)?
                // vec![mapped_addr]
            }
            AvInAddress::Symmetric((node, nodecnt, svc, svccnt)) => {
                self.inner
                    .insertsym::<()>(node, nodecnt, svc, svccnt, options.as_raw(), None)?
            }
        };

        Ok(fi_addresses
            .into_iter()
            .map(|fi_addr| {
                if fi_addr == FI_ADDR_NOTAVAIL {
                    None
                } else {
                    Some(MappedAddress::from_raw_addr(
                        RawMappedAddress::from_raw(self.inner.type_, fi_addr),
                        AddressSource::Av(self.inner.clone()),
                    ))
                }
            })
            .collect::<Vec<_>>())
    }

    /// Same as [Self::insert] but with an extra argument to provide a context
    ///
    pub fn insert_with_context<T>(
        &self,
        addr: AvInAddress,
        options: AVOptions,
        ctx: &mut Context,
    ) -> Result<Vec<Option<MappedAddress>>, crate::error::Error> {
        let fi_addresses = match addr {
            AvInAddress::String(str_addr) => {
                self.inner
                    .insertsvc_str(str_addr, options.as_raw(), Some(ctx.inner_mut()))?
            }
            AvInAddress::Encoded(addresses) => {
                self.inner.insert(addresses, options.as_raw(), Some(ctx))?
            }
            AvInAddress::Service((node, svc)) => {
                self.inner
                    .insertsvc(node, svc, options.as_raw(), Some(ctx))?
            }
            AvInAddress::Symmetric((node, nodecnt, svc, svccnt)) => {
                self.inner
                    .insertsym(node, nodecnt, svc, svccnt, options.as_raw(), Some(ctx))?
            }
        };

        Ok(fi_addresses
            .into_iter()
            .map(|fi_addr| {
                if fi_addr == FI_ADDR_NOTAVAIL {
                    None
                } else {
                    Some(MappedAddress::from_raw_addr(
                        RawMappedAddress::from_raw(self.inner.type_, fi_addr),
                        AddressSource::Av(self.inner.clone()),
                    ))
                }
            })
            .collect::<Vec<_>>())
    }
}

impl<EQ: ReadEq + ?Sized + 'static> AddressVectorBase<NoBlock, EQ> {
    /// Similar to [Self::insert] but does not wait for the operation to complete and returns a [PendingAVTranslation] instead.
    pub fn insert_no_block(
        &self,
        addr: AvInAddress,
        options: AVOptions,
    ) -> Result<PendingAVTranslation, crate::error::Error> {
        let fi_addresses = match addr {
            AvInAddress::String(str_addr) => {
                self.inner.insertsvc_str(str_addr, options.as_raw(), None)?
            }
            AvInAddress::Encoded(addresses) => {
                self.inner.insert::<()>(addresses, options.as_raw(), None)?
            }
            AvInAddress::Service((node, svc)) => {
                self.inner
                    .insertsvc::<()>(node, svc, options.as_raw(), None)?
                // vec![mapped_addr]
            }
            AvInAddress::Symmetric((node, nodecnt, svc, svccnt)) => {
                self.inner
                    .insertsym::<()>(node, nodecnt, svc, svccnt, options.as_raw(), None)?
            }
        };

        Ok(PendingAVTranslation {
            fi_addresses,
            av: self.inner.clone(),
        })
    }

    /// Same as [Self::insert_no_block] but with an extra argument to provide a context
    pub fn insert_with_context_no_block<T>(
        &self,
        addr: AvInAddress,
        options: AVOptions,
        ctx: &mut Context,
    ) -> Result<PendingAVTranslation, crate::error::Error> {
        let fi_addresses = match addr {
            AvInAddress::String(str_addr) => {
                self.inner
                    .insertsvc_str(str_addr, options.as_raw(), Some(ctx.inner_mut()))?
            }
            AvInAddress::Encoded(addresses) => {
                self.inner.insert(addresses, options.as_raw(), Some(ctx))?
            }
            AvInAddress::Service((node, svc)) => {
                self.inner
                    .insertsvc(node, svc, options.as_raw(), Some(ctx))?
            }
            AvInAddress::Symmetric((node, nodecnt, svc, svccnt)) => {
                self.inner
                    .insertsym(node, nodecnt, svc, svccnt, options.as_raw(), Some(ctx))?
            }
        };

        Ok(PendingAVTranslation {
            fi_addresses,
            av: self.inner.clone(),
        })
    }
}

/// Builder for the [`AddressVector`] type.
///
/// `AddressVectorBuilder` is used to configure and build a new `AddressVector`.
/// It encapsulates an incremental configuration of the address vector, as provided by a `fi_av_attr`,
/// followed by a call to `fi_av_open`  
pub struct AddressVectorBuilder<'a, EQ: ?Sized, Mode = Block>
where
    Mode: AVSyncMode,
{
    av_attr: AddressVectorAttr,
    eq: Option<MyRc<EQ>>,
    ctx: Option<&'a mut Context>,
    phantom: PhantomData<Mode>,
}

impl<'a> AddressVectorBuilder<'a, ()> {
    /// Initiates the creation of a new [AddressVector].
    ///
    /// The initial configuration is what would be set if no `fi_av_attr` or `context` was provided to
    /// the `fi_av_open` call.
    pub fn new() -> AddressVectorBuilder<'a, ()> {
        AddressVectorBuilder {
            av_attr: AddressVectorAttr::new(),
            eq: None,
            ctx: None,
            phantom: PhantomData,
        }
    }

    /// Opens the [AddressVector] with a `name`.
    ///
    /// Corresponds to setting field `fi_av_attr::name`
    pub fn with_name(name: &str) -> AddressVectorBuilder<'a, ()> {
        let mut av_attr = AddressVectorAttr::new();
        av_attr.name(name.to_string());

        AddressVectorBuilder {
            av_attr,
            eq: None,
            ctx: None,
            phantom: PhantomData,
        }
    }

    /// Opens the [AddressVector] to read-only mode.
    ///
    /// Corresponds to setting the corresponding bit (`FI_READ`) of the field `fi_av_attr::flags`
    pub fn read_only(name: &str) -> AddressVectorBuilder<'a, ()> {
        let mut av_attr = AddressVectorAttr::new();
        av_attr.name(name.to_string()).read_only();

        AddressVectorBuilder {
            av_attr,
            eq: None,
            ctx: None,
            phantom: PhantomData,
        }
    }
}

impl Default for AddressVectorBuilder<'_, ()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<EQ, Mode: AVSyncMode> AddressVectorBuilder<'_, EQ, Mode> {
    /// Sets the type of the [AddressVector].
    ///
    /// Corresponds to setting field `fi_av_attr::type`
    pub fn type_(mut self, av_type: crate::enums::AddressVectorType) -> Self {
        self.av_attr.type_(av_type);
        self
    }

    /// Sets address bits to identify rx ctx of the [AddressVector].
    ///
    /// Corresponds to setting field `fi_av_attr::rx_ctx_bits`
    pub fn rx_ctx_bits(mut self, rx_ctx_bits: i32) -> Self {
        //[TODO] Maybe wrap bitfield
        self.av_attr.rx_ctx_bits(rx_ctx_bits);
        self
    }

    /// Sets the number of [Address]es that will be inserted into the [AddressVector]
    ///
    /// Corresponds to setting field `fi_av_attr::count`
    pub fn count(mut self, count: usize) -> Self {
        self.av_attr.count(count);
        self
    }

    /// Sets the number of [Endpoint][crate::ep::Endpoint]s that will be inserted into the [AddressVector]
    ///
    /// Corresponds to setting field `fi_av_attr::ep_per_node`
    pub fn ep_per_node(mut self, count: usize) -> Self {
        self.av_attr.ep_per_node(count);
        self
    }

    /// Sets the system name of the [AddressVector] to `name`.
    ///
    /// Corresponds to setting field `fi_av_attr::name`
    pub fn name(mut self, name: String) -> Self {
        self.av_attr.name(name);
        self
    }

    /// Sets the base mmap address of the [AddressVector] to `addr`.
    ///
    /// Corresponds to setting field `fi_av_attr::map_addr`
    pub fn map_addr(mut self, addr: usize) -> Self {
        self.av_attr.map_addr(addr);
        self
    }
}

impl<'a> AddressVectorBuilder<'a, (), Block> {
    /// Requests that insertions to [AddressVector] be done asynchronously.
    ///
    /// An asynchronous address vector is required to be bound to an [EventQueue] before any insertions take place.
    /// Thus, setting this option requires the user to specify the queue that will be used to report the completion
    /// of address insertions.
    ///
    /// Corresponds to setting the corresponding bit (`FI_EVENT`) of the field `fi_av_attr::flags` and calling
    /// `fi_av_bind(eq)`, once the address vector has been constructed.
    pub fn no_block<EQ: ReadEq + 'static>(
        mut self,
        eq: &'a EventQueueBase<EQ>,
    ) -> AddressVectorBuilder<'a, dyn ReadEq, NoBlock> {
        self.av_attr.async_();
        AddressVectorBuilder {
            av_attr: self.av_attr,
            eq: Some(eq.inner.clone()),
            ctx: self.ctx,
            phantom: PhantomData,
        }
    }
}

impl<'a, EQ, Mode: AVSyncMode> AddressVectorBuilder<'a, EQ, Mode> {
    /// Indicates that each node will be associated with the same number of endpoints.
    ///
    /// Corresponds to setting the corresponding bit (`FI_SYMMETRIC`) of the field `fi_av_attr::flags`.
    pub fn symmetric(mut self) -> Self {
        self.av_attr.symmetric();
        self
    }

    /// Sets the context to be passed to the [AddressVector].
    ///
    /// Corresponds to passing a non-NULL `context` value to `fi_av_open`.
    pub fn context(self, ctx: &'a mut Context) -> AddressVectorBuilder<'a, EQ> {
        AddressVectorBuilder {
            av_attr: self.av_attr,
            eq: self.eq,
            ctx: Some(ctx),
            phantom: PhantomData,
        }
    }
}
impl AddressVectorBuilder<'_, ()> {
    /// Constructs a new [AddressVector] with the configurations requested so far.
    ///
    /// Corresponds to creating an `fi_av_attr`, setting its fields to the requested ones,
    /// and calling `fi_av_open` with an optional `context`.
    pub fn build<DEQ: ?Sized + 'static + SyncSend>(
        self,
        domain: &DomainBase<DEQ>,
    ) -> Result<AddressVector, crate::error::Error> {
        let av = AddressVector::new(domain, self.av_attr, self.ctx)?;
        Ok(av)
        // match self.eq {
        //     None => Ok(av),
        //     Some(eq) => {av.inner.bind(eq)?; Ok(av)}
        // }
    }
}
impl<EQ: ?Sized + ReadEq + 'static> AddressVectorBuilder<'_, EQ> {
    /// Constructs a new [AddressVector] with the configurations requested so far.
    ///
    /// Corresponds to creating an `fi_av_attr`, setting its fields to the requested ones,
    /// calling `fi_av_open` with an optional `context`, and, if asynchronous, binding with
    /// the selected [EventQueue].
    pub fn build<DEQ: 'static + SyncSend>(
        self,
        domain: &DomainBase<DEQ>,
    ) -> Result<AddressVectorBase<Block, EQ>, crate::error::Error> {
        AddressVectorBase::new(domain, self.av_attr, self.ctx)
    }
}

impl<EQ: ?Sized + ReadEq + 'static> AddressVectorBuilder<'_, EQ, NoBlock> {
    /// Constructs a new [AddressVector] with the configurations requested so far.
    ///
    /// Corresponds to creating an `fi_av_attr`, setting its fields to the requested ones,
    /// calling `fi_av_open` with an optional `context`, and, binding with
    /// the selected [EventQueue].
    pub fn build<DEQ: 'static + SyncSend>(
        self,
        domain: &DomainBase<DEQ>,
    ) -> Result<AddressVectorBase<NoBlock, EQ>, crate::error::Error> {
        let av = AddressVectorBase::new(domain, self.av_attr, self.ctx)?;
        if let Some(eq) = self.eq {
            av.inner.bind(&eq)?;
        }
        Ok(av)
    }
}

/// The result of a pending, non-blocking AddressVector insert operation.
pub struct PendingAVTranslation {
    fi_addresses: Vec<u64>,
    av: MyRc<dyn AddressVectorImplT>,
}

impl PendingAVTranslation {
    /// Completes a pending AddressVector insert operation, returning the resulting MappedAddresses.
    pub fn av_complete(self, event: AVCompleteEvent) -> Vec<MappedAddress> {
        assert_eq!(event.fid(), &self.av.as_typed_fid().as_raw_fid());
        self.fi_addresses
            .into_iter()
            .map(|fi_addr| {
                MappedAddress::from_raw_addr(
                    RawMappedAddress::from_raw(self.av.type_(), fi_addr),
                    AddressSource::Av(self.av.clone()),
                )
            })
            .collect::<Vec<_>>()
    }

    /// Completes a pending AddressVector insert operation without checking the event target fid.
    pub fn av_complete_unchecked(self, _event: AVCompleteEvent) -> Vec<MappedAddress> {
        self.fi_addresses
            .into_iter()
            .map(|fi_addr| {
                MappedAddress::from_raw_addr(
                    RawMappedAddress::from_raw(self.av.type_(), fi_addr),
                    AddressSource::Av(self.av.clone()),
                )
            })
            .collect::<Vec<_>>()
    }
}

pub(crate) trait AddressVectorImplT: SyncSend + AsTypedFid<AvRawFid> {
    fn type_(&self) -> AddressVectorType;
}

impl<EQ: ?Sized + SyncSend + ReadEq> AddressVectorImplT for AddressVectorImplBase<EQ> {
    fn type_(&self) -> AddressVectorType {
        self.type_
    }
}

impl<EQ: ?Sized + SyncSend> SyncSend for AddressVectorImplBase<EQ> {}

impl AuthKey {
    pub(crate) fn from_bytes(raw_auth_key: &[u8]) -> Self {
        Self {
            auth_key: raw_auth_key.to_vec(),
        }
    }
}

//================== Trait Impls ==================//

impl<Mode: AVSyncMode, EQ: ?Sized + ReadEq> AsTypedFid<AvRawFid> for AddressVectorBase<Mode, EQ> {
    #[inline]
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<AvRawFid> {
        self.inner.as_typed_fid()
    }
    #[inline]
    fn as_typed_fid_mut(&self) -> fid::MutBorrowedTypedFid<AvRawFid> {
        self.inner.as_typed_fid_mut()
    }
}

//================== Private Impls ==================//

pub(crate) struct AddressVectorImplBase<EQ>
where
    EQ: ?Sized + SyncSend,
{
    pub(crate) c_av: OwnedAVFid,
    pub(crate) type_: AddressVectorType,
    pub(crate) _eq_rc: MyOnceCell<MyRc<EQ>>,
    pub(crate) _domain_rc: MyRc<dyn DomainImplT>,
}

impl<EQ: ?Sized + ReadEq> AddressVectorImplBase<EQ> {
    pub(crate) fn new<DEQ: ?Sized + 'static + SyncSend>(
        domain: &MyRc<crate::domain::DomainImplBase<DEQ>>,
        mut attr: AddressVectorAttr,
        context: *mut std::ffi::c_void,
    ) -> Result<Self, crate::error::Error> {
        let mut c_av: AvRawFid = std::ptr::null_mut();

        let err = unsafe {
            libfabric_sys::inlined_fi_av_open(
                domain.as_typed_fid_mut().as_raw_typed_fid(),
                attr.get_mut(),
                &mut c_av,
                context,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(Self {
                #[cfg(not(feature = "threading-domain"))]
                c_av: OwnedAVFid::from(c_av),
                #[cfg(feature = "threading-domain")]
                c_av: OwnedAVFid::from(c_av, domain.c_domain.domain.clone()),
                type_: AddressVectorType::from_raw(attr.c_attr.type_),
                _eq_rc: MyOnceCell::new(),
                _domain_rc: domain.clone(),
            })
        }
    }
}

impl<EQ: ?Sized + ReadEq> AddressVectorImplBase<EQ> {
    /// Associates an [EventQueue](crate::eq::EventQueue) with the AddressVector.
    ///
    /// This method directly corresponds to a call to `fi_av_bind(av, eq, 0)`.
    /// # Errors
    ///
    /// This function will return an error if the underlying library call fails.
    pub(crate) fn bind(&self, eq: &MyRc<EQ>) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_av_bind(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                eq.as_typed_fid().as_raw_fid(),
                0,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            if self._eq_rc.set(eq.clone()).is_err() {
                panic!("AddressVector is alread bound to an EventQueue");
            }
            Ok(())
        }
    }
}

impl<EQ: ?Sized + ReadEq> AddressVectorImplBase<EQ> {
    fn insert<T>(
        &self,
        addr: &[Address],
        flags: u64,
        ctx: Option<&mut T>,
    ) -> Result<Vec<libfabric_sys::fi_addr_t>, crate::error::Error> {
        // [TODO] //[TODO] Handle flags, handle context, handle async
        let mut fi_addresses = vec![0u64; addr.len()];
        let total_size = addr.iter().fold(0, |acc, addr| acc + addr.as_bytes().len());
        let mut serialized: Vec<u8> = Vec::with_capacity(total_size);
        for a in addr {
            serialized.extend(a.as_bytes().iter())
        }

        let err = if let Some(ctx) = ctx {
            unsafe {
                libfabric_sys::inlined_fi_av_insert(
                    self.as_typed_fid_mut().as_raw_typed_fid(),
                    serialized.as_ptr().cast(),
                    fi_addresses.len(),
                    fi_addresses.as_mut_ptr().cast(),
                    flags,
                    (ctx as *mut T).cast(),
                )
            }
        } else {
            unsafe {
                libfabric_sys::inlined_fi_av_insert(
                    self.as_typed_fid_mut().as_raw_typed_fid(),
                    serialized.as_ptr().cast(),
                    fi_addresses.len(),
                    fi_addresses.as_mut_ptr().cast(),
                    flags,
                    std::ptr::null_mut(),
                )
            }
        };

        if err < 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            // let mapped_addresses = fi_addresses.into_iter().map(|fi_addr| if fi_addr == FI_ADDR_NOTAVAIL {None} else {Some(MappedAddress::from_raw_addr(fi_addr, self))}).collect::<Vec<_>>();
            Ok(fi_addresses)
        }
    }

    pub(crate) fn insertsvc<T>(
        &self,
        node: &str,
        service: &str,
        flags: u64,
        ctx: Option<&mut T>,
    ) -> Result<Vec<libfabric_sys::fi_addr_t>, crate::error::Error> {
        let mut fi_addresses = vec![0u64; 1];
        let ctx = if let Some(ctx) = ctx {
            ctx as *mut T
        } else {
            std::ptr::null_mut()
        };
        let err = unsafe {
            libfabric_sys::inlined_fi_av_insertsvc(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                node.as_bytes().as_ptr().cast(),
                service.as_bytes().as_ptr().cast(),
                fi_addresses.as_mut_ptr(),
                flags,
                ctx.cast(),
            )
        };

        if err < 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(fi_addresses)
        }
    }

    pub(crate) fn insertsvc_str(
        &self,
        service_str: &str,
        flags: u64,
        ctx: Option<*mut std::ffi::c_void>,
    ) -> Result<Vec<libfabric_sys::fi_addr_t>, crate::error::Error> {
        let mut fi_addresses = vec![0u64; 1];
        let ctx = if let Some(ctx) = ctx {
            ctx
        } else {
            std::ptr::null_mut()
        };

        let c_str = CString::new(service_str).unwrap();
        let err = unsafe {
            libfabric_sys::inlined_fi_av_insertsvc(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                c_str.as_ptr(),
                std::ptr::null(),
                fi_addresses.as_mut_ptr(),
                flags,
                ctx.cast(),
            )
        };

        if err < 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(fi_addresses)
        }
    }

    pub(crate) fn insertsym<T>(
        &self,
        node: &str,
        nodecnt: usize,
        service: &str,
        svccnt: usize,
        flags: u64,
        ctx: Option<&mut T>,
    ) -> Result<Vec<libfabric_sys::fi_addr_t>, crate::error::Error> {
        // [TODO] Handle case where operation partially failed
        let total_cnt = nodecnt * svccnt;
        let mut fi_addresses = vec![0u64; total_cnt];
        let c_node_str = CString::new(node).unwrap();
        let c_svc_str = CString::new(service).unwrap();
        let ctx = if let Some(ctx) = ctx {
            ctx as *mut T
        } else {
            std::ptr::null_mut()
        };

        let err = unsafe {
            libfabric_sys::inlined_fi_av_insertsym(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                c_node_str.as_ptr(),
                nodecnt,
                c_svc_str.as_ptr(),
                svccnt,
                fi_addresses.as_mut_ptr().cast(),
                flags,
                ctx.cast(),
            )
        };

        if err < 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            // let mapped_addresses = fi_addresses.into_iter().map(|fi_addr| if fi_addr == FI_ADDR_NOTAVAIL {None} else {Some(MappedAddress::from_raw_addr(fi_addr))}).collect::<Vec<_>>();
            Ok(fi_addresses)
        }
    }

    pub(crate) fn remove(
        &self,
        addr: Vec<crate::MappedAddress>,
    ) -> Result<(), crate::error::Error> {
        let mut fi_addresses = addr
            .into_iter()
            .map(|mapped_addr| mapped_addr.raw_addr())
            .collect::<Vec<libfabric_sys::fi_addr_t>>();

        let err = unsafe {
            libfabric_sys::inlined_fi_av_remove(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                fi_addresses.as_mut_ptr().cast(),
                fi_addresses.len(),
                0,
            )
        };

        if err < 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn lookup(
        &self,
        mapped_addr: crate::MappedAddress,
    ) -> Result<Address, crate::error::Error> {
        let mut addrlen: usize = 0;
        let err = unsafe {
            libfabric_sys::inlined_fi_av_lookup(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                mapped_addr.raw_addr(),
                std::ptr::null_mut(),
                &mut addrlen,
            )
        };

        if -err as u32 == libfabric_sys::FI_ETOOSMALL {
            let mut addr = vec![0u8; addrlen];
            let err = unsafe {
                libfabric_sys::inlined_fi_av_lookup(
                    self.as_typed_fid_mut().as_raw_typed_fid(),
                    mapped_addr.raw_addr(),
                    addr.as_mut_ptr().cast(),
                    &mut addrlen,
                )
            };

            if err < 0 {
                Err(crate::error::Error::from_err_code(
                    (-err).try_into().unwrap(),
                ))
            } else {
                Ok(unsafe { Address::from_bytes(&addr) })
            }
        } else {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        }
    }

    pub(crate) fn straddr(&self, addr: &Address) -> String {
        let mut addr_str: Vec<u8> = Vec::new();
        let mut strlen = addr_str.len();
        let strlen_ptr: *mut usize = &mut strlen;
        unsafe {
            libfabric_sys::inlined_fi_av_straddr(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                addr.as_bytes().as_ptr().cast(),
                addr_str.as_mut_ptr().cast(),
                strlen_ptr,
            )
        };
        addr_str.resize(strlen, 1);

        let mut strlen = addr_str.len();
        let strlen_ptr: *mut usize = &mut strlen;
        unsafe {
            libfabric_sys::inlined_fi_av_straddr(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                addr.as_bytes().as_ptr().cast(),
                addr_str.as_mut_ptr().cast(),
                strlen_ptr,
            )
        };
        std::ffi::CString::from_vec_with_nul(addr_str)
            .unwrap()
            .into_string()
            .unwrap()
    }

    pub(crate) fn insert_auth_key(&self, auth_key: &AuthKey) -> Result<(), crate::error::Error> {
        let mut fi_addr = 0u64;

        let err = unsafe {
            libfabric_sys::inlined_fi_av_insert_auth_key(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                auth_key.auth_key.as_ptr().cast(),
                std::mem::size_of_val(auth_key),
                &mut fi_addr,
                0,
            )
        };

        check_error(err as isize)
    }

    pub(crate) fn lookup_auth_key(
        &self,
        mapped_addr: &MappedAddress,
    ) -> Result<AuthKey, crate::error::Error> {
        let mut key_bytes: Vec<u8> = Vec::new();
        let mut key_len = key_bytes.len();

        unsafe {
            libfabric_sys::inlined_fi_av_lookup_auth_key(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                mapped_addr.raw_addr(),
                key_bytes.as_mut_ptr().cast(),
                &mut key_len,
            )
        };

        key_bytes.resize(key_len, 0);
        key_len = key_bytes.len();

        let err = unsafe {
            libfabric_sys::inlined_fi_av_lookup_auth_key(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                mapped_addr.raw_addr(),
                key_bytes.as_mut_ptr().cast(),
                &mut key_len,
            )
        };

        if err < 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(AuthKey::from_bytes(&key_bytes))
        }
    }

    pub(crate) fn set_user_id(
        &self,
        mapped_addr: &MappedAddress,
        user_id: &MappedAddress,
        flags: crate::enums::UserId,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_av_set_user_id(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                mapped_addr.raw_addr(),
                user_id.raw_addr(),
                flags.as_raw(),
            )
        };
        check_error(err as isize)
    }
}

//================== Attribute Structs ==================//

pub(crate) struct AddressVectorAttr {
    pub(crate) c_attr: libfabric_sys::fi_av_attr,
}

impl AddressVectorAttr {
    pub(crate) fn new() -> Self {
        let c_attr = libfabric_sys::fi_av_attr {
            type_: crate::enums::AddressVectorType::Unspec.as_raw(),
            rx_ctx_bits: 0,
            count: 0,
            ep_per_node: 0,
            name: std::ptr::null(),
            map_addr: std::ptr::null_mut(),
            flags: 0,
        };

        Self { c_attr }
    }

    pub(crate) fn type_(&mut self, av_type: crate::enums::AddressVectorType) -> &mut Self {
        self.c_attr.type_ = av_type.as_raw();
        self
    }

    pub(crate) fn rx_ctx_bits(&mut self, rx_ctx_bits: i32) -> &mut Self {
        self.c_attr.rx_ctx_bits = rx_ctx_bits;
        self
    }

    pub(crate) fn count(&mut self, count: usize) -> &mut Self {
        self.c_attr.count = count;
        self
    }

    pub(crate) fn ep_per_node(&mut self, count: usize) -> &mut Self {
        self.c_attr.ep_per_node = count;
        self
    }

    pub(crate) fn name(&mut self, name: String) -> &mut Self {
        let c_str = std::ffi::CString::new(name).unwrap();
        self.c_attr.name = c_str.into_raw();
        self
    }

    pub(crate) fn map_addr(&mut self, addr: usize) -> &mut Self {
        //[TODO] Datatype correct??
        self.c_attr.map_addr = addr as *mut std::ffi::c_void;
        self
    }

    pub(crate) fn read_only(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_READ as u64;
        self
    }

    pub(crate) fn symmetric(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_SYMMETRIC;
        self
    }

    pub(crate) fn async_(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_EVENT as u64;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_av_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_av_attr {
        &mut self.c_attr
    }
}

impl Default for AddressVectorAttr {
    fn default() -> Self {
        Self::new()
    }
}

impl<EQ: ?Sized + ReadEq> AsTypedFid<AvRawFid> for AddressVectorImplBase<EQ> {
    #[inline]
    fn as_typed_fid(&self) -> fid::BorrowedTypedFid<AvRawFid> {
        self.c_av.as_typed_fid()
    }
    #[inline]
    fn as_typed_fid_mut(&self) -> fid::MutBorrowedTypedFid<AvRawFid> {
        self.c_av.as_typed_fid_mut()
    }
}

//================== Tests ==================//

#[cfg(test)]
mod tests {
    use crate::info::Info;

    use super::AddressVectorBuilder;

    #[test]
    fn av_open_close() {
        let info = Info::new(&crate::info::libfabric_version())
            .enter_hints()
            .enter_ep_attr()
            .type_(crate::enums::EndpointType::Rdm)
            .leave_ep_attr()
            .enter_domain_attr()
            .mode(crate::enums::Mode::all())
            .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse())
            .leave_domain_attr()
            .leave_hints()
            .get()
            .unwrap();

        let entry = info.into_iter().next().unwrap();

        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let domain = crate::domain::DomainBuilder::new(&fab, &entry)
            .build()
            .unwrap();

        for i in 0..17 {
            let count = 1 << i;
            let _av = AddressVectorBuilder::new()
                .type_(crate::enums::AddressVectorType::Map)
                .count(count)
                .build(&domain)
                .unwrap();
        }
    }

    #[test]
    fn av_good_sync() {
        let info = Info::new(&crate::info::libfabric_version())
            .enter_hints()
            .enter_ep_attr()
            .type_(crate::enums::EndpointType::Rdm)
            .leave_ep_attr()
            .enter_domain_attr()
            .mode(crate::enums::Mode::all())
            .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse())
            .leave_domain_attr()
            .leave_hints()
            .get()
            .unwrap();

        let entry = info.into_iter().next().unwrap();

        let fab: crate::fabric::Fabric = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let domain = crate::domain::DomainBuilder::new(&fab, &entry)
            .build()
            .unwrap();
        let _av = AddressVectorBuilder::new()
            .type_(crate::enums::AddressVectorType::Map)
            .count(32)
            .build(&domain)
            .unwrap();
    }
}

#[cfg(test)]
mod libfabric_lifetime_tests {
    use crate::info::Info;

    use super::AddressVectorBuilder;

    #[test]
    fn av_drops_before_domain() {
        let info = Info::new(&crate::info::libfabric_version())
            .enter_hints()
            .enter_ep_attr()
            .type_(crate::enums::EndpointType::Rdm)
            .leave_ep_attr()
            .enter_domain_attr()
            .mode(crate::enums::Mode::all())
            .mr_mode(crate::enums::MrMode::new().basic().scalable().inverse())
            .leave_domain_attr()
            .leave_hints()
            .get()
            .unwrap();

        let entry = info.into_iter().next().unwrap();
        let fab = crate::fabric::FabricBuilder::new().build(&entry).unwrap();
        let domain = crate::domain::DomainBuilder::new(&fab, &entry)
            .build()
            .unwrap();

        let mut avs = Vec::new();
        for i in 0..17 {
            let count = 1 << i;
            let av = AddressVectorBuilder::new()
                .type_(crate::enums::AddressVectorType::Map)
                .count(count)
                .build(&domain)
                .unwrap();
            avs.push(av);
        }
        drop(domain);
    }
}
