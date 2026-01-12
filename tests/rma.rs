pub mod sync_;
pub mod sync_rma {
    use libfabric::{cq::{ReadCq, WaitCq}, infocapsoptions::InfoCaps, iovec::{IoVec, IoVecMut, RemoteMemAddrVec, RemoteMemAddrVecMut}, msg::{MsgRma, MsgRmaConnected, MsgRmaConnectedMut, MsgRmaMut}};
    use crate::sync_::tests::{ft_progress, handshake, handshake_connectionless, Either};
    
    fn writeread(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg().rma()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg().rma()))
        };

        ofi.exchange_keys();

        {
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            if server {
                for i in 0..1024 * 2 {
                    let v = (i % 256) as u8;
                    reg_mem[i] = v;
                }
            }
            else{
                reg_mem.fill(0);
            }
        } 


        let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();
        if server {
            // Write inject a single buffer
            ofi.write(0..128, 0, None);

            // Send completion ack
            ofi.send(512..1024, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Write a single buffer
            ofi.write(0..512, 0, None);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            // Send completion ack
            ofi.send(512..1024, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            {
                let reg_mem = ofi.reg_mem.borrow();
                let borrow = ofi.mr.borrow();
                let mr = borrow.as_ref().unwrap();
                let descs = [mr.descriptor(), mr.descriptor()];
                
                // Write vector of buffers
                let iovs = [
                    IoVec::from_slice(&reg_mem[..512]),
                    IoVec::from_slice(&reg_mem[512..1024]),
                ];
                ofi.writev(&iovs, 0, Some(&descs));
                ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            }
            // Send completion ack
            ofi.send(512..1024, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // Recv a completion ack
            ofi.recv(512..1024, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        } else {
            // Recv a completion ack
            ofi.recv(512..1024, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            while ofi.reg_mem.borrow()[127] == 0 {
                ft_progress(ofi.cq_type.tx_cq());
                ft_progress(ofi.cq_type.rx_cq());
            }
            assert_eq!(&ofi.reg_mem.borrow()[..128], &expected[..128]);

            // Recv a completion ack
            ofi.recv(512..1024, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[..512], &expected[..512]);

            // Recv a completion ack
            ofi.recv(1024..1536, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[..1024], &expected[..1024]);

            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);

            // Read buffer from remote memory
            ofi.read(1024..1536, 0);
            if let Err(_) = ofi.cq_type.tx_cq().sread(1, -1) {
                let err =  ofi.cq_type.tx_cq().readerr(0).unwrap();
                panic!("Error in read: {}", err.error());
            }
            assert_eq!(&ofi.reg_mem.borrow()[1024..1536], &expected[512..1024]);

            {
                // Read vector of buffers from remote memory
                let mut reg_mem = ofi.reg_mem.borrow_mut();
                let borrow = ofi.mr.borrow();
                let mr = borrow.as_ref().unwrap();
                let descs = [mr.descriptor(), mr.descriptor()];
                let (mem0, mem1) = reg_mem[1536..].split_at_mut(256);
                let iovs = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
                ofi.readv(&iovs, 0, Some(&descs));
                ofi.cq_type.tx_cq().sread(1, -1).unwrap();
                
                assert_eq!(mem0, &expected[..256]);
                assert_eq!(&mem1[..256], &expected[..256]);
                
            }
            // Send completion ack
            ofi.send(512..1024, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        }
    }

    #[test]
    fn conn_writeread0() {
        writeread(true, "conn_writeread0", true);
    }

    #[test]
    fn conn_writeread1() {
        writeread(false, "conn_writeread0", true);
    }

    #[test]
    fn writeread0() {
        writeread(true, "writeread0", false);
    }

    #[test]
    fn writeread1() {
        writeread(false, "writeread0", false);
    }

    fn writereadmsg(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg().rma()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg().rma()))
        };

        ofi.exchange_keys();

        {
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            if server {
                for i in 0..1024 * 2 {
                    let v = (i % 256) as u8;
                    reg_mem[i] = v;
                }
            }
            else {
                reg_mem.fill(0);
            }
        }


        // let descs = [mr.descriptor(), mr.descriptor()];
        // let desc0 = Some(mr.descriptor());
        let mapped_addr = ofi.mapped_addr.clone();

        let expected: Vec<u8> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

        let mut ctx = ofi.info_entry.allocate_context();
        if server {
            let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
            let rma_addr = remote_mem_info.slice::<u8>(..128);
            let mut rma_iov = RemoteMemAddrVec::new();
            rma_iov.push(rma_addr);

            {
                let reg_mem = ofi.reg_mem.borrow();

                let iov = IoVec::from_slice(&reg_mem[..128]);
                let borrow = ofi.mr.borrow();
                let mr = borrow.as_ref().unwrap();
                let desc0 = Some(mr.descriptor());
                
                let msg = if connected {
                    Either::Right(MsgRmaConnected::from_iov(
                        &iov,
                        desc0.as_ref(),
                        &rma_iov,
                        None,
                        &mut ctx,
                    ))
                } else {
                    Either::Left(MsgRma::from_iov(
                        &iov,
                        desc0.as_ref(),
                        &mapped_addr.as_ref().unwrap()[1],
                        &rma_iov,
                        None,
                        &mut ctx,
                    ))
                };
                // Write inject a single buffer
                ofi.writemsg(&msg);
                ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            }

            // Send completion ack
            ofi.send(512..1024, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            let reg_mem = ofi.reg_mem.borrow();

            let iov = IoVec::from_slice(&reg_mem[..512]);

            let rma_addr = remote_mem_info.slice::<u8>(..512);
            let mut rma_iov = RemoteMemAddrVec::new();
            rma_iov.push(rma_addr);

            let borrow = ofi.mr.borrow();
            let mr = borrow.as_ref().unwrap();
            let desc0 = Some(mr.descriptor());

            let msg = if connected {
                Either::Right(MsgRmaConnected::from_iov(
                    &iov,
                    desc0.as_ref(),
                    &rma_iov,
                    Some(128),
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgRma::from_iov(
                    &iov,
                    desc0.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iov,
                    Some(128),
                    &mut ctx,
                ))
            };

            // Write a single buffer
            ofi.writemsg(&msg);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            drop(reg_mem);
            // Send completion ack
            ofi.send(512..1024, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            let reg_mem = ofi.reg_mem.borrow();
            let iov0 = IoVec::from_slice(&reg_mem[..512]);
            let iov1 = IoVec::from_slice(&reg_mem[512..1024]);
            let iovs = [iov0, iov1];
            let rma_addr0 = remote_mem_info.slice::<u8>(..512);

            let rma_addr1 = remote_mem_info.slice::<u8>(512..1024);
            let borrow = ofi.mr.borrow();
            let mr = borrow.as_ref();

            let descs: Option<[libfabric::mr::MemoryRegionDesc<'_>; 2]> = if mr.is_none() {
                None
            } else {
                Some([mr.unwrap().descriptor(), mr.unwrap().descriptor()])
            };

            let desc_ref: Option<&[libfabric::mr::MemoryRegionDesc<'_>]> = descs.as_ref().map(|d| &d[..]);
            let mut rma_iov = RemoteMemAddrVec::new();
            rma_iov.push(rma_addr0);
            rma_iov.push(rma_addr1);

            let msg = if connected {
                Either::Right(MsgRmaConnected::from_iov_slice(
                    &iovs,
                    desc_ref,
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgRma::from_iov_slice(
                    &iovs,
                    desc_ref,
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            };

            ofi.writemsg(&msg);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            drop(reg_mem);

            // Send completion ack
            ofi.send(512..1024, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
            // Recv a completion ack
            ofi.recv(512..1024, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
        } else {
            let mut remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow_mut();
            
            // Recv a completion ack
            ofi.recv(512..1024, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            while ofi.reg_mem.borrow()[127] == 0 {
                ft_progress(ofi.cq_type.tx_cq());
                ft_progress(ofi.cq_type.rx_cq());
            }
            assert_eq!(&ofi.reg_mem.borrow()[..128], &expected[..128]);

            // Recv a completion ack
            ofi.recv(512..1024, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[..512], &expected[..512]);

            // Recv a completion ack
            ofi.recv(1024..1536, false);
            ofi.cq_type.rx_cq().sread(1, -1).unwrap();
            assert_eq!(&ofi.reg_mem.borrow()[..1024], &expected[..1024]);

            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);
            // let base_addr = remote_mem_info.borrow().mem_address();
            // let mapped_key  = &remote_mem_info.borrow().key();
            
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            {
                let mut iov = IoVecMut::from_slice(&mut reg_mem[1024..1536]);
                let rma_addr = remote_mem_info.slice_mut::<u8>(..);
                let mut rma_iov = RemoteMemAddrVecMut::new();
                rma_iov.push(rma_addr);

                let borrow = ofi.mr.borrow();
                let mr = borrow.as_ref().unwrap();
                let desc0 = Some(mr.descriptor());

                // Read buffer from remote memory
                let msg = if connected {
                    Either::Right(MsgRmaConnectedMut::from_iov(
                        &mut iov,
                        desc0.as_ref(),
                        &rma_iov,
                        None,
                        &mut ctx,
                    ))
                } else {
                    Either::Left(MsgRmaMut::from_iov(
                        &mut iov,
                        desc0.as_ref(),
                        &mapped_addr.as_ref().unwrap()[1],
                        &rma_iov,
                        None,
                        &mut ctx,
                    ))
                };
                ofi.readmsg(&msg);
                if let Err(_) = ofi.cq_type.tx_cq().sread(1, -1) {
                    let err =  ofi.cq_type.tx_cq().readerr(0).unwrap();
                    panic!("Error in read: {}", err.error());
                }
                assert_eq!(&reg_mem[1024..1536], &expected[512..1024]);
            }

            // Read vector of buffers from remote memory
            let (mem0, mem1) = reg_mem[1536..].split_at_mut(256);
            let mut iovs = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
            let (rma_addr0, rma_addr1) = remote_mem_info.slice_mut::<u8>(..512).split_at_mut(256);

            let mut rma_iov = RemoteMemAddrVecMut::new();
            rma_iov.push(rma_addr0);
            rma_iov.push(rma_addr1);
            let borrow = ofi.mr.borrow();
            let mr = borrow.as_ref();

            let descs: Option<[libfabric::mr::MemoryRegionDesc<'_>; 2]> = if mr.is_none() {
                None
            } else {
                Some([mr.unwrap().descriptor(), mr.unwrap().descriptor()])
            };

            let desc_ref: Option<&[libfabric::mr::MemoryRegionDesc<'_>]> = descs.as_ref().map(|d| &d[..]);

            let msg = if connected {
                Either::Right(MsgRmaConnectedMut::from_iov_slice(
                    &mut iovs,
                    desc_ref,
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgRmaMut::from_iov_slice(
                    &mut iovs,
                    desc_ref,
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            };
            ofi.readmsg(&msg);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();

            assert_eq!(mem0, &expected[..256]);
            assert_eq!(&mem1[..256], &expected[..256]);
            drop(reg_mem);

            // Send completion ack
            ofi.send(512..1024, None, false);
            ofi.cq_type.tx_cq().sread(1, -1).unwrap();
        }
    }

    #[test]
    fn writereadmsg0() {
        writereadmsg(true, "writereadmsg0", false);
    }

    #[test]
    fn writereadmsg1() {
        writereadmsg(false, "writereadmsg0", false);
    }

    #[test]
    fn conn_writereadmsg0() {
        writereadmsg(true, "conn_writereadmsg0", true);
    }

    #[test]
    fn conn_writereadmsg1() {
        writereadmsg(false, "conn_writereadmsg0", true);
    }    
}

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
pub mod async_;
#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
pub mod async_rma {
    use libfabric::{infocapsoptions::InfoCaps, iovec::{IoVec, IoVecMut, RemoteMemAddrVec, RemoteMemAddrVecMut}, msg::{MsgRma, MsgRmaConnected, MsgRmaConnectedMut, MsgRmaMut}};

    use crate::async_::{handshake, handshake_connectionless, Either};


    fn writeread(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg().rma()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg().rma()))
        };
        
        ofi.exchange_keys();

        {
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            if server {
                for i in 0..1024 * 2 {
                    let v = (i % 256) as u8;
                    reg_mem[i] = v;
                }
            }
            else{
                reg_mem.fill(0);
            }
        } 

        let expected: Vec<_> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

        if server {
            // Write inject a single buffer
            ofi.write(0..128, 0, None);

            // Send completion ack
            ofi.send(512..1024, None);

            // Write a single buffer
            ofi.write(0..512, 0, None);

            // Send completion ack
            ofi.send(512..1024, None);

            {
                let reg_mem = ofi.reg_mem.borrow();
                let borrow = ofi.mr.borrow();
                let mr = borrow.as_ref().unwrap();
                let descs = [mr.descriptor(), mr.descriptor()];

                // Write vector of buffers
                let iovs = [
                    IoVec::from_slice(&reg_mem[..512]),
                    IoVec::from_slice(&reg_mem[512..1024]),
                ];
                ofi.writev(&iovs, 0, Some(&descs));
            }

            // Send completion ack
            ofi.send(512..1024, None);

            // Recv a completion ack
            ofi.recv(512..1024);
        } else {
            // Recv a completion ack
            ofi.recv(512..1024);
            assert_eq!(&ofi.reg_mem.borrow()[..128], &expected[..128]);

            // Recv a completion ack
            ofi.recv(512..1024);
            assert_eq!(&ofi.reg_mem.borrow()[..512], &expected[..512]);

            // Recv a completion ack
            ofi.recv(1024..1536);
            assert_eq!(&ofi.reg_mem.borrow()[..1024], &expected[..1024]);

            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);

            // Read buffer from remote memory
            ofi.read(1024..1536, 0);
            assert_eq!(&ofi.reg_mem.borrow()[1024..1536], &expected[512..1024]);

            {
                // Read vector of buffers from remote memory
                let mut reg_mem = ofi.reg_mem.borrow_mut();
                let borrow = ofi.mr.borrow();
                let mr = borrow.as_ref().unwrap();
                let descs = [mr.descriptor(), mr.descriptor()];
                let (mem0, mem1) = reg_mem[1536..].split_at_mut(256);
                let iovs = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
                ofi.readv(&iovs, 0, Some(&descs));

                assert_eq!(mem0, &expected[..256]);
                assert_eq!(&mem1[..256], &expected[..256]);
            }

            // Send completion ack
            ofi.send(512..1024, None);
        }
    }

    #[test]
    fn async_conn_writeread0() {
        writeread(true, "conn_writeread0", true);
    }

    #[test]
    fn async_conn_writeread1() {
        writeread(false, "conn_writeread0", true);
    }

    #[test]
    fn async_writeread0() {
        writeread(true, "writeread0", false);
    }

    #[test]
    fn async_writeread1() {
        writeread(false, "writeread0", false);
    }

    fn writereadmsg(server: bool, name: &str, connected: bool) {
        let mut ofi = if connected {
            handshake(None, server, name, Some(InfoCaps::new().msg().rma()))
        } else {
            handshake_connectionless(None, server, name, Some(InfoCaps::new().msg().rma()))
        };

        ofi.exchange_keys();

        {
            let mut reg_mem = ofi.reg_mem.borrow_mut();
            if server {
                for i in 0..1024 * 2 {
                    let v = (i % 256) as u8;
                    reg_mem[i] = v;
                }
            }
            else {
                reg_mem.fill(0);
            }
        }


        
        let mapped_addr = ofi.mapped_addr.clone();
        let expected: Vec<u8> = (0..1024).map(|v: usize| (v % 256) as u8).collect();

        let mut ctx = ofi.info_entry.allocate_context();
        if server {
            let remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow();
            let rma_addr = remote_mem_info.slice::<u8>(..128);
            let mut rma_iov = RemoteMemAddrVec::new();
            rma_iov.push(rma_addr);

            {
                let reg_mem = ofi.reg_mem.borrow();

                let iov = IoVec::from_slice(&reg_mem[..128]);
                let borrow = ofi.mr.borrow();
                let mr = borrow.as_ref().unwrap();
                let desc0 = Some(mr.descriptor());
                
                let mut msg = if connected {
                    Either::Right(MsgRmaConnected::from_iov(
                        &iov,
                        desc0.as_ref(),
                        &rma_iov,
                        None,
                        &mut ctx,
                    ))
                } else {
                    Either::Left(MsgRma::from_iov(
                        &iov,
                        desc0.as_ref(),
                        &mapped_addr.as_ref().unwrap()[1],
                        &rma_iov,
                        None,
                        &mut ctx,
                    ))
                };
                // Write inject a single buffer
                ofi.writemsg(&mut msg);
            }


            // Send completion ack
            ofi.send(512..1024, None);

            let reg_mem = ofi.reg_mem.borrow();

            let iov = IoVec::from_slice(&reg_mem[..512]);

            let rma_addr = remote_mem_info.slice::<u8>(..512);
            let mut rma_iov = RemoteMemAddrVec::new();
            rma_iov.push(rma_addr);


            let borrow = ofi.mr.borrow();
            let mr = borrow.as_ref().unwrap();
            let desc0 = Some(mr.descriptor());

            let mut msg = if connected {
                Either::Right(MsgRmaConnected::from_iov(
                    &iov,
                    desc0.as_ref(),
                    &rma_iov,
                    Some(128),
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgRma::from_iov(
                    &iov,
                    desc0.as_ref(),
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iov,
                    Some(128),
                    &mut ctx,
                ))
            };

            // Write a single buffer
            ofi.writemsg(&mut msg);
            drop(reg_mem);

            // Send completion ack
            ofi.send(512..1024, None);

            let reg_mem = ofi.reg_mem.borrow();

            let iov0 = IoVec::from_slice(&reg_mem[..512]);
            let iov1 = IoVec::from_slice(&reg_mem[512..1024]);
            let iovs = [iov0, iov1];
            let rma_addr0 = remote_mem_info.slice::<u8>(..512);
            let rma_addr1 = remote_mem_info.slice::<u8>(512..1024);
            let borrow = ofi.mr.borrow();
            let mr = borrow.as_ref();

            let descs: Option<[libfabric::mr::MemoryRegionDesc<'_>; 2]> = if mr.is_none() {
                None
            } else {
                Some([mr.unwrap().descriptor(), mr.unwrap().descriptor()])
            };

            let desc_ref: Option<&[libfabric::mr::MemoryRegionDesc<'_>]> = descs.as_ref().map(|d| &d[..]);
            let mut rma_iov = RemoteMemAddrVec::new();
            rma_iov.push(rma_addr0);
            rma_iov.push(rma_addr1);

            let mut msg = if connected {
                Either::Right(MsgRmaConnected::from_iov_slice(
                    &iovs,
                    desc_ref,
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgRma::from_iov_slice(
                    &iovs,
                    desc_ref,
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            };

            ofi.writemsg(&mut msg);
            drop(reg_mem);

            // Send completion ack
            ofi.send(512..1024, None);

            // Recv a completion ack
            ofi.recv(512..1024);
        } else {
            let mut remote_mem_info = ofi.remote_mem_info.as_ref().unwrap().borrow_mut();

            // Recv a completion ack
            ofi.recv(512..1024);
            assert_eq!(&ofi.reg_mem.borrow()[..128], &expected[..128]);

            // Recv a completion ack
            ofi.recv(512..1024);
            assert_eq!(&ofi.reg_mem.borrow()[..512], &expected[..512]);

            // Recv a completion ack
            ofi.recv(1024..1536);
            assert_eq!(&ofi.reg_mem.borrow()[..1024], &expected[..1024]);

            ofi.reg_mem.borrow_mut().iter_mut().for_each(|v| *v = 0);

            let mut reg_mem = ofi.reg_mem.borrow_mut();
            {
                let mut iov = IoVecMut::from_slice(&mut reg_mem[1024..1536]);
                let rma_addr = remote_mem_info.slice_mut::<u8>(..512);
                let mut rma_iov = RemoteMemAddrVecMut::new();
                rma_iov.push(rma_addr);

                let borrow = ofi.mr.borrow();
                let mr = borrow.as_ref().unwrap();
                let desc0 = Some(mr.descriptor());
                // RmaIoVec::new()
                //     .address(base_addr)
                //     .len(512)
                //     .mapped_key(&key);
                // Read buffer from remote memory
                let mut msg = if connected {
                    Either::Right(MsgRmaConnectedMut::from_iov(
                        &mut iov,
                        desc0.as_ref(),
                        &rma_iov,
                        None,
                        &mut ctx,
                    ))
                } else {
                    Either::Left(MsgRmaMut::from_iov(
                        &mut iov,
                        desc0.as_ref(),
                        &mapped_addr.as_ref().unwrap()[1],
                        &rma_iov,
                        None,
                        &mut ctx,
                    ))
                };
                ofi.readmsg(&mut msg);
                assert_eq!(&reg_mem[1024..1536], &expected[512..1024]);
            }

            // // Read vector of buffers from remote memory
            let (mem0, mem1) = reg_mem[1536..].split_at_mut(256);
            let mut iovs = [IoVecMut::from_slice(mem0), IoVecMut::from_slice(mem1)];
            let (rma_addr0, rma_addr1) = remote_mem_info.slice_mut::<u8>(..512).split_at_mut(256);
            let mut rma_iov = RemoteMemAddrVecMut::new();
            rma_iov.push(rma_addr0);
            rma_iov.push(rma_addr1);

            let borrow = ofi.mr.borrow();
            let mr = borrow.as_ref();

            let descs: Option<[libfabric::mr::MemoryRegionDesc<'_>; 2]> = if mr.is_none() {
                None
            } else {
                Some([mr.unwrap().descriptor(), mr.unwrap().descriptor()])
            };

            let desc_ref: Option<&[libfabric::mr::MemoryRegionDesc<'_>]> = descs.as_ref().map(|d| &d[..]);

            let mut msg = if connected {
                Either::Right(MsgRmaConnectedMut::from_iov_slice(
                    &mut iovs,
                    desc_ref,
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            } else {
                Either::Left(MsgRmaMut::from_iov_slice(
                    &mut iovs,
                    desc_ref,
                    &mapped_addr.as_ref().unwrap()[1],
                    &rma_iov,
                    None,
                    &mut ctx,
                ))
            };
            ofi.readmsg(&mut msg);

            assert_eq!(mem0, &expected[..256]);
            assert_eq!(&mem1[..256], &expected[..256]);
            drop(reg_mem);

            // Send completion ack
            ofi.send(512..1024, None);
        }
    }

    #[test]
    fn async_writereadmsg0() {
        writereadmsg(true, "writereadmsg0", false);
    }

    #[test]
    fn async_writereadmsg1() {
        writereadmsg(false, "writereadmsg0", false);
    }

    #[test]
    fn async_conn_writereadmsg0() {
        writereadmsg(true, "conn_writereadmsg0", true);
    }

    #[test]
    fn async_conn_writereadmsg1() {
        writereadmsg(false, "conn_writereadmsg0", true);
    }

}
