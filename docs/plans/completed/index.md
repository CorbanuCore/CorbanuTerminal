# Completed plans

This directory is the durable record of released product initiatives.

A completed filename uses `<release>-<slug>.md`. Its front matter declares
`status: completed`, and its evidence tables link the final automated, true-TUI,
live-repository, human-acceptance, documentation, release, and benchmark
records. Merged code without those records is not completed work.

| Plan | Closed | Note |
| --- | --- | --- |
| [P0 `/security` levels](main-2026-10-08-p0-security-levels.md) | 2026-10-08, Travis's decision | Exception: closed without a release. Work is on main behind default-off flags; milestones, PF-13-S07 and release linkage moved to the [P1 plan](../active/p1-security-hardening.md#carried-forward-from-p0). The prefix names the tree, not a release. |
| [PF-60 accounting](main-2026-10-10-portfolio-agent-cost-accounting.md) | 2026-10-10, Travis's decision | Exception: closed without a release. Collection and `/cost` are on main, developer-only. S01–S05 accepted and archived; accepted gaps, release linkage and the isolation gate moved to the [follow-up plan](main-2026-10-20-portfolio-agent-cost-accounting-followups.md) (draft then; active from 2026-10-14; completed 2026-10-20). The prefix names the tree, not a release. |
| [Unified provider onboarding (PF-84)](main-2026-10-14-unified-provider-auth.md) | 2026-10-14, by the coordinator's worker to free slot 2 for Travis's 2026-10-14 decision to start the PF-60 follow-ups; his confirmation of the closing is pending | Exception: closed without a release. PF-84-S01–S06 accepted and archived; named accounts stay behind the default-off `named_accounts` flag. Flag-removal prerequisites moved to the draft [flag-removal plan](../proposed/named-accounts-flag-removal.md). The prefix names the tree, not a release. |
| [Accounting follow-ups after PF-60](main-2026-10-20-portfolio-agent-cost-accounting-followups.md) | 2026-10-20, when its last item (the follow-up live run, D25/D3/D16) was recorded, per the plan's completion list | Exception: closed without a release; `/cost` stays developer-only. S06–S10 accepted and archived (2026-10-14 to 2026-10-16). The follow-up live run on Tart ([record](../../../qa/portfolio/agent-cost-accounting/followups-live-20261020/README.md)): 18 passed, 2 failed, 5 blocked of 25; Travis's acceptance pending. New issues #1038, #1039 and #1045 are not carried here. The prefix names the tree, not a release. |
