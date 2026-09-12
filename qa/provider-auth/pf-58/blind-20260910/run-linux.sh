#!/usr/bin/env bash
set -euo pipefail
# Replays existing durable cases against the immutable packaged candidate.
# No real credential files or native keyring are used by this runner.
export PATH=/home/travis/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target
export CARGO_BUILD_JOBS=8
export RUST_MIN_STACK=8388608
export CORBANU_TEST_NO_NATIVE_KEYRING=1
export CORBANU_TMUX_REQUIRED=1
export INSTA_WORKSPACE_ROOT=/home/travis/security-round5/picker-repair-20260910/codex-rs
export CARGO_BIN_EXE_codex=/home/travis/security-round5/evidence/picker-20260910/candidate-final/bin/codex
export CARGO_BIN_EXE_corbanu=$CARGO_BIN_EXE_codex
export CARGO_BIN_EXE_pfterminal=$CARGO_BIN_EXE_codex
evidence=/home/travis/security-round5/evidence/blind-20260910
mkdir -p "$evidence"
export TMPDIR
TMPDIR=$(mktemp -d "$evidence/tmp.XXXXXX")
export CORBANU_TMUX_ARTIFACT_DIR=$evidence/failures
cd /home/travis/security-round5/picker-repair-20260910
sha256sum "$CARGO_BIN_EXE_codex" > "$evidence/candidate.sha256"
exec 9>/home/travis/security-round5/locks/build.lock
flock 9
just test -p codex-tui --test all \
  -E 'test(claude_auth) | test(provider_management) | test(provider_convergence) | test(provider_reauthentication)' \
  --retries 0 --test-threads 2 > "$evidence/tmux-tests.log" 2>&1
python3 qa/provider-auth/pf-58/handoff_tmux.py \
  --candidate "$CARGO_BIN_EXE_codex" \
  --repo "$PWD" --evidence "$evidence/package-tmux" \
  --stream-cancel-key C-c > "$evidence/package-tmux.log" 2>&1
