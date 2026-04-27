#!/usr/bin/env node
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";

import { normalizePinsFromCommaSeparated } from "./canonical.js";
import { generateKeyPairPem, loadPrivateKeyPem } from "./crypto.js";
import { verifyManifest, signManifestObject } from "./manifest.js";

function usage(code = 1) {
  console.error(`ombifest — TLS leaf pin & signed manifest (Ombist iOS compatible)

Commands:
  ombifest sign --pins '<hex,hex>' --valid-until <ISO8601> --version <int> --private-key <pem>
  ombifest verify --manifest <file.json> --public-key-hex <64hex> [--now <ISO8601>]
  ombifest generate-key
  ombifest leaf-pin --cert <leaf.pem>
  ombifest build-relay --leaf-cert <pem> --valid-until <ISO8601> --version <int> --private-key <pem> [--next-pin <64hex>]
`);
  process.exit(code);
}

function parseGlobalArgs(argv) {
  const cmd = argv[0];
  const rest = argv.slice(1);
  const flags = {};
  const positional = [];
  for (let i = 0; i < rest.length; i++) {
    const a = rest[i];
    if (a.startsWith("--")) {
      const key = a.slice(2);
      const next = rest[i + 1];
      if (next != null && !next.startsWith("--")) {
        flags[key] = next;
        i++;
      } else {
        flags[key] = true;
      }
    } else {
      positional.push(a);
    }
  }
  return { cmd, flags, positional };
}

function cmdSign(flags) {
  const pinsRaw = flags.pins;
  const validUntil = flags["valid-until"];
  const version = Number(flags.version);
  const privateKeyPath = flags["private-key"];
  if (!pinsRaw || !validUntil || flags.version === undefined || !privateKeyPath) usage();
  const pins = normalizePinsFromCommaSeparated(String(pinsRaw));
  const pem = readFileSync(privateKeyPath, "utf8");
  const privateKey = loadPrivateKeyPem(pem);
  const manifest = signManifestObject(pins, validUntil, version, privateKey);
  process.stdout.write(JSON.stringify(manifest, null, 2) + "\n");
}

function cmdVerify(flags) {
  const manifestPath = flags.manifest;
  const pubHex = flags["public-key-hex"];
  if (!manifestPath || !pubHex) usage();
  const body = readFileSync(manifestPath, "utf8");
  const ref = flags.now ? new Date(String(flags.now)) : new Date();
  const r = verifyManifest(body, String(pubHex).trim(), ref);
  if (!r.ok) {
    console.error("verify failed:", r.error);
    process.exit(1);
  }
  process.stderr.write(`ok: ${r.pins.length} pin(s)\n`);
}

function cmdGenerateKey() {
  const { pem, pubHex } = generateKeyPairPem();
  process.stdout.write("--- PRIVATE PEM (protect offline) ---\n");
  process.stdout.write(pem);
  process.stdout.write("\nOMBIST_PIN_MANIFEST_PUBLIC_KEY_HEX (64 hex chars):\n");
  process.stdout.write(pubHex + "\n");
}

function cmdLeafPin(flags) {
  const certPath = flags.cert;
  if (!certPath) usage();
  const openssl = spawnSync("openssl", ["x509", "-in", certPath, "-outform", "DER"], {
    encoding: "buffer",
    maxBuffer: 10 * 1024 * 1024,
  });
  if (openssl.error || openssl.status !== 0) {
    console.error("openssl x509 failed:", openssl.stderr?.toString() || openssl.error?.message);
    process.exit(1);
  }
  const digest = createHash("sha256").update(openssl.stdout).digest("hex");
  process.stdout.write(digest + "\n");
}

function cmdBuildRelay(flags) {
  const leafCert = flags["leaf-cert"];
  const validUntil = flags["valid-until"];
  const version = Number(flags.version);
  const privateKeyPath = flags["private-key"];
  const nextPin = flags["next-pin"] ? String(flags["next-pin"]).trim().toLowerCase() : "";
  if (!leafCert || !validUntil || flags.version === undefined || !privateKeyPath) usage();

  const cur = spawnSync(process.execPath, [fileURLToPath(import.meta.url), "leaf-pin", "--cert", leafCert], {
    encoding: "utf8",
  });
  if (cur.status !== 0) {
    console.error(cur.stderr || "leaf-pin failed");
    process.exit(1);
  }
  const currentPin = cur.stdout.trim().toLowerCase();
  if (!/^[0-9a-f]{64}$/.test(currentPin)) {
    console.error("failed to compute current leaf pin");
    process.exit(1);
  }
  let pinsCsv = currentPin;
  if (nextPin) {
    if (!/^[0-9a-f]{64}$/.test(nextPin)) {
      console.error("--next-pin must be 64 hex chars");
      process.exit(1);
    }
    pinsCsv = `${currentPin},${nextPin}`;
  }
  const pem = readFileSync(privateKeyPath, "utf8");
  const privateKey = loadPrivateKeyPem(pem);
  const pins = normalizePinsFromCommaSeparated(pinsCsv);
  const manifest = signManifestObject(pins, validUntil, version, privateKey);
  process.stdout.write(JSON.stringify(manifest, null, 2) + "\n");
}

const argv = process.argv.slice(2);
if (argv.length === 0 || argv[0] === "-h" || argv[0] === "--help") usage();

const { cmd, flags } = parseGlobalArgs(argv);

switch (cmd) {
  case "sign":
    cmdSign(flags);
    break;
  case "verify":
    cmdVerify(flags);
    break;
  case "generate-key":
    cmdGenerateKey();
    break;
  case "leaf-pin":
    cmdLeafPin(flags);
    break;
  case "build-relay":
    cmdBuildRelay(flags);
    break;
  default:
    console.error("unknown command:", cmd);
    usage();
}
