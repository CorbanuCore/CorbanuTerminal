#!/usr/bin/env bash
set -euo pipefail
repo=/home/travis/worktrees/pf13-main-release-20260911
evidence=/home/travis/security-round5/evidence/pf13-main-release-20260911
mkdir -p "$evidence"
export PATH=/home/travis/security-round5/tools:/home/travis/security-round5/evidence/broker/tools-venv/bin:/home/travis/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target
export CARGO_BUILD_JOBS=8 RUST_MIN_STACK=8388608 CORBANU_TEST_NO_NATIVE_KEYRING=1
export TMPDIR
TMPDIR=$(mktemp -d /home/travis/.pf13.XXXXXX)
cd "$repo"
exec 9>/home/travis/security-round5/locks/build.lock
flock 9
resume_at=${1:-}
run() {
    local label=$1
    shift
    if [[ -n "$resume_at" && "$label" != "$resume_at" ]]; then
        return 0
    fi
    resume_at=
    printf '%s START %s\n' "$(date -Is)" "$label"
    if "$@" > "$evidence/$label.log" 2>&1; then
        printf '%s PASS %s\n' "$(date -Is)" "$label"
    else
        local result=$?
        printf '%s FAIL %s %s\n' "$(date -Is)" "$label" "$result"
        tail -60 "$evidence/$label.log"
        return "$result"
    fi
}
run fix just fix -p codex-tui -p codex-wallet-daemon --lib
run fmt just fmt
git diff --binary > "$evidence/formatting.patch"
run bazel-lock just bazel-lock-update
bazel shutdown > "$evidence/bazel-shutdown.log" 2>&1 || true
run auth just test -p codex-provider-auth --retries 0 --test-threads 4
run wallet just test -p codex-wallet-daemon --lib --retries 0 --test-threads 4
run api just test -p codex-api --lib --retries 0 --test-threads 4
run catalog just test -p codex-model-provider-info --retries 0 --test-threads 4
run tui just test -p codex-tui --lib -E 'test(model_catalog) | test(model_picker) | test(provider_health) | test(provider_manager) | test(provider_management) | test(provider_status_host) | test(custom_model) | test(team_context) | test(tasknode)' --retries 0 --test-threads 4
run core just test -p codex-core --lib -E 'test(corbanu) | test(claude_plan_auth) | test(memory_stage_one) | test(provenance)' --retries 0 --test-threads 4
run security just test -p codex-secret-broker -p codex-content-security --lib --retries 0 --test-threads 4
run build bash -c 'cd codex-rs && cargo build --locked --bin codex --bin corbanu --bin corbanu-acp --bin codex-code-mode-host --bin corbanu-walletd -p codex-core -p codex-cli -p codex-code-mode-host -p codex-wallet-daemon'
run probe-build bash -c 'cd codex-rs && cargo build --locked -p codex-wallet-daemon -p codex-core --example package_client_probe'
candidate="$evidence/candidate"
run package python3 scripts/build_codex_package.py --target x86_64-unknown-linux-gnu --variant corbanu --package-dir "$candidate" --entrypoint-bin "$CARGO_TARGET_DIR/debug/corbanu" --code-mode-host-bin "$CARGO_TARGET_DIR/debug/codex-code-mode-host" --extra-bin "corbanu-acp=$CARGO_TARGET_DIR/debug/corbanu-acp" --extra-bin "corbanu-walletd=$CARGO_TARGET_DIR/debug/corbanu-walletd" --bwrap-bin /usr/bin/bwrap
run wallet-package python3 scripts/smoke_wallet_package.py --package-dir "$candidate" --probe "$CARGO_TARGET_DIR/debug/examples/package_client_probe"
run preflight python3 qa/provider-auth/pf-58/handoff_preflight.py --candidate "$candidate/bin/corbanu" --output "$evidence/preflight.json"
export CARGO_BIN_EXE_codex="$candidate/bin/corbanu"
export CARGO_BIN_EXE_corbanu="$candidate/bin/corbanu"
export CARGO_BIN_EXE_pfterminal="$candidate/bin/corbanu"
export CORBANU_TMUX_REQUIRED=1 CORBANU_TMUX_ARTIFACT_DIR="$evidence/tmux"
export INSTA_WORKSPACE_ROOT="$repo/codex-rs"
run tui-flows just test -p codex-tui --test all -E 'test(claude_auth) | test(provider_management) | test(provider_convergence) | test(provider_reauthentication)' --retries 0 --test-threads 2
run tools-tmux python3 qa/provider-auth/pf-58/handoff_tmux.py --candidate "$candidate/bin/corbanu" --repo "$repo" --evidence "$evidence/tools-tmux" --stream-cancel-key Escape
run trusted-apps python3 qa/provider-auth/pf-58/trusted_apps_journey.py --candidate "$candidate/bin/corbanu" --repo "$repo" --evidence "$evidence/trusted-apps"
sha256sum "$candidate/bin/corbanu" "$candidate/bin/codex-code-mode-host" "$candidate/bin/corbanu-walletd" > "$evidence/binaries.sha256"
printf '%s COMPLETE\n' "$(date -Is)"
