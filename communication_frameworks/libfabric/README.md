# libfabric

`libfabric` builds on top of `libfabric-sys` to expose ergonomic Rust wrappers for the OFI `fabric`, `fi_info`, `domain`, `endpoint`, and `completion queue` concepts, along with helper modules that cover asynchronous contexts, AVs, counters, and messaging abstractions.

## Highlights

- Optional async integrations: `use-tokio` wires in `tokio` plus the completion-queue spin helpers, while `use-async-std` pulls in `async-std`/`async-io` so completion queues can be polled from either runtime.
- Granular threading controls and `shared` feature mirrors how `libfabric-sys`/`libfabric-src` build the underlying OFI stack (see the `threading-*` features for fine tuning per-FID threading guarantees).
- `thread-safe` is an internal feature that wires in `parking_lot` to guard shared state; it is not meant to be set manually.

## Usage

Add the crate next to `lamellar` when you need higher-level access to `libfabric` objects without dealing with raw FFI. Enable the `shared` feature if you want the downstream `libfabric-sys` build to produce shared libraries instead of the static defaults.

```toml
libfabric = { path = "../communication_frameworks/libfabric", features = ["use-tokio", "shared"] }
```

## Features

| Feature | Description |
|---------|-------------|
| `use-tokio` | Activate `tokio` plus the completion-queue spin helper (`async-cqs-spin`). |
| `use-async-std` | Use `async-std`/`async-io` for async completion queue polling. |
| `shared` | Forward to `libfabric-sys/shared` so the OFI build produces shared objects instead of static libraries. |
| `threading-*` | Control threading guarantees (endpoint, completion queue, domain, FID) when building the OFI layer. |

## License

BSD (see [`LICENSE`](LICENSE)).
