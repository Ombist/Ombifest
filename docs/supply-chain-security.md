# Pin manifest supply chain security

Operational checklist for teams using [Ombifest](../README.md) and Ombist iOS `PinManifestService`. Cryptographic details remain in [SPEC.md](../SPEC.md) and [ADR-002](../../docs/adr/ADR-002-tls-public-key-pinning.md). First-time setup: run [`install.sh`](../install.sh) from `Ombifest/` and use [`env.manifest-operator.example`](../env.manifest-operator.example) as a template for renew-hook env (no secrets in git).

## Key custody

1. **Private signing key (PEM)**  
   - Never commit to git.  
   - Store in a secret manager or hardware-backed system where only the signing role can read.  
   - Prefer generating with `ombifest generate-key --out-private <path>` and moving the file to secure storage immediately.

2. **Separation of duties**  
   - Principals that can **read the private key** should differ from principals that can **overwrite** the hosted `manifest.json` (e.g. separate IAM / OIDC roles).

## Before every publish

1. Run **`ombifest verify`** against the exact JSON you will upload, with the same **`OMBIST_PIN_MANIFEST_PUBLIC_KEY_HEX`** as in the shipping app build.  
2. Confirm each pin matches the **ingress leaf** you intend to serve (e.g. `ombifest leaf-pin --cert` on the PEM that terminates TLS for clients).  
3. For rotations, follow [docs/ios-pin-rotation-calendar.md](../../docs/ios-pin-rotation-calendar.md) and use dual-pin overlap (`build-relay --next-pin`).

## Hosting

- Serve the manifest over **HTTPS** with a certificate **trusted by devices** (public CA or MDM-trusted internal CA).  
- Prefer **high-availability static hosting** (object storage + CDN, static Pages, etc.) decoupled from the live `wss` relay — see [examples/README.md](../examples/README.md).  
- Use **immutable or versioned object keys** where possible; restrict who can overwrite production paths.

## Monitoring

- Track **manifest fetch failure rate** and **signature / parse errors** in client or edge logs.  
- Alert before **`validUntil`** crosses your operational renewal window.

## iOS configuration

- Align **`OMBIST_PIN_MANIFEST_URL`** and **`OMBIST_PIN_MANIFEST_PUBLIC_KEY_HEX`** with your CI / release process so production builds only trust the intended publisher.  
- See [Ombers_Communicator/docs/ios-tls-pins-for-wss-ingress.md](../../Ombers_Communicator/docs/ios-tls-pins-for-wss-ingress.md) for ingress and plist notes.

## Automation (TLS renew → manifest)

After ACME / Certbot renews ingress TLS, you can optionally regenerate and verify the signed manifest from the new leaf PEM. See monorepo [docs/operations/tls-manifest-automation.md](../../docs/operations/tls-manifest-automation.md) and [docs/tools/post-leaf-cert-manifest-renew.sh](../../docs/tools/post-leaf-cert-manifest-renew.sh). Do not store signing private keys on shared CI without a vault policy.
