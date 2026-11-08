#!/usr/bin/env bash
# Builds the browser WASM module and copies it to web/axiom.wasm.
#
# Two build paths:
#   1. Threads (preferred): nightly + rust-src + the wasm32 target. Builds std
#      with atomics, bulk memory, simd128 and shared memory, following the
#      recipe in .cargo/config.toml.
#   2. Fallback: stable + the wasm32 target, no atomics. Threads are disabled.
#
# This script installs nothing. It only reports what is missing.
#
# Usage: scripts/build-wasm.sh
set -euo pipefail

readonly TARGET="wasm32-unknown-unknown"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly ROOT
readonly ARTIFACT="$ROOT/target/$TARGET/release/axiom.wasm"
readonly OUT="$ROOT/web/axiom.wasm"

die() {
    echo "error: $*" >&2
    exit 1
}

warn() {
    echo "warning: $*" >&2
}

command -v rustup >/dev/null 2>&1 \
    || die "rustup not found. Install it from https://rustup.rs and retry."

# Output of a rustup query, or empty if the query fails (e.g. no nightly).
# Captured into a variable so `grep -q` never closes a pipe under pipefail.
toolchains="$(rustup toolchain list 2>/dev/null || true)"
stable_targets="$(rustup target list --installed 2>/dev/null || true)"
nightly_targets=""
nightly_components=""
if grep -q '^nightly' <<<"$toolchains"; then
    nightly_targets="$(rustup +nightly target list --installed 2>/dev/null || true)"
    nightly_components="$(rustup +nightly component list --installed 2>/dev/null || true)"
fi

nightly_ready=false
if grep -q '^nightly' <<<"$toolchains" \
    && grep -q '^rust-src' <<<"$nightly_components" \
    && grep -qx "$TARGET" <<<"$nightly_targets"; then
    nightly_ready=true
fi

stable_ready=false
if grep -qx "$TARGET" <<<"$stable_targets"; then
    stable_ready=true
fi

if [[ "$nightly_ready" != true && "$stable_ready" != true ]]; then
    echo "error: the $TARGET target is not installed." >&2
    echo "" >&2
    echo "Install it with:" >&2
    echo "    rustup target add $TARGET" >&2
    echo "" >&2
    echo "For the threaded build, also install nightly and rust-src:" >&2
    echo "    rustup toolchain install nightly" >&2
    echo "    rustup component add rust-src --toolchain nightly" >&2
    echo "    rustup +nightly target add $TARGET" >&2
    exit 1
fi

# Cargo reads .cargo/config.toml from the current directory, so run from the
# project root.
cd "$ROOT"

if [[ "$nightly_ready" == true ]]; then
    echo "building $TARGET with nightly + build-std (threads enabled)"
    RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+simd128 -C link-arg=--shared-memory -C link-arg=--max-memory=4294967296" \
        cargo "+nightly" build --release --target "$TARGET" \
        -Z build-std=std,panic_abort
else
    warn "nightly with rust-src and the $TARGET target is not available."
    warn "building with stable: WASM THREADS ARE DISABLED (no atomics, no shared memory)."
    warn "to enable threads later: rustup toolchain install nightly; rustup component add rust-src --toolchain nightly; rustup +nightly target add $TARGET"
    echo "building $TARGET with stable (threads disabled)"
    cargo build --release --target "$TARGET"
fi

[[ -f "$ARTIFACT" ]] || die "build finished but $ARTIFACT is missing"

mkdir -p "$ROOT/web"
cp "$ARTIFACT" "$OUT"
echo "wrote $OUT ($(wc -c <"$OUT") bytes)"
