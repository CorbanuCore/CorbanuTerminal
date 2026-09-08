# Campaign Tracker qualification — 2026-09-06

Local implementation and interactive qualification for PF-45-S01. No production deployment, employee enrollment, history import or sharing change was performed.

## Candidate

- Worktree: /home/pfrpc/repos/CorbanuTerminal
- Branch: fix/tasknode-profile-isolation
- Base: ec549c0c687f50a682487e9d68289c05557ff579
- Build: codex-rs/target/debug/corbanu, version label 0.1.36 (dirty development candidate).
- Final SHA-256: 8e6b66e67c1156f6faa2f0d1c366c03bfb6ff601bf9d709b7169ded8b67cb333.
- The installed debug launcher was preserved. The included local corbanu-debug wrapper executes the rebuilt candidate with an isolated home and loopback fixture origin. The actual PTY command was **corbanu-debug --yolo**.
- Detailed logs, fixture responses and the temporary home are under /mnt/HC_Volume_101713660/pfrpc/scratch/campaign-tracker-qa.

Both repositories already contained unrelated changes. This record identifies the tested candidate; it does not claim that the entire dirty tree belongs to this task.

## Automated checks

- just fix -p codex-tasknode-session -p codex-tui: passed; existing unrelated wallet, update and IPC/test warnings remain.
- just fmt: passed.
- just test -p codex-tasknode-session -p codex-tui, filtered to the session package and tasknode/campaign_tracker/agent_control tests: **33 passed**, 3,851 skipped. This is the affected-workflow suite, not the full workspace suite.
- npm run campaign-tracker-smoke: **38 PostgreSQL checks passed**, using a disposable schema and the actual migration/repositories.
- npm run campaign-tracker-contract-smoke: passed. Covers exact Flash routing, entitlement, typed mapping, no model fallback, rejection of unsupported completion claims, and an AST prohibition on tracker LLM-path regex.
- Migration registration: 138 ordered, unique files passed.
- Route authentication policy: 174 policies / 151 registered literal paths passed.
- Active-plan checker: passed, 2/2 active plans. The global sprint checker has pre-existing duplicate PF-31–35 IDs, stale reservations and overlapping scopes. Diagnostics are preserved in sprints-check.log; other owners' records were not rewritten.

The new overview snapshot was reviewed and accepted, followed by clean runs without snapshot-update mode. Interactive checks exposed and repaired repeated vault writes, stacked refresh views, mutation refresh feedback and recording-indicator visibility.

## Actual PTY results

The TUI ran in a real tmux PTY with RUST_LOG=trace and a dedicated log directory. Prompt text and Enter were sent separately. The loopback server uses real PostgreSQL and tracker handlers with explicitly fake identities, entitlement and inference. These fixture results are integration evidence, not production inference claims.

1. Opened Task Node → Campaign Tracker and enabled the repository. The UI disclosed prompt preservation and separate sharing permission.
2. Submitted an exact Unicode prompt. PostgreSQL preserved it with repository URL, branch, commit, dirty state, actor, model, timestamp and turn identity. Assistant output persisted only as a compact summary.
3. Forced summary-provider failure. Two human prompts produced six durable events, with exactly one pending summary source retained in the encrypted outbox.
4. Restarted the terminal and restored service. The same six IDs remained; both summaries became ready; the outbox drained. No duplicate prompts or runs appeared.
5. Opened the read-only replay pager and inspected the prompt and observed metadata.
6. Granted historical prompt/replay/review access through the TUI. Collaborator replay succeeded; export remained unavailable. Revocation made the next replay return HTTP 403. The menu refreshed immediately and displayed inactive grants.
7. Started, paused and cleared an actual /goal. Many automatic continuation turns left exactly three human prompts at that checkpoint and zero verified task completions. Goal lineage and execution duration were present.
8. Submitted a final ordinary prompt and completed five human-review dimensions with a rationale. PostgreSQL confirmed five scores of 4, the exact rationale, source revision and prompt-quality-v1 rubric.
9. Created a personal campaign through the TUI and confirmed immediate list refresh.
10. Paused recording and submitted another prompt. It did not enter the tracker; the human total stayed at four. Menu dismissal returned cleanly to the composer. On the final binary, enabling and immediately pausing recording removed REC from the composer status line without waiting for the timer (13-immediate-pause-indicator.txt).

Numbered text panes and HTTP assertions are on the data volume: 01-recording through 12-paused-composer; before-restart.json, after-restart.json, shared-access.json, revoked-access.json, goal-metrics.json, prompt-review.json and paused-check.json.

## Live Flash check

A separate real subscription-backed call used corbanu/glm-5.3-flash, resolved as zai/glm-5.3-flash. Given an observed test exit code of 1 and a contradictory assistant completion claim, it returned **failed** and **unmapped**. The operational credential stayed in memory and was not written to evidence. No substitute model was used.

## Storage and limits

The Cargo target symlink remained on /mnt/HC_Volume_101713660; builds used two jobs. Root free space remained about **376 GiB**. The data volume retained about **7.9 TiB free**. A trace-heavy nine-second goal test grew its diagnostic log to about 406 MiB; the session was stopped and the log compressed to about 15 MiB. Ordinary tracker recovery storage itself used only tens of KiB. QA files totaled about 137 MiB. Shared data-volume usage rose from roughly 399 to 432 GiB during build/test; this is volume-level accounting, not an isolated measurement of this build. No global Cargo cleanup or unrelated deletion was performed.

This is a local pilot candidate. Production encryption configuration, deployment, backup/restore integration, broader capture coverage, advanced campaign/export UI and pilot calibration remain outside this qualification. See [implementation and rollout boundaries](/home/pfrpc/repos/tasknodeofficial/docs/campaign-tracker-implementation.md). Fixture timing and one live model call do not establish production throughput, mapping precision or complete employee activity.
