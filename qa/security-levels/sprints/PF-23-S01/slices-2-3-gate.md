# PF-23-S01 slices 2-3: per-sprint gate (2026-10-06)

- **Branch:** `feat/pf-23-s01-slice2-3`, off main at `8cf46179f5` (slice 1 merged as #223).
- **Scope:** behind `source_envelopes` with Moderate/Aggressive. Flag off, Permissive and untainted sessions are
  unchanged (every new check returns first). Readiness: [activation-readiness.md](activation-readiness.md).

## What ships

| Piece | Behaviour |
| --- | --- |
| Stage-one memory (slice 2) | Above Permissive Core builds the whole stage-one message (`StageOneMemoryClient::label_rollout`): it reads the claimed rollout, whose first record must be that session's; redacts each item; restores only origin records this home signed; labels everything else; drops whole middle items to fit the budget. The request must equal that message or is refused before dispatch, at every HTTP/WebSocket attempt and on completion. Moderate needs the flag; Aggressive denies; consolidation is skipped above Permissive. |
| Read denials (slice 3) | After taint, `ReadDenials` adds Deny entries for the Corbanu home (default-deny minus `tmp`, `shell_snapshots`, skills, plugins, packages, worktrees, `AGENTS.md`; fixed stores and `*.sqlite*` even if missing), other Corbanu homes and fixed `$HOME` credentials, to orchestrated commands, in-process file tools and per-turn extension contexts; Codex Apps uploads check the same policy. Keep roots come from the turn, never the command. Full access becomes a write-everywhere sandbox with the denials. Moderate: a fresh human approval of the exact protected shell/exec command lifts them for that run. |

## Gate results

- **Lint:** `just fmt`, `just fix -p codex-core -p codex-memories-write` clean. Linux clippy
  (`cargo clippy --locked -p codex-core -p codex-memories-write --all-targets -- -D warnings`) on the RTX box: clean
  at the final commit (see PR). An earlier run caught a macOS-only test helper and an `is_none_or(<= 0)` comparison.
- **Focused:** `just test -p codex-core pf_23_s01 memory_stage_one pf_30_s0 mcp_openai_file` (145 pass) and
  `just test -p codex-memories-write` (45 pass). New: read-denial unit tests (home default-deny, full access, keep
  roots, Claude dir, file tools after taint, upload reads through symlinks), stage-one lineage/foreign key/long
  rollout/redaction/exact-message tests, a Moderate worker test that sends only labelled text and refuses another
  session's file, and macOS suite tests where the sandbox (not the text check) denies a run-time-built home path,
  a model-chosen working folder inside a denied path keeps the denial, and approval lifts it under Moderate only.
- **Full crate:** `just test -p codex-core`: 3,855 of 3,858 at `18365ca3c5`; the 3 failures are the known baselines
  (`skills_append_to_developer_message`, `skills_use_aliases_in_developer_message_under_budget_pressure`,
  `remote_compact_trim_estimate_uses_session_base_instructions`).
- **Review:** Opus 5.5 High through `corbanu exec`, `.codex-work/workers-20261002/pf23s01-review{7,8,9}/`. Round 1
  changes (model-chosen working folder lifted a denial; file tools unprotected; long rollouts always refused; host
  could shape the input); round 2 changes (Codex Apps uploads; quadratic fit; grants as keep roots; redaction could
  cut a label); round 3 APPROVE at `a58a5e18e6`, non-blocking items below; its fit-loop Low is fixed after.
- **GLM 5.2 videos** at `a58a5e18e6`, in [qa/demos/index/PF-23-S01.md](../../../demos/index/PF-23-S01.md): a run-time
  home path read after untrusted content gets "Operation not permitted" with no prompt; under Moderate the approved
  literal read goes through; the memory worker sends an aged session's labelled rollout under Moderate.

## Known limits (moved or open)

- Codex Apps upload check-then-read can race a link swap by a background command: read through the protected
  sandbox before the flag is turned on (PF-23-S02, disclosure operations).
- Extension tools keep permissions a human granted earlier in the session; external sandboxes and remote exec
  servers take no extra rules (uploads refuse them); where no sandbox starts, tainted full-access commands and file
  tools fail closed; processes started before taint keep their sandbox; pre-existing hard links; the shell snapshot
  stays readable.
- Product findings: GLM 5.2 wraps stage-one JSON in a code fence, so extraction fails at every level (the video
  shows the parse error after the request); Chat Completions and Anthropic wires drop MCP tools.
