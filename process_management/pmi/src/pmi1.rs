use libc::{c_char, size_t};
use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    env,
    hash::{Hash, Hasher},
    sync::RwLock,
    thread::panicking,
};

use crate::pmi::{numeric_job_id, EncDec, ErrorKind, Pmi, PmiError};
macro_rules! check_error {
    ($err_code: expr) => {
        if $err_code as u32 != pmi_sys::PMI_SUCCESS {
            return Err(PmiError::from_pmi1_err_code($err_code));
        }
    };
}

/// PMI1 backend implementation.
///
/// This struct implements the `Pmi` trait using a PMI1 runtime. Key
/// behaviors and caveats:
///
/// - Initialization: call `Pmi1::new()` to initialize the PMI runtime if
///   required; the constructor will call `PMI_Init` when needed and arrange
///   for `PMI_Finalize` on drop when appropriate.
/// - Node detection: some PMI1 deployments do not provide a stable node
///   index. In multi-rank runs each rank publishes its hostname (KVS key
///   `rpmi-host-<rank>`), then ranks collect, sort and deduplicate hostnames
///   to produce contiguous node indices (0..num_nodes()). This produces
///   deterministic node assignments across ranks on the same set of hosts.
/// - Job id resolution: the backend prefers job-manager environment
///   variables (SLURM_JOB_ID, PMI_JOBID, PMIX_JOBID, etc.) when present,
///   falls back to a `jobid` KVS entry when available, and finally generates
///   a deterministic fallback id (hash of the host list) for singleton or
///   poorly-instrumented runtimes.
/// - KVS format: application keys use the prefix `rlibfab-<rank>-<key>` to
///   avoid collisions with other users of the PMI KVS. Values are encoded
///   into an ASCII-safe representation by the crate before insertion.
/// - Singleton mode: when the job contains a single rank the crate uses an
///   in-memory singleton store rather than invoking PMI KVS calls; this
///   allows unit tests or single-process runs to exercise the same API.
/// - Thread-safety: `Pmi1` implements `Sync + Send` but underlying PMI
///   libraries may not be thread-safe; avoid calling PMI APIs concurrently
///   from multiple threads unless the runtime documents such support.
///
/// See the `Pmi` trait documentation for method semantics (`get`, `put`,
/// `exchange`, `barrier`).
pub struct Pmi1 {
    my_rank: usize,
    node_id: usize,
    host_name: String,
    generated_job_id: String,
    ranks: Vec<usize>,
    finalize: bool,
    kvs_name: std::ffi::CString,
    singleton_kvs: RwLock<HashMap<String, Vec<u8>>>,
}

unsafe impl Sync for Pmi1 {}
unsafe impl Send for Pmi1 {}

impl Pmi1 {
    const HOST_NAME_KEY: &'static str = "rpmi-host";
    const HOST_NAME_BUF: usize = 256;

    /// Initialize the PMI1 backend and return a `Pmi1` instance.
    ///
    /// This performs PMI initialization if necessary and gathers rank and
    /// node information. Returns a `PmiError` on failure.
    pub fn new() -> Result<Self, crate::pmi::PmiError> {
        let mut finalize = false;
        let mut init = 0;
        let mut spawned = 0;
        check_error!(unsafe { pmi_sys::PMI_Initialized(&mut init) });

        if init as u32 == pmi_sys::PMI_FALSE {
            check_error!(unsafe { pmi_sys::PMI_Init(&mut spawned as *mut i32) });
            finalize = true;
        }

        let mut size = 0;
        check_error!(unsafe { pmi_sys::PMI_Get_size(&mut size) });

        let mut my_rank = 0;
        check_error!(unsafe { pmi_sys::PMI_Get_rank(&mut my_rank) });

        let mut max_name_len = 0;
        check_error!(unsafe { pmi_sys::PMI_KVS_Get_name_length_max(&mut max_name_len) });

        let mut name: Vec<u8> = vec![0; max_name_len as usize];
        check_error!(unsafe {
            pmi_sys::PMI_KVS_Get_my_name(name.as_mut_ptr() as *mut i8, max_name_len)
        });

        let c_kvs_name = std::ffi::CString::new(name);
        let kvs_name = match c_kvs_name {
            Err(error) => {
                let len: usize = error.nul_position();
                std::ffi::CString::new(&error.into_vec()[..len]).unwrap()
            }
            Ok(rkvs_name) => rkvs_name,
        };

        let host_name = Self::local_hostname();

        let mut pmi = Pmi1 {
            my_rank: my_rank as usize,
            node_id: 0,
            host_name,
            generated_job_id: String::new(),
            ranks: (0..size as usize).collect(),
            finalize,
            kvs_name,
            singleton_kvs: RwLock::new(HashMap::new()),
        };

        pmi.init_node_info()?;

        Ok(pmi)
    }

    fn init_node_info(&mut self) -> Result<(), crate::pmi::PmiError> {
        if self.ranks.len() <= 1 {
            self.node_id = 0;
            let local_hosts = vec![self.host_name.clone()];
            self.generated_job_id = Self::make_job_id(&local_hosts, self.ranks.len());
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

        self.generated_job_id = Self::make_job_id(&unique_hosts, self.ranks.len());

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

    fn make_job_id(hosts: &[String], rank_count: usize) -> String {
        if hosts.is_empty() {
            return format!("rlibfab-job-{rank_count}");
        }

        let mut hasher = DefaultHasher::new();
        for host in hosts {
            host.hash(&mut hasher);
            0u8.hash(&mut hasher);
        }
        rank_count.hash(&mut hasher);
        format!("rlibfab-job-{rank_count}-{:x}", hasher.finish())
    }

    fn job_manager_job_id_env() -> Option<String> {
        const VARS: &[&str] = &[
            "PMI_JOBID",
            "PMI_ID",
            "PMI2_JOBID",
            "PMIX_JOBID",
            "PMIX_APPNUM",
            "SLURM_JOB_ID",
            "PBS_JOBID",
            "LSB_JOBID",
            "COBALT_JOBID",
            "ALPS_JOB_ID",
            "JSRUN_JOBID",
            "JOB_ID",
        ];

        for &var in VARS {
            if let Ok(val) = env::var(var) {
                if !val.is_empty() {
                    return Some(val);
                }
            }
        }

        None
    }

    fn get_singleton(&self, key: &str) -> Result<Vec<u8>, crate::pmi::PmiError> {
        if let Some(data) = self.singleton_kvs.read().unwrap().get(key) {
            Ok(data.clone())
        } else {
            Err(crate::pmi::PmiError {
                c_err: pmi_sys::PMI_ERR_INVALID_KEY as i32,
                kind: ErrorKind::InvalidKey,
            })
        }
    }

    fn put_singleton(&self, key: &str, value: &[u8]) -> Result<(), crate::pmi::PmiError> {
        self.singleton_kvs
            .write()
            .unwrap()
            .insert(key.to_owned(), value.to_vec());
        Ok(())
    }
}

impl EncDec for Pmi1 {}

impl Pmi for Pmi1 {
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

    /// Return the job id string discovered for this job.
    ///
    /// Priority: explicit job-manager env vars -> PMI KVS jobid entry ->
    /// generated fallback. May be empty for singleton runs.
    fn job_id_str(&self) -> String {
        if let Some(id) = Self::job_manager_job_id_env() {
            return id;
        }

        if self.ranks.len() <= 1 {
            return self.generated_job_id.clone();
        }

        let key = std::ffi::CString::new("jobid").unwrap();
        let mut buf: Vec<u8> = vec![0; 256];
        let ret = unsafe {
            pmi_sys::PMI_KVS_Get(
                self.kvs_name.as_ptr(),
                key.as_ptr() as *mut i8,
                buf.as_mut_ptr() as *mut i8,
                buf.len() as i32,
            )
        };

        if ret as u32 == pmi_sys::PMI_SUCCESS {
            let cstr = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr() as *const i8) };
            let value = cstr.to_string_lossy().into_owned();
            if !value.is_empty() {
                return value;
            }
        }

        self.generated_job_id.clone()
    }

    /// Return a numeric job id. Prefers runtime-provided numeric ids; hashes
    /// non-numeric strings to a deterministic usize.
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
    /// For multi-rank runs the backend fetches from the PMI KVS and decodes
    /// the stored bytes; for singleton runs the in-memory singleton store is
    /// used.
    fn get(&self, key: &str, len: &usize, rank: &usize) -> Result<Vec<u8>, crate::pmi::PmiError> {
        if self.ranks.len() > 1 {
            let kvs_key = std::ffi::CString::new(format!("rlibfab-{}-{}", rank, key))
                .unwrap()
                .into_raw();
            let mut kvs_val: Vec<u8> = vec![0; 2 * len + 1];

            check_error!(unsafe {
                pmi_sys::PMI_KVS_Get(
                    self.kvs_name.as_ptr(),
                    kvs_key,
                    kvs_val.as_mut_ptr() as *mut i8,
                    kvs_val.len() as i32,
                )
            });
            Ok(self.decode(&kvs_val))
        } else {
            self.get_singleton(key)
        }
    }

    /// Publish `value` under `key` for this rank.
    ///
    /// Values are encoded into an ASCII-safe representation before being
    /// inserted into the KVS. For singleton runs a local store is used.
    fn put(&self, key: &str, value: &[u8]) -> Result<(), crate::pmi::PmiError> {
        if self.ranks.len() > 1 {
            let kvs_key = std::ffi::CString::new(format!("rlibfab-{}-{}", self.my_rank, key))
                .unwrap()
                .into_raw();
            let kvs_val = self.encode(value);

            check_error!(unsafe {
                pmi_sys::PMI_KVS_Put(
                    self.kvs_name.as_ptr(),
                    kvs_key,
                    kvs_val.as_ptr() as *const i8,
                )
            });
            check_error!(unsafe { pmi_sys::PMI_KVS_Commit(self.kvs_name.as_ptr()) });
            Ok(())
        } else {
            self.put_singleton(key, value)
        }
    }

    /// Ensure recent `put` operations are visible to other ranks.
    ///
    /// This calls the PMI commit/fence operations as required by PMI1.
    fn exchange(&self) -> Result<(), crate::pmi::PmiError> {
        check_error!(unsafe { pmi_sys::PMI_KVS_Commit(self.kvs_name.as_ptr()) });
        check_error!(unsafe { pmi_sys::PMI_Barrier() });
        Ok(())
    }

    /// Global barrier across all ranks. `collect_data` is currently ignored
    /// by the PMI1 backend.
    fn barrier(&self, _collect_data: bool) -> Result<(), crate::pmi::PmiError> {
        if self.ranks.len() > 1 {
            check_error!(unsafe { pmi_sys::PMI_Barrier() });
        }
        Ok(())
    }
}

impl Drop for Pmi1 {
    fn drop(&mut self) {
        if self.finalize {
            let err = unsafe { pmi_sys::PMI_Finalize() } as u32;
            if err != pmi_sys::PMI_SUCCESS && !panicking() {
                panic!("{:?}", PmiError::from_pmi1_err_code(err as i32))
            }
        }
    }
}

impl PmiError {
    pub(crate) fn from_pmi1_err_code(c_err: i32) -> Self {
        let kind = if c_err == pmi_sys::PMI_FAIL {
            ErrorKind::OperationFailed
        } else {
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
                // pmi_sys::PMI_ERR_INVALID_KVS => ErrorKind::InvalidKVS,
                _ => ErrorKind::Other,
            }
        };

        Self { c_err, kind }
    }
}
