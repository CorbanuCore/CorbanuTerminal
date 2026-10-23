You are the untrusted-content lane worker (round 6). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
first (binding, incl. disk hygiene), then sec-taint5.result.md in the same folder (PF-23-S01 slice 1 merged as #223).

1. Finish PF-23-S01: slice 2 (memory summarisation under Moderate when the input is labelled and tied to its source
   session) and slice 3 (block reads of home and credential files at the OS sandbox level under Moderate/Aggressive,
   replacing the remaining command-text gaps; plus the activation readiness matrix). Slice 3 overlaps PF-29 (sec-mig
   worker, credential migration incl. ~/.ssh and shell profiles): read docs/sprints/current/p0-security-levels/pf-29-*
   first, keep write scopes disjoint, and record the split in both sprint files. Update the index row. Close and
   archive PF-23-S01 when its gate is complete.
2. Then PF-23-S02 "Aggressive deny and grant enforcement" if time allows.
Per-sprint gate per sec-common.md. Run affected crates' tests locally and Linux clippy on the RTX box before merging.
PR to main; merge when green (stale CodeQL checks can be ignored). Report as in sec-common.md.
