use std::{collections::HashMap, thread::panicking};
use std::sync::RwLock;


use crate::pmi::{EncDec, ErrorKind, Pmi, PmiError};
macro_rules! check_error {
    ($err_code: expr) => {
        if  $err_code as u32 != pmix_sys::PMIX_SUCCESS {
            return Err(PmiError::from_pmix_err_code($err_code));
        }
    };
}

pub struct PmiX {
    my_rank: usize,
    ranks: Vec<usize>,
    nspace: pmix_sys::pmix_nspace_t,
    finalize: bool,
    singleton_kvs: RwLock<HashMap<String, Vec<u8>>>,
}

impl EncDec for PmiX {}

impl PmiX {

    pub fn new() -> Result<Self, PmiError> {
            
        let finalize;
        let mut proc = pmix_sys::pmix_proc {
            nspace: [0; 256usize],
            rank: 0,
        };

        check_error!(unsafe { pmix_sys::PMIx_Init(&mut proc, std::ptr::null_mut(), 0)}) ;
        finalize = true;
        let rank = proc.rank;
        proc.rank = pmix_sys::PMIX_RANK_WILDCARD;
        let key = pmix_sys::PMIX_JOB_SIZE;
        let mut val: *mut pmix_sys::pmix_value_t = std::ptr::null_mut();
        check_error!(unsafe {pmix_sys::PMIx_Get(&proc, key.as_ptr() as *mut i8, std::ptr::null(), 0, &mut val)});
        let size = unsafe{(*val).data.uint32};

        Ok(PmiX {
            my_rank: rank as usize,
            ranks: (0..size as usize).collect(),
            nspace: proc.nspace,
            finalize,
            singleton_kvs: RwLock::new(HashMap::new()),
        })
    }

    fn get_singleton(&self, key: &str) -> Result<Vec<u8>, PmiError> {

        if let Some(data) = self.singleton_kvs.read().unwrap().get(key) {
            Ok(data.clone())
        }
        else {
            Err(PmiError{c_err: pmix_sys::PMIX_ERR_INVALID_KEY as i32, kind: ErrorKind::InvalidKey})
        }
    }

    fn put_singleton(&self, key: &str, value: &[u8]) -> Result<(), PmiError>{
        self.singleton_kvs.write().unwrap().insert(key.to_owned(), value.to_vec());
        Ok(())
    }
}

impl Pmi for PmiX {
    
    fn rank(&self) -> usize {
        self.my_rank
    }

    fn ranks(&self) -> &[usize] {
        &self.ranks
    }

    fn get(&self, key: &str, len: &usize, rank: &usize) -> Result<Vec<u8>, PmiError> {
        if self.ranks.len() > 1 {
            let proc = pmix_sys::pmix_proc {
                nspace: self.nspace.clone(),
                rank: *rank as u32,
            };
            let mut recv_val: *mut pmix_sys::pmix_value = std::ptr::null_mut();
            let mut val: pmix_sys::pmix_value = pmix_sys::pmix_value {
                type_ : 0,
                data: pmix_sys::pmix_value__bindgen_ty_1{
                    string: std::ptr::null_mut(),
                },
            };

            let kvs_key = std::ffi::CString::new(format!("rlibfab-{}-{}",rank,key)).unwrap().into_raw();
            
            check_error!(unsafe { pmix_sys::PMIx_Get(&proc, kvs_key, std::ptr::null(), 0, &mut recv_val) });
            unsafe{pmix_sys::PMIx_Value_xfer(&mut val, recv_val)};
            let byte_array = unsafe {std::ffi::CStr::from_ptr(val.data.string)}.to_bytes_with_nul();

            Ok(self.decode(&byte_array))
        }
        else {
            self.get_singleton(key)
        }
    }

    fn put(&self, key: &str, value: &[u8]) -> Result<(), PmiError> {
        if self.ranks.len() > 1 {
            let mut val: pmix_sys::pmix_value = pmix_sys::pmix_value {
                type_ : 0,
                data: pmix_sys::pmix_value__bindgen_ty_1{
                    string: std::ptr::null_mut(),
                },
            };
            let kvs_key = std::ffi::CString::new(format!("rlibfab-{}-{}",self.my_rank, key)).unwrap().into_raw();
            let mut kvs_val = self.encode(value);
            
            unsafe{pmix_sys::PMIx_Value_load(&mut val, kvs_val.as_mut_ptr() as *mut std::ffi::c_void,  pmix_sys::PMIX_STRING as u16)};
            check_error!(unsafe { pmix_sys::PMIx_Put(pmix_sys::PMIX_GLOBAL as u8, kvs_key, &mut val) });
            check_error!(unsafe{ pmix_sys::PMIx_Commit()});
        }
        else {
            self.put_singleton(key, value)?;
        }
        Ok(())
    }

    fn exchange(&self) -> Result<(), PmiError> {
        if self.ranks.len() > 1 {

            check_error!(unsafe{ pmix_sys::PMIx_Commit()});
        }

        Ok(())
    }
}

impl Drop for PmiX {
    fn drop(&mut self) {
        if self.finalize {

            let err = unsafe{ pmix_sys::PMIx_Finalize(std::ptr::null(), 0) } as u32;
            if err != pmix_sys::PMIX_SUCCESS && ! panicking() {
                panic!("{:?}", PmiError::from_pmix_err_code(err as i32 ))
            }
        }
    }
}


impl PmiError {
    pub(crate) fn from_pmix_err_code(c_err: i32) -> Self {

        let kind;
        if c_err == pmix_sys::PMIX_ERROR {
            kind = ErrorKind::OperationFailed;
        }
        else{
            kind = match c_err {
                pmix_sys::PMIX_ERR_INIT => ErrorKind::NotInitialized,
                pmix_sys::PMIX_ERR_NOMEM => ErrorKind::NoBufSpaceAvailable,
                pmix_sys::PMIX_ERR_INVALID_ARG => ErrorKind::InvalidArg,
                pmix_sys::PMIX_ERR_INVALID_KEY => ErrorKind::InvalidKey,
                pmix_sys::PMIX_ERR_INVALID_KEY_LENGTH => ErrorKind::InvalidKeyLength,
                pmix_sys::PMIX_ERR_INVALID_VAL => ErrorKind::InvalidVal,
                pmix_sys::PMIX_ERR_INVALID_VAL_LENGTH => ErrorKind::InvalidValLength,
                pmix_sys::PMIX_ERR_INVALID_LENGTH => ErrorKind::InvalidLength,
                pmix_sys::PMIX_ERR_INVALID_NUM_ARGS => ErrorKind::InvalidNumArgs,
                pmix_sys::PMIX_ERR_INVALID_ARGS => ErrorKind::InvalidArgs,
                pmix_sys::PMIX_ERR_INVALID_NUM_PARSED => ErrorKind::InvalidNumParsed,
                pmix_sys::PMIX_ERR_INVALID_KEYVALP => ErrorKind::InvalidKeyValP,
                pmix_sys::PMIX_ERR_INVALID_SIZE => ErrorKind::InvalidSize,
                _ => ErrorKind::Other,
            };
        }

        Self {c_err: c_err as i32, kind}
    }
}