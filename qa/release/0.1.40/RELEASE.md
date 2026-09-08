# Corbanu Terminal 0.1.40 emergency release

The operator explicitly instructed repairing the account mismatch, testing it, and pushing an emergency release on 2026-09-07. Codex is executing that instruction. This release repairs Task Node agent identity propagation and provider refusal handling.

## Behavior

- Shell and unified execution inherit the active terminal's home and named/default profile, including when shell snapshots or configured overrides contain stale values.
- Task Node helpers reject absent, malformed, or conflicting agent scope before opening account credentials. Direct human CLI use retains explicit profile selection and the default profile when no scope is inherited.
- Installed Unix launchers preserve an explicitly inherited home.
- Structured `misalignment_policy_violation` responses stop immediately on HTTP/SSE and WebSockets. They do not trigger reconnect or transport fallback, and later stream tool items are discarded. The displayed error retains the provider refusal and explains that this flag does not establish a user policy violation.
- The provider's policy stop is respected. This release cannot clear provider-side restrictions or promise resumption of a stopped conversation.

## Classification and authority

Profile/home propagation belongs to the existing PF-45 Campaign Tracker initiative, [PF-45-S02](../../../docs/sprints/archive/p0-security-levels/pf-45-s02-agent-profile-scope.md). Product heading **Campaign Tracker — LOCAL PILOT CANDIDATE**: “preserve attributable user prompts and compact GLM 5.3 Flash summaries in a bounded database”. The user explicitly authorized this emergency repair.

Provider refusal handling is a bounded reliability repair under **Product principles**, “Visible control: Show what an agent can read, disclose, propose, approve, sign, and broadcast”, and preserves the existing provider refusal boundary. The implementation follows [OpenAI's stopped-request guidance](https://developers.openai.com/api/docs/guides/safety-checks/misalignment-monitoring#handle-a-stopped-request).

Source worktree: `/home/pfrpc/repos/worktrees/corbanu-release-0.1.39`; branch `fix/tasknode-agent-profile-scope`; base `9b71d86d7f`. The worktree name records its original release, while the candidate package version is 0.1.40.

## Evidence at dispatch

- Scoped Clippy/fix passed for API, core, and CLI. The API pass was repeated after final stream termination and typed retry-delay parsing changes.
- `just fmt` and `git diff --check` passed.
- Final `just test -p codex-api -p codex-cli -p codex-core -p codex-tasknode-session --lib --bin corbanu --test all -E 'package(codex-api) | package(codex-tasknode-session) | test(tasknode_) | test(misalignment) | test(responses_retry) | test(shell_command_handler_to_exec_params)'`: **228 passed**, 3,725 excluded by the explicit filter.
- Tests exercise distinct linked fixture accounts and the default account, malformed/missing/conflicting scope, both shell paths with stale overrides, fatal HTTP and WebSocket frames with varied text, compaction with a positive retry budget, and suppression of tool output after a policy stop.
- Five installer tests and the provider-path no-regex regression passed. Portable skills mirror check passed. Bazel lock update passed after refreshing the versioned Cargo lockfile; no external dependency versions were changed.
- The known affected production recording was paused after checking the exact account/workspace, then the real TUI confirmed recording OFF, 26 retained events, and zero pending uploads. No historical records were moved or deleted.
- The final built candidate's interactive qualification is in progress. It will compare read-only live account status in the TUI and agent helper with local model fixtures in disposable TensorCash and Isometric Game worktrees. No real-model qualification is claimed at dispatch.

Private local logs and QA captures remain under `/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-0.1.40-qa` and the adjacent `corbanu-hotfix-*.log` files. They are not public release assets.

## Remaining qualification

The full competitor/model benchmark matrix is incomplete, with no qualifying-cycle reset. Historical sprint-checker failures include duplicate feature identifiers and other active allocations; those are not represented as a pass. Full cross-platform interactive acceptance, a historical account-write audit beyond the identified workspace, and a separate named-human acceptance record remain unavailable. The requesting human's explicit release authorization applies; these gaps are disclosed rather than hidden.

Publication and installed-candidate evidence will be updated after the workflow and interactive checks finish.


## Final local qualification and delivery — 19:43 UTC

Immutable release code: `d42240f40c2a84be03561288aba4e1b9c3036b9f`, pushed to `main`. [Publication run 34155825393](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/34155825393) is building all five platforms with publication and latest-release selection enabled. Input validation passed. Public publication is **pending**, not claimed as complete.

The Linux dev candidate and all Rust support executables were built from 0.1.40, packaged with the existing pinned bwrap/rg/zsh resources, and installed atomically at 19:38 UTC. Current package: `/mnt/HC_Volume_101713660/pfrpc/build-cache/corbanu/candidates/0.1.40-d42240f40c/package`. Core executable SHA256: `48857cace3fcf10c376f044599a49fb9b184c6f9a2d5f12685af64b960b36bb2`. The previous 0.1.39 package and launcher backups are preserved. Already-running terminal processes retain their old executable until restarted.

Actual PTY keys exercised `/tasknode`, then a fixture-directed read-only status helper through unified execution for the default account and shell execution for the named account. Tab and helper identities agreed. Cold restart/resume repeated the same checks successfully. A structured policy-stop fixture issued exactly one request and displayed terminal guidance without reconnect/fallback. All model fixtures were local; these runs are not presented as real-model coding benchmarks.

Repository inputs:

- Canonical TensorCash: fetched `https://github.com/agtico/tensorcash.git`, base `9325ed67d23355170d6ad38ad58ea776d049ae4e`, disposable worktree `.../corbanu-0.1.40-qa/tensorcash-canonical`.
- Isometric Game: `https://github.com/goodalexander/isometricgame.git`, base `59821b7a85524f186f946c4670480c7ee96483cb`, disposable worktree `.../corbanu-0.1.40-qa/isometricgame`.
- Additional initial TensorCash fork check: `https://github.com/postfiatorg/tensorcash.git`, base `dd6e92024254090de0f596b090bd5c74c4d97b90`. It was followed by the canonical-origin check above.

The installed visible command separately passed both live account lookups, rejected missing and conflicting agent scope, and rejected an empty alternate home instead of falling back to the real account vault. Installed operator guidance was updated with backups retained.

One harmless request through the user's configured provider returned `READY` with no tool calls in 4.67 seconds (19,294 input tokens, five output tokens). This demonstrates ordinary request access; it is not an account-enforcement audit and does not restore the stopped conversation. Provider dollar cost and GitHub Actions cost are unavailable.

Private artifacts: `pty-results.json`, `canonical-pty-results.json`, `installed-helper-checks.json`, `installation.json`, `provider-health-result.json`, and the PTY captures under the private QA directory. The public 0.1.39 release notes now disclose the account-scope defect while 0.1.40 builds.

User service `corbanu-emergency-release-0140.service` monitors publication, retries a recognized transient build failure once, checks the published tag against the tested commit and the Linux archive against its checksum, and installs the official package when available. It preserves a newer user-installed version and records completion or failure in `.../corbanu-0.1.40-qa/publication-status.json`. The local dev candidate lives outside the official release directory so the official installer can replace it correctly. No claim is made that this future installation has already occurred.
