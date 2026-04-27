# Ombifest

**Ombist** monorepo tooling for **TLS leaf pin** fingerprints and **signed pin manifests** consumed by **Ombist iOS** [`PinManifestService`](../Ombist_IOS/Ombist_IOS/Services/PinManifestService.swift).

This package is the **authoritative CLI** for signing and verifying manifests. The legacy script [`docs/tools/sign-pin-manifest.mjs`](../docs/tools/sign-pin-manifest.mjs) is a thin wrapper that forwards to this CLI (same flags as before).

## Who does what

| Responsibility | Owner |
|------------------|--------|
| Sign / verify manifests, compute leaf pin from PEM | **Ombifest** (this folder) |
| Trust manifest HTTPS URL (system TLS) + merge pins in app | **Ombist iOS** |
| Present relay TLS certificate (ingress) | **Nginx / Ombers host** |
| Publish static `manifest.json` (CDN, object storage, or Nginx) | **Your ops** — see [examples/README.md](examples/README.md) |

## Requirements

- **Node.js 20+**
- **`openssl`** on `PATH` (for `ombifest leaf-pin` and `build-relay`)

## Install (monorepo)

```bash
cd Ombifest
npm ci
```

Run CLI without global install:

```bash
node Ombifest/src/cli.js --help
```

Or after `npm ci`, from `Ombifest/`:

```bash
npx ombifest generate-key
```

## Commands

### `sign`

```bash
ombifest sign --pins 'abc...,def...' --valid-until 2099-12-31T23:59:59Z --version 1 --private-key key.pem
```

Writes pretty-printed JSON to stdout.

### `verify`

```bash
ombifest verify --manifest relay-pin-manifest.json --public-key-hex <64hex> [--now 2026-01-01T00:00:00Z]
```

Exits `0` if signature and `validUntil` window pass.

### `generate-key`

```bash
ombifest generate-key
```

Prints PKCS#8 PEM and `OMBIST_PIN_MANIFEST_PUBLIC_KEY_HEX`.

### `leaf-pin`

SHA-256 of **leaf certificate DER** (same as iOS pin):

```bash
ombifest leaf-pin --cert ./ingress-leaf.pem
```

### `build-relay`

Same behavior as legacy [`docs/tools/build-relay-pin-manifest.sh`](../docs/tools/build-relay-pin-manifest.sh): current pin from `--leaf-cert`, optional `--next-pin`, then sign.

```bash
ombifest build-relay \
  --leaf-cert relay.crt \
  --valid-until 2027-12-31T23:59:59Z \
  --version 3 \
  --private-key ./pin-manifest-private.pem \
  [--next-pin <64hex>] \
  > relay-pin-manifest.json
```

## Specification

See [SPEC.md](SPEC.md).

## Tests

```bash
npm test
npm run lint
```
