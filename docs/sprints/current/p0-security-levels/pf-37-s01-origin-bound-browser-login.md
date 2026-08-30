---
sprint_id: "PF-37-S01"
title: "Origin-bound brokered browser login"
status: draft
plan_file: "docs/plans/active/p0-security-levels.md"
plan_feature: "PF-37"
execution_order: 61
owner: "Jim Ricketts"
worktree: "/Users/travisgood/Documents/ChatGPT/corbanu-security-levels"
branch: "feat/p0-security-levels"
base_commit: "7cc15ae0762664d6d01765de407329887da9f876"
depends_on: "PF-31-S03, PF-28-S02, PF-30-S03, PF-34-S03"
created: 2026-08-28
updated: 2026-08-28
---

# PF-37-S01 — Origin-bound brokered browser login

## Execution mandate

- Deliver: A model can request an approved login without seeing credentials or inheriting the user's browser profile.
- Excludes: adjacent feature implementation, Permissive policy changes, and unlisted integrations.

## Plan linkage

- Plan: [P0 `/security` levels](../../../plans/active/p0-security-levels.md#pf-37).
- Feature: `PF-37`.
- Product citation: **Non-negotiable controls** — “Permit agents to reference credentials only by label; resolve them solely inside the trusted execution boundary.”
- Acceptance advanced: A model can request an approved login without seeing credentials or inheriting the user's browser profile.
- Sources and archive disposition: [PF-37 reconciliation](../../../plans/security-source-reconciliation.md#pf-37).

## Code boundaries

- OpenClaw adoption reference: [OC-1](../../../plans/openclaw-source-review-2026-08-28.md#oc-1), [OC-8](../../../plans/openclaw-source-review-2026-08-28.md#oc-8), [OC-9](../../../plans/openclaw-source-review-2026-08-28.md#oc-9) at `13adff02ca3897768d80d2bca18f5acf08c55d91`; see the review for named functions, callers, tests and limits. Reference tests are not candidate evidence.

- Existing/foundation: PF-27 credential broker; PF-31 isolated retrieval; PF-17 bounded grants.
- Planned: codex-rs/secret-broker/src/browser_login.rs; codex-rs/web-retriever/src/authenticated_session.rs.
- Tests: planned colocated Rust test modules prefixed `pf_37_s01`; fixtures use synthetic secrets and fake services only.

## Preconditions

- [ ] All dependencies in front matter are completed and archived; plan remains active.
- [ ] Read root and nearest implementation-path AGENTS.md; verify exact plan/worktree coordinates.
- [ ] Confirm source pins, declared crate/module paths, and backend/API availability; unresolved security prerequisites block readiness.
- [ ] Human owner records one permitted HTTPS login origin, non-production test account and reviewed form contract; unavailable origin/account blocks qualification, not source-based planning.

## Done

- [x] New single-feature record reconciled with current ownership and archived design input; no implementation claimed.

## Remaining

- [ ] Require completed PF-34-S03/PF-35-S03 screening and quarantine integration before login readiness; test reflected credential DOM/error plus classifier unavailable/forced-allow and preserve source lineage end to end.

- [ ] Keep CDP/control-plane trust and public-page destination policy separate; prove exact-origin login cannot expose cookies, profile files or raw credentials to public retrieval, arbitrary evaluate or model-selected selectors. Browser helper review is not login qualification.

- [ ] Separate credentialed browser sessions from public retrieval workers and host profiles; broker stores password/cookies/session tokens in encrypted ephemeral session state.
- [ ] Define one typed login operation for an explicitly human-approved exact HTTPS origin and fixed adapter-reviewed form; agent may name label/origin but cannot supply arbitrary selectors, scripts or credential destination.
- [ ] Validate top-level and frame origin, navigation, form action and TLS/connection policy immediately before fill/submit; redirects to other origins invalidate the grant.
- [ ] Return only screened status/content through PF-28/34/35; mask credential fields and block cookie/storage export, clipboard, screenshots containing secrets and reflected DOM/error leakage.
- [ ] Qualify a deterministic one-origin login fixture, malicious iframe/form-action cases, redirects, reflected passwords and stale sessions; unsupported real origins remain denied pending reviewed adapters.
- [ ] Qualify the same bounded adapter on the recorded permitted origin with test-only credentials; capture safe origin/result evidence, never credentials or human challenge keystrokes. Other origins remain denied.
- [ ] Add named `pf_37_s01` regression tests; update affected Cargo/Bazel/lock/schema edges together without broadening this feature.

## Verification

- [ ] Run `cd codex-rs && just fix -p <affected-crate>` for each listed crate, then `just fmt`; inspect the final diff.
- [ ] Focused: `cd codex-rs && just test -p codex-secret-broker pf_37_s01 && just test -p codex-web-retriever pf_37_s01`; confirm tests actually ran.
- [ ] Integration: full affected crate suites via `just test -p <affected-crate>`; update Bazel locks when manifests change.
- [ ] TUI applicability: none; integration flows are re-run by PF-26-S02
- [ ] Record candidate/commit, commands, expected/actual outcomes and safe artifact digests; no production credentials or funds.

## Exit evidence

- [ ] Implementation commit and final-tree outputs under `qa/security-levels/sprints/PF-37-S01/`.
- [ ] Acceptance and source-mapping assertions proven; applicable true-TUI keys/checkpoints captured after formatting.
- [ ] PF-26 final-candidate and both-live-repository requalification remains mandatory; no release-complete claim here.
- [ ] Done/Remaining reflect reality; completed record moved to the archive and plan/navigation updated.
