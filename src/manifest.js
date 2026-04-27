import { sign } from "node:crypto";
import { canonicalPayloadBytes, normalizePinsArray } from "./canonical.js";
import { createPublicKeyFromRawHex, verifyCanonical } from "./crypto.js";

/**
 * Parse manifest JSON object (before signature check).
 * @param {string|Buffer} jsonUtf8
 * @returns {{ pins: string[], validUntil: string, version: number, signatureHex: string }}
 */
export function parseManifestJson(jsonUtf8) {
  const raw = Buffer.isBuffer(jsonUtf8) ? jsonUtf8.toString("utf8") : jsonUtf8;
  const obj = JSON.parse(raw);
  if (!obj || typeof obj !== "object") throw new Error("manifest must be a JSON object");
  const { pins, validUntil, version, signature } = obj;
  if (!Array.isArray(pins)) throw new Error("pins must be an array");
  if (typeof validUntil !== "string") throw new Error("validUntil must be a string");
  if (typeof version !== "number" || !Number.isInteger(version)) throw new Error("version must be an integer");
  if (typeof signature !== "string") throw new Error("signature must be a string");
  return {
    pins: pins.map((p) => String(p).trim().toLowerCase()).filter(Boolean),
    validUntil,
    version,
    signatureHex: signature.trim().toLowerCase(),
  };
}

/**
 * Swift: notAfter > referenceDate.addingTimeInterval(-300)
 * @param {string} validUntil ISO8601
 * @param {Date} referenceDate
 */
export function isValidUntilFresh(validUntil, referenceDate) {
  const notAfter = new Date(validUntil);
  if (Number.isNaN(notAfter.getTime())) return false;
  const threshold = new Date(referenceDate.getTime() - 300_000);
  return notAfter > threshold;
}

/**
 * @param {string|Buffer} manifestJson
 * @param {string} publicKeyHex64
 * @param {Date} [referenceDate=new Date()]
 * @returns {{ ok: true, pins: string[] } | { ok: false, error: string }}
 */
export function verifyManifest(manifestJson, publicKeyHex64, referenceDate = new Date()) {
  try {
    const parsed = parseManifestJson(manifestJson);
    if (!isValidUntilFresh(parsed.validUntil, referenceDate)) {
      return { ok: false, error: "validUntil expired or within 300s grace edge" };
    }
    const pinsSorted = normalizePinsArray(parsed.pins);
    const canonical = canonicalPayloadBytes(parsed.version, parsed.validUntil, pinsSorted);
    const sig = Buffer.from(parsed.signatureHex, "hex");
    if (sig.length !== 64) return { ok: false, error: "signature must be Ed25519 64 bytes (128 hex chars)" };
    const pub = createPublicKeyFromRawHex(publicKeyHex64);
    const valid = verifyCanonical(canonical, pub, sig);
    if (!valid) return { ok: false, error: "Ed25519 signature invalid" };
    if (pinsSorted.length === 0) return { ok: false, error: "no pins after normalize" };
    return { ok: true, pins: pinsSorted };
  } catch (e) {
    return { ok: false, error: e.message || String(e) };
  }
}

/**
 * @param {string[]} pinsSorted
 * @param {string} validUntil
 * @param {number} version
 * @param {import('crypto').KeyObject} privateKey
 */
export function signManifestObject(pinsSorted, validUntil, version, privateKey) {
  const canonical = canonicalPayloadBytes(version, validUntil, pinsSorted);
  const signature = sign(null, canonical, privateKey);
  return {
    pins: pinsSorted,
    validUntil,
    version,
    signature: signature.toString("hex"),
  };
}
