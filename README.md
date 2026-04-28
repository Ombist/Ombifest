# Ombifest

Authoring and verification tooling for **TLS pin manifests** used by Ombist / Ombers. The canonical implementation is the **Rust** CLI in [`rust/ombifest-cli`](rust/ombifest-cli).

## Requirements

- **Rust** (stable) and **Cargo**
- **OpenSSL** CLI (`openssl`) for leaf cert → SPKI hex in operational scripts

## Build and test

From the monorepo root:

```bash
cd Ombifest/rust/ombifest-cli
cargo build --release
cargo test
```

The release binary is `Ombifest/rust/ombifest-cli/target/release/ombifest`. Shell helpers under `docs/tools/` resolve it via [`rust/scripts/ombifest-path.sh`](rust/scripts/ombifest-path.sh) (override with `OMBIFEST_CLI`).

## Install (optional)

```bash
cd Ombifest
./install.sh              # build release binary
./install.sh --link-user-bin   # symlink into ~/.local/bin
```

## Specification

See [`SPEC.md`](SPEC.md) for the manifest JSON shape, signing rules, and CLI invariants.

## Operational scripts

- [`../docs/tools/build-relay-pin-manifest.sh`](../docs/tools/build-relay-pin-manifest.sh) — `ombifest build-relay`
- [`../docs/tools/post-leaf-cert-manifest-renew.sh`](../docs/tools/post-leaf-cert-manifest-renew.sh) — leaf SPKI + relay manifest renew
- [`../docs/tools/sign-pin-manifest.sh`](../docs/tools/sign-pin-manifest.sh) — thin wrapper for `generate-key` / `sign`
