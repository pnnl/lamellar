pub mod common; // Public to supress lint warnings (unused function)
use common::IP;

use prefix::{HintsCaps, define_test, call};

pub mod sync_; // Public to supress lint warnings (unused function)
#[cfg(any(feature="use-async-std", feature="use-tokio"))]
pub mod async_; // Public to supress lint warnings (unused function)

use sync_ as prefix;

define_test!(pp_server_rma, async_pp_server_rma, {
    let mut gl_ctx = prefix::TestsGlobalCtx::new();

    let mut dom_attr = libfabric::domain::DomainAttr::new();
    dom_attr.threading = libfabric::enums::Threading::Domain;
    dom_attr.mr_mode = libfabric::enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw();
    dom_attr.resource_mgmt = libfabric::enums::ResourceMgmt::Enabled;
    

    let mut tx_attr = libfabric::xcontext::TxAttr::new();
        tx_attr.tclass(libfabric::enums::TClass::BulkData); //.op_flags(libfabric::enums::TransferOptions::DELIVERY_COMPLETE);

   
    let hintscaps = if true {
            HintsCaps::Msg(
                libfabric::info::InfoHints::new()
                .caps(libfabric::infocapsoptions::InfoCaps::new().msg().rma())
                .tx_attr(tx_attr)
                .mode(libfabric::enums::Mode::new().context())
                .domain_attr(dom_attr)
                .addr_format(libfabric::enums::AddressFormat::Unspec)
            )
        }
        else {
            HintsCaps::Tagged(
                libfabric::info::InfoHints::new()
                .caps(libfabric::infocapsoptions::InfoCaps::new().tagged().rma())
                .tx_attr(tx_attr)
                .mode(libfabric::enums::Mode::new().context())
                .domain_attr(dom_attr)
                .addr_format(libfabric::enums::AddressFormat::Unspec)
            ) 
        };

    
    let (infocap, ep, domain, tx_cq, rx_cq, tx_cntr, rx_cntr, mut mr, _av, mut mr_desc) = 
        call!(prefix::ft_init_fabric, hintscaps, &mut gl_ctx, "".to_owned(), "9222".to_owned(), true);

    match infocap {
        prefix::InfoWithCaps::Msg(entry) => {
            
            let remote = call!(prefix::ft_exchange_keys, &entry, &mut gl_ctx, mr.as_mut().unwrap(), &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&domain, &ep, &mut mr_desc);

            let test_sizes = gl_ctx.test_sizes.clone();
            for msg_size in test_sizes {
                call!(prefix::pingpong_rma, &entry, &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, prefix::RmaOp::RMA_WRITE, &remote, 100, 10, msg_size, true);
            }

            call!(prefix::ft_finalize, &entry, &mut gl_ctx, &ep, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
        }
        prefix::InfoWithCaps::Tagged(entry) => {
            
            let remote = call!(prefix::ft_exchange_keys, &entry, &mut gl_ctx, mr.as_mut().unwrap(), &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &domain, &ep, &mut mr_desc);

            let test_sizes = gl_ctx.test_sizes.clone();
            for msg_size in test_sizes {
                call!(prefix::pingpong_rma, &entry, &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, prefix::RmaOp::RMA_WRITE, &remote, 100, 10, msg_size, true);
            }

            call!(prefix::ft_finalize, &entry, &mut gl_ctx, &ep, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
        }
    }
});

define_test!(pp_client_rma, async_pp_client_rma, {
    let mut gl_ctx = prefix::TestsGlobalCtx::new();
    let mut dom_attr = libfabric::domain::DomainAttr::new();
    dom_attr.threading = libfabric::enums::Threading::Domain;
    dom_attr.mr_mode = libfabric::enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw();
    dom_attr.resource_mgmt = libfabric::enums::ResourceMgmt::Enabled;
    

    let mut tx_attr = libfabric::xcontext::TxAttr::new();
        tx_attr.tclass(libfabric::enums::TClass::BulkData);//.op_flags(libfabric::enums::TransferOptions::DELIVERY_COMPLETE);
    
    let hintscaps = if true {
            HintsCaps::Msg(
                libfabric::info::InfoHints::new()
                .caps(libfabric::infocapsoptions::InfoCaps::new().msg().rma())
                .tx_attr(tx_attr)
                .mode(libfabric::enums::Mode::new().context())
                .domain_attr(dom_attr)
                .addr_format(libfabric::enums::AddressFormat::Unspec)
            )
        }
        else {
            HintsCaps::Tagged(
                libfabric::info::InfoHints::new()
                .caps(libfabric::infocapsoptions::InfoCaps::new().tagged().rma())
                .tx_attr(tx_attr)
                .mode(libfabric::enums::Mode::new().context())
                .domain_attr(dom_attr)
                .addr_format(libfabric::enums::AddressFormat::Unspec)
            ) 
        };
    
    
    let (infocap, ep, domain, tx_cq, rx_cq, tx_cntr, rx_cntr, mut mr, _av, mut mr_desc) = 
        call!(prefix::ft_init_fabric, hintscaps, &mut gl_ctx, IP.to_owned(), "9222".to_owned(), false);
    
    match infocap {
        prefix::InfoWithCaps::Msg(entry) => {
            let remote = call!(prefix::ft_exchange_keys, &entry, &mut gl_ctx, mr.as_mut().unwrap(), &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&domain, &ep, &mut mr_desc);

            let test_sizes = gl_ctx.test_sizes.clone();
            for msg_size in test_sizes {
                call!(prefix::pingpong_rma, &entry, &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, prefix::RmaOp::RMA_WRITE, &remote, 100, 10, msg_size, false);
            }

            call!(prefix::ft_finalize, &entry, &mut gl_ctx, &ep, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
        }
        prefix::InfoWithCaps::Tagged(entry) => {
            let remote = call!(prefix::ft_exchange_keys, &entry, &mut gl_ctx, mr.as_mut().unwrap(), &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&domain, &ep, &mut mr_desc);
            let test_sizes = gl_ctx.test_sizes.clone();
            for msg_size in test_sizes {
                call!(prefix::pingpong_rma, &entry, &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, prefix::RmaOp::RMA_WRITE, &remote, 100, 10, msg_size, false);
            }

            call!(prefix::ft_finalize, &entry, &mut gl_ctx, &ep, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
            // drop(domain);
        }
    }
});