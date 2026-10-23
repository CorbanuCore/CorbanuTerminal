PF-60 is closed. The docs-only PR is **#405**, merged as `508d87e32a` with all checks green: 21 passed, 12 skipped, 0 failed, 0 pending. PR #397 merged first (`d5004cfedf`). I merged #405 with `--match-head-commit c9a6ed4b8c`, and main hadn't moved since then.

**Checker output**

| Check | Result (branch, and main after the merge) |
|---|---|
| `python3 docs/plans/check.py` | `plans: active 2/3; available slots 1` |
| `python3 docs/sprints/check.py` | `sprints: current 104; archived 155`, no errors |
| Checker unit tests (plans 6, sprints 24) | OK |
| `git diff --check`, broken relative links in touched files, key scan | clean, 0, 0 hits |

No checker refused anything. Slot 2 is free for PF-84. There were no conflicts with the PF-84 worker: main had no PF-84 changes when I merged.

**What changed**
- **S04 archived** the same way as S05: moved to `docs/sprints/archive/portfolio-agent-cost-accounting/` with status `completed`. Its "Closure — 2026-10-10" section has Travis's acceptance and the not-verifiable items. It links `qualification.md`, the independent acceptance README and REVIEW, and PRs #382, #383, #385, #393 and #397. It also has a table mapping each known limit to its issue and follow-up sprint.
- **Waiver (a) lifted:** a dated addendum under S05's Closure records it; the original text is unchanged. Waiver (b) is carried to S10.
- **Plan completed:** it is now `docs/plans/completed/main-2026-10-10-portfolio-agent-cost-accounting.md`. Like P0 security, it closed without a release, because collection is still developer-only. It has a closing note, and its sprint map, true-TUI, live-repository, human-acceptance, release and completion sections are updated.
- **Archived records and links:** I pointed the plan link in the archived S01–S05 records to the new path, since the sprint checker requires it. I also fixed the two broken links in S03 that the S05 archive left behind.
- **Indexes and navigation:** the plans index (slot 2 marked free), the active, completed and proposed indexes, the sprints index, `mkdocs.yml` and two old portfolio links.

**New follow-up records** (drafts only, nothing started)

The plan is `docs/plans/proposed/portfolio-agent-cost-accounting-followups.md`. The sprints keep the PF-60 feature ID, the same way PF-13-S07 carried over into P1 security. Each sprint has acceptance criteria.

| Sprint | Covers | Issues |
|---|---|---|
| PF-60-S06 Side-conversation and temporary-fork attribution | (i), (ii) | #400, #401 (new) |
| PF-60-S07 Price catalogue refresh | (iii); the Sol promotion item must land before 2026-11-21 | #368 |
| PF-60-S08 `/cost` page clarity and reopen warm-up cost | (v), (vi), (vii) | #396, #403 (new) |
| PF-60-S09 Verification paths for unverified flows | (iv) ChatGPT login, image generation, realtime, TensorCash | #402 (new) |
| PF-60-S10 Isolated code-blind acceptance of the follow-ups | (viii), S05 waiver (b) | #404 (new) |

I commented on #368 and #396 to point them at S07 and S08.

**Decisions Travis will need when the follow-up plan is activated** (all recorded in the plan)
- **Activation:** which free slot it gets, and when.
- **OpenRouter pricing (S07):** price each request from the charge OpenRouter reports; without one, either show the catalogue rate labelled as an estimate or show "no price".
- **Sol price deadline (S07):** whether the Sol item can land early as a standalone fix if the plan isn't active before 2026-11-21.
- **Reopen warm-up charge (S08):** either avoid it or show it. I recommend showing it plus a setting to turn it off, because avoiding it makes the first reply slower.
- **Accounts for S09:** a dedicated ChatGPT test account in the vault, and TensorCash access (or a decision that it doesn't apply).

**Left open**
- **Unassigned accounting issues:** #352, #359, #325, #324, #314, #290, #127 and #129 are still open. Travis didn't assign them to a follow-up sprint, so the plan lists them for him to place.
- **Image generation:** #370 has since offered image generation to OpenAI API-key users, so it may already be testable with an API key. S04's acceptance didn't re-check it; #402 notes this.
- **Old paths:** the research and QA evidence files and `scripts/initiative_control/coordinator.py` still name the old `active/` path. I kept the evidence files as history, as the S05 archive did, and didn't touch the script because the PR is docs-only. The script was also left alone when P0 closed.
- **Cleanup:** my worktree is removed and there was no build output. Git refused a normal delete of my local branch, and I didn't force it.