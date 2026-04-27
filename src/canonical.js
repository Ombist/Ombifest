/**
 * Canonical manifest payload bytes — must match Ombist iOS
 * PinManifestService.canonicalPayloadJSON (JSONSerialization sortedKeys + sorted pins).
 * @param {number} version
 * @param {string} validUntil ISO8601
 * @param {string[]} pinsSorted lexicographically sorted lowercase hex pins
 * @returns {Buffer}
 */
export function canonicalPayloadBytes(version, validUntil, pinsSorted) {
  const obj = { pins: pinsSorted, validUntil, version };
  return Buffer.from(JSON.stringify(obj), "utf8");
}

/** @param {string} pinsArg comma-separated hex pins */
export function normalizePinsFromCommaSeparated(pinsArg) {
  return pinsArg
    .split(",")
    .map((s) => s.trim().toLowerCase())
    .filter(Boolean)
    .sort();
}

/** @param {string[]} pins unsorted */
export function normalizePinsArray(pins) {
  return pins.map((s) => String(s).trim().toLowerCase()).filter(Boolean).sort();
}
