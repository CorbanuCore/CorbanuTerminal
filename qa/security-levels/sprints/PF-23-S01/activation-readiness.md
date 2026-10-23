# Moderate activation readiness (PF-23-S01, 2026-10-06)

Moderate is **not offered** in `/security` and must not be until every row marked *required* is ready. A config
`security_level = "moderate"` already runs the core checks below; that is a test setting, not a product level.

## What each action gets (with `source_envelopes`)

"Tainted" = the session holds content without human, host or model standing (tool, MCP, web, agent, memory or
unattributed text); it never resets within a session or across resume, fork, compaction or summaries.

| Action | Permissive | Moderate, untainted | Moderate, tainted | Aggressive, tainted |
| --- | --- | --- | --- | --- |
| Ordinary shell / exec / patch, in-process file tools | unchanged | unchanged | runs; sandbox and file tools deny credential and Corbanu-home reads | same as Moderate; workspace sandbox, approvals on |
| Command the text net marks protected (vault, credentials, policy, disclosure, value transfer, unseen code) | unchanged | unchanged | fresh human approval; refused with approvals off; approval lifts the read denials for that run | fresh human approval; read denials stay (grants: PF-23-S02) |
| MCP call that may change/send data, moves value, or names a protected path/command | unchanged | unchanged | fresh human approval | fresh human approval |
| Typing into a running process | unchanged | unchanged | judged as one command since taint | same |
| Client, unknown built-in or unlisted extension tool | unchanged | unchanged | fresh human answer | fresh human answer |
| Stage-one memory summary | unchanged | Core-labelled rollout of the claimed session only | same | denied |
| Memory consolidation (phase 2) | unchanged | skipped | skipped | skipped |

Ancestry is conservative: data ancestry is the session taint generation (any untrusted batch raises it, nothing
lowers it); control-flow ancestry is the approval binding (an approval holds only for the taint generation and
policy epoch it was given under, and is refused stale otherwise).

## Subsystem readiness

| Subsystem | Sprint | State | Required for Moderate |
| --- | --- | --- | --- |
| Labelled untrusted content, persistent taint | PF-30-S01/S02 | merged, `source_envelopes` | yes |
| Post-taint checks: shell, exec, patch | PF-30-S03 | merged | yes |
| Post-taint checks: MCP, typing, code mode, unclassified tools | PF-23-S01 slice 1 | merged (#223) | yes |
| Labelled, session-bound stage-one memory | PF-23-S01 slice 2 | this PR | yes |
| OS read denials after taint | PF-23-S01 slice 3 | this PR (macOS, Linux bwrap, Windows elevated; not external sandboxes or remote exec servers; fails closed where no sandbox starts) | yes |
| Secretless agent launch | PF-27-S02 | merged, `secretless_agent_launch` | yes |
| Isolated credential broker; model-client auth | PF-27-S04 / PF-27-S05 | merged / open (#229) | yes |
| Secret output gate, reflected scrubbing | PF-28-S01/S02 | merged, `secret_output_gate` | yes |
| URL, DNS, pinning, alternate egress | PF-33-S01/S02 | merged, `url_destination_policy` | yes |
| Preflight inventory; credential migration | PF-29-S01 / S02 | open (#228) / in progress | yes |
| Downgrade, restart, inheritance | PF-23-S03 | not started | yes |
| Confirm, cancel, downgrade TUI | PF-24-S02 | not started | yes |
| Grants and kill-switch TUI | PF-25-S01/S02 | not started | Aggressive only |
| MCP servers, hooks, notify (run outside the sandbox) | PF-27-S02 decision | not contained | shown as "not contained" |
| MCP tools on Chat Completions / Anthropic wires | product finding | dropped by the client | no (fails safe) |

**Verdict:** not ready. Missing required rows: PF-27-S05, PF-29-S01/S02, PF-23-S03, PF-24-S02.
