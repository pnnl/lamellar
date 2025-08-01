use std::{cell::RefCell, collections::HashMap, thread::panicking};

use crate::pmi::{EncDec, ErrorKind, Pmi, PmiError};
macro_rules! check_error {
    ($err_code: expr) => {
        if  $err_code as u32 != pmi_sys::PMI_SUCCESS {
            return Err(PmiError::from_pmi1_err_code($err_code));
        }
    };
}

pub struct Pmi1 {
    my_rank: usize,
    ranks: Vec<usize>,
    finalize: bool,
    kvs_name: std::ffi::CString,
    singleton_kvs: RefCell<HashMap<String, Vec<u8> >>,
}

unsafe impl Sync for Pmi1 {}
unsafe impl Send for Pmi1 {}

impl Pmi1  {
    pub fn new() -> Result<Self, crate::pmi::PmiError> {
        let mut finalize = false;
        let mut init = 0;
        let mut spawned = 0;
        check_error!(unsafe { pmi_sys::PMI_Initialized(&mut init) });
        
        if init as u32 == pmi_sys::PMI_FALSE {
            check_error!(unsafe { pmi_sys::PMI_Init(&mut spawned as *mut i32)});
            finalize = true;
        }
        
        let mut size = 0;
        check_error!(unsafe{ pmi_sys::PMI_Get_size(&mut size)});
        
        let mut my_rank = 0;
        check_error!(unsafe{ pmi_sys::PMI_Get_rank(&mut my_rank)});
        
        let mut appnum = 0;
        check_error!(unsafe{ pmi_sys::PMI_Get_appnum(&mut appnum)});
        
        let mut max_name_len = 0;
        check_error!(unsafe {pmi_sys::PMI_KVS_Get_name_length_max(&mut max_name_len)});

        let mut name: Vec<u8> = vec![0; max_name_len as usize];
        check_error!(unsafe {pmi_sys::PMI_KVS_Get_my_name(name.as_mut_ptr() as *mut i8, max_name_len) });
                
        let c_kvs_name = std::ffi::CString::new(name);
        let kvs_name = match c_kvs_name {
            Err(error) => {let len: usize = error.nul_position(); std::ffi::CString::new(&error.into_vec()[..len]).unwrap() },
            Ok(rkvs_name) => rkvs_name,
        };

        Ok(
            Pmi1 {
                my_rank: my_rank as usize,
                ranks: (0..size as usize).collect(),
                finalize,
                kvs_name,
                singleton_kvs: RefCell::new(HashMap::new()),
            }
        )
    }

    fn get_singleton(&self, key: &str) -> Result<Vec<u8>, crate::pmi::PmiError> {
       
        if let Some(data) = self.singleton_kvs.borrow().get(key) {
            Ok(data.clone())
        }
        else {
            Err(crate::pmi::PmiError{c_err: pmi_sys::PMI_ERR_INVALID_KEY as i32, kind: ErrorKind::InvalidKey})
        }
    }

    fn put_singleton(&self, key: &str, value: &[u8]) -> Result<(), crate::pmi::PmiError> {
        
        self.singleton_kvs.borrow_mut().insert(key.to_owned(), value.to_vec());
        Ok(())
    }
}

impl EncDec for Pmi1 {}


impl Pmi for Pmi1 {
    fn rank(&self) -> usize {
        self.my_rank
    }

    fn ranks(&self) -> &[usize] {
        &self.ranks
    }

    fn get(&self, key: &str, len: &usize, rank: &usize) -> Result<Vec<u8>, crate::pmi::PmiError> {
        if self.ranks.len() > 1 {
            let kvs_key = std::ffi::CString::new(format!("rlibfab-{}-{}",rank,key)).unwrap().into_raw();
            let mut kvs_val: Vec<u8> = vec![0; 2 * len + 1];
            
            check_error!(unsafe { pmi_sys::PMI_KVS_Get(self.kvs_name.as_ptr() ,kvs_key, kvs_val.as_mut_ptr() as *mut i8, kvs_val.len() as i32) });
            Ok(self.decode(&kvs_val))
        } else {
            self.get_singleton(key)
        }
    }

    fn put(&self, key: &str, value: &[u8]) -> Result<(), crate::pmi::PmiError> {
        if self.ranks.len() > 1 {
            let kvs_key = std::ffi::CString::new(format!("rlibfab-{}-{}",self.my_rank,key)).unwrap().into_raw();
            let kvs_val = self.encode(value);
            
            check_error!(unsafe { pmi_sys::PMI_KVS_Put(self.kvs_name.as_ptr(), kvs_key, kvs_val.as_ptr() as *const i8) });
            check_error!(unsafe{ pmi_sys::PMI_KVS_Commit(self.kvs_name.as_ptr()) });
            Ok(())
        } else {
            self.put_singleton(key, value)   
        }
    }

    fn exchange(&self) -> Result<(), crate::pmi::PmiError> {
        check_error!(unsafe{ pmi_sys::PMI_KVS_Commit(self.kvs_name.as_ptr()) });
        check_error!(unsafe{ pmi_sys::PMI_Barrier() });
        Ok(())
    }

    fn barrier(&self, collect_data: bool) -> Result<(), crate::pmi::PmiError> {
        if self.ranks.len() > 1 {
            check_error!(unsafe{ pmi_sys::PMI_Barrier() });
        }
        Ok(())
    }
}

impl Drop for Pmi1 {
    fn drop(&mut self) {
        if self.finalize {
            let err =  unsafe{ pmi_sys::PMI_Finalize()} as u32;
            if err != pmi_sys::PMI_SUCCESS && ! panicking() {
                panic!("{:?}", PmiError::from_pmi1_err_code(err as i32 ))
            }
        }
    }
}

impl PmiError {
    pub(crate) fn from_pmi1_err_code(c_err: i32) -> Self {

        let kind = if c_err == pmi_sys::PMI_FAIL {
            ErrorKind::OperationFailed
        }
        else{
            let c_err = c_err as u32;
            match c_err {
                pmi_sys::PMI_ERR_INIT => ErrorKind::NotInitialized,
                pmi_sys::PMI_ERR_NOMEM => ErrorKind::NoBufSpaceAvailable,
                pmi_sys::PMI_ERR_INVALID_ARG => ErrorKind::InvalidArg,
                pmi_sys::PMI_ERR_INVALID_KEY => ErrorKind::InvalidKey,
                pmi_sys::PMI_ERR_INVALID_KEY_LENGTH => ErrorKind::InvalidKeyLength,
                pmi_sys::PMI_ERR_INVALID_VAL => ErrorKind::InvalidVal,
                pmi_sys::PMI_ERR_INVALID_VAL_LENGTH => ErrorKind::InvalidValLength,
                pmi_sys::PMI_ERR_INVALID_LENGTH => ErrorKind::InvalidLength,
                pmi_sys::PMI_ERR_INVALID_NUM_ARGS => ErrorKind::InvalidNumArgs,
                pmi_sys::PMI_ERR_INVALID_ARGS => ErrorKind::InvalidArgs,
                pmi_sys::PMI_ERR_INVALID_NUM_PARSED => ErrorKind::InvalidNumParsed,
                pmi_sys::PMI_ERR_INVALID_KEYVALP => ErrorKind::InvalidKeyValP,
                pmi_sys::PMI_ERR_INVALID_SIZE => ErrorKind::InvalidSize,
                pmi_sys::PMI_ERR_INVALID_KVS => ErrorKind::InvalidKVS,
                _ => ErrorKind::Other,
            }
        };

        Self {c_err, kind}
    }
}
