use common::{ft_sync, ft_tx, NO_CQ_DATA, ft_rx, ft_finalize};
use libfabric::{cq, enums, Msg, domain, ep, mr};


pub mod common; // Public to supress lint warnings (unused function)

// To run the following tests do:
// 1. export FI_LOG_LEVEL="info" . 
// 2. Run the server (e.g. cargo test pp_server_msg -- --ignored --nocapture) 
//    There will be a large number of info printed. What we need is the last line with: listening on: fi_sockaddr_in:// <ip:port>
// 3. Copy the ip, port of the previous step
// 4. On the client (e.g. pp_client_msg) change  ft_client_connect node(<ip>) and service(<port>) to service and port of the copied ones
// 5. Run client (e.g. cargo test pp_client_msg -- --ignored --nocapture) 

#[ignore]
#[test]
fn pp_server_msg() {
    let mut gl_ctx = common::TestsGlobalCtx::new();

    let ep_attr = ep::EndpointAttr::new()
        .ep_type(enums::EndpointType::MSG);

    let dom_attr = domain::DomainAttr::new()
        .threading(enums::Threading::DOMAIN)
        .mr_mode(enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw());
    
    let caps = libfabric::InfoCaps::new()
        .msg();
    

    let tx_attr = libfabric::TxAttr::new()
        .tclass(enums::TClass::LOW_LATENCY);

    let hints = libfabric::InfoHints::new()
        .ep_attr(ep_attr)
        .caps(caps)
        .domain_attr(dom_attr)
        .tx_attr(tx_attr)
        .addr_format(enums::AddressFormat::UNSPEC);


    let (info, fab, domain, eq, pep) = common::start_server(hints);
    let (tx_cq, rx_cq, tx_cntr, rx_cntr, ep, mr, mut mr_desc) = common::ft_server_connect(&mut gl_ctx, &eq, &domain);
    let entries = info.get();
    let test_sizes = gl_ctx.test_sizes.clone();
    for msg_size in test_sizes {
        common::pingpong(&entries[0], &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &ep, &mut mr_desc, 100, 10, msg_size, true);
    }

    ft_finalize(&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
    
    ep.shutdown(0);

    common::close_all_pep(fab, domain, eq, rx_cq, tx_cq, ep, pep, mr);
}



#[ignore]
#[test]
fn pp_client_msg() {
    let mut gl_ctx = common::TestsGlobalCtx::new();
    let ep_attr = ep::EndpointAttr::new()
        .ep_type(enums::EndpointType::MSG);

    let dom_attr = domain::DomainAttr::new()
        .threading(enums::Threading::DOMAIN)
        .mr_mode(enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw());

    let tx_attr = libfabric::TxAttr::new()
        .tclass(enums::TClass::LOW_LATENCY);

    let caps = libfabric::InfoCaps::new()
        .msg();

    let hints = libfabric::InfoHints::new()
        .ep_attr(ep_attr)
        .domain_attr(dom_attr)
        .tx_attr(tx_attr)
        .caps(caps)
        .addr_format(enums::AddressFormat::UNSPEC);

    let (info, fab, domain, eq, rx_cq, tx_cq, tx_cntr, rx_cntr, ep, mr, mut mr_desc) = 
        common::ft_client_connect(hints, &mut gl_ctx, "172.17.110.21".to_owned(), "9222".to_owned());
    let entries = info.get();
    let test_sizes = gl_ctx.test_sizes.clone();
    for msg_size in test_sizes {
        common::pingpong(&entries[0], &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &ep, &mut mr_desc, 100, 10, msg_size, false);
    }

    ft_finalize(&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
    ep.shutdown(0);

    common::close_all(fab, domain, eq, rx_cq, tx_cq, tx_cntr, rx_cntr, ep, mr, None);

}