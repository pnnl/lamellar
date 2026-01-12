

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
    use crate::sync_::tests::handshake;
    use crate::sync_::tests::handshake_connectionless;
    use crate::sync_::tests::Either;

    fn compare_atomic(server: bool, name: &str, connected: bool) {
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


        if server {
            let mut reg_mem = ofi.reg_mem.borrow_mut();

            let borrow = ofi.mr.borrow();
            let mr = borrow.as_ref().unwrap();

            let desc = Some(mr.descriptor());
            let comp_desc = Some(mr.descriptor());
            let res_desc = Some(mr.descriptor());
            let mut expected: Vec<_> = vec![1; 256];
            let ack_range = 768_usize..768+512;
            // let op_start = 0_usize;
            // let buf_range = op_start..op_start+256;
            // let comp_range = op_start+256..op_start+512;
            // let res_range = op_start+512..op_start+768;

            {
                let (op_mem, _) = reg_mem.split_at_mut(768);
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
            }
            drop(reg_mem);
            
            // Send a done ack
            ofi.send(ack_range.clone(), None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // Send a done ack

            ofi.recv(ack_range.clone(), false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            {
                let mut reg_mem = ofi.reg_mem.borrow_mut();
                let (op_mem, _) = reg_mem.split_at_mut(768);
                let (buf, mem1) = op_mem.split_at_mut(256);
                let (comp, res) = mem1.split_at_mut(256);
                
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
            }
            // Send a done ack
            ofi.send(ack_range.clone(), None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Recv a completion ack
            ofi.recv(ack_range, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        } else {
            let mut expected = vec![2u8; 256];

            // Recv a completion ack
            ofi.recv(512..1024, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[..256], &expected);

            // Send completion ack
            ofi.send(512..1024, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            expected = vec![3; 256];
            // // Recv a completion ack
            ofi.recv(512..1024, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[..256], &expected);
            ofi.send(512..1024, None, false);
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
            }
        }

        let mapped_addr = ofi.mapped_addr.clone();

        let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
        let dst_slice = remote_mem_info.slice(..256);
        let (dst_slice0, dst_slice1) = dst_slice.split_at(128);

        let mut ctx = ofi.info_entry.allocate_context();
        if server {
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            let expected = vec![1u8; 256];
            let (op_mem, _) = reg_mem.split_at_mut(768);
            let ack_start = 768_usize;
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
            let borrow = ofi.mr.borrow();
            let mr = borrow.as_ref().unwrap();
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

            drop(reg_mem);
            // Send a done ack
            ofi.send(ack_start..ack_start+512, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Recv a completion ack
            ofi.recv(ack_start..ack_start+512, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        } else {
            let expected = vec![2u8; 256];

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

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
pub mod async_;

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
pub mod async_compare_atomic {
    use libfabric::iovec::IocMut;
    use libfabric::iovec::Ioc;
    use libfabric::enums::CompareAtomicOp;
    use libfabric::iovec::RemoteMemAddrAtomicVec;
    use libfabric::infocapsoptions::InfoCaps;
    use libfabric::msg::MsgCompareAtomic;
    use libfabric::msg::MsgCompareAtomicConnected;

    use crate::async_::handshake;
    use crate::async_::handshake_connectionless;
    use crate::async_::Either;


    fn compare_atomic(server: bool, name: &str, connected: bool) {
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
            }
        }

        if server {
            let mut reg_mem = ofi.reg_mem.borrow_mut();

            let borrow = ofi.mr.borrow();
            let mr = borrow.as_ref().unwrap();

            let desc = Some(mr.descriptor());
            let comp_desc = Some(mr.descriptor());
            let res_desc = Some(mr.descriptor());
            let mut expected: Vec<_> = vec![1; 256];
            let ack_range = 768_usize..768+512;
            // let op_start = 0_usize;
            // let buf_range = op_start..op_start+256;
            // let comp_range = op_start+256..op_start+512;
            // let res_range = op_start+512..op_start+768;

            {
                let (op_mem, _) = reg_mem.split_at_mut(768);
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
                );

                assert_eq!(res, &expected);
            }
            drop(reg_mem);

            // Send a done ack
            ofi.send(ack_range.clone(), None);

            // Send a done ack

            ofi.recv(ack_range.clone());

            // expected = vec![2; 256];
            {
                let mut reg_mem = ofi.reg_mem.borrow_mut();
                let (op_mem, _) = reg_mem.split_at_mut(768);
                let (buf, mem1) = op_mem.split_at_mut(256);
                let (comp, res) = mem1.split_at_mut(256);
                
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
                );

                assert_eq!(res, &expected);
            }

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

            expected = vec![3; 256];
            // // Recv a completion ack
            ofi.recv(512..1024);

            assert_eq!(&ofi.reg_mem.borrow()[..256], &expected);
            ofi.send(512..1024, None);
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
            }
        }

        let mapped_addr = ofi.mapped_addr.clone();

        let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
        let dst_slice = remote_mem_info.slice(..256);
        let (dst_slice0, dst_slice1) = dst_slice.split_at(128);

        if server {
            let expected = vec![1u8; 256];
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            let (op_mem, _) = reg_mem.split_at_mut(768);
            let ack_start = 768_usize;
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
            let borrow = ofi.mr.borrow();
            let mr = borrow.as_ref().unwrap();
            let buf_descs = [mr.descriptor(), mr.descriptor()];
            let comp_descs = [mr.descriptor(), mr.descriptor()];
            let res_descs = [mr.descriptor(), mr.descriptor()];
            let mut rma_iocs = RemoteMemAddrAtomicVec::new();
            rma_iocs.push(dst_slice0);
            rma_iocs.push(dst_slice1);
            let mut ctx = ofi.ctx.borrow_mut();
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
            drop(reg_mem);
            drop(ctx);

            // Send a done ack
            ofi.send(ack_start..ack_start+512, None);

            // Recv a completion ack
            ofi.recv(ack_start..ack_start+512);
        } else {
            let expected = vec![2u8; 256];

            // Recv a completion ack
            ofi.recv(512..1024);

            assert_eq!(&ofi.reg_mem.borrow()[..256], &expected);

            // Send completion ack
            ofi.send(512..1024, None);
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