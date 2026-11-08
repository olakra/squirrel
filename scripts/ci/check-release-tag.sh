#!/usr/bin/env bash
# Verifies that a release tag may be published. release.yml runs it in job "verify".
# `make release-check` runs it in --local mode, which tags nothing.
#
# Checks, in order:
#   1. The tag is vMAJOR.MINOR.PATCH with an optional -prerelease suffix.
#   2. The tag equals "v" + the version in projects/axiom/Cargo.toml.
#   3. projects/axiom/CHANGELOG.md has a non-empty section for that version.
#   4. (CI mode only) The tag is an annotated tag.
#   5. (CI mode only) The tag's commit is reachable from origin/main.
#   6. (CI mode only) No GitHub release exists for the tag yet, so a tag is never republished.
#
# Usage:
#   scripts/ci/check-release-tag.sh <tag>   CI mode: checks 1 to 6. Needs origin/main and the
#                                           tag fetched, and GH_TOKEN and GITHUB_REPOSITORY set.
#   scripts/ci/check-release-tag.sh --local checks 1 to 3 against the Cargo.toml version.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly ROOT
readonly CARGO_TOML="$ROOT/projects/axiom/Cargo.toml"
readonly CHANGELOG="$ROOT/projects/axiom/CHANGELOG.md"
readonly TAG_RE='^v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z.-]+)?$'

usage() {
  echo "usage: check-release-tag.sh <tag> | --local" >&2
  exit 2
}

fail() {
  echo "error: $*" >&2
  exit 1
}

# Prints the version from the [package] table of Cargo.toml.
cargo_version() {
  awk '
    /^\[/ { section = $0; next }
    section == "[package]" && /^version[ \t]*=/ {
      line = $0
      sub(/^[^"]*"/, "", line)
      sub(/".*$/, "", line)
      print line
      exit
    }
  ' "$CARGO_TOML"
}

# Fails when any release (published or draft) already uses the tag. The list is read with
# `gh api`; if the API call fails, the script fails too, so a broken check never lets a
# republish through. Drafts count, because a draft can be published later.
check_no_existing_release() {
  local tag="$1" existing
  [[ -n "${GH_TOKEN:-}" && -n "${GITHUB_REPOSITORY:-}" ]] \
    || fail "GH_TOKEN and GITHUB_REPOSITORY must be set to check for an existing release"
  existing="$(gh api --paginate "repos/${GITHUB_REPOSITORY}/releases" --jq '.[].tag_name')" \
    || fail "could not list releases for ${GITHUB_REPOSITORY}"
  if grep -qxF "$tag" <<< "$existing"; then
    fail "a release for $tag already exists. Tags are never republished. Use a new version."
  fi
  echo "release check: no release exists for $tag"
}

main() {
  local arg="${1:-}" version tag expected
  [[ $# -eq 1 ]] || usage

  version="$(cargo_version)"
  [[ -n "$version" ]] || fail "no version found in $CARGO_TOML"

  if [[ "$arg" == "--local" ]]; then
    tag="v$version"
    [[ "$tag" =~ $TAG_RE ]] || fail "Cargo.toml version '$version' is not a valid release version"
    echo "release check: Cargo.toml version is $version (tag would be $tag)"
  else
    tag="$arg"
    [[ "$tag" =~ $TAG_RE ]] || fail "tag '$tag' must match vMAJOR.MINOR.PATCH[-prerelease]"
    expected="v$version"
    [[ "$tag" == "$expected" ]] \
      || fail "tag $tag does not match projects/axiom/Cargo.toml version ($expected)"
    echo "release check: tag $tag matches Cargo.toml"
  fi

  if ! bash "$ROOT/scripts/ci/extract-changelog.sh" "$version" "$CHANGELOG" >/dev/null; then
    fail "$CHANGELOG has no non-empty section for [$version]"
  fi
  echo "release check: CHANGELOG has a section for $version"

  [[ "$arg" == "--local" ]] && return 0

  git -C "$ROOT" rev-parse -q --verify "refs/tags/$tag" >/dev/null \
    || fail "tag $tag is not present locally. Fetch it first."

  local kind commit
  kind="$(git -C "$ROOT" cat-file -t "refs/tags/$tag")"
  [[ "$kind" == "tag" ]] || fail "tag $tag is a lightweight tag. Release tags must be annotated."
  echo "release check: tag $tag is annotated"

  git -C "$ROOT" rev-parse -q --verify origin/main >/dev/null \
    || fail "origin/main is not present. Fetch it before this check."
  commit="$(git -C "$ROOT" rev-list -n 1 "refs/tags/$tag")"
  git -C "$ROOT" merge-base --is-ancestor "$commit" origin/main \
    || fail "tag $tag points at $commit, which is not reachable from origin/main"
  echo "release check: $commit is reachable from origin/main"

  check_no_existing_release "$tag"
  echo "release check: passed for $tag"
}

main "$@"
