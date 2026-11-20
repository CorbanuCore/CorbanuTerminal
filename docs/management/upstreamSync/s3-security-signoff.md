# S3 security sign-off record (SG2) — rust-v0.167.0

**Status: PRELIMINARY. Not signed.** This record is prepared for Travis's SG2 signature. No worker, reviewer or agent approval stands in for it. It is measured on sync head `ab557d87afe6917fb9bb43385f2f122b6bf150e7` (#2789), against main `15415856d7c0d77ba82bdb234924884e8464cc1b`. The signable candidate comes later: after main merge #13 ([#2791](https://github.com/CorbanuCore/CorbanuTerminal/pull/2791)) and the remaining fixes. On that candidate, the lanes are rerun and every run link and SHA here is refreshed (see [Refreshing for the final candidate](#refreshing-for-the-final-candidate)).

Plan: [landing plan](landing-plan-v0.167.0.md), step 5 and final gate 2. Tracking: [#1630](https://github.com/CorbanuCore/CorbanuTerminal/issues/1630) (S3), [#2562](https://github.com/CorbanuCore/CorbanuTerminal/issues/2562) (security carry rows), [#1532](https://github.com/CorbanuCore/CorbanuTerminal/issues/1532). Owner decisions are cited by D-number from the [management handoff](../managementHandoff.html), section 9 and the dated decision boxes.

## What Travis is signing

1. **Scope.** The sync branch rebuilds Corbanu's security on upstream's unified exec. That covers classifier and approval decisions, the three security levels, sandbox selection, vault and broker isolation, and taint/provenance. Upstream's shell handlers, `shell-escalation` and the zsh backend are gone (D275).
2. **Parity claim.** At every security level, each tool path behaves as it does on main, on Linux, macOS and Windows. The exceptions are the differences in [Differences from main](#differences-from-main) below. Each has an accepted owner decision, except the four marked **For SG2**.
3. **Moved boundaries.** Every place where sync had to re-implement or move a protection has a regression test. Each test failed before its fix and passes after, or is listed as test-only (it pins behaviour that never broke). See the [fail-before/pass-after table](#fail-beforepass-after-for-moved-boundaries).
4. **Lanes.** On the exact candidate, these must all pass on all three platforms ([run record](#run-record)):
   - the required security lanes;
   - the Linux sandbox tests on glitch;
   - the macOS Seatbelt tests;
   - the native Windows sandbox tests.

   This preliminary run is not all green yet: see #2796, #2797 and #2808. Anything not green is listed with its owner.
5. **What stays open.** The [open residuals](#open-residuals-and-owners) do not block landing, but Travis should see them. The **For SG2** items need his explicit choice:
   - **Δ1:** accept upstream's retired known-safe list. Commands under `untrusted`, and Windows without its sandbox with approvals off, now ask or are refused more often than on main. This is stricter, not weaker.
   - **Δ2, D741:** memory v2 as the default for new profiles.
   - **Δ3, D738:** resume on the recorded provider. A product fix would need its own security sign-off.
   - **Δ4, Windows desktop checklist:** #1800/#1798/#1851 and D608 stay out of this landing. They wait for the checklist; their protections are not claimed as shipped.
6. **Not signed here.** This does not cover:
   - SG3: the security-path diff from the signed SHA to the final head;
   - the final merge;
   - the 48-hour canary;
   - functional/code-blind qualification (final gate 4);
   - non-security pending decisions D739, D740 and D742.

   A security-path change after the SG2 SHA needs SG3 re-signing.

**Signature line (Travis only):** SG2 approved on candidate SHA `________________________________________` — date ________.

## Run record

All runs use sync head `ab557d87af` unless stated otherwise. CI runs were started with `workflow_dispatch` on 2026-11-19 at 22:25Z; no workflow was edited. Local runs used Rust 1.99.1, a private HOME, no CORBANU_HOME/CODEX_HOME, `CORBANU_TEST_NO_NATIVE_KEYRING=1` and dummy keys only.

| Lane | Platform | Sync run (`ab557d87af`) | Result | Main comparison (`15415856d7` unless noted) |
| --- | --- | --- | --- | --- |
| `rust-ci-core-security` | Linux | [44628428629](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44628428629) | **failure** ([#2796](https://github.com/CorbanuCore/CorbanuTerminal/issues/2796)). security:: 1,006/0/7 twice; pf_23_s01 + pf_25_s01 61/1 (`issue_1472…`: the test's reason matcher, prompt is raised). Later steps did not run in CI; on glitch at the same head, #902 suites 13/0 and #2477 fixtures 16/0 | [44629087005](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44629087005) success: security:: 1,001 ×2; pf_23_s01 + pf_25_s01 62 ×2; #902 13; #2477 16 (all 0 failed) |
| `rust-ci-core-security` | macOS | [44628433573](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44628433573) | **cancelled at the 60-minute job limit** (cold cache: the build took 37 minutes). Completed steps pass: security:: 1,006/0/7 ×2; pf_23_s01 + pf_25_s01 83/0. The TMPDIR=/tmp repeat, #902 and #2477 steps did not run. **Attempt 2 failed:** security:: 1,006/0/7 ×2 and pf_23/25 83/0 ×2 pass. In the #902 suites, 17 passed and 1 failed: `issue_747…` wrote outside the sandbox on Permissive Retry ([#2808](https://github.com/CorbanuCore/CorbanuTerminal/issues/2808)). The #2477 step did not run. A local Mac run of these steps is **not counted**: its harness was broken (wiremock and broker-URL errors in 81 unrelated cases) | postmerge [44615159759](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44615159759), job 135881769742: security:: 1,001 ×2; pf_23/25 83 ×2; #902 18 ×2; #2477 5 (all 0 failed) |
| `security-platform-contract` | Linux, macOS, Windows | [44628438679](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44628438679) | success, 3/3 jobs | [44629093135](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44629093135) success |
| `security-ingress-contract` | Linux, macOS, Windows | [44628445968](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44628445968) | success, 3/3 jobs | [44629098304](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44629098304) success |
| `credential-boundary-canary` | Linux, macOS, Windows | [44628453033](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44628453033) | success, 3/3 (Linux, macOS, Windows PF-13 credential qualification) | [44590975298](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44590975298) at `08a23b091a`: success, 3/3 |
| `windows-security-probes` | Windows | [44628458553](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44628458553) | success (`pf_27_s06 probes`, incl. the elevated launch-contract, D324/#1648 medium-session and broker-pipe probes) | [44614770168](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44614770168) at `4c527065f1`: success |
| `windows-integration-tests` (supporting) | Windows | [44629804537](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/44629804537) | codex-core: 2,398 passed, **3 failed** (the D742 `agent_message_wake` cases, same as the reference), 103 skipped. codex-app-server: 1,569/1,569 passed, 95 skipped. Other selected packages pass | Sync-base reference: run 44594161347, at #2779 head `0f997c8c21`; 3 `agent_message_wake` failures (D742) |
| Linux sandbox: `codex-linux-sandbox`, `codex-sandboxing` | glitch (Ubuntu, kernel 6.8, bwrap 0.9.0) | local | `codex-sandboxing` 117/0. `codex-linux-sandbox` lib 255/0; integration first run 118/**1**/3 ignored (`a_command_without_bubblewrap_keeps_its_terminal`: “outlived its terminal's hangup”, a parallel-fork race its own comment describes), then 3/3 alone and the full integration rerun 119/0/3 | sync-main8 at #2450: 117 / 255 + 119 |
| macOS Seatbelt: `codex-sandboxing` lib, `codex-exec` `suite::seatbelt` | this Mac (aarch64), `-j6` | local | `codex-sandboxing` 173/0/3 ignored; `codex-exec` `suite::seatbelt` 15/15. Memory stayed at 24–27% free | astra-syncmac2 at `12d90cce4a`: exec integration incl. 15/15 Seatbelt |
| Windows sandbox: `codex-windows-sandbox`, `codex-exec-server` `windows_sandbox` | DESKTOP-UJS3HC4, `C:\CorbanuQA\sg2-evidence`, MSVC | local, over SSH (not the interactive session) | `codex-windows-sandbox` lib 608/**5**/14 ignored, bins and `helper_manifest` pass. The 5 failures are environmental: main `15415856d7` fails the **same 5** in the same session (441/5/11): ConPTY `d730_legacy_tty…` and `legacy_command_tools…`, `sec_win_345_*` ×2 (non-interactive window station) and the D324 race (CI runs it from a medium session after recording setup; see #1800). `codex-exec-server` `windows_sandbox` 4/**5**: all 5 are MXC cases with no main counterpart ([#2797](https://github.com/CorbanuCore/CorbanuTerminal/issues/2797)) | sec-2562b/c, sec-2718, sync-win2729b runs on earlier heads |

S0 reference: at baseline `23be0b3883`, core security tests passed 1,613 on Linux and 1,678 on macOS ([baseline](baseline-23be0b3883b1.md)).

On the sync branch, `blocking-ci` still skips `rust-ci`, `bazel` and `sdk` (temporary sync CI mode). Per-crate native Windows tests and the full Linux suite therefore do not run on sync PRs. This record does not claim them. The final candidate runs them after `ci_mode.py revert` (landing plan, revert checklist step 6).

## Security levels: the contract being checked

- **Permissive:** exactly main's behaviour. The only exceptions are the listed differences. Every Permissive difference below makes sync stricter, never looser.
- **Moderate:** main's Moderate protections. These are untrusted-content gates (PF-23/PF-30), protected reads and writes, the credential broker on by default, and network approval.
- **Aggressive:** main's Aggressive controls. These are read-only or workspace sandbox, approvals always on, no network by default, no vault access, and `corbanu vault` forbidden. Under strict rules, any script the reader cannot read asks, or is refused when approvals are off (D553/D691).

## Parity matrix

Each row is a tool path. The level columns say how sync compares with main. The platform columns point at the evidence for that path on each OS. "Lane" means a run in the run record above; a PR number means that PR's recorded tests and review. Δ-numbers refer to [Differences from main](#differences-from-main).

| Tool path | Permissive vs main | Moderate vs main | Aggressive vs main | Linux evidence | macOS evidence | Windows evidence |
| --- | --- | --- | --- | --- | --- | --- |
| Shell / unified exec | Same, except Δ1 (stricter), D639, D618 and D728 | Same; D713/D729–D735 parser hardening is on both branches | Same; D713/D729–D735 on both | Lane core security (security::, pf_23_s01, pf_25_s01, #902 suites); glitch linux-sandbox; #2713, #2640, #2656 parser tests | Lane core security macOS; Seatbelt run; #2708 bash/sh/zsh broker-restore rows; #2663 | Lane windows-security-probes (launch contract, token boundary, #1805 env); windows-integration codex-core; desktop windows-sandbox; #2737, #2720, #2739, #2755, #2732, #2711, #2749 |
| apply_patch / file tools | Same | Same: protected reads/writes, PF-23 denies, D537 | Same | Lane core security (protected_reads/writes, issue_2222); #2647 path-uri | Lane core security macOS; astra-syncmac2 core lib 5,026/0 | #2741 (issue_1976/2049/2050, 5/5 native; 4 fail before); #2639, #2649, #2652, #2779 native; windows-integration codex-core |
| MCP | Same | Same: D491/D522 discovery refusals, D654 old-executor refusal, D525 project roles | Same | #2626 (4/4 glitch); core mcp_sandbox_tests in core `all` | Lane core security macOS (security:: only). The 14 executor-MCP app-server cases are excluded (CLI-dependent): **gap** | windows-integration codex-core. The **gap** is the same: #2122's four MCP cases are CLI-blocked |
| Web / network proxy | Same, except D715 (local MXC loopback, Permissive only) and D612 (sync-only config check) | Same; D612 also refuses unbrokerable providers at load and write; D714 | Same | network-proxy lib (#2630: 470/0; #2721: 478; #2763: 482); #2784; #2672 | `seatbelt_network_tests` in the Seatbelt run; #2672 2/2 on Mac | windows-security-probes (#1805, launch contract); proxy lib not run natively on sync: **gap** until the final full CI |
| Credentials / vault / broker | Same; D733: no copy filtering when the broker is off, as on main | Same; D685/D690/D727/D732 copy removal is on both branches; D726 | Same | Lane credential canary (Linux); lane platform contract (secret-broker); #2501, #2522, #2676, #2721, #2624; sec-smq2 real-broker runs | Lane canary (macOS); platform contract; #2708 zsh rows; #2663 | Lane canary (Windows); platform contract; windows-security-probes (PF-27-S06/S07/S08/S09 broker over named pipes, credential watch) |
| Hooks | Same | Same; D523 hook origin through deferral and readmission | Same | #2631 (4/4; history 37/37); core `hooks*` suites | Core lib on Mac (astra-syncmac2) | windows-integration codex-core |
| Named accounts | Same; D737 (credential-only commands skip the pair check) is on both branches | Same; D657 separate named-account network policy | Same | #2441; #2770 (`codex-cli --test login` 10/0); #2789 | Not run natively on sync: **gap** | #2788/#2789 PR CI; native worktree case recorded by sync-cli4: **partial** |
| Remote executors / MXC | Same, except D715 | Same; D616 (remote MXC needs explicit `allow_local_binding`); D522 (MXC counts as no sandbox); D541 (MXC refused without Corbanu protections) | Same | #2628; #2274 D541 unit + real-config tests; #2698/#2739 Windows-executor reading on a Linux host (60 + 120 cases fail before) | #2274 D541 tests on macOS | #2274 D541 real-binary test; #2741 (remote `view_image` now refused; reached the executor before) |
| Memory | Same for existing (v1) profiles; new profiles use v2 (**Δ2, D741 pending**) | Same redaction, provenance and level gates on v1 and v2 (sync-memory audit) | Same | #2774 memory policy canary: permissive, moderate, aggressive and cancel on v2, plus v1 permissive/moderate | Not run natively on sync: **gap** | Not run natively on sync: **gap** |
| Taint / provenance through compaction and resume | Same | Same; D624 revert keeps the untrusted flag; D625; D711/D721 fork floor | Same | `provenance.rs` (pf_30_s02 resume/compaction); #2636, #2637, #2644, #2659, #2684 | Core lib on Mac | #2749 (pf_30_s01 passes on Windows after the fix); #2684 native end-to-end; windows-integration codex-core |

**Matrix gaps to close on the candidate** (owner: integrator, through the final full CI):
- The MCP executor cases need the CLI helpers (#2122).
- Native macOS runs are still missing for named accounts and memory.
- Native Windows runs are still missing for the network proxy and memory.
- The 22 CLI-dependent core/app failures and the 24 core / 13 app exclusions in the landing plan must be rerun now that the CLI builds.

None is a known failure. Each is coverage that has not run yet on that platform.

## Differences from main

"Decision" names the owner record. **For SG2** means no owner decision covers it yet, so Travis decides at signing.

| Δ | Difference (plain language) | Level / platform | Direction | Decision |
| --- | --- | --- | --- | --- |
| Δ1 | Upstream retired the known-safe command list (#39630; `is_safe_command.rs` deleted). Under the internal `untrusted` policy, commands now need approval unless a rule allows them. On Windows without its sandbox with approvals off, every unmatched command is refused, not only `df`. The ledger records this as an intended upstream change (`ledger-rust-v0.167.0-compile.tsv` line 112, #2296; #2741 item 3; #2746) | All levels incl. Permissive; all platforms (Windows no-sandbox most visible) | Stricter | **For SG2.** Recommendation: accept, since nothing gets looser. The alternative is to restore a Corbanu known-safe list for `untrusted`, which needs its own review |
| Δ2 | New profiles default to memory v2. Existing v1 profiles stay on v1. Redaction, #578 provenance clearing, PF-30-S02 and the PF-23 level gates apply to both | All levels | Neutral for security (sync-memory audit) | **For SG2: D741 pending.** Recommendation A |
| Δ3 | Resuming a chat recorded on a non-default provider follows upstream (read-only) until D738 is decided. Two TUI cases are red | All levels | Not a boundary loosening | **For SG2: D738 pending.** A product fix (option A) needs its own security sign-off |
| Δ4 | #1800 (runner account and per-command logon), #1798 (shared-folder deny) and #1851 (.git link identity) are not in sync or main. D608 (S3b 562) waits on #1800 | Windows | Protections not shipped on either branch | **For SG2:** confirm they are deferred until the Windows desktop checklist (D396/D416/D448, step 4b/#1803) |
| Δ5 | After a sandbox denial, the command is not retried outside the sandbox | All levels. **macOS in doubt:** #2808 saw a Permissive write after one approval | Stricter | D639(a); D716(a) (#2672) |
| Δ6 | Asking to run outside the sandbox is offered only under OnRequest | All levels | Stricter | D618 (#2622) |
| Δ7 | Local MXC keeps upstream's loopback default under Permissive. A remote MXC executor needs `allow_local_binding = true` set explicitly | Permissive / Windows, remote executors | Sync-only feature | D715(a), D616 (#2628, #2672) |
| Δ8 | MXC is refused without Corbanu's protections. `use_mxc=false` never selects MXC | Windows | Sync-only feature | D541(a) (#2274) |
| Δ9 | A network-proxy credential provider naming a variable Corbanu doesn't broker fails config load and app-server config writes | All levels when providers are in use | Sync-only setting, fail closed | D612 (#2630, #2763, #2784) |
| Δ10 | Guardian v2: background scoring stays off, the model's self-assessed "no review" is ignored, and the Decisions comparison sampler is removed. 85 app-server cases that need live scoring are ignored with the D604 reason | Moderate/Aggressive with automatic review | Upstream feature restricted | D604, D605, D606 (#2621) |
| Δ11 | A forced automatic review is never labelled as the person's. After a requirements reload it cannot answer a post-taint permissions request alone | Moderate/Aggressive | Stricter | D625(a) (#2637, #2644) |
| Δ12 | Detached review is replaced by thread/fork. A fork never runs below the parent's level or Windows sandbox, and keeps a stricter sandbox it asked for. Explicit overrides are uncapped above the floor. A non-running parent uses history plus the stored level | All levels | Main's fork gap is fixed on sync only | D711(a), D720, D721, D722, D724 (#2659, #2684) |
| Δ13 | An unset controller Unix-socket setting inherits the owner's allow-all grant | Network proxy, Linux/macOS | Upstream behaviour, pinned | D714(a) (#2672) |
| Δ14 | Upstream's unbounded connection retries: real exec can wait indefinitely when the provider or broker is unreachable | All levels | Availability only, not confidentiality | D728 (#2663 test-only) |
| Δ15 | After global zsh startup, `AUTH_HEADER` stays as startup set it, the same as a broker-off run; brokered keys are still restored | macOS zsh, broker on | One upstream test row dropped | D726(a) (#2708) |
| Δ16 | Upstream positive tests for unsupported host/credential contracts are replaced by fail-closed and hardened-engine matrices | Tests | Coverage, not behaviour | D682 (#2498) |
| Δ17 | Disabled managed daemon self-update: the 12 updater tests assert refusal | All | Corbanu policy kept | D564, D736 (#2769) |

Hardening that landed on **both** main and sync is parity, not a difference:
- D713/D734 shell reader;
- D729/D733 PowerShell allowlist;
- D730/D735 cmd serializer;
- D731 module autoload;
- #2699 stream names;
- #2718 nested cmd;
- D685/D690/D727/D732 copy removal;
- #2401 URL passwords;
- D737 credential-command pair check.

Main changes after `717fcfa246` (#2682–#2785) are carried by main merge #13 (#2791). Any of them on a security path need a carry row and are covered by SG3.

## Fail-before/pass-after for moved boundaries

Merges are into `sync/upstream-rust-v0.167.0`. "Before" names what failed with only the fix reverted (or on the base); "After" is the recorded pass. Every row had an independent Opus 5.5 High APPROVE on its exact head. Every row except #2774 also has a corbanu-security approval whose commit is that head (checked through the GitHub reviews API on 2026-11-19). #2774 changes only TUI test fixtures, outside the corbanu-security paths; its Opus APPROVE is a PR comment. "Test-only" means no product code changed, so there is no failing old path to show.

| Boundary | PR (merge) | Before | After |
| --- | --- | --- | --- |
| #1976/#2050 Windows refusals for file tools and typing | #2741 (`c84351f0e5`) | 4 of 5 refusal tests fail natively; remote `view_image` reached the executor | 5/5 native; windows-integration 75 failures, all within the allowed base 80 |
| Local PowerShell PATH lookup (6 sync-only extras) | #2749 (`a861f61440`) | pf_30_s01 + 5 pf_23_s01 dotenv Aggressive fail on base run 44504190140 | The 6 pass; native exec_policy 261 passed |
| D729/D733 allowlist port | #2755 (`c638f505dd`) | New design (no PowerShell before approval); Round 1 found a Medium issue, which was fixed | Native 245 and 264 passed; CI codex-core failures ⊂ base 80 |
| #2699 NTFS-stream / 8.3 PowerShell names | #2739 (`227a6218ca`) | 60 forms ran unasked with always-allow; 120 sync-only cases | All pass on glitch |
| #2718 nested cmd/start/PATHEXT batch | #2737 (`9113d71956`) | Native tests fail with the injection marker written (old `command_line.rs`) | 13/13 unit + legacy/elevated native |
| D730 cmd `/C` serializer | #2720 (`d020dcdac3`) | Native tests fail on main's old quoting; the marker is written | 29 hostile arguments arrive byte-identical, both backends |
| #2745 trailing dot/space PATHEXT | #2786 (`f9ab6a7371`) | Documented: cmd does not trim, so nothing to fix; new refusal test | Sync CI green |
| D612 unbrokerable providers at load | #2763 (`fb3467e25b`) | With the check reverted, the load test and 4 configured-provider cases fail | network-proxy lib 482 |
| D612 config writes | #2784 (`7748025d1d`) | Without the check, the new test and `provider_remapping_validates…` fail | 5 write forms refused, file unchanged |
| D612 raw built-in token named by a provider | #2630 (`49848881ad`) | `main_engine_withholds_unmasked_builtin_configured_sources` fails | 470/0 |
| D737 credential commands skip pair check | #2770 (`c8c8a79b1a`) | `logout_clears_only_the_selected_bedrock_provider` fails on base | `--test login` 10/0; `mcp list` still refuses the pair |
| #2778 startup login gates | #2789 (`ab557d87af`) | Gates read the server provider's status | 2 new tests; Opus r1 |
| D377 temp spelling | #2779 (`ee1399d04f`) | New unit test fails natively (HashMap order) | Passes; codex-core down to 3 (D742) |
| D604/D605/D606 Guardian | #2621 (`042b4baf43`) | Three tests fail with gate `true` / old fast path | 27 pass, 85 ignored (D604) |
| D618/D522/D525/D617 | #2622 (`fc761044af`) | d522, project-role EMA and 2 TUI tests fail; D618 hung | All pass |
| D491/D522 discovery refusals | #2626 (`d4b3e124a5`) | Check-level and V2 tests fail | 4/4 |
| D616 remote MXC loopback | #2628 (`22335b562a`) | Both new core tests fail | Pass |
| D523 hook origin | #2631 (`4b1fcc4c6c`) | First three fail (inject/hook_runtime reverted); readmission fails (handlers reverted) | 4/4; history 37/37 |
| D530 factory controller; D624 revert taint flag | #2636 (`9ec5633981`) | Pre-spawn refusal skipped a factory replacement | glitch 98/98 targeted |
| D625(i) label; #2092 grants | #2637 (`4ee4e3f98c`) | Label test fails reverted; #2092 test-only | Pass |
| D625(ii) reload after dispatch | #2644 (`4e2ce64d42`) | Both checks' tests fail when removed | glitch 15/15 |
| #310/D544 shell snapshot | #2638 (`ce4d9594ec`) | Removing any of three filters fails them | shell_snapshot 51/51 |
| D609/D610 CODEX_HOME junction | #2639 (`a664a1baba`) | D609: deny went on the junction | Native 7/7 + 5/5 |
| D537 `\\localhost` UNC | #2647 (`9b3e08dec1`) | Refusal disabled: 2 path-uri + 2 file-tool cases fail | Native pass |
| D611 runtime read grants | #2649 (`a735617413`) | Race test: outside folder got the grant | Pass |
| D607 world-writable folders | #2652 (`3fd9d95499`) | AccessCheck: sandbox tokens could write before the deny | Cannot after; setup-refresh call site untested |
| D714/D716 | #2672 (`f86c7672ab`) | Both socket tests fail with `unwrap_or(false)` | 2/2 Mac; issue_739 1/1 |
| D711 fork floor | #2659 (`8eab89cc4a`) | `fork_parent_settings` 2 fail (fork ran Permissive) | 3/3; detached_agent_config 7/7 |
| D721 stricter fork sandbox | #2684 (`aedd8191e2`) | 3 unit tests fail; native end-to-end fails | Native and glitch pass |
| D685/D727 broker copies | #2676 (`1fc8cd020e`) | Real-broker run on glitch: the copy reaches commands | Copy absent |
| D732 proxy broker copies | #2721 (`67573a6632`) | 3 of 5 new tests fail | 478 passed |
| D685 / D690 snapshot copies | #2501 (`978c223c9d`), #2522 (`d87085a43d`) | See PRs | See PRs |
| D726 AUTH_HEADER rows | #2708 (`b591b61a23`) | Mutation: removing the restore fails the new test | bash/sh/zsh pass on Mac |
| MSM-2 broker-off hang | #2663 (`626e520cdd`) | Killed by signal 9 after 27 s | Exits 1 in 2.26 s (Mac) |
| D713 shell reader | #2713 (`96dc713dbd`) | n/a (precision change; reproducers still ask) | bash 13.1%, zsh 23.9% |
| #2704 cmd scripts under strict rules | #2711 (`11546af5c6`) | `cmd /c "git push"` ran unasked under a forbidden rule | Asks |
| D731 module autoload | #2732 (`cd6c2f4c1c`) | Cherry-pick of #2710 | Native 234 passed |
| #2556/#2583/#2599 unread PowerShell | #2633 (`d47ac6204a`) | Revert: 6/7 new core unit tests fail | Native 211/211 |
| #2548/#2585 default-deny scripts | #2640 (`a979c4c9ba`) | See #2613 rounds | Pass |
| #2594/#2597/#2601 rebinding; .NET case folding | #2656 (`78bb5bb914`), #2692 (`2e7006ae31`) | Off-Windows `Skip` with amendment under forbidden | Asks/refused in every mode |
| #2671 Windows-executor PowerShell names | #2698 (`da8a329cb9`) | Respellings read as unknown programs | Ask/refuse on any host |
| #2401/#2604 URL passwords | #2624 (`b9b8497aee`) | #2603 evasion fixtures | 400-case property test |
| D541 MXC selection | #2274 (`dacadfcc4a`) | n/a (new code) | Linux, macOS unit; Windows real binary |
| D657 named-account network policy | #2441 (`2198b0f0ef`) | See PR | See PR |
| Memory v2 privacy | #2774 (`672d02e593`) | Test-only (fixtures) | Four cases on v2 + v1 |

#2562's 43 carry rows are mapped there row by row. 42 are resolved with sync-head evidence; S3b 562 (D608) waits for #1800.

## Open residuals and owners

| Item | What is open | Owner | Blocks landing? |
| --- | --- | --- | --- |
| Windows desktop checklist | D396/D416/D448 plus step 4b/#1803; #1800, #1798, #1851 drafts; D608 | Travis (scheduling); coordinator | No, if deferred under Δ4 |
| PF-23-S04 case 11 | Blocked until a Windows guest exists (D673); cases 7/8 dispositions (D671/D672) and the Linux leg (D669) are historical | Coordinator / QA | Final gate 5: needs an explicit disposition |
| D685 | Implemented on both branches (#2501, #2522, #2675/#2676, #2719/#2721). The landing plan asks for Travis's sign-off; this record is it | Travis at SG2 | Signed here |
| #2783 | PATHEXT lookup trims PATH entries; check the untrimmed spelling too (low) | sec-followups successor | No |
| #2743, #2756 | Remote PATH program can shadow a module cmdlet; D729 follow-ups (missing PATH, pipeline binding, `-EncodedCommand`, mapped drives) — all low | Security owner | No |
| #2780 | Main port of the D377 TEMP-spelling fix (#2779) | Windows maintainer | No (main-only) |
| #2778 | Closed by #2789; no open follow-up found | — | No |
| D742 | 3 `agent_message_wake` Windows cases; not a kill-switch bypass | Travis (decision), sync-win2729b successor | Yes for green CI |
| D738 / D741 | Pending (Δ3, Δ2) | Travis | Decide at SG2 |
| #2562 comment (V1 discovery) | Cached `SKILL.md` served without a fresh read through the call's file-tool context (low, optional hardening) | Security owner | No |
| #2122 | Four MCP cases CLI-blocked; helper build now possible | Test-isolation maintainer | Coverage gap (matrix) |
| #2728 | Remaining CLI/TUI failures; two TUI cases wait on D738 | Sync TUI maintainer | For green CI |
| #2670 | One app-server-client shutdown failure on Windows; diagnostics only | Windows maintainer | No |
| [#2796](https://github.com/CorbanuCore/CorbanuTerminal/issues/2796) | Linux `rust-ci-core-security` red: `pf_23_s01::issue_1472…` does not recognise the Aggressive prompt whose reason joins D691's rule reason and the user-namespace note. The prompt is raised and nothing runs unasked; the test's matcher needs fixing (and the joined notes lack a sentence break) | Sync security owner | Yes: the lane must be green on the candidate |
| [#2808](https://github.com/CorbanuCore/CorbanuTerminal/issues/2808) | On macOS CI, `issue_747`'s Permissive Retry row wrote outside the sandbox after one approval. D639(a) expects no retry outside. This matches main's old behaviour, so nothing is weaker than main, but Δ5 may not hold on macOS. One occurrence; rerun pending | Sync security owner | Yes: fix it, or Travis accepts a macOS exception at SG2 |
| [#2797](https://github.com/CorbanuCore/CorbanuTerminal/issues/2797) | 5 MXC `exec-server` `windows_sandbox` cases fail natively over SSH (`0xC0000142`; MXC TEMP is the package folder). Not yet shown to be environmental; no main counterpart | Windows/MXC maintainer | Yes, until run in the interactive session or CI |
| Windows SSH-session failures | 5 `codex-windows-sandbox` tests fail over SSH on sync **and** main alike (ConPTY, window station, D324 race from a high-integrity session); `windows-security-probes` covers them from a medium session | — (parity; the D324 race itself is #1721, addressed by draft #1800) | No |
| Linux sandbox flake | See run record: `a_command_without_bubblewrap_keeps_its_terminal` failed once in the full run, then passed 3/3 alone and in the full rerun | Security owner (file if it recurs) | No |

## Refreshing for the final candidate

1. Fix the candidate SHA: after #2791 and the remaining fixes, and after `ci_mode.py revert`. Replace `ab557d87af` and main `15415856d7` everywhere above.
2. Dispatch the same lanes on the candidate and refresh the run-record links and results:
   ```sh
   gh workflow run rust-ci-core-security.yml --ref <branch> -f runner=ubuntu-24.04 -f target=x86_64-unknown-linux-gnu
   gh workflow run rust-ci-core-security.yml --ref <branch> -f runner=macos-15 -f target=aarch64-apple-darwin
   gh workflow run security-platform-contract.yml --ref <branch>
   gh workflow run security-ingress-contract.yml --ref <branch>
   gh workflow run credential-boundary-canary.yml --ref <branch>
   gh workflow run windows-security-probes.yml --ref <branch>
   gh workflow run windows-integration-tests.yml --ref <branch>
   ```
   Plan for macOS: a dispatched `rust-ci-core-security` run starts with a cold cache. Its first attempt here hit the 60-minute job limit, and the build alone took 37 minutes. Allow one rerun.

   Then rerun the three local suites with the same commands:
   - glitch: `cargo test -p codex-linux-sandbox -p codex-sandboxing`;
   - Mac: `-p codex-sandboxing` and `-p codex-exec --test all -- suite::seatbelt` at `-j6`, with at least 25% memory free;
   - desktop: `-p codex-windows-sandbox` and `-p codex-exec-server --test exec_process -- windows_sandbox`.
3. Close the matrix gaps from the full final CI, or keep them listed by name.
4. Add rows for any security-path PR merged after `ab557d87af` (including #2791's carries) to the fail-before/pass-after table.
5. Update the Δ table with Travis's answers to the For SG2 items. Then mark this PR ready for his signature on its exact head.
