I merged the first part of PF-30-S02 to main behind the `source_envelopes` flag (PR #190, merge commit `7b2a04ea41`). The sprint stays `in_progress` because the rest of its checklist is still open.

**Housekeeping (folded into PR #190):**
- PR #178's merge commit is now in the PF-30-S01 QA file.
- PF-30-S01 is marked completed and archived, and the plan, index, mkdocs and reconciliation links are updated.
- PF-30-S02 is allocated, with its worktree, branch, base `b96b23344b` and write scope recorded in the sprint file and the plan front matter.
- The sprint checker passes. It already allows five concurrent sprints and this made four, so no adjustment was needed.

**What shipped, with the flag on:**
- **Resume keeps standing:** each message's origin is saved in the session file (digests only, no content) and restored on resume and fork. Old session files without these records, or with unreadable ones, are treated as untrusted.
  - A saved record can never raise the standing of something already recorded in the session.
  - Origins are saved again at each compaction point, so a resume that starts from there still has them.
- **Compaction:** the user's kept messages keep their standing. The summary is trusted only if everything it summarised was trusted; one tool output keeps it labelled.
- **Memory:** the memory summary is now its own message and reaches Moderate and Aggressive labelled as untrusted memory data.
- **Error message fix (separate commit `836ec41e04`):** with the flag off, Moderate now says it stopped because `source_envelopes` is off, instead of "registry is full or poisoned".

With the flag off, session files and requests are unchanged, and a test checks that no origin records are written.

**Gate evidence:**
- **Focused tests:** `just test -p codex-core pf_30_s0` gives 62 passes, including 12 new `pf_30_s02` tests, before and after merging main.
- **Nine affected crates:** 5,052 of 5,056 tests pass.
  - Two skills tests fail because of skills installed on this machine; they also fail on main.
  - `remote_compact_trim_estimate_uses_session_base_instructions` fails the same way on a clean `b96b23344b` checkout.
  - One shell-snapshot test failed while the machine was busy and passes when run alone.
- **CI:** all 29 checks passed.
- **GLM 5.2 tmux run (real keys):** resume, a planted memory summary, and `/compact` after tool output all behaved as expected.
  - One compaction video take got a wrong answer from GLM: it said the summary had no label. The session file showed the summary had no saved standing, so it was sent labelled. A reworded prompt passed.
- **Opus 5.5 High review:** round 1 asked for changes. The main issue was that a resume starting at a compaction point lost the saved origins. I fixed that and five smaller items, and round 2 approved. The only later change makes the overflow warning fire once. Reviews are in `.codex-work/workers-20261002/pf30s02-review{1,2}/`.
- **Videos (at `01a3a88330`, listed in `qa/demos/index/PF-30-S02.md`):**
  - [Resume keeps standing](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-30-s02-pf30s02-resume-keeps-standing-01a3a88330b9-2026-10-06.mp4)
  - [Memory labelled](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-30-s02-pf30s02-memory-labelled-01a3a88330b9-2026-10-06.mp4)
  - [Compaction summary labelled](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-30-s02-pf30s02-compaction-summary-labelled-01a3a88330b9-2026-10-06.mp4)
  - [Flag-off message](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-30-s02-pf30s02-flag-off-message-01a3a88330b9-2026-10-06.mp4)
- **Records:** the gate record is `qa/security-levels/sprints/PF-30-S02/persistent-origins-gate.md`. A docs-only PR #194 (merged, `476f42dbe3`) adds PR #190's merge commit to it and to the sprint record.

**Still open:**
- **Rest of PF-30-S02:**
  - Lineage across sub-agents and their messages.
  - Export and import.
  - Tying the memory summarisation step to the session's security level.
  - Capacity tests for stored origins, and checking content against its stored digest on read.
  - Keeping taint after a one-off approval.
  - A real test of resuming a paginated thread or referenced fork; today that path is only simulated in a unit test.
  - Moving the token-budget MCP thread hint out of trusted host context.
- **Known limits, disclosed in the gate record:**
  - With the flag on under Permissive, the memory summary is a separate message (same text, different layout).
  - Saved origins are only as trustworthy as the session folder; anything that can write `CODEX_HOME` can forge them.
  - A message containing only an image is matched by role alone (unchanged from PF-30-S01, now also saved).
  - Once a summary is labelled, GLM sometimes re-answers earlier questions after `/compact`.
- **Unrelated to this change:** the remote-compaction test failure also happens on main and needs an owner.