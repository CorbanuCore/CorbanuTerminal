# AGENTS.md and execution priority

Repository-root [`AGENTS.md`](https://github.com/CorbanuCore/CorbanuTerminal/blob/main/AGENTS.md)
is the canonical Corbanu development policy. The generic
[AGENTS.md guide](https://developers.openai.com/codex/guides/agents-md)
explains how agents discover those instructions.

## Product-initiative execution order

| Order | Record | Question answered |
| ---: | --- | --- |
| 1 | [Product specification](corbanu-product-spec.md) | What outcome is authorized? |
| 2 | [Plan](plans/index.md) | Which feature contract, scope, sequencing, and acceptance model apply? |
| 3 | [Current sprint](sprints/index.md) | What exact single-feature code tasks may be executed now? |
| 4 | `qa/release/<version>/` | What final-tree, TUI, live-repository, and human evidence proves it? |
| 5 | Feature documentation | What verified behavior is finished? |

Agents do not implement a product initiative directly from a plan. They select
one `ready` or `in_progress` sprint, work only its remaining checklist in the
recorded worktree, and update its separate `Done` and `Remaining` ledgers.
Completed sprint records move to the excluded archive and leave the docs view.

Use the [sprint process](sprints/index.md) for the exact lifecycle and checker.
