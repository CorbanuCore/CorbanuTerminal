# PF-30-S01 checkpoint — not sprint completion

Product initiative, **Non-negotiable controls**: “Classify instruction intent
and provenance before external content can influence tools or financial actions.”
Plan: `docs/plans/active/p0-security-levels.md`, feature PF-30, sprint PF-30-S01.

## Current implementation boundary

- Validated descriptive protocol envelopes cannot represent human/system authority.
- Core-only admission binds an existing screening capability to the exact source
  envelope and normalized content digest; an Allow verdict releases untrusted data.
- Bounded context fragments use the user/data role, escaped delimiters and explicit
  Unicode controls. No generated fragment exceeds 8,192 bytes. This can exceed
  1,000 tokens and requires explicit reviewer attention under Core context policy;
  it remains below the hard 10,000-token per-item bound.
- Native Responses, Chat Completions and Anthropic request constructors reject
  configured protected intent without an authenticated admitted-context carrier.
  Initial session creation and model-provider replacement preserve this restriction.
  Memory and realtime alternate input routes also fail closed.
- Permissive request behavior is unchanged; no protected mode is activated.

## Explicit remaining work

The native gate is refusal, not successful producer integration. Per-source native
capture/admission for web, files, MCP, hooks, plugins, child messages and other
external sources still needs its trusted carrier and producer registrations.
Typed host authorization notices remain separate required work. Provider-safe
projection fixtures alone do not prove native successful ingestion. Persistent
resume/memory lineage remains PF-30-S02, post-taint enforcement PF-30-S03, and
qualified detector delivery PF-35. Unknown or unavailable routes remain closed.

No tests, independent reviews, TMUX, human acceptance, release or benchmark pass
is claimed by this checkpoint. Builds/tests run only on the allocated RTX host.
The pinned OpenClaw source checkout was lost; OC-4/OC-10 repository review records
were consulted, not fresh source execution. Adversarial fixtures here are original
Corbanu tests based on the recorded requirements, not copied implementation code.
