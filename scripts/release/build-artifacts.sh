#!/usr/bin/env bash
# Builds the files a release publishes, plus SHA256SUMS, into <out-dir> (default dist/).
# release.yml runs it in job "build" through `make release-artifacts`. It runs the same locally.
#
#   axiom      axiom-<version>.wasm              (projects/axiom/scripts/build-wasm.sh)
#   tictactoe  tictactoe-web-<version>.tar.gz    (projects/tictactoe/web/build.sh, the www/ folder)
#
# Usage: scripts/release/build-artifacts.sh <project> <version> [out-dir]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly ROOT
cd "$ROOT"
# shellcheck source=projects.sh
source scripts/release/projects.sh

fail() {
  echo "error: $*" >&2
  exit 1
}

[[ $# -ge 2 && $# -le 3 ]] || fail "usage: build-artifacts.sh <project> <version> [out-dir]"
readonly PROJECT="$1" VERSION="$2" OUT="${3:-dist}"
release_is_project "$PROJECT" || fail "unknown project '$PROJECT'"
[[ "$(release_read_version "$PROJECT")" == "$VERSION" ]] \
  || fail "$PROJECT is at $(release_read_version "$PROJECT"), not $VERSION"

rm -rf "$OUT"
mkdir -p "$OUT"

case "$PROJECT" in
  axiom)
    (cd projects/axiom && bash scripts/build-wasm.sh)
    cp projects/axiom/web/axiom.wasm "$OUT/axiom-${VERSION}.wasm"
    ;;
  tictactoe)
    bash projects/tictactoe/web/build.sh
    tar -czf "$OUT/tictactoe-web-${VERSION}.tar.gz" -C projects/tictactoe/web www
    ;;
  *)
    fail "no artifact recipe for $PROJECT. Add one here."
    ;;
esac

(cd "$OUT" && sha256sum -- * > SHA256SUMS && cat SHA256SUMS)
