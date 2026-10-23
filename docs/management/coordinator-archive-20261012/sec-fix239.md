You are a security bug-fix worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding:
keychain rule, disk hygiene), then sec-qual3.result.md and the tail of sec-qual4.log (use tail/rg; it is long). The
previous worker (GLM) started fixing GitHub issue #239 (files-route read gap from the PF-13-S07 qualification) and
stopped when the Z.AI account ran out of credit; recover its worktree/branch from the log and continue it (don't redo
finished work). Finish the #239 fix with regression tests and the per-fix gate (focused tests, Linux clippy on the RTX
box, one independent Opus 5.5 High review; skip GLM runs, Z.AI is out of credit). PR to main, merge when green, close
#239. Do NOT do the Aggressive/Moderate qualification re-run; that waits for Z.AI credit. Report as in sec-common.md.
