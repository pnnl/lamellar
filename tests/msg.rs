pub mod sync_;
#[cfg(test)]
pub mod sync_msg {
    use libfabric::cq::WaitCq;
    use libfabric::msg::{Msg, MsgConnected, MsgConnectedMut, MsgMut};
    use libfabric::{cq::Completion, infocapsoptions::InfoCaps, iovec::{IoVec, IoVecMut}, mr::MemoryRegionBuilder};
    
    use crate::sync_::tests::{enable_ep_mr, handshake, handshake_connectionless, Either};
    fn sendrecv(server: bool, name: &str, connected: bool, use_context: bool) {
        let ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg()))
        };

        // ofi.reg_mem.borrow_mut().iter_mut().map
        {
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            for i in 0..1024 * 2 {
                let v = (i % 256) as u8;
                reg_mem[i] = v;
            }
        }
       

        if server {
            // Send a single buffer
            ofi.send_with_context(0..512, None);
            let ctx = ofi.ctx.borrow();
            let completion: Completion = ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            match completion {
                Completion::Unspec(entry) => {
                    assert!(entry[0].is_op_context_equal(&ctx));
                }
                Completion::Msg(entry) => {
                    assert!(entry[0].is_op_context_equal(&ctx));
                }
                Completion::Tagged(entry) => {
                    assert!(entry[0].is_op_context_equal(&ctx));
                }
                Completion::Ctx(entry) => {
                    assert!(entry[0].is_op_context_equal(&ctx));
                }
                Completion::Data(entry) => {
                    assert!(entry[0].is_op_context_equal(&ctx));
                }
            }

            // Inject a buffer
            ofi.send(0..128, None, use_context);
            // No cq.sread since inject does not generate completions

            let reg_mem = ofi.reg_mem.borrow();

            // // Send single Iov
            let iov = [IoVec::from_slice(&reg_mem[..512])];
            let mut borrow = ofi.mr.borrow_mut();
            let mr = borrow.as_mut().unwrap();


            let desc = [mr.descriptor(), mr.descriptor()];
            ofi.sendv(&iov, Some(&desc[..1]), use_context);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Send multi Iov
            let iov = [
                IoVec::from_slice(&reg_mem[..512]),
                IoVec::from_slice(&reg_mem[512..1024]),
            ];
            ofi.sendv(&iov, Some(&desc), use_context);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Send a single buffer
            // ofi.send_mr(&unsafe{mr.slice(0, ..512)}, None, false);
            let m1 = unsafe{ mr.slice(..512)};
            ofi.send_mr(&m1, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        } else {
            
            let expected: Vec<_> = (0..1024 * 2)
                .map(|v: usize| (v % 256) as u8)
                .collect();
            // let mut reg_mem = ofi.reg_mem.borrow_mut();

            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);

            // Receive a single buffer
            ofi.recv(0..512, use_context);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(ofi.reg_mem.borrow()[..512], expected[..512]);

            // Receive inject

            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);
            ofi.recv(0..128, use_context);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(ofi.reg_mem.borrow()[..128], expected[..128]);

            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);
            // // Receive into a single Iov
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            let mut borrow = ofi.mr.borrow_mut();
            let mr = borrow.as_mut().unwrap();


            let desc = [mr.descriptor(), mr.descriptor()];
            let iov = [IoVecMut::from_slice(&mut reg_mem[..512])];
            ofi.recvv(&iov, Some(&desc[..1]));
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(reg_mem[..512], expected[..512]);

            reg_mem.iter_mut().for_each(|v| *v = 0);

            // // Receive into multiple Iovs
            let (mem0, mem1) = reg_mem[..1024].split_at_mut(512);
            let iov = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
            ofi.recvv(&iov, Some(&desc));
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();

            assert_eq!(mem0, &expected[..512]);
            assert_eq!(mem1, &expected[512..1024]);

            reg_mem.iter_mut().for_each(|v| *v = 0);
            let mut mr0  = unsafe{mr.slice_mut( ..512)};
            // let (mr00, mr01) = mr0.split_at_mut(256);
            let mmr0 = mr0.as_mut_slice();
            mmr0[0] = 0;
            ofi.recv_mr(&mut mr0, false);

            // Send a single buffer
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(unsafe{mr.slice(..512)}.as_slice(), &expected[..512]);

        }
    }

    #[test]
    fn sendrecv0() {
        sendrecv(true, "sendrecv0", false, false);
    }

    #[test]
    fn sendrecv1() {
        sendrecv(false, "sendrecv0", false, false);
    }

    #[test]
    fn conn_sendrecv0() {
        sendrecv(true, "conn_sendrecv0", true, false);
    }

    #[test]
    fn conn_sendrecv1() {
        sendrecv(false, "conn_sendrecv0", true, false);
    }


    fn sendrecvmsg(server: bool, name: &str, connected: bool, use_context: bool) {
        let ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg()))
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
            let msg = if connected {
                Either::Right(MsgConnected::from_iov(&iov0, desc.as_ref(), data, &mut ctx))
            } else {
                Either::Left(Msg::from_iov(
                    &iov0,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    data,
                    &mut ctx,
                ))
            };
            ofi.sendmsg(&msg, use_context);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // let entry =
            // match entry {
            //     Completion::Data(entry) => assert_eq!(entry[0].data(), data),
            //     _ => panic!("Unexpected CQ entry format"),
            // }

            // Multi iov message with stride
            let iovs = [iov0, iov1];
            let msg = if connected {
                Either::Right(MsgConnected::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    data,
                    &mut ctx,
                ))
            } else {
                Either::Left(Msg::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    data,
                    &mut ctx,
                ))
            };

            ofi.sendmsg(&msg, use_context);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // let entry =
            // match entry {
            //     Completion::Data(entry) => assert_eq!(entry[0].data(), 128),
            //     _ => panic!("Unexpected CQ entry format"),
            // }

            // Single iov message
            let msg = if connected {
                Either::Right(MsgConnected::from_iov(
                    &iovs[0],
                    desc.as_ref(),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(Msg::from_iov(
                    &iovs[0],
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.sendmsg(&msg, use_context);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            let msg = if connected {
                Either::Right(MsgConnected::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(Msg::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };
            ofi.sendmsg(&msg, use_context);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        } else {
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

            // Receive a single message in a single buffer
            let mut iov = IoVecMut::from_slice(mem0);
            let msg = if connected {
                Either::Right(MsgConnectedMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.recvmsg(&msg, use_context);
            // ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            let entry = ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            match entry {
                Completion::Data(entry) => assert_eq!(entry[0].data(), 128),
                Completion::Tagged(entry) => assert_eq!(entry[0].data(), 128),
                _ => {},
            }
            assert_eq!(mem0.len(), expected[..512].len());
            assert_eq!(mem0, &expected[..512]);

            // Receive a multi iov message in a single buffer
            let mut iov = IoVecMut::from_slice(&mut mem1[..1024]);
            let msg = if connected {
                Either::Right(MsgConnectedMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.recvmsg(&msg, use_context);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            // let entry =
            // match entry {
            //     Completion::Data(entry) => assert_eq!(entry[0].data(), 128),
            //     _ => panic!("Unexpected CQ entry format"),
            // }
            assert_eq!(mem1[..1024], expected);

            // Receive a single iov message into two buffers
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let iov = IoVecMut::from_slice(&mut mem0[..256]);
            let iov1 = IoVecMut::from_slice(&mut mem1[..256]);
            let mut iovs = [iov, iov1];
            let msg = if connected {
                Either::Right(MsgConnectedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.recvmsg(&msg, use_context);
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
                Either::Right(MsgConnectedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.recvmsg(&msg, use_context);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(mem0[..512], expected[..512]);
            assert_eq!(mem1[..512], expected[512..1024]);
        }
    }

    #[test]
    fn sendrecvmsg0() {
        sendrecvmsg(true, "sendrecvmsg0", false, false);
    }

    #[test]
    fn sendrecvmsg1() {
        sendrecvmsg(false, "sendrecvmsg0", false, false);
    }

    #[test]
    fn conn_sendrecvmsg0() {
        sendrecvmsg(true, "conn_sendrecvmsg0", true, false);
    }

    #[test]
    fn conn_sendrecvmsg1() {
        sendrecvmsg(false, "conn_sendrecvmsg0", true, false);
    }


    fn sendrecvdata(server: bool, name: &str, connected: bool, use_context: bool) {
        let ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg()))
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
            ofi.send(0..512, data, use_context);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        } else {
            let expected: Vec<_> = (0..1024 * 2)
                .map(|v: usize| (v % 256) as u8)
                .collect();
            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);

            // Receive a single buffer
            ofi.recv(0..512, use_context);

            let completion = ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            match completion {
                Completion::Tagged(entry) => {
                    assert_eq!(entry[0].data(), data.unwrap() );
                }
                
                Completion::Data(entry) => {
                    assert_eq!(entry[0].data(), data.unwrap() );
                }
                _ => {}
            }
            assert_eq!(ofi.reg_mem.borrow()[..512], expected[..512]);
        }
    }

    #[test]
    fn sendrecvdata0() {
        sendrecvdata(true, "sendrecvdata0", false, false);
    }

    #[test]
    fn sendrecvdata1() {
        sendrecvdata(false, "sendrecvdata0", false, false);
    }

    #[test]
    fn conn_sendrecvdata0() {
        sendrecvdata(true, "conn_sendrecvdata0", true, false);
    }

    #[test]
    fn conn_sendrecvdata1() {
        sendrecvdata(false, "conn_sendrecvdata0", true, false);
    }


    // #[test]
    // fn context_sendrecv0() {
    //     sendrecv(true, "sendrecv0", false, true);
    // }

    // #[test]
    // fn context_sendrecv1() {
    //     sendrecv(false, "sendrecv0", false, true);
    // }

    // #[test]
    // fn context_conn_sendrecv0() {
    //     sendrecv(true, "conn_sendrecv0", true, true);
    // }

    // #[test]
    // fn context_conn_sendrecv1() {
    //     sendrecv(false, "conn_sendrecv0", true, true);
    // }

    // #[test]
    // fn context_sendrecvdata0() {
    //     sendrecvdata(true, "sendrecvdata0", false, true);
    // }

    // #[test]
    // fn context_sendrecvdata1() {
    //     sendrecvdata(false, "sendrecvdata0", false, true);
    // }

    // #[test]
    // fn context_conn_sendrecvdata0() {
    //     sendrecvdata(true, "conn_sendrecvdata0", true, true);
    // }

    // #[test]
    // fn context_conn_sendrecvdata1() {
    //     sendrecvdata(false, "conn_sendrecvdata0", true, true);
    // }
}

pub mod async_;

#[cfg(test)]
pub mod async_msg {
    use libfabric::{infocapsoptions::InfoCaps, iovec::{IoVec, IoVecMut}, mr::MemoryRegionBuilder, msg::{Msg, MsgConnected, MsgConnectedMut, MsgMut}};

    use crate::async_::{enable_ep_mr, handshake, handshake_connectionless, Either};


    fn sendrecv(server: bool, name: &str, connected: bool) {
        let ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg()))
        };

        {
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            for i in 0..1024 * 2 {
                let v = (i % 256) as u8;
                reg_mem[i] = v;
            }
        }


        if server {
            // Send a single buffer
            ofi.send(0..512, None);
            assert!(
                std::mem::size_of_val(&ofi.reg_mem.borrow()[..128]) <= ofi.info_entry.tx_attr().inject_size()
            );

            // Inject a buffer
            ofi.send(0..128, None);
            // No cq.sread since inject does not generate completions

            // // Send single Iov
            let reg_mem = ofi.reg_mem.borrow();

            let mut borrow = ofi.mr.borrow_mut();
            let mr = borrow.as_mut().unwrap();


            let desc = [mr.descriptor(), mr.descriptor()];
            let iov = [IoVec::from_slice(&reg_mem[..512])];
            ofi.sendv(&iov, Some(&desc[..1]));

            // Send multi Iov
            let iov = [
                IoVec::from_slice(&reg_mem[..512]),
                IoVec::from_slice(&reg_mem[512..1024]),
            ];
            ofi.sendv(&iov, Some(&desc));
        } else {

            let expected: Vec<_> = (0..1024 * 2)
                .into_iter()
                .map(|v: usize| (v % 256) as u8)
                .collect();
            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);

            // Receive a single buffer
            ofi.recv(0..512);
            assert_eq!(ofi.reg_mem.borrow()[..512], expected[..512]);

            // Receive inject
            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);
            ofi.recv(0..128);
            assert_eq!(ofi.reg_mem.borrow()[..128], expected[..128]);

            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);
            // // Receive into a single Iov
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            let mut borrow = ofi.mr.borrow_mut();
            let mr = borrow.as_mut().unwrap();


            let desc = [mr.descriptor(), mr.descriptor()];
            let iov = [IoVecMut::from_slice(&mut reg_mem[..512])];
            ofi.recvv(&iov, Some(&desc[..1]));
            assert_eq!(reg_mem[..512], expected[..512]);

            reg_mem.iter_mut().for_each(|v| *v = 0);

            // // Receive into multiple Iovs
            let (mem0, mem1) = reg_mem[..1024].split_at_mut(512);
            let iov = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
            ofi.recvv(&iov, Some(&desc));

            assert_eq!(mem0, &expected[..512]);
            assert_eq!(mem1, &expected[512..1024]);
        }
    }

    // fn sendrecv_deadlock(server: bool, name: &str, connected: bool) {
    //     let ofi = if connected {
    //         handshake(server, name, Some(InfoCaps::new().msg()))
    //     } else {
    //         handshake_connectionless(server, name, Some(InfoCaps::new().msg()))
    //     };

    //     let mut reg_mem: Vec<_> = (0..1024 * 2)
    //         .into_iter()
    //         .map(|v: usize| (v % 256) as u8)
    //         .collect();
    //     let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
    //         .access_recv()
    //         .access_send()
    //         .build(&ofi.domain)
    //         .unwrap();

    //     let mr = match mr {
    //         libfabric::mr::MaybeDisabledMemoryRegion::Enabled(mr) => mr,
    //         libfabric::mr::MaybeDisabledMemoryRegion::Disabled(disabled_mr) => {
    //             match disabled_mr {
    //                 libfabric::mr::DisabledMemoryRegion::EpBind(ep_binding_memory_region) => enable_ep_mr(&ofi.ep, ep_binding_memory_region),
    //                 libfabric::mr::DisabledMemoryRegion::RmaEvent(rma_event_memory_region) => rma_event_memory_region.enable().unwrap(),
    //             }
    //         }
    //     };

    //     let desc0 = Some(mr.descriptor());
    //     let mut send_ctx = ofi.info_entry.allocate_context();
    //     let mut recv_ctx = ofi.info_entry.allocate_context();

    //     if server {
    //         // Send a single buffer
    //         // ofi.send(&reg_mem[..512], desc0.as_ref(), None, &mut ctx);
    //         // assert!(
    //         //     std::mem::size_of_val(&reg_mem[..128]) <= ofi.info_entry.tx_attr().inject_size()
    //         // );

    //         ofi.sendrecv_deadlock(&mut reg_mem[..512], desc0.as_ref(), None, &mut send_ctx, &mut recv_ctx);
    //         // Inject a buffer
    //         // ofi.send(&reg_mem[..128], desc0.as_ref(), None, &mut ctx);
    //         // // No cq.sread since inject does not generate completions

    //         // // // Send single Iov
    //         // let iov = [IoVec::from_slice(&reg_mem[..512])];
    //         // ofi.sendv(&iov, Some(&desc[..1]), &mut ctx);

    //         // // Send multi Iov
    //         // let iov = [
    //         //     IoVec::from_slice(&reg_mem[..512]),
    //         //     IoVec::from_slice(&reg_mem[512..1024]),
    //         // ];
    //         // ofi.sendv(&iov, Some(&desc), &mut ctx);
    //     } else {
    //         ofi.sendrecv_deadlock(&mut reg_mem[..512], desc0.as_ref(), None, &mut send_ctx, &mut recv_ctx);
    //     }
    // }

    #[test]
    fn async_sendrecv0() {
        sendrecv(true, "sendrecv0", false);
    }

    #[test]
    fn async_sendrecv1() {
        sendrecv(false, "sendrecv0", false);
    }

    // #[test]
    // fn async_sendrecv_deadlock0() {
    //     sendrecv_deadlock(true, "sendrecv0", false);
    // }

    // #[test]
    // fn async_sendrecv_deadlock1() {
    //     sendrecv_deadlock(false, "sendrecv0", false);
    // }

    #[test]
    fn async_conn_sendrecv0() {
        sendrecv(true, "conn_sendrecv0", true);
    }

    #[test]
    fn async_conn_sendrecv1() {
        sendrecv(false, "conn_sendrecv0", true);
    }


    fn sendrecvdata(server: bool, name: &str, connected: bool) {
        let ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg()))
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
            ofi.send(0..512, data);
        } else {
            let expected: Vec<_> = (0..1024 * 2)
                .into_iter()
                .map(|v: usize| (v % 256) as u8)
                .collect();
            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);

            // Receive a single buffer
            ofi.recv(0..512);
            assert_eq!(ofi.reg_mem.borrow()[..512], expected[..512]);
        }
    }

    #[test]
    fn async_sendrecvdata0() {
        sendrecvdata(true, "sendrecvdata0", false);
    }

    #[test]
    fn async_sendrecvdata1() {
        sendrecvdata(false, "sendrecvdata0", false);
    }

    #[test]
    fn async_conn_sendrecvdata0() {
        sendrecvdata(true, "conn_sendrecvdata0", true);
    }

    #[test]
    fn async_conn_sendrecvdata1() {
        sendrecvdata(false, "conn_sendrecvdata0", true);
    }


    fn sendrecvmsg(server: bool, name: &str, connected: bool) {
        let ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg()))
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

        if server {
            // Single iov message
            let (mem0, mem1) = (&reg_mem[..512], &reg_mem[1024..1536]);
            let iov0 = IoVec::from_slice(mem0);
            let iov1 = IoVec::from_slice(mem1);
            let data = Some(128);
            let mut msg = if connected {
                Either::Right(MsgConnected::from_iov(&iov0, desc.as_ref(), data, &mut ctx))
            } else {
                Either::Left(Msg::from_iov(
                    &iov0,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    data,
                    &mut ctx,
                ))
            };
            ofi.sendmsg(&mut msg);

            // let entry =
            // match entry {
            //     Completion::Data(entry) => assert_eq!(entry[0].data(), 128),
            //     _ => panic!("Unexpected CQ entry format"),
            // }

            // Multi iov message with stride
            let iovs = [iov0, iov1];
            let mut msg = if connected {
                Either::Right(MsgConnected::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    data,
                    &mut ctx,
                ))
            } else {
                Either::Left(Msg::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    data,
                    &mut ctx,
                ))
            };

            ofi.sendmsg(&mut msg);

            // Single iov message
            let mut msg = if connected {
                Either::Right(MsgConnected::from_iov(
                    &iovs[0],
                    desc.as_ref(),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(Msg::from_iov(
                    &iovs[0],
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.sendmsg(&mut msg);

            let mut msg = if connected {
                Either::Right(MsgConnected::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(Msg::from_iov_slice(
                    &iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };
            ofi.sendmsg(&mut msg);
        } else {
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

            // Receive a single message in a single buffer
            let mut iov = IoVecMut::from_slice(mem0);
            let mut msg = if connected {
                Either::Right(MsgConnectedMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.recvmsg(&mut msg);

            assert_eq!(mem0.len(), expected[..512].len());
            assert_eq!(mem0, &expected[..512]);

            // Receive a multi iov message in a single buffer
            let mut iov = IoVecMut::from_slice(&mut mem1[..1024]);
            let mut msg = if connected {
                Either::Right(MsgConnectedMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgMut::from_iov(
                    &mut iov,
                    desc.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.recvmsg(&mut msg);
            assert_eq!(mem1[..1024], expected);

            // Receive a single iov message into two buffers
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let iov = IoVecMut::from_slice(&mut mem0[..256]);
            let iov1 = IoVecMut::from_slice(&mut mem1[..256]);
            let mut iovs = [iov, iov1];
            let mut msg = if connected {
                Either::Right(MsgConnectedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.recvmsg(&mut msg);
            assert_eq!(mem0[..256], expected[..256]);
            assert_eq!(mem1[..256], expected[256..512]);

            // Receive a two iov message into two buffers
            reg_mem.iter_mut().for_each(|v| *v = 0);
            let (mem0, mem1) = reg_mem.split_at_mut(512);
            let iov = IoVecMut::from_slice(&mut mem0[..512]);
            let iov1 = IoVecMut::from_slice(&mut mem1[..512]);
            let mut iovs = [iov, iov1];
            let mut msg = if connected {
                Either::Right(MsgConnectedMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgMut::from_iov_slice(
                    &mut iovs,
                    Some(&descs),
                    &mapped_addr.as_ref().unwrap()[1],
                    None,
                    &mut ctx,
                ))
            };

            ofi.recvmsg(&mut msg);
            assert_eq!(mem0[..512], expected[..512]);
            assert_eq!(mem1[..512], expected[512..1024]);
        }
    }

    #[test]
    fn async_sendrecvmsg0() {
        sendrecvmsg(true, "sendrecvmsg0", false);
    }

    #[test]
    fn async_sendrecvmsg1() {
        sendrecvmsg(false, "sendrecvmsg0", false);
    }

    #[test]
    fn async_conn_sendrecvmsg0() {
        sendrecvmsg(true, "conn_sendrecvmsg0", true);
    }

    #[test]
    fn async_conn_sendrecvmsg1() {
        sendrecvmsg(false, "conn_sendrecvmsg0", true);
    }
}