//[TODO] All of the contents of this file need further testing
// and validation. The code is not guaranteed to be correct or complete.

use std::{ffi::CStr, os::raw::c_void};

use crate::{
    enums::{DataType, ProfileDataType, Type},
    fid::{
        AsRawFid, AsRawTypedFid, AsTypedFid, BorrowedTypedFid, DomainRawFid, EpRawFid,
        OwnedProfileFid, ProfileRawFid, RawFid,
    },
    utils::check_error,
    Context,
};

/// Represents a profile in the system.
///
/// Corresponds to a `fi_profile` struct.
pub struct Profile {
    c_profile: OwnedProfileFid,
}

struct ClosureWrapper {
    closure: Box<dyn Fn(&Profile, &ProfileDesc, usize, &mut Context) -> i32>,
}

unsafe extern "C" fn prof_callback<'a>(
    fid: *mut libfabric_sys::fid_profile,
    desc: *mut libfabric_sys::fi_profile_desc,
    _data: *mut c_void,
    size: usize,
    ctx: *mut c_void,
) -> i32 {
    let profile = unsafe { &*(fid as *const Profile) };
    let desc = unsafe { &*(desc as *const ProfileDesc<'a>) };
    let ctx = unsafe { &mut *(ctx as *mut Context) };
    let raw_closure = match ctx.0 {
        crate::ContextType::Context1(ref context1) => context1.c_val.internal[0],
        crate::ContextType::Context2(ref context2) => context2.c_val.internal[0],
    };
    let boxed_closure = unsafe { Box::from_raw(raw_closure as *mut ClosureWrapper) };
    (*boxed_closure.closure)(profile, desc, size, ctx)
}

impl Profile {
    fn new(
        fid: &impl AsRawFid,
        ctx: Option<&mut crate::Context>,
    ) -> Result<Self, crate::error::Error> {
        let mut c_profile: ProfileRawFid = std::ptr::null_mut();
        let err = unsafe {
            libfabric_sys::inlined_fi_profile_open(
                fid.as_raw_fid(),
                0,
                &mut c_profile,
                ctx.map_or(std::ptr::null_mut(), |ctx| ctx.inner_mut()),
            )
        };
        check_error(err.try_into().unwrap())?;
        Ok(Self {
            c_profile: OwnedProfileFid::from(c_profile),
        })
    }

    /// Resets the profile to its initial state.
    ///
    /// Corresponds to a `fi_profile_reset` function.
    pub fn reset(&mut self) {
        unsafe {
            libfabric_sys::inlined_fi_profile_reset(
                self.c_profile.as_typed_fid().as_raw_typed_fid(),
                0,
            );
        }
    }

    /// Queries the variables of the profile.
    ///
    /// Corresponds to a `fi_profile_query_vars` function.
    pub fn query_vars(&self) -> Result<Vec<ProfileDesc<'_>>, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe {
            libfabric_sys::inlined_fi_profile_query_vars(
                self.as_typed_fid().as_raw_typed_fid(),
                std::ptr::null_mut(),
                &mut count,
            )
        };
        if err < 0 {
            return Err(crate::error::Error::from_err_code(err.try_into().unwrap()));
        }

        let mut descs: Vec<libfabric_sys::fi_profile_desc> = Vec::with_capacity(count);
        let err = unsafe {
            libfabric_sys::inlined_fi_profile_query_vars(
                self.c_profile.as_typed_fid().as_raw_typed_fid(),
                descs.as_mut_ptr(),
                &mut count,
            )
        };

        if err < 0 {
            return Err(crate::error::Error::from_err_code(err.try_into().unwrap()));
        }

        unsafe { descs.set_len(err as usize) };
        Ok(descs
            .into_iter()
            .map(ProfileDesc::from_raw)
            .collect())
    }

    /// Queries the events of the profile.
    ///
    /// Corresponds to a `fi_profile_query_events` function.
    pub fn query_events(&self) -> Result<Vec<ProfileDesc<'_>>, crate::error::Error> {
        let mut count: usize = 0;
        let err = unsafe {
            libfabric_sys::inlined_fi_profile_query_events(
                self.as_typed_fid().as_raw_typed_fid(),
                std::ptr::null_mut(),
                &mut count,
            )
        };
        if err < 0 {
            return Err(crate::error::Error::from_err_code(err.try_into().unwrap()));
        }

        let mut descs: Vec<libfabric_sys::fi_profile_desc> = Vec::with_capacity(count);
        let err = unsafe {
            libfabric_sys::inlined_fi_profile_query_events(
                self.c_profile.as_typed_fid().as_raw_typed_fid(),
                descs.as_mut_ptr(),
                &mut count,
            )
        };

        if err < 0 {
            return Err(crate::error::Error::from_err_code(err.try_into().unwrap()));
        }

        unsafe { descs.set_len(err as usize) };
        Ok(descs
            .into_iter()
            .map(ProfileDesc::from_raw)
            .collect())
    }

    /// Reads a 64-bit unsigned integer variable from the profile.
    ///
    /// Corresponds to a `fi_profile_read_u64` function.
    pub fn read_u64(&self, var_id: u32) -> Result<u64, crate::error::Error> {
        let mut value: u64 = 0;
        let err = unsafe {
            libfabric_sys::inlined_fi_profile_read_u64(
                self.c_profile.as_typed_fid().as_raw_typed_fid(),
                var_id,
                &mut value,
            )
        };
        if err < 0 {
            return Err(crate::error::Error::from_err_code(err.try_into().unwrap()));
        }
        Ok(value)
    }

    /// Registers a callback for profile events.
    ///
    /// Corresponds to a `fi_profile_register_callback` function.
    pub fn register_callback<'a>(
        &self,
        event_id: u32,
        callback: impl Fn(&Profile, &ProfileDesc, usize, &mut Context) -> i32 + 'static,
        ctx: &mut Context,
    ) -> Result<(), crate::error::Error> {
        let boxed_closure = Box::new(ClosureWrapper {
            closure: Box::new(callback),
        });
        match &mut ctx.0 {
            crate::ContextType::Context1(ref mut context1) => {
                context1.c_val.internal[0] = Box::into_raw(boxed_closure) as *mut c_void;
            }
            crate::ContextType::Context2(ref mut context2) => {
                context2.c_val.internal[0] = Box::into_raw(boxed_closure) as *mut c_void;
            }
        }
        let err = unsafe {
            libfabric_sys::inlined_fi_profile_register_callback(
                self.c_profile.as_typed_fid().as_raw_typed_fid(),
                event_id,
                Some(prof_callback),
                ctx.inner_mut(),
            )
        };

        check_error(err.try_into().unwrap())
    }
}

impl AsTypedFid<ProfileRawFid> for Profile {
    fn as_typed_fid(&self) -> BorrowedTypedFid<'_, ProfileRawFid> {
        self.c_profile.as_typed_fid()
    }

    fn as_typed_fid_mut(&self) -> crate::fid::MutBorrowedTypedFid<'_, ProfileRawFid> {
        self.c_profile.as_typed_fid_mut()
    }
}

impl AsRawFid for Profile {
    fn as_raw_fid(&self) -> RawFid {
        self.c_profile.as_typed_fid().as_raw_fid()
    }
}

pub struct ProfileBuilder<'a> {
    pub(crate) ctx: Option<&'a mut Context>,
    pub(crate) fid: RawFid,
}

impl<'a> ProfileBuilder<'a> {
    /// Initiates the process for building a Profile for a specific [crate::ep::Endpoint].
    pub fn endpoint(ep: &impl AsTypedFid<EpRawFid>) -> Self {
        Self {
            fid: ep.as_typed_fid().as_raw_fid(),
            ctx: None,
        }
    }

    /// Initiates the process for building a Profile for a specific [crate::domain::Domain].
    pub fn domain(domain: &impl AsTypedFid<DomainRawFid>) -> Self {
        Self {
            fid: domain.as_typed_fid().as_raw_fid(),
            ctx: None,
        }
    }

    /// Sets a [Context] for the ProfileBuilder.
    pub fn context(mut self, ctx: &'a mut Context) -> Self {
        self.ctx = Some(ctx);
        self
    }

    /// Builds the Profile.
    pub fn build(self) -> Result<Profile, crate::error::Error> {
        Profile::new(&self.fid, self.ctx)
    }
}

/// Represents a profile descriptor in the system.
///
/// Corresponds to a `fi_profile_desc` struct.
pub struct ProfileDesc<'a> {
    id: u32,
    flags: u64,
    profile_data: ProfileDataType,
    name: &'a str,
    desc: &'a str,
}

impl ProfileDesc<'_> {
    fn from_raw(c_profile_desc: libfabric_sys::fi_profile_desc) -> Self {
        let profile_data = if c_profile_desc.datatype_sel
            == libfabric_sys::fi_profile_type_fi_primitive_type
        {
            unsafe {
                ProfileDataType::Primitive(DataType::from_raw(c_profile_desc.datatype.primitive))
            }
        } else {
            unsafe { ProfileDataType::Defined(Type::from_raw(c_profile_desc.datatype.defined)) }
        };

        Self {
            id: c_profile_desc.id,
            flags: c_profile_desc.flags,
            profile_data,
            name: unsafe { CStr::from_ptr(c_profile_desc.name).to_str().unwrap() },
            desc: unsafe { CStr::from_ptr(c_profile_desc.desc).to_str().unwrap() },
        }
    }

    /// Returns the ID of the profile.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the flags of the profile.
    pub fn flags(&self) -> u64 {
        self.flags
    }

    /// Returns the profile data of the profile.
    pub fn profile_data(&self) -> &ProfileDataType {
        &self.profile_data
    }

    /// Returns the name of the profile.
    pub fn name(&self) -> &str {
        self.name
    }

    /// Returns the description of the profile.
    pub fn desc(&self) -> &str {
        self.desc
    }
}
