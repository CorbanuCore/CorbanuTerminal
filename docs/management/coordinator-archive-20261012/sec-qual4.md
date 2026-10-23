You are the convergence-lane worker (round 4). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
(binding: keychain rule, disk hygiene) and sec-qual3.result.md (PF-13-S07 merged as #242; issue #239 filed).
1. Fix GitHub issue #239 (the files-route read gap found by the qualification), own PR, with a regression test and the
   per-fix gate; merge when green.
2. Re-run the PF-13-S07 route matrix with a SAVED security level, once under Aggressive and once under Moderate (all
   protection flags on, disposable homes, fake canaries only), on macOS and on the RTX box. File and fix/hand off any
   leaks as before. Add the results to qa/security-levels/sprints/PF-13-S07/, get one independent read-only review of the
   new evidence (installed `corbanu exec` on claude-plan), PR, merge when green, and archive PF-13-S07 if its gate is
   complete.
Report as in sec-common.md.
