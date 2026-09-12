# PF-80-S01 offline decision projection candidate — 2026-09-12

## Parent correction, September12 09:40 UTC — supersedes original code hashes below

First actual Astra helper review exited1 with two P2 findings; both reproduced
by two new regression tests (five failing assertions before correction).
Resolution now rejects changed/intervening closed context while permitting
unchanged acknowledgment; canonical output is bounded before persistence.
Final full Python suite100passed2.980s; whitespace passes. Source SHA256
decisions.py:566c1053a94cb80ea76ce9548ced7d43da3957983b88a7b35c234341bfba8a25;
test_decisions.py:e5a2976e8f9eec33e766130fc53569ff36aa1e11f589eefc704574e4e8441301.
Integrator adds one scoped correction-review pass beyond prior+3; prior design
and code passes remain consumed, future evidence pass retained. No live wiring.

Uncommitted worker handoff for parent review; whole sprint remains in_progress.
Product initiative: **Internal delivery control — TO BUILD**, “Show blockers, rendered sprints, human test plans, machines, run logs and freshness.” Active plan: `docs/plans/active/initiative-delivery-control.md`; sprint: `docs/sprints/current/initiative-delivery-control/pf-80-s01-delivery-control.md`.
Worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-pf80-s01-20260911`, branch `workstream/tasknode-pf80-s01-20260911`; verified clean launch and unchanged HEAD `b8da69d744aa65ea68e5b71c199d401922f84602`. Plan/sprint allocation base remains `87e31f521672e627e6230d48fc16a4cfaa7ff44c`; launch is the manager-authorized descendant, not a rewritten base.
Applied corbanu-terminal-development; read root policy, active plan/current sprint, allocated packet, decision amendment and frozen design. Shared plan/sprint ledgers remain manager-owned under the explicit five-file allocation.

## Implemented contract and use

`decisions.validate(raw, now)` accepts bounded JSON bytes or a JSON object and returns a detached complete feed; schema 1 binds feed_id/revision/assessed_at to stable decision IDs with append-only revision snapshots. `test_decisions.fixture()` documents every field. Required identity/question/choices are strict; optional unknown context is null. Evidence retains its own assessment timestamp. Canonical digest uses sorted compact UTF-8 JSON; input is bounded to 1 MiB, summaries to 300 bytes and other strings to 2000 bytes. Capacity errors reject; no history pruning occurs.
`save_fixture(directory, raw, expected_digest, now)` requires an existing canonical owner-only directory, stores only `decisions.fixture.json`, and serializes cooperating writers with `.decisions.fixture.lock`. It rejects symlinks/nonregular or nonprivate files, CAS conflicts, source changes, rollback and history rewrites/deletion. Exact retries retain identity/assessment. New question revisions may follow a recorded resolution while retaining that resolution. Resolution data never applies operational authority.
`load_fixture` returns None only for a missing fixture; corrupt/unreadable input raises a content-free error. File fsync, atomic replace and directory fsync are attempted. Tested pre-replacement failures retain identical last-good bytes; a post-replacement failure is an uncertain outcome requiring reload, not proof of power-loss durability. Locking assumes cooperating writers and a trusted private directory, not hostile same-user filesystem mutation.
`attention.render_decisions(raw, now, sprints, documents)` is an unconnected pure HTML fragment. Caller supplies the approved sanitized published corpus and exact sprint identity/path mappings. Missing references get unavailable-context anchors; historical PF-76-S01 links only to its explanation. Existing attention functions are unchanged. Assessment becomes stale strictly after 1200 seconds; record/evidence age remains separately qualified after a newer feed assessment. No rendering refreshes source times.
The schema excludes raw-log fields and rejects unsafe paths, controls, common token patterns and synthetic secret/raw-private-log canaries. This is not a general detector for arbitrary secrets disguised as ordinary prose: manager sanitization/allowlisting remains required before any feed wiring. No corpus is read or network link fetched by these functions.

## Final code evidence

Pinned interpreter: `/Volumes/CorbanuDrive/Corbanu/.codex-work/initiative-control.oGQGyA/venv/bin/python` (all test invocations use `-B`).
Commands: `-m unittest discover -s scripts/initiative_control -p test_decisions.py -v` **9 passed**; same command with `test_attention.py` **14 passed** (8 unchanged pre-existing cases); with `'test_*.py'` **98 passed**. Complete-object assertions cover reload/projection/lifecycle; spawned-process contention yields one success and one CAS conflict; no tests removed or skipped.
Governance: `docs/plans/check.py` passed (3/3 active); `docs/sprints/check.py` passed (115 current, 126 archived); `git diff --check` passed. Existing suite messages about activation/publication are fixture-test output, not service/deployment evidence.

| Changed path (repository relative) | Added / removed | SHA-256 of final code |
| --- | ---: | --- |
| scripts/initiative_control/decisions.py | 249 / 0 | `93480359fba0b7cc98024b3821ad8db68078f1b288fd05da61a9a0c442ab1a10` |
| scripts/initiative_control/attention.py | 83 / 0 | `1864a87bdba10e8b47792345f06c2ec5049df13bcbc3989227ac58440e11f267` |
| scripts/initiative_control/test_decisions.py | 211 / 0 | `787431c8f9e743f14a62e6529147f5241542560412f6da44ed9e7c5737307311` |
| scripts/initiative_control/test_attention.py | 113 / 0 | `656237b5569fbc48b3404f65ee4fe379a40b5978dd31ef2c8572395bede79482` |

Code/test subtotal 656 changed lines, 332 non-test; receipt lines add to both totals. Final receipt hash and exact total are supplied in the worker return (avoids a self-referential hash). Parent must include its own allocation edits in integration accounting under 800 total / 500 non-test.
Frozen original design SHA-256 `c4f95f0123b1a28603a3ec67214d029e597a20fb1d5d46853a77c23ecad89c6d`; allocated packet SHA-256 `d9a69ec0134283eb6dd5a31a96056af65b8e75d46878b5187dfd9658053a9906`. Neither was edited.

## Frozen-case accounting

Original Volta task `01a094c9-1364-7783-9493-9900d25f6fae` used no tools, code or results; its intent-only design and ambiguities are preserved unchanged. Packet clarifications supply 20-minute assessment freshness, append-only retention, unknown/rejected input semantics, published-corpus boundary, and later 1440x900/390x844 inspection targets. Browser choice and actual inspection remain for parent qualification.
Below, R means `test_attention.DecisionRenderingTests.test_`; D means `test_decisions.DecisionTests.test_`. “Supporting” means automated first-slice evidence only: no original browser/full-feed case is declared fully passed or waived.

| Frozen case | First-slice evidence / remaining disposition |
| --- | --- |
| DEC-001 | Supporting R`dec001_005_full_context_exact_summary_links_and_purity`; full-page placement/discoverability deferred to wiring/browser. |
| DEC-002 | Same R: full context and deterministic unchanged records; actual expand/collapse/reopen deferred. |
| DEC-003 | Same R: exact two-sprint links and similar identity fixture; actual published navigation/return deferred. |
| DEC-004 | Same R: stopped versus continuing text; away-from-computer readability deferred. |
| DEC-005 | Same R: explicit options/question/recommendation; human interpretation remains unverified. |
| DEC-006 | R`dec006_007_019_notices_separate_acknowledged_unresolved`; bookkeeping/acknowledgment notices remain question-free; full feed classification deferred. |
| DEC-007 | Same R and D`all_lifecycle_states_and_exact_answer_revision`; acknowledged remains in open queue. |
| DEC-008 | R`dec008_014_freshness_unknown_and_mixed_age`; explicit fresh empty wording; dashboard inspection deferred. |
| DEC-009 | Same R and D`unknown_empty_stale_boundary_and_new_assessment`; missing/invalid unknown, no invented assessment. |
| DEC-010 | Same R: stale feed and stale stopped/continuing context; browser inspection deferred. |
| DEC-011 | Same R: stale empty distinct from fresh empty; full-feed refresh deferred. |
| DEC-012 | Same R and D`append_only_persistence_retry_reload_and_rollback`; repeated renders/retries retain original timestamps. |
| DEC-013 | Same R and D`unknown_empty_stale_boundary_and_new_assessment`; newer assessment clears only feed warning; real import/refresh deferred. |
| DEC-014 | Same R: unknown owner/impact and independently stale evidence/context; actual browsing deferred. |
| DEC-015 | R`dec015_018_resolution_anchors_history_and_no_actions`, D`append_only_persistence_retry_reload_and_rollback`; stable anchors/history, saved-link browser reopening deferred. |
| DEC-016 | Same R: mixed open/resolved/superseded queue membership and inspectable history; navigation deferred. |
| DEC-017 | Same R: explicit Slack not connected/offline wording; no integration or live activity evidence. |
| DEC-018 | Same R and purity test: no input/form/button/script, no answer mutation; real interaction deferred. |
| DEC-019 | R`dec006_007_019_notices_separate_acknowledged_unresolved` and all 8 original AttentionTests; complete-page coexistence deferred. |
| DEC-020 | R`dec020_historical_identity_and_missing_context` plus original legacy/missing/modern tests; published history navigation deferred. |
| DEC-021 | R`dec021_escape_and_reject_canaries_even_in_retained_history`, D`schema_time_bounds_references_and_safe_errors`; full export/ingest/redaction and browser destinations/accessibility inspection deferred. |
| DEC-022 | Deferred: actual desktop browser 1440x900, long/multiple-item layout, links and expand/collapse. No browser pass claimed. |
| DEC-023 | Deferred: keyboard focus/order, native details operation and link navigation. Semantic HTML alone is not keyboard proof. |
| DEC-024 | Deferred: actual 390x844 phone viewport, readability and operability. No phone pass claimed. |
| DEC-025 | Deferred advisory: multi-owner/long-evidence scan and readability. Preserved, not deleted or waived. |
| DEC-026 | This receipt discloses offline-only evidence and all outstanding inspection; parent independent evidence check remains required. |

## Handoff limitations and next ownership

No collect/export/serve/main-UI wiring, real decision file, native changes, credential access, flags, Slack/network calls, posting, deployment, commits/pushes, extra agents or reviews performed by this worker. No native/TUI, TensorCash/Isometric, benchmark or release qualification is claimed; the active plan marks those repositories inapplicable to this internal document projection. Named-human acceptance is absent. Unit tests and model review cannot prove deployment, live delivery or human acceptance.
Integrator's additional three-pass allowance remains design used, candidate code/evidence review pending, retaining earlier usage; parent reviews once and handles substantive corrections. Later actual feed/Slack and routine progress writeback are manager-owned under Travis's priority update, only after qualified validity/enrollment/mapping/redaction/idempotency/recovery. The next useful allocation is the complete sanitized feed/export/import/render/health chain, then separately bounded Slack/dispatch and live qualification; this candidate enables none of them.
