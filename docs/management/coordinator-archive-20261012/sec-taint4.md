You are the untrusted-content lane worker (round 4). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
first (binding), then /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-taint3.result.md (PF-30-S02 archived via
#198; PF-30-S03 slice 1 merged as #204, flag `source_envelopes`).

0. First, a tiny separate PR: `config_schema_matches_fixture` fails on current main because #201 left the config
   schema fixture out of date. Regenerate it the repo's way (see codex-rs/AGENTS.md / justfile), confirm the test
   passes, PR, merge when green. Also note in the PR why CI didn't catch it, and if a CI job should have run that test,
   say which.
1. Finish PF-30-S03 "Post-taint authority checks": ties to policy changes, end-to-end tests for memory recall and a
   tricked child agent, approval counts for PF-26, the reviewer's listed test gaps, and the indirect classifier routes
   (`python -c`, symlinks, etc.) as far as practical within this sprint; anything that belongs to PF-23-S01 (MCP calls,
   typing into running processes, code mode) is moved there explicitly. Close and archive PF-30-S03 when done.
2. Then start PF-23-S01 "Moderate ingress and disclosure enforcement" under the same gate if time allows.
Per-sprint gate per sec-common.md. Fresh worktree off origin/main; own CARGO_TARGET_DIR. PR to main; merge when green
(stale CodeQL checks on PRs can be ignored). Update sprint files and index rows. Report as in sec-common.md.
