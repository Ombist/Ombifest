# Ombifest manifest specification (v1)

Normative verification logic lives in **Ombist iOS** [`PinManifestService`](../Ombist_IOS/Ombist_IOS/Services/PinManifestService.swift) and [`PinManifestServiceTests`](../Ombist_IOS/Ombist_IOSTests/PinManifestServiceTests.swift). Ombifest CLI **must** produce bytes and signatures that those routines accept.

## JSON shape (hosted file)

Single JSON object:

| Field | Type | Notes |
|-------|------|--------|
| `pins` | `string[]` | Leaf **DER SHA-256** as **64 lowercase hex** chars each. |
| `validUntil` | `string` | ISO8601 UTC (with or without fractional seconds). |
| `version` | `number` | Integer (monotonic policy is operational; iOS validates signature + expiry). |
| `signature` | `string` | Ed25519 over canonical payload, **128 hex chars** (64 bytes). |

## Canonical signed payload

UTF-8 JSON object with **exactly** these keys, **sorted lexicographically** (matches `JSONSerialization` with `.sortedKeys` in Swift):

1. `pins` — array of strings, **sorted lexicographically** after trimming and lowercasing.
2. `validUntil` — string as stored in the manifest.
3. `version` — integer.

No whitespace between tokens is **not** required for signing; the Rust `ombifest` CLI and Swift both serialize with default spacing rules. **Interoperability rule:** use the same serialization as [`rust/ombifest-cli/src/canonical.rs`](rust/ombifest-cli/src/canonical.rs) (`serde_json` compact object `{ pins, validUntil, version }` with keys sorted and `pins` already sorted).

## Signature

- Algorithm: **Ed25519** (PureEdDSA).
- Message: **canonical payload bytes** (UTF-8).
- Private key: PKCS#8 PEM (`-----BEGIN PRIVATE KEY-----`).
- Public key in app: **`OMBIST_PIN_MANIFEST_PUBLIC_KEY_HEX`** — **64 hex chars** = raw 32-byte Ed25519 public key (same extraction as the `ombifest` tooling from SPKI).

## Expiry (client behavior)

Client accepts manifest only if parsed `validUntil` instant is **strictly greater than** `referenceDate − 300 seconds` (see Swift `parseAndValidateManifest`). Ombifest `verify` uses the same rule.

## Operational references

- Supply chain checklist: [docs/supply-chain-security.md](docs/supply-chain-security.md)
- Dual-pin rotation: [docs/ios-pin-rotation-calendar.md](../docs/ios-pin-rotation-calendar.md)
- ADR: [docs/adr/ADR-002-tls-public-key-pinning.md](../docs/adr/ADR-002-tls-public-key-pinning.md)
- Relay ingress leaf: [Ombers_Communicator/docs/ios-tls-pins-for-wss-ingress.md](../Ombers_Communicator/docs/ios-tls-pins-for-wss-ingress.md)

## Manifest fetch TLS

The iOS app fetches the manifest with **system** `URLSession.shared` (**no** relay leaf pinning on that URL). The manifest host must present a **chain trusted by the device** (public CA or MDM-installed roots).

## Ombifest CLI (reference implementation)

The **Rust** CLI in [`rust/ombifest-cli`](rust/ombifest-cli) (`ombifest` binary) is the monorepo authoring tool; it **must** stay compatible with Swift verification above. Build with `cargo build --release` (see [`install.sh`](install.sh)).

- **Pin format:** `sign` rejects any pin that is not exactly **64 lowercase hex** characters after the same trimming/lowercasing as hosted JSON. `verify` rejects manifests whose `pins` fail this rule (before accepting a signature).
- **Manifest file size:** `verify` reads at most **1 MiB** from the `--manifest` path.
- **`generate-key`:** Prefer `--out-private <path.pem>` so the private PEM is not printed; `--print-private-to-stdout` retains the previous behavior for backwards-compatible wrappers ([`docs/tools/sign-pin-manifest.sh`](../docs/tools/sign-pin-manifest.sh)).
