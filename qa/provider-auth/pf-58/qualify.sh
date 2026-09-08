#!/usr/bin/env bash
set -euo pipefail
# Run only in the allocated RTX mirror, under the shared build lock.
export PATH=/home/travis/security-round5/tools:/home/travis/security-round5/evidence/broker/tools-venv/bin:/home/travis/.cargo/bin:$PATH
export CARGO_TARGET_DIR=/home/travis/repos/CorbanuTerminal-harness/codex-rs/target
export CARGO_BUILD_JOBS=8
# Match the harness child process: fixture vaults must never use the real keyring.
export CORBANU_TEST_NO_NATIVE_KEYRING=1
evidence=/home/travis/security-round5/evidence/provider-reauth-health-final
mkdir -p "$evidence"
export TMPDIR
TMPDIR=$(mktemp -d "$evidence/tmp.XXXXXX")
exec 9>/home/travis/security-round5/locks/build.lock
flock 9
trap 'bazel shutdown || true' EXIT
just bazel-lock-update > "$evidence/bazel-lock.log" 2>&1
just fix -p codex-provider-auth -p codex-login -p codex-protocol -p codex-rmcp-client -p codex-mcp -p codex-tui -p codex-core > "$evidence/fix.log" 2>&1
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
just test -p codex-app-server-protocol --retries 0 --test-threads 4 > "$evidence/protocol.log" 2>&1
just test -p codex-tui --lib -E 'test(provider_health) | test(provider_manager) | test(provider_management) | test(provider_status_host)' --retries 0 --test-threads 4 > "$evidence/tui.log" 2>&1
just codex --version > "$evidence/build.log" 2>&1
mkdir -p "$evidence/candidate"
cp "$CARGO_TARGET_DIR/debug/codex" "$evidence/candidate/codex"
sha256sum "$evidence/candidate/codex" > "$evidence/candidate.sha256"
export CORBANU_TMUX_REQUIRED=1
export CORBANU_TMUX_ARTIFACT_DIR="$evidence/tmux"
just test -p codex-tui --test all -E 'test(provider_management) | test(provider_convergence)' --retries 0 --test-threads 1 > "$evidence/tmux.log" 2>&1
