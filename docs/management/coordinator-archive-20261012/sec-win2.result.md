Both Windows limits are fixed and merged to main, and PF-27-S08 and PF-27-S09 are planned. PF-27-S07 is now `blocked`, like PF-27-S06, only because the GLM 5.2 tmux run and the videos need a real Windows machine.

## What shipped
All PRs were merged with merge commits and `--match-head-commit` once green.

| PR | What |
| --- | --- |
| #280 | The PF-27-S07 sprint record, plus draft records for PF-27-S08 (broker under its own restricted token or AppContainer, including Credential Manager access) and PF-27-S09 (Windows model auth through the broker). Both drafts have acceptance criteria, code boundaries, a test plan and a "needs a real Windows machine" precondition. To place S07–S09 directly after S06 in the plan, I moved S06 from order 45 to 42; S07–S09 are 43–45. |
| #281 | **`CODEX_HOME` deny removal.** With the flag off, Corbanu removes exactly the deny we added, plus the copies files inherited from it. Every other ACL entry stays the same, compared as SDDL before and after. A lock file stops this from running while another session is using the protection. |
| #282 | **New threads protected from creation.** In Core and the broker, every Rust thread now gets the protected DACL the moment it is created. The broker also changes its own default DACL, which covers every thread it starts, whoever starts it. |
| #284 | **Broker never openable.** Core now starts the broker already protected, and keeps it suspended until its default DACL is set. |
| #296 | Evidence in `qa/security-levels/sprints/PF-27-S07/`, including all 7 review texts. Since #279 activated the P1 plan with this worktree registered, S07 is now `blocked`. I also added a "merged behind flag" note to the PF-27-S06 record so S07 could take that status. |

## Gate evidence
- **New-thread probe (the one you asked for):** a test process holds brand-new threads open before their protection would normally apply. A same-user process and a process under the sandbox's restricted token are both denied context read, context write and suspend. As a control, a thread created another way can still be opened in that moment, which shows the probe really reaches it.
- **Broker probes:** a broker started the new way is fully denied, both while still suspended and after it runs. As a control, the same process started the old way is readable.
- **Windows tests** (`windows-2022` CI runner), green on every merged head:
  - Slice 1 (#281, job 113293258039): 4 sandbox, 9 core, 24 broker tests.
  - Slice 2a (#282, job 113280283309): 12 process-hardening, 24 broker, 5 core tests.
  - Slice 2b (#284, job 113292935422): 18 process-hardening, 24 broker, 5 core tests.
- **Linux clippy** (`-D warnings`, RTX box) is clean for every slice; it caught a Linux build error in slice 1 before merge. The macOS suites pass.
- **Opus 5.5 High reviews:** all slices ended at APPROVE (slice 1 after 3 rounds, the others after 2).
- **Not done:** the GLM 5.2 tmux run and the videos, which need a Windows machine.

## Problems found during the work (all fixed)
- A thread created already protected has no right to change its own DACL. The existing per-thread protection step counted that as a failure; it now treats it as "already protected".
- A broker started already protected couldn't rerun its own hardening, so the whole broker test suite failed on CI. Hardening now accepts permissions it reads back as being at least as strict.
- An existing ACL helper treated an inherited write deny as a read deny, so a test's deny was silently skipped. The test now uses a helper that only counts explicit entries.

## Needs Travis
1. **Accept the remaining limits:**
   - In Core, threads that Windows or other DLLs start are still protected only once they begin running, and the loader's own worker threads not at all. Core can't use the default-DACL fix the broker uses, because its own pipes and child processes would inherit it and break.
   - Protected threads can't impersonate. Nothing in Core or the broker does today.
   - After upgrading from a PF-27-S06 build, restart protected sessions before turning the flag off.
2. **Decide how the vault key reaches a confined broker (PF-27-S08).** Such a broker can't read Credential Manager, so something else has to pass it the key:
   - **(a) A one-shot helper reads it and hands it to the broker** (my recommendation, because Core never holds the key).
   - (b) Core sends it.
   - (c) Grant the broker access, which isn't possible with an AppContainer.
3. **A real Windows machine** for the tmux run and videos, and for the Credential Manager parts of S08 and S09.

I removed my other worktrees, build folders, Docker image and volumes, and the scratch folder on the RTX box. I kept `worktrees/pf-27-s07-20261008` because the active P1 plan registers it; no owner settings reference anything I removed. The short result is in `.codex-work/workers-20261002/sec-win2.result.md`.