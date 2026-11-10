#!/usr/bin/env bash
# Prepares a release pull request for one project. `make release-prepare` runs it.
#
#   1. Checks the tree is clean, the version is valid and new, and [Unreleased] has entries.
#   2. Creates branch release/<project>-v<version> from <base> (default origin/main).
#   3. Moves the [Unreleased] entries into "## [<version>] - <today>" and updates the link
#      references at the bottom of the changelog.
#   4. Sets the version in the project's Cargo.toml, its package.json mirrors, and Cargo.lock.
#   5. Commits "chore(<project>): release v<version>". The commit is signed if git is set to sign.
#
# It pushes nothing and tags nothing. Next steps are printed at the end.
# Runbook: docs/engineering/release-process.md.
#
# Usage: scripts/release/prepare.sh <project> <version> [base-ref]
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

[[ $# -ge 2 && $# -le 3 ]] || fail "usage: prepare.sh <project> <version> [base-ref]"
readonly PROJECT="$1" VERSION="$2" BASE="${3:-origin/main}"

release_is_project "$PROJECT" || fail "unknown project '$PROJECT'. Known: ${RELEASE_PROJECTS[*]}"
[[ "$VERSION" =~ ^${SEMVER_RE}$ ]] || fail "'$VERSION' is not MAJOR.MINOR.PATCH[-prerelease]"

readonly TAG="$(release_tag_name "$PROJECT" "$VERSION")"
readonly BRANCH="release/${PROJECT}-v${VERSION}"
readonly CHANGELOG="$(release_changelog "$PROJECT")"
readonly VERSION_FILE="$(release_version_file "$PROJECT")"
readonly VERSION_TABLE="$(release_version_table "$PROJECT")"
readonly PROJECT_DIR="$(release_project_dir "$PROJECT")"
readonly TODAY="$(date -u +%Y-%m-%d)"

git diff --quiet && git diff --cached --quiet \
  || fail "the working tree has uncommitted changes. Commit or stash them first."

echo "fetching origin..."
git fetch --quiet --tags origin
git rev-parse -q --verify "$BASE" >/dev/null || fail "base ref '$BASE' not found"
git rev-parse -q --verify "refs/tags/$TAG" >/dev/null && fail "tag $TAG already exists"
git rev-parse -q --verify "refs/heads/$BRANCH" >/dev/null && fail "branch $BRANCH already exists"

# Read the current version and changelog from the base, not from whatever is checked out.
current="$(git show "$BASE:$VERSION_FILE" | awk -v want="$VERSION_TABLE" '
  /^\[/ { section = $0; next }
  section == want && /^version[ \t]*=/ { line = $0; sub(/^[^"]*"/, "", line); sub(/".*$/, "", line); print line; exit }
')"
[[ -n "$current" ]] || fail "no version in $VERSION_FILE at $BASE"
lowest="$(printf '%s\n%s\n' "$current" "$VERSION" | sort -V | head -n1)"
[[ "$lowest" == "$current" ]] || fail "$VERSION is lower than the current version $current"

git switch --quiet --no-track -c "$BRANCH" "$BASE"
echo "created $BRANCH from $BASE ($PROJECT $current -> $VERSION)"

bash scripts/ci/extract-changelog.sh Unreleased "$CHANGELOG" >/dev/null \
  || { git switch --quiet -; git branch -D "$BRANCH" >/dev/null; fail "$CHANGELOG has nothing under [Unreleased]"; }

# --- changelog: cut the section and update the link references --------------------------------
previous="$(release_previous_tag "$PROJECT")"
base_url="$(sed -n 's/^\[Unreleased\]:[[:space:]]*\(https:\/\/[^ ]*\)$/\1/p' "$CHANGELOG" | head -n1)"
[[ -n "$base_url" ]] || fail "$CHANGELOG has no '[Unreleased]: https://...' link reference"
base_url="$(sed -E 's#/(compare|commits|releases)/.*$##' <<< "$base_url")"
if [[ -n "$previous" ]]; then
  version_link="${base_url}/compare/${previous}...${TAG}"
else
  version_link="${base_url}/releases/tag/${TAG}"
fi

awk -v version="$VERSION" -v today="$TODAY" -v unreleased_link="${base_url}/compare/${TAG}...HEAD" \
  -v version_link="$version_link" '
  $0 == "## [Unreleased]" { print; print ""; print "## [" version "] - " today; next }
  /^\[Unreleased\]:/ { print "[Unreleased]: " unreleased_link; print "[" version "]: " version_link; next }
  { print }
' "$CHANGELOG" > "$CHANGELOG.tmp"
mv "$CHANGELOG.tmp" "$CHANGELOG"

# --- versions ------------------------------------------------------------------------------------
awk -v want="$VERSION_TABLE" -v version="$VERSION" '
  /^\[/ { section = $0 }
  !done && section == want && /^version[ \t]*=/ { sub(/"[^"]*"/, "\"" version "\""); done = 1 }
  { print }
' "$VERSION_FILE" > "$VERSION_FILE.tmp"
mv "$VERSION_FILE.tmp" "$VERSION_FILE"

files=("$CHANGELOG" "$VERSION_FILE")
while IFS= read -r mirror; do
  [[ -n "$mirror" ]] || continue
  sed -i -E '0,/^  "version":/s/^(  "version":[[:space:]]*")[^"]*(")/\1'"$VERSION"'\2/' "$mirror"
  files+=("$mirror")
done < <(release_mirror_files "$PROJECT")

# Cargo.lock records the workspace's own versions. --workspace touches only those entries.
if [[ -f "$PROJECT_DIR/Cargo.lock" ]]; then
  (cd "$PROJECT_DIR" && { cargo update --workspace --offline --quiet 2>/dev/null || cargo update --workspace --quiet; })
  files+=("$PROJECT_DIR/Cargo.lock")
fi

[[ "$(release_read_version "$PROJECT")" == "$VERSION" ]] || fail "version bump in $VERSION_FILE did not apply"
bash scripts/ci/check-release-tag.sh --local "$PROJECT"

git add -- "${files[@]}"
git commit --quiet -m "chore(${PROJECT}): release v${VERSION}" \
  -m "Moves the [Unreleased] entries into the ${VERSION} section and sets the version in
${VERSION_FILE} and its mirrors. The tag ${TAG} is created by the owner after merge."
echo
git log --oneline -1
cat <<EOF

Next:
  make ci                                            # same gates as CI
  make release-pr PROJECT=${PROJECT} VERSION=${VERSION}   # push ${BRANCH} and open the PR
After the PR merges (at least 24 hours open for a normal release):
  git switch main && git pull --ff-only
  make release-tag PROJECT=${PROJECT} && make release-push PROJECT=${PROJECT}
EOF
