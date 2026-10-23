You are the Task Node submission worker for round 4. Read tn-sec4.result.md and tn-verify3.result.md in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/, the round-4 kit README and scripts in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261009/tn-sec4/, and the tasknode-usage skill
(/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/.system/tasknode-usage/).
Travis ACCEPTED SECACCT-TN-01..08 in Task Node (2026-10-09). Status changes since the evidence was drafted — update the
evidence files (and regenerate with the kit's generator if that's how they're built) BEFORE submitting:
- PF-27-S08: Travis decided the key path on 2026-10-09 = option (c), the broker reads Credential Manager itself (as on
  macOS/Linux). Another worker is recording that in the sprint record now; describe it as decided, with the known limit
  that a compromised broker could read the user's other generic credentials. Don't claim S08 is archived.
- #351 is fixed by PR #358 (merged); #354 (#345) is merged; #356 (#323) may still be open — check and state current truth.
- evidence-06.md was already hand-corrected by the coordinator (reel approved by Travis, Breeze terms confirmed) — keep that.
- PF-60-S05: still awaiting Travis's acceptance unless `gh`/the sprint record shows otherwise — say so honestly.
Then run ./poll.sh to confirm all eight read Accepted, ./submit.sh, follow each receipt's next step, and poll every 5 minutes
(up to 3 hours) answering any verification request exactly as asked with fresh command output (respond.sh; extend it
minimally if a request needs an attachment). Never print secrets; scan answers for key material. Report: per task the
status, what was asked/answered, and reward amounts.
