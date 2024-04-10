use common::{HintsCaps, IP};
use libfabric::{enums, xcontext::TxAttr, info::InfoHints};

pub mod common; // Public to supress lint warnings (unused function)
#[ignore]
#[test]
fn pp_server_rma() {
    let mut gl_ctx = common::TestsGlobalCtx::new();

    let mut dom_attr = libfabric::domain::DomainAttr::new();
        dom_attr
        .threading(enums::Threading::Domain)
        .mr_mode(enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw())
        .resource_mgmt(enums::ResourceMgmt::Enabled);
    

    let mut tx_attr = TxAttr::new();
        tx_attr.tclass(libfabric::enums::TClass::BulkData); //.op_flags(enums::TransferOptions::DELIVERY_COMPLETE);

   
    let hintscaps = if true {
            HintsCaps::Msg(
                InfoHints::new()
                .caps(libfabric::infocapsoptions::InfoCaps::new().msg().rma())
                .tx_attr(tx_attr)
                .mode(libfabric::enums::Mode::new().context())
                .domain_attr(dom_attr)
                .addr_format(libfabric::enums::AddressFormat::Unspec)
            )
        }
        else {
            HintsCaps::Tagged(
                InfoHints::new()
                .caps(libfabric::infocapsoptions::InfoCaps::new().tagged().rma())
                .tx_attr(tx_attr)
                .mode(libfabric::enums::Mode::new().context())
                .domain_attr(dom_attr)
                .addr_format(libfabric::enums::AddressFormat::Unspec)
            ) 
        };

    
    let (info, _fabric, ep, domain, tx_cq, rx_cq, tx_cntr, rx_cntr, _eq, mut mr, _av, mut mr_desc) = 
        common::ft_init_fabric(hintscaps, &mut gl_ctx, "".to_owned(), "9222".to_owned(), libfabric_sys::FI_SOURCE);

    match info {
        common::InfoWithCaps::Msg(info) => {
            let entries = info.get();
            
            if entries.is_empty() {
                panic!("No entires in fi_info");
            }
            let remote: libfabric::iovec::RmaIoVec = common::ft_exchange_keys(&entries[0], &mut gl_ctx, mr.as_mut().unwrap(), &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&domain, &ep, &mut mr_desc);

            let test_sizes = gl_ctx.test_sizes.clone();
            for msg_size in test_sizes {
                common::pingpong_rma(&entries[0], &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, common::RmaOp::RMA_WRITE, &remote, 100, 10, msg_size, true);
            }

            common::ft_finalize(&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
        }
        common::InfoWithCaps::Tagged(info) => {
            let entries = info.get();
            
            if entries.is_empty() {
                panic!("No entires in fi_info");
            }
            let remote: libfabric::iovec::RmaIoVec = common::ft_exchange_keys(&entries[0], &mut gl_ctx, mr.as_mut().unwrap(), &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&domain, &ep, &mut mr_desc);

            let test_sizes = gl_ctx.test_sizes.clone();
            for msg_size in test_sizes {
                common::pingpong_rma(&entries[0], &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, common::RmaOp::RMA_WRITE, &remote, 100, 10, msg_size, true);
            }

            common::ft_finalize(&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
        }
    }
}

#[ignore]
#[test]
fn pp_client_rma() {
    let mut gl_ctx = common::TestsGlobalCtx::new();
    let mut dom_attr = libfabric::domain::DomainAttr::new();
        dom_attr
        .threading(enums::Threading::Domain)
        .mr_mode(enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw())
        .resource_mgmt(enums::ResourceMgmt::Enabled);
    

    let mut tx_attr = TxAttr::new();
        tx_attr.tclass(libfabric::enums::TClass::BulkData);//.op_flags(enums::TransferOptions::DELIVERY_COMPLETE);
    
    let hintscaps = if true {
            HintsCaps::Msg(
                InfoHints::new()
                .caps(libfabric::infocapsoptions::InfoCaps::new().msg().rma())
                .tx_attr(tx_attr)
                .mode(libfabric::enums::Mode::new().context())
                .domain_attr(dom_attr)
                .addr_format(libfabric::enums::AddressFormat::Unspec)
            )
        }
        else {
            HintsCaps::Tagged(
                InfoHints::new()
                .caps(libfabric::infocapsoptions::InfoCaps::new().tagged().rma())
                .tx_attr(tx_attr)
                .mode(libfabric::enums::Mode::new().context())
                .domain_attr(dom_attr)
                .addr_format(libfabric::enums::AddressFormat::Unspec)
            ) 
        };
    
    
    let (info, _fabric, ep, domain, tx_cq, rx_cq, tx_cntr, rx_cntr,_eqq, mut mr, _av, mut mr_desc) = 
        common::ft_init_fabric(hintscaps, &mut gl_ctx, IP.to_owned(), "9222".to_owned(), 0);
    
    match info {
        common::InfoWithCaps::Msg(info) => {
            let entries = info.get();
            
            if entries.is_empty() {
                panic!("No entires in fi_info");
            }
            let remote: libfabric::iovec::RmaIoVec = common::ft_exchange_keys(&entries[0], &mut gl_ctx, mr.as_mut().unwrap(), &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&domain, &ep, &mut mr_desc);

            let test_sizes = gl_ctx.test_sizes.clone();
            for msg_size in test_sizes {
                common::pingpong_rma(&entries[0], &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, common::RmaOp::RMA_WRITE, &remote, 100, 10, msg_size, false);
            }

            common::ft_finalize(&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
        }
        common::InfoWithCaps::Tagged(info) => {
            let entries = info.get();
            
            if entries.is_empty() {
                panic!("No entires in fi_info");
            }
            let remote: libfabric::iovec::RmaIoVec = common::ft_exchange_keys(&entries[0], &mut gl_ctx, mr.as_mut().unwrap(), &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&domain, &ep, &mut mr_desc);

            let test_sizes = gl_ctx.test_sizes.clone();
            for msg_size in test_sizes {
                common::pingpong_rma(&entries[0], &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, common::RmaOp::RMA_WRITE, &remote, 100, 10, msg_size, false);
            }

            common::ft_finalize(&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
        }
    }
}