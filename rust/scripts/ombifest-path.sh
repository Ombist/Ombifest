# shellcheck shell=bash
# Resolve the `ombifest` binary for shell scripts.
#
# Callers MUST set:
#   OMBIST_REPO_ROOT — absolute path to the Ombist monorepo root (directory containing Ombifest/).
#
# Optional override:
#   OMBIFEST_CLI — absolute path to the `ombifest` executable.
#
# Resolution order: OMBIFEST_CLI → target/release/ombifest → ombifest on PATH.
#
ombifest_bin() {
  if [[ -n "${OMBIFEST_CLI:-}" ]]; then
    if [[ -f "$OMBIFEST_CLI" && -x "$OMBIFEST_CLI" ]]; then
      printf '%s\n' "$OMBIFEST_CLI"
      return 0
    fi
    echo "ombifest-path: OMBIFEST_CLI is set but not an executable file: $OMBIFEST_CLI" >&2
    return 1
  fi
  local rel="${OMBIST_REPO_ROOT:?OMBIST_REPO_ROOT must be set}/Ombifest/rust/ombifest-cli/target/release/ombifest"
  if [[ -x "$rel" ]]; then
    printf '%s\n' "$rel"
    return 0
  fi
  if command -v ombifest >/dev/null 2>&1; then
    command -v ombifest
    return 0
  fi
  echo "ombifest-path: ombifest not found. Build the Rust CLI:" >&2
  echo "  cd \"\$OMBIST_REPO_ROOT/Ombifest/rust/ombifest-cli\" && cargo build --release" >&2
  echo "Or set OMBIFEST_CLI to the absolute path of the ombifest binary." >&2
  return 1
}
