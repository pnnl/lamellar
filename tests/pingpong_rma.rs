use libfabric::{enums, TxAttr};

pub mod common; // Public to supress lint warnings (unused function)
#[ignore]
#[test]
fn pp_server_rma() {
    let mut gl_ctx = common::TestsGlobalCtx::new();

    let dom_attr = libfabric::domain::DomainAttr::new()
        .threading(enums::Threading::DOMAIN)
        .mr_mode(enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw())
        .resource_mgmt(enums::ResourceMgmt::ENABLED);
    
    let caps = libfabric::InfoCaps::new().msg().rma();
    

    let tx_attr = TxAttr::new().tclass(libfabric::enums::TClass::BULK_DATA); //.op_flags(enums::TransferOptions::DELIVERY_COMPLETE);

    let hints = libfabric::InfoHints::new()
        .caps(caps)
        .tx_attr(tx_attr)
        .mode(libfabric_sys::FI_CONTEXT) // [TODO]
        .domain_attr(dom_attr)
        .addr_format(libfabric::enums::AddressFormat::UNSPEC);
    
    
    let (info, fabric, ep, domain, tx_cq, rx_cq, tx_cntr, rx_cntr, eq, mut mr, av, mut mr_desc) = 
        common::ft_init_fabric(hints, &mut gl_ctx, "".to_owned(), "9222".to_owned(), libfabric_sys::FI_SOURCE);

    let entries: Vec<libfabric::InfoEntry> = info.get();
    
    if entries.is_empty() {
        panic!("No entires in fi_info");
    }
    let remote: libfabric::RmaIoVec = common::ft_exchange_keys(&entries[0], &mut gl_ctx, &mut mr.as_mut().unwrap(), &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&domain, &ep, &mut mr_desc);

    let test_sizes = gl_ctx.test_sizes.clone();
    for msg_size in test_sizes {
        common::pingpong_rma(&entries[0], &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, common::RmaOp::RMA_WRITE, &remote, 100, 10, msg_size, true);
    }

    common::ft_finalize(&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
    common::close_all(fabric, domain, eq, rx_cq, tx_cq, tx_cntr, rx_cntr, ep, mr.into(),av.into());
}

#[ignore]
#[test]
fn pp_client_rma() {
    let mut gl_ctx = common::TestsGlobalCtx::new();
    let dom_attr = libfabric::domain::DomainAttr::new()
        .threading(enums::Threading::DOMAIN)
        .mr_mode(enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw())
        .resource_mgmt(enums::ResourceMgmt::ENABLED);
    
    let caps = libfabric::InfoCaps::new().msg().rma();
    

    let tx_attr = TxAttr::new().tclass(libfabric::enums::TClass::BULK_DATA);//.op_flags(enums::TransferOptions::DELIVERY_COMPLETE);

    let hints = libfabric::InfoHints::new()
        .caps(caps)
        .tx_attr(tx_attr)
        .mode(libfabric_sys::FI_CONTEXT) // [TODO]
        .domain_attr(dom_attr)
        .addr_format(libfabric::enums::AddressFormat::UNSPEC);
    
    
    let (info, fabric, ep, domain, tx_cq, rx_cq, tx_cntr, rx_cntr, eq, mut mr, av, mut mr_desc) = 
        common::ft_init_fabric(hints, &mut gl_ctx, "172.17.110.21".to_owned(), "9222".to_owned(), 0);
    let entries: Vec<libfabric::InfoEntry> = info.get();
    
    if entries.is_empty() {
        panic!("No entires in fi_info");
    }
    let remote = common::ft_exchange_keys(&entries[0], &mut gl_ctx, &mut mr.as_mut().unwrap(), &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &domain, &ep, &mut mr_desc);
    

    let test_sizes = gl_ctx.test_sizes.clone();
    for msg_size in test_sizes {
        common::pingpong_rma(&entries[0], &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, common::RmaOp::RMA_WRITE, &remote, 100, 10, msg_size, false);
    }

    common::ft_finalize(&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
    common::close_all(fabric, domain, eq, rx_cq, tx_cq, tx_cntr, rx_cntr, ep, mr.into(), av.into());
}