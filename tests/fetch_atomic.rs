

pub mod sync_;

pub mod sync_fetch_atomic {
    use libfabric::iovec::RemoteMemAddrAtomicVec;
    use libfabric::msg::{MsgFetchAtomic, MsgFetchAtomicConnected};
    use libfabric::{cq::WaitCq, enums::FetchAtomicOp, infocapsoptions::InfoCaps, iovec::{Ioc, IocMut}};

    use crate::sync_::tests::{handshake, handshake_connectionless, Either};

    fn fetch_atomic(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg().atomic()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg().atomic()))
        };

        ofi.exchange_keys();

        {
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            if server {
                reg_mem.fill(2);
            } else {
                reg_mem.fill(1);
            };
        }
        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();


        let desc0 = Some(mr.descriptor());
        let desc1 = Some(mr.descriptor());
        // let mapped_addr = ofi.mapped_addr.clone();
        if server {
            let mut expected = vec![1; 256];
            // let (op_mem, _) = reg_mem.split_at_mut(512);
            // let (mem0, mem1) = op_mem.split_at_mut(256);
            let mem0_range = 0_usize..256;
            let mem1_range = 256_usize..512;
            let ack_range = 512_usize..512+512;

            // let (bool_mem0, bool_mem1) = bool_reg_mem.split_at_mut(256);
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1,
                FetchAtomicOp::Min,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected[..256]);

            expected = vec![1; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1,
                FetchAtomicOp::Max,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1,
                FetchAtomicOp::Sum,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            expected = vec![4; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1,
                FetchAtomicOp::Prod,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            expected = vec![8; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1,
                FetchAtomicOp::Bor,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            expected = vec![10; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1,
                FetchAtomicOp::Band,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            // Send a done ack
            ofi.send(ack_range.clone(), None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // Send a done ack

            ofi.recv(ack_range.clone(), false);
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
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1,
                FetchAtomicOp::Bxor,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            // Send a done ack
            ofi.send(ack_range.clone(), None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // Send a done ack

            ofi.recv(ack_range.clone(), false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();

            expected = vec![0; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1,
                FetchAtomicOp::Bor,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1,
                FetchAtomicOp::Band,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1,
                FetchAtomicOp::AtomicWrite,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            // Send a done ack
            ofi.send(ack_range.clone(), None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // Send a done ack

            ofi.recv(ack_range.clone(), false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();

            expected = vec![2; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1,
                FetchAtomicOp::AtomicRead,
            );
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            expected = vec![2; 256];
            let mut borrow = ofi.reg_mem.borrow_mut();
            let (read_mem, write_mem) = borrow.split_at_mut(256);
            let iocs = [
                Ioc::from_slice(&read_mem[..128]),
                Ioc::from_slice(&read_mem[128..256]),
            ];
            let write_mems = write_mem.split_at_mut(128);
            let mut res_iocs = [
                IocMut::from_slice(write_mems.0),
                IocMut::from_slice(&mut write_mems.1[..128]),
            ];

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
            // let cerr = ofi.cq_type.tx_cq().readerr(0).unwrap();
            // panic!("CERR: {:?}", cerr.error());

            assert_eq!(&write_mem[..256], &expected);
            drop(borrow);

            // Send a done ack
            ofi.send(ack_range.clone(), None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Recv a completion ack
            ofi.recv(ack_range, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        } else {
            let mut expected = vec![2; 256];
            // Recv a completion ack
            ofi.recv(512..1024,  false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[..256], &expected);
            // Send completion ack
            ofi.send(512..1024, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            expected = vec![0; 256];
            // Recv a completion ack
            ofi.recv(512..1024, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[..256], &expected);
            
            ofi.send(512..1024, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            expected = vec![2; 256];
            // Recv a completion ack
            ofi.recv(512..1024, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            
            assert_eq!(&ofi.reg_mem.borrow()[..256], &expected);
            ofi.send(512..1024, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            expected = vec![4; 256];
            // Recv a completion ack
            ofi.recv(512..1024, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[..256], &expected);
            ofi.send(512..1024, None, false);
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
            handshake(None, server, name, Some(InfoCaps::new().msg().atomic()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg().atomic()))
        };

        ofi.exchange_keys();

        if server {
            ofi.reg_mem.borrow_mut().fill(2);
        } else {
            ofi.reg_mem.borrow_mut().fill(1);
        };

        let mut reg_mem = ofi.reg_mem.borrow_mut();

        let mapped_addr = ofi.mapped_addr.clone();

        let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
        let dst_slice = remote_mem_info.slice(..256);
        let (dst_slice0, dst_slice1) = dst_slice.split_at(128);

        let mut ctx = ofi.info_entry.allocate_context();
        if server {
            let expected = vec![1u8; 256];
            let (op_mem, _) = reg_mem.split_at_mut(512);

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
            let ack_range = 512_usize..512+512;
            // let read_range = 0_usize..256;
            // let write_range = 256_usize..512;

            let borrow = ofi.mr.borrow();
            let mr = borrow.as_ref().unwrap();
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
            drop(reg_mem);
            // Send a done ack
            ofi.send(ack_range.clone(), None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Recv a completion ack
            ofi.recv(ack_range.clone(), false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        } else {
            let expected = vec![2u8; 256];
            drop(reg_mem);

            // Recv a completion ack
            ofi.recv(512..1024, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[..256], &expected);

            // Send completion ack
            ofi.send(512..1024, None, false);
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
    use libfabric::msg::{MsgFetchAtomic, MsgFetchAtomicConnected};

    use crate::async_::{handshake, handshake_connectionless, Either};


    fn fetch_atomic(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg().atomic()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg().atomic()))
        };

        ofi.exchange_keys();

        {
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            if server {
                reg_mem.fill(2);
            } else {
                reg_mem.fill(1);
            };
        }

        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();


        let desc0 = Some(mr.descriptor());
        let desc1 = Some(mr.descriptor());

        if server {
            let mut expected: Vec<_> = vec![1; 256];
            let mem0_range = 0_usize..256;
            let mem1_range = 256_usize..512;
            let ack_range = 512_usize..512+512;

            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Min,
            );

            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected[..256]);

            expected = vec![1; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Max,
            );

            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Sum,
            );

            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            expected = vec![4; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Prod,
            );

            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            expected = vec![8; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Bor,
            );

            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            expected = vec![10; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Band,
            );

            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            // Send a done ack
            ofi.send(ack_range.clone(), None);

            // Send a done ack

            ofi.recv(ack_range.clone());

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
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Bxor,
            );

            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            // Send a done ack
            ofi.send(ack_range.clone(), None);

            // Send a done ack

            ofi.recv(ack_range.clone());

            expected = vec![0; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Bor,
            );

            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::Band,
            );

            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            expected = vec![2; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1.clone(),
                FetchAtomicOp::AtomicWrite,
            );

            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);

            // Send a done ack
            ofi.send(ack_range.clone(), None);

            ofi.recv(ack_range.clone());

            expected = vec![2; 256];
            ofi.fetch_atomic(
                mem0_range.clone(),
                mem1_range.clone(),
                0,
                desc0,
                desc1,
                FetchAtomicOp::AtomicRead,
            );

            assert_eq!(&ofi.reg_mem.borrow()[mem1_range.clone()], &expected);


            expected = vec![2; 256];
            let mut borrow = ofi.reg_mem.borrow_mut();
            let (read_mem, write_mem) = borrow.split_at_mut(256);

            let iocs = [
                Ioc::from_slice(&read_mem[..128]),
                Ioc::from_slice(&read_mem[128..256]),
            ];
            let write_mems = write_mem.split_at_mut(128);
            let mut res_iocs = [
                IocMut::from_slice(write_mems.0),
                IocMut::from_slice(&mut write_mems.1[..128]),
            ];


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

            assert_eq!(&write_mem[..256], &expected);
            drop(borrow);

            // Send a done ack
            ofi.send(ack_range.clone(), None);

            // Recv a completion ack
            ofi.recv(ack_range);
        } else {
            let mut expected = vec![2u8; 256];

            // Recv a completion ack
            ofi.recv(512..1024);

            assert_eq!(&ofi.reg_mem.borrow()[..256], &expected);

            // Send completion ack
            ofi.send(512..1024, None);

            expected = vec![0; 256];
            // Recv a completion ack
            ofi.recv(512..1024);

            assert_eq!(&ofi.reg_mem.borrow()[..256], &expected);

            ofi.send(512..1024, None);

            expected = vec![2; 256];
            // Recv a completion ack
            ofi.recv(512..1024);

            assert_eq!(&ofi.reg_mem.borrow()[..256], &expected);
            ofi.send(512..1024, None);

            expected = vec![4; 256];
            // Recv a completion ack
            ofi.recv(512..1024);

            assert_eq!(&ofi.reg_mem.borrow()[..256], &expected);
            ofi.send(512..1024, None);
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
            handshake(None, server, name, Some(InfoCaps::new().msg().atomic()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg().atomic()))
        };

        ofi.exchange_keys();

        if server {
            ofi.reg_mem.borrow_mut().fill(2);
        } else {
            ofi.reg_mem.borrow_mut().fill(1);
        };

        let mut reg_mem = ofi.reg_mem.borrow_mut();

        let mapped_addr = ofi.mapped_addr.clone();

        let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
        let dst_slice = remote_mem_info.slice(..256);
        let (dst_slice0, dst_slice1) = dst_slice.split_at(128);

        if server {
            let expected = vec![1u8; 256];
            let (op_mem, _) = reg_mem.split_at_mut(512);

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
            let ack_range = 512_usize..512+512;
            // let read_range = 0_usize..256;
            // let write_range = 256_usize..512;

            let borrow = ofi.mr.borrow();
            let mr = borrow.as_ref().unwrap();
            let descs = [mr.descriptor(), mr.descriptor()];
            let res_descs = [mr.descriptor(), mr.descriptor()];
            let mut rma_iocs = RemoteMemAddrAtomicVec::new();
            rma_iocs.push(dst_slice0);
            rma_iocs.push(dst_slice1);
            let mut ctx = ofi.ctx.borrow_mut();

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
            drop(reg_mem);
            drop(ctx);

            // Send a done ack
            ofi.send(ack_range.clone(), None);

            // Recv a completion ack
            ofi.recv(ack_range);
        } else {
            let expected = vec![2u8; 256];
            drop(reg_mem);

            // Recv a completion ack
            ofi.recv(512..1024);

            assert_eq!(&ofi.reg_mem.borrow()[..256], &expected);

            // Send completion ack
            ofi.send(512..1024, None);
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