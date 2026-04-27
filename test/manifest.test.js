import assert from "node:assert/strict";
import test from "node:test";
import { generateKeyPairSync, sign } from "node:crypto";
import { canonicalPayloadBytes, normalizePinsArray } from "../src/canonical.js";
import { rawPublicKeyHexFromKeyPair } from "../src/crypto.js";
import { verifyManifest, signManifestObject, isValidUntilFresh } from "../src/manifest.js";

test("sign then verify round-trip", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const pubHex = rawPublicKeyHexFromKeyPair(publicKey);
  const pins = normalizePinsArray(["bb", "aa"]);
  const manifest = signManifestObject(pins, "2099-12-31T23:59:59Z", 2, privateKey);
  const json = JSON.stringify(manifest);
  const r = verifyManifest(json, pubHex, new Date("2020-01-01T00:00:00Z"));
  assert.equal(r.ok, true);
  if (r.ok) assert.deepEqual(r.pins, ["aa", "bb"]);
});

test("verify rejects expired validUntil (Swift semantics)", () => {
  const { publicKey, privateKey } = generateKeyPairSync("ed25519");
  const pubHex = rawPublicKeyHexFromKeyPair(publicKey);
  const pins = ["aa"].sort();
  const manifest = signManifestObject(pins, "2020-01-01T00:00:00Z", 1, privateKey);
  const json = JSON.stringify(manifest);
  const r = verifyManifest(json, pubHex, new Date("2025-01-01T00:00:00Z"));
  assert.equal(r.ok, false);
});

test("verify rejects wrong key", () => {
  const { publicKey } = generateKeyPairSync("ed25519");
  const { privateKey: otherPriv } = generateKeyPairSync("ed25519");
  const pubHex = rawPublicKeyHexFromKeyPair(publicKey);
  const pins = ["aa"].sort();
  const canonical = canonicalPayloadBytes(1, "2099-01-01T00:00:00Z", pins);
  const badSig = sign(null, canonical, otherPriv);
  const manifest = {
    pins,
    validUntil: "2099-01-01T00:00:00Z",
    version: 1,
    signature: badSig.toString("hex"),
  };
  const r = verifyManifest(JSON.stringify(manifest), pubHex, new Date("2020-01-01T00:00:00Z"));
  assert.equal(r.ok, false);
});

test("isValidUntilFresh matches Swift 300s window", () => {
  const ref = new Date("2025-06-15T12:00:00.000Z");
  assert.equal(isValidUntilFresh("2025-06-15T11:55:00.000Z", ref), false);
  assert.equal(isValidUntilFresh("2025-06-15T11:55:01.000Z", ref), true);
});
