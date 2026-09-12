#!/usr/bin/env bash
set -euo pipefail
export PATH=/home/travis/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target
export CARGO_BUILD_JOBS=8 RUST_MIN_STACK=8388608 CORBANU_TEST_NO_NATIVE_KEYRING=1
export CORBANU_TMUX_REQUIRED=1 INSTA_UPDATE=no
export INSTA_WORKSPACE_ROOT=/home/travis/security-round5/picker-repair-20260910/codex-rs
export CARGO_BIN_EXE_codex=/home/travis/security-round5/evidence/blind-repairs-20260910/candidate/bin/codex
export CARGO_BIN_EXE_corbanu=$CARGO_BIN_EXE_codex
export CARGO_BIN_EXE_pfterminal=$CARGO_BIN_EXE_codex
evidence=/home/travis/security-round5/evidence/blind-repairs-20260910
export TMPDIR
TMPDIR=$(mktemp -d "$evidence/tmp-final.XXXXXX")
export CORBANU_TMUX_ARTIFACT_DIR=$evidence/apps-failures
cd /home/travis/security-round5/picker-repair-20260910
exec 9>/home/travis/security-round5/locks/build.lock
flock 9
just test -p codex-provider-auth -p codex-tui --lib \
  -E 'test(credential_health) | test(model_catalog) | test(model_picker) | test(provider_manager) | test(provider_management) | test(provider_model_policy) | test(provider_health) | test(provider_status_host) | test(status_line) | test(terminal_title) | test(interrupt)' \
  --retries 0 --test-threads 4 > "$evidence/unit-final.log" 2>&1
just test -p codex-tui --test all -E 'test(reauthentication_apps)' \
  --retries 0 --test-threads 1 > "$evidence/apps-test.log" 2>&1
