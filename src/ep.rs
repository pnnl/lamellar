// static inline int
// fi_passive_ep(struct fid_fabric *fabric, struct fi_info *info,
// 	     struct fid_pep **pep, void *context)
// {
// 	return fabric->ops->passive_ep(fabric, info, pep, context);
// }
// pub fn inline_fi_passive_ep(
//     fabric: *mut fid_fabric,
//     info: *mut fi_info,
//     pep: *mut *mut fid_pep,
//     context: *mut ::std::os::raw::c_void,
// ) -> ::std::os::raw::c_int;
// inlined_fi_endpoint

// pub fn inlined_fi_endpoint(
//     domain: *mut fid_domain,
//     info: *mut fi_info,
//     ep: *mut *mut fid_ep,
//     context: *mut ::std::os::raw::c_void,
// ) -> ::std::os::raw::c_int;

use core::panic;
use std::ffi::c_void;
// use std::{io::Error, ptr::null_mut};

use crate::{FID, Address};

pub struct PassiveEndPoint {
    pub(crate) c_pep: *mut libfabric_sys::fid_pep,
}

impl PassiveEndPoint {
    pub fn new(fabric: &crate::fabric::Fabric, info: &crate::InfoEntry) -> Self {
        let mut c_pep: *mut libfabric_sys::fid_pep = std::ptr::null_mut();
        let c_pep_ptr: *mut *mut libfabric_sys::fid_pep = &mut c_pep;
        let err = unsafe { libfabric_sys::inlined_fi_passive_ep(fabric.c_fabric, info.c_info, c_pep_ptr, std::ptr::null_mut()) };
        
        if err != 0 {
            panic!("fi_passive_ep failed {}", err);
        }
        
        Self { c_pep }
    }
    
    pub fn bind(&self, fid: &impl FID, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_pep_bind(self.c_pep,fid.fid(), flags) };
        
        if err != 0 {
            panic!("fi_pep_bind failed {}", err);
        }
    }

    pub fn listen(&self) {
        let err = unsafe {libfabric_sys::inlined_fi_listen(self.c_pep)};
        
        if err != 0 {
            panic!("fi_listen failed {}", err);
        }
    }

    pub fn reject<T0>(&self, fid: &impl FID, params: &[T0]) {
        let err = unsafe {libfabric_sys::inlined_fi_reject(self.c_pep, fid.fid(), params.as_ptr() as *const std::ffi::c_void, params.len())};

        if err != 0 {
            panic!("fi_reject failed {}", err);
        }

    }
}

// pub struct ScalableEndPoint {
//     pub(crate) c_ep: *mut libfabric_sys::fid_ep,
// }

// impl ScalableEndPoint {
//     pub fn new(fabric: &crate::fabric::Fabric, info: &crate::InfoEntry) -> Self {
//         let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
//         let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
//         let err = unsafe { libfabric_sys::inlined_fi_scalable_ep(fabric.c_fabric, info.c_info, c_pep_ptr, std::ptr::null_mut()) };
        
//         if err != 0 {
//             panic!("fi_passive_ep failed {}", err);
//         }
        
//         Self { c_ep }
//     }

//     pub fn bind(&self, fid: &impl FID, flags: u64) {
//         let err = unsafe { libfabric_sys::inlined_fi_scalable_ep_bind(self.c_ep,fid.fid(), flags) };
        
//         if err != 0 {
//             panic!("fi_scalable_ep_bind failed {}", err);
//         }
//     } 
// }

impl Endpoint {
    // static inline int
    // fi_passive_ep(struct fid_fabric *fabric, struct fi_info *info,
    //          struct fid_pep **pep, void *context) [TODO]

    //     static inline int
    // fi_endpoint(struct fid_domain *domain, struct fi_info *info,
    // 	    struct fid_ep **ep, void *context)
    pub fn new<T0>(domain: &crate::domain::Domain, info: &crate::InfoEntry, context: &mut T0) -> Self {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err = unsafe { libfabric_sys::inlined_fi_endpoint(domain.c_domain, info.c_info, c_ep_ptr, context as *mut T0 as *mut std::ffi::c_void) };

        if err != 0 {
            panic!("fi_endpoint failed {}", err);
        }

        Self { c_ep }
    }

    pub fn new2<T0>(domain: &crate::domain::Domain, info: &crate::InfoEntry, flags: u64, context: &mut T0) -> Self {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err = unsafe { libfabric_sys::inlined_fi_endpoint2(domain.c_domain, info.c_info, c_ep_ptr, flags, context as *mut T0 as *mut std::ffi::c_void) };

        if err != 0 {
            panic!("fi_endpoint2 failed {}", err);
        }

        Self { c_ep }
    }

    pub fn new_scalable(domain: &crate::domain::Domain, info: &crate::InfoEntry) -> Self {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err = unsafe { libfabric_sys::inlined_fi_scalable_ep(domain.c_domain, info.c_info, c_ep_ptr, std::ptr::null_mut()) };

        if err != 0 {
            panic!("fi_scalable_ep failed {}", err);
        }

        Self { c_ep }
    }

    //     static inline int
    // fi_endpoint2(struct fid_domain *domain, struct fi_info *info,
    // 	     struct fid_ep **ep, uint64_t flags, void *context) [TODO]

    // static inline int
    // fi_scalable_ep(struct fid_domain *domain, struct fi_info *info,
    //         struct fid_ep **sep, void *context) [TODO]

    
    
    
    
    pub fn bind(&self, fid: &impl FID, flags: u64) {
        let err = unsafe { libfabric_sys::inlined_fi_ep_bind(self.c_ep,fid.fid(), flags) };
        
        if err != 0 {
            panic!("fi_ep_bind failed {}", err);
        }
    } 
    
    // static inline int fi_pep_bind(struct fid_pep *pep, struct fid *bfid, uint64_t flags) [TODO]
    // static inline int fi_scalable_ep_bind(struct fid_ep *sep, struct fid *bfid, uint64_t flags) [TODO]
    
    // static inline int fi_enable(struct fid_ep *ep)
    pub fn enable(&self) {
        let err = unsafe { libfabric_sys::inlined_fi_enable(self.c_ep) };
        
        if err != 0 {
            panic!("fi_enable failed {}", err);
        }
    }


    // static inline int fi_ep_alias(struct fid_ep *ep, struct fid_ep **alias_ep,
    //     uint64_t flags)
    pub fn alias(&self, flags: u64) -> Endpoint {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;
        let err = unsafe { libfabric_sys::inlined_fi_ep_alias(self.c_ep, c_ep_ptr, flags) };
        
        if err != 0 {
            panic!("fi_ep_alias failed {}", err);
        }
        Endpoint { c_ep }
    }

    //     static inline int fi_tx_context(struct fid_ep *ep, int idx, struct fi_tx_attr *attr,
    // 	      struct fid_ep **tx_ep, void *context)
    pub fn tx_context(&self, idx: i32, mut txattr: crate::TxAttr) -> Self {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;

        let err = unsafe {libfabric_sys::inlined_fi_tx_context(self.c_ep, idx, txattr.get_mut(), c_ep_ptr, std::ptr::null_mut())};
        
        if err != 0 {
            panic!("fi_tx_context failed {}", err);
        }

        Self { c_ep }
    }

    //     static inline int
    // fi_rx_context(struct fid_ep *ep, int idx, struct fi_rx_attr *attr,
    // 	      struct fid_ep **rx_ep, void *context)
    pub fn rx_context(&self, idx: i32, mut rxattr: crate::RxAttr) -> Self {
        let mut c_ep: *mut libfabric_sys::fid_ep = std::ptr::null_mut();
        let c_ep_ptr: *mut *mut libfabric_sys::fid_ep = &mut c_ep;

        let err = unsafe {libfabric_sys::inlined_fi_rx_context(self.c_ep, idx, rxattr.get_mut(), c_ep_ptr, std::ptr::null_mut())};
        
        if err != 0 {
            panic!("fi_tx_context_failed {}", err);
        }

        Self { c_ep }
    }

    // static inline FI_DEPRECATED_FUNC ssize_t
    // fi_rx_size_left(struct fid_ep *ep) [TODO]
    pub fn rx_size_left(&self) -> isize {
        unsafe {libfabric_sys::inlined_fi_rx_size_left(self.c_ep)}
    }

    // static inline FI_DEPRECATED_FUNC ssize_t
    // fi_tx_size_left(struct fid_ep *ep) [TODO]
    pub fn tx_size_left(&self) -> isize {
        unsafe {libfabric_sys::inlined_fi_tx_size_left(self.c_ep)}
    }
    
    // static inline ssize_t
    // fi_recv(struct fid_ep *ep, void *buf, size_t len, void *desc, fi_addr_t src_addr,
    //     void *context) 
    pub fn recv<T0,T1>(&self, buf: &mut [T0], len: usize, desc: &mut [T1], addr: crate::Address) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_recv(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, desc.as_mut_ptr() as *mut std::ffi::c_void, addr, std::ptr::null_mut()) };
        ret
    }


    // static inline ssize_t
    // fi_trecv(struct fid_ep *ep, void *buf, size_t len, void *desc,
    // 	 fi_addr_t src_addr, uint64_t tag, uint64_t ignore, void *context)
    pub fn trecv<T0,T1>(&self, buf: &mut [T0], len: usize, desc: &mut [T1], addr: crate::Address, tag: u64, ignore:u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_trecv(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, desc.as_mut_ptr() as *mut std::ffi::c_void, addr, tag, ignore, std::ptr::null_mut()) };
        ret  
    }

    //     static inline ssize_t
    // fi_read(struct fid_ep *ep, void *buf, size_t len, void *desc,
    // 	fi_addr_t src_addr, uint64_t addr, uint64_t key, void *context)
    pub fn read<T0,T1>(&self, buf: &mut [T0], len: usize, desc: &mut [T1], src_addr: crate::Address, addr: u64,  key: u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_read(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, desc.as_mut_ptr() as *mut std::ffi::c_void, src_addr, addr, key, std::ptr::null_mut()) };
        ret
    }

    // static inline ssize_t
    // fi_recvv(struct fid_ep *ep, const struct iovec *iov, void **desc,
    //      size_t count, fi_addr_t src_addr, void *context) [TODO]
    pub fn recvv<T0,T1>(&self, iov: &crate::IoVec, desc: &mut [T0], count: usize, addr: crate::Address, context: &mut T1) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_recvv(self.c_ep, iov.get(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, addr, context as *mut T1 as *mut std::ffi::c_void) };
        ret
    }


    // static inline ssize_t
    // fi_readv(struct fid_ep *ep, const struct iovec *iov, void **desc,
    //     size_t count, fi_addr_t src_addr, uint64_t addr, uint64_t key,
    //     void *context) [TODO]
    pub fn readv<T0,T1>(&self, iov: &crate::IoVec, desc: &mut [T0], count: usize, src_addr: crate::Address, addr: u64, key: u64, context : &mut T1) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_readv(self.c_ep, iov.get(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, src_addr, addr, key, context as *mut T1 as *mut std::ffi::c_void) };
        ret 
    }

    //     static inline ssize_t
    // fi_trecvv(struct fid_ep *ep, const struct iovec *iov, void **desc,
    // 	  size_t count, fi_addr_t src_addr, uint64_t tag, uint64_t ignore,
    // 	  void *context) [TODO]

    pub fn trecvv<T0,T1>(&self, iov: &crate::IoVec, desc: &mut [T0], count: usize, src_addr: crate::Address, tag: u64, ignore:u64, context : &mut T1) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_trecvv(self.c_ep, iov.get(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, src_addr, tag, ignore, context as *mut T1 as *mut std::ffi::c_void) };
        ret   
    }

    // static inline ssize_t
    // fi_recvmsg(struct fid_ep *ep, const struct fi_msg *msg, uint64_t flags)
    pub fn recvmsg(&self, msg: &crate::Msg, flags: u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_recvmsg(self.c_ep, msg.c_msg, flags) };
        ret
    }

    //     static inline ssize_t
    // fi_trecvmsg(struct fid_ep *ep, const struct fi_msg_tagged *msg, uint64_t flags)
    pub fn trecvmsg(&self, msg: &crate::MsgTagged, flags: u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_trecvmsg(self.c_ep, msg.c_msg_tagged, flags) };
        ret
    }

    //     static inline ssize_t
    // fi_readmsg(struct fid_ep *ep, const struct fi_msg_rma *msg, uint64_t flags)
    pub fn readmsg(&self, msg: &crate::MsgRma, flags: u64) -> isize{
        let ret = unsafe{ libfabric_sys::inlined_fi_readmsg(self.c_ep, msg.c_msg_rma, flags) };
        ret
    }

    //     static inline ssize_t
    // fi_send(struct fid_ep *ep, const void *buf, size_t len, void *desc,
    // 	fi_addr_t dest_addr, void *context) 
    pub fn send<T0,T1>(&self, buf: &mut [T0], len: usize, desc: &mut [T1], addr: crate::Address) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_send(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, desc.as_mut_ptr() as *mut std::ffi::c_void, addr, std::ptr::null_mut()) };
        ret
    }

    //     static inline ssize_t
    // fi_tsend(struct fid_ep *ep, const void *buf, size_t len, void *desc,
    // 	 fi_addr_t dest_addr, uint64_t tag, void *context)
    pub fn tsend<T0,T1>(&self, buf: &mut [T0], len: usize, desc: &mut [T1], addr: crate::Address, tag:u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_tsend(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, desc.as_mut_ptr() as *mut std::ffi::c_void, addr, tag, std::ptr::null_mut()) };
        ret
    }

//     static inline ssize_t
// fi_tsendv(struct fid_ep *ep, const struct iovec *iov, void **desc,
// 	  size_t count, fi_addr_t dest_addr, uint64_t tag, void *context) [TODO]

    pub fn tsendv<T0,T1>(&self, iov: &crate::IoVec, desc: &mut [T0], count: usize, dest_addr: crate::Address, tag:u64, context : &mut T1) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_tsendv(self.c_ep, iov.get(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, dest_addr, tag, context as *mut T1 as *mut std::ffi::c_void) };
        ret   
    }

    // static inline ssize_t
    // fi_write(struct fid_ep *ep, const void *buf, size_t len, void *desc,
    // 	 fi_addr_t dest_addr, uint64_t addr, uint64_t key, void *context)
    pub fn write<T0,T1>(&self, buf: &mut [T0], len: usize, desc: &mut [T1], dest_addr: crate::Address, addr: u64, key:u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_write(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, desc.as_mut_ptr() as *mut std::ffi::c_void, dest_addr, addr, key, std::ptr::null_mut()) };
        ret   
    }

    pub fn writev<T0,T1>(&self, iov: &crate::IoVec, desc: &mut [T0], count: usize,  dest_addr: crate::Address, addr: u64, key:u64, context : &mut T1) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_writev(self.c_ep, iov.get(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, dest_addr, addr, key, context as *mut T1 as *mut std::ffi::c_void) };
        ret   
    }
    //     static inline ssize_t
    // fi_sendv(struct fid_ep *ep, const struct iovec *iov, void **desc,
    // 	 size_t count, fi_addr_t dest_addr, void *context) [TODO]
    pub fn sendv<T0,T1>(&self, iov: &crate::IoVec, desc: &mut [T0], count: usize, addr: crate::Address, context: &mut T1) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_sendv(self.c_ep, iov.get(), desc.as_mut_ptr() as *mut *mut std::ffi::c_void, count, addr, context as *mut T1 as *mut std::ffi::c_void) };
        ret
    }
    //     static inline ssize_t
    // fi_sendmsg(struct fid_ep *ep, const struct fi_msg *msg, uint64_t flags)
    pub fn sendmsg(&self, msg: &crate::Msg, flags: u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_sendmsg(self.c_ep, msg.c_msg, flags) };
        ret
    }

    //     static inline ssize_t
    // fi_tsendmsg(struct fid_ep *ep, const struct fi_msg_tagged *msg, uint64_t flags)
    pub fn tsendmsg(&self, msg: &crate::MsgTagged, flags: u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_tsendmsg(self.c_ep, msg.c_msg_tagged, flags) };
        ret
    }

    //     static inline ssize_t
    // fi_writemsg(struct fid_ep *ep, const struct fi_msg_rma *msg, uint64_t flags)
    pub fn writemsg(&self, msg: &crate::MsgRma, flags: u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_writemsg(self.c_ep, msg.c_msg_rma, flags) };
        ret
    }

    // static inline ssize_t
    // fi_inject(struct fid_ep *ep, const void *buf, size_t len, fi_addr_t dest_addr) 
    pub fn inject<T0>(&self, buf: &mut [T0], len: usize, addr: crate::Address) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_inject(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, addr) };
        ret
    }


    // static inline ssize_t
    // fi_tinject(struct fid_ep *ep, const void *buf, size_t len,
    //     fi_addr_t dest_addr, uint64_t tag)
    pub fn tinject<T0>(&self, buf: &mut [T0], len: usize, addr: crate::Address, tag:u64 ) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_tinject(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, addr, tag) };
        ret
    }


    //     static inline ssize_t
    // fi_inject_write(struct fid_ep *ep, const void *buf, size_t len,
    // 		fi_addr_t dest_addr, uint64_t addr, uint64_t key)
    pub fn inject_write<T0>(&self, buf: &mut [T0], len: usize, dest_addr: crate::Address, addr: u64, key:u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_inject_write(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, dest_addr, addr, key) };
        ret
    }     

    // static inline ssize_t
    // fi_senddata(struct fid_ep *ep, const void *buf, size_t len, void *desc,
    //           uint64_t data, fi_addr_t dest_addr, void *context)
    pub fn senddata<T0,T1>(&self, buf: &mut [T0], len: usize, desc: &mut [T1], data: u64, addr: crate::Address) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_senddata(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, desc.as_mut_ptr() as *mut std::ffi::c_void, data, addr, std::ptr::null_mut()) };
        ret
    }

    // static inline ssize_t
    // fi_tsenddata(struct fid_ep *ep, const void *buf, size_t len, void *desc,
    //          uint64_t data, fi_addr_t dest_addr, uint64_t tag, void *context)
    pub fn tsenddata<T0,T1>(&self, buf: &mut [T0], len: usize, desc: &mut [T1], data: u64, addr: crate::Address, tag: u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_tsenddata(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, desc.as_mut_ptr() as *mut std::ffi::c_void, data, addr, tag, std::ptr::null_mut()) };
        ret
    }

    //     static inline ssize_t
    // fi_writedata(struct fid_ep *ep, const void *buf, size_t len, void *desc,
    // 	       uint64_t data, fi_addr_t dest_addr, uint64_t addr, uint64_t key,
    // 	       void *context)
    pub fn writedata<T0,T1>(&self, buf: &mut [T0], len: usize, desc: &mut [T1], data: u64, addr: crate::Address, other_addr: u64, key: u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_writedata(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, desc.as_mut_ptr() as *mut std::ffi::c_void, data, addr, other_addr, key, std::ptr::null_mut()) };
        ret
    }

    //     static inline ssize_t
    // fi_injectdata(struct fid_ep *ep, const void *buf, size_t len,
    // 		uint64_t data, fi_addr_t dest_addr)
    pub fn injectdata<T0>(&self, buf: &mut [T0], len: usize, data: u64, addr: crate::Address) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_injectdata(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, data, addr) };
        ret
    }



    //     static inline ssize_t
    // fi_tinjectdata(struct fid_ep *ep, const void *buf, size_t len,
    // 		uint64_t data, fi_addr_t dest_addr, uint64_t tag)
    pub fn tinjectdata<T0>(&self, buf: &mut [T0], len: usize, data: u64, addr: crate::Address, tag: u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_tinjectdata(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, data, addr, tag) };
        ret
    }

    // static inline ssize_t
    // fi_inject_writedata(struct fid_ep *ep, const void *buf, size_t len,
    //         uint64_t data, fi_addr_t dest_addr, uint64_t addr, uint64_t key)
    pub fn inject_writedata<T0>(&self, buf: &mut [T0], len: usize, data: u64, dest_addr: crate::Address, addr: u64, key: u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_inject_writedata(self.c_ep, buf.as_mut_ptr() as *mut std::ffi::c_void, len, data, dest_addr, addr, key) };
        ret
    }

    // static inline int fi_getpeer(struct fid_ep *ep, void *addr, size_t *addrlen)
    pub fn getpeer<T0>(&self, addr: &mut [T0]) -> usize { //[TODO] Return result
        let mut len = addr.len();
        let len_ptr: *mut usize = &mut len;
        let _ = unsafe { libfabric_sys::inlined_fi_getpeer(self.c_ep, addr.as_mut_ptr() as *mut std::ffi::c_void, len_ptr)};
        
        len
    }

    // static inline int
    // fi_connect(struct fid_ep *ep, const void *addr,
    //        const void *param, size_t paramlen)
    pub fn connect<T0,T1>(&self, addr: & [T0], param: &[T1]) {
        let ret = unsafe { libfabric_sys::inlined_fi_connect(self.c_ep, addr.as_ptr() as *const std::ffi::c_void, param.as_ptr() as *const std::ffi::c_void, param.len()) };
        
        if ret != 0 {
            panic!("fi_connect failed {}", ret);
        }
    }

    //     static inline int
    // fi_accept(struct fid_ep *ep, const void *param, size_t paramlen)
    pub fn accept<T0>(&self, param: &[T0]) {
        let ret = unsafe { libfabric_sys::inlined_fi_accept(self.c_ep, param.as_ptr() as *const std::ffi::c_void, param.len()) };
        
        if ret != 0 {
            panic!("fi_connect failed {}", ret);
        }
    }
    // static inline int fi_shutdown(struct fid_ep *ep, uint64_t flags)
    pub fn shutdown(&self, flags: u64) {
        let ret = unsafe { libfabric_sys::inlined_fi_shutdown(self.c_ep, flags) };

        if ret != 0 {
            panic!("fi_shutdown failed {}", ret);
        }
    }

    pub fn atomic<T0,T1>(&self, buf: &mut [T0], count : usize, desc: &mut T1, dest_addr: Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::Op) -> isize{
        let ret = unsafe{ libfabric_sys::inlined_fi_atomic(self.c_ep, buf.as_mut_ptr()  as *mut std::ffi::c_void, count, desc as *mut T1  as *mut std::ffi::c_void, dest_addr, addr, key, datatype, op, std::ptr::null_mut())};
        ret
    }

    pub fn atomicv<T0,T1>(&self, iov: &crate::Ioc, desc: &mut [T1], count : usize, dest_addr: Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::Op) -> isize{
        let ret = unsafe{ libfabric_sys::inlined_fi_atomicv(self.c_ep, iov.get(), desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, count, dest_addr, addr, key, datatype, op, std::ptr::null_mut())};
        ret
    }

    pub fn atomicmsg(&self, msg: &crate::MsgAtomic, flags: u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_atomicmsg(self.c_ep, msg.c_msg_atomic, flags) };
        ret
    }

    pub fn inject_atomic<T0,T1>(&self, buf: &mut [T0], count : usize, dest_addr: Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::Op) -> isize{
        let ret = unsafe{ libfabric_sys::inlined_fi_inject_atomic(self.c_ep, buf.as_mut_ptr()  as *mut std::ffi::c_void, count, dest_addr, addr, key, datatype, op)};
        ret
    }

    pub fn fetch_atomic<T0,T1>(&self, buf: &mut [T0], count : usize, desc: &mut [T1], res: &mut [T0], res_desc: &mut [T1], dest_addr: Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::Op) -> isize{
        let ret = unsafe{ libfabric_sys::inlined_fi_fetch_atomic(self.c_ep, buf.as_mut_ptr()  as *mut std::ffi::c_void, count, desc.as_mut_ptr()  as *mut std::ffi::c_void, res.as_mut_ptr()  as *mut std::ffi::c_void, res_desc.as_mut_ptr() as *mut std::ffi::c_void, dest_addr, addr, key, datatype, op, std::ptr::null_mut())};
        ret
    }


    pub fn fetch_atomicv<T0,T1>(&self, iov: &crate::Ioc, desc: &mut [T1], count : usize, resultv: &mut crate::Ioc,  res_desc: &mut [T1], res_count : usize, dest_addr: Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::Op) -> isize{
        let ret = unsafe{ libfabric_sys::inlined_fi_fetch_atomicv(self.c_ep, iov.get(), desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, count, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, dest_addr, addr, key, datatype, op, std::ptr::null_mut())};
        ret
    }

    pub fn fetch_atomicmsg<T0>(&self, msg: &crate::MsgAtomic,  resultv: &mut crate::Ioc,  res_desc: &mut [T0], res_count : usize, flags: u64) -> isize {
        let ret = unsafe{ libfabric_sys::inlined_fi_fetch_atomicmsg(self.c_ep, msg.c_msg_atomic, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, flags) };
        ret
    }

    pub fn compare_atomic<T0, T1>(&self, buf: &mut [T0], count : usize, desc: &mut [T1], compare: &mut [T0], compare_desc: &mut [T1], 
            result: &mut [T0], result_desc: &mut [T1], dest_addr: Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::Op) -> isize {
        let ret = unsafe {libfabric_sys::inlined_fi_compare_atomic(self.c_ep, buf.as_mut_ptr()  as *mut std::ffi::c_void, count, desc.as_mut_ptr()  as *mut std::ffi::c_void, compare.as_mut_ptr()  as *mut std::ffi::c_void, 
            compare_desc.as_mut_ptr()  as *mut std::ffi::c_void, result.as_mut_ptr()  as *mut std::ffi::c_void, result_desc.as_mut_ptr()  as *mut std::ffi::c_void, dest_addr, addr, key, datatype, op, std::ptr::null_mut())};
        
        ret
    }

    pub fn compare_atomicv<T0>(&self, iov: &crate::Ioc, desc: &mut [T0], count : usize, comparetv: &mut crate::Ioc,  compare_desc: &mut [T0], compare_count : usize, 
        resultv: &mut crate::Ioc,  res_desc: &mut [T0], res_count : usize, dest_addr: Address, addr: u64, key: u64, datatype: crate::DataType, op: crate::Op) -> isize {
        let ret = unsafe {libfabric_sys::inlined_fi_compare_atomicv(self.c_ep, iov.get(), desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, count, comparetv.get_mut(), compare_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, compare_count, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, dest_addr, addr, key, datatype, op, std::ptr::null_mut())};
        
        ret
    }

    pub fn compare_atomicmsg<T0>(&self, msg: &crate::MsgAtomic, comparev: &crate::Ioc, compare_desc: &mut [T0], compare_count : usize, resultv: &mut crate::Ioc,  res_desc: &mut [T0], res_count : usize, flags: u64) -> isize {
        let res: isize = unsafe { libfabric_sys::inlined_fi_compare_atomicmsg(self.c_ep, msg.c_msg_atomic, comparev.get(), compare_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, compare_count, resultv.get_mut(), res_desc.as_mut_ptr()  as *mut *mut std::ffi::c_void, res_count, flags) };

        res
    }

    pub fn atomicvalid(&self, datatype: crate::DataType, op: crate::Op) -> usize {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_atomicvalid(self.c_ep, datatype, op, &mut count as *mut usize)};

        if err != 0 {
            panic!("fi_atomicvalid failed {}", err);
        }

        count
    }

    pub fn fetch_atomicvalid(&self, datatype: crate::DataType, op: crate::Op) -> usize {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_fetch_atomicvalid(self.c_ep, datatype, op, &mut count as *mut usize)};

        if err != 0 {
            panic!("fi_fetch_atomicvalid failed {}", err);
        }

        count
    }

    pub fn compare_atomicvalid(&self, datatype: crate::DataType, op: crate::Op) -> usize {
        let mut count: usize = 0;
        let err = unsafe { libfabric_sys:: inlined_fi_compare_atomicvalid(self.c_ep, datatype, op, &mut count as *mut usize)};

        if err != 0 {
            panic!("fi_fetch_atomicvalid failed {}", err);
        }

        count
    }

    pub fn join<T0,T1>(&self, addr: &T0, flags: u64, context: &mut T1 ) -> crate::Mc {
        crate::Mc::new(self, addr, flags, context)
    }
}

impl FID for Endpoint {
    fn fid(&self) -> *mut libfabric_sys::fid {
        unsafe { &mut (*self.c_ep).fid }
    }
}


pub struct Endpoint {
    pub(crate) c_ep: *mut libfabric_sys::fid_ep,
}

