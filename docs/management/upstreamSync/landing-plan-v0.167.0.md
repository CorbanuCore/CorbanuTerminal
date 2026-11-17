# Landing rust-v0.167.0 on main

Status: **not ready to land**. Evidence cut: **2026-11-17 06:30 UTC** (November 16 in Arizona). Finish the contributor-held CLI, settle seven owner decisions, reconcile the ledgers, then qualify one final candidate on all three platforms. Travis retains **“Sprint 3 security sign-off and the final sync merge.”** This document authorizes neither.

This is a routine planning update under [the existing program plan](../upstreamSyncPlan.md), [governed plan](../../plans/active/upstream-sync-rust-v0.167.0.md), and PF-85-S03/S04/S05. Product specification, **Product principles**: “Maintain continuous Codex parity without removing Corbanu-specific behavior.” Tracking: [#1532](https://github.com/CorbanuCore/CorbanuTerminal/issues/1532), [#1628](https://github.com/CorbanuCore/CorbanuTerminal/issues/1628), [S3](https://github.com/CorbanuCore/CorbanuTerminal/issues/1630), [S4](https://github.com/CorbanuCore/CorbanuTerminal/issues/1631), [S5](https://github.com/CorbanuCore/CorbanuTerminal/issues/1632).

## Current state

The inspected sync head is `4d1defcb34a4f938765c5e9d61bf103f2da9d13a` (#2471). Main is `2da8deea6252093fd01aaf51f30751c56b7a80fe`. The pinned upstream target remains `086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a`; do not advance it during landing.

Main merge #9, [#2457](https://github.com/CorbanuCore/CorbanuTerminal/pull/2457), carried main through `ef6b1ca9abe30a06431da59634a3a36e411102f8`. Main has since merged #2456, #2459, #2478, #2492 and #2439. They need the next carry, especially the command-wrapper protections in #2459/#2478 and Windows test scheduling in #2439. Main PR #2493 implements accepted D676–D678; carry it when merged. Recheck both branches before assigning work.

### Measured test baselines

These are the latest located worker measurements, with their actual revisions. **There is no full-workspace, all-platform pass on the inspected head.** Older runs are evidence to carry forward, not passes relabelled onto that head. No new full run was needed for this planning task.

| Suite / platform | Passed | Failed | Ignored / excluded | Revision and dated source |
| --- | ---: | ---: | --- | --- |
| Core `all`, Linux | 3,109 | 81 occurrences, 80 names | 24 ignored; 24 CLI exclusions also recorded, not passes | `880997572351e8d018d39f8aafe2c1e004785a21`, astra-synccore6 round 3, November 17 |
| Core library, Linux | 4,843 | 4 | 10 ignored | Same round 3 |
| App-server suite, Linux | 1,714 | 12 | 2 ignored; 13 CLI exclusions | Same round 3 |
| App-server library, Linux | 464 | 0 | 0 ignored | Same round 3 |
| TUI library, Linux | 7,492 | 0 | 9 ignored | #2440 head `a4dcdbb84672c78aa60400abe653b2bd5baf4ac7`, astra-syncapp2, November 16 20:39 UTC |
| TUI `tests/all.rs`, Linux | 65 | 58 | All 58 failures need the CLI | #2367 head `98068acce3`, sync-compile14, November 16 |
| Exec suite, Linux | 124 | 0 | Not reported in worker summary | #2354, sync-compile10, November 15 |
| Exec seatbelt subset, macOS | 15 | 0 | Not reported in worker summary | #2367, sync-compile14, November 16 |
| Exec-server library / `exec_process`, macOS | 486 / 67 | 0 / 0 | 17 integration cases ignored | #2397 head `1ec71a03830b2431a261b8871cb0aa53f81e5395`, sec-2397b, November 16 20:02 UTC |
| HTTP client, Linux | 159 | 0 | 0 skipped | #2449, astra-2446, November 16 21:51 UTC |
| Sandboxing / Linux-sandbox library + integration, Linux | 117 / 255 + 119 | 0 | Separate suite counts | #2450, sync-main8, November 16 23:00 UTC |

The [astra-synccore6 handoff](https://github.com/CorbanuCore/CorbanuTerminal/issues/1532#issuecomment-6939867211) records the full baseline and subsequent **failure-only** rerun: core **49 pass / 32 fail**, core library **2 / 2**, app suite **5 / 7**. That is **56 / 41** of the original 97 failed occurrences, with no hangs or missing cases. Two additional passing fork-compaction cases are outside that total. Its tested tree `e6411e30f9af03b4d7de2fccac63020a86a49b49` matched sync at `db663c985f156d041a480c708e5455a5963096ba`.

[#2471 then merged](https://github.com/CorbanuCore/CorbanuTerminal/issues/1532#issuecomment-6940024857) at 05:54 UTC: its two enterprise-login cases pass separately, as do 14 keyring cases. Thus **39 previously failing occurrences remain unresolved by the recorded repairs: 22 CLI-dependent plus 17 decision-dependent**. This is a remaining-work count, not a fresh 58/39 run or a reconstructed full-suite result. Exclusions and the separate TUI/MCP suites must still be restored and run.

Supporting records: `$W/astra-synccore6.result.md` (full report), `$HOME/.corbanu-workers/astra-synccore6.result.md` (short handoff), `$HOME/.corbanu-workers/sync-main9.result.md`, `$W/astra-syncapp2.result.md`, and the dated comments linked above. Here `$W` means the coordinator's `workers-20261002` directory. The detailed failure map and raw summaries are archived with astra-synccore6; use them when assigning cases.

## Remaining blockers

### CLI and open sync PRs

Only **two open PRs target sync** at the cut: [#2274](https://github.com/CorbanuCore/CorbanuTerminal/pull/2274) and [#2122](https://github.com/CorbanuCore/CorbanuTerminal/pull/2122), both drafts. #2454 and #2471 have merged.

**#2274 — owner: sync-compile5 / CLI maintainer, with the remaining sandbox row reserved for contributors until about November 21.** That date ends the hold; it is not a promised completion date. The remaining row has no named contributor in the ledger. The coordinator should confirm ownership then, without duplicating contributor work.

- Resolve `cli/src/debug_sandbox.rs` and its D541 MXC contract. With `--sandbox-state-json`, `use_mxc=true` must select MXC and run `refuse_mxc_without_corbanu_protections`; `false` must never select MXC and must refuse if the child's config would select it. Prove the false-state/configured-MXC refusal.
- Merge the current sync base into #2274 and resolve its two ledger conflicts without restoring obsolete pre-split rows. Its other eight CLI conflict rows are prepared; the current head is `1a1042a556907cfeb86a3559d3c9ba7d100cf1f9`.
- Apply **accepted D620: remove the OpenAI desktop-app doctor rows**. This is deferred implementation, not a pending decision. Read and regenerate doctor/help/worktree snapshots with agreed per-binary names and Corbanu wording.
- Replace the diagnostic upstream `debug_sandbox.rs` stand-in with the real resolved file and run actual CLI clippy/tests on macOS, Linux and Windows. The November 15 stand-in run passed library 15/15, but each binary had roughly 7–11 snapshot failures and integrations had **21 failures**: app-server 3, daemon startup 4, daemon package 1, cloud config 1, doctor enterprise/path 4, features 1, login 2, NFS migration 1, nested launch 1, provider accounts 1, worktree 2. These are stale diagnostic counts, neither final-head results nor additive to the core/app failure totals. The nested Aggressive cause was subsequently addressed under D621; rerun it.
- Rerun #2423's `tmux_resume_last_notice_keeps_provider_searched_before_onboarding`, the 58 TUI integration failures, the 22 core/app helper failures and the 24 core/13 app exclusions. Address review follow-ups, including new nested-command classification coverage. Get a fresh exact-head Opus review and corbanu-security approval; the earlier approval does not cover the finished CLI.

**#2122 — owner: sync test-isolation maintainer, after #2274.** It prepares full CLI helpers for `codex-mcp` and `codex-rmcp-client`. Its eight Python wrapper tests pass, but four MCP cases remain CLI-blocked (three `streamable_http_remote` cases and `binding::tests::sandbox_executable_requires_local_stdio_and_a_full_cli`). Merge sync into the draft, qualify the isolated packages on all three platforms, review the exact head and merge. Do not enable eager helper builds while the CLI remains broken. The four cases are a separate package inventory, not additional core-suite failures.

### Owner decisions

[The current queue](https://github.com/CorbanuCore/CorbanuTerminal/issues/1532#issuecomment-6939945244) maps `$W/astra-synccore6-decisions.md` SC6-1…7 to D679…D685. **Travis decides; the coordinator assigns implementation afterwards.** No option below is approved by this plan.

| Decision | Failing occurrences | Choice needed; recommendation |
| --- | ---: | --- |
| D679 / SC6-1 | 2 app | Preserve an explicitly saved low effort on cold resume, or let an edited role's high default override it. Recommend preserving the explicit choice. |
| D680 / SC6-2 | 1 app | Keep frozen base instructions after revert/model change, or rebuild them. Recommend keeping them; a new conversation rebuilds instructions. |
| D681 / SC6-3 | 1 app | Document unsupported Cursor paths, or recover multiple punctuated path components under a hard search bound and reject ambiguity. Recommend bounded recovery, with security review. |
| D682 / SC6-4 | 9 core + 1 core library | Replace unsupported positive tests with fail-closed/hardened-engine matrices, or leave them failing pending separately approved implementations. Recommend replacement, preserving secret-exclusion assertions and the distinct copied-token regression. D530/D353/D354/D640 restrictions remain binding. |
| D683 / SC6-5 | 1 core | Notify only for explicitly delegated work, or inherit an old completion recipient after a queue-only wake. Recommend explicit delegation only. |
| D684 / SC6-6 | 1 core | Send a full current skill catalog on reconnect, or preserve full catalog plus availability history in section-replacing adapters. Recommend the full current catalog; test both provider styles through compaction/resume. |
| D685 / SC6-7 | 1 core library | Keep the copied-token gap when its source is unset, or filter using broker-known credentials with bounded matching and refusal when protection is unavailable. Recommend filtering, with independent security review and Travis's sign-off. |

These are the **17 decision-dependent failures**, grouped by root cause. The remaining **22 CLI failures** are remote executor/plugin/hook/MCP/credential-history startup failures (21 core, one app), not evidence that those security cases passed. D685 is a real protection gap shared with main, not just an upstream expectation mismatch.

Other owner actions: schedule the Windows desktop checklist; sign SG2 and the final SG3/merge. **D676–D678 were accepted at 05:45 UTC** and are being implemented in #2493; the handoff's older pending labels are stale. D659–D674 are accepted; D675 was superseded. D628 deliberately leaves the Windows provisioning service **off**; activation is a separate future decision, not permission to enable it for landing. Deferred named-account flag removal #1080 is likewise outside this sync. No additional undecided D-row was found in the latest coordinator record.

### Other test, CI and tracking gaps

- **macOS is not qualified by Linux green.** astra-syncapp2 records one unchanged Mac-only TUI expectation failure, `tooltips::tests::plans_use_platform_app_tip`, plus historical path/socket/startup and load timeouts. Some passed short-path/focused replays; those replays do not establish a final full Mac pass. Core clippy still had base lints in sync-main9. Assign these to the platform/core maintainers and rerun after fixes.
- **Bazel remains unqualified.** #2471's Cargo fixtures pass, but unchanged `package_name` metadata stopped Bazel before compilation. The integrator must repair the metadata and run the actual lane, including native Windows and nightly coverage. Do not equate skipped sync PR Bazel jobs with a pass.
- **Open issue labels are not a fresh failure inventory.** #2430's native collaboration-schema regression needs an explicit fix/test cross-reference before closure; reconcile it with the 30 synccore6 repairs. #2082 was implemented by contributor #2101 despite remaining open. Likewise reconcile #2270/#2051, #2347/#2356/#2402, #2287, #2308/#2327, #2351/#2370, #2296/#2277/#2312 against their merged fixes and remaining cases. Do not silently count stale issue titles as new failures, or close them solely because another suite passed.
- **Supply chain:** #2349's Zod fix is already in sync via main merge #6. #2305 still tracks four dev/build advisories on its last post-fix audit (three moderate, one low; production-only zero). Re-audit the final lockfiles under SG1, retaining explicit dispositions rather than treating dev dependencies as harmless. #2477 tracks file-descriptor-limit test failures; #1448 remains open after Windows serialization mitigation #2439. Carry their final fixes/evidence as needed.

**Workflow inventory at the recorded SHAs: sync 51, main 47.** Four are sync-only; 24 shared workflow files also differ. The integrator owns this reconciliation, with security review of owned paths.

| Sync-only workflow | Required disposition before landing |
| --- | --- |
| `sync-package-ci.yml` | Remove with temporary sync CI mode. Remove its temporary selectors/configuration too. |
| `python-sdk-build.yml` | Review reusable SDK/runtime build inputs, artifacts and callers; retain only with a recorded Corbanu packaging disposition. |
| `python-sdk-cli-release.yml` | Review `workflow_run: rust-release` and manual publishing triggers, permissions and destinations before it becomes active on main. |
| `rust-release-provisioned-macos.yml` | Review signing/profile pins, candidate flag and caller wiring; record whether retained and disabled or enabled. Do not infer permission to publish from its presence. |

Revert `sync-ci-mode` using the program tooling: `blocking-ci` currently skips Bazel/rust-ci/SDK on sync-base PRs and permits those skips in its aggregate; tmux and TUI matrix also have temporary exclusions. Restore the full gates, then remove only the sync-specific CODEOWNERS block after the final main carry, preserving main's normal security ownership. Check the remaining changes in V8 setup, Rust/platform jobs, SDK/runtime releases, packaging/signing and Windows security probes. The target is correct coverage and safe triggers, not an arbitrary return to 47 files. D623's accepted macOS required-lane promotion waits for its November 17 09:41 UTC observation window and fresh flake audit (#1989); carry the resulting gate when ready.

### Ledger and release-note completeness

At the sync cut the eight live ledgers contain **3,679 rows: 3,583 resolved and 96 open**. Preview ledgers are excluded. These are row counts, not unique defects; several old rows describe fixes that have since merged.

| Ledger suffix after `ledger-rust-v0.167.0` | Resolved | Open |
| --- | ---: | ---: |
| `.tsv` | 1 | 29 |
| `-resolved.tsv` | 1,006 | 0 |
| `-s1c.tsv` | 202 | 0 |
| `-s3b.tsv` | 547 | 27 |
| `-s4a.tsv` | 136 | 0 |
| `-compile.tsv` | 183 | 12 |
| `-main-merges.tsv` | 636 | 18 |
| `-main-merges-2.tsv` | 872 | 10 |

The integrator and security owner must give **every open row** a current disposition, linked implementation and test/review evidence. Work includes the CLI/MXC row and obsolete MCP-server remnants; S3b guardian/disclosure, sandbox/credential and Windows audit/ACL rows; old core/session/test ports; Telegram notification handling; D525 EMA/config/role documentation; remote MCP discovery coverage; and the 28 main-carry rows. A later passing suite alone does not resolve a documented boundary-review obligation. Audit migration renumbering and preserved released checksums too. Preserve split ledgers and the blob-size limit.

There are **140 release-note fragments** on the inspected sync head. Presence is not completeness. Audit user-visible removals, provider/auth behavior, security changes, doctor/CLI changes and D679–D685 outcomes against fragments, then run `release_notes.py check` and `preview`. Never recreate `RELEASE_NOTES.md`. Main merge #9 deliberately omitted the duplicate #2443 fragment because sync's D659–D664 fragment already covers it; preserve that explanation. Finish app-server/config guidance, the upgrade/compatibility record, footprint dispositions and sprint evidence. Do not mark S3/S4/S5 complete from ledger counts alone.

## Sequence and effort

Estimates below are **worker-days of active work**, including focused verification and review preparation, not measured throughput or a deadline. Contributor waiting, CI queues and Travis's availability add elapsed time. The coordinator assigns people within the existing sprint/write-scope limits; this plan creates no new parallelism grant.

| Order | Work and accountable owner | Can run alongside | Estimate / exit |
| --- | --- | --- | --- |
| 1 | Coordinator obtains D679–D685 answers, assigns residuals and confirms CLI hold/ownership; schedules Travis's desktop session. | Read-only ledger/workflow audit and the next main carry. | 0.5–1 day; named owners and recorded choices. |
| 2 | Core/app/security maintainers implement decided cases; platform maintainers reconcile other failures and issue tails. | Disjoint CLI preparation, ledger/docs audit and main carry, within existing limits. | 3–5 days; focused regressions pass without weakening assertions. |
| 3 | CLI owner finishes #2274 after contributor handoff; test-isolation owner then finishes #2122 and restores all CLI-dependent coverage. | Step 2 where files do not overlap. #2122 execution follows CLI build. | 2–3 days after hold; real CLI builds, snapshots reviewed, helper suites green. |
| 4 | Integrator completes ledger/release-note audit, Bazel/workflow repair, required-lane carry and latest-main integration. | Audit starts at step 1; finalization waits for steps 2–3. | 1.5–2.5 days; zero unresolved ledger/re-apply rows, no conflict markers, full CI mode restored. |
| 5 | Security owner assembles S3 parity evidence, final security lanes and code-blind cases; Travis signs **SG2 on the exact candidate**. | Evidence preparation starts early; platform runs may overlap after the same candidate builds. | 2–3 days plus Travis's availability; S3 acceptance recorded before S5. |
| 6 | Integrator/QA qualify the final main merge candidate, publish SG3 diff, collect exact-head Opus review and Travis's final approval. | Independent platform/functional lanes within machine limits. | 2–3 days; every gate below passes. |
| 7 | Travis performs the final guarded merge; coordinator runs Mac/Windows installed canary. | Only sync fixes merge during canary. | 0.5–1 worker-day spread over **48 elapsed hours**, then release readiness decision. |

Total planning allowance: **11.5–18.5 worker-days**, with overlapping preparation. Earliest planning window is roughly **4–7 working days after the CLI hold ends**, plus the 48-hour canary, assuming prompt decisions, available Windows testing and no new boundary defect. This is conditional; November 21 is not a landing commitment.

### Final qualification gates

1. **SG1 and structural integrity:** pinned upstream ancestry and two-parent history preserved; every ledger/re-apply/migration row resolved; repository-wide conflict-marker scan clean; locked metadata, dependency/advisory checks and V8 provenance reverified. Remove temporary CI/CODEOWNERS mode before the final full run. Restore main's ordinary gates, with no expected-red exception on the final PR.
2. **S3 / SG2 security:** record security-level × tool-path × platform parity and fail-before/pass-after evidence for moved boundaries. Run `rust-ci-core-security` on Linux and macOS, `security-platform-contract`, `security-ingress-contract`, `credential-boundary-canary` on all applicable OSes, `windows-security-probes`, Linux sandbox tests on glitch, macOS seatbelt and native Windows sandbox tests/PC checks. Include credential/browser backend isolation, remote executors/MXC, taint and provenance through compaction/resume, hooks, named accounts and D685. Slice approvals are supporting evidence; **Travis signs SG2 against its exact SHA**.
3. **Full final-tree CI:** `blocking-ci` including Windows clippy `-D warnings --tests` and native Windows tests; `rust-ci-full` with full Linux local/remote coverage; Bazel and nightly lanes; cargo-deny, codespell, repository/schema/snapshot checks; `just test` on macOS and Linux clippy with Rust 1.99.1. Run all helper-dependent suites, not just prior failure selectors. Conditional skips must be justified; no failed, pending or cancelled required qualification run is green. Fix failures and rerun affected gates on the changed candidate.
4. **Functional qualification:** fresh-context code-blind design and separate isolated execution of frozen cases, negative isolation probes and positive PTY controls, then independent evidence review. Exercise real keys through success/cancel/failure/recovery/resume for providers/routes, vault, Task Node, spawn/mail, security levels, wallet/GPU without paid spend, and new upstream UI. Run true-TUI workflows in both TensorCash and Isometric Game; archive demos and exact binary hashes. A hosted-model refusal requires the local-model replay path. Unit tests do not replace this gate.
5. **Windows human and VM gaps:** Travis's roughly 30-minute D396/D416/D448 checklist plus step 4b/#1803 is still outstanding; automated evidence and D461/D462 acceptance do not replace it. #1800/#1798/#1851 stay draft-gated; #1798 lands after or with #1800. If included before sync lands, carry and requalify them; otherwise record their deferred scope for Travis's SG2 decision rather than implying they shipped. Separately, PF-23-S04 case 11 remains blocked until a Windows guest exists (D673). The accepted Linux leg (D669), case 7 extra prompt (D671) and case 8 partial disposition (D672) are historical evidence, not an all-platform sync pass. Requalify affected cases on the sync candidate and obtain explicit final dispositions.
6. **Compatibility/release readiness:** back up and upgrade a released state DB, prove the previous released binary can reopen it, and verify vault/archive compatibility. Complete the qualification record from `upstream-integration.md`, user docs/release-note preview, due benchmark evidence and a non-publishing `corbanu-terminal-release` dry run.
7. **SG3 and merge:** final main carry no more than 48 hours before landing. Publish the security-path diff from the SG2-signed SHA to final head; it must be empty or Travis must re-sign. Get independent read-only **Opus 5.5 High APPROVE on the exact head**, applicable corbanu-security approval with matching author/commit/body SHA, all checks green and none pending, and **Travis's own final approval**. Workers never impersonate Travis with a default-account approval. Tag pre-sync main, then Travis merges with `gh pr merge N --merge --match-head-commit <literal-40-character-head-SHA>` after an immediate nonempty head check.
8. **Canary:** 48 hours on the installed Mac and Windows PC candidates before release. D306 holds main merges except sync fixes during this window. For a P0, prepare a reviewed `git revert -m 1` rollback; otherwise fix forward. Preserve the pre-sync tag and DB backup; link the upgrade/canary record from #1532.

### Keeping main moving

Continue normal reviewed, green merges to main while steps 1–5 run. The integrator carries main at useful boundaries, serializing shared manifests/workflows/ledgers and recording any re-apply work; do not merge main into every PR merely because it moved. Security fixes get explicit carry rows and reviews. Refresh overlapping test evidence under the same-test rule. In the final 48-hour preparation window, use the existing soft freeze on hotspot files, not a premature repository-wide freeze. A late security change invalidates the relevant qualification evidence and requires SG3 re-signing. The full main merge hold starts only with the final merge's 48-hour canary.

## Risks

- The held CLI can expose additional Windows/helper failures once real binaries run; the stand-in result cannot predict that total.
- Main drift can reintroduce protection or test gaps. Latest-main carry, per-path ownership and SG3 are essential.
- Green focused repairs can conceal untested combinations. Preserve exclusions, stale baselines, Mac failures and invalid runs until final full qualification replaces them.
- Credential, MXC and Windows desktop boundaries remain the highest-consequence areas. No row count or agent approval substitutes for Travis's reserved sign-offs.
- CI/Bazel/V8 capacity, the Windows guest and human availability can dominate elapsed time. Keep build limits and review budgets; do not trade evidence for a target date.
