use std::marker::PhantomData;

use crate::{mr::MappedMemoryRegionKey, RemoteMemoryAddress, RemoteMemAddrSlice, RemoteMemAddrSliceMut};

unsafe impl<'a> Send for IoVec<'a> {}
unsafe impl<'a> Sync for IoVec<'a> {}
#[repr(C)]
/// A wrapper for `libfabric_sys::iovec`
pub struct IoVec<'a> {
    c_iovec: libfabric_sys::iovec,
    borrow: PhantomData<&'a ()>,
}

impl<'a> IoVec<'a> {

    /// Creates a new IoVec from a reference to a memory region.
    pub fn from<T>(mem: &'a T) -> Self {
        let c_iovec = libfabric_sys::iovec {
            iov_base: (mem as *const T as *mut T).cast(),
            iov_len: std::mem::size_of_val(mem),
        };

        Self {
            c_iovec,
            borrow: PhantomData,
        }
    }

    /// Creates a new IoVec from a slice of data
    pub fn from_slice<T>(mem: &'a [T]) -> Self {
        let c_iovec = libfabric_sys::iovec {
            iov_base: (mem.as_ptr() as *mut T).cast(),
            iov_len: std::mem::size_of_val(mem),
        };

        Self {
            c_iovec,
            borrow: PhantomData,
        }
    }

    /// Returns the length of the IoVec
    pub fn len(&self) -> usize {
        self.c_iovec.iov_len
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> &libfabric_sys::iovec {
        &self.c_iovec
    }
}

unsafe impl<'a> Send for IoVecMut<'a> {}
unsafe impl<'a> Sync for IoVecMut<'a> {}

#[repr(C)]
/// Represents a mutable [IoVec]
pub struct IoVecMut<'a> {
    c_iovec: libfabric_sys::iovec,
    borrow: PhantomData<&'a mut ()>,
}

impl<'a> IoVecMut<'a> {
    pub fn from<T>(mem: &'a mut T) -> Self {
        let c_iovec = libfabric_sys::iovec {
            iov_base: (mem as *mut T).cast(),
            iov_len: std::mem::size_of_val(mem),
        };

        Self {
            c_iovec,
            borrow: PhantomData,
        }
    }

    pub fn from_slice<T>(mem: &'a mut [T]) -> Self {
        let c_iovec = libfabric_sys::iovec {
            iov_base: mem.as_mut_ptr().cast(),
            iov_len: std::mem::size_of_val(mem),
        };

        Self {
            c_iovec,
            borrow: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.c_iovec.iov_len
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) -> &mut libfabric_sys::iovec {
        &mut self.c_iovec
    }
}

#[repr(C)]
pub struct Ioc<'a, T> {
    c_ioc: libfabric_sys::fi_ioc,
    borrow: PhantomData<&'a T>,
}

impl<'a, T> Ioc<'a, T> {
    pub fn from(mem: &'a T) -> Self {
        let c_ioc = libfabric_sys::fi_ioc {
            addr: (mem as *const T as *mut T).cast(),
            count: 1,
        };

        Self {
            c_ioc,
            borrow: PhantomData,
        }
    }


    pub fn from_slice(mem: &'a [T]) -> Self {
        let c_ioc = libfabric_sys::fi_ioc {
            addr: (mem.as_ptr() as *mut T).cast(),
            count: mem.len(),
        };

        Self {
            c_ioc,
            borrow: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.c_ioc.count * std::mem::size_of::<T>()
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> &libfabric_sys::fi_ioc {
        &self.c_ioc
    }
}

#[repr(C)]
pub struct IocMut<'a, T> {
    c_ioc: libfabric_sys::fi_ioc,
    borrow: PhantomData<&'a mut T>,
}

impl<'a, T> IocMut<'a, T> {
    pub fn from(mem: &'a mut T) -> Self {
        let c_ioc = libfabric_sys::fi_ioc {
            addr: (mem as *mut T).cast(),
            count: 1,
        };

        Self {
            c_ioc,
            borrow: PhantomData,
        }
    }

    pub fn from_slice(mem: &'a mut [T]) -> Self {
        let c_ioc = libfabric_sys::fi_ioc {
            addr: mem.as_mut_ptr().cast(),
            count: mem.len(),
        };

        Self {
            c_ioc,
            borrow: PhantomData,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn get_mut(&mut self) -> &mut libfabric_sys::fi_ioc {
        &mut self.c_ioc
    }
}


#[derive(Clone, Debug)]
pub struct RemoteMemAddrVec<'a> {
    pub(crate) c_rma_iovecs: Vec<libfabric_sys::fi_rma_iov>,
    phantom: PhantomData<&'a ()>
}

impl<'a> RemoteMemAddrVec<'a> {
    pub fn new() -> Self {
        Self {
            c_rma_iovecs: Vec::new(),
            phantom: PhantomData,
        }
    }

    pub fn push<T: Copy>(&mut self, rma_iovec: RemoteMemAddrSlice<'a, T>) {
        let c_rma_iov = libfabric_sys::fi_rma_iov {
            addr: rma_iovec.mem_address().into(),
            len: rma_iovec.mem_size(),
            key: rma_iovec.key().key(),
        };

        self.c_rma_iovecs.push(c_rma_iov);
    }
    
    pub fn len(&self) -> usize {
        self.c_rma_iovecs.len()
    }

    pub(crate) fn get(&self) -> &[libfabric_sys::fi_rma_iov] {
        &self.c_rma_iovecs
    }
}

pub struct RemoteMemAddrVecMut<'a> { 
    pub(crate) c_rma_iovecs: Vec<libfabric_sys::fi_rma_iov>,
    phantom: PhantomData<&'a mut ()>
}

impl<'a> RemoteMemAddrVecMut<'a> {
    pub fn new() -> Self {
        Self {
            c_rma_iovecs: Vec::new(),
            phantom: PhantomData,
        }
    }

    pub fn push<T: Copy>(&mut self, rma_iovec: RemoteMemAddrSliceMut<'a, T>) {
        let c_rma_iov = libfabric_sys::fi_rma_iov {
            addr: rma_iovec.mem_address().into(),
            len: rma_iovec.mem_size(),
            key: rma_iovec.key().key(),
        };

        self.c_rma_iovecs.push(c_rma_iov);
    }

    pub fn push_raw(&mut self, addr: RemoteMemoryAddress, len: usize, key: &MappedMemoryRegionKey) {
        let c_rma_iov = libfabric_sys::fi_rma_iov {
            addr: addr.into(),
            len,
            key: key.key(),
        };

        self.c_rma_iovecs.push(c_rma_iov);
    }

    pub fn len(&self) -> usize {
        self.c_rma_iovecs.len()
    }

    pub(crate) fn get(&self) -> &[libfabric_sys::fi_rma_iov] {
        &self.c_rma_iovecs
    }
}

pub struct RemoteMemAddrAtomicVec<'a, T> {
    pub(crate) c_rma_iocs: Vec<libfabric_sys::fi_rma_ioc>,
    phantom: PhantomData<&'a T>
}

impl<'a, T: Copy> RemoteMemAddrAtomicVec<'a, T> {
    pub fn new() -> Self {
        Self {
            c_rma_iocs: Vec::new(),
            phantom: PhantomData,
        }
    }

    pub fn push(&mut self, rma_ioc: RemoteMemAddrSlice<'a, T>) {
        let c_rma_ioc = libfabric_sys::fi_rma_ioc {
            addr: rma_ioc.mem_address().into(),
            count: rma_ioc.len(),
            key: rma_ioc.key().key(),
        };

        self.c_rma_iocs.push(c_rma_ioc);
    }

    pub fn push_raw(&mut self, addr: RemoteMemoryAddress, count: usize, key: &MappedMemoryRegionKey) {
        let c_rma_ioc = libfabric_sys::fi_rma_ioc {
            addr: addr.into(),
            count,
            key: key.key(),
        };

        self.c_rma_iocs.push(c_rma_ioc);
    }

    pub fn len(&self) -> usize {
        self.c_rma_iocs.len()
    }

    pub(crate) fn get(&self) -> &[libfabric_sys::fi_rma_ioc] {
        &self.c_rma_iocs
    }
}

pub struct RemoteMemAddrAtomicVecMut<'a, T> {
    pub(crate) c_rma_iocs: Vec<libfabric_sys::fi_rma_ioc>,
    phantom: PhantomData<&'a mut T>
}

impl<'a, T: Copy> RemoteMemAddrAtomicVecMut<'a, T> {
    pub fn new() -> Self {
        Self {
            c_rma_iocs: Vec::new(),
            phantom: PhantomData,
        }
    }

    pub fn push(&mut self, rma_ioc: RemoteMemAddrSliceMut<'a, T>) {
        let c_rma_ioc = libfabric_sys::fi_rma_ioc {
            addr: rma_ioc.mem_address().into(),
            count: rma_ioc.len(),
            key: rma_ioc.key().key(),
        };

        self.c_rma_iocs.push(c_rma_ioc);
    }

    pub fn push_raw(&mut self, addr: RemoteMemoryAddress, count: usize, key: &MappedMemoryRegionKey) {
        let c_rma_ioc = libfabric_sys::fi_rma_ioc {
            addr: addr.into(),
            count,
            key: key.key(),
        };

        self.c_rma_iocs.push(c_rma_ioc);
    }

    pub fn len(&self) -> usize {
        self.c_rma_iocs.len()
    }

    #[allow(dead_code)]
    pub(crate) fn get(&self) -> &[libfabric_sys::fi_rma_ioc] {
        &self.c_rma_iocs
    }
}

// pub struct RmaIoc<T> {
//     pub(crate) c_rma_ioc: libfabric_sys::fi_rma_ioc,
//     phantom: PhantomData<T>,
// }

// impl<T: Copy> RmaIoc<T> {
//     pub fn new(addr: RemoteMemoryAddress, count: usize, key: &MappedMemoryRegionKey) -> Self {
//         Self {
//             c_rma_ioc: libfabric_sys::fi_rma_ioc {
//                 addr: addr.into(),
//                 count,
//                 key: key.key(),
//             },
//             phantom: PhantomData,
//         }
//     }

//     pub fn from_slice(addr: &RemoteMemAddrSlice<T>) -> Self {
//         Self {
//             c_rma_ioc: libfabric_sys::fi_rma_ioc {
//                 addr: addr.mem_address().into(),
//                 count: addr.mem_size() / std::mem::size_of::<T>(),
//                 key: addr.key().key(),
//             },
//             phantom: PhantomData,
//         }
//     }

//     pub fn count(&self) -> usize {
//         self.c_rma_ioc.count
//     }

//     #[allow(dead_code)]
//     pub(crate) fn get(&self) -> *const libfabric_sys::fi_rma_ioc {
//         &self.c_rma_ioc
//     }
// }