#!/usr/bin/env bash
# Release registry: the one place that lists the projects that ship versions, and where each
# keeps its version and changelog. Every release script, check, and workflow reads this file.
# Policy: docs/engineering/versioning.md and docs/engineering/adr/0006-per-project-release-tags.md.
#
# Source it; do not run it:   source scripts/release/projects.sh
#
# To add a project: add its id to RELEASE_PROJECTS and a case to each function below, then add
# its changelog and its watched paths. Nothing else needs to change for tagging and verification.

# Project ids. A release tag is "<id>-vMAJOR.MINOR.PATCH[-prerelease]".
readonly RELEASE_PROJECTS=(axiom tictactoe)

# The project whose stable releases are marked "Latest" on GitHub. GitHub has one Latest
# release per repository, so the other projects publish with --latest=false.
readonly RELEASE_PRIMARY_PROJECT=axiom

# SemVer core with an optional pre-release suffix. No leading zeros.
readonly SEMVER_RE='(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z.-]+)?'

release_is_project() {
  local p
  for p in "${RELEASE_PROJECTS[@]}"; do
    [[ "$p" == "$1" ]] && return 0
  done
  return 1
}

# Directory of the project, relative to the repository root.
release_project_dir() {
  case "$1" in
    axiom) echo "projects/axiom" ;;
    tictactoe) echo "projects/tictactoe" ;;
    *) return 1 ;;
  esac
}

# The TOML file and table that hold the authoritative version.
release_version_file() {
  case "$1" in
    axiom) echo "projects/axiom/Cargo.toml" ;;
    tictactoe) echo "projects/tictactoe/Cargo.toml" ;;
    *) return 1 ;;
  esac
}

release_version_table() {
  case "$1" in
    axiom) echo "[package]" ;;
    tictactoe) echo "[workspace.package]" ;;
    *) return 1 ;;
  esac
}

# package.json files whose "version" mirrors the authoritative version. May print nothing.
release_mirror_files() {
  case "$1" in
    axiom) echo "projects/axiom/desktop/package.json" ;;
    tictactoe) echo "projects/tictactoe/web/package.json" ;;
    *) return 1 ;;
  esac
}

release_changelog() {
  case "$1" in
    axiom) echo "projects/axiom/CHANGELOG.md" ;;
    tictactoe) echo "projects/tictactoe/CHANGELOG.md" ;;
    *) return 1 ;;
  esac
}

# Extended regex of the paths whose change needs a changelog entry (scripts/ci/check-changelog.sh).
release_watched_re() {
  case "$1" in
    axiom) echo '^projects/axiom/(src/|Cargo\.toml$)' ;;
    tictactoe) echo '^projects/tictactoe/(Cargo\.toml$|(core|web)/(src/|Cargo\.toml$)|web/www/)' ;;
    *) return 1 ;;
  esac
}

# Glob of tags that belong to the project. axiom also owns the legacy unprefixed v0.1.2 tag,
# cut before per-project tags existed; it is never moved (docs/engineering/tag-protection.md).
release_tag_globs() {
  case "$1" in
    axiom) printf '%s\n' 'axiom-v*' 'v[0-9]*' ;;
    *) printf '%s\n' "$1-v*" ;;
  esac
}

release_tag_name() {
  echo "$1-v$2"
}

# Prints "<project> <version>" for a tag such as axiom-v0.2.0. Fails for anything else,
# including the legacy unprefixed form, which no new release may use.
release_parse_tag() {
  local tag="$1" project version
  [[ "$tag" =~ ^([a-z][a-z0-9]*)-v(${SEMVER_RE})$ ]] || return 1
  project="${BASH_REMATCH[1]}"
  version="${BASH_REMATCH[2]}"
  release_is_project "$project" || return 1
  echo "$project $version"
}

# Prints the version from the project's TOML table. Run from the repository root.
release_read_version() {
  local file table
  file="$(release_version_file "$1")" || return 1
  table="$(release_version_table "$1")"
  awk -v want="$table" '
    /^\[/ { section = $0; next }
    section == want && /^version[ \t]*=/ {
      line = $0
      sub(/^[^"]*"/, "", line)
      sub(/".*$/, "", line)
      print line
      exit
    }
  ' "$file"
}

# Latest existing tag of the project by version order, or nothing. Needs tags fetched.
# Sorts on the version part only, so the legacy "v0.1.2" and "axiom-v0.2.0" compare correctly.
release_previous_tag() {
  local -a globs
  mapfile -t globs < <(release_tag_globs "$1")
  git tag -l "${globs[@]}" \
    | awk '{ v = $0; sub(/^([a-z][a-z0-9]*-)?v/, "", v); print v "\t" $0 }' \
    | sort -V -k1,1 \
    | tail -n1 \
    | cut -f2
}

# Prints the top-level "version" of a package.json without jq (not every machine has it).
release_json_version() {
  sed -n 's/^  "version":[[:space:]]*"\([^"]*\)".*/\1/p' "$1" | head -n1
}
