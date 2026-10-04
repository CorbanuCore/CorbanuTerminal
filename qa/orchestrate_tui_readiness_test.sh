#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
source "$ROOT/qa/orchestrate_tui_readiness.sh"

# Composer/footer excerpt from the public real-tui failure at merge
# a9815af79f1c3a8ffb39ed8005bd1260cf73eb4d; only the directory is synthetic.
ready=$(cat <<'SCREEN'
╭───────────────────────────────────────────────────╮
│ >_ Corbanu Terminal (v0.1.44)                     │
│ model:     qa-model   /model to change            │
╰───────────────────────────────────────────────────╯

› Explain this codebase

  qa-model via qa default · /tmp/qa-repository · Corbanu Terminal · TPS: -- tok/s
SCREEN
)

assert_ready() {
  local label=$1 screen=$2
  if ! orchestrate_composer_ready <<<"$screen"; then
    printf 'FAIL: %s should be ready\n' "$label" >&2
    exit 1
  fi
  printf 'PASS: %s\n' "$label"
}

assert_unready() {
  local label=$1 screen=$2
  if orchestrate_composer_ready <<<"$screen"; then
    printf 'FAIL: %s should not be ready\n' "$label" >&2
    exit 1
  fi
  printf 'PASS: %s\n' "$label"
}

# This is the old wait_screen predicate, including its newline flattening.
if grep -Fq 'qa-model default' <<<"${ready//$'\n'/ }"; then
  printf 'FAIL: current fixture unexpectedly satisfies the obsolete wait\n' >&2
  exit 1
fi
printf 'PASS: old readiness predicate rejects the current composer\n'
assert_ready 'current QA composer' "$ready"
assert_ready 'resumed QA composer' "${ready/Explain this codebase/Find and fix a bug}"
assert_ready 'viewport trailing blank lines' "$ready"$'\n\n  \n'
assert_ready 'old onboarding above current composer' $'Do you trust the contents?\n› 1. Yes, continue\n'"$ready"
assert_unready 'wrong model' "${ready//qa-model/other-model}"
assert_unready 'wrong provider' "${ready/via qa/via production}"
assert_unready 'model name suffix' "${ready//qa-model/other-qa-model}"
assert_unready 'provider name suffix' "${ready/via qa/via qa-other}"
assert_unready 'obsolete footer' "${ready/via qa /}"
assert_unready 'footer without composer' "${ready/› Explain this codebase/}"
assert_unready 'composer without footer' $'› Explain this codebase\n'
assert_unready 'onboarding only' $'Do you trust the contents of this directory?\n› 1. Yes, continue\n  Press enter to continue'
assert_unready 'quoted footer in content' $'› Explain this codebase\nThe previous screen said qa-model via qa default · /tmp/qa-repository'
assert_unready 'incomplete footer' $'› Explain this codebase\n  qa-model via qa'
assert_unready 'stale footer above prompt' $'  qa-model via qa default · /tmp/qa-repository\n› Explain this codebase'
assert_unready 'dialog after old composer' "$ready"$'\n› 1. Confirm action'
assert_unready 'wrong provider after old footer' "$ready"$'\n  qa-model via production default · /tmp/qa-repository'
assert_unready 'modal text after old footer' "$ready"$'\nApproval required\nPress enter to continue'
