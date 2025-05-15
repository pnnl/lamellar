#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
pub mod async_; // Public to supress lint warnings (unused function)
pub mod common;
pub mod sync_; // Public to supress lint warnings (unused function) // Public to supress lint warnings (unused function)

#[cfg(any(feature = "use-async-std", feature = "use-tokio"))]
use async_ as prefix;
use libfabric::info::Info;
use prefix::{call, define_test, HintsCaps};
#[cfg(not(any(feature = "use-async-std", feature = "use-tokio")))]
use sync_ as prefix;

// To run the following tests do:
// 1. export FI_LOG_LEVEL="info" .
// 2. Run the server (e.g. cargo test pp_server_msg -- --ignored --nocapture)
//    There will be a large number of info printed. What we need is the last line with: listening on: fi_sockaddr_in:// <ip:port>
// 3. Copy the ip, port of the previous step
// 4. On the client (e.g. pp_client_msg) change  ft_client_connect node(<ip>) and service(<port>) to service and port of the copied ones
// 5. Run client (e.g. cargo test pp_client_msg -- --ignored --nocapture)

define_test!(pp_server_rdm_tagged, asyn_pp_server_rdm_tagged, {
    let mut gl_ctx = prefix::TestsGlobalCtx::new();

    let info = Info::new(&libfabric::info::libfabric_version())
        .enter_hints()
        .enter_ep_attr()
        .type_(libfabric::enums::EndpointType::Rdm)
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
        HintsCaps::Msg(info.caps(libfabric::infocapsoptions::InfoCaps::new().msg()))
    } else {
        HintsCaps::Tagged(info.caps(libfabric::infocapsoptions::InfoCaps::new().tagged()))
    };

    let (infocap, ep, _domain, cq_type, tx_cntr, rx_cntr, mr, _av) = call!(
        prefix::ft_init_fabric,
        hintscaps,
        &mut gl_ctx,
        "".to_owned(),
        "9222".to_owned(),
        true
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
                    &mr,
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
                &mr
            );
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
                    &mr,
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
                &mr
            );
        }
    }
});

define_test!(pp_client_rdm_tagged, async_pp_client_rdm_tagged, {
    let hostname = std::process::Command::new("hostname")
        .output()
        .expect("Failed to execute hostname")
        .stdout;
    let hostname = String::from_utf8(hostname[2..].to_vec()).unwrap();
    let ip = "172.17.110.".to_string() + &hostname;
    let mut gl_ctx = prefix::TestsGlobalCtx::new();

    let info = Info::new(&libfabric::info::libfabric_version())
        .enter_hints()
        .enter_ep_attr()
        .type_(libfabric::enums::EndpointType::Rdm)
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
        HintsCaps::Msg(info.caps(libfabric::infocapsoptions::InfoCaps::new().msg()))
    } else {
        HintsCaps::Tagged(info.caps(libfabric::infocapsoptions::InfoCaps::new().tagged()))
    };

    let (infocap, ep, _domain, cq_type, tx_cntr, rx_cntr, mr, _av) = call!(
        prefix::ft_init_fabric,
        hintscaps,
        &mut gl_ctx,
        ip.strip_suffix("\n").unwrap_or(&ip).to_owned(),
        "9222".to_owned(),
        false
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
                    &mr,
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
                &mr
            );
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
                    &mr,
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
                &mr
            );
        }
    }
});
