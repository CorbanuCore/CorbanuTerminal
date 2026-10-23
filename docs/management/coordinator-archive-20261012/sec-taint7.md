You are the untrusted-content lane worker (round 7). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
first (binding), then sec-taint6.result.md (PF-23-S01 archived via #233).

1. Small separate PR first: memory summaries fail with GLM 5.2 at every level because GLM wraps its JSON answer in a
   code fence and the parser rejects it. Make the parser accept a single fenced JSON block (```json ... ``` or ```
   ... ```), with tests; no other loosening.
2. PF-23-S02 "Aggressive deny and grant enforcement"
   (docs/sprints/current/p0-security-levels/pf-23-s02-aggressive-deny-and-grant-enforcement.md), including the items
   moved into it: the Codex Apps upload check-then-read race (read must go through the sandbox), Aggressive grants,
   command-text gaps for writes, and processes started before untrusted content. Then PF-23-S03 if time allows
   (PF-24-S02 in the TUI lane is waiting on PF-23-S03).
Per-sprint gate per sec-common.md (GLM 5.2 is available again). Linux clippy on the RTX box. PR to main; merge when
green. Report as in sec-common.md.
