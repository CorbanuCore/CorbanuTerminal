# Main Responsiveness Round 4 Dossier

Timestamp: 2026-07-03 06:11 UTC

Checkout: `/home/pfrpc/repos/PfTerminal-bench`
Branch: `qa/release-fixes-20260703`
HEAD: `37698e1d0848` (`Fix stale active-turn routing for idle panes`)
Binary: `/home/pfrpc/repos/PfTerminal-bench/codex-rs/target/debug/pfterminal`
Provider/model: `vercel-anthropic-fast` + `zai/glm-5.2-fast`
Route confirmation: `tui_evidence_phase_b_round4_gorkul_attempt3/session_log.jsonl` session header has `model_provider_id=vercel-anthropic-fast`.

## Gate Result

STOP: I did not change code in this round.

The Round 4 mandate required live reproduction before code edits. I ran the live TUI sequence and could not reproduce the Round 3 Main hang on this checkout. Because the target failure did not reproduce, the correct gate action is to stop and leave this dossier rather than make a fourth blind fix.

## Evidence Locations

- Primary successful baseline: `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_b_round4_gorkul`
- Timing-faithful queued-slash attempt: `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_b_round4_gorkul_attempt3`
- Timing probe that still completed the initial prompt before spawn: `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence_phase_b_round4_gorkul_attempt2`
- Scratch for primary successful baseline: `/home/pfrpc/repos/pfterminal_qa_20260703/scratch_phase_b_round4_gorkul`
- Scratch for timing-faithful attempt: `/home/pfrpc/repos/pfterminal_qa_20260703/scratch_phase_b_round4_gorkul_attempt3`

Key scan over new Round 4 evidence: PASS, no matches for key/header patterns.

## What I Observed

Attempt 1 followed the same user-visible sequence after allowing the startup prompt to settle:

- Spawned Burzum and Snaga.
- Burzum exact-response task completed and reported `QA_TROLL_REPORT_DELIVERED`.
- Snaga exact-response task completed and bubbled through Burzum with `QA_ORC_REPORT_DELIVERED`.
- `/spawn status` showed Main current, Burzum done, Snaga done.
- Main file-edit prompt started and completed within the 45-second checkpoint.
- `qa_loop.txt` existed with exactly `QA_FILE_EDIT_OK`.

Evidence:

- `tui_evidence_phase_b_round4_gorkul/022_status_after_snaga.txt`
- `tui_evidence_phase_b_round4_gorkul/023_file_edit_wait45.txt`
- `tui_evidence_phase_b_round4_gorkul/024_file_edit_wait90.txt`

Attempt 3 forced the startup timing closer to the Round 3 captures by submitting `/spawn` immediately while the initial Main turn was still active. The TUI queued `/spawn`, then replayed it into the command UI after the initial turn completed. Main still passed the file-edit checkpoint:

- The queued `/spawn` opened the spawn menu after startup completion.
- Burzum and Snaga spawned successfully.
- Burzum exact-response task completed.
- Snaga reached `done` with `latest result: QA_ORC_REPORT_DELIVERED`.
- Main file-edit prompt emitted a fresh `UserTurn` in `session_log.jsonl` at `2026-07-03T06:08:55.102Z`.
- Main created `qa_loop.txt` and replied `QA_FILE_EDIT_DONE` by `2026-07-03T06:08:57.947Z`.

Evidence:

- `tui_evidence_phase_b_round4_gorkul_attempt3/003_after_queued_spawn_wait12.txt`
- `tui_evidence_phase_b_round4_gorkul_attempt3/018_status_after_snaga_hang.txt`
- `tui_evidence_phase_b_round4_gorkul_attempt3/019_file_edit_wait45.txt`
- `tui_evidence_phase_b_round4_gorkul_attempt3/session_log.jsonl`

## Ctrl-C Recovery Check

The original Round 3 Ctrl-C step depended on the file-edit turn hanging. Because the file-edit turn completed, I ran a controlled active-turn interrupt instead:

- Submitted `Run the shell command sleep 120, then reply QA_SLEEP_DONE.`
- Sent Ctrl-C while the turn was active.
- TUI recorded `AppCommand::Interrupt { behavior: RestorePromptIfNoOutput }` at `2026-07-03T06:10:39.749Z`.
- Submitted `Reply exactly QA_MAIN_RESPONSIVE.`
- Main replied `QA_MAIN_RESPONSIVE` within the 25-second window.

Evidence:

- `tui_evidence_phase_b_round4_gorkul_attempt3/021_sleep_ctrlc.txt`
- `tui_evidence_phase_b_round4_gorkul_attempt3/022_main_responsive_after_active_ctrlc.txt`
- `tui_evidence_phase_b_round4_gorkul_attempt3/session_log.jsonl`

## Remaining Signal

Attempt 3 did expose a related but non-blocking routing/render divergence after Snaga:

- The Main transcript did not visibly show the final bubbled Burzum synthesis after Snaga by the 60-second checkpoint.
- `/spawn status` showed Snaga `done` with `latest result: QA_ORC_REPORT_DELIVERED`.
- `/spawn status` also showed Burzum `done`, current task set to the child-report review prompt, but `latest result` still displayed the earlier `QA_TROLL_REPORT_DELIVERED`.
- The session log shows the child-report prompt was submitted to Burzum at `2026-07-03T06:07:24.143Z`.

That is not the Round 3 Main-responsiveness failure: after this state, Main still accepted a fresh file-edit `UserTurn`, created the file, and replied. But it is worth tracking separately as "child-report synthesis/result display may lag or stale-display under queued slash timing."

## Turn Start vs Steer Evidence

The enabled session logger records TUI/core commands, not the app-server JSON-RPC method name. For the successful Main file-edit check, the live log has a fresh `from_tui` `AppCommand::UserTurn` at `2026-07-03T06:08:55.102Z`. For the post-interrupt exact-response check, it has a fresh `AppCommand::UserTurn` at `2026-07-03T06:10:44.772Z`.

I did not capture lower-level `turn/start` versus `turn/steer` JSON-RPC method logs in this run. The observable result is that neither prompt was a no-op or stuck steer: both produced visible model output and the file-edit turn modified the filesystem.

## Recommended Next Step

Do not patch Main routing again from this run. The actionable next step is to re-run tasknodeorc's exact verifier harness with session logging enabled, or add explicit app-server method logging to that harness, because the failure appears sensitive to verifier timing/state that I could not reproduce manually.

Release remains held until the verifier that produced Round 3 FAIL records a PASS or produces method-level logs showing the still-failing boundary.
