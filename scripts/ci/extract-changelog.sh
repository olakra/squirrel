#!/usr/bin/env bash
# Prints the body of one version section from a Keep a Changelog file.
#
# The body is every line after the "## [<version>]" heading, up to the next "## " heading
# or the link-reference block at the end of the file. The heading itself is not printed.
# Blank lines at the start and end of the body are removed.
#
# Usage:
#   scripts/ci/extract-changelog.sh <version> [changelog-path]
#   scripts/ci/extract-changelog.sh --self-test
#
# Exit status: 0 when the section exists and has text. 1 when it is missing or empty.
set -euo pipefail

readonly DEFAULT_CHANGELOG="projects/axiom/CHANGELOG.md"

# Read as: one awk program, so the file is scanned once.
extract() {
  local version="$1" file="$2"
  awk -v heading="## [${version}]" '
    BEGIN { inside = 0; found = 0; n = 0 }
    !inside {
      # Exact match on "## [<version>]", followed by the end of the line or a space.
      if (substr($0, 1, length(heading)) == heading) {
        rest = substr($0, length(heading) + 1)
        if (rest == "" || substr(rest, 1, 1) == " ") {
          inside = 1
        }
      }
      next
    }
    /^## / || /^\[[^]]+\]:[ ]/ { exit }
    { body[++n] = $0 }
    END {
      if (!inside) { exit 2 }
      s = 1
      e = n
      while (s <= e && body[s] ~ /^[ \t]*$/) s++
      while (e >= s && body[e] ~ /^[ \t]*$/) e--
      if (e < s) { exit 3 }
      for (i = s; i <= e; i++) print body[i]
    }
  ' "$file"
}

self_test() {
  local failed=0 out tmp
  tmp="$(mktemp -d)"

  cat > "$tmp/CHANGELOG.md" <<'EOF'
# Changelog

## [Unreleased]

Nothing yet.

## [0.1.0] - 2026-11-08

### Added
- ring-buffer belts

## [0.0.1]

## [0.1.10]

- later patch text

[Unreleased]: https://example.invalid/compare
[0.1.0]: https://example.invalid/tag
EOF

  expect_ok() {
    local version="$1" needle="$2" got
    if ! got="$(extract "$version" "$tmp/CHANGELOG.md")"; then
      echo "self-test FAIL: $version should exist" >&2
      failed=1
      return
    fi
    if ! grep -qF -- "$needle" <<< "$got"; then
      echo "self-test FAIL: $version body lacks '$needle'" >&2
      failed=1
    fi
    if grep -qE '^## |^\[[^]]+\]: ' <<< "$got"; then
      echo "self-test FAIL: $version body leaked a heading or link definition" >&2
      failed=1
    fi
  }

  expect_missing() {
    local version="$1"
    if extract "$version" "$tmp/CHANGELOG.md" >/dev/null 2>&1; then
      echo "self-test FAIL: $version should not be extracted" >&2
      failed=1
    fi
  }

  expect_ok "0.1.0" "- ring-buffer belts"
  expect_ok "Unreleased" "Nothing yet."
  expect_ok "0.1.10" "later patch text"
  expect_missing "0.1"        # a prefix of 0.1.0 must not match
  expect_missing "0.2.0"      # no such section
  expect_missing "0.0.1"      # heading with an empty body

  out="$(extract "0.1.0" "$tmp/CHANGELOG.md")"
  if [[ "$(head -n1 <<< "$out")" != "### Added" ]]; then
    echo "self-test FAIL: leading blank line was not trimmed" >&2
    failed=1
  fi

  rm -rf "$tmp"
  if (( failed != 0 )); then
    echo "self-test failed" >&2
    return 1
  fi
  echo "self-test passed: 6 checks"
}

main() {
  if [[ "${1:-}" == "--self-test" ]]; then
    self_test
    return
  fi
  local version="${1:?usage: extract-changelog.sh <version> [changelog-path]}"
  local file="${2:-$DEFAULT_CHANGELOG}" out
  if [[ ! -f "$file" ]]; then
    echo "error: changelog not found: $file" >&2
    return 1
  fi
  if ! out="$(extract "$version" "$file")"; then
    echo "error: no non-empty section [$version] in $file" >&2
    return 1
  fi
  printf '%s\n' "$out"
}

main "$@"
