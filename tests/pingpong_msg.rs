pub mod sync_; // Public to supress lint warnings (unused function)

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
pub mod async_; // Public to supress lint warnings (unused function)
pub mod common; // Public to supress lint warnings (unused function)

use common::IP;
use libfabric::{
    ep::ActiveEndpoint,
    info::{Info, Version},
};
use prefix::{call, define_test, EndpointCaps, HintsCaps};
use sync_ as prefix;

// To run the following tests do:
// 1. export FI_LOG_LEVEL="info" .
// 2. Run the server (e.g. cargo test pp_server_msg -- --ignored --nocapture)
//    There will be a large number of info printed. What we need is the last line with: listening on: fi_sockaddr_in:// <ip:port>
// 3. Copy the ip, port of the previous step
// 4. On the client (e.g. pp_client_msg) change  ft_client_connect node(<ip>) and service(<port>) to service and port of the copied ones
// 5. Run client (e.g. cargo test pp_client_msg -- --ignored --nocapture)

define_test!(pp_server_msg, async_pp_server_msg, {
    let hostname = std::process::Command::new("hostname")
        .output()
        .expect("Failed to execute hostname")
        .stdout;
    let hostname = String::from_utf8(hostname[2..].to_vec()).unwrap();
    let ip = "172.17.110.".to_string() + &hostname;
    let info = Info::new(&Version {
        major: 1,
        minor: 19,
    })
    .enter_hints()
    .enter_ep_attr()
    .type_(libfabric::enums::EndpointType::Msg)
    .leave_ep_attr()
    .enter_domain_attr()
    .threading(libfabric::enums::Threading::Domain)
    .mr_mode(
        libfabric::enums::MrMode::new()
            .prov_key()
            .allocated()
            .virt_addr()
            .local()
            .endpoint()
            .raw(),
    )
    .leave_domain_attr()
    .enter_tx_attr()
    .traffic_class(libfabric::enums::TrafficClass::LowLatency)
    .leave_tx_attr()
    .addr_format(libfabric::enums::AddressFormat::Unspec);

    let hintscaps = if true {
        HintsCaps::Msg(info.caps(libfabric::infocapsoptions::InfoCaps::new().msg().clone()))
    } else {
        HintsCaps::Tagged(info.caps(libfabric::infocapsoptions::InfoCaps::new().tagged().clone()))
    };

    // match hintscaps {
    // HintsCaps::Msg(hints) => {
    let (infocap, fab, eq, pep) = prefix::start_server(
        hintscaps,
        ip.strip_suffix("\n").unwrap_or(&ip).to_owned(),
        "9222".to_owned(),
    );

    let mut gl_ctx = prefix::TestsGlobalCtx::new();

    let (cq_type, tx_cntr, rx_cntr, ep, _mr, mut mr_desc) =
        call!(prefix::ft_server_connect, &pep, &mut gl_ctx, &eq, &fab);
    match infocap {
        prefix::InfoWithCaps::Msg(entry) => {
            let test_sizes = gl_ctx.test_sizes.clone();
            let inject_size = entry.tx_attr().inject_size();
            for msg_size in test_sizes {
                call!(
                    prefix::pingpong,
                    inject_size,
                    &mut gl_ctx,
                    &cq_type,
                    &tx_cntr,
                    &rx_cntr,
                    &ep,
                    &mut mr_desc,
                    100,
                    10,
                    msg_size,
                    true
                );
            }

            call!(
                prefix::ft_finalize,
                &entry,
                &mut gl_ctx,
                &ep,
                &cq_type,
                &tx_cntr,
                &rx_cntr,
                &mut mr_desc
            );

            match ep {
                EndpointCaps::ConnectedMsg(ep) => {
                    ep.shutdown().unwrap();
                }
                EndpointCaps::ConnectedTagged(ep) => {
                    ep.shutdown().unwrap();
                }
                _ => {}
            }
        }
        prefix::InfoWithCaps::Tagged(entry) => {
            let test_sizes = gl_ctx.test_sizes.clone();
            let inject_size = entry.tx_attr().inject_size();
            for msg_size in test_sizes {
                call!(
                    prefix::pingpong,
                    inject_size,
                    &mut gl_ctx,
                    &cq_type,
                    &tx_cntr,
                    &rx_cntr,
                    &ep,
                    &mut mr_desc,
                    100,
                    10,
                    msg_size,
                    true
                );
            }

            call!(
                prefix::ft_finalize,
                &entry,
                &mut gl_ctx,
                &ep,
                &cq_type,
                &tx_cntr,
                &rx_cntr,
                &mut mr_desc
            );

            match ep {
                EndpointCaps::ConnectedMsg(ep) => {
                    ep.shutdown().unwrap();
                }
                EndpointCaps::ConnectedTagged(ep) => {
                    ep.shutdown().unwrap();
                }
                _ => {}
            }
        }
    }
    // common::close_all_pep(fab, domain, eq, rx_cq, tx_cq, ep, pep, mr);
});

define_test!(pp_client_msg, async_pp_client_msg, {
    let hostname = std::process::Command::new("hostname")
        .output()
        .expect("Failed to execute hostname")
        .stdout;
    let hostname = String::from_utf8(hostname[2..].to_vec()).unwrap();
    let ip = "172.17.110.".to_string() + &hostname;
    let mut gl_ctx = prefix::TestsGlobalCtx::new();

    let info = Info::new(&Version {
        major: 1,
        minor: 19,
    })
    .enter_hints()
    .enter_ep_attr()
    .type_(libfabric::enums::EndpointType::Msg)
    .leave_ep_attr()
    .enter_domain_attr()
    .threading(libfabric::enums::Threading::Domain)
    .mr_mode(
        libfabric::enums::MrMode::new()
            .prov_key()
            .allocated()
            .virt_addr()
            .local()
            .endpoint()
            .raw(),
    )
    .leave_domain_attr()
    .enter_tx_attr()
    .traffic_class(libfabric::enums::TrafficClass::LowLatency)
    .leave_tx_attr()
    .addr_format(libfabric::enums::AddressFormat::Unspec);

    let hintscaps = if true {
        HintsCaps::Msg(info.caps(libfabric::infocapsoptions::InfoCaps::new().msg().clone()))
    } else {
        HintsCaps::Tagged(info.caps(libfabric::infocapsoptions::InfoCaps::new().tagged().clone()))
    };

    // match hintscaps {
    // HintsCaps::Msg(hints) => {

    let (infocap, cq_type, tx_cntr, rx_cntr, ep, _mr, mut mr_desc) = call!(
        prefix::ft_client_connect,
        hintscaps,
        &mut gl_ctx,
        ip.strip_suffix("\n").unwrap_or(&ip).to_owned(),
        "9222".to_owned()
    );

    match infocap {
        prefix::InfoWithCaps::Msg(entry) => {
            let test_sizes = gl_ctx.test_sizes.clone();
            let inject_size = entry.tx_attr().inject_size();
            for msg_size in test_sizes {
                call!(
                    prefix::pingpong,
                    inject_size,
                    &mut gl_ctx,
                    &cq_type,
                    &tx_cntr,
                    &rx_cntr,
                    &ep,
                    &mut mr_desc,
                    100,
                    10,
                    msg_size,
                    false
                );
            }

            call!(
                prefix::ft_finalize,
                &entry,
                &mut gl_ctx,
                &ep,
                &cq_type,
                &tx_cntr,
                &rx_cntr,
                &mut mr_desc
            );
            match ep {
                EndpointCaps::ConnectedMsg(ep) => {
                    ep.shutdown().unwrap();
                }
                EndpointCaps::ConnectedTagged(ep) => {
                    ep.shutdown().unwrap();
                }
                _ => {}
            }
        }
        prefix::InfoWithCaps::Tagged(entry) => {
            let test_sizes = gl_ctx.test_sizes.clone();
            let inject_size = entry.tx_attr().inject_size();
            for msg_size in test_sizes {
                call!(
                    prefix::pingpong,
                    inject_size,
                    &mut gl_ctx,
                    &cq_type,
                    &tx_cntr,
                    &rx_cntr,
                    &ep,
                    &mut mr_desc,
                    100,
                    10,
                    msg_size,
                    false
                );
            }

            call!(
                prefix::ft_finalize,
                &entry,
                &mut gl_ctx,
                &ep,
                &cq_type,
                &tx_cntr,
                &rx_cntr,
                &mut mr_desc
            );
            match ep {
                EndpointCaps::ConnectedMsg(ep) => {
                    ep.shutdown().unwrap();
                }
                EndpointCaps::ConnectedTagged(ep) => {
                    ep.shutdown().unwrap();
                }
                _ => {}
            }
        }
    }
});
