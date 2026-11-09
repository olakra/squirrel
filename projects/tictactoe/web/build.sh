#!/usr/bin/env bash
# Builds the browser bundle. It compiles the web crate to WebAssembly, then runs
# wasm-bindgen to write the JavaScript glue into www/pkg, which www/main.js imports.
#
# Needs: the wasm32-unknown-unknown Rust target and the wasm-bindgen CLI. The CLI
# version must match the wasm-bindgen crate version in Cargo.lock. The script stops
# with a message naming anything that is missing.
set -euo pipefail

readonly TARGET="wasm32-unknown-unknown"
readonly HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly WORKSPACE="$(cd "${HERE}/.." && pwd)"
readonly OUT_DIR="${HERE}/www/pkg"
readonly WASM_FILE="${WORKSPACE}/target/${TARGET}/release/tictactoe_web.wasm"

die() {
  echo "build.sh: $*" >&2
  exit 1
}

check_target() {
  local sysroot
  sysroot="$(rustc --print sysroot)"
  if [[ ! -d "${sysroot}/lib/rustlib/${TARGET}" ]]; then
    die "the ${TARGET} target is not installed. Run: rustup target add ${TARGET}"
  fi
}

check_bindgen() {
  if ! command -v wasm-bindgen >/dev/null 2>&1; then
    die "the wasm-bindgen CLI is not installed. Run: cargo install wasm-bindgen-cli --version <version in Cargo.lock> --locked"
  fi
}

main() {
  check_target
  check_bindgen

  cargo build --manifest-path "${WORKSPACE}/Cargo.toml" -p tictactoe-web --release --target "${TARGET}"

  mkdir -p "${OUT_DIR}"
  wasm-bindgen --target web --out-dir "${OUT_DIR}" "${WASM_FILE}"
  echo "build.sh: wrote ${OUT_DIR}"
}

main "$@"
