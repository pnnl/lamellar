use common::{ft_sync, ft_tx, NO_CQ_DATA, ft_rx, ft_finalize};
use libfabric::{cq, enums, Msg, domain, ep, mr};

use crate::common::ft_init_fabric;


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
fn pp_server_rdm() {
    let mut gl_ctx = common::TestsGlobalCtx::new();

    let ep_attr = ep::EndpointAttr::new()
        .ep_type(enums::EndpointType::RDM);

    let dom_attr = domain::DomainAttr::new()
        .threading(enums::Threading::DOMAIN)
        .mr_mode((enums::MrType::PROV_KEY.get_value() | enums::MrType::ALLOCATED.get_value() | enums::MrType::VIRT_ADDR.get_value()  | enums::MrType::LOCAL.get_value() | enums::MrType::ENDPOINT.get_value()| enums::MrType::RAW.get_value()) as i32 );
    
    let caps = libfabric::InfoCaps::new()
        .msg();
    

    let tx_attr = libfabric::TxAttr::new()
        .tclass(enums::TClass::LOW_LATENCY);

    let hints = libfabric::InfoHints::new()
        .mode(libfabric_sys::FI_CONTEXT) // [TODO]
        .ep_attr(ep_attr)
        .caps(caps)
        .domain_attr(dom_attr)
        .tx_attr(tx_attr)
        .addr_format(enums::AddressFormat::UNSPEC);

    let (info, fabric, ep, domain, tx_cq, rx_cq, eq, mut mr, av, mut mr_desc) = 
        common::ft_init_fabric(hints, &mut gl_ctx, "127.0.0.1".to_owned(), "".to_owned(), libfabric_sys::FI_SOURCE);
    
    let entries: Vec<libfabric::InfoEntry> = info.get();

    if entries.is_empty() {
        panic!("No entires in fi_info");
    }

    let test_sizes = gl_ctx.test_sizes.clone();
    
    for msg_size in test_sizes {
        common::pingpong(&entries[0], &mut gl_ctx, &tx_cq, &rx_cq, &ep, &mut mr_desc, 100, 10, msg_size, false);
    }

    ft_finalize(&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &mut mr_desc);
    
    common::close_all(fabric, domain, eq, rx_cq, tx_cq, ep, mr, av.into());
}



#[ignore]
#[test]
fn pp_client_rdm() {
    let mut gl_ctx = common::TestsGlobalCtx::new();

    let ep_attr = ep::EndpointAttr::new()
        .ep_type(enums::EndpointType::RDM);

    let dom_attr = domain::DomainAttr::new()
        .threading(enums::Threading::DOMAIN)
        .mr_mode((enums::MrType::PROV_KEY.get_value() | enums::MrType::ALLOCATED.get_value() | enums::MrType::VIRT_ADDR.get_value()  | enums::MrType::LOCAL.get_value() | enums::MrType::ENDPOINT.get_value()| enums::MrType::RAW.get_value()) as i32 );
    
    let caps = libfabric::InfoCaps::new()
        .msg();
    

    let tx_attr = libfabric::TxAttr::new()
        .tclass(enums::TClass::LOW_LATENCY);

    let hints = libfabric::InfoHints::new()
        .mode(libfabric_sys::FI_CONTEXT) // [TODO]
        .ep_attr(ep_attr)
        .caps(caps)
        .domain_attr(dom_attr)
        .tx_attr(tx_attr)
        .addr_format(enums::AddressFormat::UNSPEC);

    let (info, fabric, ep, domain, tx_cq, rx_cq, eq, mut mr, av, mut mr_desc) = 
        common::ft_init_fabric(hints, &mut gl_ctx, "172.17.110.6".to_owned(), "45911".to_owned(), 0);

    let entries: Vec<libfabric::InfoEntry> = info.get();
    
    if entries.is_empty() {
        panic!("No entires in fi_info");
    }
    let test_sizes = gl_ctx.test_sizes.clone();
    
    for msg_size in test_sizes {
        common::pingpong(&entries[0], &mut gl_ctx, &tx_cq, &rx_cq, &ep, &mut mr_desc, 100, 10, msg_size, false);
    }

    ft_finalize(&entries[0], &mut gl_ctx, &ep, &domain, &tx_cq, &rx_cq, &mut mr_desc);

    common::close_all(fabric, domain, eq, rx_cq, tx_cq, ep, mr, av.into());

}