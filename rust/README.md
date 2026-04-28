# Ombifest — Rust CLI

This directory contains **`ombifest-cli`**, the canonical Rust implementation of the [Ombifest SPEC](../SPEC.md).

## Build

```bash
cd Ombifest/rust/ombifest-cli
cargo build --release
# binary: target/release/ombifest
```

Commit **`Cargo.lock`** after the first successful `cargo build` / `cargo test` so CI and teammates resolve the same dependency versions.

## Tests

```bash
cd Ombifest/rust/ombifest-cli
cargo test
```

Integration-style checks (sign/verify round-trip, wrong key rejection, stable JSON shape) live in `src/manifest.rs` tests; extend there instead of external golden scripts.
