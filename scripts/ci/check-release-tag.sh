#!/usr/bin/env bash
# Verifies that a release tag may be published. release.yml runs it in job "verify".
# `make release-check PROJECT=<id>` runs it in --local mode, which tags nothing.
#
# A release tag is "<project>-vMAJOR.MINOR.PATCH[-prerelease]", for example axiom-v0.2.0.
# The projects, and where each keeps its version and changelog: scripts/release/projects.sh.
#
# Checks, in order:
#   1. The tag has the form above and names a known project.
#   2. The tag version equals the project's authoritative version (Cargo.toml).
#   3. Every mirror file (package.json) carries the same version.
#   4. The project's CHANGELOG.md has a non-empty section for that version.
#   5. (CI mode only) The tag is an annotated tag.
#   6. (CI mode only) The tag's commit is on origin/main, or on the project's support branch
#      origin/support/<project>-vMAJOR.MINOR (hotfix line, docs/engineering/branching-strategy.md).
#   7. (CI mode only) No GitHub release exists for the tag yet, so a tag is never republished.
#
# Usage:
#   scripts/ci/check-release-tag.sh <tag>              CI mode: checks 1 to 7. Needs origin/main
#                                                      (and the support branch, if any) and the tag
#                                                      fetched, and GH_TOKEN and GITHUB_REPOSITORY set.
#   scripts/ci/check-release-tag.sh --local <project>  checks 1 to 4 against the current version.
#   scripts/ci/check-release-tag.sh --self-test        checks the tag parser.
#
# In CI mode, "project=<id>" and "version=<x.y.z>" are appended to $GITHUB_OUTPUT when it is set.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly ROOT
# shellcheck source=../release/projects.sh
source "$ROOT/scripts/release/projects.sh"

usage() {
  echo "usage: check-release-tag.sh <project>-v<version> | --local <project> | --self-test" >&2
  exit 2
}

fail() {
  echo "error: $*" >&2
  exit 1
}

# Fails when any release (published or draft) already uses the tag. If the API call fails,
# the script fails too, so a broken check never lets a republish through.
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

# Checks 2 to 4: the version files and the changelog agree with <version>.
check_sources() {
  local project="$1" version="$2" current mirror mirror_version changelog
  current="$(cd "$ROOT" && release_read_version "$project")"
  [[ -n "$current" ]] || fail "no version found in $(release_version_file "$project")"
  [[ "$current" == "$version" ]] \
    || fail "$project version in $(release_version_file "$project") is $current, not $version"
  echo "release check: $(release_version_file "$project") has version $version"

  while IFS= read -r mirror; do
    [[ -n "$mirror" ]] || continue
    mirror_version="$(release_json_version "$ROOT/$mirror")"
    [[ "$mirror_version" == "$version" ]] \
      || fail "$mirror has version $mirror_version, not $version. Run scripts/release/prepare.sh."
    echo "release check: $mirror has version $version"
  done < <(release_mirror_files "$project")

  changelog="$(release_changelog "$project")"
  bash "$ROOT/scripts/ci/extract-changelog.sh" "$version" "$ROOT/$changelog" >/dev/null \
    || fail "$changelog has no non-empty section for [$version]"
  echo "release check: $changelog has a section for $version"
}

# Check 6. A tag may sit on main, or on the support branch of its own minor line.
check_reachable() {
  local tag="$1" project="$2" version="$3" commit minor support
  commit="$(git -C "$ROOT" rev-list -n 1 "refs/tags/$tag")"
  git -C "$ROOT" rev-parse -q --verify origin/main >/dev/null \
    || fail "origin/main is not present. Fetch it before this check."
  if git -C "$ROOT" merge-base --is-ancestor "$commit" origin/main; then
    echo "release check: $commit is reachable from origin/main"
    return 0
  fi
  minor="$(cut -d. -f1,2 <<< "$version")"
  support="origin/support/${project}-v${minor}"
  if git -C "$ROOT" rev-parse -q --verify "$support" >/dev/null \
    && git -C "$ROOT" merge-base --is-ancestor "$commit" "$support"; then
    echo "release check: $commit is reachable from $support (hotfix line)"
    return 0
  fi
  fail "tag $tag points at $commit, which is not reachable from origin/main or $support"
}

self_test() {
  local failed=0 count=0 expected tag got
  while IFS='|' read -r expected tag; do
    [[ -n "$tag" ]] || continue
    count=$((count + 1))
    got="$(release_parse_tag "$tag" 2>/dev/null || echo FAIL)"
    if [[ "$got" != "$expected" ]]; then
      echo "self-test FAIL: $tag: expected '$expected', got '$got'" >&2
      failed=1
    fi
  done <<'EOF'
axiom 0.2.0|axiom-v0.2.0
axiom 1.0.0-rc.1|axiom-v1.0.0-rc.1
tictactoe 0.1.0|tictactoe-v0.1.0
FAIL|v0.2.0
FAIL|axiom-0.2.0
FAIL|axiom-v0.2
FAIL|axiom-v01.2.0
FAIL|unknown-v0.1.0
FAIL|Axiom-v0.1.0
EOF
  (( failed == 0 )) || { echo "self-test failed" >&2; return 1; }
  echo "self-test passed: $count tags"
}

main() {
  local project version tag parsed
  case "${1:-}" in
    --self-test)
      [[ $# -eq 1 ]] || usage
      self_test
      return
      ;;
    --local)
      [[ $# -eq 2 ]] || usage
      project="$2"
      release_is_project "$project" || fail "unknown project '$project'. Known: ${RELEASE_PROJECTS[*]}"
      version="$(cd "$ROOT" && release_read_version "$project")"
      tag="$(release_tag_name "$project" "$version")"
      release_parse_tag "$tag" >/dev/null || fail "$project version '$version' is not a valid release version"
      echo "release check: $project is at $version (tag would be $tag)"
      check_sources "$project" "$version"
      return 0
      ;;
    ''|-*)
      usage
      ;;
  esac

  [[ $# -eq 1 ]] || usage
  tag="$1"
  parsed="$(release_parse_tag "$tag")" \
    || fail "tag '$tag' must be <project>-vMAJOR.MINOR.PATCH[-prerelease] for one of: ${RELEASE_PROJECTS[*]}"
  read -r project version <<< "$parsed"
  echo "release check: tag $tag is project $project, version $version"
  if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    printf 'project=%s\nversion=%s\n' "$project" "$version" >> "$GITHUB_OUTPUT"
  fi

  check_sources "$project" "$version"

  git -C "$ROOT" rev-parse -q --verify "refs/tags/$tag" >/dev/null \
    || fail "tag $tag is not present locally. Fetch it first."
  [[ "$(git -C "$ROOT" cat-file -t "refs/tags/$tag")" == "tag" ]] \
    || fail "tag $tag is a lightweight tag. Release tags must be annotated."
  echo "release check: tag $tag is annotated"

  check_reachable "$tag" "$project" "$version"
  check_no_existing_release "$tag"
  echo "release check: passed for $tag"
}

main "$@"
