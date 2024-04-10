use crate::error;

pub(crate) struct OwnedFid {
    fid: *mut libfabric_sys::fid,
}

impl OwnedFid {
    pub(crate) fn from(fid: *mut libfabric_sys::fid) -> Self{
        OwnedFid{fid}
    }
}


impl Drop for OwnedFid {
    fn drop(&mut self) {
        let err = unsafe { libfabric_sys::inlined_fi_close(self.fid) };

        if err != 0 {
            panic!("{}", error::Error::from_err_code((-err).try_into().unwrap()));
        }
    }
}

impl AsFid for OwnedFid {
    fn as_fid(&self) -> *mut libfabric_sys::fid {
        self.fid
    }
}

pub trait AsFid {
    fn as_fid(&self) -> *mut libfabric_sys::fid;
}