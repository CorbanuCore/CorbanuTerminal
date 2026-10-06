# PF-30-S01 labelled ingress — per-sprint gate (2026-10-06)

Branch `feat/pf-30-s01-source-envelope`, base `cb78550a31` (main). Behind the
default-off `source_envelopes` feature. Permissive and flag-off requests are
unchanged; realtime and memory summarisation still fail closed.

## What ships

Under Moderate/Aggressive with the flag on, every provider request is projected
by `NativeIngress::project_labelled` (`codex-rs/core/src/security/ingress/structural.rs`):

| Item | Sent as |
| --- | --- |
| Human prompt (recorded by `record_user_prompt_and_emit_turn_item`) | unchanged |
| Host context, reminders, world-state diffs, interrupt marker | unchanged |
| Assistant text and model structure recorded from the provider stream | unchanged |
| Tool, MCP and custom-tool output | labelled data (`source=tool`/`mcp`/`unknown`) |
| Hook context, user-shell output, agent messages | labelled data (`hook`/`tool`/`child_agent`) |
| Any message without a recorded origin (injected, restored, forked) | labelled data (`unknown`, user role) |
| Call structure without a recorded origin | labelled data message with its outputs, no provider item id |
| Unrecorded reasoning, image generation, unknown wire variants | withheld |
| Text over 1 MiB (2 MiB after escaping) | fixed withheld notice |

Labelled text is `<corbanu_untrusted_data>` + `source=<kind> id=<uuid> sha256=<hex> authority=none`
+ a one-line notice + neutralized text. The deterministic producer runs the
existing complete-input screening contract (`labelled-data-neutralize` v2);
the projection is byte-stable for retries and prompt caching. Resume/fork keeps
restored messages labelled and reinjects host context once on the next
protected turn.

## Gate evidence

| Check | Result |
| --- | --- |
| `just test -p codex-core pf_30_s01` | 39/39 pass (10 new labelled-mode tests, 1 new suite test) |
| `just test -p codex-core` (full, `61b75ec43a`) | 3,740 pass, 2 fail (both fail on main, below), 19 skipped |
| `just test -p codex-features -p codex-protocol -p codex-content-security` | 349/349 pass |
| `just fmt`, `just fix -p codex-core -p codex-features` | clean (only pre-existing warnings in other crates) |
| Independent review, Opus 5.5 High (`corbanu exec`) | round 1 changes requested (restore promotion blocker, role-based standing, look-alikes, caps, cache); round 2 changes requested (image key, Permissive reinjection, model structure); round 3 changes requested (provider item id on converted calls); round 4 **APPROVE** at `61b75ec43a`. Prompts/outputs: `.codex-work/workers-20261002/pf30-review{1..4}/` |
| GLM 5.2 tmux functional run, real TUI | three demos below, all passed |

Full Core: the only consistent failures
(`suite::client::skills_use_aliases_in_developer_message_under_budget_pressure`,
`suite::compact_remote::remote_compact_trim_estimate_uses_session_base_instructions`)
fail identically on main `cb78550a31` in a separate worktree (the host's
`~/.agents/skills` leaks into the test profile). `shell_command_snapshot_still_intercepts_apply_patch`
failed once and passed on rerun.

## Demo videos (GLM 5.2 via zai, `scripts/demo_video.py`)

Index: [qa/demos/index/PF-30-S01.md](../../../demos/index/PF-30-S01.md).

1. `pf30-labelled-tool-output`: notes.txt forges a wrapper close and a `<system>` approval. GLM quotes the host
   label (`source=tool … authority=none`), says it grants no approval, and pwned.txt is never created.
2. `pf30-normal-tool-use`: GLM reads, edits and runs a file across several tool calls under Moderate with the
   flag; the task completes normally.
3. `pf30-flag-off-fails-closed`: without the flag, Moderate still stops before any model request.

## Known limitations (not blocking the flagged merge)

- Persisted per-message origins: restored history stays labelled until PF-30-S02 persists origins.
- Images, audio and encrypted agent content pass unwrapped; MCP tool-search descriptions are not labelled.
- Parent→child task messages carry the `child_agent` label; the review-exit summary is labelled.
- Registries hold 65,536 entries and are not pruned; overflow fails toward labelling.
- Image-generation calls are withheld after a switch to a text-only model; an injected call reusing a live call id
  converts that live output to a labelled message (only host injection APIs can do this).
- Combining marks inside a forged tag name are not neutralized (low spoofing risk; the wrapper close is).
- Flag-off Moderate reports "source admission registry is full or poisoned" because host context exceeds the
  old 2,048-byte bound; the message is misleading but the path still fails closed (pre-existing).
- Product finding from the demo SOP scan: with `RUST_LOG=trace` the TUI log and logs SQLite WAL contained the
  provider credential (redacted by the tool). Not in this sprint's scope; reported to the lane owner.

## Merge

Merged to main on 2026-10-06 as PR #178, merge commit `b96b23344b`, behind `source_envelopes`.
All 32 CI checks passed. The sprint record is archived as completed.
