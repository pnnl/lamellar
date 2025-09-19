

pub mod sync_;
#[cfg(test)]
pub mod sync_compare_atomic {
    use libfabric::cq::WaitCq;
    use libfabric::iovec::IocMut;
    use libfabric::iovec::Ioc;
    use libfabric::enums::CompareAtomicOp;
    use libfabric::iovec::RemoteMemAddrAtomicVec;
    use libfabric::infocapsoptions::InfoCaps;
    use libfabric::msg::MsgCompareAtomic;
    use libfabric::msg::MsgCompareAtomicConnected;
    use libfabric::mr::MemoryRegionBuilder;


    use crate::sync_::enable_ep_mr;
    use crate::sync_::handshake;
    use crate::sync_::handshake_connectionless;
    use crate::sync_::Either;

    fn compare_atomic(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().atomic()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
        };

        let mut reg_mem: Vec<_> = if server {
            vec![2; 1024 * 2]
        } else {
            vec![1; 1024 * 2]
        };
        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
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
        let comp_desc = Some(mr.descriptor());
        let res_desc = Some(mr.descriptor());
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        if server {
            let mut expected: Vec<_> = vec![1; 256];
            let (op_mem, ack_mem) = reg_mem.split_at_mut(768);
            let (buf, mem1) = op_mem.split_at_mut(256);
            let (comp, res) = mem1.split_at_mut(256);
            comp.iter_mut().for_each(|v| *v = 1);

            ofi.compare_atomic(
                buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc,
                CompareAtomicOp::Cswap,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(res, &expected[..256]);

            expected = vec![2; 256];
            ofi.compare_atomic(
                buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc,
                CompareAtomicOp::CswapNe,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(res, &expected);

            buf.iter_mut().for_each(|v| *v = 3);
            expected = vec![2; 256];
            ofi.compare_atomic(
                buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc,
                CompareAtomicOp::CswapLe,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(res, &expected);

            buf.iter_mut().for_each(|v| *v = 2);
            expected = vec![3; 256];
            ofi.compare_atomic(
                buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc,
                CompareAtomicOp::CswapLt,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(res, &expected);

            buf.iter_mut().for_each(|v| *v = 3);
            expected = vec![2; 256];
            ofi.compare_atomic(
                buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc,
                CompareAtomicOp::CswapGe,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(res, &expected);

            expected = vec![2; 256];
            ofi.compare_atomic(
                buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc,
                CompareAtomicOp::CswapGt,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(res, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // Send a done ack

            ofi.recv(&mut ack_mem[..512], desc, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();

            // expected = vec![2; 256];
            let (buf0, buf1) = buf.split_at_mut(128);
            let (comp0, comp1) = comp.split_at_mut(128);
            let (res0, res1) = res.split_at_mut(128);

            let buf_iocs = [Ioc::from_slice(buf0), Ioc::from_slice(buf1)];
            let comp_iocs = [Ioc::from_slice(comp0), Ioc::from_slice(comp1)];
            let mut res_iocs = [IocMut::from_slice(res0), IocMut::from_slice(res1)];
            let buf_descs = [mr.descriptor(), mr.descriptor()];
            let comp_descs = [mr.descriptor(), mr.descriptor()];
            let res_descs = [mr.descriptor(), mr.descriptor()];

            ofi.compare_atomicv(
                &buf_iocs,
                &comp_iocs,
                &mut res_iocs,
                0,
                Some(&buf_descs),
                Some(&comp_descs),
                Some(&res_descs),
                CompareAtomicOp::CswapLe,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(res, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Recv a completion ack
            ofi.recv(&mut ack_mem[..512], desc, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        } else {
            let mut expected = vec![2u8; 256];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[..256], &expected);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            expected = vec![3; 256];
            // // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[..256], &expected);
            ofi.send(&reg_mem[512..1024], desc, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        }
    }

    #[test]
    fn compare_atomic0() {
        compare_atomic(true, "compare_atomic0", false);
    }

    #[test]
    fn compare_atomic1() {
        compare_atomic(false, "compare_atomic0", false);
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn conn_compare_atomic0() {
    //     compare_atomic(true, "conn_compare_atomic0", true);
    // }

    // #[test]
    // fn conn_compare_atomic1() {
    //     compare_atomic(false, "conn_compare_atomic0", true);
    // }



    fn compare_atomicmsg(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().atomic()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
        };

        let mut reg_mem: Vec<_> = if server {
            vec![2; 1024 * 2]
        } else {
            vec![1; 1024 * 2]
        };
        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
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
        let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
        let dst_slice = remote_mem_info.slice(..256);
        let (dst_slice0, dst_slice1) = dst_slice.split_at(128);

        let mut ctx = ofi.info_entry.allocate_context();
        if server {
            let expected = vec![1u8; 256];
            let (op_mem, ack_mem) = reg_mem.split_at_mut(768);
            let (buf, mem1) = op_mem.split_at_mut(256);
            let (comp, res) = mem1.split_at_mut(256);
            comp.iter_mut().for_each(|v| *v = 1);

            // expected = vec![2; 256];
            let (buf0, buf1) = buf.split_at_mut(128);
            let (comp0, comp1) = comp.split_at_mut(128);
            let (res0, res1) = res.split_at_mut(128);

            let buf_iocs = [Ioc::from_slice(buf0), Ioc::from_slice(buf1)];
            let comp_iocs = [Ioc::from_slice(comp0), Ioc::from_slice(comp1)];
            let mut res_iocs = [IocMut::from_slice(res0), IocMut::from_slice(res1)];
            let buf_descs = [mr.descriptor(), mr.descriptor()];
            let comp_descs = [mr.descriptor(), mr.descriptor()];
            let res_descs = [mr.descriptor(), mr.descriptor()];
            let mut rma_iocs = RemoteMemAddrAtomicVec::new();
            rma_iocs.push(dst_slice0);
            rma_iocs.push(dst_slice1);
            // let rma_ioc0 = RmaIoc::from_slice(&dst_slice0);
            // let rma_ioc1 = RmaIoc::from_slice(&dst_slice1);
            // let rma_iocs = [rma_ioc0, rma_ioc1];

            let msg = if connected {
                Either::Right(MsgCompareAtomicConnected::from_ioc_slice(
                    &buf_iocs,
                    Some(&buf_descs),
                    &rma_iocs,
                    CompareAtomicOp::CswapGe,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgCompareAtomic::from_ioc_slice(
                    &buf_iocs,
                    Some(&buf_descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iocs,
                    CompareAtomicOp::CswapGe,
                    None,
                    &mut ctx,
                ))
            };

            ofi.compare_atomicmsg(
                &msg,
                &comp_iocs,
                &mut res_iocs,
                Some(&comp_descs),
                Some(&res_descs),
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(res, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Recv a completion ack
            ofi.recv(&mut ack_mem[..512], desc, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        } else {
            let expected = vec![2u8; 256];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[..256], &expected);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        }
    }

    #[test]
    fn compare_atomicmsg0() {
        compare_atomicmsg(true, "compare_atomicmsg0", false);
    }

    #[test]
    fn compare_atomicmsg1() {
        compare_atomicmsg(false, "compare_atomicmsg0", false);
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn conn_compare_atomic0() {
    //     compare_atomic(true, "conn_compare_atomic0", true);
    // }

    // #[test]
    // fn conn_compare_atomic1() {
    //     compare_atomic(false, "conn_compare_atomic0", true);
    // }
}


pub mod async_;
pub mod async_compare_atomic {
    use libfabric::iovec::IocMut;
    use libfabric::iovec::Ioc;
    use libfabric::enums::CompareAtomicOp;
    use libfabric::iovec::RemoteMemAddrAtomicVec;
    use libfabric::infocapsoptions::InfoCaps;
    use libfabric::mr::MemoryRegionBuilder;
    use libfabric::msg::MsgCompareAtomic;
    use libfabric::msg::MsgCompareAtomicConnected;

    use crate::async_::enable_ep_mr;
    use crate::async_::handshake;
    use crate::async_::handshake_connectionless;
    use crate::async_::Either;


    fn compare_atomic(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().atomic()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
        };

        let mut reg_mem: Vec<_> = if server {
            vec![2; 1024 * 2]
        } else {
            vec![1; 1024 * 2]
        };
        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
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
        let comp_desc = Some(mr.descriptor());
        let res_desc = Some(mr.descriptor());
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let mut ctx = ofi.info_entry.allocate_context();

        if server {
            let mut expected: Vec<_> = vec![1; 256];
            let (op_mem, ack_mem) = reg_mem.split_at_mut(768);
            let (buf, mem1) = op_mem.split_at_mut(256);
            let (comp, res) = mem1.split_at_mut(256);
            comp.iter_mut().for_each(|v| *v = 1);

            ofi.compare_atomic(
                &buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc.clone(),
                CompareAtomicOp::Cswap,
                &mut ctx,
            );

            assert_eq!(res, &expected[..256]);

            expected = vec![2; 256];
            ofi.compare_atomic(
                &buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc.clone(),
                CompareAtomicOp::CswapNe,
                &mut ctx,
            );

            assert_eq!(res, &expected);

            buf.iter_mut().for_each(|v| *v = 3);
            expected = vec![2; 256];
            ofi.compare_atomic(
                &buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc.clone(),
                CompareAtomicOp::CswapLe,
                &mut ctx,
            );

            assert_eq!(res, &expected);

            buf.iter_mut().for_each(|v| *v = 2);
            expected = vec![3; 256];
            ofi.compare_atomic(
                &buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc.clone(),
                CompareAtomicOp::CswapLt,
                &mut ctx,
            );

            assert_eq!(res, &expected);

            buf.iter_mut().for_each(|v| *v = 3);
            expected = vec![2; 256];
            ofi.compare_atomic(
                &buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc.clone(),
                CompareAtomicOp::CswapGe,
                &mut ctx,
            );

            assert_eq!(res, &expected);

            expected = vec![2; 256];
            ofi.compare_atomic(
                &buf,
                comp,
                res,
                0,
                desc,
                comp_desc,
                res_desc.clone(),
                CompareAtomicOp::CswapGt,
                &mut ctx,
            );

            assert_eq!(res, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc, None, &mut ctx);

            // Send a done ack

            ofi.recv(&mut ack_mem[..512], desc.clone(), &mut ctx);

            // expected = vec![2; 256];
            let (buf0, buf1) = buf.split_at_mut(128);
            let (comp0, comp1) = comp.split_at_mut(128);
            let (res0, res1) = res.split_at_mut(128);

            let buf_iocs = [Ioc::from_slice(&buf0), Ioc::from_slice(&buf1)];
            let comp_iocs = [Ioc::from_slice(&comp0), Ioc::from_slice(&comp1)];
            let mut res_iocs = [IocMut::from_slice(res0), IocMut::from_slice(res1)];
            let buf_descs = [mr.descriptor(), mr.descriptor()];
            let comp_descs = [mr.descriptor(), mr.descriptor()];
            let res_descs = [mr.descriptor(), mr.descriptor()];

            ofi.compare_atomicv(
                &buf_iocs,
                &comp_iocs,
                &mut res_iocs,
                0,
                Some(&buf_descs),
                Some(&comp_descs),
                Some(&res_descs),
                CompareAtomicOp::CswapLe,
                &mut ctx,
            );

            assert_eq!(res, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut ack_mem[..512], desc.clone(), &mut ctx);
        } else {
            let mut expected = vec![2u8; 256];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            expected = vec![3; 256];
            // // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);
        }
    }

    #[test]
    fn async_compare_atomic0() {
        compare_atomic(true, "compare_atomic0", false);
    }

    #[test]
    fn async_compare_atomic1() {
        compare_atomic(false, "compare_atomic0", false);
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn async_conn_compare_atomic0() {
    //     compare_atomic(true, "conn_compare_atomic0", true);
    // }

    // #[test]
    // fn async_conn_compare_atomic1() {
    //     compare_atomic(false, "conn_compare_atomic0", true);
    // }


    fn compare_atomicmsg(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(server, name, Some(InfoCaps::new().msg().atomic()))
        } else {
            handshake_connectionless(server, name, Some(InfoCaps::new().msg().atomic()))
        };

        let mut reg_mem: Vec<_> = if server {
            vec![2; 1024 * 2]
        } else {
            vec![1; 1024 * 2]
        };
        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
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
        let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
        let (dst_slice0, dst_slice1) = remote_mem_info.slice::<u8>(..256).split_at(128);
        // let base_addr = remote_mem_info.mem_address();
        // let key = &remote_mem_info.key();
        let mut ctx = ofi.info_entry.allocate_context();

        if server {
            let expected = vec![1u8; 256];
            let (op_mem, ack_mem) = reg_mem.split_at_mut(768);
            let (buf, mem1) = op_mem.split_at_mut(256);
            let (comp, res) = mem1.split_at_mut(256);
            comp.iter_mut().for_each(|v| *v = 1);

            // expected = vec![2; 256];
            let (buf0, buf1) = buf.split_at_mut(128);
            let (comp0, comp1) = comp.split_at_mut(128);
            let (res0, res1) = res.split_at_mut(128);

            let buf_iocs = [Ioc::from_slice(&buf0), Ioc::from_slice(&buf1)];
            let comp_iocs = [Ioc::from_slice(&comp0), Ioc::from_slice(&comp1)];
            let mut res_iocs = [IocMut::from_slice(res0), IocMut::from_slice(res1)];
            let buf_descs = [mr.descriptor(), mr.descriptor()];
            let comp_descs = [mr.descriptor(), mr.descriptor()];
            let res_descs = [mr.descriptor(), mr.descriptor()];
            let mut rma_iocs = RemoteMemAddrAtomicVec::new();
            rma_iocs.push(dst_slice0);
            rma_iocs.push(dst_slice1);

            let mut msg = if connected {
                Either::Right(MsgCompareAtomicConnected::from_ioc_slice(
                    &buf_iocs,
                    Some(&buf_descs),
                    &rma_iocs,
                    CompareAtomicOp::CswapGe,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgCompareAtomic::from_ioc_slice(
                    &buf_iocs,
                    Some(&buf_descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iocs,
                    CompareAtomicOp::CswapGe,
                    None,
                    &mut ctx,
                ))
            };

            ofi.compare_atomicmsg(
                &mut msg,
                &comp_iocs,
                &mut res_iocs,
                Some(&comp_descs),
                Some(&res_descs),
            );

            assert_eq!(res, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut ack_mem[..512], desc.clone(), &mut ctx);
        } else {
            let expected = vec![2u8; 256];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);
        }
    }

    #[test]
    fn async_compare_atomicmsg0() {
        compare_atomicmsg(true, "compare_atomicmsg0", false);
    }

    #[test]
    fn async_compare_atomicmsg1() {
        compare_atomicmsg(false, "compare_atomicmsg0", false);
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn async_conn_compare_atomic0() {
    //     compare_atomic(true, "conn_compare_atomic0", true);
    // }

    // #[test]
    // fn async_conn_compare_atomic1() {
    //     compare_atomic(false, "conn_compare_atomic0", true);
    // }

}