# PF-84 #428: D3 follows the level the TUI shows (2026-10-11)

Travis's decision (2026-10-11, option 1): a spawn `account` switch chosen by the model needs the human's approval
(decision D3) whenever the TUI shows Aggressive. That includes "Aggressive enforced, boundary unverified" and
"Active in this session: Aggressive", not only sessions where Core's own level is Aggressive. When approvals are off, the
switch is refused.

**Change.** An Aggressive launch (from the stored `/security` level, or a nested launch held to it) always selects
the `corbanu-aggressive` permission profile and sets the `CORBANU_SECURITY_ORIGIN` marker. Core now treats either
signal as Aggressive for D3, through `Permissions::launch_enforces_aggressive`. This happens even when Core's own level
stays Permissive because no activation preflight ran. Launch refuses to start Aggressive if any other config layer
defines that profile. Either signal alone can only make the check stricter.

Review fix: when no answer comes back (a client that can't ask, such as `corbanu exec`, or a dismissed question), the
model is told that no answer was given. It is no longer told the user declined.

## Live check (tmux, GLM 5.3 Flash)

Setup for every run:

- A private tmux socket and a disposable home with `CORBANU_TEST_NO_NATIVE_KEYRING=1`.
- The real default Z.AI key, from the vault helper (`provider/zai_api_key`).
- A named account `work` holding a fake key, so a child on `work` gets a 401.
- The prompt asks the model to call `spawn_agent` once with `account = "work"`.

Builds and states:

- Both binaries are macOS arm64 debug builds of `147b6d9131`.
  - **after** is the build as committed.
  - **before** is the same tree with only the new D3 condition removed, which matches `origin/main`.
- **restart:** Aggressive is saved through `/security` without the preflight feature ("Core's level stays Permissive"),
  then `r` restarts the TUI, and `/security` shows "Active in this session: Aggressive".
- **unverified:** `security_level.toml` says Aggressive and `protected_mode_preflight` is on, so the TUI warns
  "Aggressive is enforced, but the protected boundary is unverified".

| Run | D3 question | Result | Core turn context |
| --- | --- | --- | --- |
| [before, restart](captures/before-restart.txt) | not shown | child spawned on `work`, 401 | `security_level=permissive`; child `provider_account=work` |
| [before, unverified](captures/before-unverified.txt) | not shown | child spawned on `work`, 401 | `security_level=permissive`; child `provider_account=work` |
| [after, restart](captures/after-restart.txt) | shown | Cancel: "The user declined…", no child | `security_level=permissive`, no child rollout |
| [after, unverified](captures/after-unverified.txt) | shown | Cancel: declined, no child | `security_level=permissive`, no child rollout |

An earlier run on the first commit `7aa6d93577`, in the unverified state, answered **Allow once**: the child ran on `work`
and got a 401. Scripts: [launch.sh](scripts/launch.sh), [case.sh](scripts/case.sh). A scan of the run folders for the
real Z.AI key found 0 hits, and the folders were deleted afterwards.

Video (SOP, leak scan passed):
[pf84-d3-tui-aggressive](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s03-pf84-d3-tui-aggressive-147b6d91318b-2026-10-10.mp4).
It saves Aggressive without the preflight, restarts, confirms Aggressive is active, then shows the spawn on `work`
asking first; Cancel runs nothing.

## Tests

The new tests fail on `origin/main` behaviour. With the condition removed, the approvals-off and ask tests fail.

- `core` `multi_agents_tests.rs`:
  - `tui_aggressive_spawn_account_switch_is_refused_with_approvals_off`: profile plus marker, profile only, marker only.
  - `tui_aggressive_spawn_account_switch_asks_the_human`: Cancel, no answer, Allow once.
  - `permissive_spawn_account_switch_asks_nothing_even_when_it_could`.
- `tui` `security/launch_tests.rs` `shown_aggressive_states_gate_spawn_account_switches_in_core` covers three states:
  - the level file missing while the Aggressive rules are present;
  - Aggressive saved without a preflight, with the preflight feature on;
  - Permissive saved for the next start ("Active in this session: Aggressive").

Gate:

- `just test -p codex-core -p codex-tui -p codex-security-level` (filters `multi_agents|provider_account|security::launch|
  security::aggressive|security::nested|nested_launch|level::`, plus the whole `codex-security-level` package): 221 passed.
- The same run with `developer-accounting`: 202 passed.
- `protected_surface|write_stdin|mcp_tool_call|post_taint`: 130 passed.
- Linux clippy `-D warnings` on glitch, with and without `developer-accounting`, for `codex-security-level`,
  `codex-core`, `codex-tui`, `codex-exec` and `codex-cli`: clean.
- One independent Opus 5.5 High review (read-only), APPROVE WITH CHANGES:
  1. Fixed: the no-answer message.
  2. Recorded: the #428 decision in the S03 record and the plan's D3 row, and this tmux run.
  3. Fixed: the shared question id.
  4. Fixed: the unverified-state test turns the preflight feature on.
