# Archived Histo implementations

These files preserve the pre-unified Histo variants, including the historical
master-line implementations. They are intentionally not modules or Cargo
binary targets: the supported benchmark is `histo/src/main.rs` with
implementations selected through `--variants`.

Useful completion and result-recording behavior from these implementations is
carried by the active Dev modules and the unified runner. Keep changes here
limited to archival maintenance rather than reintroducing standalone binaries.
