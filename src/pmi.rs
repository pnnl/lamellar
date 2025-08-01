
pub struct PmiError {
    pub(crate) c_err: i32,
    pub kind : ErrorKind,
}


impl std::fmt::Debug for PmiError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        write!(f, "Error {:?}, code: {}", self.kind, self.c_err)
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    NotInitialized,
    NoBufSpaceAvailable,
    InvalidArg,
    InvalidKey,
    InvalidKeyLength,
    InvalidVal,
    InvalidValLength,
    InvalidLength,
    InvalidNumArgs,
    InvalidArgs,
    InvalidNumParsed,
    InvalidKeyValP,
    InvalidSize,
    InvalidKVS,
    OperationFailed,
    Other,
}

pub(crate) trait EncDec: Pmi {

    fn encode(&self, val: &[u8]) -> Vec<u8> {
        let mut res = vec![0; 2 * val.len() + 1];

        let encodings = ['0','1','2','3','4','5','6','7','8','9','a','b','c','d','e','f'];


        for i in 0..val.len() {
            res[2*i] = encodings[(val[i] & 0xf) as usize] as u8;
            res[2*i+1] = encodings[(val[i] >> 4) as usize] as u8;
        }

        res
    }

    fn decode(&self, val: &[u8]) -> Vec<u8> {
        let mut res = vec![0; val.len()/2];

        let mut j = 0;
        for el in &mut res {
            if val[j] >= (b'0') && val[j] <= (b'9') {
                *el = val[j] - b'0';
            }
            else {
                *el = val[j] - b'a' + 10;
            }
            j += 1;

            if val[j] >= b'0' && val[j] <= b'9' {
                *el |= (val[j] - b'0' ) << 4;
            }
            else {
                *el |= ((val[j] - b'a') + 10) << 4;
            }

            j += 1;
        }
    
        res
    }
}

pub trait Pmi {
    fn rank(&self) -> usize;
    fn ranks(&self) -> &[usize];
    fn get(&self, key: &str, len: &usize, rank: &usize) -> Result<Vec<u8>, PmiError>;
    fn put(&self, key: &str, value: &[u8]) -> Result<(), PmiError>;
    fn exchange(&self) -> Result<(), PmiError>;
    fn barrier(&self,collect_data: bool) -> Result<(), PmiError>;

}

pub struct PmiBuilder {


}

impl PmiBuilder {

    #[cfg(any(feature = "with-pmi1", feature = "with-pmi2", feature = "with-pmix"))]
    pub fn init() -> Result<impl Pmi, PmiError> {
        #[cfg(not(any(feature="with-pmi2", feature="with-pmix")))] 
        return crate::pmi1::Pmi1::new();
        #[cfg(not(feature="with-pmix"))] 
        return crate::pmi2::Pmi2::new();
        #[cfg(feature="with-pmix")] 
        return crate::pmix::PmiX::new();
    }

    #[cfg(feature="with-pmi1")] 
    pub fn with_pmi1() -> Result<impl Pmi, PmiError> {
        return crate::pmi1::Pmi1::new();
    }

    #[cfg(feature="with-pmi2")] 
    pub fn with_pmi2() -> Result<impl Pmi, PmiError> {
        return crate::pmi2::Pmi2::new();
    }

    #[cfg(feature="with-pmix")] 
    pub fn with_pmix() -> Result<impl Pmi, PmiError> {
        return crate::pmix::PmiX::new();
    }
}

#[test]
fn init() {
    let pmi = PmiBuilder::init().unwrap();
    println!("Hello world from ranks : {}/{}", pmi.rank(), pmi.ranks().len());
}

#[test]
fn init_pmi1() {
    let pmi = PmiBuilder::with_pmi1().unwrap();
    println!("Hello world from ranks : {}/{}", pmi.rank(), pmi.ranks().len());
}

#[test]
fn put_get() {
    let pmi = PmiBuilder::with_pmi1().unwrap();
    println!("Hello world from rank : {}/{}", pmi.rank(), pmi.ranks().len());
    for i in 0..50 {
        let val = pmi.rank() as u8 + i as u8;
        pmi.put(format!("put{}", i).as_str(), std::slice::from_ref(&val)).unwrap();
        pmi.exchange().unwrap();
        let res = pmi.get(format!("put{}", i).as_str(), &1, &((pmi.rank()+1) % pmi.ranks().len())).unwrap();
        assert_eq!(res[0], (i + (pmi.rank() + 1) % pmi.ranks().len()) as u8);
    }
}