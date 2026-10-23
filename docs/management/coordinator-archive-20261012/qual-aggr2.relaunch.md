RECOVERY NOTE (coordinator, 2026-10-12T11:07:29Z): your previous run of this brief hung at ~11:03Z when a transient macOS
access error broke the claude-plan auth helper (not your fault). Your transcript is /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/qual-aggr2.log.died-20261012T1107 — read its
last ~500 lines and grep it to recover state. The isolated execution you started is STILL RUNNING on its own:
/Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-rerun-20261011/h/run_all.sh (tmux socket aggressive-rerun-20261011/tmux.sock,
session runall; progress in runs/seq.log; it was on G27 at 11:10Z). Do NOT start it again. Wait (sleep in ~5-minute steps,
cheaply) until runs/seq.log says "RUN-ALL done", then continue from where you were: independent evidence check (fresh agent),
check.py, the docs-only records PR, plan rows, stop the "macOS 3" VM with utmctl, and the plain-language sign-off request.
Claude credits are low this week and the coordinator is doing an orderly pause: be economical, no optional extras, leave
nothing uncommitted (records PR merged or open), and report as in sec-common.md.

You are the integrator for the RE-RUN of the P1 security milestone **"Aggressive ships"**. Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding), qual-aggr.md and qual-aggr.result.md in
that folder (the first run's brief and result), the milestone record on main at
qa/security-levels/milestones/aggressive-20261010/ (README, frozen design, results, evidence index), the run harness in
/Volumes/CorbanuDrive/Corbanu/.codex-work/aggressive-ms-20261010 (build.sh, candidate/, gate/, common/), root AGENTS.md
("Code-blind functional test design", "Model for adversarial functional cases" incl. the GLM 5.3 Flash amendment), and
qa/code-blind-functional/ (README.md, isolated-execution.md, executor-prompt.md, RECORD_TEMPLATE.md, check.py).

First run (#447): 17 pass, 15 fail, 7 blocked; sign-off held. Since then the defects #434–#445 and Travis's decisions were
implemented and merged (among others #449 #453 #454 #456 #458 #459 #463 #464 #470 #499 #520 #521 #522 #526 #537 #541 #542).
Owner decisions that change expected behaviour vs. the first run's design (Travis, 2026-10-11): protected reads are never
grantable under Aggressive, not even once, and the denial is explained at the point of failure (#442/#464); protections
apply immediately on confirmation except the restart-only list named on screen 07 (#443/#453, #505); the kill switch refuses
model turns, interrupts the current turn in every pane incl. Claude panes, and queued agent messages wait (#445/#456/#507);
listing the Corbanu home is denied under Aggressive (#506); `.env` files created after start are protected and
`.codex`/`.agents` are always read-only (#520); the credential broker stays one-way, so "restart now" is not offered when a
key was handed to the broker (#436/#449). Get the exact list from the issues' decision comments.

Goal: re-run the milestone end to end on a fresh candidate and bring Travis an honest sign-off request.
1. Candidate: current origin/main — record the exact commit; build the packaged macOS candidate with the harness's build.sh
   (adapt paths only). Do heavy builds where the harness expects; the Mac is busy, be patient rather than parallel.
2. Design: REUSE the frozen design of the first run (do not redesign). For cases whose expected result was changed by an
   owner decision above, do not edit the frozen file: record an amendment table (case id, old expectation, decision +
   issue link, new expectation) written and frozen with SHA-256 BEFORE execution, reviewed by a fresh read-only agent
   (installed `corbanu exec`, Opus 5.5 High) against the decision comments only. Add new cases only for the decisions'
   new behaviours (kill-switch interrupt incl. Claude panes if the guest has Claude Code ≥ 2.1.292 — else record blocked;
   home listing; `.env` created mid-session), designed by that same fresh agent from a user-intent packet, frozen likewise.
3. Isolated execution in the existing macOS guest "macOS 3" (agent@192.168.64.3) per isolated-execution.md. UTM is open
   on the host and both VMs are stopped; start ONLY "macOS 3" with `/Applications/UTM.app/Contents/MacOS/utmctl start
   "macOS 3"` once the candidate is built, keep "macOS 2" stopped, and stop "macOS 3" again when done
   (`utmctl stop`). Never delete or recreate a VM: erase guest data programmatically for a fresh run (Travis's standing
   instruction). Re-measure the isolation gate first (source hosts denied, inference reachable, native-prompt guard,
   negative probes). Product under test on GLM 5.3 Flash (`-m glm-5.3-flash -c model_provider="zai"`); the ZAI key reaches
   the guest only through the owner-approved path used in the first run. Fresh executor context per case group. Record
   every refusal verbatim.
4. Evidence check by an independent fresh agent (not the executor); run qa/code-blind-functional/check.py.
5. Records: a docs-only PR to main with the re-run record (qa/security-levels/milestones/aggressive-<date>-rerun/ or where the
   procedure says), plan row updates, results.json, amendments and evidence index; key scan before commit. Merge only when
   ALL checks are green, never with checks pending (`gh pr merge N --merge --match-head-commit <sha>`; never force-push;
   never rm -f). File issues for defects found; do not fix product code yourself.
6. Do NOT mark the milestone passed: it needs Travis's named sign-off. End with a plain-language sign-off request: passes,
   failures, blocked/out-of-scope cases, comparison with the first run per case, and known limits (Windows/Linux
   applicability, #505 restart-only items, open issues).
If a step needs an owner action (VM login, guest password, network), stop that step and report exactly what's needed
(password only via `corbanu vault auth-helper` on a consuming command if a label exists). Keep context lean. Report as in
sec-common.md.
