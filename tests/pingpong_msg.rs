pub mod sync_; // Public to supress lint warnings (unused function)

#[cfg(any(feature="use-async-std", feature="use-tokio"))]
pub mod async_; // Public to supress lint warnings (unused function)
pub mod common; // Public to supress lint warnings (unused function)

use common::IP;
use prefix::{HintsCaps, EndpointCaps, define_test, call};
use sync_ as prefix; 


// To run the following tests do:
// 1. export FI_LOG_LEVEL="info" . 
// 2. Run the server (e.g. cargo test pp_server_msg -- --ignored --nocapture) 
//    There will be a large number of info printed. What we need is the last line with: listening on: fi_sockaddr_in:// <ip:port>
// 3. Copy the ip, port of the previous step
// 4. On the client (e.g. pp_client_msg) change  ft_client_connect node(<ip>) and service(<port>) to service and port of the copied ones
// 5. Run client (e.g. cargo test pp_client_msg -- --ignored --nocapture) 

define_test!(pp_server_msg, async_pp_server_msg, {
    let mut gl_ctx = prefix::TestsGlobalCtx::new();

    let mut ep_attr = libfabric::ep::EndpointAttr::new();
        ep_attr.ep_type(libfabric::enums::EndpointType::Msg);

    let mut dom_attr = libfabric::domain::DomainAttr::new();
        dom_attr
        .threading(libfabric::enums::Threading::Domain)
        .mr_mode(libfabric::enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw());
    
    
    let mut tx_attr = libfabric::xcontext::TxAttr::new();
        tx_attr.tclass(libfabric::enums::TClass::LowLatency);

    let hintscaps = if true {
        HintsCaps::Msg(libfabric::info::InfoHints::new()
            .ep_attr(ep_attr)
            .caps(
                libfabric::infocapsoptions::InfoCaps::new()
                .msg()
                .clone())
            .domain_attr(dom_attr)
            .tx_attr(tx_attr)
            .addr_format(libfabric::enums::AddressFormat::Unspec))
        }
        else {
            HintsCaps::Tagged(libfabric::info::InfoHints::new()
            .ep_attr(ep_attr)
            .caps(
                libfabric::infocapsoptions::InfoCaps::new()
                .tagged().clone())
            .domain_attr(dom_attr)
            .tx_attr(tx_attr)
            .addr_format(libfabric::enums::AddressFormat::Unspec))
        };

    // match hintscaps {
        // HintsCaps::Msg(hints) => {
    let (info, fab, eq, pep) = prefix::start_server(hintscaps.clone(), IP.to_owned(), "9222".to_owned());


        let (domain, tx_cq, rx_cq, tx_cntr, rx_cntr, ep, _mr, mut mr_desc) = call!(prefix::ft_server_connect,&pep, &mut gl_ctx, &eq, &fab);
        match info {
            prefix::InfoWithCaps::Msg(info) => {
                let entries = info.get();
                let test_sizes = gl_ctx.test_sizes.clone();
                let inject_size = entries[0].get_tx_attr().get_inject_size();
                for msg_size in test_sizes {
                    call!(prefix::pingpong,inject_size, &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &ep, &mut mr_desc, 100, 10, msg_size, true);
                }
                
                call!(prefix::ft_finalize,&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
                
                match ep {
                    EndpointCaps::Msg(ep) => {
                        ep.shutdown().unwrap();
                    }
                    EndpointCaps::Tagged(ep) => {
                        ep.shutdown().unwrap();
                    }
                }
            }
            prefix::InfoWithCaps::Tagged(info) => {
                let entries = info.get();
                let test_sizes = gl_ctx.test_sizes.clone();
                let inject_size = entries[0].get_tx_attr().get_inject_size();
                for msg_size in test_sizes {
                    call!(prefix::pingpong,inject_size, &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &ep, &mut mr_desc, 100, 10, msg_size, true);
                }
                
                call!(prefix::ft_finalize,&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
                
                match ep {
                    EndpointCaps::Msg(ep) => {
                        ep.shutdown().unwrap();
                    }
                    EndpointCaps::Tagged(ep) => {
                        ep.shutdown().unwrap();
                    }
                }
            }
        }
    // common::close_all_pep(fab, domain, eq, rx_cq, tx_cq, ep, pep, mr);
});


define_test!(pp_client_msg, async_pp_client_msg, {
    let mut gl_ctx = prefix::TestsGlobalCtx::new();
    let mut ep_attr = libfabric::ep::EndpointAttr::new();
        ep_attr    .ep_type(libfabric::enums::EndpointType::Msg);

    let mut dom_attr = libfabric::domain::DomainAttr::new();
        dom_attr
        .threading(libfabric::enums::Threading::Domain)
        .mr_mode(libfabric::enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw());

    let mut tx_attr = libfabric::xcontext::TxAttr::new();
        tx_attr.tclass(libfabric::enums::TClass::LowLatency);

    let hintscaps = if true {
        HintsCaps::Msg(libfabric::info::InfoHints::new()
            .ep_attr(ep_attr)
            .caps(
                libfabric::infocapsoptions::InfoCaps::new()
                .msg().clone())
            .domain_attr(dom_attr)
            .tx_attr(tx_attr)
            .addr_format(libfabric::enums::AddressFormat::Unspec))
        }
        else {
            HintsCaps::Tagged(libfabric::info::InfoHints::new()
            .ep_attr(ep_attr)
            .caps(
                libfabric::infocapsoptions::InfoCaps::new()
                .tagged().clone())
            .domain_attr(dom_attr)
            .tx_attr(tx_attr)
            .addr_format(libfabric::enums::AddressFormat::Unspec))
        };


    // match hintscaps {
        // HintsCaps::Msg(hints) => {

    let (info, _fab, domain, _eq, rx_cq, tx_cq, tx_cntr, rx_cntr, ep, _mr, mut mr_desc) = 
        call!(prefix::ft_client_connect,hintscaps, &mut gl_ctx, IP.to_owned(), "9222".to_owned());
        
    match info {
        prefix::InfoWithCaps::Msg(info) => {
            let entries = info.get();
            let test_sizes = gl_ctx.test_sizes.clone();
            let inject_size = entries[0].get_tx_attr().get_inject_size();
            for msg_size in test_sizes {
                call!(prefix::pingpong,inject_size, &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &ep, &mut mr_desc, 100, 10, msg_size, false);
            }
            
            call!(prefix::ft_finalize,&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
            match ep {
                EndpointCaps::Msg(ep) => {
                    ep.shutdown().unwrap();
                }
                EndpointCaps::Tagged(ep) => {
                    ep.shutdown().unwrap();
                }
            }
        }
        prefix::InfoWithCaps::Tagged(info) => {
            let entries = info.get();
            let test_sizes = gl_ctx.test_sizes.clone();
            let inject_size = entries[0].get_tx_attr().get_inject_size();
            for msg_size in test_sizes {
                call!(prefix::pingpong,inject_size, &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &ep, &mut mr_desc, 100, 10, msg_size, false);
            }
            
            call!(prefix::ft_finalize,&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
            match ep {
                EndpointCaps::Msg(ep) => {
                    ep.shutdown().unwrap();
                }
                EndpointCaps::Tagged(ep) => {
                    ep.shutdown().unwrap();
                }
            }
        }
    }
});