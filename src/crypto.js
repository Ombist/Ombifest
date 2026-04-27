import { createPrivateKey, createPublicKey, generateKeyPairSync, verify } from "node:crypto";

/** SPKI prefix + 32-byte raw Ed25519 public key (matches sign-pin-manifest.mjs extraction). */
const ED25519_SPKI_PREFIX = Buffer.from("302a300506032b6570032100", "hex");

/**
 * @param {string} rawHex64 64 hex chars (32 bytes) as in OMBIST_PIN_MANIFEST_PUBLIC_KEY_HEX
 */
export function createPublicKeyFromRawHex(rawHex64) {
  const raw = Buffer.from(rawHex64.trim().toLowerCase(), "hex");
  if (raw.length !== 32) {
    throw new Error("public key hex must decode to 32 bytes (Ed25519 raw)");
  }
  const spki = Buffer.concat([ED25519_SPKI_PREFIX, raw]);
  return createPublicKey({ key: spki, format: "der", type: "spki" });
}

export function rawPublicKeyHexFromKeyPair(publicKey) {
  const spki = publicKey.export({ type: "spki", format: "der" });
  if (spki.length < 32) throw new Error("unexpected SPKI length");
  return Buffer.from(spki.subarray(spki.length - 32)).toString("hex");
}

/**
 * @param {Buffer} canonical
 * @param {import('crypto').KeyObject} publicKey
 * @param {Buffer} signature
 */
export function verifyCanonical(canonical, publicKey, signature) {
  return verify(null, canonical, publicKey, signature);
}

export function generateKeyPairPem() {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const pem = privateKey.export({ type: "pkcs8", format: "pem" });
  const pubHex = rawPublicKeyHexFromKeyPair(publicKey);
  return { pem, pubHex };
}

export function loadPrivateKeyPem(pemUtf8) {
  return createPrivateKey(pemUtf8);
}
