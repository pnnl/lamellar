use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    env,
    hash::{Hash, Hasher},
    sync::RwLock,
    thread::panicking,
};

use libc::{c_char, size_t};

use crate::pmi::{numeric_job_id, EncDec, ErrorKind, Pmi, PmiError};
macro_rules! check_error {
    ($err_code: expr) => {
        if $err_code as u32 != pmi2_sys::PMI2_SUCCESS {
            return Err(PmiError::from_pmi2_err_code($err_code));
        }
    };
}

/// PMI2 backend implementation.
///
/// Detailed behavior and notes:
///
/// - Initialization: call `Pmi2::new()` which performs `PMI2_Init` and
///   collects the job size and rank information. The implementation finalizes
///   the runtime on `Drop` when appropriate.
/// - Node detection: identical host-exchange strategy to the PMI1 backend is
///   used when the runtime does not provide a stable node index. Hostnames
///   are published under `rpmi2-host-<rank>` and collected to build
///   contiguous node indices.
/// - Job id: for PMI2 the backend queries `PMI2_Job_Get`. If that returns an
///   empty value the backend falls back to a generated deterministic id.
/// - KVS behavior: uses `PMI2_KVS_Get`/`PMI2_KVS_Put` and issues
///   `PMI2_KVS_Fence`/commit operations (also used for `exchange` and
///   `barrier`) to ensure visibility.
/// - Singleton mode: a local in-memory store is used for single-rank runs.
/// - Caveats: as with PMI1, PMI2 runtimes may impose ordering or concurrency
///   constraints; callers should treat PMI operations as collective/fence
///   paired operations and avoid concurrent PMI calls from multiple threads
///   unless the runtime documents thread-safety.
pub struct Pmi2 {
    my_rank: usize,
    node_id: usize,
    host_name: String,
    ranks: Vec<usize>,
    finalize: bool,
    singleton_kvs: RwLock<HashMap<String, Vec<u8>>>,
}

impl Pmi2 {
    const HOST_NAME_KEY: &'static str = "rpmi2-host";
    const HOST_NAME_BUF: usize = 256;

    /// Initialize the PMI2 backend and return a `Pmi2` instance.
    ///
    /// Performs PMI2 initialization and collects rank/node metadata. Returns a
    /// `PmiError` on failure.
    pub fn new() -> Result<Self, crate::pmi::PmiError> {
        let mut rank = 0;
        let mut size = 0;
        let mut spawned = 0;
        let mut appnum = 0;
        let finalize = if unsafe { pmi2_sys::PMI2_Initialized() } == 0 {
            check_error!(unsafe {
                pmi2_sys::PMI2_Init(&mut spawned, &mut size, &mut rank, &mut appnum)
            });
            true
        } else {
            panic!("PMI2 is already initialized. Cannot retrieve environment");
        };

        let host_name = Self::local_hostname();

        let mut pmi = Pmi2 {
            my_rank: rank as usize,
            node_id: 0,
            host_name,
            ranks: (0..size as usize).collect(),
            finalize,
            singleton_kvs: RwLock::new(HashMap::new()),
        };

        pmi.init_node_info()?;

        Ok(pmi)
    }

    fn init_node_info(&mut self) -> Result<(), crate::pmi::PmiError> {
        if self.ranks.len() <= 1 {
            self.node_id = 0;
            return Ok(());
        }

        let host_key = Self::HOST_NAME_KEY;
        let my_key = format!("{host_key}-{}", self.my_rank);
        let host_buf = Self::host_buf_from_name(&self.host_name);
        self.put(my_key.as_str(), &host_buf)?;
        self.exchange()?;

        let mut hosts = Vec::with_capacity(self.ranks.len());
        for rank in 0..self.ranks.len() {
            let key = format!("{host_key}-{rank}");
            let raw = self.get(key.as_str(), &Self::HOST_NAME_BUF, &rank)?;
            hosts.push(Self::decode_host_name(raw));
        }

        let mut unique_hosts = hosts.clone();
        unique_hosts.sort();
        unique_hosts.dedup();

        self.node_id = unique_hosts
            .iter()
            .position(|h| h == &self.host_name)
            .unwrap_or(0);

        Ok(())
    }

    fn host_buf_from_name(name: &str) -> [u8; Self::HOST_NAME_BUF] {
        let mut buf = [0u8; Self::HOST_NAME_BUF];
        let bytes = name.as_bytes();
        let copy_len = bytes.len().min(Self::HOST_NAME_BUF);
        buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
        buf
    }

    fn decode_host_name(mut raw: Vec<u8>) -> String {
        if let Some(pos) = raw.iter().position(|&b| b == 0) {
            raw.truncate(pos);
        }
        String::from_utf8_lossy(&raw).into_owned()
    }

    fn local_hostname() -> String {
        let mut buf = [0u8; Self::HOST_NAME_BUF];
        let ret = unsafe {
            libc::gethostname(
                buf.as_mut_ptr() as *mut c_char,
                Self::HOST_NAME_BUF as size_t,
            )
        };

        if ret == 0 {
            if let Some(pos) = buf.iter().position(|&b| b == 0) {
                return String::from_utf8_lossy(&buf[..pos]).into_owned();
            }
            return String::from_utf8_lossy(&buf).into_owned();
        }

        env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string())
    }

    fn get_singleton(&self, key: &str) -> Result<Vec<u8>, PmiError> {
        if let Some(data) = self.singleton_kvs.read().unwrap().get(key) {
            Ok(data.clone())
        } else {
            Err(PmiError {
                c_err: pmi2_sys::PMI2_ERR_INVALID_KEY as i32,
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
impl EncDec for Pmi2 {}

impl Pmi for Pmi2 {
    fn rank(&self) -> usize {
        self.my_rank
    }

    fn node(&self) -> usize {
        self.node_id
    }

    fn num_nodes(&self) -> usize {
        if self.ranks.len() <= 1 {
            return 1;
        }

        let key = "node";
        let my_bytes = (self.node_id as u32).to_le_bytes();
        let _ = self.put(key, &my_bytes);
        let _ = self.exchange();

        let mut set = HashSet::new();
        for r in 0..self.ranks.len() {
            if let Ok(v) = self.get(key, &4usize, &r) {
                if v.len() >= 4 {
                    let mut arr = [0u8; 4];
                    arr.copy_from_slice(&v[..4]);
                    let nid = u32::from_le_bytes(arr) as usize;
                    set.insert(nid);
                }
            }
        }

        set.len()
    }

    fn ranks_on_node(&self, node: usize) -> Vec<usize> {
        if self.ranks.len() <= 1 {
            return vec![0usize];
        }

        let key = "node";
        let my_bytes = (self.node_id as u32).to_le_bytes();
        let _ = self.put(key, &my_bytes);
        let _ = self.exchange();

        let mut res = Vec::new();
        for r in 0..self.ranks.len() {
            if let Ok(v) = self.get(key, &4usize, &r) {
                if v.len() >= 4 {
                    let mut arr = [0u8; 4];
                    arr.copy_from_slice(&v[..4]);
                    let nid = u32::from_le_bytes(arr) as usize;
                    if nid == node {
                        res.push(r);
                    }
                }
            }
        }

        res
    }

    fn ranks(&self) -> &[usize] {
        &self.ranks
    }

    fn node_map(&self) -> HashMap<usize, Vec<usize>> {
        if self.ranks.len() <= 1 {
            let mut m = HashMap::new();
            m.insert(0usize, vec![0usize]);
            return m;
        }

        let key = "node";
        let my_bytes = (self.node_id as u32).to_le_bytes();
        let _ = self.put(key, &my_bytes);
        let _ = self.exchange();

        let mut map: HashMap<usize, Vec<usize>> = HashMap::new();
        for r in 0..self.ranks.len() {
            if let Ok(v) = self.get(key, &4usize, &r) {
                if v.len() >= 4 {
                    let mut arr = [0u8; 4];
                    arr.copy_from_slice(&v[..4]);
                    let nid = u32::from_le_bytes(arr) as usize;
                    map.entry(nid).or_default().push(r);
                }
            }
        }

        map
    }

    fn job_id_str(&self) -> String {
        if self.ranks.len() <= 1 {
            return String::new();
        }

        let mut buf: Vec<u8> = vec![0; 1024];
        let rc = unsafe { pmi2_sys::PMI2_Job_GetId(buf.as_mut_ptr() as *mut i8, buf.len() as i32) };
        if rc != 0 {
            return String::new();
        }

        let cstr = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr() as *const i8) };
        cstr.to_string_lossy().into_owned()
    }

    fn job_id(&self) -> usize {
        let s = self.job_id_str();
        if let Some(n) = numeric_job_id(&s) {
            return n;
        }
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish() as usize
    }

    /// Retrieve a value published by `rank` under `key`.
    ///
    /// Uses the PMI2 KVS for multi-rank runs or a local singleton store for
    /// single-rank execution.
    fn get(&self, key: &str, len: &usize, rank: &usize) -> Result<Vec<u8>, PmiError> {
        if self.ranks.len() > 1 {
            let kvs_key = std::ffi::CString::new(format!("rlibfab-{}-{}", rank, key))
                .unwrap()
                .into_raw();
            let mut kvs_val: Vec<u8> = vec![0; 2 * len + 1];
            let mut len = 0;
            check_error!(unsafe {
                pmi2_sys::PMI2_KVS_Get(
                    std::ptr::null(),
                    pmi2_sys::PMI2_ID_NULL,
                    kvs_key,
                    kvs_val.as_mut_ptr().cast(),
                    kvs_val.len() as i32,
                    &mut len,
                )
            });

            Ok(self.decode(&kvs_val))
        } else {
            self.get_singleton(key)
        }
    }

    /// Publish `value` under `key` for this rank.
    ///
    /// Encodes `value` for KVS storage. Calls the PMI2 fence/commit as
    /// required to make the value visible to peers.
    fn put(&self, key: &str, value: &[u8]) -> Result<(), PmiError> {
        if self.ranks.len() > 1 {
            let kvs_key = std::ffi::CString::new(format!("rlibfab-{}-{}", self.my_rank, key))
                .unwrap()
                .into_raw();
            let kvs_val = self.encode(value);

            check_error!(unsafe { pmi2_sys::PMI2_KVS_Put(kvs_key, kvs_val.as_ptr().cast()) });
            check_error!(unsafe { pmi2_sys::PMI2_KVS_Fence() });
        } else {
            self.put_singleton(key, value)?;
        }
        Ok(())
    }

    /// Ensure recent `put` operations are visible to other ranks.
    fn exchange(&self) -> Result<(), PmiError> {
        if self.ranks.len() > 1 {
            check_error!(unsafe { pmi2_sys::PMI2_KVS_Fence() });
        }

        Ok(())
    }

    /// Global barrier across all ranks. `collect_data` is currently ignored
    /// by the PMI2 backend.
    fn barrier(&self, _collect_data: bool) -> Result<(), PmiError> {
        if self.ranks.len() > 1 {
            check_error!(unsafe { pmi2_sys::PMI2_KVS_Fence() });
        }
        Ok(())
    }
}

impl Drop for Pmi2 {
    fn drop(&mut self) {
        if self.finalize {
            let err = unsafe { pmi2_sys::PMI2_Finalize() } as u32;
            if err != pmi2_sys::PMI2_SUCCESS && !panicking() {
                panic!("{:?}", PmiError::from_pmi2_err_code(err as i32))
            }
        }
    }
}

impl PmiError {
    pub(crate) fn from_pmi2_err_code(c_err: i32) -> Self {
        let kind = if c_err == pmi2_sys::PMI2_FAIL {
            ErrorKind::OperationFailed
        } else {
            let c_err = c_err as u32;
            match c_err {
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
            }
        };

        Self { c_err, kind }
    }
}
