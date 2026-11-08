#!/usr/bin/env bash
# Fails unless a bench log contains at least one benchmark result line.
#
# Contract for benchmark authors: every benchmark result is printed on its own
# line starting with "bench " (for example: "bench belt_stress_100k  1234 ns/iter").
# A run that compiles but prints no such line is treated as a failure, so the
# smoke test cannot pass silently.
#
# Usage: scripts/ci/check-bench-output.sh <path-to-log>
set -euo pipefail

log="${1:?usage: check-bench-output.sh <path-to-log>}"

if [[ ! -f "$log" ]]; then
  echo "error: bench log not found: $log" >&2
  exit 1
fi

count="$(grep -c '^bench ' "$log" || true)"
if [[ "$count" -lt 1 ]]; then
  echo "error: no lines starting with 'bench ' in $log" >&2
  exit 1
fi

echo "bench smoke: found $count result line(s) in $log"
