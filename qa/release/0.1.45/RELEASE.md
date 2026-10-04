# Corbanu Terminal 0.1.45 — authorized P0 cost fix

## Authority and scope

On 2026-09-23 the requesting repository operator classified the prompt-cache
defect as P0 and, after being offered the fix, patch release and verification
plan, said: “yes do that fix”. This is explicit human release authorization
under the root AGENTS.md release gate. The requesting operator is release owner.

Classification: **bounded fix**. It restores already-authorized cost behavior of
an existing provider route and adds no user goal, credential, financial, data or
persistence boundary. Product specification heading: **Shipping MVP — LIVE**,
row **Multi-provider inference**: “OpenAI, Anthropic/Claude Plan, Kimi, Z.AI,
DeepSeek, OpenRouter, … Vercel … and custom providers.”

Branch: `release/corbanu-0.1.45`, cut from published `release/corbanu-0.1.44`
(`64ffe30dd9`). It contains exactly one code change (`fix(core): keep chat
prompt-cache breakpoints on the newest turn`) plus this version bump. Main has
not absorbed 0.1.44 (PR 123 open), so this release follows the same release-branch
path.

## Defect

Chat-completions routes that require explicit `cache_control` markers
(`anthropic/*`, `minimax/*`, and the listed Qwen and DeepSeek V3.2 models on the
`openrouter` and `vercel` providers) marked the system prompt and the last two
**user** messages. Agent turns append assistant tool calls and `tool` results, not
user messages, so the markers stayed on the opening messages and each later turn
re-billed the whole transcript at the uncached input rate. The code has been
unchanged since 2026-07-07; release probes of 0.1.25, 0.1.34, 0.1.41 and 0.1.44
place markers identically. Anthropic Messages routes, including Claude Plan,
already moved markers correctly because tool results are user-role content there;
a 355-turn Claude Plan session measured 99.3% cached input.

The fix marks the last two text-bearing messages of any role and skips
tool-call-only or empty assistant messages, which Anthropic rejects as cache
breakpoints.

## Evidence

- New regression tests: `chat_cache_breakpoints_follow_the_newest_tool_turn` and
  `chat_cache_breakpoints_skip_messages_without_text`.
- `cargo test -p codex-core --lib client::` on the fix commit: **50/50 passed**.
- Versioned release tree, `cargo nextest run` (8 MiB stack, local profile) for
  `codex-core`, `codex-model-provider-info` and `codex-models-manager` libraries:
  2555 run, 2551 passed (2 flaky passed on retry), 4 failed under full parallel
  load. Rerun in isolation, three of those (`snapshot_shell_does_not_inherit_stdin`,
  `completed_pipe_commands_preserve_exit_code`,
  `shell_tool_cancellation_waits_for_runtime_cleanup`) passed.
  `spawn_agent_allows_depth_up_to_configured_max_depth` fails identically with
  this fix reverted, so it predates this release. It is not claimed as passed.
- Leak-isolated benchmark (repository harness `benchmarks/coding`, per-run
  containers, OpenRouter relay, Opus 5.5, one wave each):

| Run | Queuecraft | Cached input | Queryforge |
| --- | ---: | ---: | ---: |
| 0.1.44 | $0.534 | 38% | $1.303 (56% cached) |
| 0.1.44 + fix, same default effort | $0.302 | 79% | $1.136 (86% cached) |
| Hermes v2026.9.21 | $0.281 | 91% | $0.676 |
| Kilo 7.7.9 | $0.391 | 81% | $0.664 |

Correctness was unchanged: every Corbanu run passed Queuecraft (7/7 bug
probes) and scored 10/12 on Queryforge; the Queryforge results match the other
harnesses within one hidden case. These are single-wave diagnostics, not a
qualifying benchmark.

## Disclosed gaps

- The remaining Queryforge gap is output volume from the Opus 5.5 default
  reasoning effort (`high`); with `medium` the fixed build cost $0.446. The
  reasoning default is a separate product decision and is unchanged here.
- Candidate binaries in the table were built locally for the host; release
  binaries are built by the release workflow.
- Full workspace tests, cross-platform true-TUI qualification and the release
  suite in both default live repositories are not claimed.
- The 0.1.44 cancellation-recovery limitation is carried forward unchanged.
- The competitor/model benchmark cycle remains incomplete; see
  [benchmarks](benchmarks/README.md).
