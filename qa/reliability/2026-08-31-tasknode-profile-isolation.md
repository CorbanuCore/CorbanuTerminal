# Task Node profile isolation — P0 acceptance

- Reported: 2026-08-31. A terminal launched with the `goodalexander` Corbanu
  profile displayed the machine-global `secondfoundation` Task Node identity.
- Classification: P0 product-initiative authorization-boundary repair under
  explicit CEO direction; active-plan feature `PF-42`, completed sprint
  `PF-42-S01`.
- Branch: `fix/tasknode-profile-isolation` from
  `5e681b388e4d19d0bdd49d07c08803591bad31e5`.
- Implementation commit: `4cb0a21352331999f6292db89dcfc5d702fc2759`.
- Product linkage: **Shipping MVP — LIVE**, “Task Node and identity”; **Product
  principles**, “Security is the product” and “One product.”
- Next-agent handoff:
  [P0 Task Node profile isolation handoff](2026-08-31-tasknode-profile-isolation-handoff.md).

## Root cause and repair

Task Node active sessions and pending links used one pair of fixed encrypted
vault labels for the whole machine. Corbanu profile selection never reached the
session store, so the last linked account became authoritative in every tab.

Named Corbanu profiles now receive independent, stable hashed vault namespaces.
The selected profile is threaded through TUI and CLI linking, status, requests,
promotion, cancellation, and logout. Unprofiled launches retain the legacy
namespace. A named profile may import a legacy active session only when the
linked GitHub username matches the profile name case-insensitively; mismatched
authority is never returned.

## Automated acceptance

From `codex-rs`:

```text
cargo test -p codex-tasknode-session
# 13 passed; 0 failed

cargo check -p codex-cli -p codex-tui
# passed (pre-existing TUI dead-code warnings only)

cargo test -p codex-cli --bin codex tasknode_cli_accepts_profile_scoped_credentials
# 1 passed; 0 failed

cargo test -p codex-cli --bin codex profile_v2_is_allowed_for_runtime_subcommands
# 1 passed; 0 failed

cargo test -p codex-tui tasknode_menu_renders_the_selected_profile_scope --lib
# 1 passed; 0 failed (pre-existing warnings only)
```

The session tests use two adjacent identities, `goodalexander` and
`secondfoundation`, and prove independent active tokens, pending links, logout,
mismatched legacy rejection, and matching legacy migration.

Governance validation:

```text
python3 docs/plans/check.py
# passed: active 2/2

python3 docs/sprints/check.py
# failed on pre-existing duplicate sprint IDs, dependency resolution, and
# Corbanu API allocation/capacity records; no finding named PF-42-S01
```

The global sprint-check failure is not represented as a pass and was not
expanded into unrelated governance repair during this P0 incident.

## Real PTY acceptance

Launch commands used the normal local Corbanu configuration and trace logging:

```text
RUST_LOG=trace just codex -p goodalexander \
  -c 'log_dir="/tmp/corbanu-tasknode-profile-isolation-qa"'
RUST_LOG=trace just codex -p secondfoundation \
  -c 'log_dir="/tmp/corbanu-tasknode-profile-isolation-qa"'
```

For each launch, `/tasknode` was typed and submitted as separate PTY inputs.
Observed results:

- the `goodalexander` menu displayed `Corbanu profile: goodalexander` and did
  not display or authorize `secondfoundation`;
- the `secondfoundation` menu displayed
  `Corbanu profile: secondfoundation` in its independent scope;
- both menus offered linking in the local state present during QA, rather than
  importing a mismatched machine-global identity.

The root-level profile form used by scripts also completed successfully:

```text
cargo run --quiet --bin codex -- \
  -p goodalexander tasknode link status --json
```

Its redacted result reported `"profile":"goodalexander"` with no active or
pending session and no credential value.

No Task Node network action, financial action, credential disclosure, push, or
release was performed during acceptance.
