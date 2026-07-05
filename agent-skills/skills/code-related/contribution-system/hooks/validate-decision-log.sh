#!/usr/bin/env bash
# PostToolUse hook (Write|Edit): validates decision-log.yaml files with
# `diffviz validate decision-log` right after they're written, feeding
# failures back to the agent so it self-corrects instead of leaving a
# non-compliant log (missing reasoning prefix, unresolvable commit hash)
# to be caught later by someone running the validator by hand.
set -euo pipefail

input="$(cat)"
file_path="$(jq -r '.tool_input.file_path // empty' <<<"$input")"

[[ -n "$file_path" ]] || exit 0
[[ "$file_path" == *contributions* ]] || exit 0
[[ "$(basename -- "$file_path")" == "decision-log.yaml" ]] || exit 0

if ! command -v diffviz >/dev/null 2>&1; then
  # diffviz isn't installed in this environment; nothing to validate against.
  exit 0
fi

if output="$(diffviz validate decision-log "$file_path" 2>&1)"; then
  exit 0
fi

reason="$(printf '%s' "$output" | jq -Rs .)"
printf '{"decision":"block","reason":%s}\n' "$reason"
