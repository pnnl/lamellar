use std::collections::HashMap;

/// Error returned by the PMI implementations.
///
/// `c_err` carries the original PMI library error code when available;
/// `kind` is a normalized `ErrorKind` useful for matching at call sites.
pub struct PmiError {
    pub(crate) c_err: i32,
    /// Normalized enum describing the kind of error.
    pub kind: ErrorKind,
}

impl PmiError {
    /// Return the PMI library error code associated with this error, if any.
    ///
    /// The value is intended for diagnostic and logging purposes; callers
    /// should match on `kind` for programmatic handling.
    pub fn c_err(&self) -> i32 {
        self.c_err
    }
}

impl std::fmt::Debug for PmiError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error {:?}, code: {}", self.kind, self.c_err)
    }
}

/// Normalized categories for PMI errors.
///
/// These mirror common PMI failure modes to make handling consistent across
/// the different backend implementations (PMI1/PMI2/PMIx).
#[derive(Debug)]
pub enum ErrorKind {
    /// The PMI library was not initialized.
    NotInitialized,
    /// Not enough buffer/memory available.
    NoBufSpaceAvailable,
    /// An invalid argument was supplied to the PMI call.
    InvalidArg,
    /// A key was invalid or not found.
    InvalidKey,
    /// The provided key length was invalid.
    InvalidKeyLength,
    /// The provided value was invalid.
    InvalidVal,
    /// The provided value length was invalid.
    InvalidValLength,
    /// A general invalid length error.
    InvalidLength,
    /// The number of arguments parsed was invalid.
    InvalidNumArgs,
    /// Generic invalid arguments.
    InvalidArgs,
    /// Failed to parse a numeric value.
    InvalidNumParsed,
    /// Invalid key/value pointer.
    InvalidKeyValP,
    /// Reported size exceeded expectations.
    InvalidSize,
    /// KVS operation failed or KVS not found.
    InvalidKVS,
    /// Operation failed with no more specific reason.
    OperationFailed,
    /// Other/unknown error.
    Other,
}

/// Internal helper trait that combines encoding/decoding helpers with the
/// public `Pmi` trait. Implemented by each backend.
pub(crate) trait EncDec: Pmi {
    fn encode(&self, val: &[u8]) -> Vec<u8> {
        let mut res = vec![0; 2 * val.len() + 1];

        let encodings = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
        ];

        for i in 0..val.len() {
            res[2 * i] = encodings[(val[i] & 0xf) as usize] as u8;
            res[2 * i + 1] = encodings[(val[i] >> 4) as usize] as u8;
        }

        res
    }

    fn decode(&self, val: &[u8]) -> Vec<u8> {
        let mut res = vec![0; val.len() / 2];

        let mut j = 0;
        for el in &mut res {
            if val[j] >= (b'0') && val[j] <= (b'9') {
                *el = val[j] - b'0';
            } else {
                *el = val[j] - b'a' + 10;
            }
            j += 1;

            if val[j] >= b'0' && val[j] <= b'9' {
                *el |= (val[j] - b'0') << 4;
            } else {
                *el |= ((val[j] - b'a') + 10) << 4;
            }

            j += 1;
        }

        res
    }
}

/// Public PMI abstraction used by examples and tests.
///
/// Implementations for PMI1, PMI2 and PMIx expose rank/node information,
/// simple KVS `put`/`get` operations, and collective operations such as
/// `exchange` and `barrier`.
pub trait Pmi: Sync + Send {
    /// Return this process' rank in the job (0..N-1).
    fn rank(&self) -> usize;

    /// Return the node index assigned to this rank.
    ///
    /// Node indices are contiguous in the range 0..num_nodes().
    fn node(&self) -> usize;

    /// Return the number of distinct nodes in the job.
    fn num_nodes(&self) -> usize;

    /// Return the list of ranks that live on the given `node` index.
    ///
    /// Example:
    ///
    /// ```no_run
    /// use pmi::Pmi;
    /// let pmi = pmi::PmiBuilder::init().unwrap();
    /// let ranks = pmi.ranks_on_node(0);
    /// println!("ranks on node 0: {:?}", ranks);
    /// ```
    fn ranks_on_node(&self, node: usize) -> Vec<usize>;

    /// Return a slice containing all ranks in the job (0..N-1).
    fn ranks(&self) -> &[usize];

    /// Return a mapping from node index to the ranks on that node.
    ///
    /// Example:
    ///
    /// ```no_run
    /// use pmi::Pmi;
    /// let pmi = pmi::PmiBuilder::init().unwrap();
    /// let map = pmi.node_map();
    /// for (n, ranks) in map {
    ///     println!("node {} -> ranks {:?}", n, ranks);
    /// }
    /// ```
    fn node_map(&self) -> HashMap<usize, Vec<usize>>;

    /// Return the raw job id string as provided by the runtime or derived by
    /// the backend. May be empty for singleton runs.
    ///
    /// Example:
    ///
    /// ```no_run
    /// use pmi::Pmi;
    /// let pmi = pmi::PmiBuilder::init().unwrap();
    /// println!("job id string: {}", pmi.job_id_str());
    /// ```
    fn job_id_str(&self) -> String;

    /// Return a deterministic numeric job id.
    ///
    /// If the runtime provided a numeric job id string the numeric value is
    /// returned directly; otherwise a deterministic hash is returned.
    ///
    /// Example:
    ///
    /// ```no_run
    /// use pmi::Pmi;
    /// let pmi = pmi::PmiBuilder::init().unwrap();
    /// let jid = pmi.job_id();
    /// println!("numeric job id: {}", jid);
    /// ```
    fn job_id(&self) -> usize;

    /// Retrieve the value previously stored under `key` by `rank`.
    ///
    /// `len` is the expected encoded buffer length for the backend; returns
    /// the raw decoded bytes on success or a `PmiError` on failure.
    ///
    /// Example:
    ///
    /// ```no_run
    /// use pmi::Pmi;
    /// let pmi = pmi::PmiBuilder::init().unwrap();
    /// pmi.put("k", b"v").unwrap();
    /// pmi.exchange().unwrap();
    /// let v = pmi.get("k", &2usize, &0usize).unwrap();
    /// assert_eq!(v, b"v");
    /// ```
    fn get(&self, key: &str, len: &usize, rank: &usize) -> Result<Vec<u8>, PmiError>;

    /// Store `value` under `key` for this rank.
    ///
    /// Values are encoded by the backend before insertion into the KVS.
    ///
    /// Example:
    ///
    /// ```no_run
    /// use pmi::Pmi;
    /// let pmi = pmi::PmiBuilder::init().unwrap();
    /// pmi.put("hello", b"world").unwrap();
    /// pmi.exchange().unwrap();
    /// ```
    fn put(&self, key: &str, value: &[u8]) -> Result<(), PmiError>;

    /// Make recent `put` operations visible to other ranks (backend fence).
    fn exchange(&self) -> Result<(), PmiError>;

    /// Global barrier across all ranks. `collect_data` is reserved for
    /// backends that return additional collective information; it may be
    /// ignored by some implementations.
    ///
    /// Example:
    ///
    /// ```no_run
    /// use pmi::Pmi;
    /// let pmi = pmi::PmiBuilder::init().unwrap();
    /// pmi.barrier(false).unwrap();
    /// ```
    fn barrier(&self, collect_data: bool) -> Result<(), PmiError>;
}

/// Returns `Some(usize)` when `id` contains only ASCII digits.
///
/// Useful for preferring numeric job id strings as a raw `usize` instead of
/// hashing non-numeric job id strings.
pub fn numeric_job_id(id: &str) -> Option<usize> {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        return trimmed.parse::<usize>().ok();
    }
    None
}

/// Builder helpers for selecting and initializing a concrete PMI backend.
///
/// Use `PmiBuilder::init()` to create the default enabled backend, or
/// `with_pmi1`/`with_pmi2`/`with_pmix` to force a specific backend.
pub struct PmiBuilder {}

impl PmiBuilder {
    /// Initialize and return the default enabled PMI backend.
    ///
    /// The concrete backend returned depends on enabled Cargo features
    /// (`with-pmi1`, `with-pmi2`, `with-pmix`).
    ///
    /// Example (runtime required; shown for documentation):
    ///
    /// ```no_run
    /// use pmi::Pmi;
    /// use pmi::PmiBuilder;
    /// let pmi = PmiBuilder::init().expect("failed to init PMI backend");
    /// println!("rank {} / {} ranks", pmi.rank(), pmi.ranks().len());
    /// pmi.put("greeting", b"hello").unwrap();
    /// pmi.exchange().unwrap();
    /// ```
    #[cfg(any(feature = "with-pmi1", feature = "with-pmi2", feature = "with-pmix"))]
    pub fn init() -> Result<impl Pmi, PmiError> {
        #[cfg(not(any(feature = "with-pmi2", feature = "with-pmix")))]
        return crate::pmi1::Pmi1::new();
        #[cfg(all(not(feature = "with-pmix"), feature = "with-pmi2"))]
        return crate::pmi2::Pmi2::new();
        #[cfg(feature = "with-pmix")]
        return crate::pmix::PmiX::new();
    }

    #[cfg(feature = "with-pmi1")]
    /// Initialize and return a PMI1 backend instance when the
    /// `with-pmi1` feature is enabled.
    pub fn with_pmi1() -> Result<impl Pmi, PmiError> {
        crate::pmi1::Pmi1::new()
    }

    #[cfg(feature = "with-pmi2")]
    /// Initialize and return a PMI2 backend instance when the
    /// `with-pmi2` feature is enabled.
    pub fn with_pmi2() -> Result<impl Pmi, PmiError> {
        crate::pmi2::Pmi2::new()
    }

    #[cfg(feature = "with-pmix")]
    /// Initialize and return a PMIx backend instance when the
    /// `with-pmix` feature is enabled.
    pub fn with_pmix() -> Result<impl Pmi, PmiError> {
        crate::pmix::PmiX::new()
    }
}

#[test]
fn init() {
    let pmi = PmiBuilder::init().unwrap();
    println!(
        "Hello world from ranks : {}/{}",
        pmi.rank(),
        pmi.ranks().len()
    );
}

#[test]
fn init_pmi1() {
    let pmi = PmiBuilder::with_pmi1().unwrap();
    println!(
        "Hello world from ranks : {}/{}",
        pmi.rank(),
        pmi.ranks().len()
    );
}

#[test]
fn put_get() {
    let pmi = PmiBuilder::with_pmi1().unwrap();
    println!(
        "Hello world from rank : {}/{}",
        pmi.rank(),
        pmi.ranks().len()
    );
    for i in 0..50 {
        let val = pmi.rank() as u8 + i as u8;
        pmi.put(format!("put{}", i).as_str(), std::slice::from_ref(&val))
            .unwrap();
        pmi.exchange().unwrap();
        let res = pmi
            .get(
                format!("put{}", i).as_str(),
                &1,
                &((pmi.rank() + 1) % pmi.ranks().len()),
            )
            .unwrap();
        assert_eq!(res[0], (i + (pmi.rank() + 1) % pmi.ranks().len()) as u8);
    }
}
