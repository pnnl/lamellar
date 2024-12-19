use crate::domain::DomainBase;
use crate::enums::{MrMode, MrRegOpt};
use crate::eq::Event;
use crate::fid::{AsRawTypedFid, AsTypedFid, Fid, OwnedMrFid};
use crate::mr::{MemoryRegionAttr, MemoryRegionBuilder};
use crate::{
    enums::MrAccess,
    mr::{MemoryRegion, MemoryRegionImpl},
};
use crate::{Context, MyOnceCell, MyRc};

use super::eq::AsyncReadEq;

impl MemoryRegionImpl {
    #[allow(dead_code)]
    pub(crate) async fn from_buffer_async<T>(
        domain: &MyRc<crate::async_::domain::AsyncDomainImpl>,
        buf: &[T],
        access: &MrAccess,
        requested_key: u64,
        flags: MrMode,
        ctx: &mut Context,
    ) -> Result<(Event, Self), crate::error::Error> {

        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let c_mr_ptr: *mut *mut libfabric_sys::fid_mr = &mut c_mr;
        let err = unsafe {
            libfabric_sys::inlined_fi_mr_reg(
                domain.as_typed_fid().as_raw_typed_fid(),
                buf.as_ptr().cast(),
                std::mem::size_of_val(buf),
                access.as_raw().into(),
                0,
                requested_key,
                flags.as_raw() as u64,
                c_mr_ptr,
                ctx.inner_mut(),
            )
        };

        if err == 0 {
            if let Some((eq, mem_reg)) = domain._eq_rc.get() {
                if !mem_reg {
                    panic!("Domain has to be bound with async_mem_reg to an event queue to use async memory registration");
                } else {
                    // let res = crate::async_::eq::EventQueueFut::<{libfabric_sys::FI_MR_COMPLETE}>::new(std::ptr::null_mut(), eq.clone(),&mut async_ctx as *mut AsyncCtx as usize).await?;
                    let res = eq
                        .async_event_wait(
                            libfabric_sys::FI_MR_COMPLETE,
                            Fid(0),
                            Some(ctx),
                        )
                        .await?;

                    return Ok((
                        res,
                        Self {
                            c_mr: OwnedMrFid::from(c_mr),
                            _domain_rc: domain.clone(),
                            bound_cntr: MyOnceCell::new(),
                            bound_ep: MyOnceCell::new(),
                        },
                    ));
                }
            } else {
                panic!("Domain has to be bound with async_mem_reg to an event queue to use async memory registration");
            }
        }

        Err(crate::error::Error::from_err_code(
            (-err).try_into().unwrap(),
        ))
    }

    #[allow(dead_code)]
    pub(crate) async fn from_attr_async(
        domain: &MyRc<crate::async_::domain::AsyncDomainImpl>,
        mut attr: MemoryRegionAttr,
        flags: MrRegOpt,
        ctx: &mut Context
    ) -> Result<(Event, Self), crate::error::Error> {
        // [TODO] Add context version

        attr.c_attr.context = ctx.inner_mut();

        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let c_mr_ptr: *mut *mut libfabric_sys::fid_mr = &mut c_mr;
        let err = unsafe {
            libfabric_sys::inlined_fi_mr_regattr(
                domain.as_typed_fid().as_raw_typed_fid(),
                attr.get(),
                flags.as_raw(),
                c_mr_ptr,
            )
        };

        if err == 0 {
            if let Some((eq, mem_reg)) = domain._eq_rc.get() {
                if !mem_reg {
                    panic!("Domain has to be bound with async_mem_reg to an event queue to use async memory registration");
                } else {
                    // let res = crate::async_::eq::EventQueueFut::<{libfabric_sys::FI_MR_COMPLETE}>::new(std::ptr::null_mut(), eq.clone(), attr.c_attr.context as usize).await?;
                    let res = eq
                        .async_event_wait(
                            libfabric_sys::FI_MR_COMPLETE,
                            Fid(0),
                            Some(ctx),
                        )
                        .await?;

                    return Ok((
                        res,
                        Self {
                            c_mr: OwnedMrFid::from(c_mr),
                            _domain_rc: domain.clone(),
                            bound_cntr: MyOnceCell::new(),
                            bound_ep: MyOnceCell::new(),
                        },
                    ));
                }
            } else {
                panic!("Domain has to be bound with async_mem_reg to an event queue to use async memory registration");
            }
        }

        Err(crate::error::Error::from_err_code(
            (-err).try_into().unwrap(),
        ))
    }

    #[allow(dead_code)]
    pub(crate) async fn from_iovec_async<'a>(
        domain: &'a MyRc<crate::async_::domain::AsyncDomainImpl>,
        iov: &[crate::iovec::IoVec<'a>],
        access: &MrAccess,
        requested_key: u64,
        flags: MrRegOpt,
        ctx: &mut Context,
    ) -> Result<(Event, Self), crate::error::Error> {
        // let mut async_ctx = AsyncCtx { user_ctx };
        let mut c_mr: *mut libfabric_sys::fid_mr = std::ptr::null_mut();
        let c_mr_ptr: *mut *mut libfabric_sys::fid_mr = &mut c_mr;
        let err = unsafe {
            libfabric_sys::inlined_fi_mr_regv(
                domain.as_typed_fid().as_raw_typed_fid(),
                iov.as_ptr().cast(),
                iov.len(),
                access.as_raw().into(),
                0,
                requested_key,
                flags.as_raw(),
                c_mr_ptr,
                ctx.inner_mut(),
            )
        };

        if err == 0 {
            if let Some((eq, mem_reg)) = domain._eq_rc.get() {
                if !mem_reg {
                    panic!("Domain has to be bound with async_mem_reg to an event queue to use async memory registration");
                } else {
                    // let res = crate::async_::eq::EventQueueFut::<{libfabric_sys::FI_MR_COMPLETE}>::new(std::ptr::null_mut(), eq.clone(), &mut async_ctx as *mut AsyncCtx as usize).await?;
                    let res = eq
                        .async_event_wait(
                            libfabric_sys::FI_MR_COMPLETE,
                            Fid(0),
                            Some(ctx),
                        )
                        .await?;

                    return Ok((
                        res,
                        Self {
                            c_mr: OwnedMrFid::from(c_mr),
                            _domain_rc: domain.clone(),
                            bound_cntr: MyOnceCell::new(),
                            bound_ep: MyOnceCell::new(),
                        },
                    ));
                }
            } else {
                panic!("Domain has to be bound with async_mem_reg to an event queue to use async memory registration");
            }
        }

        Err(crate::error::Error::from_err_code(
            (-err).try_into().unwrap(),
        ))
    }
}

impl MemoryRegion {
    #[allow(dead_code)]
    pub(crate) async fn from_buffer_async<T>(
        domain: &crate::async_::domain::Domain,
        buf: &[T],
        access: &MrAccess,
        requested_key: u64,
        flags: MrMode,
        ctx: &mut Context,
    ) -> Result<(Event, Self), crate::error::Error> {
        let (event, mr) = MemoryRegionImpl::from_buffer_async(
            &domain.inner,
            buf,
            access,
            requested_key,
            flags,
            ctx,
        )
        .await?;
        Ok((
            event,
            Self {
                inner: MyRc::new(mr),
            },
        ))
    }

    #[allow(dead_code)]
    pub(crate) async fn from_attr_async(
        domain: &crate::async_::domain::Domain,
        attr: MemoryRegionAttr,
        flags: MrRegOpt,
        ctx: &mut Context,
    ) -> Result<(Event, Self), crate::error::Error> {
        // [TODO] Add context version
        let (event, mr) = MemoryRegionImpl::from_attr_async(&domain.inner, attr, flags, ctx).await?;
        Ok((
            event,
            Self {
                inner: MyRc::new(mr),
            },
        ))
    }

    #[allow(dead_code)]
    async fn from_iovec_async<'a>(
        domain: &'a crate::async_::domain::Domain,
        iov: &[crate::iovec::IoVec<'a>],
        access: &MrAccess,
        requested_key: u64,
        flags: MrRegOpt,
        ctx: &mut Context,
    ) -> Result<(Event, Self), crate::error::Error> {
        let (event, mr) = MemoryRegionImpl::from_iovec_async(
            &domain.inner,
            iov,
            access,
            requested_key,
            flags,
            ctx,
        )
        .await?;
        Ok((
            event,
            Self {
                inner: MyRc::new(mr),
            },
        ))
    }
}

impl<'a> MemoryRegionBuilder<'a> {
    /// Constructs a new [MemoryRegion] with the configurations requested so far.
    ///
    /// Corresponds to creating a `fi_mr_attr`, setting its fields to the requested ones,
    /// and passign it to `fi_mr_regattr`.
    #[allow(unreachable_code, unused)]
    pub async fn build_async(
        self,
        domain: &DomainBase<dyn AsyncReadEq>,
        ctx: &mut Context,
    ) -> Result<(Event, MemoryRegion), crate::error::Error> {
        panic!("Async memory registration is currently not supported due to a potential bug in libfabric");
        self.mr_attr.iov(&self.iovs);
        MemoryRegion::from_attr_async(domain, self.mr_attr, self.flags, ctx).await
    }
}

//================== Memory Region tests ==================//
// #[cfg(test)]
// mod tests {
//     use crate::{info::{Info, InfoHints}, enums::MrAccess, domain::DomainBuilder, async_::eq::EventQueueBuilder};
//     use super::MemoryRegionBuilder;

//     pub fn ft_alloc_bit_combo(fixed: u64, opt: u64) -> Vec<u64> {
//         let bits_set = |mut val: u64 | -> u64 { let mut cnt = 0; while val > 0 {  cnt += 1 ; val &= val-1; } cnt };
//         let num_flags = bits_set(opt) + 1;
//         let len = 1 << (num_flags - 1) ;
//         let mut flags = vec![0_u64 ; num_flags as usize];
//         let mut num_flags = 0;
//         for i in 0..8*std::mem::size_of::<u64>(){
//             if opt >> i & 1 == 1 {
//                 flags[num_flags] = 1 << i;
//                 num_flags += 1;
//             }
//         }
//         let mut combos = Vec::new();

//         for index in 0..len {
//             combos.push(fixed);
//             for (i, val) in flags.iter().enumerate().take(8*std::mem::size_of::<u64>()){
//                 if index >> i & 1 == 1 {
//                     combos[index] |= val;
//                 }
//             }
//         }

//         combos
//     }
//     pub struct TestSizeParam(pub u64, pub u64);
//     pub const DEF_TEST_SIZES: [TestSizeParam; 6] = [TestSizeParam(1 << 0,0), TestSizeParam(1 << 1,0), TestSizeParam(1 << 2,0), TestSizeParam(1 << 3,0), TestSizeParam(1 << 4,0), TestSizeParam(1 << 5,0) ];

//     #[test]
//     fn mr_reg() {
//         let ep_attr = crate::ep::EndpointAttr::new();
//         let mut dom_attr = crate::domain::DomainAttr::new();
//             dom_attr
//             .mode(crate::enums::Mode::all())
//             .mr_mode(crate::enums::MrMode::new().basic().scalable().local().inverse());

//         let hints = InfoHints::new()
//             .caps(crate::infocapsoptions::InfoCaps::new().msg().rma())
//             .ep_attr(ep_attr)
//             .domain_attr(dom_attr);

//         let info = Info::new().hints(&hints).request().unwrap();
//         let entries = info.get();

//         if !entries.is_empty() {

//             let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
//             let eq = EventQueueBuilder::new(&fab).build().unwrap();
//             let domain = DomainBuilder::new(&fab, &entries[0]).build_and_bind_async(&eq, true).unwrap();

//             let mut mr_access: u64 = 0;
//             if entries[0].get_mode().is_local_mr() || entries[0].get_domain_attr().get_mr_mode().is_local() {

//                 if entries[0].get_caps().is_msg() || entries[0].get_caps().is_tagged() {
//                     let mut on = false;
//                     if entries[0].get_caps().is_send() {
//                         mr_access |= libfabric_sys::FI_SEND as u64;
//                         on = true;
//                     }
//                     if entries[0].get_caps().is_recv() {
//                         mr_access |= libfabric_sys::FI_RECV  as u64 ;
//                         on = true;
//                     }
//                     if !on {
//                         mr_access |= libfabric_sys::FI_SEND as u64 & libfabric_sys::FI_RECV as u64;
//                     }
//                 }
//             }
//             else if entries[0].get_caps().is_rma() || entries[0].get_caps().is_atomic() {
//                 if entries[0].get_caps().is_remote_read() || !(entries[0].get_caps().is_read() || entries[0].get_caps().is_write() || entries[0].get_caps().is_remote_write()) {
//                     mr_access |= libfabric_sys::FI_REMOTE_READ as u64 ;
//                 }
//                 else {
//                     mr_access |= libfabric_sys::FI_REMOTE_WRITE as u64 ;
//                 }
//             }

//             let combos = ft_alloc_bit_combo(0, mr_access);

//             for test in &DEF_TEST_SIZES {
//                 let buff_size = test.0;
//                 let buf = vec![0_u64; buff_size as usize];
//                 for combo in &combos {
//                     let _mr = MemoryRegionBuilder::new(&buf)
//                         // .iov(std::slice::from_mut(&mut IoVec::from_slice_mut(&mut buf)))
//                         .access(&MrAccess::from_value(*combo as u32))
//                         .requested_key(0xC0DE)

//                         .build_async(&domain).await
//                         .unwrap();
//                     // mr.close().unwrap();
//                 }
//             }

//             // domain.close().unwrap();
//             // fab.close().unwrap();
//         }
//         else {
//             panic!("No capable fabric found!");
//         }
//     }

//     fn mr_regv() {
//         let ep_attr = crate::ep::EndpointAttr::new();
//         let mut dom_attr = crate::domain::DomainAttr::new();
//             dom_attr
//             .mode(!0)
//             .mr_mode(!(crate::enums::MrMode::BASIC.as_raw() | crate::enums::MrMode::scalable(self).as_raw() | crate::enums::MrMode::LOCAL.as_raw() ) as i32 );
//         let mut hints = crate::InfoHints::new();
//             hints
//             .caps(crate::InfoCaps::new().msg().rma())
//             .ep_attr(ep_attr)
//             .domain_attr(dom_attr);
//         let info = crate::Info::with_hints(hints);
//         let entries: Vec<crate::InfoEntry> = info.get();
//         if entries.len() > 0 {

//             let mut eq_attr = crate::eq::EventQueueAttr::new();
//                 eq_attr
//                 .size(32)
//                 .flags(libfabric_sys::FI_WRITE.into())
//                 .wait_obj(crate::enums::WaitObj::FD);
//             let mut fab = crate::fabric::Fabric::new(entries[0].fabric_attr.clone());
//             let mut eq = fab.eq_open(eq_attr);
//             let mut domain = fab.domain(&entries[0]);

//             let mut mr_access: u64 = 0;
//             if entries[0].get_mode() & libfabric_sys::FI_LOCAL_MR == libfabric_sys::FI_LOCAL_MR || entries[0].get_domain_attr().get_mr_mode() as u32 & libfabric_sys::FI_MR_LOCAL == libfabric_sys::FI_MR_LOCAL {

//                 if entries[0].caps.is_msg() || entries[0].caps.is_tagged() {
//                     let mut on = false;
//                     if entries[0].caps.is_send() {
//                         mr_access |= libfabric_sys::FI_SEND as u64;
//                         on = true;
//                     }
//                     if entries[0].caps.is_recv() {
//                         mr_access |= libfabric_sys::FI_RECV  as u64 ;
//                         on = true;
//                     }
//                     if !on {
//                         mr_access |= libfabric_sys::FI_SEND as u64 & libfabric_sys::FI_RECV as u64;
//                     }
//                 }
//             }
//             else {
//                 if entries[0].caps.is_rma() || entries[0].caps.is_atomic() {
//                     if entries[0].caps.is_remote_read() || !(entries[0].caps.is_read() || entries[0].caps.is_write() || entries[0].caps.is_remote_write()) {
//                         mr_access |= libfabric_sys::FI_REMOTE_READ as u64 ;
//                     }
//                     else {
//                         mr_access |= libfabric_sys::FI_REMOTE_WRITE as u64 ;
//                     }
//                 }
//             }

//             let iovec = IoVec::new();
//             if mr_access != 0 {
//                 let i = 0;
//                 let buf = vec![0; DEF_TEST_SIZES[DEF_TEST_SIZES.len()-1].0 as usize];
//                 while i < DEF_TEST_SIZES.len() && entries[0].get_domain_attr().get_mr_iov_limit() < DEF_TEST_SIZES[i].0 {
//                     let n = DEF_TEST_SIZES[i].0;
//                     let base = &buf[0..];

//                 }
//             }
//             else {
//                 domain.close().unwrap();
//                 eq.close().unwrap();
//                 fab.close().unwrap();
//                 panic!("mr access == 0");
//             }

//             domain.close().unwrap();
//             eq.close().unwrap();
//             fab.close().unwrap();
//         }
//         else {
//             panic!("No capable fabric found!");
//         }
//     }
// }

// #[cfg(test)]
// mod libfabric_lifetime_tests {
//     use crate::{info::{Info, InfoHints}, enums::MrAccess};
//     use crate::async_::domain::DomainBuilder;
//     use super::MemoryRegionBuilder;

//     #[test]
//     fn mr_drops_before_domain() {
//         let ep_attr = crate::ep::EndpointAttr::new();
//         let mut dom_attr = crate::domain::DomainAttr::new();
//             dom_attr
//             .mode(crate::enums::Mode::all())
//             .mr_mode(crate::enums::MrMode::new().basic().scalable().local().inverse());

//         let hints = InfoHints::new()
//             .caps(crate::infocapsoptions::InfoCaps::new().msg().rma())
//             .ep_attr(ep_attr)
//             .domain_attr(dom_attr);

//         let info = Info::new().hints(&hints).request().unwrap();
//         let entries = info.get();

//         if !entries.is_empty() {

//             let fab = crate::fabric::FabricBuilder::new(&entries[0]).build().unwrap();
//             let domain = DomainBuilder::new(&fab, &entries[0]).build().unwrap();

//             let mut mr_access: u64 = 0;
//             if entries[0].get_mode().is_local_mr() || entries[0].get_domain_attr().get_mr_mode().is_local() {

//                 if entries[0].get_caps().is_msg() || entries[0].get_caps().is_tagged() {
//                     let mut on = false;
//                     if entries[0].get_caps().is_send() {
//                         mr_access |= libfabric_sys::FI_SEND as u64;
//                         on = true;
//                     }
//                     if entries[0].get_caps().is_recv() {
//                         mr_access |= libfabric_sys::FI_RECV  as u64 ;
//                         on = true;
//                     }
//                     if !on {
//                         mr_access |= libfabric_sys::FI_SEND as u64 & libfabric_sys::FI_RECV as u64;
//                     }
//                 }
//             }
//             else if entries[0].get_caps().is_rma() || entries[0].get_caps().is_atomic() {
//                 if entries[0].get_caps().is_remote_read() || !(entries[0].get_caps().is_read() || entries[0].get_caps().is_write() || entries[0].get_caps().is_remote_write()) {
//                     mr_access |= libfabric_sys::FI_REMOTE_READ as u64 ;
//                 }
//                 else {
//                     mr_access |= libfabric_sys::FI_REMOTE_WRITE as u64 ;
//                 }
//             }

//             let combos = super::tests::ft_alloc_bit_combo(0, mr_access);

//             let mut mrs = Vec::new();
//             for test in &super::tests::DEF_TEST_SIZES {
//                 let buff_size = test.0;
//                 let buf = vec![0_u64; buff_size as usize ];
//                 for combo in &combos {
//                     let mr = MemoryRegionBuilder::new(&buf)
//                         .access(&MrAccess::from_value(*combo as u32))
//                         .requested_key(0xC0DE)
//                         .build_async(&domain).await
//                         .unwrap();
//                     mrs.push(mr);
//                     println!("Count = {} \n", std::rc::MyRc::strong_count(&domain.inner));
//                 }
//             }
//             drop(domain);
//             // println!("Count = {} After dropping domain\n", std::rc::MyRc::strong_count(&mrs[0].inner._domain_rc));

//             // domain.close().unwrap();
//             // fab.close().unwrap();
//         }
//         else {
//             panic!("No capable fabric found!");
//         }
//     }
// }
