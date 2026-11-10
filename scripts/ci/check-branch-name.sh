#!/usr/bin/env bash
# Checks a branch name against the branching model in docs/engineering/branching-strategy.md.
#
#   <type>/<short-name>            work branch, type: feature fix chore docs ci build perf refactor test hotfix
#   release/<project>-vX.Y.Z       release preparation branch (make release-prepare)
#   support/<project>-vX.Y         maintenance line for hotfixes (make hotfix-start)
#   dependabot/...                 Dependabot
#   revert-<n>-...                 GitHub's "Revert" button
#   claude/...                     Claude Code cloud sessions
#
# Names are lowercase: letters, digits, ".", "_", "-", with "/" between segments.
# `main` is valid only as a base, never as a pull request head, so it fails here.
#
# Usage:
#   scripts/ci/check-branch-name.sh <branch>
#   scripts/ci/check-branch-name.sh              check the current branch
#   scripts/ci/check-branch-name.sh --self-test
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly ROOT
# shellcheck source=../release/projects.sh
source "$ROOT/scripts/release/projects.sh"

readonly WORK_TYPES='feature|fix|chore|docs|ci|build|perf|refactor|test|hotfix'
readonly SEGMENT='[a-z0-9][a-z0-9._-]*'

projects_alt() {
  local IFS='|'
  echo "${RELEASE_PROJECTS[*]}"
}

# Returns 0 for a valid name. Prints the kind on success.
classify() {
  local name="$1" p
  p="$(projects_alt)"
  if [[ "$name" =~ ^(${WORK_TYPES})/${SEGMENT}(/${SEGMENT})*$ ]]; then
    echo "work (${BASH_REMATCH[1]})"
  elif [[ "$name" =~ ^release/(${p})-v${SEMVER_RE}$ ]]; then
    echo "release preparation"
  elif [[ "$name" =~ ^support/(${p})-v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
    echo "support line"
  elif [[ "$name" =~ ^dependabot/.+$ ]]; then
    echo "dependabot"
  elif [[ "$name" =~ ^revert-[0-9]+-.+$ ]]; then
    echo "revert"
  elif [[ "$name" =~ ^claude/${SEGMENT}(/${SEGMENT})*$ ]]; then
    echo "agent session"
  else
    return 1
  fi
}

self_test() {
  local expected name rc failed=0 count=0
  while IFS='|' read -r expected name; do
    [[ -n "$name" ]] || continue
    count=$((count + 1))
    rc=0
    classify "$name" >/dev/null || rc=1
    if [[ "$rc" != "$expected" ]]; then
      echo "self-test FAIL: expected $expected, got $rc: $name" >&2
      failed=1
    fi
  done <<'EOF'
0|feature/inserter-state-machine
0|fix/lane-compression
0|chore/branching-release-strategy
0|docs/roadmap
0|ci/checkout-v7
0|hotfix/crash-on-load
0|feature/axiom/belt-sleep
0|release/axiom-v0.2.0
0|release/tictactoe-v0.1.0
0|release/axiom-v1.0.0-rc.1
0|support/axiom-v0.2
0|dependabot/cargo/projects/axiom/serde-1.0.200
0|revert-41-feature/belt-sleep
0|claude/fix-ci-run
1|main
1|support/axiom-v0.2.1
1|release/0.1.1
1|release/unknown-v0.1.0
1|feature/Upper-Case
1|feature/
1|my-branch
1|feat/belt
1|feature/two  spaces
1|feature//double
EOF
  (( failed == 0 )) || { echo "self-test failed" >&2; return 1; }
  echo "self-test passed: $count names"
}

main() {
  local name kind
  case "${1:-}" in
    --self-test) self_test; return ;;
    '') name="$(git -C "$ROOT" rev-parse --abbrev-ref HEAD)" ;;
    *) name="$1" ;;
  esac
  if kind="$(classify "$name")"; then
    echo "branch-name: '$name' is a valid $kind branch"
    return 0
  fi
  cat >&2 <<EOF
error: branch name '$name' does not follow docs/engineering/branching-strategy.md.
Use one of:
  <type>/<short-name>         type: ${WORK_TYPES//|/, }
  release/<project>-vX.Y.Z    projects: ${RELEASE_PROJECTS[*]}
  support/<project>-vX.Y
Lowercase letters, digits, '.', '_' and '-' only. Rename with: git branch -m <new-name>
EOF
  return 1
}

main "$@"
