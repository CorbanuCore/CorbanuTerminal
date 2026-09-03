# Handoff — P0 Task Node profile isolation

## Mission

Help the CEO validate and, if authorized, ship the P0 repair for Task Node
identity bleed between multiple Corbanu Terminal users on one machine.

The reported failure was concrete: a tab understood by the user as
`goodalexander` displayed `Linked as secondfoundation`, even though
`goodalexander` had linked previously. Treat that as evidence of the general
authorization failure, not as a two-name special case.

## Current state

| Field | Value |
| --- | --- |
| Repository | `/home/pfrpc/repos/CorbanuTerminal` |
| Branch | `fix/tasknode-profile-isolation` |
| Base | `5e681b388e4d19d0bdd49d07c08803591bad31e5` |
| Implementation | `4cb0a21352331999f6292db89dcfc5d702fc2759` |
| Governance/evidence | `ce8c6e6b46f1` |
| Change class | P0 product initiative: authorization/vault boundary |
| Plan feature | Active P0 security plan, `PF-42` |
| Sprint | `PF-42-S01`, completed and archived |
| Push/release | Not performed; no explicit release instruction was given |
| Human acceptance | Pending CEO test |

The tracked worktree was clean when this handoff was written. The branch has no
configured upstream and contains two local commits beyond the recorded base.

Canonical records:

- [Acceptance evidence](2026-08-31-tasknode-profile-isolation.md)
- [Completed sprint](../../docs/sprints/archive/p0-security-levels/pf-42-s01-tasknode-profile-isolation.md)
- [Active plan feature](../../docs/plans/active/p0-security-levels.md#pf-42)
- [Finished user guidance](../../docs/features/tasknode.md)

## Root cause

`codex-tasknode-session` stored the active bearer session and pending link under
two fixed vault labels for the entire `CODEX_HOME`:

```text
tasknode/session
tasknode/link-pending
```

Corbanu's existing `--profile` selection never reached this store. Every tab
using the same home therefore shared the last linked Task Node authority.

## Implemented boundary

Named Corbanu profiles now use deterministic hashed vault namespaces for both
active and pending state. The validated profile is threaded through Task Node
TUI and CLI link, poll, status, request, chat, promotion, cancellation, and
logout paths.

Important behavior:

- Named profiles never read another named profile's bearer or pending link.
- Logout clears only the selected profile.
- The `/tasknode` menu displays `Corbanu profile: <name>` beside linked state.
- The unprofiled/default launch retains the old machine-global namespace for
  backward compatibility and single-user use.
- A named profile imports a legacy global session only when its profile name
  matches the linked GitHub username case-insensitively.
- A mismatched legacy session is never returned. The user sees an unlinked
  profile and must link it once.
- A previously overwritten token cannot be recovered locally.
- No usernames are hard-coded into production routing; the reported names
  appear only in generalized tests, docs, and evidence.

Primary code:

- `codex-rs/tasknode-session/src/lib.rs`
- `codex-rs/tasknode-session/src/tests.rs`
- `codex-rs/tui/src/chatwidget/tasknode_menu.rs`
- `codex-rs/cli/src/tasknode_cmd.rs`
- `codex-rs/cli/src/main.rs`

## Profile UX clarification

`--profile` was an existing Corbanu configuration/workspace concept; this fix
did not invent it. The new behavior is that Task Node finally honors it.

For one Task Node user, this remains valid and uses the legacy default scope:

```bash
corbanu-debug --yolo
```

For two independent Task Node users, Corbanu needs a local namespace selector:

```bash
corbanu-debug --yolo -p goodalexander
corbanu-debug --yolo -p secondfoundation
```

A terminal tab merely titled `goodalexander` is not necessarily a Corbanu
profile. If the user's normal tab launcher already passes `-p goodalexander`,
no new launch step is required. If it only sets a visual tab name, the exact
reported workflow is not yet automatic: trace that launcher and either make it
pass the existing profile or return to product authority for an account-switcher
decision. Do not silently infer identity from arbitrary tab titles.

This UX question is the main follow-up raised by the CEO after implementation.
Do not describe the P0 as fully human-accepted until the normal named-tab launch
path—not only a manual `-p` command—has been confirmed.

## Local `corbanu-debug` wiring

The external wrapper was intentionally changed outside Git:

```text
/home/pfrpc/.local/bin/corbanu-debug
```

It currently contains:

```sh
#!/bin/sh
export CODEX_HOME='/home/pfrpc/.corbanu-debug'
exec '/home/pfrpc/repos/CorbanuTerminal/codex-rs/target/debug/codex' "$@"
```

Therefore `corbanu-debug --yolo` runs this worktree's debug binary and forwards
all arguments. Rebuilding `codex-rs/target/debug/codex` updates what the wrapper
runs without another relink.

Verified binary SHA-256 at handoff:

```text
d362832ee943bd98b296621fe8650895e9ce43de67922d4b0d274b85d62bbf4e
```

The debug wrapper uses `~/.corbanu-debug`, not the normal `~/.corbanu`. Existing
normal-home profile links will not automatically appear in debug. This is
intentional test isolation and likely means each debug profile must link once.

To rebuild and verify the link:

```bash
cd /home/pfrpc/repos/CorbanuTerminal/codex-rs
cargo build --bin codex
corbanu-debug --yolo --version
```

Do not replace the packaged release under
`~/.corbanu/packages/standalone/current/` unless the CEO explicitly authorizes a
release or installation change.

## CEO acceptance flow

Use two separate terminals or tabs:

1. Launch `corbanu-debug --yolo -p goodalexander`.
2. Open `/tasknode`; confirm `Corbanu profile: goodalexander`.
3. Link the intended Task Node account if the debug scope is unlinked.
4. Launch `corbanu-debug --yolo -p secondfoundation` separately.
5. Open `/tasknode`; confirm `Corbanu profile: secondfoundation` and link the
   second account.
6. Reopen `/tasknode` in both terminals and confirm each shows its own linked
   GitHub identity.
7. Log out one profile; confirm the other remains linked and usable.
8. Restart both commands; confirm each identity persists in its own scope.
9. If the CEO normally uses a named-tab launcher, repeat through that exact
   launcher and inspect whether it actually passes the expected profile.

Never request or print a Task Node token. Linking stays in the browser/TUI flow.

## Automated and PTY evidence

Passed on the implementation tree:

```text
cargo clippy -p codex-tasknode-session --tests -- -D warnings
cargo test -p codex-tasknode-session
# 13 passed

cargo test -p codex-cli --bin codex tasknode_cli_accepts_profile_scoped_credentials
cargo test -p codex-cli --bin codex profile_v2_is_allowed_for_runtime_subcommands
# 1 passed in each focused run

cargo test -p codex-tui tasknode_menu_renders_the_selected_profile_scope --lib
# 1 passed; unrelated existing warnings disclosed

cargo check -p codex-cli -p codex-tui
```

The real PTY launched both named profiles, sent `/tasknode` text and Enter as
separate key inputs, displayed the correct profile label, and never displayed
the other identity. Both local states offered linking during that run.

Root-level scripted profile selection was also exercised:

```bash
cargo run --quiet --bin codex -- \
  -p goodalexander tasknode link status --json
```

It returned `"profile":"goodalexander"` with no secret value.

## Governance and known limitations

- `python3 docs/plans/check.py` passes with two active plans and zero free slots.
- `python3 docs/sprints/check.py` remains globally red on pre-existing duplicate
  sprint IDs, dependency resolution, and Corbanu API allocation/capacity errors.
  It emitted no PF-42-S01-specific finding. The full result is disclosed in the
  acceptance record; do not claim the global checker passed.
- No release record was created, no remote branch was pushed, and no packaged
  binary was replaced.
- The feature-level benchmark was marked not applicable because this is one
  local vault lookup on an interactive path.
- Live application-repository qualification was marked not applicable to this
  local identity store; the affected Terminal PTY was exercised directly.
- Named human acceptance and the exact normal tab-launcher behavior remain the
  meaningful release blockers.

## Next-agent priorities

1. Sit with the CEO's test result. If it fails, capture the exact launch command,
   visible profile line, and linked identity without exposing tokens.
2. Determine whether the existing named-tab launcher passes a Corbanu profile.
   Fix the launcher/profile boundary if authorized; do not parse tab-title text
   as authentication state.
3. If the two-profile link/logout/restart matrix passes, record named human
   acceptance in the P0 evidence and sprint/plan release linkage as required.
4. Only push, install, or publish when the CEO explicitly authorizes that
   release action. Follow the repository release gate and disclose the existing
   global sprint-check failures accurately.
5. Preserve the general invariant: one selected local profile owns exactly one
   Task Node authority namespace; no literal-user special cases or regex routing.
