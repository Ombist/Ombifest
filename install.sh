#!/usr/bin/env bash
# Build the Rust `ombifest` CLI (cargo + openssl). No Node.js required.
# Usage: bash install.sh [--skip-toolchain-check] [--suggest-rustup] [--link-user-bin] [--patch-profile]
#
# --skip-toolchain-check  Skip `cargo --version` check (e.g. CI after rust-toolchain action)
# --suggest-rustup        When cargo is missing, print rustup install hints
# --link-user-bin         Copy or symlink release ombifest to ~/.local/bin/ombifest
# --patch-profile         Append PATH hint to ~/.profile if missing (use with care on shared accounts)

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE="$ROOT/rust/ombifest-cli"

SKIP_TC=0
SUGGEST_RUSTUP=0
LINK_USER_BIN=0
PATCH_PROFILE=0

for arg in "$@"; do
  case "$arg" in
    --skip-toolchain-check) SKIP_TC=1 ;;
    --skip-node-check) SKIP_TC=1 ;; # deprecated alias
    --suggest-rustup) SUGGEST_RUSTUP=1 ;;
    --suggest-apt) SUGGEST_RUSTUP=1 ;; # deprecated alias
    --link-user-bin) LINK_USER_BIN=1 ;;
    --patch-profile) PATCH_PROFILE=1 ;;
    -h|--help)
      grep '^#' "$0" | head -n 14 | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "ombifest/install.sh: unknown option: $arg" >&2
      exit 1
      ;;
  esac
done

die() { echo "ombifest/install.sh: $*" >&2; exit 1; }

print_rustup_hints() {
  echo ""
  echo "Install Rust (example — run yourself):"
  echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  echo "  # then: source \"\$HOME/.cargo/env\""
  echo ""
}

if [[ "$SKIP_TC" -eq 0 ]]; then
  if ! command -v cargo >/dev/null 2>&1; then
    echo "ombifest/install.sh: cargo not found on PATH." >&2
    [[ "$SUGGEST_RUSTUP" -eq 1 ]] && print_rustup_hints
    die "Install Rust or re-run with --skip-toolchain-check if an orchestrator already provided cargo."
  fi
fi

if ! command -v openssl >/dev/null 2>&1; then
  die "openssl not found on PATH (required for leaf-pin / build-relay). On Debian/Ubuntu: sudo apt-get install -y openssl"
fi

echo "ombifest/install.sh: building release binary in $CRATE"
(cd "$CRATE" && cargo build --release)

release="$CRATE/target/release/ombifest"
[[ -x "$release" ]] || die "expected binary not found: $release"

if [[ "$LINK_USER_BIN" -eq 1 ]]; then
  bin_dir="${HOME}/.local/bin"
  mkdir -p "$bin_dir"
  target="$bin_dir/ombifest"
  if ln -sf "$release" "$target" 2>/dev/null; then
    echo "ombifest/install.sh: symlinked $release -> $target"
  else
    cp -f "$release" "$target"
    chmod +x "$target"
    echo "ombifest/install.sh: copied $release -> $target"
  fi
  echo "ombifest/install.sh: ensure $bin_dir is on your PATH (e.g. export PATH=\"\$HOME/.local/bin:\$PATH\")"
fi

if [[ "$PATCH_PROFILE" -eq 1 ]]; then
  line='export PATH="$HOME/.local/bin:$PATH"'
  prof="${HOME}/.profile"
  if [[ -f "$prof" ]] && grep -Fq '.local/bin' "$prof"; then
    echo "ombifest/install.sh: $prof already mentions .local/bin; not modifying."
  else
    echo "" >>"$prof"
    echo "# Added by Ombifest install.sh (--patch-profile)" >>"$prof"
    echo "$line" >>"$prof"
    echo "ombifest/install.sh: appended PATH line to $prof (review in a new login shell)."
  fi
fi

echo "ombifest/install.sh: done. Binary: $release"
echo "ombifest/install.sh: optional: export OMBIFEST_CLI=$release for scripts that auto-detect the monorepo build."
