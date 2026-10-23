# #443 follow-up: a command the sandbox stops as it starts is shown (2026-10-11)

**Observation.** In the milestone (AGG-04, evidence check r5), the agent's `head -3 "$CORBANU_HOME/config.toml"` ran and
the protected-path rules denied it, but the command never appeared on screen; only the model's summary did. A
unified-exec process that ends within the spawn-time sandbox check returned `SandboxDenied` before the begin event, so
neither begin nor end was emitted.

**Change.** Such a command now emits begin and end events with its output and exit code, like any command that ran.

## Live check (tmux, GLM 5.3 Flash)

Same setup and script as `../issue-443-20261011` (`case443.sh`): confirm Aggressive, press Esc on screen 07, then ask
for `head -3 "$CORBANU_HOME/config.toml"`.

| Build | Screen |
| --- | --- |
| before (#443 branch at `03f165ac28`) | no command line; only the model's text quoting the denial ([capture](captures/before-home.txt)) |
| after (this branch at `492a40066b`) | `• Ran head -3 "$CORBANU_HOME/config.toml"` and `└ head: …: Operation not permitted` ([capture](captures/after-home.txt)) |

The two captures also differ on the earlier `corbanu vault list` step. The before binary is the #443 branch, which
refuses it. The after binary is this branch, cut from `main` without #443, so the command runs and fails with a usage
error. That difference is unrelated to this change.

Not covered here (found in review):

- **Deferred network check.** When a deferred network check also fails, the denial is replaced by a rejection, and the
  command is still not shown.
- **Sandbox start-up failures.** On Linux (bwrap/namespace) these also arrive as a sandbox denial and now render as
  "Ran" with the error output, although the command did not start.
- **Read-like commands.** A denied read-like command renders in an "Explored" cell, which is upstream behaviour.
