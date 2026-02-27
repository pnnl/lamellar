use crate::{
    av::{AVSyncMode, AddressVectorBase, AddressVectorImplT},
    enums::AddressVectorType,
    eq::ReadEq,
    fid::{
        AVSetRawFid, AsRawTypedFid, AsTypedFid, BorrowedTypedFid, MutBorrowedTypedFid,
        OwnedAVSetFid,
    },
    AddressSource, Context, MappedAddress, MyRc, RawMappedAddress, FI_ADDR_NOTAVAIL,
};

//================== AddressVectorSet Public ==================//

/// Owned wrapper around a libfabric `fid_av_set`.
///
/// This type wraps an instance of a `fid_av_set`, monitoring its lifetime and closing it when it goes out of scope.
/// For more information see the libfabric [documentation](https://ofiwg.github.io/libfabric/v1.22.0/man/fi_av_set.3.html).
///
/// Note that other objects that rely on an AddressVectorSet (e.g., [crate::comm::collective::MulticastGroupCollective]) will extend its lifetime until they
/// are also dropped.
pub struct AddressVectorSet {
    pub(crate) inner: MyRc<AddressVectorSetImpl>,
}

impl AddressVectorSet {
    pub(crate) fn new<Mode: AVSyncMode, EQ: 'static + ?Sized + ReadEq>(
        av: &AddressVectorBase<Mode, EQ>,
        attr: AddressVectorSetAttr,
        context: Option<&mut Context>,
    ) -> Result<Self, crate::error::Error> {
        let c_void = match context {
            Some(ctx) => ctx.inner_mut(),
            None => std::ptr::null_mut(),
        };

        Ok(Self {
            inner: MyRc::new(AddressVectorSetImpl::new(av, attr, c_void)?),
        })
    }

    /// Perform a set union operation on two AV sets
    ///
    /// The result is stored in `Self`, which is modified.
    ///
    /// Corresponds to `fi_av_set_union`
    pub fn union(&mut self, other: &AddressVectorSet) -> Result<(), crate::error::Error> {
        self.inner.union(&other.inner)
    }

    /// Perform a set intersection operation on two AV sets
    ///
    /// The result is stored in `Self`, which is modified.
    ///
    /// Corresponds to `fi_av_set_intersect`
    pub fn intersect(&mut self, other: &AddressVectorSet) -> Result<(), crate::error::Error> {
        self.inner.intersect(&other.inner)
    }

    /// Perform a set difference operation on two AV sets
    ///
    /// The result is stored in `Self`, which is modified.
    ///
    /// Corresponds to `fi_av_set_diff`
    pub fn diff(&mut self, other: &AddressVectorSet) -> Result<(), crate::error::Error> {
        self.inner.diff(&other.inner)
    }

    /// Adds an address to the [AddressVectorSet].
    ///
    /// `Self` is modified.
    ///
    /// Corresponds to `fi_av_set_insert`
    pub fn insert(
        &mut self,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error> {
        self.inner.insert(mapped_addr)
    }

    /// Removes an address to the [AddressVectorSet].
    ///
    /// `Self` is modified.
    ///
    /// Corresponds to `fi_av_set_remove`
    pub fn remove(
        &mut self,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error> {
        self.inner.remove(mapped_addr)
    }

    /// Retrieves an address associated with the [AddressVectorSet].
    ///
    /// Corresponds to `fi_av_set_addr`
    pub fn address(&self) -> Result<crate::MappedAddress, crate::error::Error> {
        let raw_addr = self.inner.address()?;
        Ok(MappedAddress::from_raw_addr(
            raw_addr,
            AddressSource::AvSet(self.inner.clone()),
        ))
    }
}

/// Builder for the AddressVectorSet type.
///
/// `AddressVectorSetBuilder` is used to configure and build a new [AddressVectorSet].
/// It encapsulates an incremental configuration of the address vector set, as provided by a `fi_av_set_attr`,
/// followed by a call to `fi_av_set`  
pub struct AddressVectorSetBuilder<'a, Mode: AVSyncMode, EQ: ReadEq + ?Sized> {
    avset_attr: AddressVectorSetAttr,
    ctx: Option<&'a mut Context>,
    av: &'a AddressVectorBase<Mode, EQ>,
}

impl<'a, Mode: AVSyncMode, EQ: ?Sized + ReadEq> AddressVectorSetBuilder<'a, Mode, EQ> {
    /// Initiates the creation of a new [AddressVectorSet] from a range of addresses in an existing [AddressVector] of Table type.
    pub fn new_from_range(
        av: &'a AddressVectorBase<Mode, EQ>,
        start_addr: &crate::MappedAddress,
        end_addr: &crate::MappedAddress,
        stride: usize,
    ) -> AddressVectorSetBuilder<'a, Mode, EQ> {
        if !matches!(av.inner.type_(), AddressVectorType::Table) {
            panic!("Can only use new_from_range for AVs of Table addressing type");
        }

        let mut avset_attr = AddressVectorSetAttr::new();
        avset_attr
            .start_addr(start_addr)
            .end_addr(end_addr)
            .stride(stride);

        AddressVectorSetBuilder {
            avset_attr,
            ctx: None,
            av,
        }
    }

    /// Initiates the creation of a new empty [AddressVectorSet] from an existing [AddressVector].
    pub fn new(av: &'a AddressVectorBase<Mode, EQ>) -> AddressVectorSetBuilder<'a, Mode, EQ> {
        let mut avset_attr = AddressVectorSetAttr::new();
        avset_attr.c_attr.start_addr = FI_ADDR_NOTAVAIL;
        avset_attr.c_attr.end_addr = FI_ADDR_NOTAVAIL;
        avset_attr.c_attr.count = 0;

        AddressVectorSetBuilder {
            avset_attr,
            ctx: None,
            av,
        }
    }
}

impl<'a, Mode: AVSyncMode, EQ: ?Sized + ReadEq + 'static> AddressVectorSetBuilder<'a, Mode, EQ> {
    /// Indicates the expected the number of members that will be a part of the AV set.
    ///
    /// Corresponds to setting the `fi_av_set_attr::count` field.
    pub fn count(mut self, size: usize) -> Self {
        self.avset_attr.count(size);
        self
    }

    /// If supported by the fabric, this represents a key associated with the AV set.
    ///
    /// Corresponds to setting the `fi_av_set_attr::comm_key` and `fi_av_set_attr::comm_key_size` fields.
    pub fn comm_key(mut self, key: &mut [u8]) -> Self {
        self.avset_attr.comm_key(key);
        self
    }

    /// May be used to configure the AV set, including restricting which collective operations the AV set needs to support.
    ///
    /// Corresponds to oring the `fi_av_set_attr::flags` field with FI_BARRIER_SET .
    pub fn support_barrier(mut self) -> Self {
        self.avset_attr.support_barrier();
        self
    }

    /// May be used to configure the AV set, including restricting which collective operations the AV set needs to support.
    ///
    /// Corresponds to oring the `fi_av_set_attr::flags` field with FI_BROADCAST_SET .
    pub fn support_broadcast(mut self) -> Self {
        self.avset_attr.support_broadcast();
        self
    }

    /// May be used to configure the AV set, including restricting which collective operations the AV set needs to support.
    ///
    /// `options` captures the [flags](AVSetOptions) that can be possibly set for an AV set.
    ///
    /// Corresponds to oring the `fi_av_set_attr::flags` field with FI_ALLTOALL_SET .
    pub fn support_alltoall(mut self) -> Self {
        self.avset_attr.support_alltoall();
        self
    }

    /// May be used to configure the AV set, including restricting which collective operations the AV set needs to support.
    ///
    /// Corresponds to oring the `fi_av_set_attr::flags` field with FI_ALLREDUCE_SET .
    pub fn support_allreduce(mut self) -> Self {
        self.avset_attr.support_allreduce();
        self
    }

    /// May be used to configure the AV set, including restricting which collective operations the AV set needs to support.
    ///
    /// Corresponds to oring the `fi_av_set_attr::flags` field with FI_GATHER_SET .
    pub fn support_allgather(mut self) -> Self {
        self.avset_attr.support_allgather();
        self
    }

    /// May be used to configure the AV set, including restricting which collective operations the AV set needs to support.
    ///
    /// Corresponds to oring the `fi_av_set_attr::flags` field with FI_REDUCE_SCATTER_SET .
    pub fn support_reduce_scatter(mut self) -> Self {
        self.avset_attr.support_reduce_scatter();
        self
    }

    /// May be used to configure the AV set, including restricting which collective operations the AV set needs to support.
    ///
    /// Corresponds to oring the `fi_av_set_attr::flags` field with FI_REDUCE_SET .
    pub fn support_reduce(mut self) -> Self {
        self.avset_attr.support_reduce();
        self
    }

    /// May be used to configure the AV set, including restricting which collective operations the AV set needs to support.
    ///
    /// Corresponds to oring the `fi_av_set_attr::flags` field with FI_SCATTER_SET .
    pub fn support_scatter(mut self) -> Self {
        self.avset_attr.support_scatter();
        self
    }

    /// May be used to configure the AV set, including restricting which collective operations the AV set needs to support.
    ///
    /// Corresponds to oring the `fi_av_set_attr::flags` field with FI_GATHER_SET .
    pub fn support_gather(mut self) -> Self {
        self.avset_attr.support_gather();
        self
    }

    /// Sets the context to be passed to the AV set.
    ///
    /// Corresponds to passing a non-NULL `context` value to `fi_av_set`.
    pub fn context(self, ctx: &'a mut Context) -> AddressVectorSetBuilder<'a, Mode, EQ> {
        AddressVectorSetBuilder {
            avset_attr: self.avset_attr,
            av: self.av,
            ctx: Some(ctx),
        }
    }

    /// Constructs a new [AddressVectorSet] with the configurations requested so far.
    ///
    /// Corresponds to creating an `fi_av_set_attr`, setting its fields to the requested ones,
    /// passing it to a `fi_av_set` call with an optional `context` (set by [Self::context]).
    pub fn build(self) -> Result<AddressVectorSet, crate::error::Error> {
        AddressVectorSet::new(self.av, self.avset_attr, self.ctx)
    }
}

//================== Trait Implementations ==================//

impl AsTypedFid<AVSetRawFid> for AddressVectorSet {
    #[inline]
    fn as_typed_fid(&self) -> BorrowedTypedFid<'_, AVSetRawFid> {
        self.inner.as_typed_fid()
    }

    #[inline]
    fn as_typed_fid_mut(&self) -> MutBorrowedTypedFid<'_, AVSetRawFid> {
        self.inner.as_typed_fid_mut()
    }
}

//================== Private Impl ==================//

pub(crate) struct AddressVectorSetImpl {
    pub(crate) c_set: OwnedAVSetFid,
    pub(crate) _av_rc: MyRc<dyn AddressVectorImplT>,
}

impl AddressVectorSetImpl {
    fn new<Mode: AVSyncMode, EQ: ?Sized + ReadEq + 'static>(
        av: &AddressVectorBase<Mode, EQ>,
        mut attr: AddressVectorSetAttr,
        context: *mut std::ffi::c_void,
    ) -> Result<Self, crate::error::Error> {
        let mut c_set: AVSetRawFid = std::ptr::null_mut();

        let err = unsafe {
            libfabric_sys::inlined_fi_av_set(
                av.as_typed_fid_mut().as_raw_typed_fid(),
                attr.get_mut(),
                &mut c_set,
                context,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(Self {
                c_set: OwnedAVSetFid::from(c_set),
                _av_rc: av.inner.clone(),
            })
        }
    }

    pub(crate) fn union(&self, other: &AddressVectorSetImpl) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_av_set_union(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                other.as_typed_fid().as_raw_typed_fid(),
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn intersect(
        &self,
        other: &AddressVectorSetImpl,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_av_set_intersect(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                other.as_typed_fid().as_raw_typed_fid(),
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn diff(&self, other: &AddressVectorSetImpl) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_av_set_diff(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                other.as_typed_fid().as_raw_typed_fid(),
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn insert(
        &self,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_av_set_insert(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                mapped_addr.raw_addr(),
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn remove(
        &self,
        mapped_addr: &crate::MappedAddress,
    ) -> Result<(), crate::error::Error> {
        let err = unsafe {
            libfabric_sys::inlined_fi_av_set_remove(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                mapped_addr.raw_addr(),
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(())
        }
    }

    pub(crate) fn address(&self) -> Result<RawMappedAddress, crate::error::Error> {
        let mut addr = 0u64;
        // let addr_ptr: *mut crate::MappedAddress = &mut addr;
        let err = unsafe {
            libfabric_sys::inlined_fi_av_set_addr(
                self.as_typed_fid_mut().as_raw_typed_fid(),
                &mut addr,
            )
        };

        if err != 0 {
            Err(crate::error::Error::from_err_code(
                (-err).try_into().unwrap(),
            ))
        } else {
            Ok(RawMappedAddress::from_raw(self._av_rc.type_(), addr))
        }
    }
}

impl AsTypedFid<AVSetRawFid> for AddressVectorSetImpl {
    #[inline]
    fn as_typed_fid(&self) -> BorrowedTypedFid<'_, AVSetRawFid> {
        self.c_set.as_typed_fid()
    }
    #[inline]
    fn as_typed_fid_mut(&self) -> MutBorrowedTypedFid<'_, AVSetRawFid> {
        self.c_set.as_typed_fid_mut()
    }
}

pub(crate) struct AddressVectorSetAttr {
    c_attr: libfabric_sys::fi_av_set_attr,
}

impl AddressVectorSetAttr {
    pub(crate) fn new() -> Self {
        Self {
            c_attr: libfabric_sys::fi_av_set_attr {
                count: 0,
                start_addr: 0,
                end_addr: 0,
                stride: 0,
                comm_key_size: 0,
                comm_key: std::ptr::null_mut(),
                flags: 0,
            },
        }
    }

    pub(crate) fn count(&mut self, size: usize) -> &mut Self {
        self.c_attr.count = size;
        self
    }

    pub(crate) fn start_addr(&mut self, mapped_addr: &crate::MappedAddress) -> &mut Self {
        self.c_attr.start_addr = mapped_addr.raw_addr();
        self
    }

    pub(crate) fn end_addr(&mut self, mapped_addr: &crate::MappedAddress) -> &mut Self {
        self.c_attr.end_addr = mapped_addr.raw_addr();
        self
    }

    pub(crate) fn stride(&mut self, stride: usize) -> &mut Self {
        self.c_attr.stride = stride as u64;
        self
    }

    pub(crate) fn comm_key(&mut self, key: &mut [u8]) -> &mut Self {
        self.c_attr.comm_key_size = key.len();
        self.c_attr.comm_key = key.as_mut_ptr();
        self
    }

    pub(crate) fn support_barrier(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_BARRIER_SET;
        self
    }

    pub(crate) fn support_broadcast(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_BROADCAST_SET;
        self
    }

    pub(crate) fn support_alltoall(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_ALLTOALL_SET;
        self
    }

    pub(crate) fn support_allreduce(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_ALLREDUCE_SET;
        self
    }

    pub(crate) fn support_allgather(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_ALLGATHER_SET;
        self
    }

    pub(crate) fn support_reduce_scatter(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_REDUCE_SCATTER_SET;
        self
    }

    pub(crate) fn support_reduce(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_REDUCE_SET;
        self
    }

    pub(crate) fn support_scatter(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_REDUCE_SCATTER_SET;
        self
    }

    pub(crate) fn support_gather(&mut self) -> &mut Self {
        self.c_attr.flags |= libfabric_sys::FI_GATHER_SET;
        self
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> *const libfabric_sys::fi_av_set_attr {
        &self.c_attr
    }

    pub(crate) fn get_mut(&mut self) -> *mut libfabric_sys::fi_av_set_attr {
        &mut self.c_attr
    }
}

impl Default for AddressVectorSetAttr {
    fn default() -> Self {
        Self::new()
    }
}
