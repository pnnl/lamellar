pub mod sync_;
#[cfg(test)]
pub mod sync_collective {
    use libfabric::{av_set::AddressVectorSetBuilder, comm::collective::CollectiveEp, cq::{ReadCq, WaitCq}, enums::CollectiveOptions, eq::{Event, ReadEq}, infocapsoptions::{CollCap, InfoCaps}, mcast::MultiCastGroup, mr::{MemoryRegion, MemoryRegionBuilder}};

    use crate::sync_::tests::{enable_ep_mr, handshake, handshake_connectionless, MyEndpoint, Ofi};


    fn collective(server: bool, name: &str, connected: bool) -> (Ofi<impl CollCap>, MultiCastGroup) {
        let mut ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg().collective()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg().collective()))
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

        let mut avset = if server {
            AddressVectorSetBuilder::new_from_range(
                ofi.av.as_ref().unwrap(),
                &ofi.mapped_addr.as_ref().unwrap()[0],
                &ofi.mapped_addr.as_ref().unwrap()[0],
                1,
            )
            .count(2)
            .build()
            .unwrap()
        } else {
            AddressVectorSetBuilder::new_from_range(
                ofi.av.as_ref().unwrap(),
                &ofi.mapped_addr.as_ref().unwrap()[1],
                &ofi.mapped_addr.as_ref().unwrap()[1],
                1,
            )
            .count(2)
            .build()
            .unwrap()
        };

        if server {
            for addr in ofi.mapped_addr.as_ref().unwrap().iter().skip(1) {
                avset.insert(addr).unwrap();
            }
        } else {
            avset.insert(&ofi.mapped_addr.as_ref().unwrap()[0]).unwrap();
        }

        let mut ctx = ofi.info_entry.allocate_context();
        let mc = libfabric::mcast::MulticastGroupBuilder::from_av_set(&avset).build();

        let mc = match &ofi.ep {
            MyEndpoint::Connected(ep) => mc
                .join_collective_with_context(ep, libfabric::enums::JoinOptions::new(), &mut ctx)
                .unwrap(),
            MyEndpoint::Connectionless(ep) => mc
                .join_collective_with_context(ep, libfabric::enums::JoinOptions::new(), &mut ctx)
                .unwrap(),
        };

        let join_event;
        loop {
            let event = ofi.eq.read();
            if let Ok(Event::JoinComplete(join)) = event {
                join_event = join;
                break;
            }
            let _ = ofi.cq_type.tx_cq().read(0);
            let _ = ofi.cq_type.rx_cq().read(0);
        }

        (ofi, mc.join_complete(join_event))
    }

    #[test]
    fn collective_0() {
        collective(true, "collective_0", false);
    }

    #[test]
    fn collective_1() {
        collective(false, "collective_0", false);
    }

    fn barrier(server: bool, name: &str, connected: bool) {
        println!("Start barrier Collective!");
        let (ofi, mc) = collective(server, name, connected);
        match &ofi.ep {
            MyEndpoint::Connected(_) => todo!(),
            MyEndpoint::Connectionless(ep) => ep.barrier(&mc).unwrap(),
        }
        println!("Done barrier Collective!");
    }

    #[test]
    fn barrier0() {
        barrier(true, "barrier0", false);
    }

    #[test]
    fn barrier1() {
        barrier(false, "barrier0", false);
    }

    fn broadcast(server: bool, name: &str, connected: bool) {
        println!("Start broadcast sync Collective!");
        let (ofi, mc) = collective(server, name, connected);
        
        {
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            if server {
                reg_mem.fill(2);
            } else {
                reg_mem.fill(1);
            };
        }


        let expected = if server {
            ofi.reg_mem.borrow()[..4096].to_vec()
        } else {
            ofi.reg_mem.borrow()[..4096].iter().map(|v| v + 1).collect()
        };

        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();

        match &ofi.ep {
            MyEndpoint::Connected(_) => todo!(),
            MyEndpoint::Connectionless(ep) => {
                ep.broadcast(
                    &mut ofi.reg_mem.borrow_mut()[..4096],
                    Some(&mr.descriptor()),
                    &mc,
                    &ofi.mapped_addr.as_ref().unwrap()[0],
                    CollectiveOptions::new(),
                )
                .unwrap();
                ofi.cq_type.tx_cq().sread(1, -1).unwrap();
                // ofi.cq_type.rx_cq().sread(1, -1).unwrap();

                assert_eq!(&ofi.reg_mem.borrow()[..4096], expected);
            }
        }
        println!("Done broadcast Collective!");
    }

    #[test]
    fn broadcast0() {
        broadcast(true, "broadcast0", false);
    }

    #[test]
    fn broadcast1() {
        broadcast(false, "broadcast0", false);
    }

    fn alltoall(server: bool, name: &str, connected: bool) {
        println!("Start alltoall Collective!");
        let (ofi, mc) = collective(server, name, connected);
        let expected = if server {
            vec![1; 1024 * 2]
        } else {
            vec![2; 1024 * 2]
        };

        {
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            if server {
                reg_mem.fill(2);
            } else {
                reg_mem.fill(1);
            };
        }

        let half = ofi.reg_mem.borrow().len() / 2;
        let mut reg_mem = ofi.reg_mem.borrow_mut();
        let (send_buf, recv_buf) = reg_mem.split_at_mut(half);
        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();

        match &ofi.ep {
            MyEndpoint::Connected(_) => todo!(),
            MyEndpoint::Connectionless(ep) => {
                ep.alltoall(
                    send_buf,
                    Some(&mr.descriptor()),
                    recv_buf,
                    Some(&mr.descriptor()),
                    &mc,
                    CollectiveOptions::new(),
                )
                .unwrap();
                ofi.cq_type.tx_cq().sread(1, -1).unwrap();
                // ofi.cq_type.rx_cq().sread(1, -1).unwrap();

                assert_eq!(&reg_mem[..], expected);
            }
        }
        println!("Done alltoall Collective!");
    }

    // #[test]
    // fn alltoall0() {
    //     alltoall(true, "alltoall0", false);
    // }

    // #[test]
    // fn alltoall1() {
    //     alltoall(false, "alltoall0", false);
    // }

    fn allreduce(server: bool, name: &str, connected: bool) {
        println!("Start allreduce Sync Collective!");
        let (ofi, mc) = collective(server, name, connected);
        {
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            if server {
                reg_mem.fill(2);
            } else {
                reg_mem.fill(1);
            };
        }
        
        let expected = if server {
            vec![3; 1024]
        } else {
            vec![3; 1024]
        };


        let mut reg_mem = ofi.reg_mem.borrow_mut();
        let (send_buf, recv_buf) = reg_mem.split_at_mut(1024);
        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();

        match &ofi.ep {
            MyEndpoint::Connected(_) => todo!(),
            MyEndpoint::Connectionless(ep) => {
                ep.allreduce(
                    send_buf,
                    Some(&mr.descriptor()),
                    &mut recv_buf[..1024],
                    Some(&mr.descriptor()),
                    &mc,
                    libfabric::enums::CollAtomicOp::Sum,
                    CollectiveOptions::new(),
                )
                .unwrap();
                ofi.cq_type.tx_cq().sread(1, -1).unwrap();
                // ofi.cq_type.rx_cq().sread(1, -1).unwrap();

                assert_eq!(&recv_buf[..1024], expected);
            }
        }
        println!("Done allreduce Collective!");
    }

    #[test]
    fn allreduce0() {
        allreduce(true, "allreduce0", false);
    }

    #[test]
    fn allreduce1() {
        allreduce(false, "allreduce0", false);
    }

    fn allgather(server: bool, name: &str, connected: bool) {
        println!("Start Allgather Collective!");
        let (ofi, mc) = collective(server, name, connected);
        let mut reg_mem = ofi.reg_mem.borrow_mut();
        if server {
            reg_mem.fill(2);
        } else {
            reg_mem.fill(1);
        };

        let expected = if server {
            [vec![2; 512], vec![1; 512]].concat()
        } else {
            [vec![2; 512], vec![1; 512]].concat()
        };
        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();

        // let quart = reg_mem.len()/4;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(512);
        match &ofi.ep {
            MyEndpoint::Connected(_) => todo!(),
            MyEndpoint::Connectionless(ep) => {
                ep.allgather(
                    send_buf,
                    Some(&mr.descriptor()),
                    &mut recv_buf[..1024],
                    Some(&mr.descriptor()),
                    &mc,
                    CollectiveOptions::new(),
                )
                .unwrap();
                ofi.cq_type.tx_cq().sread(1, -1).unwrap();
                // ofi.cq_type.rx_cq().sread(1, -1).unwrap();
                assert_eq!(&recv_buf[..1024], expected);
            }
        }
        println!("Done Allgather Collective!");
    }

    #[test]
    fn allgather0() {
        allgather(true, "allgather0", false);
    }

    #[test]
    fn allgather1() {
        allgather(false, "allgather0", false);
    }

    fn reduce_scatter(server: bool, name: &str, connected: bool) {
        println!("Start reduce_scatter Collective!");
        let (ofi, mc) = collective(server, name, connected);
        let mut reg_mem = ofi.reg_mem.borrow_mut();
        if server {
            reg_mem.fill(2);
        } else {
            reg_mem.fill(1);
        };

        let expected = if server {
            vec![3; 1024]
        } else {
            vec![3; 1024]
        };
        
        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();

        let (send_buf, recv_buf) = reg_mem.split_at_mut(1024);
        match &ofi.ep {
            MyEndpoint::Connected(_) => todo!(),
            MyEndpoint::Connectionless(ep) => {
                ep.reduce_scatter(
                    send_buf,
                    Some(&mr.descriptor()),
                    &mut recv_buf[..1024],
                    Some(&mr.descriptor()),
                    &mc,
                    libfabric::enums::CollAtomicOp::Sum,
                    CollectiveOptions::new(),
                )
                .unwrap();
                ofi.cq_type.tx_cq().sread(1, -1).unwrap();
                // ofi.cq_type.rx_cq().sread(1, -1).unwrap();
                assert_eq!(recv_buf[..1024], expected);
            }
        }
        println!("Done reduce_scatter Collective!");
    }

    // #[test]
    // fn reduce_scatter0() {
    //     reduce_scatter(true, "reduce_scatter0", false);
    // }

    // #[test]
    // fn reduce_scatter1() {
    //     reduce_scatter(false, "reduce_scatter0", false);
    // }

    fn reduce(server: bool, name: &str, connected: bool) {
        println!("Start reduce Collective!");
        let (ofi, mc) = collective(server, name, connected);
        let mut reg_mem = ofi.reg_mem.borrow_mut();
        if server {
            reg_mem.fill(2);
        } else {
            reg_mem.fill(1);
        };
        let expected = if server {
            vec![3; 1024]
        } else {
            vec![1; 1024]
        };
        
        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();

        // let quart = reg_mem.len()/4;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(1024);
        match &ofi.ep {
            MyEndpoint::Connected(_) => todo!(),
            MyEndpoint::Connectionless(ep) => {
                ep.reduce(
                    send_buf,
                    Some(&mr.descriptor()),
                    &mut recv_buf[..1024],
                    Some(&mr.descriptor()),
                    &mc,
                    &ofi.mapped_addr.as_ref().unwrap()[0],
                    libfabric::enums::CollAtomicOp::Sum,
                    CollectiveOptions::new(),
                )
                .unwrap();
                ofi.cq_type.tx_cq().sread(1, -1).unwrap();
                // ofi.cq_type.rx_cq().sread(1, -1).unwrap();
                assert_eq!(recv_buf[..1024], expected);
            }
        }
        println!("Done reduce Collective!");
    }

    // #[test]
    // fn reduce0() {
    //     reduce(true, "reduce0", false);
    // }

    // #[test]
    // fn reduce1() {
    //     reduce(false, "reduce0", false);
    // }

    fn scatter(server: bool, name: &str, connected: bool) {
        println!("Start scatter Collective!");
        let (ofi, mc) = collective(server, name, connected);
        let mut reg_mem = ofi.reg_mem.borrow_mut();
        if server {
            reg_mem.fill(2);
        } else {
            reg_mem.fill(1);
        };

        let expected = if server {
            vec![2; 512]
        } else {
            vec![2; 512]
        };
        
        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();

        // let quart = reg_mem.len()/4;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(1024);
        match &ofi.ep {
            MyEndpoint::Connected(_) => todo!(),
            MyEndpoint::Connectionless(ep) => {
                ep.scatter(
                    send_buf,
                    Some(&mr.descriptor()),
                    &mut recv_buf[..512],
                    Some(&mr.descriptor()),
                    &mc,
                    &ofi.mapped_addr.as_ref().unwrap()[0],
                    CollectiveOptions::new(),
                )
                .unwrap();
                ofi.cq_type.tx_cq().sread(1, -1).unwrap();
                // ofi.cq_type.rx_cq().sread(1, -1).unwrap();
                assert_eq!(recv_buf[..512], expected);
            }
        }
        println!("Done scatter Collective!");
    }

    // #[test]
    // fn scatter0() {
    //     scatter(true, "scatter0", false);
    // }

    // #[test]
    // fn scatter1() {
    //     scatter(false, "scatter0", false);
    // }

    fn gather(server: bool, name: &str, connected: bool) {
        println!("Start gather Collective!");
        let (ofi, mc) = collective(server, name, connected);
        let mut reg_mem = ofi.reg_mem.borrow_mut();
        if server {
            reg_mem.fill(2);
        } else {
            reg_mem.fill(1);
        };

        let expected = if server {
            [vec![2; 512], vec![1; 512]].concat()
        } else {
            vec![1; 512]
        };
        
        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();

        // let quart = reg_mem.len()/4;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(512);
        match &ofi.ep {
            MyEndpoint::Connected(_) => todo!(),
            MyEndpoint::Connectionless(ep) => {
                ep.gather(
                    send_buf,
                    Some(&mr.descriptor()),
                    &mut recv_buf[..1024],
                    Some(&mr.descriptor()),
                    &mc,
                    &ofi.mapped_addr.as_ref().unwrap()[0],
                    CollectiveOptions::new(),
                )
                .unwrap();
                ofi.cq_type.tx_cq().sread(1, -1).unwrap();
                // ofi.cq_type.rx_cq().sread(1, -1).unwrap();
                assert_eq!(recv_buf[..1024], expected);
            }
        }
        println!("Done gather Collective!");
    }

    // #[test]
    // fn gather0() {
    //     gather(true, "gather0", false);
    // }

    // #[test]
    // fn gather1() {
    //     gather(false, "gather0", false);
    // }
}

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
pub mod async_;

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
pub mod async_collective {
    use libfabric::{async_::comm::collective::AsyncCollectiveEp, av_set::AddressVectorSetBuilder, enums::CollectiveOptions, infocapsoptions::{CollCap, InfoCaps}, mcast::MultiCastGroup, mr::{MemoryRegion, MemoryRegionBuilder}};

    use crate::async_::{enable_ep_mr, handshake, handshake_connectionless, MyEndpoint, Ofi};


    fn collective(
        server: bool,
        name: &str,
        connected: bool,
    ) -> (Ofi<impl CollCap>, MultiCastGroup) {
        let mut ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg().collective()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg().collective()))
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

        let mut avset = if server {
            AddressVectorSetBuilder::new_from_range(
                &ofi.av.as_ref().unwrap(),
                &ofi.mapped_addr.as_ref().unwrap()[0],
                &ofi.mapped_addr.as_ref().unwrap()[0],
                1,
            )
            .count(2)
            .build()
            .unwrap()
        } else {
            AddressVectorSetBuilder::new_from_range(
                &ofi.av.as_ref().unwrap(),
                &ofi.mapped_addr.as_ref().unwrap()[1],
                &ofi.mapped_addr.as_ref().unwrap()[1],
                1,
            )
            .count(2)
            .build()
            .unwrap()
        };

        if server {
            for addr in ofi.mapped_addr.as_ref().unwrap().iter().skip(1) {
                avset.insert(addr).unwrap();
            }
        } else {
            avset.insert(&ofi.mapped_addr.as_ref().unwrap()[0]).unwrap();
        }

        let mut ctx = ofi.info_entry.allocate_context();
        let mc = libfabric::mcast::MulticastGroupBuilder::from_av_set(&avset).build();
        let mc = async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(ep) => mc
                    .join_collective_async(&ep, libfabric::enums::JoinOptions::new(), &mut ctx)
                    .await
                    .unwrap(),
                MyEndpoint::Connectionless(ep) => mc
                    .join_collective_async(&ep, libfabric::enums::JoinOptions::new(), &mut ctx)
                    .await
                    .unwrap(),
            }
        });

        (ofi, mc.1)
    }

    #[test]
    fn collective_0() {
        collective(true, "collective_0", false);
    }

    #[test]
    fn collective_1() {
        collective(false, "collective_0", false);
    }

    fn barrier(server: bool, name: &str, connected: bool) {
        println!("Start barrier Collective!");
        let (ofi, mc) = collective(server, name, connected);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => ep.barrier_async(&mc, &mut ctx).await.unwrap(),
            }
        });
        println!("Done barrier Collective!");
    }

    #[test]
    fn barrier0() {
        barrier(true, "barrier0", false);
    }

    #[test]
    fn barrier1() {
        barrier(false, "barrier0", false);
    }

    fn broadcast(server: bool, name: &str, connected: bool) {
        println!("Start broadcast Collective!");
        let (ofi, mc) = collective(server, name, connected);
        
        let mut reg_mem = ofi.reg_mem.borrow_mut();
        if server {
            reg_mem.fill(2);
        } else {
            reg_mem.fill(1);
        };

        let expected = if server {
            reg_mem[..4096].to_vec()
        } else {
            reg_mem[..4096].iter().map(|v| v + 1).collect()
        };
        
        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();

        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.broadcast_async(
                        &mut reg_mem[..4096],
                        Some(&mr.descriptor()),
                        &mc,
                        &ofi.mapped_addr.as_ref().unwrap()[0],
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        println!("Done broadcast Collective!");
        assert_eq!(&reg_mem[..4096], expected);
    }

    #[test]
    fn broadcast0() {
        broadcast(true, "broadcast0", false);
    }

    #[test]
    fn broadcast1() {
        broadcast(false, "broadcast0", false);
    }

    fn alltoall(server: bool, name: &str, connected: bool) {
        println!("Start alltoall Collective!");
        let (ofi, mc) = collective(server, name, connected);
        
        let mut reg_mem = ofi.reg_mem.borrow_mut();
        if server {
            reg_mem.fill(2);
        } else {
            reg_mem.fill(1);
        };

        let expected = if server {
            vec![1; 1024 * 2]
        } else {
            vec![2; 1024 * 2]
        };
        
        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();

        let half = reg_mem.len() / 2;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(half);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.alltoall_async(
                        send_buf,
                        Some(&mr.descriptor()),
                        recv_buf,
                        Some(&mr.descriptor()),
                        &mc,
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        println!("Done alltoall Collective!");
        assert_eq!(&reg_mem[..], expected);
    }

    // #[test]
    // fn alltoall0() {
    //     alltoall(true, "alltoall0", false);
    // }

    // #[test]
    // fn alltoall1() {
    //     alltoall(false, "alltoall0", false);
    // }

    fn allreduce(server: bool, name: &str, connected: bool) {
        println!("Start allreduce Collective!");
        let (ofi, mc) = collective(server, name, connected);
                
        let mut reg_mem = ofi.reg_mem.borrow_mut();
        if server {
            reg_mem.fill(2);
        } else {
            reg_mem.fill(1);
        };

        let expected = if server {
            vec![3; 1024 * 1]
        } else {
            vec![3; 1024 * 1]
        };
        
        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();

        let half = reg_mem.len() / 2;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(1024);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.allreduce_async(
                        send_buf,
                        Some(&mr.descriptor()),
                        &mut recv_buf[..1024],
                        Some(&mr.descriptor()),
                        &mc,
                        libfabric::enums::CollAtomicOp::Sum,
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        println!("Done allreduce Collective!");
        assert_eq!(&recv_buf[..1024], expected);
    }

    #[test]
    fn allreduce0() {
        allreduce(true, "allreduce0", false);
    }

    #[test]
    fn allreduce1() {
        allreduce(false, "allreduce0", false);
    }

    fn allgather(server: bool, name: &str, connected: bool) {
        println!("Start Allgather Collective!");

        let (ofi, mc) = collective(server, name, connected);
        let mut reg_mem = ofi.reg_mem.borrow_mut();
        if server {
            reg_mem.fill(2);
        } else {
            reg_mem.fill(1);
        };

        let expected = if server {
            [vec![2; 512], vec![1; 512]].concat()
        } else {
            [vec![2; 512], vec![1; 512]].concat()
        };
        
        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();

        // let quart = reg_mem.len()/4;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(512);
        let mut ctx = ofi.info_entry.allocate_context();

        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.allgather_async(
                        send_buf,
                        Some(&mr.descriptor()),
                        &mut recv_buf[..1024],
                        Some(&mr.descriptor()),
                        &mc,
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        println!("Done Allgather Collective!");

        assert_eq!(&recv_buf[..1024], expected);
    }

    // #[test]
    // fn allgather0() {
    //     allgather(true, "allgather0", false);
    // }

    // #[test]
    // fn allgather1() {
    //     allgather(false, "allgather0", false);
    // }

    fn reduce_scatter(server: bool, name: &str, connected: bool) {
        println!("Start reduce_scatter Collective!");

        let (ofi, mc) = collective(server, name, connected);
        let mut reg_mem = ofi.reg_mem.borrow_mut();
        if server {
            reg_mem.fill(2);
        } else {
            reg_mem.fill(1);
        };
        
        let expected = if server {
            vec![3; 1024]
        } else {
            vec![3; 1024]
        };
        
        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();

        // let quart = reg_mem.len()/4;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(1024);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.reduce_scatter_async(
                        send_buf,
                        Some(&mr.descriptor()),
                        &mut recv_buf[..1024],
                        Some(&mr.descriptor()),
                        &mc,
                        libfabric::enums::CollAtomicOp::Sum,
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        println!("Done reduce_scatter Collective!");

        assert_eq!(&recv_buf[..1024], expected);
    }

    // #[test]
    // fn reduce_scatter0() {
    //     reduce_scatter(true, "reduce_scatter0", false);
    // }

    // #[test]
    // fn reduce_scatter1() {
    //     reduce_scatter(false, "reduce_scatter0", false);
    // }

    fn reduce(server: bool, name: &str, connected: bool) {
        println!("Start reduce Collective!");
        let (ofi, mc) = collective(server, name, connected);
        let mut reg_mem = ofi.reg_mem.borrow_mut();
        if server {
            reg_mem.fill(2);
        } else {
            reg_mem.fill(1);
        };

        let expected = if server {
            vec![3; 1024]
        } else {
            vec![1; 1024]
        };
        
        let borrow = ofi.mr.borrow();
        let mr = borrow.as_ref().unwrap();

        // let quart = reg_mem.len()/4;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(1024);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.reduce_async(
                        send_buf,
                        Some(&mr.descriptor()),
                        &mut recv_buf[..1024],
                        Some(&mr.descriptor()),
                        &mc,
                        &ofi.mapped_addr.as_ref().unwrap()[0],
                        libfabric::enums::CollAtomicOp::Sum,
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        println!("Done reduce Collective!");
        assert_eq!(&recv_buf[..1024], expected);
    }

    // #[test]
    // fn reduce0() {
    //     reduce(true, "reduce0", false);
    // }

    // #[test]
    // fn reduce1() {
    //     reduce(false, "reduce0", false);
    // }

    fn scatter(server: bool, name: &str, connected: bool) {
        println!("Start scatter Collective!");
        let (ofi, mc) = collective(server, name, connected);
        let (mut reg_mem, expected) = if server {
            (vec![2; 1024 * 2], vec![2; 512])
        } else {
            (vec![1; 1024 * 2], vec![2; 512])
        };

        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
            .access_collective()
            .build(&ofi.domain)
            .unwrap();

        let mr: MemoryRegion = match mr {
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
        // let quart = reg_mem.len()/4;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(1024);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.scatter_async(
                        send_buf,
                        Some(&mr.descriptor()),
                        &mut recv_buf[..512],
                        Some(&mr.descriptor()),
                        &mc,
                        &ofi.mapped_addr.as_ref().unwrap()[0],
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        println!("Done scatter Collective!");
        assert_eq!(recv_buf[..512], expected);
    }

    #[test]
    fn scatter0() {
        scatter(true, "scatter0", false);
    }

    #[test]
    fn scatter1() {
        scatter(false, "scatter0", false);
    }

    fn gather(server: bool, name: &str, connected: bool) {
        println!("Start gather Collective!");
        let (ofi, mc) = collective(server, name, connected);
        let (mut reg_mem, expected) = if server {
            (vec![2; 1024 * 2], [vec![2; 512], vec![1; 512]].concat())
        } else {
            (vec![1; 1024 * 2], vec![1; 512])
        };

        let mr = MemoryRegionBuilder::new(&reg_mem, libfabric::enums::HmemIface::System)
            .access_recv()
            .access_send()
            .access_write()
            .access_read()
            .access_remote_write()
            .access_remote_read()
            .access_collective()
            .build(&ofi.domain)
            .unwrap();

        let mr: MemoryRegion = match mr {
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
        // let quart = reg_mem.len()/4;
        let (send_buf, recv_buf) = reg_mem.split_at_mut(512);
        let mut ctx = ofi.info_entry.allocate_context();
        async_std::task::block_on(async {
            match &ofi.ep {
                MyEndpoint::Connected(_) => todo!(),
                MyEndpoint::Connectionless(ep) => {
                    ep.gather_async(
                        send_buf,
                        Some(&mr.descriptor()),
                        &mut recv_buf[..1024],
                        Some(&mr.descriptor()),
                        &mc,
                        &ofi.mapped_addr.as_ref().unwrap()[0],
                        CollectiveOptions::new(),
                        &mut ctx,
                    )
                    .await
                    .unwrap();
                }
            }
        });
        println!("Done gather Collective!");
        assert_eq!(recv_buf[..1024], expected);
    }

    // #[test]
    // fn gather0() {
    //     gather(true, "gather0", false);
    // }

    // #[test]
    // fn gather1() {
    //     gather(false, "gather0", false);
    // }
}