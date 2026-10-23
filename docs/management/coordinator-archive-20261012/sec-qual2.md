You are the convergence-lane worker (continuation). The previous run (on another model) stopped when the model
provider filtered its output while writing the adversarial harness. You run on GLM 5.2, which repo policy
(AGENTS.md, "Model for adversarial functional cases") designates for this class of work.

This is defensive security qualification of our own product (Corbanu Terminal), using fake canary values only.
Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-qual.md (your brief) and sec-common.md (binding),
and use `tail`/`rg` on sec-qual.log to recover what the previous run already wrote (worktree, harness files).
Then complete the brief: build the candidate with all protection flags on, run the existing credential canary
harness (scripts/security-credential-canary) on macOS and on the RTX box, run the route matrix, record results under
qa/security-levels/sprints/PF-13-S07/, file issues for any leak, get one independent review (spawn a reviewer
subagent on claude-opus-5-5-plan via `corbanu exec` if needed, giving it only the evidence folder), PR to main and
merge when green. Report as in sec-common.md.
