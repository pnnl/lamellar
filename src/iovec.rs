use std::marker::PhantomData;

#[repr(C)]
pub struct IoVec<'a, T> {
    pub(crate) c_iovec: libfabric_sys::iovec,
    pub(crate) borrow: PhantomData<&'a T>,
}

impl<'a, T> IoVec<'a, T> {

    pub fn from(mem: &'a T) -> Self {
        let c_iovec = libfabric_sys::iovec{
            iov_base:  (mem as *const T as *mut T).cast(),
            iov_len: std::mem::size_of_val(mem),
        };

        Self { c_iovec, borrow: PhantomData }
    }

    pub fn from_mut(mem: &'a mut T) -> Self {
        let c_iovec = libfabric_sys::iovec{
            iov_base:  (mem as *mut T).cast(),
            iov_len: std::mem::size_of_val(mem),
        };

        Self { c_iovec, borrow: PhantomData }
    }

    pub fn from_slice(mem: &'a [T]) -> Self {
        let c_iovec = libfabric_sys::iovec{
            iov_base:  (mem.as_ptr() as *mut T).cast(),
            iov_len: std::mem::size_of_val(mem),
        };

        Self { c_iovec, borrow: PhantomData }
    }

    pub fn from_slice_mut(mem: &'a mut [T]) -> Self {
        let c_iovec = libfabric_sys::iovec{
            iov_base:  mem.as_mut_ptr().cast(),
            iov_len: std::mem::size_of_val(mem),
        };

        Self { c_iovec, borrow: PhantomData }
    }

    pub(crate) fn get(&self) ->  *const libfabric_sys::iovec {
        &self.c_iovec
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::iovec {
        &mut self.c_iovec
    }
}

#[repr(C)]
pub struct Ioc<'a, T>{
    pub(crate) c_ioc: libfabric_sys::fi_ioc,
    pub(crate) borrow: PhantomData<&'a T>,
}

impl<'a, T> Ioc<'a, T> {

    pub fn from(mem: &'a T) -> Self {
        let c_ioc = libfabric_sys::fi_ioc{
            addr:  (mem as *const T as *mut T).cast(),
            count: 1,
        };

        Self { c_ioc, borrow: PhantomData }
    }

    pub fn from_mut(mem: &'a mut T) -> Self {
        let c_ioc = libfabric_sys::fi_ioc{
            addr:  (mem as *mut T).cast(),
            count: 1,
        };

        Self { c_ioc, borrow: PhantomData }
    }

    pub fn from_slice(mem: &'a [T]) -> Self {
        let c_ioc = libfabric_sys::fi_ioc{
            addr:  (mem.as_ptr() as *mut T).cast(),
            count: mem.len(),
        };

        Self { c_ioc, borrow: PhantomData }
    }

    pub fn from_slice_mut(mem: &'a mut [T]) -> Self {
        let c_ioc = libfabric_sys::fi_ioc{
            addr:  mem.as_mut_ptr().cast(),
            count: mem.len(),
        };

        Self { c_ioc, borrow: PhantomData }
    }

    pub(crate) fn get(&self) ->  *const libfabric_sys::fi_ioc {
        &self.c_ioc
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) ->  *mut libfabric_sys::fi_ioc {
        &mut self.c_ioc
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct RmaIoVec {
    pub(crate) c_rma_iovec: libfabric_sys::fi_rma_iov,
}

impl RmaIoVec {
    pub fn new() -> Self {
        Self {
            c_rma_iovec: libfabric_sys::fi_rma_iov {
                addr: 0,
                len: 0,
                key: 0,
            }
        }
    }

    pub fn address(mut self, addr: u64) -> Self {
        self.c_rma_iovec.addr = addr;
        self
    }

    pub fn len(mut self, len: usize) -> Self {
        self.c_rma_iovec.len = len;
        self
    }

    pub fn key(mut self, key: u64) -> Self {
        self.c_rma_iovec.key = key;
        self
    }

    pub fn get_address(&self) -> u64 {
        self.c_rma_iovec.addr
    }

    pub fn get_len(&self) -> usize {
        self.c_rma_iovec.len
    }

    pub fn get_key(&self) -> u64 {
        self.c_rma_iovec.key
    }

}

impl Default for RmaIoVec {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg(ignore)]
mod rust_lifetime_tests {
    use crate::iovec::IoVec;

    fn foo(data: &mut [usize]) {}
    fn foo_ref(data: & [usize]) {}
    fn foo2<T>(data: & iovec::IoVec<T>) {}

    #[test]
    fn iovec_lifetime() {
        let mut  data: Vec<usize> = Vec::new();
        let iov = iovec::IoVec::from_slice(&data);
        drop(data);
        iov.get();
    }
    
    #[test]
    fn iovec_borrow_mut() {
        let mut  data: Vec<usize> = Vec::new();
        let iov = iovec::IoVec::from_slice(&data);
        foo(&mut data);
        drop(data);
        iov.get();
    }
    

    #[test]
    fn iovec_mut_mut() {
        let mut  data: Vec<usize> = Vec::new();
        let iov = iovec::IoVec::from_slice_mut(&mut data);
        foo(&mut data);
        iov.get();
    }
    
    #[test]
    fn iovec_mut_borrow() {
        let mut  data: Vec<usize> = Vec::new();
        let iov = iovec::IoVec::from_slice_mut(&mut data);
        foo_ref(&data);
        iov.get();
    }
}