use std::{cell::RefCell, collections::HashMap, thread::panicking};
use std::sync::RwLock;

use crate::pmi::{EncDec, ErrorKind, Pmi, PmiError};
macro_rules! check_error {
    ($err_code: expr) => {
        if  $err_code as u32 != pmi2_sys::PMI2_SUCCESS {
            return Err(PmiError::from_pmi2_err_code($err_code));
        }
    };
}

pub struct Pmi2 {
    my_rank: usize,
    ranks: Vec<usize>,
    finalize: bool,
    singleton_kvs: RwLock<HashMap<String, Vec<u8> >>
}

impl Pmi2  {
    pub fn new() -> Result<Self, crate::pmi::PmiError> {
        let mut rank = 0;
        let mut size = 0;
        let mut spawned = 0;
        let mut appnum = 0;
        let finalize;

        if  unsafe{ pmi2_sys::PMI2_Initialized() } == 0 {
            check_error!(unsafe { pmi2_sys::PMI2_Init(&mut spawned, &mut size, &mut rank, &mut appnum)});
            finalize = true;
        } else {
            panic!("PMI2 is already initialized. Cannot retrieve environment");
        }

        Ok(Pmi2{
            my_rank: rank as usize,
            ranks: (0..size as usize).collect(),
            finalize,
            singleton_kvs: RwLock::new(HashMap::new())
        })
    }

    fn get_singleton(&self, key: &str) -> Result<Vec<u8>, PmiError> {
       
        if let Some(data) = self.singleton_kvs.read().unwrap().get(key) {
            Ok(data.clone())
        }
        else {
            Err(PmiError{c_err: pmi2_sys::PMI2_ERR_INVALID_KEY as i32, kind: ErrorKind::InvalidKey})
        }
    }

    fn put_singleton(&self, key: &str, value: &[u8]) -> Result<(), PmiError>{
        
        self.singleton_kvs.write().unwrap().insert(key.to_owned(), value.to_vec());

        Ok(())
    }


}
impl EncDec for Pmi2 {}

impl  Pmi for Pmi2 {
    fn rank(&self) -> usize {
        self.my_rank
    }

    fn ranks(&self) -> &[usize] {
        &self.ranks
    }

    fn get(&self, key: &str, len: &usize, rank: &usize) -> Result<Vec<u8>, PmiError> {
        if self.ranks.len() > 1 {
            let kvs_key = std::ffi::CString::new(format!("rlibfab-{}-{}",rank,key)).unwrap().into_raw();
            let mut kvs_val: Vec<u8> = vec![0; 2 * len + 1];
            let mut len = 0;
            check_error!(unsafe { pmi2_sys::PMI2_KVS_Get(std::ptr::null(), pmi2_sys::PMI2_ID_NULL ,kvs_key, kvs_val.as_mut_ptr().cast(), kvs_val.len() as i32, &mut len) });

            Ok(self.decode(&kvs_val))
        }
        else {
            self.get_singleton(key)
        }
    }

    fn put(&self, key: &str, value: &[u8]) -> Result<(), PmiError> {
        if self.ranks.len() > 1 {
            let kvs_key = std::ffi::CString::new(format!("rlibfab-{}-{}",self.my_rank, key)).unwrap().into_raw();
            let kvs_val = self.encode(value);

            check_error!( unsafe { pmi2_sys::PMI2_KVS_Put(kvs_key, kvs_val.as_ptr().cast()) });
        }
        else {
            self.put_singleton(key, value)?;
        }
        Ok(())
    }

    fn exchange(&self) -> Result<(), PmiError> {
        if self.ranks.len() > 1 {
            check_error!(unsafe{ pmi2_sys::PMI2_KVS_Fence() });
        }

        Ok(())
    }
}

impl Drop for Pmi2 {
    fn drop(&mut self) {
        if self.finalize {
            let err = unsafe{ pmi2_sys::PMI2_Finalize() } as u32;
            if err != pmi2_sys::PMI2_SUCCESS && ! panicking() {
                panic!("{:?}", PmiError::from_pmi2_err_code(err as i32 ))
            }
        }
    }
}

impl PmiError {
    pub(crate) fn from_pmi2_err_code(c_err: i32) -> Self {

        let kind;
        if c_err == pmi2_sys::PMI2_FAIL {
            kind = ErrorKind::OperationFailed;
        }
        else{
            let c_err = c_err as u32;
            kind = match c_err {
                pmi2_sys::PMI2_ERR_INIT => ErrorKind::NotInitialized,
                pmi2_sys::PMI2_ERR_NOMEM => ErrorKind::NoBufSpaceAvailable,
                pmi2_sys::PMI2_ERR_INVALID_ARG => ErrorKind::InvalidArg,
                pmi2_sys::PMI2_ERR_INVALID_KEY => ErrorKind::InvalidKey,
                pmi2_sys::PMI2_ERR_INVALID_KEY_LENGTH => ErrorKind::InvalidKeyLength,
                pmi2_sys::PMI2_ERR_INVALID_VAL => ErrorKind::InvalidVal,
                pmi2_sys::PMI2_ERR_INVALID_VAL_LENGTH => ErrorKind::InvalidValLength,
                pmi2_sys::PMI2_ERR_INVALID_LENGTH => ErrorKind::InvalidLength,
                pmi2_sys::PMI2_ERR_INVALID_NUM_ARGS => ErrorKind::InvalidNumArgs,
                pmi2_sys::PMI2_ERR_INVALID_ARGS => ErrorKind::InvalidArgs,
                pmi2_sys::PMI2_ERR_INVALID_NUM_PARSED => ErrorKind::InvalidNumParsed,
                pmi2_sys::PMI2_ERR_INVALID_KEYVALP => ErrorKind::InvalidKeyValP,
                pmi2_sys::PMI2_ERR_INVALID_SIZE => ErrorKind::InvalidSize,
                _ => ErrorKind::Other,
            };
        }

        Self {c_err: c_err as i32, kind}
    }
}