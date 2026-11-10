#!/usr/bin/env bash
# Applies the repository's protection settings through the GitHub REST API:
#   .github/rulesets/main.json          branch ruleset for main: squash PRs only, every required
#                                       check green, signed commits, linear history, no bypass
#   .github/rulesets/support.json       the same rules for support/** hotfix lines
#   .github/rulesets/tags.json          release tags can never be deleted or moved (no bypass)
#   .github/rulesets/tag-creation.json  only repository admins can create release tags
#   repository merge settings           squash only, PR title and body as the squash message,
#                                       auto-merge on, head branches deleted after merge
#   environment "production"            the owner must approve every release; only release tags
#                                       may deploy to it
#
# A mode flag is required. Without one, the script prints usage and changes nothing.
#   --dry-run   print what would be applied. Makes no API call that changes anything.
#   --apply     create or update everything above. Needs gh, logged in with admin rights.
#
# The repository is taken from REPO=owner/name, or from `gh repo view` in the current checkout.
# Rulesets are matched by name, so a second run updates in place and does not duplicate.
# Needs gh and node (no jq). Read docs/engineering/branch-protection.md before --apply.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly ROOT
readonly RULESET_FILES=(
  "$ROOT/.github/rulesets/main.json"
  "$ROOT/.github/rulesets/support.json"
  "$ROOT/.github/rulesets/tags.json"
  "$ROOT/.github/rulesets/tag-creation.json"
)
# Rulesets that were created by hand under another name. They are renamed by the update.
# Format: "<old name>=<name in the file>".
readonly LEGACY_NAMES=("tags=release tags")

readonly ENVIRONMENT="production"
readonly ENVIRONMENT_REVIEWER="olakra"
readonly ENVIRONMENT_TAG_PATTERN="*-v*.*.*"

usage() {
  cat >&2 <<'EOF'
usage:
  apply-branch-protection.sh --dry-run   print the settings, change nothing
  apply-branch-protection.sh --apply     create or update the settings on GitHub
EOF
  exit 2
}

# Prints the "name" of a ruleset file, and fails when the file is not valid JSON.
ruleset_name() {
  node -e 'const f = process.argv[1]; const j = JSON.parse(require("fs").readFileSync(f, "utf8")); process.stdout.write(j.name);' "$1" \
    || { echo "error: invalid JSON: $1" >&2; exit 1; }
}

existing_ruleset_id() {
  local repo="$1" name="$2" id legacy
  id="$(gh api --paginate "repos/${repo}/rulesets" --jq ".[] | select(.name == \"${name}\") | .id" | head -n1)"
  if [[ -z "$id" ]]; then
    for legacy in "${LEGACY_NAMES[@]}"; do
      [[ "${legacy#*=}" == "$name" ]] || continue
      id="$(gh api --paginate "repos/${repo}/rulesets" --jq ".[] | select(.name == \"${legacy%%=*}\") | .id" | head -n1)"
      [[ -n "$id" ]] && break
    done
  fi
  printf '%s' "$id"
}

apply_ruleset() {
  local file="$1" repo="$2" name id
  name="$(ruleset_name "$file")"
  id="$(existing_ruleset_id "$repo" "$name")"
  if [[ -n "$id" ]]; then
    echo "updating ruleset '$name' (id $id)" >&2
    gh api --method PUT "repos/${repo}/rulesets/${id}" --input "$file" >/dev/null
  else
    echo "creating ruleset '$name'" >&2
    gh api --method POST "repos/${repo}/rulesets" --input "$file" >/dev/null
  fi
}

apply_merge_settings() {
  local repo="$1"
  echo "setting merge options: squash only, PR title and body, auto-merge, delete head branches" >&2
  gh api --method PATCH "repos/${repo}" \
    -F allow_squash_merge=true -F allow_merge_commit=false -F allow_rebase_merge=false \
    -f squash_merge_commit_title=PR_TITLE -f squash_merge_commit_message=PR_BODY \
    -F allow_auto_merge=true -F delete_branch_on_merge=true -F allow_update_branch=true >/dev/null
}

apply_environment() {
  local repo="$1" reviewer_id existing
  reviewer_id="$(gh api "users/${ENVIRONMENT_REVIEWER}" --jq .id)"
  echo "configuring environment '$ENVIRONMENT': reviewer $ENVIRONMENT_REVIEWER, tags $ENVIRONMENT_TAG_PATTERN" >&2
  gh api --method PUT "repos/${repo}/environments/${ENVIRONMENT}" --input - >/dev/null <<EOF
{
  "prevent_self_review": false,
  "reviewers": [{ "type": "User", "id": ${reviewer_id} }],
  "deployment_branch_policy": { "protected_branches": false, "custom_branch_policies": true }
}
EOF
  existing="$(gh api "repos/${repo}/environments/${ENVIRONMENT}/deployment-branch-policies" \
    --jq ".branch_policies[] | select(.name == \"${ENVIRONMENT_TAG_PATTERN}\" and .type == \"tag\") | .id")"
  if [[ -z "$existing" ]]; then
    gh api --method POST "repos/${repo}/environments/${ENVIRONMENT}/deployment-branch-policies" \
      -f name="$ENVIRONMENT_TAG_PATTERN" -f type=tag >/dev/null
  fi
}

main() {
  [[ $# -eq 1 ]] || usage
  local file
  for file in "${RULESET_FILES[@]}"; do
    ruleset_name "$file" >/dev/null
  done

  case "$1" in
    --dry-run)
      for file in "${RULESET_FILES[@]}"; do
        echo "# ruleset: $(ruleset_name "$file") ($(basename "$file"))"
        cat "$file"
        echo
      done
      echo "# repository merge settings"
      echo "allow_squash_merge=true allow_merge_commit=false allow_rebase_merge=false"
      echo "squash_merge_commit_title=PR_TITLE squash_merge_commit_message=PR_BODY"
      echo "allow_auto_merge=true delete_branch_on_merge=true allow_update_branch=true"
      echo
      echo "# environment: $ENVIRONMENT"
      echo "reviewers=$ENVIRONMENT_REVIEWER prevent_self_review=false deployment tags=$ENVIRONMENT_TAG_PATTERN"
      ;;
    --apply)
      local repo="${REPO:-}"
      [[ -n "$repo" ]] || repo="$(gh repo view --json nameWithOwner --jq .nameWithOwner)"
      echo "applying to $repo" >&2
      for file in "${RULESET_FILES[@]}"; do
        apply_ruleset "$file" "$repo"
      done
      apply_merge_settings "$repo"
      apply_environment "$repo"
      if gh api "repos/${repo}/branches/main/protection" >/dev/null 2>&1; then
        echo "note: classic branch protection is still set on main. It overlaps the ruleset." >&2
        echo "      Remove it once the ruleset shows as active (docs/engineering/branch-protection.md)." >&2
      fi
      echo "done. Verify in Settings > Rules > Rulesets and Settings > Environments." >&2
      ;;
    *)
      usage
      ;;
  esac
}

main "$@"
