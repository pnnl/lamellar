pub mod sync_; // Public to supress lint warnings (unused function)
#[cfg(any(feature="use-async-std", feature="use-tokio"))]
pub mod async_; // Public to supress lint warnings (unused function)

pub mod common; // Public to supress lint warnings (unused function)

// Public to supress lint warnings (unused function)
use common::IP;

use libfabric::info::{Info, Version};
use prefix::{ft_finalize, HintsCaps, define_test, call};

use sync_ as prefix;


// To run the following tests do:
// 1. export FI_LOG_LEVEL="info" . 
// 2. Run the server (e.g. cargo test pp_server_msg -- --ignored --nocapture) 
//    There will be a large number of info printed. What we need is the last line with: listening on: fi_sockaddr_in:// <ip:port>
// 3. Copy the ip, port of the previous step
// 4. On the client (e.g. pp_client_msg) change  ft_client_connect node(<ip>) and service(<port>) to service and port of the copied ones
// 5. Run client (e.g. cargo test pp_client_msg -- --ignored --nocapture) 


define_test!(pp_server_rdm_cntr, async_pp_server_rdm_cntr, {
    let mut gl_ctx = prefix::TestsGlobalCtx::new();
    gl_ctx.options = prefix::FT_OPT_RX_CNTR | prefix::FT_OPT_TX_CNTR;
    // todo!();
    // gl_ctx.comp_method = CompMeth::Sread;

    let info = Info::new(&Version{major: 1, minor: 19})
    .enter_hints()
        .enter_ep_attr()
            .type_(libfabric::enums::EndpointType::Rdm)
        .leave_ep_attr()
        .enter_domain_attr()
            .threading(libfabric::enums::Threading::Domain)
            .mr_mode(libfabric::enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw())
        .leave_domain_attr()
        .enter_tx_attr()
            .traffic_class(libfabric::enums::TrafficClass::LowLatency)
        .leave_tx_attr()
        .addr_format(libfabric::enums::AddressFormat::Unspec);

    let hintscaps = if true {
        HintsCaps::Msg(info.caps(libfabric::infocapsoptions::InfoCaps::new().msg()))
    }
    else {
        HintsCaps::Tagged(info.caps(libfabric::infocapsoptions::InfoCaps::new().tagged()))
    };


    let (infocap, ep, _domain, cq_type, tx_cntr, rx_cntr, _mr, _av, mut mr_desc) = 
        prefix::ft_init_fabric(hintscaps, &mut gl_ctx, "".to_owned(), "9222".to_owned(), true);
    
    match infocap {
        prefix::InfoWithCaps::Msg(entry) => {
            
            let test_sizes = gl_ctx.test_sizes.clone();
            let inject_size = entry.tx_attr().inject_size();
            for msg_size in test_sizes {
                prefix::pingpong(inject_size, &mut gl_ctx, &cq_type, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, 100, 10, msg_size, true);
            }
            
            ft_finalize(&entry, &mut gl_ctx, &ep, &cq_type, &tx_cntr, &rx_cntr, &mut mr_desc);
        }
        prefix::InfoWithCaps::Tagged(entry) => {
            
            let test_sizes = gl_ctx.test_sizes.clone();
            let inject_size = entry.tx_attr().inject_size();
            for msg_size in test_sizes {
                prefix::pingpong(inject_size, &mut gl_ctx, &cq_type, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, 100, 10, msg_size, true);
            }
            
            ft_finalize(&entry, &mut gl_ctx, &ep, &cq_type, &tx_cntr, &rx_cntr, &mut mr_desc);
        }
    }
});
    


define_test!(pp_client_rdm_cntr, async_pp_client_rdm_cntr, {
    let mut gl_ctx = prefix::TestsGlobalCtx::new();
    gl_ctx.options = prefix::FT_OPT_RX_CNTR | prefix::FT_OPT_TX_CNTR;
    // todo!();
    // gl_ctx.comp_method = CompMeth::Sread;

    let info = Info::new(&Version{major: 1, minor: 19})
    .enter_hints()
        .enter_ep_attr()
            .type_(libfabric::enums::EndpointType::Rdm)
        .leave_ep_attr()
        .enter_domain_attr()
            .threading(libfabric::enums::Threading::Domain)
            .mr_mode(libfabric::enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw())
        .leave_domain_attr()
        .enter_tx_attr()
            .traffic_class(libfabric::enums::TrafficClass::LowLatency)
        .leave_tx_attr()
        .addr_format(libfabric::enums::AddressFormat::Unspec);

    let hintscaps = if true {
        HintsCaps::Msg(info.caps(libfabric::infocapsoptions::InfoCaps::new().msg()))
    }
    else {
        HintsCaps::Tagged(info.caps(libfabric::infocapsoptions::InfoCaps::new().tagged()))
    };


    let (infocap, ep, _domain, cq_type, tx_cntr, rx_cntr, _mr, _av, mut mr_desc) = 
        prefix::ft_init_fabric(hintscaps, &mut gl_ctx, IP.to_owned(), "9222".to_owned(), false);

    match infocap {
        prefix::InfoWithCaps::Msg(entry) => {
            let test_sizes = gl_ctx.test_sizes.clone();
            let inject_size = entry.tx_attr().inject_size();
            for msg_size in test_sizes {
                prefix::pingpong(inject_size, &mut gl_ctx, &cq_type, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, 100, 10, msg_size, false);
            }
            
            ft_finalize(&entry, &mut gl_ctx, &ep, &cq_type, &tx_cntr, &rx_cntr, &mut mr_desc);
        }
        prefix::InfoWithCaps::Tagged(entry) => {
            let test_sizes = gl_ctx.test_sizes.clone();
            let inject_size = entry.tx_attr().inject_size();
            for msg_size in test_sizes {
                prefix::pingpong(inject_size, &mut gl_ctx, &cq_type, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, 100, 10, msg_size, false);
            }
            
            ft_finalize(&entry, &mut gl_ctx, &ep, &cq_type, &tx_cntr, &rx_cntr, &mut mr_desc);
        }
    }
});