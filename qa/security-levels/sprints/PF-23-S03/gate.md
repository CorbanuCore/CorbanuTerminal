# PF-23-S03: per-sprint gate (2026-10-07)

- **Branches:** four stacked PRs off main at `6b1c8b8873`: `feat/pf-23-s03-recovery` → `-transition` → `-fanout` →
  `-memory`. With no `security_state.json` and no `[security]` anywhere, behaviour is today's. `commit_transition`
  has no production caller yet: PF-24-S02 (TUI confirmation) is the first.

## Transition states

```text
           prepare(request, probes)            commit(store)
  current ------------------------> Prepared -------------> committed (epoch+1, generation+1)
     ^                                 |  stale epoch, refused merge, failed save of a
     +------------- cancel ------------+  downgrade or release: nothing changes
```

| Request | Applies | Saved as | Save fails |
| --- | --- | --- | --- |
| Stricter level (probes must pass) | now | max(stored, level) | applies anyway, reported `not_saved` |
| Revocation, kill switch on | now | stored level kept | applies anyway, reported |
| Downgrade | next start (revocation now) | the lower level, and `config.toml` | nothing changes; stored state restored |
| Kill switch off (only the one shown) | now | stored level kept | nothing changes |

Start: level = max(every config layer, stored floor); stored revocations restored; unreadable or invalid file →
Aggressive + kill switch + warning. Commits merge under `security_state.lock` (2 s wait): union of revocations, a
stored stricter level is never lowered unseen (`StoredLevelChanged`), a newer stored kill switch is never released
unseen (`StoredStateChanged`). A commit drops the tree's grants, fences pending post-taint approvals, "for
session" approval caches and child snapshots, closes broker channels (all but single grant/mandate revocations and
releases), and reaches the other trees of this process on the same home.

## Results

- **Focused (per PR, final tree):** `just test -p codex-core security_transition security_recovery` 26/26; with
  `pf_23 pf_30 memory_stage_one network_approval sandboxing mcp_tool_call control_tests config:: session::`
  1,270/1,271 (the one failure, `config_schema_matches_fixture`, fails on main).
  `just test -p codex-security-policy revocation` 11/11. Suite: `pf_23_s03_stored_level_survives_restart_over_a_lower_config`
  (real macOS sandbox).
- **Full:** `just test -p codex-core -p codex-memories-write -p codex-security-policy -p codex-network-proxy -p codex-config
  -p codex-protocol -p codex-state -p codex-rollout -p codex-thread-store`: 5,524 of 5,528. Failures are the known
  baselines (`config_schema_matches_fixture`, `skills_use_aliases_in_developer_message_under_budget_pressure`,
  `remote_compact_trim_estimate_uses_session_base_instructions`) and the load flake
  `shell_command_snapshot_still_intercepts_apply_patch`.
- **Linux clippy** (`-D warnings`, core, network-proxy, config, protocol, memories-write, state, thread-store,
  rollout, app-server, security-policy) on the RTX box: clean at the final tree (round 1 caught a type-complexity error).
- **Review (Opus 5.5 High):** `.codex-work/workers-20261002/pf23s03-review{1,2,3}/`. Round 1 REQUEST CHANGES
  (last-writer-wins file, kill switch refused on a failed save, approval races, downgrade left broker channels
  open, dropped project `[security]`, unrepairable state); round 2 REQUEST CHANGES (revocations replaced instead
  of merged, unbounded lock wait, a repository's raise persisted for the user); round 3 APPROVE. Its follow-ups M1
  (release of an unseen kill switch), L1 (failed downgrade mirror) and L3 (repair only by a level) are fixed with
  tests; the rest are handed to PF-24-S02 (sprint record).
- **Videos:** GLM 5.2 runs are blocked: Z.AI answers "Insufficient balance or no resource package" (code 1113).
  One model-free video on the final candidate: [unreadable state warns](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-23-s03-pf23s03-unreadable-state-warns-c60857fc5d47-2026-10-07.mp4).
  Specs for the model runs are ready: `pf23s03-stored-level-next-session`, `pf23s03-unreadable-state`,
  `pf23s03-project-cannot-lower`.

## Merging step and summaries under Aggressive

Kept off, the strictest option: under Aggressive stage one is refused, and consolidation is skipped above
Permissive. Now also: a session that ran under Aggressive is never summarized, and one that ran under Moderate is
labelled even if the level is Permissive now. Turning either on under Aggressive is a product decision.

## Known limits

- Same-user deletion or rollback of `security_state.json` needs the PF-20 anchor; agent commands cannot reach it
  once the protected-path rules apply. Another process sees a commit at its next session start.
- Network hosts approved "for session" in the moment after a commit can still be stored (the commit clears them;
  a marker fence would break guardian review sessions that copy hosts); the MCP "remember" path uses the
  last-checked cache marker.
- A level raised during a session's last turn is recorded from its next turn; the memory worker reads a rollout
  twice. A single grant revocation drops every grant of the tree (stricter).
- Three level stores are not reconciled yet: the picker's `security_level.toml`, `config.toml` `[security]` and
  `security_state.json` (PF-24-S02, PF-41-S01).
