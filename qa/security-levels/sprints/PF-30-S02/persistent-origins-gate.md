# PF-30-S02 persistent origins — per-sprint gate (2026-10-06)

Branch `feat/pf-30-s02-persistent-taint`, base `b96b23344b` (main after PR #178), candidate `01a3a88330`.
Behind the default-off `source_envelopes` feature. Flag off writes no new rollout lines and sends the same
requests as before.

## What ships

| Path | Behaviour with the flag on |
| --- | --- |
| Rollout | Core record seams append a `source_origin` line (`version: 1`, SHA-256 keys only, no content) after the items whose origin they record: message (human/host/model/external kind), model item, tool-call route |
| Resume, fork | Records restore standing. Missing, unknown-version, malformed or impossible scope/origin records leave content unattributed (labelled `source=unknown`). A record never upgrades an existing registration. Host context is reinjected only when restored messages lack records |
| Compaction checkpoint | The current origin of every replacement item is restated after the `compacted` line, so resumes that start at the checkpoint keep it |
| Local compaction | Retained user messages keep human standing only if a human-recorded message had exactly their text. The summary gets host standing only if every compacted input had standing; any tool/MCP/agent output, memory, unattributed or opaque input leaves it labelled |
| Memory | The memory summary is its own developer message, registered `External(Memory)` first, so it reaches Moderate/Aggressive as `source=memory authority=none` |
| Token-budget window reset | Rebuilt host context is registered as host (was labelled before) |
| Flag off, Moderate | Fails closed with: “Moderate and Aggressive need the `source_envelopes` feature, which is off, so the request was stopped before anything was sent” |

## Gate results

- `just fmt`; `just fix -p` for codex-core, codex-protocol, codex-rollout, codex-state, codex-thread-store,
  codex-app-server-protocol, codex-memories-write, codex-memories-extension, codex-extension-api: clean.
- Focused: `just test -p codex-core pf_30_s0` — 62 pass, also after merging main (12 `pf_30_s02`: 8 unit, 1 client, 1 session, 2 suite).
- Final tree, all nine crates: 5,056 run, 5,052 pass. Failures: `skills_append_to_developer_message`,
  `skills_use_aliases_in_developer_message_under_budget_pressure` (machine-installed skills, also on main per
  PF-30-S01), `remote_compact_trim_estimate_uses_session_base_instructions` (reproduced on a clean `b96b23344b`
  worktree), `shell_command_snapshot_still_intercepts_apply_patch` (load flake; passes alone).
- App-server schema fixture regenerated (`RolloutItem` gained `SourceOrigin`).
- GLM 5.2 tmux run (real keys, Moderate + flag): resume after `/new` + `/resume`, hostile memory summary, `/compact`
  after tool output. No credential found in the run directory. One compaction recording got a wrong model answer
  (“none”); the rollout showed the summary had no record (so it was labelled); the reworded prompt passed.
- Independent review (Opus 5.5 High, `corbanu exec`, read-only): round 1 CHANGES REQUESTED (one medium:
  checkpoint resumes lost records; five low). All fixed in `f3a5768b02`; round 2 APPROVE. Outputs in
  `.codex-work/workers-20261002/pf30s02-review{1,2}/`. The later `01a3a88330` only makes the overflow warning fire once.
- Videos at `01a3a88330`: [qa/demos/index/PF-30-S02.md](../../../demos/index/PF-30-S02.md).

## Known limitations

- Flag on, Permissive: the memory summary is a separate developer message (same text, different layout).
- Records are only as trustworthy as the session store; a process that can write `CODEX_HOME` can forge them.
- Opaque remote-compaction items and encrypted agent content keep their S01 handling.
- An image-only human message keys on its role alone (S01 behaviour), now also persisted.
- After `/compact`, a labelled summary can make a model repeat earlier answers (seen once with GLM 5.2).
- Not yet covered: a real paginated/referenced-fork resume test, fork restore tests, agent spawn and mailbox
  lineage, export/import, memory stage-one policy binding, the token-budget MCP thread hint (still host).
