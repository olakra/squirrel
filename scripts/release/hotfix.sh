#!/usr/bin/env bash
# Starts a hotfix for a released version when main holds unreleased work that must not ship.
# `make hotfix-start` runs it. Owner only, because it may push a new support branch.
#
#   1. Finds the release tag for <from-version> (<project>-v<from-version>, or the legacy
#      v<from-version> for axiom).
#   2. Makes sure support/<project>-vMAJOR.MINOR exists on origin. If it does not, it is created
#      from the tag and pushed. The support ruleset protects it like main.
#   3. Creates the work branch hotfix/<name> from the support branch, with no upstream.
#
# If main has no unreleased work for the project, do not use this. Fix on main and cut a patch.
# Runbook: docs/engineering/release-process.md, "Hotfix flow".
#
# Usage: scripts/release/hotfix.sh <project> <from-version> <name>
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

[[ $# -eq 3 ]] || fail "usage: hotfix.sh <project> <from-version> <name>"
readonly PROJECT="$1" FROM="$2" NAME="$3"
release_is_project "$PROJECT" || fail "unknown project '$PROJECT'. Known: ${RELEASE_PROJECTS[*]}"
[[ "$FROM" =~ ^${SEMVER_RE}$ ]] || fail "'$FROM' is not a version"

git diff --quiet && git diff --cached --quiet || fail "the working tree has uncommitted changes"
echo "fetching origin..."
git fetch --quiet --tags origin

tag="$(release_tag_name "$PROJECT" "$FROM")"
if ! git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
  if [[ "$PROJECT" == axiom ]] && git rev-parse -q --verify "refs/tags/v$FROM" >/dev/null; then
    tag="v$FROM"
  else
    fail "no release tag for $PROJECT $FROM"
  fi
fi

minor="$(cut -d. -f1,2 <<< "${FROM%%-*}")"
support="support/${PROJECT}-v${minor}"
work="hotfix/${NAME}"
bash scripts/ci/check-branch-name.sh "$work" >/dev/null

if git rev-parse -q --verify "refs/remotes/origin/$support" >/dev/null; then
  echo "using existing origin/$support"
else
  echo "creating $support from $tag and pushing it to origin"
  git push origin "$(git rev-list -n 1 "refs/tags/$tag"):refs/heads/$support"
  git fetch --quiet origin "$support"
fi

git switch --quiet --no-track -c "$work" "origin/$support"
cat <<EOF
created $work from origin/$support

Next:
  1. Fix, add a test that fails before the fix, and add a changelog entry under [Unreleased].
  2. Open a PR into $support (not main):  gh pr create --base $support
  3. Release: make release-prepare PROJECT=$PROJECT VERSION=<x.y.z> BASE=origin/$support
              then: gh pr create --base $support, merge, and tag from $support.
  4. Forward-port the fix to main with a normal PR.
EOF
