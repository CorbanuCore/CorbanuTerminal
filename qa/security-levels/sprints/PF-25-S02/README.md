# PF-25-S02: per-sprint gate (2026-10-07)

- **Branch:** `feat/pf-25-s02-revocation-tui`, stacked on PF-25-S01 (`feat/pf-25-s01-grant-tui`). Behind
  `security_levels`. `/security` offers `k` only when Core enforces a protected level, the kill switch is on, or grants
  are held, so Permissive sessions see nothing new.

## What each choice does

| Choice | Review | Applies | Saved | Released by |
| --- | --- | --- | --- | --- |
| One grant | command, runs left, expiry, session | now (this process) | no (grants are memory-only) | — |
| Revoke all active authority | what ends | now: grants, "for session" approvals, broker channels, in every tree of the process on the home | `security_state.json` (never comes back) | — |
| Kill switch on | what it refuses until turned off | now, as above, plus new grants, protected actions after untrusted content, browser, memory writes | yes; holds after restart | "Turn the kill switch off" |
| Kill switch off | opens on "Back"; ↓ then Enter | now | yes | — (the level stays) |

All commits go through PF-23-S03's `commit_transition` (`core/src/security/revocation_change.rs`), off the UI thread,
only while Core's state is still what the review showed. Kill-switch changes made within one second of the one in
force are dated a second later, so a quick off after on is not ignored. A revocation never stores a level of its own.

## Results

- **Focused tests:** see the sprint record's Verification.
- **Linux clippy:** see below.
- **tmux (GLM 5.2):** see below.
- **Review (Opus 5.5 High):** see `review/`.
- **Videos:** `qa/demos/index/PF-25-S02.md`.

## Known limits

- A financial effect submitted before the kill switch is not exercised here (PF-38-S03 / PF-26).
- No mandates are issued yet, so none are listed; Core's `Actor` revocation has no UI.
- `/security` lists grants of every session of the process, including ended ones, until they expire.
