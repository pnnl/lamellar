

pub mod sync_;

pub mod sync_fetch_atomic {
    use libfabric::iovec::RemoteMemAddrAtomicVec;
    use libfabric::msg::{MsgFetchAtomic, MsgFetchAtomicConnected};
    use libfabric::{cq::WaitCq, enums::FetchAtomicOp, infocapsoptions::InfoCaps, iovec::{Ioc, IocMut}, mr::MemoryRegionBuilder};

    use crate::sync_::{enable_ep_mr, handshake, handshake_connectionless, Either};

    fn fetch_atomic(server: bool, name: &str, connected: bool) {
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
        
        let bool_reg_mem: Vec<_> = if server {
            vec![true; 1024 * 2]
        } else {
            vec![false; 1024 * 2]
        };

        let bool_mr = MemoryRegionBuilder::new(&bool_reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
            .build(&ofi.domain)
            .unwrap();

        let _bool_mr = match bool_mr {
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


        let desc0 = Some(mr.descriptor());
        let desc1 = Some(mr.descriptor());
        // let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        if server {
            let mut expected: Vec<u64> = vec![1; 256];
            let (op_mem, ack_mem) = reg_mem.split_at_mut(512);
            let (mem0, mem1) = op_mem.split_at_mut(256);
            // let (bool_mem0, bool_mem1) = bool_reg_mem.split_at_mut(256);
            ofi.fetch_atomic(
                mem0,
                mem1,
                0,
                desc0,
                desc1,
                FetchAtomicOp::Min,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(mem1, &expected[..256]);

            expected = vec![1; 256];
            ofi.fetch_atomic(
                mem0,
                mem1,
                0,
                desc0,
                desc1,
                FetchAtomicOp::Max,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(mem1, &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                mem0,
                mem1,
                0,
                desc0,
                desc1,
                FetchAtomicOp::Sum,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(mem1, &expected);

            expected = vec![4; 256];
            ofi.fetch_atomic(
                mem0,
                mem1,
                0,
                desc0,
                desc1,
                FetchAtomicOp::Prod,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(mem1, &expected);

            expected = vec![8; 256];
            ofi.fetch_atomic(
                mem0,
                mem1,
                0,
                desc0,
                desc1,
                FetchAtomicOp::Bor,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(mem1, &expected);

            expected = vec![10; 256];
            ofi.fetch_atomic(
                mem0,
                mem1,
                0,
                desc0,
                desc1,
                FetchAtomicOp::Band,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(mem1, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // Send a done ack

            ofi.recv(&mut ack_mem[..512], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();

            // let bool_expected = vec![true; 256];
            // ofi.fetch_atomic_bool(
            //     &bool_mem0[..256],
            //     &mut bool_mem1[..256],
            //     0,
            //     desc0,
            //     desc1,
            //     FetchAtomicOp::Lor,
            // );
            // ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // assert_eq!(bool_mem1, &bool_expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                mem0,
                mem1,
                0,
                desc0,
                desc1,
                FetchAtomicOp::Bxor,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(mem1, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // Send a done ack

            ofi.recv(&mut ack_mem[..512], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();

            expected = vec![0; 256];
            ofi.fetch_atomic(
                mem0,
                mem1,
                0,
                desc0,
                desc1,
                FetchAtomicOp::Bor,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(mem1, &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                mem0,
                mem1,
                0,
                desc0,
                desc1,
                FetchAtomicOp::Band,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(mem1, &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                mem0,
                mem1,
                0,
                desc0,
                desc1,
                FetchAtomicOp::AtomicWrite,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(mem1, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // Send a done ack

            ofi.recv(&mut ack_mem[..512], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();

            expected = vec![2; 256];
            ofi.fetch_atomic(
                mem0,
                mem1,
                0,
                desc0,
                desc1,
                FetchAtomicOp::AtomicRead,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(mem1, &expected);

            expected = vec![2; 256];
            let (read_mem, write_mem) = op_mem.split_at_mut(256);
            let iocs = [
                Ioc::from_slice(&read_mem[..128]),
                Ioc::from_slice(&read_mem[128..256]),
            ];
            let write_mems = write_mem.split_at_mut(128);
            let mut res_iocs = [
                IocMut::from_slice(write_mems.0),
                IocMut::from_slice(write_mems.1),
            ];

            let desc0 = Some(mr.descriptor());
            let descs = [mr.descriptor(), mr.descriptor()];
            let res_descs = [mr.descriptor(), mr.descriptor()];
            ofi.fetch_atomicv(
                &iocs,
                &mut res_iocs,
                0,
                Some(&descs),
                Some(&res_descs),
                FetchAtomicOp::Prod,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(write_mem, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Recv a completion ack
            ofi.recv(&mut ack_mem[..512], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        } else {
            let mut expected = vec![2u64; 256];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[..256], &expected);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            expected = vec![0; 256];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[..256], &expected);
            ofi.send(&reg_mem[512..1024], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            expected = vec![2; 256];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[..256], &expected);
            ofi.send(&reg_mem[512..1024], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            expected = vec![4; 256];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[..256], &expected);
            ofi.send(&reg_mem[512..1024], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        }
    }

    #[test]
    fn fetch_atomic0() {
        fetch_atomic(true, "fetch_atomic0", false);
    }

    #[test]
    fn fetch_atomic1() {
        fetch_atomic(false, "fetch_atomic0", false);
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn conn_fetch_atomic0() {
    //     fetch_atomic(true, "conn_fetch_atomic0", true);
    // }

    // #[test]
    // fn conn_fetch_atomic1() {
    //     fetch_atomic(false, "conn_fetch_atomic0", true);
    // }



    fn fetch_atomicmsg(server: bool, name: &str, connected: bool) {
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
        let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
        let dst_slice = remote_mem_info.slice(..256);
        let (dst_slice0, dst_slice1) = dst_slice.split_at(128);

        let mut ctx = ofi.info_entry.allocate_context();
        if server {
            let expected = vec![1u8; 256];
            let (op_mem, ack_mem) = reg_mem.split_at_mut(512);

            let (read_mem, write_mem) = op_mem.split_at_mut(256);
            let iocs = [
                Ioc::from_slice(&read_mem[..128]),
                Ioc::from_slice(&read_mem[128..256]),
            ];
            let write_mems = write_mem.split_at_mut(128);
            let mut res_iocs = [
                IocMut::from_slice(write_mems.0),
                IocMut::from_slice(write_mems.1),
            ];

            let desc0 = Some(mr.descriptor());
            let descs = [mr.descriptor(), mr.descriptor()];
            let res_descs = [mr.descriptor(), mr.descriptor()];
            let mut rma_iocs = RemoteMemAddrAtomicVec::new();
            rma_iocs.push(dst_slice0);
            rma_iocs.push(dst_slice1);

            let msg = if connected {
                Either::Right(MsgFetchAtomicConnected::from_ioc_slice(
                    &iocs,
                    Some(&descs),
                    &rma_iocs,
                    FetchAtomicOp::Prod,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgFetchAtomic::from_ioc_slice(
                    &iocs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iocs,
                    FetchAtomicOp::Prod,
                    None,
                    &mut ctx,
                ))
            };

            ofi.fetch_atomicmsg(&msg, &mut res_iocs, Some(&res_descs));
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(write_mem, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Recv a completion ack
            ofi.recv(&mut ack_mem[..512], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        } else {
            let desc0 = Some(mr.descriptor());
            let expected = vec![2u8; 256];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[..256], &expected);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        }
    }

    #[test]
    fn fetch_atomicmsg0() {
        fetch_atomicmsg(true, "fetch_atomicmsg0", false);
    }

    #[test]
    fn fetch_atomicmsg1() {
        fetch_atomicmsg(false, "fetch_atomicmsg0", false);
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn conn_fetch_atomic0() {
    //     fetch_atomic(true, "conn_fetch_atomic0", true);
    // }

    // #[test]
    // fn conn_fetch_atomic1() {
    //     fetch_atomic(false, "conn_fetch_atomic0", true);
    // }
}

pub mod async_;

pub mod async_fetch_atomic {
    use libfabric::enums::FetchAtomicOp;
    use libfabric::infocapsoptions::InfoCaps;
    use libfabric::iovec::{Ioc, IocMut, RemoteMemAddrAtomicVec};
    use libfabric::mr::MemoryRegionBuilder;
    use libfabric::msg::{MsgFetchAtomic, MsgFetchAtomicConnected};

    use crate::async_::{enable_ep_mr, handshake, handshake_connectionless, Either};


    fn fetch_atomic(server: bool, name: &str, connected: bool) {
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

        let desc0 = Some(mr.descriptor());
        let desc1 = Some(mr.descriptor());
        // let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let mut ctx = ofi.info_entry.allocate_context();
        if server {
            let mut expected: Vec<_> = vec![1; 256];
            let (op_mem, ack_mem) = reg_mem.split_at_mut(512);
            let (mem0, mem1) = op_mem.split_at_mut(256);
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Min,
                &mut ctx,
            );

            assert_eq!(mem1, &expected[..256]);

            expected = vec![1; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Max,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Sum,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            expected = vec![4; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Prod,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            expected = vec![8; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Bor,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            expected = vec![10; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Band,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, &mut ctx);

            // Send a done ack

            ofi.recv(&mut ack_mem[..512], desc0.clone(), &mut ctx);

            expected = vec![2; 256];
            // ofi.fetch_atomic(
            //     &mem0,
            //     mem1,
            //     0,
            //     desc0,
            //     desc1.clone(),
            //     FetchAtomicOp::Lor,
            //     &mut ctx,
            // );

            // assert_eq!(mem1, &expected);

            // expected = vec![1; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Bxor,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, &mut ctx);

            // Send a done ack

            ofi.recv(&mut ack_mem[..512], desc0.clone(), &mut ctx);

            expected = vec![0; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Bor,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Band,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::AtomicWrite,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, &mut ctx);

            ofi.recv(&mut ack_mem[..512], desc0.clone(), &mut ctx);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                &mem0,
                mem1,
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::AtomicRead,
                &mut ctx,
            );

            assert_eq!(mem1, &expected);

            expected = vec![2; 256];
            let (read_mem, write_mem) = op_mem.split_at_mut(256);
            let iocs = [
                Ioc::from_slice(&read_mem[..128]),
                Ioc::from_slice(&read_mem[128..256]),
            ];
            let write_mems = write_mem.split_at_mut(128);
            let mut res_iocs = [
                IocMut::from_slice(write_mems.0),
                IocMut::from_slice(write_mems.1),
            ];

            let desc0 = Some(mr.descriptor());
            let descs = [mr.descriptor(), mr.descriptor()];
            let res_descs = [mr.descriptor(), mr.descriptor()];
            ofi.fetch_atomicv(
                &iocs,
                &mut res_iocs,
                0,
                Some(&descs),
                Some(&res_descs),
                FetchAtomicOp::Prod,
                &mut ctx,
            );

            assert_eq!(write_mem, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut ack_mem[..512], desc0.clone(), &mut ctx);
        } else {
            let mut expected = vec![2u8; 256];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc0, None, &mut ctx);

            expected = vec![0; 256];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);
            ofi.send(&reg_mem[512..1024], desc0, None, &mut ctx);

            expected = vec![2; 256];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);
            ofi.send(&reg_mem[512..1024], desc0, None, &mut ctx);

            expected = vec![4; 256];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);
            ofi.send(&reg_mem[512..1024], desc0, None, &mut ctx);
        }
    }

    #[test]
    fn async_fetch_atomic0() {
        fetch_atomic(true, "fetch_atomic0", false);
    }

    #[test]
    fn async_fetch_atomic1() {
        fetch_atomic(false, "fetch_atomic0", false);
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn async_conn_fetch_atomic0() {
    //     fetch_atomic(true, "conn_fetch_atomic0", true);
    // }

    // #[test]
    // fn async_conn_fetch_atomic1() {
    //     fetch_atomic(false, "conn_fetch_atomic0", true);
    // }


    fn fetch_atomicmsg(server: bool, name: &str, connected: bool) {
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
            let (op_mem, ack_mem) = reg_mem.split_at_mut(512);

            let (read_mem, write_mem) = op_mem.split_at_mut(256);
            let iocs = [
                Ioc::from_slice(&read_mem[..128]),
                Ioc::from_slice(&read_mem[128..256]),
            ];
            let write_mems = write_mem.split_at_mut(128);
            let mut res_iocs = [
                IocMut::from_slice(write_mems.0),
                IocMut::from_slice(write_mems.1),
            ];

            let desc0 = Some(mr.descriptor());
            let descs = [mr.descriptor(), mr.descriptor()];
            let res_descs = [mr.descriptor(), mr.descriptor()];
            let mut rma_iocs = RemoteMemAddrAtomicVec::new();
            rma_iocs.push(dst_slice0);
            rma_iocs.push(dst_slice1);

            let mut msg = if connected {
                Either::Right(MsgFetchAtomicConnected::from_ioc_slice(
                    &iocs,
                    Some(&descs),
                    &rma_iocs,
                    FetchAtomicOp::Prod,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgFetchAtomic::from_ioc_slice(
                    &iocs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iocs,
                    FetchAtomicOp::Prod,
                    None,
                    &mut ctx,
                ))
            };

            ofi.fetch_atomicmsg(&mut msg, &mut res_iocs, Some(&res_descs));

            assert_eq!(write_mem, &expected);

            // Send a done ack
            ofi.send(&ack_mem[..512], desc0, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut ack_mem[..512], desc0.clone(), &mut ctx);
        } else {
            let desc0 = Some(mr.descriptor());
            let expected = vec![2u8; 256];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0.clone(), &mut ctx);

            assert_eq!(&reg_mem[..256], &expected);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc0, None, &mut ctx);
        }
    }

    #[test]
    fn async_fetch_atomicmsg0() {
        fetch_atomicmsg(true, "fetch_atomicmsg0", false);
    }

    #[test]
    fn async_fetch_atomicmsg1() {
        fetch_atomicmsg(false, "fetch_atomicmsg0", false);
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn async_conn_fetch_atomic0() {
    //     fetch_atomic(true, "conn_fetch_atomic0", true);
    // }

    // #[test]
    // fn async_conn_fetch_atomic1() {
    //     fetch_atomic(false, "conn_fetch_atomic0", true);
    // }
}