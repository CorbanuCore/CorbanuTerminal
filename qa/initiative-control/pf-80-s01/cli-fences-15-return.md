# tn-cli-fences-15 implementation return

Allocation: `4056da0b9d044cd2efbdda1cb1718d1fb99d92586dc433a5d55c8996cb2a2146`; claim `bdb100d4-e66a-4edc-a1e6-b35a7760e466`.
Runtime: gpt-6-astra, high. Base verified: `1b1ea6912ef2a8ff2d005c202fba33c9256f04a6`.
Brief SHA-256 verified with `shasum -a 256`: `2657d1f89f95656b8fe45784d197239ab6c4b7ba6a3bff83e556bfc0c53d8498`.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-cli-fences-20260916`; branch: `bootstrap/tasknode-cli-fences-20260916`.
Class: product-initiative implementation slice, PF-80-S01 (in_progress), active `initiative-delivery-control.md`.
Product heading: **Internal delivery control — TO BUILD**; excerpt: “Use sequential sprints per initiative, incremental merges behind verified default-OFF feature boundaries”.
The plan/sprint coordinates predate this explicit dispatch and do not name this worktree; manager must reconcile them. No worker edits to manager-owned records.

## Enforcement and CLI evidence

All implementation paths below are in `scripts/initiative_control/tasknode.py`; all listed tests are `CLIFenceTests` in `scripts/initiative_control/test_tasknode.py`.

| Fence | Enforcement | CLI tests |
| --- | --- | --- |
| Cross-process | `main → CLI_CONTEXT → locked`: nonblocking exclusive flock, two-second monotonic bound, exit 2 naming another Task Node operation and the lock path. `control.locked` unchanged; ordinary library calls delegate to it. | `test_cli_lock_expiry_names_real_holder_and_path_and_recovers`: real second holder; retry/enqueue/flush/live-send expiry, lock-free status/preview, then successful acquisition after release. |
| Restart | `flush` persists status uncertain and increments attempts before transport; only pending records can dispatch. `delivery_status` overlays retained single-send intent/result evidence. Existing `send` never re-POSTs an intent. | `test_cli_batch_crash_stays_uncertain_until_explicit_named_retry`: child writes synthetic acceptance marker then exits 73 inside transport; fresh CLI flushes do not resend; status reports uncertainty; explicit retry restores same payload. `test_cli_single_send_crash_retains_intent_and_denies_retry`: real child exit leaves intent without result, fresh CLI replay does not send, named retry refuses with reconciliation reason. |
| Recovery | `cli` exposes no destructive operation; `retry` selects one existing hash-bound record and permits only blocked/uncertain batch records without single-send receipts. | `test_cli_recovery_operations_are_absent_and_denied`; `test_cli_selectors_cannot_create_rewrite_or_bulk_retry_records`; `test_cli_inspection_is_read_only_and_status_is_json_lines`. Covers absent destructive commands/flags, bulk/repeated/unknown/path-traversal selectors, pending/delivered retry denial, valid blocked retry, and inspection refusing live/auth inputs. |
| Input | `cli` accepts file credentials only, disables abbreviated options, rejects incompatible report/auth/live flags. `CLIParser.error` never echoes unknown arguments. `credentials` redacts parse errors. `enqueue → checked_run → event_for` validates even deduplicated reports. | `test_cli_credentials_are_file_only_and_never_echoed`: raw flags/abbreviations, environment-only auth, malformed/non-object files and permissive modes refused; file-only success uses synthetic values without output disclosure. `test_cli_enqueue_requires_validated_report_and_field_byte_checks`: arbitrary events, extra fields, byte limits, secret-like content and invalid timestamps refused; valid report accepted. |
| Retention | No age eviction path; rationale beside `delivery_status` distinguishes accounting telemetry from irreplaceable external-post evidence. `retry` cannot erase receipts. | `test_cli_old_records_and_receipts_are_never_aged_out`: old file timestamps and clock ten years ahead, inspection/retry denial/flush preserve all retained bytes. |
| Accessibility/OFF | `CLIParser.error` keeps refusal plus reason on one line. `print_status` emits JSON lines with worded blockers; status includes uncertainty/recovery reason. Configuration is never enabled by these changes. | All `denied` assertions require a single reason-bearing line and unchanged fixture bytes. `test_cli_posting_stays_off_and_refusal_names_local_blockers` covers OFF, missing activation, rejected activation, and missing enrollment. |

## Retry decision and scope interpretation

Explicit named retry may reset an **uncertain batch** record to pending/zero attempts, preserving its immutable event: the operator deliberately accepts possible duplication. Automatic restart never makes that decision.
Single-event intent/result receipts remain immutable and refuse retry, requiring external reconciliation. Deleting receipts or inventing a resend mechanism would break the existing create-once/no-re-POST contract and exceed this allocation. The generic frozen wording “requires an explicit retry” is therefore necessary but not sufficient for receipt-bearing single sends; manager review must retain this stated limitation.
There is no direct conflicting requirement between the brief and frozen scope. Both descriptions lag actual code: retry already required blocked status, and exhaustion records blocked rather than failed. The frozen lock-free preview remains lock-free; only actual CLI lock acquisitions are bounded.
This cannot reconstruct whether an unmarked pending record crashed before this implementation existed. Existing explicit transient-result backoff behavior is retained; this change fences missing completion after process death.

## Validation

Read `docs/development/test-isolation.md` before tests. Created a NEW disposable venv using `venv.EnvBuilder(with_pip=True)`, with no system site packages.
Copied the three exact wheel archives from local pip HTTP cache into its temporary wheel directory, then ran `venv/bin/python -m pip install --no-index --no-deps --find-links <temp>/wheels -r scripts/initiative_control/requirements.txt`.
Only application dependencies installed: markdown-it-py 3.0.0, mdurl 0.1.2, slack-sdk 3.44.1; no downloads or pre-existing venv reuse.
Temporary evidence root: `/private/tmp/ctncli15.YZc0c6/tmp/tn-cli-fences-15-4xz0q9dm`.
Runner uses `env -i`, a tool-only PATH, temporary HOME and all three profile aliases, native-keyring denial, `PYTHONDONTWRITEBYTECODE=1`, and explicit `PYTHONPATH=<worktree>/scripts/initiative_control`.
Final discovery command: `venv/bin/python -m unittest discover -s scripts/initiative_control -p 'test_*.py' -v`; **699 tests run, 3 failures and 1 error**, 462.246s, exit 1; raw `suite-final-replay.log`. Full-suite gate is NOT satisfied; no skips reported.
Final affected run: **51 tests passed**, all Task Node and preparation tests; `affected-final.log`. Earlier corrected-environment run: **31 passed**, including 30 Task Node tests and coordinator crash regression; `targeted-final.log`.
Preserved attempts: first Task Node run **30 tests, 1 failure**, exposing deduplication bypass of invalid timestamp validation; corrected in enqueue. First broad run omitted child PYTHONPATH, exposed import failures and was deliberately interrupted (exit 130); `suite.log`. It is not a pass.
First completed discovery: **699 tests, 22 failures and 1 error**, 470.594s, `suite-final.log`. Seven failure instances were an over-redacted missing-argument reason; corrected while preserving secret redaction. The remaining failures/error involved TMUX fixtures. Standalone launcher replay failed once, diagnostic replay and exact-environment replay then passed; preserved in `launcher-diagnostic.log`, `launcher-probe.log`, and `launcher-exact-env-replay.log`. No unrelated implementation was changed.
Remaining full-suite failures: `test_decision_manager.ManagerTests.test_tmux_handoff_end_to_end_mixed_transport_refusals_and_one_unlock`; `test_owner_tmux.TmuxTests.test_bridge_incomplete_and_malformed_rollouts_never_issue_evidence` (partial failure, malformed error); `test_owner_tmux.TmuxTests.test_bridge_uncertain_enter_never_resends_after_receiver_reload`. These paths are outside the writable scope; attribution remains unresolved, not declared pre-existing or waived.
Governance: plan checker 3/3 active; sprint checker 115 current/127 archived; `git diff --check` clean. Changed lines: tests **254 additions / 0 deletions**; implementation **113 additions / 15 deletions**; this evidence **50 additions**, total outside tests **178** (target 220, hard limit 320).
Final source SHA-256: tasknode.py `9d209ae3801c0b48481afd6ffbcad6bcd719ded96c7a5e662b59eeedf2d79037`; test_tasknode.py `f9e1e42da42f34919178ce025c432c0864e38e9b9f686364f1db257834856f8c`.

## Remaining / deliberately excluded

No live state, credential-store access, Task Node network calls, enrollment, activation, payload selection, posting enablement, push, Rust changes, workspace-wide formatter, dependency changes or unrelated files.
Tests use synthetic transports; existing suite loopback fixtures are not live service access. No release, human acceptance, independent review, isolated functional acceptance, or TUI qualification is claimed.
Manager owns independent review, coordinate/ledger reconciliation and applicable functional gates before human handoff; native TUI/runtime code was not changed. No sprint completion or release readiness claim.
