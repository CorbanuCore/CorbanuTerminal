# PF-30-S02 slice 2 — per-sprint gate (2026-10-06)

Branch `feat/pf-30-s02-slice2`, base `476f42dbe3` (main after PR #194). Behind the default-off
`source_envelopes` feature; flag off writes no records, creates no key file and sends the same requests.

## What ships

| Path | Behaviour with the flag on |
| --- | --- |
| Signed records | `source_origin` records are version 2 with `mac`, an HMAC-SHA256 tag of the entries under a per-home key (`$CODEX_HOME/source-origin.key`, 0600, written whole then linked into place, or created in place where hard links are unsupported and removed if that write fails; read without following symlinks, refused if others can read it). Records that do not verify (another home, edited, version 1, no key) restore nothing, so their content is labelled. Without a key nothing is written |
| Imports | External-agent session imports and thread-history projections never carry records |
| Agent hand-offs | Input one agent submits to another (spawn task, `send_input`, the automatic reviewer's transcript prompt) is marked on the receiver by its exact `Vec<UserInput>` digest before submission. The receiver's prompt seam records it with the sender's standing: host while every item in the sender's history had standing, otherwise `source=child_agent` data, never human. A full mark ledger fails closed (later prompts count as agent data); identical inputs take the most restrictive mark first; a hook-blocked prompt drops the least restrictive mark. `/review` delegates keep their human prompt |
| Child results | A V1 child's completion notice reaches the parent as agent data; V2 mailbox messages already were |
| Token-budget hint | The MCP `notes.thread_hint` text is its own `source=mcp` data message, not part of the host developer message |
| Memory | Unchanged: stage one still denies under Moderate/Aggressive (PF-30-S04); the flag does not reopen it |

## Gate results

- `just fmt`; `just fix -p` for codex-core, codex-protocol, codex-external-agent-migration, codex-app-server: clean.
- Focused: `just test -p codex-core pf_30_s0` — 75 pass (25 `pf_30_s02`); `just test -p codex-app-server
  thread_fork` — 25 pass, including the real paginated, reference-backed fork plus cold resume;
  `just test -p codex-external-agent-migration pf_30_s02` passes.
- Full `just test -p codex-core -p codex-protocol -p codex-external-agent-migration`: 4,210 run, 4,207 pass.
  The three failures are the known baselines from slice 1: `skills_append_to_developer_message`,
  `skills_use_aliases_in_developer_message_under_budget_pressure` (machine-installed skills) and
  `remote_compact_trim_estimate_uses_session_base_instructions` (fails on clean main).
- GLM 5.2 tmux runs (real keys, Moderate + flag, recorded at `833be22574`): a session whose home key is replaced
  resumes with the earlier human message labelled `source=unknown`; a once-approved command's output stays
  `source=tool authority=none`; a sub-agent spawned after `cat notes.txt` quotes its task header as
  `source=child_agent authority=none`. No credential found in any run directory.
- Independent review (Opus 5.5 High, `corbanu exec`, read-only), three rounds in
  `.codex-work/workers-20261002/pf30s02b-review{1,2,3}/`:
  round 1 CHANGES REQUESTED (High: agent text could be recorded as human when media preparation changed the
  item key; fixed by marking the exact input). Round 2 CHANGES REQUESTED (Medium: `/review` prompts were
  needlessly labelled; fixed). Round 3 APPROVE; its one new Low (a blocked copy could consume the strictest
  mark) and the short-key cleanup are fixed in `675d3c1fee`, after the videos.
- Videos: [qa/demos/index/PF-30-S02.md](../../../demos/index/PF-30-S02.md).

## Known limitations

- A process that can read the key or write `CODEX_HOME` can still forge records.
- The tag covers entries, not the session: a record from this home spliced into another rollout of this
  home grants standing only to the identical text it already had.
- A sender's standing is judged from its stored history; prompt-only context (MCP tool descriptions) is
  not counted, as for compaction summaries.
- Encrypted agent content and opaque remote-compaction items keep their earlier handling.
- Marks for agent input that is never recorded (steer rejected after an authorization change, an aborted
  turn) stay pending; 1,024 of them make that session treat every later prompt as agent data.
- No test drives the automatic reviewer end to end with the flag on and a tainted parent.
- On FAT/exFAT homes mode bits read as 0777, so the key is refused and that home runs without records (labelled).
- Moved: positive protected memory extraction to PF-23-S01; tainted follow-on authority to PF-30-S03.
