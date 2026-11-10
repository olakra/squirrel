#!/usr/bin/env bash
# Fails a pull request that changes a released project without a changelog entry.
#
# Each project in scripts/release/projects.sh has watched paths and a CHANGELOG.md. A pull
# request passes for a project when it leaves the watched paths alone, or also changes that
# project's CHANGELOG.md, or carries the label "no-changelog". Give a reason for the label in
# the pull request body. The label opts out every project at once.
#
# Labels come from the pull_request event payload in $GITHUB_EVENT_PATH. ci.yml runs this
# job on labeled and unlabeled events, so adding the label re-runs the check.
#
# Usage:
#   scripts/ci/check-changelog.sh <base-sha> <head-sha>   check the pull request range
#   scripts/ci/check-changelog.sh --self-test             check the fixed scenarios below
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly ROOT
# shellcheck source=../release/projects.sh
source "$ROOT/scripts/release/projects.sh"

readonly OPT_OUT_LABEL='no-changelog'

usage() {
  cat >&2 <<'EOF'
usage:
  check-changelog.sh <base-sha> <head-sha>   check the changes in base...head
  check-changelog.sh --self-test             check the fixed scenarios
EOF
  exit 2
}

# evaluate <labels, one per line> <changed files, one per line>
# Returns 0 when the change passes for every project, 1 when any project fails. Prints the reason.
evaluate() {
  local labels="$1" files="$2" project watched_re changelog file watched changed failed=0 any=0
  for project in "${RELEASE_PROJECTS[@]}"; do
    watched_re="$(release_watched_re "$project")"
    changelog="$(release_changelog "$project")"
    watched=0
    changed=0
    while IFS= read -r file; do
      [[ -n "$file" ]] || continue
      if [[ "$file" =~ $watched_re ]]; then watched=1; fi
      if [[ "$file" == "$changelog" ]]; then changed=1; fi
    done <<< "$files"

    (( watched == 1 )) || continue
    any=1
    if (( changed == 1 )); then
      echo "changelog-check: $project: $changelog is updated"
    elif grep -qxF "$OPT_OUT_LABEL" <<< "$labels"; then
      echo "changelog-check: $project: skipped because the pull request has the '$OPT_OUT_LABEL' label"
    else
      echo "error: $project source changed, but $changelog did not." >&2
      echo "Add an entry under [Unreleased], or add the '$OPT_OUT_LABEL' label with a reason." >&2
      failed=1
    fi
  done

  (( any == 1 )) || echo "changelog-check: no change to the source of a released project"
  return "$failed"
}

# Reads the label names from the pull_request event payload. Prints nothing outside
# pull_request events, so a local run has no labels.
read_labels() {
  if [[ -z "${GITHUB_EVENT_PATH:-}" || ! -f "$GITHUB_EVENT_PATH" ]]; then
    return 0
  fi
  jq -r '.pull_request.labels[]?.name' "$GITHUB_EVENT_PATH"
}

self_test() {
  local failed=0 count=0 name expected labels files rc line
  local -a file_list
  # One scenario per line: name|expected exit|labels (comma separated)|files (space separated)
  while IFS='|' read -r name expected labels files; do
    [[ -n "$name" ]] || continue
    count=$((count + 1))
    # Turn the comma and space lists into the newline-separated form evaluate expects.
    line="${labels//,/$'\n'}"
    file_list=($files)
    rc=0
    evaluate "$line" "$(printf '%s\n' "${file_list[@]}")" >/dev/null 2>&1 || rc=$?
    if [[ "$rc" != "$expected" ]]; then
      echo "self-test FAIL: $name (expected exit $expected, got $rc)" >&2
      failed=1
    fi
  done <<'EOF'
source changed without changelog|1||projects/axiom/src/simulation/belt.rs
source changed with changelog|0||projects/axiom/src/simulation/belt.rs projects/axiom/CHANGELOG.md
source changed with no-changelog label|0|no-changelog|projects/axiom/src/simulation/belt.rs
manifest changed alone|1||projects/axiom/Cargo.toml
docs only|0||docs/engineering/versioning.md
web file only|0||projects/axiom/web/main.js
similar directory name is not watched|0||projects/axiom/src_notes/todo.md
other label does not opt out|1|bug|projects/axiom/src/wasm.rs
tictactoe core without changelog|1||projects/tictactoe/core/src/game.rs
tictactoe front end without changelog|1||projects/tictactoe/web/www/main.js
tictactoe with its changelog|0||projects/tictactoe/web/src/lib.rs projects/tictactoe/CHANGELOG.md
tictactoe test only|0||projects/tictactoe/web/test/page.test.js
wrong changelog does not count|1||projects/tictactoe/core/src/game.rs projects/axiom/CHANGELOG.md
one of two projects without changelog|1||projects/axiom/src/lib.rs projects/axiom/CHANGELOG.md projects/tictactoe/core/src/lib.rs
EOF

  if (( failed != 0 )); then
    echo "self-test failed" >&2
    return 1
  fi
  echo "self-test passed: $count scenarios"
}

main() {
  case "${1:-}" in
    --self-test)
      [[ $# -eq 1 ]] || usage
      self_test
      ;;
    *)
      [[ $# -eq 2 ]] || usage
      local files labels
      files="$(git diff --name-only "$1...$2")"
      labels="$(read_labels)"
      evaluate "$labels" "$files"
      ;;
  esac
}

main "$@"
