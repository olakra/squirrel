#!/usr/bin/env bash
# Applies the repository rulesets in .github/rulesets/ through the GitHub REST API:
#   main.json   branch ruleset for main (required checks, linear history, no force push)
#   tags.json   tag ruleset for v* (immutable release tags)
#
# A mode flag is required. Without one, the script prints usage and changes nothing.
#   --dry-run   print each validated JSON payload to stdout. Makes no API call.
#   --apply     create or update each ruleset by name, through `gh api`. Needs gh, logged in
#               with admin rights on the repository.
#
# The repository is taken from REPO=owner/name, or from `gh repo view` in the current checkout.
# Rulesets are matched by name, so a second run updates in place and does not duplicate.
# Read docs/engineering/branch-protection.md before running this with --apply.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly ROOT
readonly RULESET_FILES=("$ROOT/.github/rulesets/main.json" "$ROOT/.github/rulesets/tags.json")

usage() {
  cat >&2 <<'EOF'
usage:
  apply-branch-protection.sh --dry-run   print the ruleset payloads, no API call
  apply-branch-protection.sh --apply     create or update the rulesets on GitHub
EOF
  exit 2
}

# Fails when a payload is not valid JSON, before anything is sent.
validate() {
  local file="$1"
  jq empty "$file" || { echo "error: invalid JSON: $file" >&2; exit 1; }
}

apply_one() {
  local file="$1" repo="$2" name existing_id ids
  name="$(jq -r '.name' "$file")"
  ids="$(gh api --paginate "repos/${repo}/rulesets" --jq ".[] | select(.name == \"${name}\") | .id")"
  existing_id="$(printf '%s\n' "$ids" | head -n1)"
  if [[ -n "$existing_id" ]]; then
    echo "updating ruleset '$name' (id $existing_id) on $repo" >&2
    gh api --method PUT "repos/${repo}/rulesets/${existing_id}" --input "$file" >/dev/null
  else
    echo "creating ruleset '$name' on $repo" >&2
    gh api --method POST "repos/${repo}/rulesets" --input "$file" >/dev/null
  fi
}

main() {
  [[ $# -eq 1 ]] || usage
  local file
  for file in "${RULESET_FILES[@]}"; do
    validate "$file"
  done

  case "$1" in
    --dry-run)
      for file in "${RULESET_FILES[@]}"; do
        echo "# $(basename "$file")" >&2
        jq . "$file"
      done
      ;;
    --apply)
      local repo="${REPO:-}"
      if [[ -z "$repo" ]]; then
        repo="$(gh repo view --json nameWithOwner --jq .nameWithOwner)"
      fi
      for file in "${RULESET_FILES[@]}"; do
        apply_one "$file" "$repo"
      done
      echo "done. Verify in Settings > Rules > Rulesets." >&2
      ;;
    *)
      usage
      ;;
  esac
}

main "$@"
