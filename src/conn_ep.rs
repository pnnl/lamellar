use crate::{ep::Address, fid::{AsFid, AsRawFid, AsRawTypedFid, EpRawFid, OwnedEpFid}, utils::check_error};

pub struct ConnectionOrientedEndPoint {
    fid: OwnedEpFid
}

impl AsRawFid for ConnectionOrientedEndPoint {
    fn as_raw_fid(&self) -> crate::fid::RawFid {
        self.fid.as_raw_fid()
    }
}

impl AsFid for ConnectionOrientedEndPoint {
    fn as_fid(&self) -> crate::fid::BorrowedFid {
        self.fid.as_fid()
    }
}

impl AsRawTypedFid for ConnectionOrientedEndPoint {
    type Output = EpRawFid;

    fn as_raw_typed_fid(&self) -> Self::Output {
        self.fid.as_raw_typed_fid()
    }
}

impl ConnectionOrientedEndPoint {
    pub fn connect_with<T>(&self, addr: &Address, param: &[T]) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_connect(self.as_raw_typed_fid(), addr.as_bytes().as_ptr().cast(), param.as_ptr().cast(), param.len()) };
        
        check_error(err.try_into().unwrap())
    }

    pub fn connect(&self, addr: &Address) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_connect(self.as_raw_typed_fid(), addr.as_bytes().as_ptr().cast(), std::ptr::null_mut(), 0) };

        check_error(err.try_into().unwrap())
    }

    pub fn accept_with<T0>(&self, param: &[T0]) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_accept(self.as_raw_typed_fid(), param.as_ptr().cast(), param.len()) };
        
        check_error(err.try_into().unwrap())
    }

    pub fn accept(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_accept(self.as_raw_typed_fid(), std::ptr::null_mut(), 0) };
        
        check_error(err.try_into().unwrap())
    }

    pub fn shutdown(&self) -> Result<(), crate::error::Error> {
        let err = unsafe { libfabric_sys::inlined_fi_shutdown(self.as_raw_typed_fid(), 0) };

        check_error(err.try_into().unwrap())
    }

    pub fn peer(&self) -> Result<Address, crate::error::Error> {
        let mut len = 0;
        let err = unsafe { libfabric_sys::inlined_fi_getpeer(self.as_raw_typed_fid(), std::ptr::null_mut(), &mut len)};
        
        if -err as u32 ==  libfabric_sys::FI_ETOOSMALL{
            let mut address = vec![0; len];
            let err = unsafe { libfabric_sys::inlined_fi_getpeer(self.as_raw_typed_fid(), address.as_mut_ptr().cast(), &mut len)};
            if err != 0 {
                Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
            }
            else {
                Ok(Address{address})
            }
        }
        else {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
    }

    pub fn name(&self) -> Result<Address, crate::error::Error> {
        let mut len = 0;
        let err: i32 = unsafe { libfabric_sys::inlined_fi_getname(self.as_raw_fid(), std::ptr::null_mut(), &mut len) };
        if -err as u32  == libfabric_sys::FI_ETOOSMALL {
            let mut address = vec![0; len];
            let err: i32 = unsafe { libfabric_sys::inlined_fi_getname(self.as_raw_fid(), address.as_mut_ptr().cast(), &mut len) };
            if err < 0
            {
                Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
            }
            else 
            {
                Ok(Address{address})
            }
        }
        else
        {
            Err(crate::error::Error::from_err_code((-err).try_into().unwrap()))
        }
    }
}

