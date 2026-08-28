# PF-22-S01 final-tree evidence

- Date: 2026-08-24 UTC
- Baseline commit: `3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb`
- Primary implementation: `9711bcd94c`
- Auxiliary-agent inheritance correction: `2a0f3abfd0`
- Thread-manager security-default correction: `18c4a0a3fa`
- Tested code tree: `2a0f3abfd0974841cc881d82b09796a6a9bf436e`
- TUI applicability: none; PF-24 and PF-25 own interactive qualification.

## Result

Core now owns one typed effective-policy state built from persisted human security state and existing lower-level decisions. Permissive leaves existing decisions unchanged; Moderate and Aggressive can only narrow them. Level transitions apply atomically after a valid human confirmation, stale confirmations fail, and unknown or corrupt state fails closed.

Every policy view binds the effective level, human-and-agent actor chain, session and task identifiers, revocation generation, and kill-switch state. Explicit child spawns inherit this state. Core-owned auxiliary agents, including Guardian sessions, inherit from the root binding without being mistaken for unbound user-spawned children. Policy text arriving from models, project instructions, tools, hooks, plugins, connectors, or MCP servers is rejected without parsing it into mutable state or reflecting it into trusted output.

## Final-tree commands

```text
cd codex-rs && just fix -p codex-core
PASS — clippy completed; only pre-existing Core dead-code warnings were emitted.

cd codex-rs && just fmt
PASS — formatting completed and the final diff was inspected.

cd codex-rs && just test -p codex-core effective_policy
PASS — 6 focused effective-policy tests passed.

cd codex-rs && just test -p codex-core security_inheritance
PASS — 3 inheritance tests passed, including the live parent/child spawn path.

cd codex-rs && just test -p codex-core guardian_ephemeral_retry_preserves_parallel_trunk_and_fork_history
PASS — 1 Guardian auxiliary-session regression passed.

cd codex-rs && TMPDIR=/var/tmp/corbanu-pf22-tests just test
BASELINE NOT GREEN — 15,788 tests ran: 15,617 passed, 169 failed, 2 timed out, 28 skipped. Failures were distributed across unrelated pre-existing snapshot, plugin, code-mode timing, shell, permissions, and vault-login suites; no PF-22 focused selector failed. Test-generated snapshot and fixture debris was removed afterward.

cargo build -p codex-cli --bin corbanu
PASS — current Corbanu binary built successfully.

python3 scripts/security-level-compat --baseline 3c1b2f6cbe11657ff4e3b72b11db029c9e7a92eb --candidate codex-rs/target/debug/corbanu --output qa/security-levels/sprints/PF-21-S01/harness
PASS — all 5 immutable probes passed across 10 frozen Permissive policy surfaces.
```

## Changed paths

- `codex-rs/Cargo.lock`
- `codex-rs/core-api/Cargo.toml`
- `codex-rs/core-api/src/lib.rs`
- `codex-rs/core/src/agent/control.rs`
- `codex-rs/core/src/agent/control/spawn.rs`
- `codex-rs/core/src/agent/control_tests.rs`
- `codex-rs/core/src/lib.rs`
- `codex-rs/core/src/security/effective_policy.rs`
- `codex-rs/core/src/security/effective_policy_tests.rs`
- `codex-rs/core/src/security/mod.rs`
- `codex-rs/core/src/session/session.rs`
- `codex-rs/thread-manager-sample/src/main.rs`
