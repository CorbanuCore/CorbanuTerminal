PF-23-S01 slice 1 is merged to main: PR #223, merge commit `f79887d182`, all checks green. I didn't start PF-23-S02, because it depends on PF-23-S01 being completed and archived, and two slices of S01 are still open. Everything is behind the default-off `source_envelopes` flag, so with the flag off, under Permissive, or before any untrusted content arrives, nothing changes.

## Allocation
- PF-23-S01 is set to `in_progress` with its worktree (`worktrees/pf23-s01-20261006`), branch, base `55339d5b24`, lane and write_scope.
- I replaced the stale PF-30-S03 slice-1 worktree entry in the plan front matter with the slice-2 coordinates. Both checkers pass.

## What shipped
- **Every tool route is classified at the point where tool calls are dispatched.** After untrusted content arrives, tools supplied by the client, unknown built-in tools and unlisted extension tools need a fresh human answer. With approvals off they are refused.
  - `request_permissions` asks the human when an automatic reviewer would otherwise answer it. `request_plugin_install` always asks.
- **MCP calls** count as protected in three cases: the tool may change or send data, its name suggests moving money, or its arguments point at a protected path or command. This check runs before remembered approvals, hooks, auto-approve and the automatic reviewer, and the question shows the arguments.
- **Typing into a running process (`write_stdin`)** is judged together with everything typed into that process since untrusted content arrived. Splitting a command across calls, line-editing keys, shell history and interrupts don't get around it, and writes to one process run one at a time.
- **Code mode:** a script that reads a tool result now marks the session as tainted straight away, so its later calls in the same script are checked. This was a real hole: a single script could read hostile output and act on it unchecked.
- **Uploads and transfers:** the shell checker now recognises local files or hidden text being sent to another machine, and token or wallet transfers.

## Gate evidence
- **Tests:**
  - The focused PF-23-S01 tests pass, and the 102 PF-30 tests still pass.
  - The full codex-core run at `eac4be8903` passed 3,841 of 3,844. The 3 failures are the known baseline ones.
  - The focused tests pass again on the merged main.
- **Lint:** `just fmt` and `just fix` are clean. Linux clippy with warnings as errors is clean on the RTX box.
- **Review (Opus 5.5 High):** six rounds. Rounds 1–4 asked for changes, round 5 approved `8037f122a9`, and round 6 approved the small follow-up at `eac4be8903`. Outputs are in `.codex-work/workers-20261002/pf23s01-review{1..6}/`.
- **GLM 5.2 videos**, recorded at `eac4be8903`:
  - [An MCP call reaching a credential asks the human](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-mcp-needs-human-eac4be890364-2026-10-06.mp4)
  - [An ordinary MCP call still runs without asking](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-mcp-ordinary-unchanged-eac4be890364-2026-10-06.mp4)
  - [Typing a vault command into bash asks the human](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-typed-vault-needs-human-eac4be890364-2026-10-06.mp4)
  - [A code-mode script can't act on output it just read](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-code-mode-cell-tainted-eac4be890364-2026-10-06.mp4)
  - [Uploading a local file asks the human](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-upload-needs-human-eac4be890364-2026-10-06.mp4)
  - [A token transfer asks the human](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s01-pf23s01-transfer-needs-human-eac4be890364-2026-10-06.mp4)
- **Model refusals:** GLM refused several earlier takes on its own (echoing an SSH key path, and uploading to `example.com`). I reworded the demo scripts, not the product. The gate record, `qa/security-levels/sprints/PF-23-S01/slice-1-gate.md`, lists them.
- **Commits after the review and videos:** two small commits came later. One removes a stale cargo-shear ignore entry; the other adds argument comments in tests. Neither changes product behaviour.

## Still open
- **PF-23-S01 slice 2:** allow memory summarisation under Moderate when the input is labelled and tied to its source session.
- **PF-23-S01 slice 3:** block reads of home and credential files at the OS sandbox level. This replaces the remaining command-text gaps listed in the sprint file, plus the activation readiness matrix.
- **Accepted extra prompts after untrusted content:** these ask the human but never run unasked. Examples are `$` variables or template literals typed into a Perl, Ruby or Node prompt, and a wrapped command with more than 63 arguments. The full list is in the gate record.
- **Product finding (not fixed here):** the Chat Completions and Anthropic connections drop MCP tools entirely (`core/src/client.rs`, `tool_spec_to_chat_tool`). So GLM 5.2 and the Claude-plan models never see MCP tools directly; the MCP videos reach them through code mode. This needs an owner.
- The sprint index row still reads "in progress: slice 1"; I didn't open a separate docs PR to mark it merged.

My report is saved as `.codex-work/workers-20261002/sec-taint5.result.md`.