
pub mod sync_;
pub mod sync_tagged {
    use libfabric::{cq::{Completion, WaitCq}, infocapsoptions::InfoCaps, iovec::{IoVec, IoVecMut}, mr::MemoryRegionBuilder, msg::{MsgTagged, MsgTaggedConnected, MsgTaggedConnectedMut, MsgTaggedMut}};

    use crate::sync_::tests::{enable_ep_mr, handshake, handshake_connectionless, Either};

    fn tsendrecv(server: bool, name: &str, connected: bool, use_context: bool) {
        let ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg().tagged()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg().tagged()))
        };


        {
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            for i in 0..1024 * 2 {
                let v = (i % 256) as u8;
                reg_mem[i] = v;
            }
        }

        let data = Some(128u64);
        if server {
            // Send a single buffer
            ofi.tsend(0..512, 10, data, use_context);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // match entry {
            //     Completion::Tagged(entry) => {assert_eq!(entry[0].data(), data.unwrap()); assert_eq!(entry[0].tag(), 10)},
            //     _ => panic!("Unexpected CQ entry format"),
            // }

            // assert!(std::mem::size_of_val(&reg_mem[..128]) <= ofi.info_entry.tx_attr().inject_size());

            // Inject a buffer
            ofi.tsend(0..128, 1, data, use_context);
            // No cq.sread since inject does not generate completions

            // // Send single Iov
            let reg_mem = ofi.reg_mem.borrow();
            let borrow = ofi.mr.borrow();
            let mr = borrow.as_ref();

            let desc:Option<[libfabric::mr::MemoryRegionDesc<'_>; 2]>  = if mr.is_none() {
                None
            }
            else {
                Some([mr.unwrap().descriptor(), mr.unwrap().descriptor()])
            };

            let desc_ref: Option<&[libfabric::mr::MemoryRegionDesc<'_>]> = desc.as_ref().map(|d| &d[..]);
            let iov = [IoVec::from_slice(&reg_mem[..512])];
            ofi.tsendv(&iov, desc_ref, 2, use_context);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Send multi Iov
            let iov = [
                IoVec::from_slice(&reg_mem[..512]),
                IoVec::from_slice(&reg_mem[512..1024]),
            ];
            ofi.tsendv(&iov, desc_ref, 3, use_context);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            drop(reg_mem);
            // Send a single buffer
            if let Some(mr) = mr {
                ofi.tsend_mr(unsafe{&mr.slice(..512)}, 0, None, false);
            }
            else {
                ofi.tsend(0..512, 0, None, false);
            }
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            
        } else {
            let expected: Vec<_> = (0..1024 * 2)
                .map(|v: usize| (v % 256) as u8)
                .collect();
            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);
            
            // Receive a single buffer
            ofi.trecv(0..512, 10, use_context);
            let entry = ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            match entry {
                Completion::Tagged(entry) => {
                    assert_eq!(entry[0].data(), data.unwrap());
                    assert_eq!(entry[0].tag(), 10)
                }
                _ => panic!("Unexpected CQ entry format"),
            }
            assert_eq!(ofi.reg_mem.borrow()[..512], expected[..512]);

            // Receive inject
            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);
            ofi.trecv(0..128, 1, use_context);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(ofi.reg_mem.borrow()[..128], expected[..128]);

            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);
            // // Receive into a single Iov
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            let mut borrow = ofi.mr.borrow_mut();
            let mut mr: Option<&mut libfabric::mr::MemoryRegion> = borrow.as_mut();


            let desc:Option<[libfabric::mr::MemoryRegionDesc<'_>; 2]>  = if mr.is_none() {
                None
            }
            else {
                Some([mr.as_ref().unwrap().descriptor(), mr.as_ref().unwrap().descriptor()])
            };

            let desc_ref = desc.as_ref().map(|d| &d[..]);
            let iov = [IoVecMut::from_slice(&mut reg_mem[..512])];
            ofi.trecvv(&iov, desc_ref, 2, use_context);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(reg_mem[..512], expected[..512]);

            reg_mem.iter_mut().for_each(|v| *v = 0);

            // // Receive into multiple Iovs
            let (mem0, mem1) = reg_mem[..1024].split_at_mut(512);
            let iov = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
            ofi.trecvv(&iov, desc_ref, 3, use_context);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();

            assert_eq!(mem0, &expected[..512]);
            assert_eq!(mem1, &expected[512..1024]);

            reg_mem.iter_mut().for_each(|v| *v = 0);
            drop(reg_mem);
            // Send a single buffer
            if let Some(ref mut mr_) = mr {
                ofi.trecv_mr(unsafe{&mut mr_.slice_mut(..512)}, 0, false);
            }
            else {
                drop(borrow);
                ofi.trecv(0..512, 0, false);
            }
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();

            assert_eq!(ofi.reg_mem.borrow()[..512], expected[..512]);
        }
    }

    #[test]
    fn tsendrecv0() {
        tsendrecv(true, "tsendrecv0", false, false);
    }

    #[test]
    fn tsendrecv1() {
        tsendrecv(false, "tsendrecv0", false, false);
    }

    #[test]
    fn conn_tsendrecv0() {
        tsendrecv(true, "conn_tsendrecv0", true, false);
    }

    #[test]
    fn conn_tsendrecv1() {
        tsendrecv(false, "conn_tsendrecv0", true, false);
    }

    // #[test]
    // fn context_tsendrecv0() {
    //     tsendrecv(true, "tsendrecv0", false, true);
    // }

    // #[test]
    // fn context_tsendrecv1() {
    //     tsendrecv(false, "tsendrecv0", false, true);
    // }

    // #[test]
    // fn context_conn_tsendrecv0() {
    //     tsendrecv(true, "conn_tsendrecv0", true, true);
    // }

    // #[test]
    // fn context_conn_tsendrecv1() {
    //     tsendrecv(false, "conn_tsendrecv0", true, true);
    // }

    // #[test]
    // fn context_sendrecvmsg0() {
    //     sendrecvmsg(true, "sendrecvmsg0", false, true);
    // }

    // #[test]
    // fn context_sendrecvmsg1() {
    //     sendrecvmsg(false, "sendrecvmsg0", false, true);
    // }

    // #[test]
    // fn context_conn_sendrecvmsg0() {
    //     sendrecvmsg(true, "conn_sendrecvmsg0", true, true);
    // }

    // #[test]
    // fn context_conn_sendrecvmsg1() {
    //     sendrecvmsg(false, "conn_sendrecvmsg0", true, true);
    // }



    fn tsendrecvmsg(server: bool, name: &str, connected: bool, use_context: bool) {
        let ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg().tagged()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg().tagged()))
        };

        let mut reg_mem: Vec<_> = (0..1024 * 2)
            .map(|v: usize| (v % 256) as u8)
            .collect();
        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .build(&ofi.domain)
            .unwrap();
        let mr = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };

        let descs = [mr.descriptor(), mr.descriptor()];
        let desc0 = Some(mr.descriptor());
        let mapped_addr = ofi.mapped_addr.clone();
        let data = Some(128);
        let mut ctx = ofi.info_entry.allocate_context();
        if server {
            // Single iov message
            let (mem0, mem1) = (&reg_mem[..512], &reg_mem[1024..1536]);
            let iov0 = IoVec::from_slice(mem0);
            let iov1 = IoVec::from_slice(mem1);
            let msg = if connected {
                Either::Right(MsgTaggedConnected::from_iov(
                    &iov0,
                    Some(&descs[0]),
                    data,
                    0,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTagged::from_iov(
                    &iov0,
                    Some(&descs[0]),
                    &mapped_addr.as_ref().unwrap()[1],
                    data,
                    0,
                    None,
                    &mut ctx,
                ))
            };
            ofi.tsendmsg(&msg, use_context);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // let entry =
            // match entry {
            //     Completion::Tagged(entry) => assert_eq!(entry[0].data(), 128),
            //     _ => panic!("Unexpected CQ entry format"),
            // }

            // Multi iov message with stride
            let iovs = [iov0, iov1];
            let msg = if connected {
                Either::Right(MsgTaggedConnected::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    None,
                    1,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTagged::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    1,
                    None,
                    &mut ctx,
                ))
            };

            ofi.tsendmsg(&msg, use_context);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Single iov message
            let msg = if connected {
                Either::Right(MsgTaggedConnected::from_iov(
                    &iovs[0],
                    desc0.as_ref(),
                    None,
                    2,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTagged::from_iov(
                    &iovs[0],
                    desc0.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    2,
                    None,
                    &mut ctx,
                ))
            };

            ofi.tsendmsg(&msg, use_context);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            let msg = if connected {
                Either::Right(MsgTaggedConnected::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    None,
                    3,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTagged::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    3,
                    None,
                    &mut ctx,
                ))
            };
            ofi.tsendmsg(&msg, use_context);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        } else {
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

            // Receive a single message in a single buffer
            let mut iov = IoVecMut::from_slice(mem0);
            let msg = if connected {
                Either::Right(MsgTaggedConnectedMut::from_iov(
                    &mut iov,
                    desc0.as_ref(),
                    None,
                    0,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTaggedMut::from_iov(
                    &mut iov,
                    desc0.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    0,
                    None,
                    &mut ctx,
                ))
            };

            ofi.trecvmsg(&msg, use_context);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            // let entry =
            // match entry {
            //     Completion::Tagged(entry) => assert_eq!(entry[0].data(), 128),
            //     _ => panic!("Unexpected CQ entry format"),
            // }
            assert_eq!(mem0.len(), expected[..512].len());
            assert_eq!(mem0, &expected[..512]);

            // Receive a multi iov message in a single buffer
            let mut iov = IoVecMut::from_slice(&mut mem1[..1024]);
            let msg = if connected {
                Either::Right(MsgTaggedConnectedMut::from_iov(
                    &mut iov,
                    desc0.as_ref(),
                    None,
                    1,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTaggedMut::from_iov(
                    &mut iov,
                    desc0.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    1,
                    None,
                    &mut ctx,
                ))
            };

            ofi.trecvmsg(&msg, use_context);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(mem1[..1024], expected);

            // Receive a single iov message into two buffers
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let iov = IoVecMut::from_slice(&mut mem0[..256]);
            let iov1 = IoVecMut::from_slice(&mut mem1[..256]);
            let mut iovs = [iov, iov1];
            let msg = if connected {
                Either::Right(MsgTaggedConnectedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    None,
                    2,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTaggedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    2,
                    None,
                    &mut ctx,
                ))
            };

            ofi.trecvmsg(&msg, use_context);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(mem0[..256], expected[..256]);
            assert_eq!(mem1[..256], expected[256..512]);

            // Receive a two iov message into two buffers
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let iov = IoVecMut::from_slice(&mut mem0[..512]);
            let iov1 = IoVecMut::from_slice(&mut mem1[..512]);
            let mut iovs = [iov, iov1];
            let msg = if connected {
                Either::Right(MsgTaggedConnectedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    None,
                    3,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTaggedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    3,
                    None,
                    &mut ctx,
                ))
            };

            ofi.trecvmsg(&msg, use_context);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(mem0[..512], expected[..512]);
            assert_eq!(mem1[..512], expected[512..1024]);
        }
    }

    #[test]
    fn tsendrecvmsg0() {
        tsendrecvmsg(true, "tsendrecvmsg0", false, false);
    }

    #[test]
    fn tsendrecvmsg1() {
        tsendrecvmsg(false, "tsendrecvmsg0", false, false);
    }

    // #[test]
    // fn conn_tsendrecvmsg0() {
    //     tsendrecvmsg(true, "conn_tsendrecvmsg0", true, false);
    // }

    // #[test]
    // fn conn_tsendrecvmsg1() {
    //     tsendrecvmsg(false, "conn_tsendrecvmsg0", true, false);
    // }

    // #[test]
    // fn context_tsendrecvmsg0() {
    //     tsendrecvmsg(true, "tsendrecvmsg0", false, true);
    // }

    // #[test]
    // fn context_tsendrecvmsg1() {
    //     tsendrecvmsg(false, "tsendrecvmsg0", false, true);
    // }

    // #[test]
    // fn context_conn_tsendrecvmsg0() {
    //     tsendrecvmsg(true, "conn_tsendrecvmsg0", true, true);
    // }

    // #[test]
    // fn context_conn_tsendrecvmsg1() {
    //     tsendrecvmsg(false, "conn_tsendrecvmsg0", true, true);
    // }
}

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
pub mod async_;

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
pub mod async_tagged {
    use libfabric::{infocapsoptions::InfoCaps, iovec::{IoVec, IoVecMut}, mr::MemoryRegionBuilder, msg::{MsgTagged, MsgTaggedConnected, MsgTaggedConnectedMut, MsgTaggedMut}};

    use crate::async_::{enable_ep_mr, handshake, handshake_connectionless, Either};


    fn tsendrecv(server: bool, name: &str, connected: bool) {
        let ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg().tagged()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg().tagged()))
        };

        {
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            for i in 0..1024 * 2 {
                let v = (i % 256) as u8;
                reg_mem[i] = v;
            }
        }

        let data = Some(128u64);

        if server {
            // Send a single buffer
            ofi.tsend(0..512, 10, data);
            // match entry {
            //     Completion::Tagged(entry) => {assert_eq!(entry[0].data(), data.unwrap()); assert_eq!(entry[0].tag(), 10)},
            //     _ => panic!("Unexpected CQ entry format"),
            // }

            assert!(
                std::mem::size_of_val(&ofi.reg_mem.borrow()[0..128]) <= ofi.info_entry.tx_attr().inject_size()
            );

            // Inject a buffer
            ofi.tsend(0..128, 1, data);
            // No cq.sread since inject does not generate completions

            let reg_mem = ofi.reg_mem.borrow();
            let borrow = ofi.mr.borrow();
            let mr = borrow.as_ref();

            let desc:Option<[libfabric::mr::MemoryRegionDesc<'_>; 2]>  = if mr.is_none() {
                None
            }
            else {
                Some([mr.unwrap().descriptor(), mr.unwrap().descriptor()])
            };
            let desc_ref: Option<&[libfabric::mr::MemoryRegionDesc<'_>]> = desc.as_ref().map(|d| &d[..]);
            // Send single Iov
            let iov = [IoVec::from_slice(&reg_mem[..512])];
            ofi.tsendv(&iov, desc_ref, 2);

            // Send multi Iov
            let iov = [
                IoVec::from_slice(&reg_mem[..512]),
                IoVec::from_slice(&reg_mem[512..1024]),
            ];
            ofi.tsendv(&iov, desc_ref, 3);
        } else {
            let expected: Vec<_> = (0..1024 * 2)
                .into_iter()
                .map(|v: usize| (v % 256) as u8)
                .collect();
            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);

            // Receive a single buffer
            ofi.trecv(0..512, 10);

            assert_eq!(ofi.reg_mem.borrow()[..512], expected[..512]);

            // Receive inject
            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);
            ofi.trecv(0..128, 1);
            assert_eq!(ofi.reg_mem.borrow()[..128], expected[..128]);

            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);
            // // Receive into a single Iov
            
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            let mut borrow = ofi.mr.borrow_mut();
            let mr: Option<&mut libfabric::mr::MemoryRegion> = borrow.as_mut();


            let desc:Option<[libfabric::mr::MemoryRegionDesc<'_>; 2]>  = if mr.is_none() {
                None
            }
            else {
                Some([mr.as_ref().unwrap().descriptor(), mr.as_ref().unwrap().descriptor()])
            };

            let desc_ref = desc.as_ref().map(|d| &d[..]);
            let iov = [IoVecMut::from_slice(&mut reg_mem[..512])];

            ofi.trecvv(&iov, desc_ref, 2);
            assert_eq!(reg_mem[..512], expected[..512]);

            reg_mem.iter_mut().for_each(|v| *v = 0);

            // // Receive into multiple Iovs
            let (mem0, mem1) = reg_mem[..1024].split_at_mut(512);
            let iov = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
            ofi.trecvv(&iov, desc_ref, 3);

            assert_eq!(mem0, &expected[..512]);
            assert_eq!(mem1, &expected[512..1024]);
        }
    }

    #[test]
    fn async_tsendrecv0() {
        tsendrecv(true, "tsendrecv0", false);
    }

    #[test]
    fn async_tsendrecv1() {
        tsendrecv(false, "tsendrecv0", false);
    }

    #[test]
    fn async_conn_tsendrecv0() {
        tsendrecv(true, "conn_tsendrecv0", true);
    }

    #[test]
    fn async_conn_tsendrecv1() {
        tsendrecv(false, "conn_tsendrecv0", true);
    }


    fn tsendrecvmsg(server: bool, name: &str, connected: bool) {
        let ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg().tagged()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg().tagged()))
        };

        let mut reg_mem: Vec<_> = (0..1024 * 2)
            .into_iter()
            .map(|v: usize| (v % 256) as u8)
            .collect();
        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .build(&ofi.domain)
            .unwrap();
        let mr = match mr {
            libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
            libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => match disabled_mr {
                libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => {
                    enable_ep_mr(&ofi.ep, ep_binding_memory_region)
                }
                libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => {
                    rma_event_memory_region.enable().unwrap()
                }
            },
        };

        let desc = Some(mr.descriptor());
        let descs = [mr.descriptor(), mr.descriptor()];
        let mapped_addr = ofi.mapped_addr.clone();
        let mut ctx = ofi.info_entry.allocate_context();
        let data = Some(128);
        if server {
            // Single iov message
            let (mem0, mem1) = (&reg_mem[..512], &reg_mem[1024..1536]);
            let iov0 = IoVec::from_slice(mem0);
            let iov1 = IoVec::from_slice(mem1);
            let mut msg = if connected {
                Either::Right(MsgTaggedConnected::from_iov(
                    &iov0,
                    desc.as_ref(),
                    data,
                    0,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTagged::from_iov(
                    &iov0,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    data,
                    0,
                    None,
                    &mut ctx,
                ))
            };
            ofi.tsendmsg(&mut msg);

            // Multi iov message with stride
            let iovs = [iov0, iov1];
            let mut msg = if connected {
                Either::Right(MsgTaggedConnected::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    None,
                    1,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTagged::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    1,
                    None,
                    &mut ctx,
                ))
            };

            ofi.tsendmsg(&mut msg);

            // Single iov message
            let mut msg = if connected {
                Either::Right(MsgTaggedConnected::from_iov(
                    &iovs[0],
                    desc.as_ref(),
                    None,
                    2,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTagged::from_iov(
                    &iovs[0],
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    Some(0),
                    2,
                    None,
                    &mut ctx,
                ))
            };

            ofi.tsendmsg(&mut msg);

            let mut msg = if connected {
                Either::Right(MsgTaggedConnected::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    None,
                    3,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTagged::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    3,
                    None,
                    &mut ctx,
                ))
            };
            ofi.tsendmsg(&mut msg);
        } else {
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

            // Receive a single message in a single buffer
            let mut iov = IoVecMut::from_slice(mem0);
            let mut msg = if connected {
                Either::Right(MsgTaggedConnectedMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    None,
                    0,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTaggedMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    0,
                    None,
                    &mut ctx,
                ))
            };

            ofi.trecvmsg(&mut msg);
            assert_eq!(mem0.len(), expected[..512].len());
            assert_eq!(mem0, &expected[..512]);

            // Receive a multi iov message in a single buffer
            let mut iov = IoVecMut::from_slice(&mut mem1[..1024]);
            let mut msg = if connected {
                Either::Right(MsgTaggedConnectedMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    None,
                    1,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTaggedMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    1,
                    None,
                    &mut ctx,
                ))
            };

            ofi.trecvmsg(&mut msg);
            assert_eq!(mem1[..1024], expected);

            // Receive a single iov message into two buffers
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let iov = IoVecMut::from_slice(&mut mem0[..256]);
            let iov1 = IoVecMut::from_slice(&mut mem1[..256]);
            let mut iovs = [iov, iov1];
            let mut msg = if connected {
                Either::Right(MsgTaggedConnectedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    None,
                    2,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTaggedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    2,
                    None,
                    &mut ctx,
                ))
            };

            ofi.trecvmsg(&mut msg);
            assert_eq!(mem0[..256], expected[..256]);
            assert_eq!(mem1[..256], expected[256..512]);

            // Receive a two iov message into two buffers
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let iov = IoVecMut::from_slice(&mut mem0[..512]);
            let iov1 = IoVecMut::from_slice(&mut mem1[..512]);
            let mut iovs = [iov, iov1];
            let mut msg = if connected {
                Either::Right(MsgTaggedConnectedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    None,
                    3,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgTaggedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    3,
                    None,
                    &mut ctx,
                ))
            };

            ofi.trecvmsg(&mut msg);
            assert_eq!(mem0[..512], expected[..512]);
            assert_eq!(mem1[..512], expected[512..1024]);
        }
    }

    #[test]
    fn async_tsendrecvmsg0() {
        tsendrecvmsg(true, "tsendrecvmsg0", false);
    }

    #[test]
    fn async_tsendrecvmsg1() {
        tsendrecvmsg(false, "tsendrecvmsg0", false);
    }

    #[test]
    fn async_conn_tsendrecvmsg0() {
        tsendrecvmsg(true, "conn_tsendrecvmsg0", true);
    }

    #[test]
    fn async_conn_tsendrecvmsg1() {
        tsendrecvmsg(false, "conn_tsendrecvmsg0", true);
    }

}
