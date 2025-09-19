pub mod sync_;
#[cfg(test)]
pub mod sync_atomic {
    use libfabric::{cq::{ReadCq, WaitCq}, enums::AtomicOp, infocapsoptions::InfoCaps, iovec::{Ioc, RemoteMemAddrAtomicVec}, mr::MemoryRegionBuilder, msg::{MsgAtomic, MsgAtomicConnected}};

    use crate::sync_::{enable_ep_mr, handshake, handshake_connectionless, Either};


    fn atomic(server: bool, name: &str, connected: bool) {
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

        let descs = [mr.descriptor(), mr.descriptor()];
        let desc0 = Some(mr.descriptor());
        // let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        if server {
            ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Min);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Max);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Sum);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Prod);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Bor);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Band);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            ofi.send(&reg_mem[512..1024], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();

            ofi.atomic_bool(&bool_reg_mem[..512], 0, desc0, AtomicOp::Lor);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Bxor);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            ofi.atomic_bool(&bool_reg_mem[..512], 0, desc0, AtomicOp::Land);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            ofi.send(&reg_mem[512..1024], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();

            // ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::Lxor);
            // ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            ofi.atomic(&reg_mem[..512], 0, desc0, AtomicOp::AtomicWrite);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            ofi.send(&reg_mem[512..1024], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            let iocs = [
                Ioc::from_slice(&reg_mem[..256]),
                Ioc::from_slice(&reg_mem[256..512]),
            ];

            ofi.atomicv(&iocs, 0, Some(&descs), AtomicOp::Prod);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            ofi.send(&reg_mem[512..1024], desc0, None, false);
            let err = ofi.cq_type.tx_cq().sread(1, -1);
            if let Err(e) = err {
                if matches!(e.kind, libfabric::error::ErrorKind::ErrorAvailable) {
                    let realerr = ofi.cq_type.tx_cq().readerr(0).unwrap();
                    panic!("{:?}", realerr.error());
                }
            }

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        } else {
            let mut expected = vec![2u8; 1024 * 2];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[..512], &expected[..512]);
            // Send completion ack
            reg_mem.iter_mut().for_each(|v| *v= 1);
            // reg_mem = vec![1; 1024 * 2];

            ofi.send(&reg_mem[512..1024], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            expected = vec![1; 1024 * 2];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[..512], &expected[..512]);
            ofi.send(&reg_mem[512..1024], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            expected = vec![2;1024*2];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[..512], &expected[..512]);

            expected = vec![4; 1024 * 2];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc0, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[..512], &expected[..512]);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc0, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        }
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn conn_atomic0() {
    //     atomic(true, "conn_atomic0", true);
    // }

    // #[test]
    // fn conn_atomic1() {
    //     atomic(false, "conn_atomic0", true);
    // }


    #[test]
    fn atomic0() {
        atomic(true, "atomic0", false);
    }

    #[test]
    fn atomic1() {
        atomic(false, "atomic0", false);
    }



    fn atomicmsg(server: bool, name: &str, connected: bool) {
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
        let descs = [mr.descriptor(), mr.descriptor()];
        let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
        let dst_slice = remote_mem_info.slice(..512);
        let (dst_slice0, dst_slice1) = dst_slice.split_at(256 * std::mem::size_of::<u8>());

        let mut ctx = ofi.info_entry.allocate_context();
        if server {
            let iocs = [
                Ioc::from_slice(&reg_mem[..256]),
                Ioc::from_slice(&reg_mem[256..512]),
            ];
            let mut rma_iocs = RemoteMemAddrAtomicVec::new();
            rma_iocs.push(dst_slice0);
            rma_iocs.push(dst_slice1);

            let msg = if connected {
                Either::Right(MsgAtomicConnected::from_ioc_slice(
                    &iocs,
                    Some(&descs),
                    &rma_iocs,
                    AtomicOp::Bor,
                    Some(128),
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgAtomic::from_ioc_slice(
                    &iocs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iocs,
                    AtomicOp::Bor,
                    Some(128),
                    &mut ctx,
                ))
            };

            ofi.atomicmsg(&msg);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            ofi.send(&reg_mem[512..1024], desc, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        } else {
            let expected = vec![3u8; 1024 * 2];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&reg_mem[..512], &expected[..512]);
            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        }
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn conn_atomic0() {
    //     atomic(true, "conn_atomic0", true);
    // }

    // #[test]
    // fn conn_atomic1() {
    //     atomic(false, "conn_atomic0", true);
    // }

    #[test]
    fn atomicmsg0() {
        atomicmsg(true, "atomicmsg0", false);
    }

    #[test]
    fn atomicmsg1() {
        atomicmsg(false, "atomicmsg0", false);
    }
}

pub mod async_;

pub mod async_atomic {
    use libfabric::{enums::AtomicOp, infocapsoptions::InfoCaps, iovec::{Ioc, RemoteMemAddrAtomicVec}, mr::MemoryRegionBuilder, msg::{MsgAtomic, MsgAtomicConnected}};

    use crate::async_::{enable_ep_mr, handshake, handshake_connectionless, Either};


    fn atomic(server: bool, name: &str, connected: bool) {
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
        let descs = [mr.descriptor(), mr.descriptor()];
        // let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let mut ctx = ofi.info_entry.allocate_context();

        if server {
            ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Min, &mut ctx);

            ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Max, &mut ctx);

            ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Sum, &mut ctx);

            ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Prod, &mut ctx);

            ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Bor, &mut ctx);

            ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Band, &mut ctx);

            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            // ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Lor, &mut ctx);

            ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Bxor, &mut ctx);

            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            // ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Land, &mut ctx);

            // ofi.atomic(&reg_mem[..512], 0, desc, AtomicOp::Lxor, &mut ctx);

            ofi.atomic(
                &reg_mem[..512],
                0,
                desc,
                AtomicOp::AtomicWrite,
                &mut ctx,
            );
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            let iocs = [
                Ioc::from_slice(&reg_mem[..256]),
                Ioc::from_slice(&reg_mem[256..512]),
            ];

            ofi.atomicv(&iocs, 0, Some(&descs), AtomicOp::Prod, &mut ctx);
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);
            // match err {
            //     Err(e) => {
            //         if matches!(e.kind, libfabric::error::ErrorKind::ErrorAvailable) {
            //             let realerr = ofi.cq_type.tx_cq().readerr(0).unwrap();
            //             panic!("{:?}", realerr.error());
            //         }
            //     }
            //     Ok(_) => {}
            // }

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);
        } else {
            let mut expected = vec![2u8; 1024 * 2];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            assert_eq!(&reg_mem[..512], &expected[..512]);
            // Send completion ack
            reg_mem.iter_mut().for_each(|v| *v= 1);

            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            expected = vec![3; 1024 * 2];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            assert_eq!(&reg_mem[..512], &expected[..512]);
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            expected = vec![2;1024*2];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);
            assert_eq!(&reg_mem[..512], &expected[..512]);

            expected = vec![4; 1024 * 2];
            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);
            assert_eq!(&reg_mem[..512], &expected[..512]);

            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);
        }
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn async_conn_atomic0() {
    //     atomic(true, "conn_atomic0", true);
    // }

    // #[test]
    // fn async_conn_atomic1() {
    //     atomic(false, "conn_atomic0", true);
    // }

    #[test]
    fn async_atomic0() {
        atomic(true, "atomic0", false);
    }

    #[test]
    fn async_atomic1() {
        atomic(false, "atomic0", false);
    }


    fn atomicmsg(server: bool, name: &str, connected: bool) {
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
        let descs = [mr.descriptor(), mr.descriptor()];
        let mapped_addr = ofi.mapped_addr.clone();
        let key = mr.key().unwrap();
        ofi.exchange_keys(&key, &reg_mem[..]);
        let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
        let (dst_slice0, dst_slice1) = remote_mem_info.slice::<u8>(..512).split_at(256);

        let mut ctx = ofi.info_entry.allocate_context();

        if server {
            let iocs = [
                Ioc::from_slice(&reg_mem[..256]),
                Ioc::from_slice(&reg_mem[256..512]),
            ];
            let mut rma_iocs = RemoteMemAddrAtomicVec::new();
            rma_iocs.push(dst_slice0);
            rma_iocs.push(dst_slice1);

            let mut msg = if connected {
                Either::Right(MsgAtomicConnected::from_ioc_slice(
                    &iocs,
                    Some(&descs),
                    &rma_iocs,
                    AtomicOp::Bor,
                    Some(128),
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgAtomic::from_ioc_slice(
                    &iocs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iocs,
                    AtomicOp::Bor,
                    Some(128),
                    &mut ctx,
                ))
            };

            ofi.atomicmsg(&mut msg);

            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);
        } else {
            let expected = vec![3u8; 1024 * 2];

            // Recv a completion ack
            ofi.recv(&mut reg_mem[512..1024], desc.clone(), &mut ctx);

            assert_eq!(&reg_mem[..512], &expected[..512]);
            // Send completion ack
            ofi.send(&reg_mem[512..1024], desc, None, &mut ctx);
        }
    }

    // [TODO Not sure why, but connected endpoints fail with atomic ops
    // #[test]
    // fn async_conn_atomic0() {
    //     atomic(true, "conn_atomic0", true);
    // }

    // #[test]
    // fn async_conn_atomic1() {
    //     atomic(false, "conn_atomic0", true);
    // }

    #[test]
    fn async_atomicmsg0() {
        atomicmsg(true, "atomicmsg0", false);
    }

    #[test]
    fn async_atomicmsg1() {
        atomicmsg(false, "atomicmsg0", false);
    }
}
