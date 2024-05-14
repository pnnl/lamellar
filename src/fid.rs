use std::marker::PhantomData;

use crate::error;
pub(crate) type Fid = *mut libfabric_sys::fid;

#[derive(Clone)]
pub struct BorrowedFid<'a> {
    fid: Fid,
    phantom: PhantomData<&'a OwnedFid>
}

impl<'a> BorrowedFid<'a> {
    pub(crate) fn from(fid: &'a OwnedFid) -> Self {
        Self {
            fid: fid.fid,
            phantom: PhantomData,
        }
    }
}

pub(crate) struct OwnedFid {
    fid: Fid,
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

impl PartialEq for OwnedFid {
    fn eq(&self, other: &Self) -> bool {
        self.fid == other.fid
    }
}

impl AsFid for OwnedFid {
    fn as_fid(&self) -> BorrowedFid {
        BorrowedFid::from(self)
    }
}

// impl AsFid for OwnedFid {
//     fn as_fid(&self) -> *mut libfabric_sys::fid {
//         self.fid
//     }
// }

pub trait AsFid {
    fn as_fid(&self) -> BorrowedFid;
}

pub(crate) trait AsRawFid {
    fn as_raw_fid(&self) -> Fid;
}

impl<'a> AsRawFid for BorrowedFid<'a> {
    fn as_raw_fid(&self) -> Fid {
        self.fid
    }
}



// pub trait AsFid {
//     fn as_fid(&self) -> *mut libfabric_sys::fid;
// }

impl<T: AsFid> AsRawFid for T  {
    fn as_raw_fid(&self) -> Fid {
        self.as_fid().as_raw_fid()
    }
}