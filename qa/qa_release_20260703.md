# PfTerminal QA Release Report - 2026-07-03

Evidence in this report is cited using repo-relative paths under
`qa/evidence_20260703/`. The uncopied raw evidence tree remains at
`/home/pfrpc/repos/pfterminal_qa_20260703/`.

## 1. Executive Summary

Alex's mandate for the overnight release was:

> "proper code review of everything we did today; make sure dispatch tools work -
> especially nazgûl routing, trolls, orcs; get to a quality gate; re-release the app
> (linux/mac/windows) so it is installable; then reset the pfterminal install on this
> machine, verify it loads, deal with QA bugs; report findable + committed with evidence;
> TUI testing required."

Evidence: `/home/pfrpc/repos/orc_directives/overnight_qa_release_mandate_20260703.md`.

What shipped in Phase C: PfTerminal `0.1.2`, tag `rust-v0.1.2`, from commit
`9956f0336f921c12901cfd8fef895f7cb465274f`, with Linux, macOS, and Windows release
artifacts attached to a successful release workflow.

Evidence: `qa/evidence_20260703/status/phase_c_status.md`,
`qa/evidence_20260703/phase_c/release_view.json`.

Quality story: 6 code-review blockers plus 4 live TUI QA failures went through 5 verified
fix layers, a 3x live verifier gate, a final Phase A sweep, and then the tri-platform
release.

Evidence: `qa/evidence_20260703/status/review_findings.md`,
`qa/evidence_20260703/status/tui_qa_findings.md`,
`qa/evidence_20260703/status/phase_b_status.md`,
`qa/evidence_20260703/status/round5_tasknode_3x_result.env`,
`qa/evidence_20260703/round5_verifier/final_sweep/final_sweep_verdict.env`,
`qa/evidence_20260703/status/phase_c_status.md`.

## 2. Phase A Findings

### 2.1 Adversarial Code Review Findings

Source evidence: `qa/evidence_20260703/status/review_findings.md`.

| ID | Severity | Area | Finding | Phase A gate |
| --- | --- | --- | --- | --- |
| R1 | P1 | OpenRouter stream hardening | Comment-only OpenRouter keepalives still hit the shorter actionable-silence timeout before the intended long transport idle window. | Blocker |
| R2 | P1 | Spawn report delivery | Pending child reports could be drained before the parent turn accepted them, losing reports after queue or turn-start failures. | Blocker |
| R3 | P1 | TUI release gate | `codex-tui` was red, including the `AuthManager` runtime-layer dependency regression. | Blocker |
| R4 | P1 | `ModelProviderInfo` blast radius | New `chat_completions_provider` field broke downstream test initializers in `codex-login` and `codex-app-server`. | Blocker |
| R5 | P1 | App-server protocol schema | Schema/fixture tests were red, including duplicate `ThreadStartParams` exposure and missing generated auth-status fixture fields. | Blocker |
| R6 | P1 | Core release gate | `codex-core` had broad red clusters across role application, config/provider behavior, tool routing, guardian, shell, and snapshots. | Blocker |

### 2.2 Live TUI QA Findings

Source evidence: `qa/evidence_20260703/status/tui_qa_findings.md`.
Representative captures: `qa/evidence_20260703/phase_a_tui_representative/`.

| Severity | Area | Finding | Representative evidence |
| --- | --- | --- | --- |
| P0/P1 | Spawn routing | Targeted child tasks duplicated or leaked into Main; Snaga work visibly ran as a Main prompt and child reports followed the wrong parent path. | `phase_a_tui_representative/036_burzum_task_confirm_retry.txt`, `041_snaga_task_wait20.txt`, `042_status_after_snaga_task.txt` |
| P1 | Child report visibility | Child results were visible in `/spawn status` but did not surface cleanly back to Main as reports. | `phase_a_tui_representative/036_burzum_task_confirm_retry.txt`, `042_status_after_snaga_task.txt` |
| P1 | Main responsiveness | After dispatch stress, Main did not complete a file-edit prompt and then did not answer a simple exact-response prompt. | `phase_a_tui_representative/047_file_edit_wait35.txt`, `051_main_responsive_wait25.txt` |
| P2 | Slash/picker stickiness | `/help`, `/spawn`, and `/model` often required a second Enter; spawn picker text filtering was inconsistent. | `phase_a_tui_representative/043_model_picker.txt` |

Phase A also recorded passing startup/status, basic spawn creation, model picker visibility,
and closed-stdin exec behavior, but the P0/P1 dispatch and Main-responsiveness failures
blocked release.

Evidence: `qa/evidence_20260703/status/tui_qa_findings.md`,
`qa/evidence_20260703/phase_a_tui_representative/045_exec_devnull.status`.

## 3. Phase B Fix and Verification Ledger

Primary ledger: `qa/evidence_20260703/status/phase_b_status.md`.

### 3.1 Fix Layers

| Layer | Commit(s) | What changed | Verification evidence |
| --- | --- | --- | --- |
| 1 | `3fefcd6f9`, `fc62a3eab` | Fixed downstream `ModelProviderInfo` initializers, updated generated auth-status fixture, and removed the TUI runtime dependency on `AuthManagerConfig`. | `status/phase_b_status.md` entries at 04:06 and 04:10 |
| 2 | `31b591e47`, `b096e82be` | Added checked/retryable child-report delivery, visible Main report surfacing, operator-pane isolation, and removed misleading automatic report-processing task toasts. | `status/phase_b_status.md` entries at 04:37, 04:32, and 04:47 |
| 3 | `9013d06f0` | Fixed core/config/tool regressions: ambient model normalization, role reload model preservation, and deferred tool-search metadata exposure. | `status/phase_b_status.md` entry at 04:56 |
| 4 | `60014b5f6` | Kept chat-completions streams alive on SSE comments, preserved true-silence failure, exposed built-in provider transport knobs, and capped same-request idle retries. | `status/phase_b_status.md` entries at 05:00 and 05:04 |
| 5 | `37698e1d0`, `f76b2de86` | First fixed stale active-turn routing for visible idle panes, then fixed the real remaining TUI submission boundary: long single-line paste bursts swallowing Enter instead of submitting. | `status/phase_b_status.md` entries at 05:33, 06:36, 06:48, and 07:12 |

### 3.2 Round-by-Round Verifier Story

| Round | Build / head | Result | What it proved | Evidence |
| --- | --- | --- | --- | --- |
| Readiness | `d86721e33` | Blocked | No verifier run before a fixer-ready marker. | `status/phase_b_status.md` 03:56 entry |
| R2 initial | `31b591e47` | Fail | Duplicate Main toast improved, but Burzum report visibility, Snaga parent routing, and Main responsiveness still failed. | `status/phase_b_status.md` 04:32 entry, `key_scans/phase_b_initial_key_scan.txt` |
| R2 follow-up | `60014b5f6` | Fail | Spawn hierarchy, operator-pane isolation, Burzum report visibility, and Snaga isolation passed; Main file-edit/exact-response still hung. | `status/phase_b_status.md` 05:16 entry, `round2_round3_representative/round2_032_main_responsive_wait25.txt` |
| R3 | `37698e1d0` | Fail | Stale active-turn routing fix was valid but incomplete; Main file-edit and Ctrl-C recovery exact-response still produced no outbound turn. | `status/phase_b_status.md` 05:46 entry, `round2_round3_representative/round3_032_main_responsive_wait25.txt` |
| R4 arbitration | `37698e1d0` | Mixed; held | Gorkul could not reproduce the Main hang and stopped; tasknodeorc reproduced it with method logs, proving the failure boundary was before app-server turn start. | `status/main_responsiveness_dossier.md`, `round4_method/method_log_extract.txt`, `round4_method/result.env` |
| R5 | `f76b2de86` | Pass | Three independent live runs emitted immediate file-edit and post-Ctrl-C `UserTurn` events with zero concatenation, then the final Phase A sweep passed the priority gate. | `status/round5_tasknode_3x_result.env`, `round5_verifier/attempt1/result.env`, `attempt2/result.env`, `attempt3/result.env`, `round5_verifier/final_sweep/final_sweep_verdict.env` |

### 3.3 Root Causes That Mattered

Spawn/report failures had multiple boundaries, not one bug. Report delivery originally
treated "queued event submitted" as equivalent to "parent turn accepted report"; the fix
kept retry state until acceptance and made failures checked. Separately, operator-pane
selection switched the human into child panes after spawn, which made later prompts land
in the wrong surface. Both were fixed and then verified through Round 2.

Evidence: `qa/evidence_20260703/status/phase_b_status.md` entries at 04:37, 04:47, and
05:16; `qa/evidence_20260703/round5_verifier/final_sweep/019_spawn_status_initial.txt`.

The first Main-responsiveness fix addressed a real stale active-turn bug: a visible idle
Main prompt could still be submitted as `turn/steer` against a cached active turn instead
of starting a fresh turn. That was tested and preserved, but it did not explain the final
hang reproductions.

Evidence: `qa/evidence_20260703/status/phase_b_status.md` entries at 05:33 and 05:46.

The decisive Round 4 logs showed a different boundary: after a fast, long, single-line
prompt was injected, the TUI appended visible history later but emitted no `from_tui`
`UserTurn`; the later sleep prompt was concatenated into the same delayed append. The
Round 5 fix made Enter submit after long single-line paste bursts outside slash-command
context while preserving multiline paste behavior.

Evidence: `qa/evidence_20260703/round4_method/method_log_extract.txt`,
`qa/evidence_20260703/round4_method/main_prompt_userturn_summary.txt`,
`qa/evidence_20260703/status/phase_b_status.md` entry at 06:48.

The final gate was not a single lucky run. The verifier required and obtained 3/3 passes,
then ran the final sweep covering startup, spawn, Orc-before-Troll guard, Troll/Orc spawn,
Burzum report visibility, Snaga isolation, Main file-edit submission, Ctrl-C recovery,
model picker, exec closed-stdin behavior, and key scan.

Evidence: `qa/evidence_20260703/status/round5_tasknode_3x_result.env`,
`qa/evidence_20260703/round5_verifier/final_sweep/final_sweep_verdict.env`,
`qa/evidence_20260703/status/phase_b_status.md` entry at 07:12.

## 4. Phase C Release Evidence

Phase C source: `qa/evidence_20260703/status/phase_c_status.md`.
Release metadata: `qa/evidence_20260703/phase_c/release_view.json`.

| Item | Result | Evidence |
| --- | --- | --- |
| Merge PR | PR #22, `QA release fixes 2026-07-03`, merged as `26eaec0e09f16c2f73c89d2c3ba06c41d3f331e9`. | `status/phase_c_status.md` |
| Version/tag | Version bump commit `9956f0336f921c12901cfd8fef895f7cb465274f`; tag `rust-v0.1.2`. | `status/phase_c_status.md` |
| Release URL | `https://github.com/agtico/PfTerminal/releases/tag/rust-v0.1.2`. | `status/phase_c_status.md`, `phase_c/release_view.json` |
| Workflow | GitHub Actions run `28645682785` succeeded; validate, Linux, macOS, Windows, and assemble jobs passed. | `status/phase_c_status.md` |
| Linux installability | Downloaded Linux GNU package checksum verified OK; `pfterminal --version` and bundled `codex --version` both reported `codex-cli 0.1.2`. | `phase_c/linux_checksum_check.txt`, `linux_pfterminal_version.txt`, `linux_codex_version.txt` |

### 4.1 Release Assets

| Asset | Size | SHA256 | Evidence |
| --- | ---: | --- | --- |
| `pfterminal-package-x86_64-unknown-linux-gnu.tar.gz` | 219479996 | `22e639fd0d3c4d2889a25bb0285b7d5d30005dee552a80bb6ad8929f284263d9` | `status/phase_c_status.md`, `phase_c/pfterminal-package_SHA256SUMS` |
| `pfterminal-package-aarch64-unknown-linux-musl.tar.gz` | 208255070 | `842148058dcfb7c7a7dd71e54493451c8b0eb50df2049c32f8240e011b5682a6` | `status/phase_c_status.md`, `phase_c/pfterminal-package_SHA256SUMS` |
| `pfterminal-package-aarch64-apple-darwin.tar.gz` | 200656076 | `3c5cf6ddc41d842b25fd8ae9d7cabdeceb20191f5ea431f64db64985d881cdd0` | `status/phase_c_status.md`, `phase_c/pfterminal-package_SHA256SUMS` |
| `pfterminal-package-x86_64-apple-darwin.tar.gz` | 211583767 | `110e06ce8da23f0870560e81ece34e335b53d1be08c3b12241e361522c44a9b5` | `status/phase_c_status.md`, `phase_c/pfterminal-package_SHA256SUMS` |
| `pfterminal-package-x86_64-pc-windows-msvc.zip` | 236569901 | `2edc9cd92b793ff4a2823e3edca435da8d34e05bd337ed62b87a5ceb474da1be` | `status/phase_c_status.md`, `phase_c/pfterminal-package_SHA256SUMS` |
| `PFTerminal-aarch64-apple-darwin.dmg` | 202140320 | `bd9fe05a7bd2284ee33a3cb3980597f9655641f017a4a88a16e0cdbe358b3f3d` | `status/phase_c_status.md`, `phase_c/pfterminal-dmg_SHA256SUMS` |
| `PFTerminal-x86_64-apple-darwin.dmg` | 213294925 | `e2fae9a4be68188fe7f964447b207554cc37f23a7ca969c9403be2cfc0395be9` | `status/phase_c_status.md`, `phase_c/pfterminal-dmg_SHA256SUMS` |

macOS and Windows artifacts were verified as present with checksums, but were not executed
on this Linux host.

Evidence: `qa/evidence_20260703/status/phase_c_status.md`.

## 5. Phase D Local Install Reset and Fresh-Install QA

Phase D result: PASS. No blocking install or PfTerminal `0.1.2` code defects were found.

Primary evidence: `qa/evidence_20260703/status/phase_d_status.md`.

| Gate | Result | Evidence |
| --- | --- | --- |
| Backup before reset | PASS. `~/.pfterminal` was backed up before any local reset; source size was `1.6G`, backup tarball size was `531M`, and the backup SHA256 was recorded. | `status/phase_d_status.md` |
| Reset + fresh install | PASS. Old launchers/state were moved aside, release `0.1.2` was installed through the release installer, the Linux package checksum verified OK, and `pfterminal --version` reported `codex-cli 0.1.2`. | `status/phase_d_status.md`, `phase_d/install/download_checksum_verify.txt`, `phase_d/install/version_after_install.txt`, `phase_d/install/which_pfterminal_after_install.txt` |
| Fresh-load TUI QA | PASS. A clean installed build loaded the provider picker without panic, launched with Vercel env-key auth, returned `QA_PHASE_D_BASIC`, spawned Burzum and Snaga, showed Main current with both children addressable, delivered `QA_PHASE_D_TROLL_REPORT`, kept Main responsive with `QA_PHASE_D_MAIN_OK`, exited cleanly, and relaunched cleanly. | `phase_d/fresh_tui/fresh_qa_analysis.txt`, `phase_d/fresh_tui/001_initial_noauth_provider_picker.txt`, `phase_d/fresh_tui/011_env_loaded.txt`, `phase_d/fresh_tui/012_basic_wait25.txt`, `phase_d/fresh_tui/026_spawn_status_initial.txt`, `phase_d/fresh_tui/029_burzum_report_wait25.txt`, `phase_d/fresh_tui/031_main_responsive_wait25.txt`, `phase_d/fresh_tui/040_relaunch_loaded.txt`, `phase_d/fresh_tui/043_relaunch_spawn_status.txt` |
| Restore usability | PASS. The backed-up config/auth/vault/session/plugin state was restored while preserving the installed `0.1.2` standalone package, and a no-env-key Ambient plan-auth exec returned `QA_PHASE_D_RESTORE_OK` with exit status `0`. | `phase_d/restore/restore_overlay_log.txt`, `phase_d/restore/restored_auth_paths_listing.txt`, `phase_d/restore/ambient_restore_exec.status`, `phase_d/restore/ambient_restore_exec.stdout`, `phase_d/restore/ambient_restore_exec.stderr` |
| Final machine state | PASS. `which pfterminal` resolved to `/home/pfrpc/.local/bin/pfterminal`; installed launcher and standalone binary reported `codex-cli 0.1.2`; no `phase_d_tui` tmux session, `pfterminal`, or `sleep 120` process was left running. | `status/phase_d_status.md` |
| Phase D key scan | PASS. Fresh TUI evidence, clean fresh TUI evidence, install evidence, and restore evidence scanned with `hit_count=0`. | `key_scans/phase_d_key_scan.txt`, `phase_d/fresh_tui/key_scan.txt` |

Phase D observations were non-blocking and are listed in Known Issues / Deferred Work.

## 6. Known Issues and Deferred Work

| Item | Status | Evidence |
| --- | --- | --- |
| `codex-app-server-protocol` generated fixture red | Documented as pre-existing/base-red after regression fixes; the `GetAuthStatusResponse.hasCodexBackendAuth` regression was fixed, while the remaining `v2/ThreadItem.ts` `taskPreview` fixture mismatch stayed outside the Phase B regression gate. | `status/phase_b_status.md` entries at 04:06 and 05:04 |
| Full `codex-tui` snapshot/model-picker/status cluster | Documented as unrelated/pre-existing during Phase B; targeted TUI regression tests and the live final sweep passed. | `status/phase_b_status.md` entries at 04:10, 05:33, and 07:12 |
| Guardian retry/denial cluster | `guardian_review_does_not_retry_valid_denial` was red at the pre-merge base with the same failure, so it stayed outside the Phase B regression gate. | `status/phase_b_status.md` entry at 04:56 |
| R1 stream residuals | The release fixed the observed OpenRouter comment-keepalive/actionable-silence bug, true-silence behavior, built-in provider transport overrides, and same-request idle retry cap. Broader resume semantics and provider-pinning strategy remain future hardening, not part of this release gate. | `status/phase_b_status.md` entry at 05:00 |
| Gorkul P2 bubbled-report render lag | Gorkul Round 4 observed Snaga done/status correct while the final bubbled Burzum report did not visibly render before checkpoint; Main remained responsive, so this stayed P2. | `status/main_responsiveness_dossier.md`, `status/phase_b_status.md` entry at 06:11 |
| P2 slash command stickiness | Still reproduces in final sweep; first `/spawn` Enter can leave `/spawn` in the composer and require a second Enter. | `round5_verifier/final_sweep/003_spawn_orc_before_troll_first_enter.txt`, `round5_verifier/final_sweep/final_sweep_verdict.env` |
| Strict file-content exactness | Final sweep strict checker recorded `strict_pass_all=0` because the model wrote `QA_FILE_EDIT_OK\n`; verifier classified this as model output variance because the UI emitted immediate `UserTurn` and completed the file-edit turn. | `round5_verifier/final_sweep/final_sweep_verdict.env` |
| Phase D effort-suffix display observation | The clean installed TUI displayed `zai/glm-5.2-fast` without an `xhigh` effort suffix when launched with `-m zai/glm-5.2-fast` and no explicit effort override. This was observed but was not a Phase D blocker because the directive did not define effort display as an acceptance gate. | `status/phase_d_status.md`, `phase_d/fresh_tui/011_env_loaded.txt` |
| Phase D discarded no-auth probe state | The first no-auth probe accidentally selected Ambient from the provider picker and treated the typed `/providers` probe as an API-key entry because of verifier input sequencing. That fresh home was moved aside at `/home/pfrpc/.pfterminal.phaseD-contaminated-20260703T093246Z`, and the clean rerun passed from a newly reinstalled fresh state. | `status/phase_d_status.md`, `phase_d/install/clean_reinstall_after_harness_contamination.txt` |
| PR #21 / Fable | Left untouched and awaiting Alex, per release mandate and Phase C status. | `/home/pfrpc/repos/orc_directives/overnight_qa_release_mandate_20260703.md`, `status/phase_c_status.md` |
| Snapshot hygiene product fix | Previous benchmark evidence had generated shell snapshots with secret-risk history; the product-level snapshot hygiene review remains awaiting Alex. This report's committed evidence excludes generated shell snapshots and includes a fresh `qa/` key scan. | `/home/pfrpc/repos/orc_directives/nazgul_overnight_state_20260702.md`, `key_scans/repo_qa_key_scan.txt` |
| Nested bwrap sandbox fail-fast improvement | The QA mandate required non-nested bwrap coverage in the worker shell; nested-sandbox failure handling remains a suggested product hardening item, not a Phase B/C blocker. | `/home/pfrpc/repos/orc_directives/tasknodeorc_directive_qa_tui_0325.md` |

## 7. Evidence Appendix

Curated evidence copied into the repo: `qa/evidence_20260703/`.

| Subtree | Contents | Notes |
| --- | --- | --- |
| `status/` | Phase A findings, Phase B ledger, Phase C status, final Phase D status, main-responsiveness dossier, and Round 5 aggregate env. | Status files are the main citation surface for this report. |
| `phase_c/` | Release view JSON, checksum manifests, Linux checksum output, Linux extracted binary version outputs, and executable inventory. | Large release archives and extracted binaries were intentionally omitted. |
| `phase_d/` | Install checksum/version proof, clean fresh-load TUI captures, restore overlay logs, and plan-auth exec proof. | Large packages, backup tarballs, full raw session logs, and restored secrets are omitted. |
| `key_scans/` | Phase A/B/D scan results plus the fresh whole-`qa/` scan result. | All copied scan files report zero hits. |
| `phase_a_tui_representative/` | Representative Phase A failure captures for dispatch, Main responsiveness, model picker, and closed-stdin exec. | Full Phase A capture directory remains in the raw evidence tree. |
| `round2_round3_representative/` | Representative Main-responsiveness failure captures from failed verifier rounds. | Full round directories remain in the raw evidence tree. |
| `round4_method/` | Method-level arbitration logs and selected captures proving no outbound Main `UserTurn` before the paste-burst fix. | The full session JSONL remains in the raw evidence tree. |
| `round5_verifier/` | 3x pass envs/logs/captures plus final sweep verdict, analysis, and representative captures. | Enough evidence is copied to audit the final gate without storing the full raw capture set. |

The curated copy is intentionally small. After adding Phase D and the final repo-level
key-scan record, it contained 94 files and was about 512 KiB.

Evidence: local assembly command output; raw tree at `/home/pfrpc/repos/pfterminal_qa_20260703/`.

### 7.1 Omitted Artifacts

The following artifacts were left out of the repo copy to avoid committing gigabytes or
duplicated/generated material:

- Release package tarballs, including
  `/home/pfrpc/repos/pfterminal_qa_20260703/phase_c_release_0.1.2/pfterminal-package-x86_64-unknown-linux-gnu.tar.gz`
  and the duplicate Phase D Linux package.
- Extracted release binaries under
  `/home/pfrpc/repos/pfterminal_qa_20260703/phase_c_release_0.1.2/linux_extract/`.
- Scratch workspaces under `/home/pfrpc/repos/pfterminal_qa_20260703/scratch*`.
- Full TUI capture directories and full session JSONL logs, except selected method extracts
  and representative captures copied under `qa/evidence_20260703/`.
- Backup tarballs and local install move-aside artifacts referenced by Phase D status.

All omitted artifacts remain on this machine under `/home/pfrpc/repos/pfterminal_qa_20260703/`
or the explicit backup paths recorded in `qa/evidence_20260703/status/phase_d_status.md`.

### 7.2 Key Scan

Whole-tree `qa/` key scan was run after report and evidence assembly.

Evidence: `qa/evidence_20260703/key_scans/repo_qa_key_scan.txt`.
