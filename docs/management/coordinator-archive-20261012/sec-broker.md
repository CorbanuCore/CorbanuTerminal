You are the broker lane worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first.

Sprint: PF-27-S04 "Isolated credential broker process"
(docs/sprints/current/p0-security-levels/pf-27-s04-isolated-credential-broker.md). Existing work:
branch feat/security-broker-resume-20260911 (worktree /Volumes/CorbanuDrive/Corbanu/worktrees/security-broker-resume-20260911,
base d870c92dab), plus the handoffs the sprint links. Service stage reviewed; native bootstrap pending.

1. Inventory what is already on that branch versus main and what of the sprint's Remaining list is done.
   Don't redo accepted work. Bring the work onto current main in a new branch (merge or cherry-pick; no force-push).
2. Finish the Remaining items under the per-sprint gate in sec-common.md (focused tests incl. the named pf_27_s01
   tests, GLM 5.2 tmux functional test of the user-visible effect, one independent review, short demo videos).
   The old review-budget and system-preflight micro-allocations are superseded by decision 5.
3. Ship behind the feature flag if it changes default behaviour. PR to main; merge when green.
4. Then continue the lane: PF-27-S02 next, same gate, if time allows. Update the sprint files and index rows.
Report as in sec-common.md.
