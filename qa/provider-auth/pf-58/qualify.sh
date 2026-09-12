#!/usr/bin/env bash
set -euo pipefail
# Run only in the allocated RTX mirror, under the shared build lock.
export PATH=/home/travis/security-round5/tools:/home/travis/security-round5/evidence/broker/tools-venv/bin:/home/travis/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target
export CARGO_BUILD_JOBS=8
# Match the harness child process: fixture vaults must never use the real keyring.
export CORBANU_TEST_NO_NATIVE_KEYRING=1
export RUST_MIN_STACK=8388608
evidence=/home/travis/security-round5/evidence/provider-reauth-health-final
mkdir -p "$evidence"
export TMPDIR
TMPDIR=$(mktemp -d "$evidence/tmp.XXXXXX")
exec 9>/home/travis/security-round5/locks/build.lock
flock 9
trap 'bazel shutdown || true' EXIT
just bazel-lock-update > "$evidence/bazel-lock.log" 2>&1
just fix -p codex-provider-auth -p codex-login -p codex-protocol -p codex-rmcp-client -p codex-mcp -p codex-tui -p codex-core -p codex-app-server -p codex-wallet-daemon > "$evidence/fix.log" 2>&1
just write-app-server-schema > "$evidence/schema.log" 2>&1
just write-app-server-schema --experimental > "$evidence/schema-experimental.log" 2>&1
just fmt > "$evidence/fmt.log" 2>&1
git diff --binary > "$evidence/formatted.patch"
just test -p codex-provider-auth --retries 0 --test-threads 4 > "$evidence/provider-auth.log" 2>&1
just test -p codex-protocol -E 'test(error)' --retries 0 --test-threads 4 > "$evidence/error-protocol.log" 2>&1
just test -p codex-login -E 'test(openai_auth_metadata) | test(load_auth_keeps_codex_api_key_env_precedence) | test(load_auth_reads_ambient_api_key_from_env)' --retries 0 --test-threads 1 > "$evidence/login.log" 2>&1
just test -p codex-rmcp-client --test streamable_http_recovery --retries 0 --test-threads 2 > "$evidence/rmcp.log" 2>&1
just test -p codex-mcp -E 'test(startup) | test(authentication)' --retries 0 --test-threads 4 > "$evidence/mcp.log" 2>&1
just test -p codex-core -E 'test(mcp_auth_refresh)' --retries 0 --test-threads 2 > "$evidence/core.log" 2>&1
just test -p codex-core --lib -E 'test(direct_native_pane_inherits_live_security_policy) | test(corbanu_auth_helper) | test(claude_plan_auth_uses_the_runtime) | test(rebuild_preserving_session_layers)' --retries 0 --test-threads 2 > "$evidence/core-runtime.log" 2>&1
just test -p codex-app-server --test all -E 'test(permission_reload_preserves_active_runtime) | test(turn_start_rejects_invalid_permission_selection) | test(turn_start_permission_profile_rebinds_runtime_workspace_roots)' --retries 0 --test-threads 1 > "$evidence/permission-runtime.log" 2>&1
just test -p codex-wallet-daemon --lib --retries 0 --test-threads 2 > "$evidence/wallet-package.log" 2>&1
python3 -m unittest discover -s scripts/codex_package -p 'test_*.py' > "$evidence/package-python.log" 2>&1
python3 -m unittest discover -s qa/provider-auth/pf-58 -p 'test_handoff_gate.py' > "$evidence/gate-python.log" 2>&1
just test -p codex-app-server-protocol --retries 0 --test-threads 4 > "$evidence/protocol.log" 2>&1
just test -p codex-tui --lib -E 'test(provider_health) | test(provider_manager) | test(provider_management) | test(provider_status_host)' --retries 0 --test-threads 4 > "$evidence/tui.log" 2>&1
just codex --version > "$evidence/build.log" 2>&1
just code-mode-host --help > "$evidence/build-host.log" 2>&1
(cd codex-rs && cargo build --bin pfterminal-walletd) > "$evidence/build-walletd.log" 2>&1
# Use the canonical layout, including search/sandbox resources, not a loose CLI.
package=$(mktemp -d "$evidence/package.XXXXXX")
python3 scripts/build_codex_package.py --target x86_64-unknown-linux-gnu --variant codex \
    --package-dir "$package" --entrypoint-bin "$CARGO_TARGET_DIR/debug/codex" \
    --code-mode-host-bin "$CARGO_TARGET_DIR/debug/codex-code-mode-host" \
    --extra-bin "pfterminal=$CARGO_TARGET_DIR/debug/codex" \
    --extra-bin "pfterminal-walletd=$CARGO_TARGET_DIR/debug/pfterminal-walletd" \
    --bwrap-bin /usr/bin/bwrap
candidate="$package/bin"
sha256sum "$candidate/codex" "$candidate/codex-code-mode-host" "$candidate/pfterminal-walletd" > "$candidate/binaries.sha256"
python3 qa/provider-auth/pf-58/handoff_preflight.py --candidate "$candidate/codex" --output "$candidate/package-preflight.json"
python3 qa/provider-auth/pf-58/handoff_tmux.py --candidate "$candidate/codex" --repo "$PWD" --evidence "$candidate/tools-tmux"
export CARGO_BIN_EXE_codex="$candidate/codex"
export CARGO_BIN_EXE_corbanu="$candidate/codex"
export CARGO_BIN_EXE_pfterminal="$candidate/codex"
export CORBANU_TMUX_REQUIRED=1
export CORBANU_TMUX_ARTIFACT_DIR="$evidence/tmux"
just test -p codex-tui --test all -E 'test(provider_management) | test(provider_convergence)' --retries 0 --test-threads 1 > "$evidence/tmux.log" 2>&1
# Package checks alone do not prove the actual launch environment or all human
# conditions. This explicit input must come from that launcher's MCP preflight.
: "${CORBANU_HANDOFF_RUNTIME_REPORT:?Actual launcher preflight required before human handoff}"
python3 qa/provider-auth/pf-58/handoff_gate.py --candidate "$candidate/codex" --repo "$PWD" \
    --runtime-report "$CORBANU_HANDOFF_RUNTIME_REPORT" --tools-report "$candidate/tools-tmux/result.json" \
    --evidence "$candidate/human-readiness"
