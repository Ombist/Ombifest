# Hosting the signed manifest

The manifest is a **static JSON** file. Prefer **high availability** decoupled from the live WebSocket relay:

- **S3 + CloudFront** (or equivalent) with HTTPS and short TTL / versioned object keys.
- **GitHub Pages** or internal static site for low rate of change.
- **Dedicated Nginx** (see [nginx-static.conf](nginx-static.conf) and [Dockerfile](Dockerfile)) on a small VM **only** if you accept that host’s uptime as the pin distribution SPOF.

**Do not** require the manifest URL to use the same **relay leaf pin** path as `wss://` — iOS fetches the manifest with **system trust** only ([`PinManifestService`](../../Ombist_IOS/Ombist_IOS/Services/PinManifestService.swift)). Use a **publicly trusted** server certificate or an **MDM-trusted** internal CA for the manifest host.

## Quick try (Docker)

From `Ombifest/examples/`:

```bash
# Optionally overwrite relay-pin-manifest.example.json with a real signed manifest, then:
docker build -t ombifest-static .
docker run --rm -p 8080:80 ombifest-static
# curl -sS http://127.0.0.1:8080/relay-pin-manifest.json
```

For production, terminate **TLS** in front (e.g. Caddy/Nginx on host or CDN).

## Security checklist

Before relying on a hosted manifest in production, walk through [docs/supply-chain-security.md](../docs/supply-chain-security.md) (key custody, `verify` + leaf pin checks, dual-pin rotation, monitoring). Pin rotation calendar: [docs/ios-pin-rotation-calendar.md](../../docs/ios-pin-rotation-calendar.md).
