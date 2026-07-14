# PFTerminal 0.1.11

## Fixed

- Kept the TUI alive when a hierarchy dispatch hits a busy pane or a per-pane `turn/start`
  request fails; work sent to a running native pane is now steered into that turn.
- Fixed remote compaction request ordering so `compaction_trigger` remains the final input item,
  and normalized accidental assistant-prefill requests without rewriting durable history.
- Added an explicit child-to-parent checkpoint channel so Troll and Orc progress can reach their
  manager during long-running turns without self-dispatching or starting competing turns.
- Removed the global single-process state lock so users can run multiple PFTerminal processes.

## Changed

- Removed the game-specific visual workflow, mandatory evidence headers, visual-judge command,
  and hard-coded hierarchy disk policy. Hierarchy dispatches are domain-neutral again.
- Changed the standard crew to Claude Fable Plan Nazgul, GPT-5.6-Sol Troll, GPT-5.6-Luna and
  GPT-5.6-Terra Orcs, plus an OpenRouter Grok 4.5 Orc.
- Self-dispatch is rejected as a routing error; panes report upward through the checkpoint channel.

## Qualification status

- Focused regressions cover compaction ordering, assistant-prefill normalization, free-form
  dispatch, busy-pane steering, parent checkpoints, self-dispatch rejection, and crew creation.
- This combined emergency patch has not completed extended human stress qualification. The release
  is being published because the defects in 0.1.10 materially block normal use.

Previous release: 0.1.10.

The changelog can be found on the [releases page](https://github.com/agtico/PfTerminal/releases).
