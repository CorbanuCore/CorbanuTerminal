# PFTerminal 0.1.10

## Fixed

- Prevented hierarchy-wide termination after routine `turn/start` failures and capacity pressure.
- Removed the capacity-retry notification flood and hardened zombie/stillborn pane recovery.
- Added a durable exactly-once dispatch queue with crash-consistent persistence and ordered
  shutdown that fences headless writers.
- Agent compaction is no longer treated as agent death, and encrypted task payloads no longer
  replace readable `/spawn status` task text.

## Qualified

- Automated gates 16.1–16.4 passed, including the deterministic ten-cut crash matrix and the
  full-process PTY gate.
- Free-form stress testing found and fixed two dispatch correctness defects.
- One clean 1,200-second free-form session passed on the qualified release lineage. The final
  section-17 45–60 minute qualification was not completed and is not claimed as passing.

## Known issues

- The required implementer-blind 45–60 minute free-form session was not run before release.
- Queued panes can be briefly mislabeled during bootstrap.
- Grok 4.5 can emit integer tool arguments as JSON floats, causing typed tool-call rejection.

Previous release: New OpenAI models GPT-5.6 Sol/Terra/Luna (ChatGPT plan).

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).
