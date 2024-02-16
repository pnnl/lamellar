use libfabric::{enums, domain, ep};

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
fn pp_server_rdm_tagged() {
    let mut gl_ctx = common::TestsGlobalCtx::new();

    let ep_attr = ep::EndpointAttr::new()
        .ep_type(enums::EndpointType::RDM);

    let dom_attr = domain::DomainAttr::new()
        .threading(enums::Threading::DOMAIN)
        .mr_mode(enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw());
    
    let caps = libfabric::InfoCaps::new()
        .tagged();
    

    let tx_attr = libfabric::TxAttr::new()
        .tclass(enums::TClass::LOW_LATENCY);

    let hints = libfabric::InfoHints::new()
        .mode(libfabric::enums::Mode::new().context())
        .ep_attr(ep_attr)
        .caps(caps)
        .domain_attr(dom_attr)
        .tx_attr(tx_attr)
        .addr_format(enums::AddressFormat::UNSPEC);

    let (info, _fabric, ep, domain, tx_cq, rx_cq, tx_cntr, rx_cntr, _eq, _mr, _av, mut mr_desc) = 
        common::ft_init_fabric(hints, &mut gl_ctx, "".to_owned(), "9222".to_owned(), libfabric_sys::FI_SOURCE);
    
    let entries: Vec<libfabric::InfoEntry> = info.get();

    if entries.is_empty() {
        panic!("No entires in fi_info");
    }

    let test_sizes = gl_ctx.test_sizes.clone();
    
    for msg_size in test_sizes {
        common::pingpong(&entries[0], &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr,&ep, &mut mr_desc, 100, 10, msg_size, false);
    }

    common::ft_finalize(&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
}



#[ignore]
#[test]
fn pp_client_rdm_tagged() {
    let mut gl_ctx = common::TestsGlobalCtx::new();

    let ep_attr = ep::EndpointAttr::new()
        .ep_type(enums::EndpointType::RDM);

    let dom_attr = domain::DomainAttr::new()
        .threading(enums::Threading::DOMAIN)
        .mr_mode(enums::MrMode::new().prov_key().allocated().virt_addr().local().endpoint().raw());
    
    let caps = libfabric::InfoCaps::new()
        .tagged();
    

    let tx_attr = libfabric::TxAttr::new()
        .tclass(enums::TClass::LOW_LATENCY);

    let hints = libfabric::InfoHints::new()
        .mode(libfabric::enums::Mode::new().context())
        .ep_attr(ep_attr)
        .caps(caps)
        .domain_attr(dom_attr)
        .tx_attr(tx_attr)
        .addr_format(enums::AddressFormat::UNSPEC);

    let (info, _fabric, ep, domain, tx_cq, rx_cq, tx_cntr, rx_cntr, _eq, _mr, _av, mut mr_desc) = 
        common::ft_init_fabric(hints, &mut gl_ctx, "172.17.110.21".to_owned(), "9222".to_owned(), 0);

    let entries: Vec<libfabric::InfoEntry> = info.get();
    
    if entries.is_empty() {
        panic!("No entires in fi_info");
    }
    let test_sizes = gl_ctx.test_sizes.clone();
    
    for msg_size in test_sizes {
        common::pingpong(&entries[0], &mut gl_ctx, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &ep, &mut mr_desc, 100, 10, msg_size, false);
    }

    common::ft_finalize(&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &tx_cntr, &rx_cntr, &mut mr_desc);
}