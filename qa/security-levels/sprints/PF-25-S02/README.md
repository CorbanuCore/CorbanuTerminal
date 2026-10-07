# PF-25-S02: per-sprint gate (2026-10-07)

- **Branch:** `feat/pf-25-s02-revocation-tui`, stacked on PF-25-S01 (`feat/pf-25-s01-grant-tui`, with main and
  PF-41-S01 merged in). Behind
  `security_levels`. `/security` offers `g` only when Core enforces a protected level, the kill switch is on, or grants
  are held, so Permissive sessions see nothing new.

## What each choice does

| Choice | Review | Applies | Saved | Released by |
| --- | --- | --- | --- | --- |
| One grant | command, runs left, expiry, session | now (this process) | no (grants are memory-only) | — |
| Revoke all active authority | what ends | now: grants, "for session" approvals, broker channels, in every tree of the process on the home | `security_state.json` (never comes back) | — |
| Kill switch on | what it refuses until turned off | now, as above, plus new grants, protected actions after untrusted content, browser, memory writes | yes; holds after restart | "Turn the kill switch off" |
| Kill switch off | opens on "Back"; ↓ then Enter | now, also in other sessions holding the same switch | yes | — (the level stays) |

All commits go through PF-23-S03's `commit_transition` (`core/src/security/revocation_change.rs`), off the UI thread,
only while Core's state is still what the review showed. Kill-switch changes made within one second of the one in
force are dated a second later, so a quick off after on is not ignored. A revocation never stores a level of its own.

## Results

- **Focused tests (final tree):** `just test -p codex-core` (`security_revocation`, `transition`, `level_change`,
  `grant_offer`, `aggressive`, `pf_25_s01`, `inspection`): 66/66. `just test -p codex-tui` (`pf_25`, `pf_41`,
  `security`, `approval_overlay`): 190/190.
- **Linux clippy** (`-D warnings`; security-policy, core, tui, app-server-client, `--tests`) on the RTX box: clean at
  `4cdcce8f7a` and `cef16ad0e8` (and the final head, below).
- **tmux (GLM 5.2, `-c model_provider="zai"`, disposable homes, `CORBANU_TEST_NO_NATIVE_KEYRING=1`;
  `.codex-work/workers-20261002/sec-tui8.log`):** under Aggressive (confirmed, restarted): a grant "until it expires"
  for GLM's hidden-home read is listed by `/security`, `g`; Enter reviews it and Enter revokes it, and the next
  identical read, approved, gets "Operation not permitted". The kill switch turned on applies at once, `r` restarts,
  and `/security`, `g` still shows "Kill switch: on"; GLM's next approval offers no grant and the read is denied. GLM
  turns keep working with the kill switch on. Turning it off opens on "Back" (Enter alone goes back), ↓ Enter turns it
  off and the level stays Aggressive.
- **Review (Opus 5.5 High, installed `corbanu exec`, read-only):** round 1 APPROVE WITH FIXES (3 medium, 7 low/nit),
  round 2 APPROVE WITH FIXES (low/nit), round 3 **APPROVE** (low/nit, fixed). Each finding is fixed or recorded in
  `review/disposition.md`.
- **Videos (GLM 5.2):** revoke one grant, kill switch on and after a restart, kill switch off keeps the level
  (`qa/demos/index/PF-25-S02.md`).

## Known limits

- With `security_levels` off there is no `/security` picker, so a saved kill switch cannot be turned off from the
  TUI. Under Permissive with no grants and the switch off, `g` is not offered (product decision for Travis).
- A review taller than the pane starts at its end (its choices stay visible).

- A financial effect submitted before the kill switch is not exercised here (PF-38-S03 / PF-26).
- No mandates are issued yet, so none are listed; Core's `Actor` revocation has no UI.
- `/security` lists grants of every session of the process, including ended ones, until they expire.
