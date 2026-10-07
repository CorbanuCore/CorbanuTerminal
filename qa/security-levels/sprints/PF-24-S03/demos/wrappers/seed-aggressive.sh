#!/bin/bash
# Demo wrapper: stores Aggressive in the disposable profile (as the picker
# would) before the first launch, then runs the candidate.
# Keyring isolation: never let a demo candidate reach the real OS keyring.
[ -n "${CORBANU_TEST_NO_NATIVE_KEYRING:-}" ] || {
  echo "refusing: CORBANU_TEST_NO_NATIVE_KEYRING is not set" >&2; exit 1; }
REAL="${PF24_CANDIDATE:?set PF24_CANDIDATE to the candidate corbanu binary}"
printf 'version = 1\nlevel = "aggressive"\n' > "$CODEX_HOME/security_level.toml"
# Synthetic canary (not a credential) that Aggressive must strip from agent commands.
export DEMO_VAULT_CANARY=fake-canary-0001
exec "$REAL" "$@"
