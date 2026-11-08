#!/usr/bin/env bash
# Builds the engineering docs site from the Markdown in docs/ and projects/axiom/docs/.
#
# Offline: needs only bash and python3 (standard library). Output goes to docs/site/_site, or to
# $SITE_OUT. The staging tree goes to a temporary directory unless $SITE_STAGE is set.
# The GitHub Pages workflow runs this script and uploads the output directory.
set -euo pipefail

site_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$site_dir/../.." && pwd)"
out_dir="${SITE_OUT:-$site_dir/_site}"

if [[ -z "${SITE_STAGE:-}" ]]; then
  # The staging tree holds Markdown, which the repository's markdownlint job would otherwise lint.
  stage_dir="$(mktemp -d)"
  trap 'rm -rf "$stage_dir"' EXIT
else
  stage_dir="$SITE_STAGE"
fi

python3 "$site_dir/build.py" \
  --repo-root "$repo_root" \
  --site-dir "$site_dir" \
  --stage "$stage_dir" \
  --out "$out_dir"
