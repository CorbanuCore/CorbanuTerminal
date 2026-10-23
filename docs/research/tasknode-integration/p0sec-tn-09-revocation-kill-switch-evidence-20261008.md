# Task Node evidence record: P0SEC-TN-09

Compile the P0SEC-TN-09 PF-25-S02 Sprint Evidence Record. Task `task_8e353c0cb1ee43239c5a7691668087ab`, request `req_05dcfc83d07c336d3feb70b9642dd4b6a06b4661dc5dd29820b49e50558d8abb`. Every PR below is merged to `main` in
CorbanuCore/CorbanuTerminal. Each merge commit was checked with `git merge-base --is-ancestor <sha> origin/main`
at `319f7695f6` (2026-10-08). The quoted blocks are copied verbatim from the sprint and gate records (links
re-pointed to this file's location).

Per-sprint gate (Travis, 2026-10-06): focused `just test`, a GLM 5.2 tmux run, one independent Opus 5.5 High
review and SOP demo videos, then merge behind the feature flag. **Not claimed:** the milestone code-blind VM run,
human sign-off, flag removal and the P1 hardening plan. Carried-forward items are recorded as open, not done.

| Sprint | PRs (merge commit on main) | Flag | Record status |
| --- | --- | --- | --- |
| [PF-25-S02](../../sprints/archive/p0-security-levels/pf-25-s02-revocation-and-kill-switch-tui.md) Revocation and kill-switch TUI | #261 (`43b21fc899`) | `security_levels` | completed (archived) |

PR links:

- #261: https://github.com/CorbanuCore/CorbanuTerminal/pull/261

## PF-25-S02: Revocation and kill-switch TUI

Summary: tests core 66/66; tui 190/190 focused; Linux clippy clean on the RTX box. TUI run: GLM 5.2 tmux run under Aggressive (revoke a grant, kill switch on across restart, off keeps the level). Review: Opus 5.5 High: three rounds, round 3 APPROVE.

Gate record: [qa/security-levels/sprints/PF-25-S02/README.md](../../../qa/security-levels/sprints/PF-25-S02/README.md).

Verbatim, sprint record `docs/sprints/archive/p0-security-levels/pf-25-s02-revocation-and-kill-switch-tui.md`, Verification:

> - [x] `just fix -p codex-core -p codex-tui -p codex-app-server-client`; `just fmt`.
> - [x] `just test -p codex-core` (`security_revocation`, `transition`, `level_change`, `grant_offer`);
>   `just test -p codex-tui` (`pf_25_s02`, `security`).
> - [x] Snapshots reviewed and accepted (PF-25 revocation output only).
> - [x] GLM 5.2 tmux run and three videos (revoke one grant, kill switch on and after restart, off keeps the level);
>   Linux clippy on the RTX box; Opus 5.5 High review (three rounds, final APPROVE).

Verbatim, gate record `qa/security-levels/sprints/PF-25-S02/README.md`:

> - **Focused tests (final tree):** `just test -p codex-core` (`security_revocation`, `transition`, `level_change`,
>   `grant_offer`, `aggressive`, `pf_25_s01`, `inspection`): 66/66. `just test -p codex-tui` (`pf_25`, `pf_41`,
>   `security`, `approval_overlay`): 190/190.
>
> - **Linux clippy** (`-D warnings`; security-policy, core, tui, app-server-client, `--tests`) on the RTX box: clean at
>   `4cdcce8f7a` and `cef16ad0e8` (and the final head, below).
>
> - **Review (Opus 5.5 High, installed `corbanu exec`, read-only):** round 1 APPROVE WITH FIXES (3 medium, 7 low/nit),
>   round 2 APPROVE WITH FIXES (low/nit), round 3 **APPROVE** (low/nit, fixed). Each finding is fixed or recorded in
>   `review/disposition.md`.

Verbatim, `qa/security-levels/sprints/PF-25-S02/README.md` lines 28-34:

> - **tmux (GLM 5.2, `-c model_provider="zai"`, disposable homes, `CORBANU_TEST_NO_NATIVE_KEYRING=1`;
>   `.codex-work/workers-20261002/sec-tui8.log`):** under Aggressive (confirmed, restarted): a grant "until it expires"
>   for GLM's hidden-home read is listed by `/security`, `g`; Enter reviews it and Enter revokes it, and the next
>   identical read, approved, gets "Operation not permitted". The kill switch turned on applies at once, `r` restarts,
>   and `/security`, `g` still shows "Kill switch: on"; GLM's next approval offers no grant and the read is denied. GLM
>   turns keep working with the kill switch on. Turning it off opens on "Back" (Enter alone goes back), ↓ Enter turns it
>   off and the level stays Aggressive.

Demo videos (3, index [qa/demos/index/PF-25-S02.md](../../../qa/demos/index/PF-25-S02.md)):

- `pf25s02-revoke-grant`: Revoke one grant from /security: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s02-pf25s02-revoke-grant-be5d33660c7e-2026-10-07.mp4
- `pf25s02-kill-switch-restart`: Kill switch applies now and holds after a restart: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s02-pf25s02-kill-switch-restart-be5d33660c7e-2026-10-07.mp4
- `pf25s02-kill-switch-off`: Turning the kill switch off keeps the level: https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-25-s02-pf25s02-kill-switch-off-be5d33660c7e-2026-10-07.mp4

Carried forward (open, not claimed): Fake financial effect under the kill switch (PF-38-S03/PF-26); mandates; per-agent revocation UI.
