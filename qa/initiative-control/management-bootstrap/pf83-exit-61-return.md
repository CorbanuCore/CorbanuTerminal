# RETURN — pf83-exit-61

Action `pf83-exit-61`; allocation digest `a1f933b575a5132c2764e735c2e3f51a644ca111d5395cabdd617ae90e4c3466`; claim `97558b5e-90e9-44e4-bee7-dc61bec0cadb`; runtime `gpt-6-astra high`.
Read the brief first and verified its SHA-256: `ed796222810593e23cd4592ed4a110709baf141985fc1dd7bf409e74ea71ba76`.
Base/current HEAD `8ad587541e45e519dc0b5181ac5be36ecaf90b13`; branch `bootstrap/pf83-rebind-20260916`; worktree `/Volumes/CorbanuDrive/Corbanu/worktrees/pf83-rebind-20260916`; initially clean. No commit, push, live-profile access or subagent dispatch.

Routine evidence correction supporting PF-83-S01 (`in_progress`) under active plan `docs/plans/active/p0-security-levels.md`. Product heading **Permission selection confirmation — TO BUILD**, excerpt: “A submitted selection is not a confirmed change”; “Error, disconnect and restart recovery must not claim success or retry an uncertain change blindly.” Used the Corbanu Terminal development skill. No product/source behavior changes. New functional design/execution is not applicable to this evidence-only revision; the feature's existing independent exact-package gate remains open, with no newly claimed integrator acceptance of a waiver.

## Corrected claim and eleven-case record

[Corrected exit record](pf83-escalation-60-f10-f11-exit.md): permission/model picker key-event navigation, labels/current markers, consent/reasoning popup transitions and exact provider/model selection events are natively expressible **today**. Eight existing native navigation/identity/consent tests were rerun, plus the next-turn status-label test: **9/9 pass**. Existing fixtures provide the relevant coverage; duplicate tests would add no missing navigation proof.

The record has exactly eleven F01–F11 lines, each with its native subset and exact residual. All are partial against the complete original human case; F11's qualified subset is local navigation only, with actual-route neutrality explicitly unqualified. Native scripted inference is not actual supported-route inference. A combined synthetic model switch during pending permission confirmation is expressible but not added: it would not close the actual-route F03/F04/F05 requirement. That native combination is explicitly untested, not described as requiring packaging. F08 cancel/exposed-failure traversal and F10 unanswered-approval full-authority-pair assertions are likewise admitted native gaps.

Narrower earlier closures are explicit: F03 is truthful next-turn labeling without running-authority comparison; F04 is the post-Applied admission form; F05 exercises retained explicit accept/decline; F06 exercises continuation; F07 composes native RPC effects with stale-completion UI tests; F09 proves command-count/authority continuity; F10's renamed reload fixture proves request-time nonmutation only. No frozen original was edited or retired.

## Frozen production surface

[Complete production freeze and SHA-256 table](pf83-exit-61-production-freeze.md): **34 files**, each marked **final for this sprint** for the next review/package candidate; zero files currently expected to change. This is a proposed source freeze, not independent approval or completed qualification. Later source changes require re-freezing and affected evidence.

Derived from the complete relevant Git source history since the sprint base, with nineteen exact PF-83 commit identities, per-commit changed-path union, and current-byte hashes. The audit accounts for all 71 touched source-tree paths: 27 production Rust files, six generated protocol artifacts, one API README, and 37 separately listed test-only paths. It includes later Core/goal/status edits absent from the old sprint write_scope, and the formerly reserved shared event files that PF-83 actually touched. A final rehash verified **34/34**. No production digest was taken from a prior return.

The sprint file is outside this allocation's writable scope. The manager can cite these records; its independent-review and functional-execution checkboxes remain open. The historical integration receipt `ed5a837d583dd01aa1e1e144837b775aec76d138` is a merge of this branch's frozen `8ad587541`; those different hashes are not a brief/base mismatch.

## Exact validation

Read `docs/development/test-isolation.md` before tests. All commands ran from `codex-rs` using this checkout's guarded `just test`, `CARGO_TARGET_DIR=/Volumes/CorbanuDrive/Corbanu/.codex-work/targets/pf83-rebind-31`, `NEXTEST_TEST_THREADS=4`, `INSTA_UPDATE=no`, `--locked --offline --retries 0`. Synthetic disposable profiles only; no native credential prompt observed. No raw Cargo test/nextest, whole-crate lane, workspace formatter or fix tool. No Rust edit, hence no Rust formatting was needed.

Prerequisites built first: `cargo build --locked --offline -p codex-cli -p codex-rmcp-client -p codex-code-mode-host --bins`, exit 0.

| Command suffix after `just test` (common flags above) | Run / pass / fail / skipped | Exit | Raw log suffix |
| --- | --- | --- | --- |
| `-p codex-app-server thread_settings` | 26 / 26 / 0 / 1082 | 0 | thread-settings |
| `-p codex-app-server settings_confirmation` | 11 / 11 / 0 / 1097 | 0 | settings-confirmation |
| `-p codex-core session::tests` | 268 / 268 / 0 / 3401 | 0 | core-session |
| `-p codex-tui permission_confirmation` | 12 / 12 / 0 / 4161 | 0 | permission-confirmation |
| `-p codex-tui -E '<nine exact-name alternatives below>'` | 9 / 9 / 0 / 4164 | 0 | picker-route |

Required gate: **317 passing test executions**, plus **9 focused supporting executions**; **326 total**, not a unique-test count because required filters overlap. Exact failure names: **none**. No retry or zero-test lane.

One non-failure qualification: nextest marked `suite::v2::experimental_api::thread_settings_update_requires_experimental_api_capability` **LEAK**, 21.950 seconds, in thread-settings (summary: 26 passed, 1 leaky). This passing-but-leaky process/pipe teardown observation is preserved; it is not represented as a clean teardown or silently omitted. No source change or retry was made for it.

Supplemental exact filter (the test names also identify the source evidence in the corrected exit record):

```text
test(permissions_selection_requests_full_access_after_consent) | test(permissions_selection_requests_restriction_without_optimistic_mutation) | test(permissions_selection_requests_change_without_optimistic_history) | test(permissions_selection_requests_confirmation_when_current_is_selected) | test(permissions_full_access_consent_does_not_claim_application) | test(model_picker_dismisses_after_selecting_openrouter_model_without_effort_choices) | test(model_picker_same_slug_marks_only_exact_provider_current) | test(custom_model_routes_are_distinguishable_before_selection) | test(status_permissions_compact_next_turn_label_at_narrow_widths)
```

Raw logs are local ignored files alongside this receipt, named `pf83-exit-61-<suffix>.log`; they were not overwritten. 2370 generated lines / 115559 bytes, separate from authored evidence changes.

| Raw log | SHA-256 |
| --- | --- |
| `pf83-exit-61-core-session.log` | `3aa9cb3724ac037647265ea71a9703390aeae9eb3b628eb438291a24e79ca33a` |
| `pf83-exit-61-permission-confirmation.log` | `79863fb7f5b14d02373577b6759ede6bbb5bd7fd853222e508e948a7c1c0e26a` |
| `pf83-exit-61-picker-route.log` | `c0ce0eb5ee7ef63df0b60ed65780f2330002a14e4570fb2c38738d6d6fd48732` |
| `pf83-exit-61-prerequisites.log` | `ea4ee1ba61dca6f81a5cdc9372e820efb1ffe75e54b93a46f4aa580e5f0fbde4` |
| `pf83-exit-61-settings-confirmation.log` | `7075adb09c4a9b6c57387563f44cf71fdde3775a76b2118718ede5749da88a2c` |
| `pf83-exit-61-thread-settings.log` | `c8f3d1a4d1d981ee3654add88a78a80a929fef856b98e92c0805cd93d51d92a1` |

## Changed lines, checks and brief corrections

- `pf83-escalation-60-f10-f11-exit.md`: **+38/-10** (48 changed lines), replacing the ten-line overstatement with native evidence, eleven case dispositions and exact residuals.
- `pf83-exit-61-production-freeze.md`: **+118/-0**; complete production hash inventory, provenance and test-only exclusions.
- This receipt: **+68/-0**. Total authored change: **+224/-10 = 234 changed lines**. Zero Rust/test/snapshot/production lines changed; all three edited/created tracked candidates are under the authorized QA prefix. Raw ignored logs are under the same prefix.
- `git diff --check` passed; final writable-scope and 34-digest/eleven-case audits passed. `python3 -B docs/sprints/check.py` passed: **115 current / 127 archived**.
- The brief's P3 is correct: packaging was overstated as necessary for navigation. No substantive error found in the new assignment. “F11 unbuilt” must be qualified: local navigation exists, actual supported-route neutrality remains unbuilt in these fixtures. A package alone also cannot supply F10's pre-application barrier or F11's mediated routes.
- Additional tree discrepancy preserved: the F04 fixture's old “pending product decision” comment predates the recorded re-binding ruling. It is not current authority; this evidence revision leaves source bytes unchanged.
- No new true-TUI/code-blind execution, native credential qualification, live-repository proof, human sign-off, benchmark, release or human-test-readiness claim. Evidence correction and source freeze do not waive those remaining gates.
