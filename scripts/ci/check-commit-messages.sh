#!/usr/bin/env bash
# Checks commit subjects against Conventional Commits 1.0, using this repo's type list.
# Rules and examples: docs/engineering/commit-conventions.md.
#
# Only the subject line (the first line) is checked. Bodies and footers are free text.
# Merge commits are skipped. Reverts made by `git revert` ("Revert \"...\"") pass.
#
# Usage:
#   scripts/ci/check-commit-messages.sh <base-sha> <head-sha>   check commits in base..head
#   scripts/ci/check-commit-messages.sh --subject "<text>"      check one subject (a PR title)
#   scripts/ci/check-commit-messages.sh --self-test             check the fixed samples below
set -euo pipefail

# type(scope)!: summary. The scope is optional, lowercase, and may contain . _ / -.
# The "!" marks a breaking change.
readonly SUBJECT_RE='^(feat|fix|perf|refactor|test|docs|build|ci|chore|revert)(\([a-z0-9][a-z0-9._/-]*\))?!?: [^ ].*$'
readonly MAX_SUBJECT_LENGTH=72

usage() {
  cat >&2 <<'EOF'
usage:
  check-commit-messages.sh <base-sha> <head-sha>   check every commit in base..head
  check-commit-messages.sh --subject "<text>"      check one subject line (for a PR title)
  check-commit-messages.sh --self-test             check the fixed samples
EOF
  exit 2
}

# Returns 0 when the subject is valid, 1 when the format is wrong, 2 when it is too long.
check_subject() {
  local subject="$1"
  case "$subject" in
    Merge\ *) return 0 ;;
    'Revert "'*) return 0 ;;
  esac
  if [[ ! "$subject" =~ $SUBJECT_RE ]]; then
    return 1
  fi
  if (( ${#subject} > MAX_SUBJECT_LENGTH )); then
    return 2
  fi
  return 0
}

# Prints one offending subject with the reason. Output goes to stderr.
report() {
  local ref="$1" rc="$2" subject="$3" reason
  if [[ "$rc" == 1 ]]; then
    reason="does not match <type>(<scope>): <summary>, with a type from the list"
  else
    reason="longer than $MAX_SUBJECT_LENGTH characters"
  fi
  printf '  %s  %s\n      %s\n' "$ref" "$subject" "$reason" >&2
}

self_test() {
  # Each line: expected exit code, a pipe, then the subject. Exit codes match check_subject.
  local expected subject rc failed=0 count=0
  while IFS='|' read -r expected subject; do
    [[ -n "$expected" ]] || continue
    count=$((count + 1))
    rc=0
    check_subject "$subject" || rc=$?
    if [[ "$rc" != "$expected" ]]; then
      echo "self-test FAIL: expected $expected, got $rc: $subject" >&2
      failed=1
    fi
  done <<'EOF'
0|feat(axiom): add ring-buffer belt lanes
0|fix(sim): compress back-pressure in one pass
0|feat(axiom)!: change the WASM export signature
0|docs: add the versioning policy
0|revert: undo the chunk sleep change
0|ci(release): verify the tag before building
0|build: pin the wasm toolchain
0|chore(axiom): release v0.1.0
0|Revert "feat(axiom): add splitter"
0|Merge pull request #7 from olakra/feature/splitter
1|Added new stuff
1|feature(axiom): add a thing
1|fix:
1|fix(Axiom): uppercase scope is not allowed
1|fix(axiom) missing the colon
1|fix:no space after the colon
2|feat(axiom): add a chunk grid that keeps every chunk sorted and wakes it on change
EOF

  if (( failed != 0 )); then
    echo "self-test failed" >&2
    return 1
  fi
  echo "self-test passed: $count samples"
}

check_range() {
  local base="$1" head="$2" log sha subject rc count=0 failures=0
  log="$(git log --no-merges --format='%H%x09%s' "${base}..${head}")"
  while IFS=$'\t' read -r sha subject; do
    [[ -n "$sha" ]] || continue
    count=$((count + 1))
    rc=0
    check_subject "$subject" || rc=$?
    if (( rc != 0 )); then
      failures=$((failures + 1))
      report "${sha:0:10}" "$rc" "$subject"
    fi
  done <<< "$log"

  if (( failures > 0 )); then
    echo "error: $failures of $count commit subject(s) do not follow Conventional Commits" >&2
    return 1
  fi
  echo "commit-lint: $count commit(s) in ${base}..${head} follow Conventional Commits"
}

main() {
  case "${1:-}" in
    --self-test)
      [[ $# -eq 1 ]] || usage
      self_test
      ;;
    --subject)
      [[ $# -eq 2 ]] || usage
      local rc=0
      check_subject "$2" || rc=$?
      if (( rc != 0 )); then
        report "title" "$rc" "$2"
        echo "error: the pull request title is not a valid Conventional Commit subject" >&2
        return 1
      fi
      echo "commit-lint: pull request title is valid"
      ;;
    *)
      [[ $# -eq 2 ]] || usage
      check_range "$1" "$2"
      ;;
  esac
}

main "$@"
