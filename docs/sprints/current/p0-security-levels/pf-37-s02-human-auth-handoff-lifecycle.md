---
sprint_id: "PF-37-S02"
title: "Human authentication handoff and session revocation"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-37"
execution_order: 62
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-37-S01, PF-25-S02"
created: 2026-08-28
updated: 2026-08-28
---

# PF-37-S02 — Human authentication handoff and session revocation

## Execution mandate

- Deliver: Credentialed browsing remains origin-bound and human-controlled across challenges, cancellation and resume.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-37).
- Feature: `PF-37`.
- Product citation: **Non-negotiable controls** — “Permit agents to reference credentials only by label; resolve them solely inside the trusted execution boundary.”
- Acceptance advanced: Credentialed browsing remains origin-bound and human-controlled across challenges, cancellation and resume.
- Sources and archive disposition: [PF-37 reconciliation](../../../plans/security-source-reconciliation.md#pf-37).

## Code boundaries

- OpenClaw adoption reference: [OC-2](../../../plans/openclaw-source-review-2026-08-28.md#oc-2), [OC-8](../../../plans/openclaw-source-review-2026-08-28.md#oc-8) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: codex-rs/tui/src/bottom_pane/approval_overlay.rs; PF-19 revocation.
- Planned: codex-rs/tui/src/bottom_pane/browser_auth_handoff.rs; codex-rs/web-retriever/src/auth_session_lifecycle.rs.
- Tests: planned colocated Rust test modules prefixed `pf_37_s02`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] Active plan; PF-37-S01, PF-25-S02 completed and archived.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Test challenge cancellation, browser/engine restart, session replacement and revocation of already-open credentialed channels; no stale authenticated profile/handle may survive a new run's human handoff.

- [ ] Present origin, credential label, purpose, session duration and permissions on trusted UI; require human consent and show challenge-required without sending OTP/MFA/passkey input to the model.
- [ ] Keep MFA/CAPTCHA/passkey/payment steps human-only on an isolated trusted surface; no host-profile import, challenge bypass or recording of sensitive keystrokes.
- [ ] Expire and revoke cookies, sessions and pending browser grants on cancel, kill, downgrade, origin change or run end; restart requires revalidation rather than replay.
- [ ] Resume ordinary screened reads only within the approved session scope; denied/unsupported login offers safe manual guidance without silent fallback.
- [ ] Exercise login success, Esc, human challenge, wrong origin, revoked session, crash and restart with fake credentials and actual TUI keys.
- [ ] Add named `pf_37_s02` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-tui pf_37_s02 && just test -p codex-web-retriever pf_37_s02 && just test -p codex-secret-broker pf_37_s02`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: required: request fixture login → review origin → cancel → approve → human challenge → revoke → restart denied.
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-37-S02/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
