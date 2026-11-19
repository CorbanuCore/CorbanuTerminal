# Landing rust-v0.167.0 on main

Status: **not ready to land**. Evidence cut: **2026-11-17 06:30 UTC** (November 16 in Arizona). Finish the contributor-held CLI, settle seven owner decisions, reconcile the ledgers, then qualify one final candidate on all three platforms. Travis retains **“Sprint 3 security sign-off and the final sync merge.”** This document authorizes neither.

This is a routine planning update under [the existing program plan](../upstreamSyncPlan.md), [governed plan](../../plans/active/upstream-sync-rust-v0.167.0.md), and PF-85-S03/S04/S05. Product specification, **Product principles**: “Maintain continuous Codex parity without removing Corbanu-specific behavior.” Tracking: [#1532](https://github.com/CorbanuCore/CorbanuTerminal/issues/1532), [#1628](https://github.com/CorbanuCore/CorbanuTerminal/issues/1628), [S3](https://github.com/CorbanuCore/CorbanuTerminal/issues/1630), [S4](https://github.com/CorbanuCore/CorbanuTerminal/issues/1631), [S5](https://github.com/CorbanuCore/CorbanuTerminal/issues/1632).

## Current state

The inspected sync head is `4d1defcb34a4f938765c5e9d61bf103f2da9d13a` (#2471). Main is `2da8deea6252093fd01aaf51f30751c56b7a80fe`. The pinned upstream target remains `086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a`; do not advance it during landing.

Main merge #9, [#2457](https://github.com/CorbanuCore/CorbanuTerminal/pull/2457), carried main through `ef6b1ca9abe30a06431da59634a3a36e411102f8`. Main has since merged #2456, #2459, #2478, #2492 and #2439. They need the next carry, especially the command-wrapper protections in #2459/#2478 and Windows test scheduling in #2439. Main PR #2493 implements accepted D676–D678; carry it when merged. Recheck both branches before assigning work.

### Measured test baselines

The fresh astra-synccore7 Linux measurements below began on **November 17, 2026**, at sync head `51dc00e7067b3e32d167e0fcb681cd9249209f22`, using Rust 1.99.1, an 8 MiB test stack and isolated test homes on glitch. Follow-up runs are labelled separately with their actual revisions. **There is no full-workspace, all-platform pass.** CLI exclusions, ignored tests and interrupted cases are not passes.

| Suite / platform | Passed | Failed | Ignored / excluded | Revision and dated source |
| --- | ---: | ---: | --- | --- |
| Core `all`, Linux | 3,169 | 24 | 24 ignored; 24 CLI exclusions | `101e9ff5845f67b419edcf1e28658768f1d0b99e`, astra-synccore7 round 4, November 17 |
| Core library, Linux | 4,876 | 0 | 10 ignored | `51dc00e7067b3e32d167e0fcb681cd9249209f22`, astra-synccore7 round 1, November 17 |
| App-server suite, Linux | 1,725 | 1 | 2 ignored; 13 CLI exclusions | Same round 1; remaining selected-capability-stack case depends on #2274 |
| App-server library, Linux | 464 | 0 | 0 ignored | Same round 1 |
| TUI library, Linux | 7,490 | 2 | 9 ignored | Same round 1; long-path snapshot and Unix-socket fixture failures |
| TUI `tests/all.rs`, Linux | 65 | 58 | All 58 failures need the CLI | #2367 head `98068acce3`, sync-compile14, November 16 |
| Exec integration / library / binary, Linux | 127 / 83 / 1 | 0 / 0 / 0 | 0 ignored; event-processor tests are included in integration | `51dc00e7067b3e32d167e0fcb681cd9249209f22`, astra-synccore7 round 1, November 17 |
| Execpolicy library / integration, Linux | 8 / 33 | 0 / 0 | 0 ignored; binary target has no tests | Same round 1 |
| Exec seatbelt subset, macOS | 15 | 0 | Not reported in worker summary | #2367, sync-compile14, November 16 |
| Exec-server library / `exec_process`, macOS | 486 / 67 | 0 / 0 | 17 integration cases ignored | #2397 head `1ec71a03830b2431a261b8871cb0aa53f81e5395`, sec-2397b, November 16 20:02 UTC |
| HTTP client, Linux | 159 | 0 | 0 skipped | #2449, astra-2446, November 16 21:51 UTC |
| Sandboxing / Linux-sandbox library + integration, Linux | 117 / 255 + 119 | 0 | 3 Linux-sandbox cases ignored; separate suite counts | #2450, sync-main8, November 16 23:00 UTC |

The [dated astra-synccore7 measurement record](https://github.com/CorbanuCore/CorbanuTerminal/issues/1532#issuecomment-6949219793) records the exact revisions and root causes. The initial core run on `51dc00e7067b3e32d167e0fcb681cd9249209f22` aborted after **316 passes**, leaving **2,901 unfinished**, when a network-approval future overflowed the standard 8 MiB stack. The complete core result above uses [#2532](https://github.com/CorbanuCore/CorbanuTerminal/pull/2532)'s repair and has no unfinished cases. Its **24 failures** comprise **21 CLI-dependent cases**, two D634 metadata fixtures ([#2530](https://github.com/CorbanuCore/CorbanuTerminal/pull/2530), final-head focused replay **2/0**) and a shared permission-hook marker ([#2535](https://github.com/CorbanuCore/CorbanuTerminal/pull/2535)).

Together with the app-server case, **22 measured failures remain in the #2274 CLI group**; the 24 core and 13 app exclusions are additional unexecuted coverage. The TUI startup snapshot needs normalization before a long path is truncated ([#2533](https://github.com/CorbanuCore/CorbanuTerminal/pull/2533)); its socket fixture passes unchanged when the harness uses a shorter temporary-path alias. These focused follow-ups do not reconstruct a passing full-suite count. No new owner decision emerged from this measurement campaign. The earlier [astra-synccore6 handoff](https://github.com/CorbanuCore/CorbanuTerminal/issues/1532#issuecomment-6939867211) and its 17 decision-dependent failures describe the earlier evidence cut, not this fresh failure inventory.

Supporting records: `$HOME/.corbanu-workers/astra-synccore7.result.md`, `$W/astra-synccore7-measurements.json`, and the dated comment linked above. Here `$W` means the coordinator's `workers-20261002` directory. Per-run summaries preserve source SHAs, exclusions, failure names and raw artifact locations. TUI counts were reconciled with libtest's final suite record to account for terminal output preceding JSON events without a newline.

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

[The current queue](https://github.com/CorbanuCore/CorbanuTerminal/issues/1532#issuecomment-6939945244) maps `$W/astra-synccore6-decisions.md` SC6-1…7 to D679…D685. **Travis accepted D679–D685 as recommended on November 17 at 06:31Z** ([owner record](https://github.com/CorbanuCore/CorbanuTerminal/issues/1532#issuecomment-6940669510)). The table preserves the original choices; implementation/evidence links are reconciled in [#2565](https://github.com/CorbanuCore/CorbanuTerminal/pull/2565), without claiming final qualification.

| Decision | Failing occurrences | Choice needed; recommendation |
| --- | ---: | --- |
| D679 / SC6-1 | 2 app | Preserve an explicitly saved low effort on cold resume, or let an edited role's high default override it. Recommend preserving the explicit choice. |
| D680 / SC6-2 | 1 app | Keep frozen base instructions after revert/model change, or rebuild them. Recommend keeping them; a new conversation rebuilds instructions. |
| D681 / SC6-3 | 1 app | Document unsupported Cursor paths, or recover multiple punctuated path components under a hard search bound and reject ambiguity. Recommend bounded recovery, with security review. |
| D682 / SC6-4 | 9 core + 1 core library | Replace unsupported positive tests with fail-closed/hardened-engine matrices, or leave them failing pending separately approved implementations. Recommend replacement, preserving secret-exclusion assertions and the distinct copied-token regression. D530/D353/D354/D640 restrictions remain binding. |
| D683 / SC6-5 | 1 core | Notify only for explicitly delegated work, or inherit an old completion recipient after a queue-only wake. Recommend explicit delegation only. |
| D684 / SC6-6 | 1 core | Send a full current skill catalog on reconnect, or preserve full catalog plus availability history in section-replacing adapters. Recommend the full current catalog; test both provider styles through compaction/resume. |
| D685 / SC6-7 | 1 core library | Keep the copied-token gap when its source is unset, or filter using broker-known credentials with bounded matching and refusal when protection is unavailable. Recommend filtering, with independent security review and Travis's sign-off. |

These were the **17 decision-dependent failures** at the original cut, grouped by root cause; their choices are now accepted and the implementations are linked in #2565. They are not a current pending-decision count. The remaining **22 CLI failures** are remote executor/plugin/hook/MCP/credential-history startup failures (21 core, one app), not evidence that those security cases passed. D685 is a real protection gap shared with main, not just an upstream expectation mismatch.

Other owner actions: schedule the Windows desktop checklist; sign SG2 and the final SG3/merge. **D676–D678 were accepted at 05:45 UTC** and are being implemented in #2493; the handoff's older pending labels are stale. D659–D674 are accepted; D675 was superseded. D628 deliberately leaves the Windows provisioning service **off**; activation is a separate future decision, not permission to enable it for landing. Deferred named-account flag removal #1080 is likewise outside this sync. No additional undecided D-row was found in the latest coordinator record.

### Other test, CI and tracking gaps

- **macOS selected-suite requalification — November 19, 2026 UTC (November 18 in Arizona).** astra-syncmac2 re-ran the same suites after MSM-1 [#2708](https://github.com/CorbanuCore/CorbanuTerminal/pull/2708), MSM-2 [#2663](https://github.com/CorbanuCore/CorbanuTerminal/pull/2663), main merge #12 [#2677](https://github.com/CorbanuCore/CorbanuTerminal/pull/2677) and the intervening security ports. The starting sync head was `b591b61a237d242de8b0f082b207f4bebe071c20`. The measured head `12d90cce4a0674f42deb58ece5f4b8508a694ee5` adds only the unused macOS test import removal in [#2712](https://github.com/CorbanuCore/CorbanuTerminal/pull/2712); its tree is identical to the fetched sync merge head `775dc5a83c459a9c333f62c2ac13bdd2f77bb937`. This is selected native macOS evidence, **not a full-workspace or all-platform pass**.

| Suite (macOS aarch64) | Passed | Failed | Ignored / excluded | Measured head SHA |
| --- | ---: | ---: | --- | --- |
| Core library | 5,026 | 0 | 10 ignored | `12d90cce4a0674f42deb58ece5f4b8508a694ee5` |
| Core macOS exec subset | 6 | 0 | 3,169 other cases filtered out | `12d90cce4a0674f42deb58ece5f4b8508a694ee5` |
| Exec library / binary | 83 / 1 | 0 / 0 | 0 | `12d90cce4a0674f42deb58ece5f4b8508a694ee5` |
| Exec integration (includes 15/15 Seatbelt cases) | 142 | 0 | 0 | `12d90cce4a0674f42deb58ece5f4b8508a694ee5` |
| Exec-server library | 486 | 0 | 0 | `12d90cce4a0674f42deb58ece5f4b8508a694ee5` |
| Exec-server integration (includes native filesystem-spawn case) | 286 | 0 | 18 ignored | `12d90cce4a0674f42deb58ece5f4b8508a694ee5` |
| App-server library | 469 | 0 | 0 | `12d90cce4a0674f42deb58ece5f4b8508a694ee5` |
| App-server integration, original run | 1,641 | 1 flake | 86 ignored; 14 CLI exclusions | `12d90cce4a0674f42deb58ece5f4b8508a694ee5` |
| App-server integration, full rerun after fixture synchronization | 1,642 | 0 | 86 ignored; same 14 CLI exclusions | `6bd6dfc070225592477aff08dd09af4289dbbc7f` |
| TUI library | 7,501 | 0 | 9 ignored | `12d90cce4a0674f42deb58ece5f4b8508a694ee5` |

**New failure: fixture timing flake.** `thread_settings_confirmation_leaves_mcp_status_responses_unaffected` compared two independently started MCP fixtures while one reported `Starting` and the other `Connected`; their tool data matched. An unchanged isolated replay passed. The test now polls within its existing ten-second deadline until each fixture reports `Connected`, rejects unexpected states, and retains the tool-list checks and entire-response equality assertion. The full selected app-server rerun above passed; this is a shared fixture race observed on macOS, not a Linux-only assumption or a newly identified product defect. No assertion was removed and no timeout or skip was increased.

Both MSM-1 shell-wrapper tests passed with native `/bin/zsh`; MSM-2's broker test exited normally and passed in 1.196 seconds. The earlier baseline and four fixture repairs remain recorded in [#2654](https://github.com/CorbanuCore/CorbanuTerminal/pull/2654). Native Rust 1.99.1 core integration-test clippy with `-D warnings` and the affected sandbox test (1/1) passed after #2712 removed the unused `Op` import.

Runs used Rust 1.99.1, six Cargo jobs, four test workers, zero automatic retries, non-CLI helpers built first, private child HOME/profile directories and native-keyring denial. Native sandbox skip flags were unset. Memory was monitored before and throughout the campaign, with owned-process pausing below 20% free. CLI #2274, all ignored cases and the 14 app-server CLI exclusions remain unqualified. Those exclusions are `suite::v2::executor_mcp::*`, `managed_plugins_requirement_disables_selected_executor_plugin_capabilities`, `selected_plugin_mcp_startup_respects_explicit_mentions` and `selected_capability_stack_tracks_environment_selection_and_resume`; nextest inventories confirm their counts.

Evidence: `$W/astra-syncmac2-evidence/` contains per-run raw logs, source-SHA/argument/exit metadata, summaries and test inventories; `app-server-baseline.log` preserves the failure, `mcp-status-unchanged-replay.log` the unchanged pass, and `app-server-final.log` the full repaired rerun. Other stems: `tui-lib-baseline`, `other-libs-baseline`, `core-macos-exec-baseline`, `exec-baseline`, `exec-server-baseline`, `core-integration-clippy`, `sandbox-import-test`. Result: `$HOME/.corbanu-workers/astra-syncmac2.result.md`; tracking: [#1632](https://github.com/CorbanuCore/CorbanuTerminal/issues/1632). CLI, Bazel, workflow qualification and final landing sign-offs are separate gates.
- **Bazel qualification:** [#2645](https://github.com/CorbanuCore/CorbanuTerminal/pull/2645) repairs workspace package metadata, D479 compatibility targets and the network contract input. Separately reviewed sync repairs cover D316 voice exclusion and MSVC long paths (#2662); owning-crate fixtures, migration expectations, helpers, OTLP exports and compile inputs (#2664/#2665/#2674); duplicate disk-cache use and the guarded keyring lint (#2666); accepted D725 command-temp authority and D377 private-temp ordering, with protected RPC parent canaries (#2673/#2687/#2668); D696 partial-command allow spelling (#2679); Windows proxy, sandbox, timing and effective-backend fixtures (#2680/#2683/#2696/#2703); Linux fixture lookup, mapped-group selection and strict clippy diagnostics (#2689/#2691/#2693); and the compile-time guardian disclosure assertion (#2715). `codex-voice-host` remains excluded pending its owner decision; no MSVC runtime EULA is accepted. Exploratory failures and their repairs are recorded on #1632; #2702 tracks the observed inline-cmd quoting defect; temporary diagnostic deadlines and logging must not land. Qualification is tied to #2645's exact final-head checks and linked evidence: every check must finish, every non-#2274 check must pass, and only documented #2274 failures may remain before this sync repair merges. Cross shards that skip execution without BuildBuddy are not test evidence. Final upstream landing still requires full Linux, native Windows and nightly qualification with no #2274 exception.
- **Open issue labels are not a fresh failure inventory.** #2430's native collaboration-schema regression needs an explicit fix/test cross-reference before closure; reconcile it with the 30 synccore6 repairs. #2082 was implemented by contributor #2101 despite remaining open. Likewise reconcile #2270/#2051, #2347/#2356/#2402, #2287, #2308/#2327, #2351/#2370, #2296/#2277/#2312 against their merged fixes and remaining cases. Do not silently count stale issue titles as new failures, or close them solely because another suite passed.
- **Supply chain:** #2349's Zod fix is already in sync via main merge #6. #2305 still tracks four dev/build advisories on its last post-fix audit (three moderate, one low; production-only zero). Re-audit the final lockfiles under SG1, retaining explicit dispositions rather than treating dev dependencies as harmless. #2477 tracks file-descriptor-limit test failures; #1448 remains open after Windows serialization mitigation #2439. Carry their final fixes/evidence as needed.

### Workflow reconciliation — November 18

Rechecked at sync `dd2ea9308774ff2d267f6be5af7a52113b523b2d` and main `e815d43cc560810d5302b063cea5918834ef17a5`: **51 sync directory entries, 47 main; four sync-only workflows and 24 shared-but-different workflows**. The original inventory counted three shared support files (`Dockerfile.bazel`, `README.md`, `zstd`) too: executable YAML workflow definitions number **48 sync / 44 main**. The table covers that exact comparison, not the older test-baseline cut above. This bounded CI fix preserves **Product principles**: “Maintain continuous Codex parity without removing Corbanu-specific behavior.” Tracking: #1632. CLI/#2274 and Bazel metadata remain with their existing owners.

**Disposition semantics:** keep-main restores main's content; merge retains the already-integrated Corbanu gates plus the named sync adaptation; disable makes every job unconditionally false, or removes the file where D480 already authorizes deletion. No new publishing route is authorized. No file needs an unconditional take-upstream replacement. Temporary CI exclusions stay until the separate landing step below.

All paths in the table are under `.github/workflows/`.

| Workflow | Inventory | Decision | Reason and resulting behavior |
| --- | --- | --- | --- |
| `sync-package-ci.yml` | Sync-only | disable **at landing** | Keep the existing `sync-ci/**` package lane now; remove with CI mode, including any temporary package/platform/test selectors. It is not full-workspace qualification. |
| `python-sdk-build.yml` | Sync-only | disable | Keep source for a future packaging decision; all jobs are false. Reusable inputs select source/version, tools use `github.workflow_sha`, output is `python-sdk-package`; callers are the two disabled SDK publishers. No Corbanu package namespace is approved. |
| `python-sdk-cli-release.yml` | Sync-only | disable | All jobs are false, including resolver, build calls and both publishers. Neither completion of `rust-release` nor manual dispatch can publish. Preserve source/artifact validation for review; OpenAI PyPI names are not Corbanu destinations. |
| `rust-release-provisioned-macos.yml` | Sync-only | disable | Both signing and verification jobs are false; setting `CODEX_PROVISIONED_MACOS_CANDIDATE` cannot enable them. Preserve profile SHA, certificate pin, team validation, notarization and candidate-only artifact logic for an owner decision. No active caller is added. |
| `blob-size-policy.yml` | Shared | merge | Keep explicit read-only job permission, credential-free checkout and merge-candidate blob inspection; do not restore head-only inspection that misses combined-file growth. |
| `blocking-ci.yml` | Shared | merge | Keep all eight aggregate dependencies, including Bazel and D623's macOS lane (#2514), `always()`, candidate-revision helper and strict result policy outside sync. Remove only tagged exemptions at landing. |
| `cargo-deny.yml` | Shared | merge | Keep merge-candidate checkout and main's advisory/report policy, fetch depth and credential-free checkout. No advisory exemption or permission changes. |
| `codespell.yml` | Shared | merge | Check the combined candidate; retain main's matcher, spelling policy and credential-free checkout. |
| `corbanu-terminal-ci.yml` | Shared | merge | Keep Corbanu build/package checks plus checksum-verified V8 setup pinned to upstream `086ccc833bc9cf01024ad2dd6af7dbd421b6bf9a`. |
| `corbanu-terminal-release.yml` | Shared | merge | Keep Corbanu destinations, publish opt-in, release checks and signing behavior; retain pinned V8 setup. Default 0.1.48 matches this sync workspace, whereas main is 0.1.49; align to the final workspace version at landing. |
| `issue-labeler.yml` | Shared | keep-main | Restore main's label taxonomy; the upstream ChatGPT dots label is not a Corbanu product disposition. Existing token/permissions and events stay unchanged. |
| `orchestrate-tui-matrix.yml` | Shared | merge | Only the temporary sync-base exclusion differs; preserve now, remove at landing without changing main's paths or matrix. |
| `python-runtime-build.yml` | Shared | disable | All jobs false. Preserve source-ref input, workflow-revision tool pins, offline wheel build and runtime-version output for later review. Both runtime callers and the SDK callers are disabled. |
| `python-runtime-release.yml` | Shared | disable | All jobs false, including reusable build and PyPI publisher. Preserve wheel verification code; existing manual trigger grants no publishing approval. |
| `python-sdk-release.yml` | Shared | disable | All jobs false. Upstream adds stable releases and splits build/publish jobs; do not activate that expanded package/version surface or its OIDC publishing without an owner decision. |
| `rust-ci-changed-crate-tests.yml` | Shared | merge | Preserve main's package selection, isolated helpers and tests; retain pinned V8 setup required by the synced dependency. |
| `rust-ci-core-developer-accounting.yml` | Shared | merge | Preserve developer-accounting features, helper isolation and suite; retain pinned V8 setup. |
| `rust-ci-core-security.yml` | Shared | merge | Preserve Linux/macOS security suites and existing permissions; retain pinned V8 setup. Its callers stay intact; sync's temporary parent skip is not security qualification. |
| `rust-ci-full-nextest-platform.yml` | Shared | merge | Preserve platform matrix, isolated tests and full-suite coverage; retain pinned V8 setup. |
| `rust-ci-lint-build.yml` | Shared | merge | Preserve main's lint/build gate, including Windows clippy; use the pinned upstream V8 action on Windows too for the synced artifacts. |
| `rust-ci.yml` | Shared | merge | Keep all main gates and the already-landed schema-dependency selectors for synced crates. Do not drop security callers or native Windows coverage. No event trigger changes here. |
| `rust-release-windows.yml` | Shared | disable (delete, D480) | Remove the unused upstream release helper with its parent; Corbanu's release workflow builds its own Windows artifacts. Do not transplant upstream signing permissions. |
| `rusty-v8-release.yml` | Shared | merge | Keep existing publication scope/guards and Corbanu provenance checks; retain sync's V8 target conventions, pinned Windows tools, Array.sort backport and patch-sensitive cache key. Bazel execution remains a separate qualification gate. |
| `sdk.yml` | Shared | merge | Preserve main's SDK checks, helper staging and package coverage; retain pinned V8 setup. Restore execution through blocking-ci at landing. |
| `tmux-smoke.yml` | Shared | merge | Keep smoke assertions, isolated helper set and pinned V8 setup. Remove only the temporary sync-base exclusion at landing. |
| `v8-canary.yml` | Shared | merge | Preserve canary matrix, provenance and checksum checks with the same synced V8 source/tool/backport adaptations as rusty-v8-release. No Bazel metadata edits or claim of Bazel qualification. |
| `windows-integration-tests.yml` | Shared | merge | Preserve native Windows tests and known-failure reporting; retain pinned V8 setup. No skips or failures removed. |
| `windows-security-probes.yml` | Shared | merge | Preserve credential/provenance/ACL probes plus already-landed #2315 service coverage and the moved #1953/#1776 library selectors. D628 still leaves the provisioning service off; testing it does not enable it. |

**D480 also applies to three files identical at the comparison cut:** delete `rust-release.yml`, `r2-release.yml` and `rust-release-zsh.yml` together with `rust-release-windows.yml`. Travis accepted this on November 9, 10:35 UTC ([decision record](../managementHandoff.html)); Corbanu ships through `corbanu-terminal-release.yml`. Removing this caller graph prevents inherited npm, R2, winget, website and signing routes from becoming release paths. The retained `rust-release-argument-comment-lint.yml` still supports its separately authorized Corbanu manual dispatch. Inventory becomes **47 directory files (44 YAML workflows) now, 46 files (43 workflows) after removing sync-package-ci at landing**; matching main's count is not the goal.

The unchanged `credential-boundary-canary.yml`, `security-ingress-contract.yml`, `security-platform-contract.yml`, `codeql.yml`, ordinary postmerge/nightly and macOS required-lane workflows remain in place. This reconciliation adds no permissions, event triggers, secrets or publishing destinations. Disabled packaging files and their regression test have permanent corbanu-security ownership, outside the removable sync block.

#### Decision candidates for Travis

- **Python SDK/runtime distribution:** leave all five Python packaging workflows disabled (recommended until a Corbanu SDK release is requested), or commission a separate reviewed enablement defining package names, source/artifact provenance, caller graph, version policy, PyPI environment/trusted publisher and credentials. Never simply replace the old `openai/codex` repository guard.
- **Provisioned macOS candidates:** leave `rust-release-provisioned-macos.yml` disabled (recommended), or separately approve the public-distribution profile, certificate/team pins, signing/notarization environment, caller and artifact destination. A repository variable alone is insufficient.
- D480's four deletions are an accepted decision, not a new approval request. Neither candidate blocks landing while disabled.

#### Exact sync-ci-mode revert checklist — landing only

1. Finish #2274/helper and Bazel qualification prerequisites; carry the latest main and record both SHAs. Preserve D623's required macOS lane (already present), all eight `blocking-ci.required.needs` entries and main's current credential/security/signing protections. Reconcile `corbanu-terminal-release.yml`'s default version to `codex-rs/Cargo.toml`; do not copy main's old workflow wholesale over pinned V8 setup.
2. From the clean final-candidate worktree run:
   ```sh
   python3 scripts/upstream_sync/ci_mode.py revert .
   python3 scripts/upstream_sync/ci_mode.py check .
   ```
   Review the exact effects: remove the three tagged `if` lines on `blocking-ci`'s `bazel`, `rust-ci`, `sdk`; remove its `SYNC_SKIPPED_OK` environment line and the tagged allowance in `.github/scripts/check_ci_results.py`; remove `branches-ignore: ["sync/**"]` from `tmux-smoke.yml` and `orchestrate-tui-matrix.yml`; delete `.github/workflows/sync-package-ci.yml`. Do not alter the reusable jobs, their matrices, security callers or existing publishing guards.
3. Remove `.github/sync-package-ci.json` if a later package run introduced it (absent at this audit). Inspect `git grep -n -E 'sync-package-ci|SYNC_SKIPPED_OK|sync-ci-mode|sync/|sync-ci/' -- .github codex-rs/.config scripts/isolated_rust_tests.py`; remove any remaining *active* sync selectors or exemptions, preserving unrelated configuration and ordinary tests. The package workflow template and apply/revert tooling under `scripts/upstream_sync/` are historical program tooling, not active workflows.
4. Before removing temporary ownership, run `python3 scripts/upstream_sync/ci_mode.py codeowners drift . @corbanu-security`. Carry `SYNC_SUCCESSORS` from that script and every still-required moved/new security path into permanent ownership (D481); retain main's latest D318 block and the disabled packaging ownership added here. Then run:
   ```sh
   python3 scripts/upstream_sync/ci_mode.py codeowners revert .
   python3 scripts/upstream_sync/ci_mode.py codeowners check .
   ```
   Review the diff: only the temporary block disappears; no live security path loses its owner. Include an exact-head security approval.
5. Run `python3 -m unittest discover -s .github/scripts -p 'test_*.py'`, `python3 docs/sprints/check.py`, workflow syntax validation and `git diff --check`. Exercise `check_ci_results.py` with all-success inputs (pass), then each dependency individually set to `failure`, `cancelled` and `skipped` (all fail), including an inherited `SYNC_SKIPPED_OK=bazel,rust-ci,sdk` environment. Verify there is no allowance after revert.
6. On the exact final candidate run all restored PR checks plus full Rust/platform, credential canary, ingress/platform, native Windows security, tmux/TUI, Bazel/nightly and release dry-run lanes listed below. Keep the five Python workflows and provisioned macOS workflow disabled; do not dispatch them as qualification. Require completed green checks with **no #2274 expected-red exception at landing**, independent Opus APPROVE, matching corbanu-security approval and Travis's SG2/SG3/final authorization.

This PR deliberately does **not** execute the revert. The checklist is separate from, and does not replace, final security and release qualification.

### Ledger and release-note completeness

The November 17 reconciliation in [#2565](https://github.com/CorbanuCore/CorbanuTerminal/pull/2565), against sync source `5b5c85143884c903cdcceecca276c06f6f9eee94`, records **3,749 rows: 3,670 resolved and 79 open** after audit. Preview ledgers are excluded. These are row counts, not unique defects. The earlier cut had 3,679 rows; 70 resolved entries were added subsequently, then this audit resolved 17 of the 96 previously open rows. Each remaining row has a dated disposition and owner: 10 are held by #2274 and 69 are tracked in [#2562](https://github.com/CorbanuCore/CorbanuTerminal/issues/2562), [#2563](https://github.com/CorbanuCore/CorbanuTerminal/issues/2563) and [#2564](https://github.com/CorbanuCore/CorbanuTerminal/issues/2564). These counts do not establish SG2/SG3 or sprint completion.

| Ledger suffix after `ledger-rust-v0.167.0` | Resolved | Open |
| --- | ---: | ---: |
| `.tsv` | 5 | 25 |
| `-resolved.tsv` | 1,006 | 0 |
| `-s1c.tsv` | 202 | 0 |
| `-s3b.tsv` | 547 | 27 |
| `-s4a.tsv` | 136 | 0 |
| `-compile.tsv` | 192 | 7 |
| `-main-merges.tsv` | 643 | 11 |
| `-main-merges-2.tsv` | 939 | 9 |

The integrator and security owner must finish the **79 dispositioned open rows**, linking implementation and current-head test/review evidence before closure. Work includes the CLI/MXC row and obsolete MCP-server remnants; S3b guardian/disclosure, sandbox/credential and Windows audit/ACL rows; old core/session/test ports; Telegram notification handling; D525 EMA/config/role documentation; remote MCP discovery coverage; and the 20 remaining main-carry rows. A later passing suite alone does not resolve a documented boundary-review obligation. Audit migration renumbering and preserved released checksums too. Preserve split ledgers and the blob-size limit.

There are **152 Markdown release-note fragments** (plus `.gitkeep`) on the reconciled sync source. Check and preview pass; [#2564](https://github.com/CorbanuCore/CorbanuTerminal/issues/2564) records the coverage, sprint-record and migration/rollback gaps. Presence is not completeness. Audit user-visible removals, provider/auth behavior, security changes, doctor/CLI changes and D679–D685 outcomes against fragments, then run `release_notes.py check` and `preview`. Never recreate `RELEASE_NOTES.md`. Main merge #9 deliberately omitted the duplicate #2443 fragment because sync's D659–D664 fragment already covers it; preserve that explanation. Finish app-server/config guidance, the upgrade/compatibility record, footprint dispositions and sprint evidence. Do not mark S3/S4/S5 complete from ledger counts alone.

**Sprint records also block handoff; owner: the integrator.** At the cut, main's PF-85-S01–S05 records are `draft` with no implementation worktree assigned. On sync, S01 is `in_progress`, while S02–S05 remain `draft` and unassigned. Reconcile each record with its actual authorized worktree, completed work and final-tree evidence; do not relabel them complete merely because code merged. S03 must record Travis's exact-head SG2 sign-off on the dedicated **“needs security sign-off”** PR. Complete and archive S01–S04 when their gates are met. S05 must accurately separate pre-merge gates from its later merge/canary tasks; complete and archive it only after those tasks finish. Run `python3 docs/sprints/check.py` on the reconciled records.

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
2. **S3 / SG2 security:** record security-level × tool-path × platform parity and fail-before/pass-after evidence for moved boundaries. Run `rust-ci-core-security` on Linux and macOS, `security-platform-contract`, `security-ingress-contract`, `credential-boundary-canary` on all applicable OSes, `windows-security-probes`, Linux sandbox tests on glitch, macOS seatbelt and native Windows sandbox tests/PC checks. Include credential/browser backend isolation, remote executors/MXC, taint and provenance through compaction/resume, hooks, named accounts and D685. Slice approvals are supporting evidence; **Travis signs the dedicated S3 “needs security sign-off” PR against its exact SHA**, and S03 records that approval.
3. **Full final-tree CI:** `blocking-ci` including Windows clippy `-D warnings --tests` and native Windows tests; `rust-ci-full` with full Linux local/remote coverage; Bazel and nightly lanes; cargo-deny, codespell, repository/schema/snapshot checks; `just test` on macOS and Linux clippy with Rust 1.99.1. Run all helper-dependent suites, not just prior failure selectors. Conditional skips must be justified; no failed, pending or cancelled required qualification run is green. Fix failures and rerun affected gates on the changed candidate.
4. **Functional qualification:** fresh-context code-blind design and separate isolated execution of frozen cases, negative isolation probes and positive PTY controls, then independent evidence review. Exercise real keys through success/cancel/failure/recovery/resume for providers/routes, vault, Task Node, spawn/mail, security levels, wallet/GPU without paid spend, and new upstream UI. Run true-TUI workflows in both TensorCash and Isometric Game; archive demos and exact binary hashes. A hosted-model refusal requires the local-model replay path. Unit tests do not replace this gate.
5. **Windows human and VM gaps:** Travis's roughly 30-minute D396/D416/D448 checklist plus step 4b/#1803 is still outstanding; automated evidence and D461/D462 acceptance do not replace it. #1800/#1798/#1851 stay draft-gated; #1798 lands after or with #1800. If included before sync lands, carry and requalify them; otherwise record their deferred scope for Travis's SG2 decision rather than implying they shipped. Separately, PF-23-S04 case 11 remains blocked until a Windows guest exists (D673). The accepted Linux leg (D669), case 7 extra prompt (D671) and case 8 partial disposition (D672) are historical evidence, not an all-platform sync pass. Requalify affected cases on the sync candidate and obtain explicit final dispositions.
6. **Compatibility/release readiness:** back up and upgrade a released state DB, prove the previous released binary can reopen it, and verify vault/archive compatibility. Complete the qualification record from `upstream-integration.md`, user docs/release-note preview, due benchmark evidence and a non-publishing `corbanu-terminal-release` dry run. Before landing, S01–S04 must be complete with final-tree evidence and archived, S05's pre-merge checklist must be satisfied, and `python3 docs/sprints/check.py` must pass. Before release, finish and archive S05 after merge/canary; every included product initiative must link to completed sprint evidence as required by AGENTS.md.
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
