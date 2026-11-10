#!/usr/bin/env bash
# Creates the signed, annotated release tag for one project. `make release-tag` runs it.
# Owner only: CLAUDE.md forbids agents from running it.
#
# The tag goes on the squash commit of the release pull request ("chore(<project>): release
# v<version>"), found on the current branch. The current branch must be main or the project's
# support branch, and must equal its origin counterpart. Nothing is pushed: run
# `make release-push PROJECT=<project>` after checking the tag.
#
# Usage: scripts/release/tag.sh <project>
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

[[ $# -eq 1 ]] || fail "usage: tag.sh <project>"
readonly PROJECT="$1"
release_is_project "$PROJECT" || fail "unknown project '$PROJECT'. Known: ${RELEASE_PROJECTS[*]}"

git diff --quiet && git diff --cached --quiet || fail "the working tree has uncommitted changes"

branch="$(git rev-parse --abbrev-ref HEAD)"
version="$(release_read_version "$PROJECT")"
tag="$(release_tag_name "$PROJECT" "$version")"
minor="$(cut -d. -f1,2 <<< "${version%%-*}")"
case "$branch" in
  main | "support/${PROJECT}-v${minor}") ;;
  *) fail "tag from main or support/${PROJECT}-v${minor}, not '$branch'" ;;
esac

echo "fetching origin..."
git fetch --quiet --tags origin
[[ "$(git rev-parse HEAD)" == "$(git rev-parse "origin/$branch")" ]] \
  || fail "$branch is not equal to origin/$branch. Run: git pull --ff-only"
git rev-parse -q --verify "refs/tags/$tag" >/dev/null && fail "tag $tag already exists locally"
[[ -z "$(git ls-remote --tags origin "refs/tags/$tag")" ]] || fail "tag $tag already exists on origin"

bash scripts/ci/check-release-tag.sh --local "$PROJECT"

# The release commit, searched on the first-parent history of this branch only.
release_subject="chore(${PROJECT}): release v${version}"
commit=""
while IFS=$'\t' read -r sha subject; do
  # The squash commit subject is the PR title, which GitHub ends with " (#<number>)".
  if [[ "$subject" == "$release_subject" || "$subject" =~ ^"$release_subject"\ \(#[0-9]+\)$ ]]; then
    commit="$sha"
    break
  fi
done < <(git log --first-parent --format='%H%x09%s' -n 200)
[[ -n "$commit" ]] || fail "no commit 'chore(${PROJECT}): release v${version}' on $branch. Merge the release PR first."

ahead="$(git rev-list --count "${commit}..HEAD")"
if (( ahead > 0 )); then
  echo "note: $branch has $ahead commit(s) after the release commit. They are not in $tag."
fi

if [[ -z "$(git config --get user.signingkey || true)" ]]; then
  fail "no user.signingkey set. Release tags are signed. See docs/engineering/claude-code-setup.md#signed-commits"
fi

notes="$(bash scripts/ci/extract-changelog.sh "$version" "$(release_changelog "$PROJECT")")"
git tag -s "$tag" -m "${PROJECT} v${version}" -m "$notes" "$commit"
git verify-tag "$tag" >/dev/null 2>&1 || echo "note: git cannot verify the tag locally (gpg.ssh.allowedSignersFile unset?)"

echo
echo "created signed tag $tag on $(git log --format='%h %s' -1 "$commit")"
echo "push it with: make release-push PROJECT=${PROJECT}"
