use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    sync::RwLock,
    thread::panicking,
};

use crate::pmi::{numeric_job_id, EncDec, ErrorKind, Pmi, PmiError};
macro_rules! check_error {
    ($err_code: expr) => {
        if $err_code as u32 != pmix_sys::PMIX_SUCCESS {
            return Err(PmiError::from_pmix_err_code($err_code));
        }
    };
}

/// PMIx backend implementation.
///
/// Notes and important details:
///
/// - Initialization: call `PmiX::new()` to perform `PMIx_Init` and to query
///   job size, rank and namespace information. The backend calls
///   `PMIx_Finalize` on `Drop` when it initially initialized the runtime.
/// - Node id mapping: PMIx exposes a `PMIX_NODEID` value which may be sparse
///   or non-contiguous; this backend collects raw PMIx node ids for all
///   ranks, sorts/deduplicates them and remaps them into contiguous node
///   indices (0..num_nodes()) so `node()` and `node_map()` are stable and
///   compact.
/// - Job id handling: PMIx may return a string job id or a numeric id in a
///   `uint64`; the backend converts numeric job ids to decimal strings and
///   prefers numeric strings (returned as `usize` by `job_id()`). Non-numeric
///   job ids are hashed deterministically to produce a `usize` job id.
/// - KVS semantics: uses `PMIx_Put` / `PMIx_Commit` and `PMIx_Get` to store
///   and retrieve values. `exchange()` is implemented via a `PMIx_Fence` for
///   multi-rank runs.
/// - Collect data: when `barrier(true)` is used the backend requests
///   `PMIX_COLLECT_DATA` and forwards the PMIx info structure to the fence
///   call; runtimes that support collect data may return additional per-rank
///   info.
/// - Singleton mode: behaves like other backends and uses an in-memory
///   store for single-rank operation.
///
/// Caveat: PMIx semantics vary between implementations — prefer testing the
/// crate under the target runtime used in production.
pub struct PmiX {
    my_rank: usize,
    node_id: usize,
    job_id_str: String,
    num_nodes: usize,
    node_map_store: HashMap<usize, Vec<usize>>,
    ranks: Vec<usize>,
    nspace: pmix_sys::pmix_nspace_t,
    finalize: bool,
    singleton_kvs: RwLock<HashMap<String, Vec<u8>>>,
}

impl EncDec for PmiX {}

impl PmiX {
    /// Initialize the PMIx backend and return a `PmiX` instance.
    ///
    /// Performs `PMIx_Init` and queries job/rank/node metadata. Returns a
    /// `PmiError` on failure.
    pub fn new() -> Result<Self, PmiError> {
        let finalize;
        let mut proc = pmix_sys::pmix_proc {
            nspace: [0; 256usize],
            rank: 0,
        };

        check_error!(unsafe { pmix_sys::PMIx_Init(&mut proc, std::ptr::null_mut(), 0) });
        finalize = true;
        let rank = proc.rank;
        let nspace = proc.nspace.clone();

        // get job size
        let wildcard = pmix_sys::pmix_proc {
            nspace: nspace.clone(),
            rank: pmix_sys::PMIX_RANK_WILDCARD,
        };
        let key = pmix_sys::PMIX_JOB_SIZE;
        let mut val: *mut pmix_sys::pmix_value_t = std::ptr::null_mut();
        check_error!(unsafe {
            pmix_sys::PMIx_Get(
                &wildcard,
                key.as_ptr() as *mut i8,
                std::ptr::null(),
                0,
                &mut val,
            )
        });
        let size = unsafe { (*val).data.uint32 };

        // get node rank for this process
        let myproc = pmix_sys::pmix_proc {
            nspace: nspace.clone(),
            rank,
        };
        let mut node_val: *mut pmix_sys::pmix_value_t = std::ptr::null_mut();
        check_error!(unsafe {
            pmix_sys::PMIx_Get(
                &myproc,
                pmix_sys::PMIX_NODEID.as_ptr() as *mut i8,
                std::ptr::null(),
                0,
                &mut node_val,
            )
        });
        let raw_node_id = unsafe { (*node_val).data.uint16 } as usize;

        // build node map and compute num_nodes
        let mut node_map_store_temp: HashMap<usize, Vec<usize>> = HashMap::new();
        if size <= 1 {
            node_map_store_temp.insert(0usize, vec![0usize]);
        } else {
            for r in 0..size as u32 {
                let proc_r = pmix_sys::pmix_proc {
                    nspace: nspace.clone(),
                    rank: r,
                };
                let mut nid_val: *mut pmix_sys::pmix_value_t = std::ptr::null_mut();
                let rc = unsafe {
                    pmix_sys::PMIx_Get(
                        &proc_r,
                        pmix_sys::PMIX_NODEID.as_ptr() as *mut i8,
                        std::ptr::null(),
                        0,
                        &mut nid_val,
                    )
                };

                if rc != pmix_sys::PMIX_SUCCESS as i32 || nid_val.is_null() {
                    continue;
                }

                let nid = unsafe { (*nid_val).data.uint16 } as usize;
                unsafe { pmix_sys::PMIx_Value_free(nid_val, 1) };
                node_map_store_temp
                    .entry(nid)
                    .or_insert_with(Vec::new)
                    .push(r as usize);
            }
        }

        let mut unique_node_ids: Vec<usize> = node_map_store_temp.keys().copied().collect();
        unique_node_ids.sort();
        unique_node_ids.dedup();

        let mut node_id_map: HashMap<usize, usize> = HashMap::new();
        for (index, nid) in unique_node_ids.iter().enumerate() {
            node_id_map.insert(*nid, index);
        }

        let mut node_map_store: HashMap<usize, Vec<usize>> = HashMap::new();
        for (actual_nid, ranks) in node_map_store_temp {
            if let Some(&mapped) = node_id_map.get(&actual_nid) {
                node_map_store.insert(mapped, ranks);
            }
        }

        let num_nodes = node_map_store.len();
        let mapped_node_id = node_id_map.get(&raw_node_id).copied().unwrap_or(0);

        // compute job id now and store it (string)
        let job_id = if size <= 1 {
            String::new()
        } else {
            let proc_j = pmix_sys::pmix_proc {
                nspace: nspace.clone(),
                rank: pmix_sys::PMIX_RANK_WILDCARD,
            };
            let key_job = pmix_sys::PMIX_JOBID;
            let mut job_val: *mut pmix_sys::pmix_value_t = std::ptr::null_mut();
            let rc = unsafe {
                pmix_sys::PMIx_Get(
                    &proc_j,
                    key_job.as_ptr() as *mut i8,
                    std::ptr::null(),
                    0,
                    &mut job_val,
                )
            };

            if rc != pmix_sys::PMIX_SUCCESS as i32 || job_val.is_null() {
                String::new()
            } else {
                let out = unsafe {
                    if !(*job_val).data.string.is_null() {
                        let s = std::ffi::CStr::from_ptr((*job_val).data.string)
                            .to_string_lossy()
                            .into_owned();
                        pmix_sys::PMIx_Value_free(job_val, 1);
                        s
                    } else {
                        let v = (*job_val).data.uint64 as u64;
                        pmix_sys::PMIx_Value_free(job_val, 1);
                        format!("{}", v)
                    }
                };
                out
            }
        };

        Ok(PmiX {
            my_rank: rank as usize,
            node_id: mapped_node_id,
            job_id_str: job_id,
            num_nodes,
            node_map_store,
            ranks: (0..size as usize).collect(),
            nspace: proc.nspace,
            finalize,
            singleton_kvs: RwLock::new(HashMap::new()),
        })
    }

    fn get_singleton(&self, key: &str) -> Result<Vec<u8>, PmiError> {
        if let Some(data) = self.singleton_kvs.read().unwrap().get(key) {
            Ok(data.clone())
        } else {
            Err(PmiError {
                c_err: pmix_sys::PMIX_ERR_INVALID_KEY as i32,
                kind: ErrorKind::InvalidKey,
            })
        }
    }

    fn put_singleton(&self, key: &str, value: &[u8]) -> Result<(), PmiError> {
        self.singleton_kvs
            .write()
            .unwrap()
            .insert(key.to_owned(), value.to_vec());
        Ok(())
    }
}

impl Pmi for PmiX {
    fn rank(&self) -> usize {
        self.my_rank
    }

    fn node(&self) -> usize {
        self.node_id
    }

    fn num_nodes(&self) -> usize {
        self.num_nodes
    }

    fn ranks_on_node(&self, node: usize) -> Vec<usize> {
        if let Some(v) = self.node_map_store.get(&node) {
            v.clone()
        } else {
            Vec::new()
        }
    }

    fn ranks(&self) -> &[usize] {
        &self.ranks
    }

    fn node_map(&self) -> HashMap<usize, Vec<usize>> {
        self.node_map_store.clone()
    }

    /// Return the raw job id string for this job as provided by PMIx or the
    /// backend fallback.
    fn job_id_str(&self) -> String {
        self.job_id_str.clone()
    }

    /// Return a deterministic numeric job id. Prefers numeric PMIx job ids
    /// when available and hashes otherwise.
    fn job_id(&self) -> usize {
        let s = self.job_id_str();
        if let Some(n) = numeric_job_id(&s) {
            return n;
        }
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish() as usize
    }

    /// Retrieve a value published by `rank` under `key` using PMIx KVS
    /// semantics. For singleton runs the local store is used instead.
    fn get(&self, key: &str, _len: &usize, rank: &usize) -> Result<Vec<u8>, PmiError> {
        if self.ranks.len() > 1 {
            let proc = pmix_sys::pmix_proc {
                nspace: self.nspace.clone(),
                rank: *rank as u32,
            };
            let mut recv_val: *mut pmix_sys::pmix_value = std::ptr::null_mut();
            let mut val: pmix_sys::pmix_value = pmix_sys::pmix_value {
                type_: 0,
                data: pmix_sys::pmix_value__bindgen_ty_1 {
                    string: std::ptr::null_mut(),
                },
            };

            let kvs_key = std::ffi::CString::new(format!("rlibfab-{}-{}", rank, key))
                .unwrap()
                .into_raw();

            check_error!(unsafe {
                pmix_sys::PMIx_Get(&proc, kvs_key, std::ptr::null(), 0, &mut recv_val)
            });
            unsafe { pmix_sys::PMIx_Value_xfer(&mut val, recv_val) };
            let byte_array =
                unsafe { std::ffi::CStr::from_ptr(val.data.string) }.to_bytes_with_nul();

            Ok(self.decode(&byte_array))
        } else {
            self.get_singleton(key)
        }
    }

    /// Publish `value` under `key` for this rank using PMIx put/commit.
    fn put(&self, key: &str, value: &[u8]) -> Result<(), PmiError> {
        if self.ranks.len() > 1 {
            let mut val: pmix_sys::pmix_value = pmix_sys::pmix_value {
                type_: 0,
                data: pmix_sys::pmix_value__bindgen_ty_1 {
                    string: std::ptr::null_mut(),
                },
            };
            let kvs_key = std::ffi::CString::new(format!("rlibfab-{}-{}", self.my_rank, key))
                .unwrap()
                .into_raw();
            let mut kvs_val = self.encode(value);

            unsafe {
                pmix_sys::PMIx_Value_load(
                    &mut val,
                    kvs_val.as_mut_ptr() as *mut std::ffi::c_void,
                    pmix_sys::PMIX_STRING as u16,
                )
            };
            check_error!(unsafe {
                pmix_sys::PMIx_Put(pmix_sys::PMIX_GLOBAL as u8, kvs_key, &mut val)
            });
            check_error!(unsafe { pmix_sys::PMIx_Commit() });
            // check_error!(unsafe { pmix_sys::PMIx_Commit() });
        } else {
            self.put_singleton(key, value)?;
        }
        Ok(())
    }

    /// Ensure recent `put` operations are visible to other ranks. Implemented
    /// via a PMIx fence for multi-rank jobs.
    fn exchange(&self) -> Result<(), PmiError> {
        if self.ranks.len() > 1 {
            // check_error!(unsafe { pmix_sys::PMIx_Progress() });
            self.barrier(true)?;
        }

        Ok(())
    }

    /// Perform a PMIx fence across the job. When `collect_data` is true the
    /// backend requests PMIX_COLLECT_DATA from the runtime and passes it back
    /// via PMIx info structures.
    fn barrier(&self, collect_data: bool) -> Result<(), PmiError> {
        if self.ranks.len() > 1 {
            if collect_data {
                let mut cd: std::mem::MaybeUninit<pmix_sys::pmix_info_t> =
                    std::mem::MaybeUninit::uninit();
                unsafe { pmix_sys::PMIx_Info_construct(cd.as_mut_ptr()) };
                let mut cd = unsafe { cd.assume_init() };
                unsafe {
                    pmix_sys::PMIx_Info_load(
                        &mut cd,
                        pmix_sys::PMIX_COLLECT_DATA as *const _ as *const _,
                        &collect_data as *const _ as *const _,
                        pmix_sys::PMIX_BOOL as u16,
                    )
                };

                check_error!(unsafe { pmix_sys::PMIx_Fence(std::ptr::null(), 0, &mut cd, 1) });
            } else {
                check_error!(unsafe {
                    pmix_sys::PMIx_Fence(std::ptr::null(), 0, std::ptr::null(), 0)
                });
            }
        }
        Ok(())
    }
}

impl Drop for PmiX {
    fn drop(&mut self) {
        if self.finalize {
            let err = unsafe { pmix_sys::PMIx_Finalize(std::ptr::null(), 0) } as u32;
            if err != pmix_sys::PMIX_SUCCESS && !panicking() {
                panic!("{:?}", PmiError::from_pmix_err_code(err as i32))
            }
        }
    }
}

impl PmiError {
    pub(crate) fn from_pmix_err_code(c_err: i32) -> Self {
        let kind;
        if c_err == pmix_sys::PMIX_ERROR {
            kind = ErrorKind::OperationFailed;
        } else {
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

        Self {
            c_err: c_err as i32,
            kind,
        }
    }
}
