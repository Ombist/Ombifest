import assert from "node:assert/strict";
import test from "node:test";
import { canonicalPayloadBytes, normalizePinsFromCommaSeparated } from "../src/canonical.js";

test("canonical JSON matches Swift PinManifestService key order and sorted pins", () => {
  const pins = ["aa11", "bb22"].sort();
  const buf = canonicalPayloadBytes(1, "2099-06-01T00:00:00Z", pins);
  assert.equal(
    buf.toString("utf8"),
    '{"pins":["aa11","bb22"],"validUntil":"2099-06-01T00:00:00Z","version":1}',
  );
});

test("normalizePinsFromCommaSeparated sorts and lowercases", () => {
  assert.deepEqual(normalizePinsFromCommaSeparated("BB22, aa11 "), ["aa11", "bb22"]);
});
