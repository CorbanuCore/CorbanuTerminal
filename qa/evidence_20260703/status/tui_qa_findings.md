# PFTerminal Phase A TUI QA Findings

Directive: `/home/pfrpc/repos/orc_directives/tasknodeorc_directive_qa_tui_0325.md`
Parent mandate: `/home/pfrpc/repos/orc_directives/overnight_qa_release_mandate_20260703.md`
Evidence dir: `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/`

## Scope

- Target binary: `/home/pfrpc/repos/PfTerminal-bench/codex-rs/target/debug/pfterminal`
- Binary version: `codex-cli 0.1.1`
- Repo state: `/home/pfrpc/repos/PfTerminal-bench` on `main`, HEAD `d86721e33`, matching `origin/main`
- Test method: real TUI driven through tmux session `qa_tui`
- Provider/model: `vercel-anthropic-fast` + `zai/glm-5.2-fast`
- Permissions: YOLO mode
- Scratch dir: `/home/pfrpc/repos/pfterminal_qa_20260703/scratch`
- Session cleanup: `qa_tui` killed after test
- Key scan: `0` hits across `56` evidence files, `90,322` bytes

## Findings

### P0/P1: Spawn task routing duplicates into Main and wrong parent path

Status: FAIL

Evidence:

- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/036_burzum_task_confirm_retry.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/041_snaga_task_wait20.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/042_status_after_snaga_task.txt`

Repro:

1. Start TUI with `model_provider=vercel-anthropic-fast`, model `zai/glm-5.2-fast`.
2. `/spawn` -> create native Codex Troll `Burzum`.
3. `/spawn` -> create native Codex Orc `Snaga` supervised by Burzum.
4. `/spawn` -> status -> choose `Send task to Burzum [troll]`.
5. Submit `Reply exactly QA_TROLL_REPORT_DELIVERED. Do not run commands.`
6. `/spawn` -> status -> choose `Send task to Snaga [orc]`.
7. Submit `Reply exactly QA_ORC_REPORT_DELIVERED. Do not run commands.`

Expected:

- Targeted task starts only in the selected child pane.
- Main receives a child-report notification when the child finishes.
- Parent routing is deterministic: Orc reports to Troll, Troll reports to Main.

Actual:

- Sending task to Burzum produced both `Task sent to Burzum [troll]` and `Task sent to Main [default]`.
- Sending task to Snaga produced `Task sent to Snaga [orc]`, then the task text ran visibly as a normal Main prompt and produced `QA_ORC_REPORT_DELIVERED` on Main.
- After the Snaga task, Main emitted `Task sent to Burzum [troll]`.
- Spawn status later showed Snaga done, and Burzum's current task had become a child-report review prompt for Snaga.

Impact:

- The operator's B5/D5 dispatch concern reproduces in live TUI: target selection does not reliably isolate work to the selected pane.
- Main can execute work that was meant for a child pane.
- A child report can be converted into a new parent task rather than surfacing cleanly to Main.

### P1: Child reports are not delivered back to Main as visible reports

Status: FAIL

Evidence:

- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/037_burzum_report_after_interrupt.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/039_status_after_burzum_task.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/042_status_after_snaga_task.txt`

Repro:

1. Send exact-response task to Burzum.
2. Wait.
3. Observe Main surface.
4. Open spawn status.

Expected:

- Main transcript receives a visible child report from Burzum.

Actual:

- Main transcript did not show a child report.
- Spawn status showed Burzum `done` with `latest result: QA_TROLL_REPORT_DELIVERED`.
- For Snaga, spawn status showed `latest result: QA_ORC_REPORT_DELIVERED`, but Main did not receive a clean child-report block; instead the Snaga result was routed into Burzum's current task text.

Impact:

- Completed child work is discoverable only by manually opening spawn status.
- Supervisory loops can silently stall because Main does not get the report in the expected surface.

### P1: Main surface becomes non-responsive after dispatch tests

Status: FAIL

Evidence:

- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/047_file_edit_wait35.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/048_file_edit_wait65.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/049_file_edit_escape_interrupt.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/050_file_edit_ctrlc_interrupt.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/051_main_responsive_wait25.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/052_final_before_kill.txt`

Repro:

1. After the Burzum/Snaga dispatch tests, submit:
   `Create file qa_loop.txt containing exactly QA_FILE_EDIT_OK and nothing else. Then reply QA_FILE_EDIT_DONE.`
2. Wait about 65 seconds.
3. Check `/home/pfrpc/repos/pfterminal_qa_20260703/scratch/qa_loop.txt`.
4. Submit a simpler prompt:
   `Reply exactly QA_MAIN_RESPONSIVE.`

Expected:

- TUI runs the file-edit task or returns an error.
- Simple Main prompt returns a response.

Actual:

- No response appeared for the file-edit task.
- `qa_loop.txt` was not created.
- `Esc` did not visibly interrupt or recover the turn.
- `Ctrl-C` did recover the composer.
- A later simple `QA_MAIN_RESPONSIVE` prompt also produced no response within 25 seconds.

Impact:

- End-to-end Main workflow could not be completed after spawn dispatch testing.
- Interrupt behavior is inconsistent: `Esc` did not recover, `Ctrl-C` did.

### P2: Slash command execution and picker filtering are sticky/inconsistent under tmux-driven TUI

Status: FAIL

Evidence:

- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/003_help.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/004_slash_menu.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/006_slash_only_menu.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/011_troll_filter.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/017_spawn_enter_after_sticky.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/043_model_picker.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/044_model_picker_confirmed.txt`

Observed:

- `/help`, `/spawn`, and `/model` often remained in the composer after the first Enter and required a second Enter to execute.
- Typing `Troll` into the spawn picker search returned `no matches` even though the visible list contained `Troll`.
- Arrow navigation still worked after backing out and reopening the picker.

Impact:

- TUI automation and human operation can both misfire commands.
- Search/filter behavior is unreliable for spawn role selection.

## Passing Checks

### TUI startup and status line

Status: PASS

Evidence:

- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/001_startup.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/002_trust_continue_status.txt`

Result:

- TUI loaded after the trust prompt.
- Status line showed `PFTerminal (v0.1.1)`, `zai/glm-5.2-fast xhigh`, scratch directory, and YOLO mode.

### Spawn UI and role creation

Status: PASS with routing failures above

Evidence:

- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/008_spawn_open.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/021_troll_confirmed.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/022_troll_harness_codex.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/023_troll_spawned.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/029_orc_flow_open.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/030_orc_supervisor_selected.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/031_orc_harness_codex.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/032_orc_spawned.txt`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/033_status_after_orc_attempt.txt`

Result:

- `/spawn` opened.
- Attempting Orc before Troll produced a useful block message.
- Native Codex Troll `Burzum` spawned on `zai/glm-5.2-fast`.
- Native Codex Orc `Snaga` spawned under Burzum on `zai/glm-5.2-fast`.
- Spawn status showed hierarchy and addressable send-task rows.

### Model picker

Status: PASS with command-stickiness caveat

Evidence:

- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/044_model_picker_confirmed.txt`

Result:

- `/model` opened the model picker.
- Current model `zai/glm-5.2-fast` was selected and visible.

### Exec mode closed stdin sanity

Status: PASS for no hang, nonzero expected-error exit

Evidence:

- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/045_exec_devnull.status`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/045_exec_devnull.stdout`
- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/045_exec_devnull.stderr`

Command:

`pfterminal exec --skip-git-repo-check -C /home/pfrpc/repos/pfterminal_qa_20260703/scratch -c model_provider=vercel-anthropic-fast -m zai/glm-5.2-fast --dangerously-bypass-approvals-and-sandbox < /dev/null`

Result:

- Completed immediately.
- Return code: `1`.
- Output: `Reading prompt from stdin... No prompt provided via stdin.`
- No hang.

## Not Completed

Status: STOPPED at Phase A evidence gate due P0/P1 dispatch failures

Items not completed:

- Multiple concurrent spawned task run beyond the observed Snaga-to-Burzum report path.
- Kill spawned agent mid-run.
- Headless spawn mode, if supported.
- Successful end-to-end file edit in TUI.

Reason:

- Live dispatch routing produced incorrect Main/child/parent behavior.
- Main then failed to complete a simple file-edit turn and later a simple exact-response turn.
- Continuing would add spend and noise without improving the defect signal.

## Key Scan

Evidence:

- `/home/pfrpc/repos/pfterminal_qa_20260703/tui_evidence/key_scan.txt`

Result:

- Files scanned: `56`
- Bytes scanned: `90,322`
- Hit count: `0`

## Phase A Verdict

FAIL.

The target build loads and can spawn native Troll/Orc panes, but live dispatch is not release-quality. The critical failure class is not spawn creation; it is routing and report delivery after target selection. Tasks sent to child panes can also run on Main, child reports do not surface cleanly to Main, and post-dispatch Main turns stopped producing responses in this QA session.
