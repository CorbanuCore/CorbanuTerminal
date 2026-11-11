# Scope of amendment v9

34 cases were designed (MOD-01..33 in `design-cases.md`, MOD-34 in `amendments-v2.md`). The tester packet for the runs
this amendment is for holds `design-cases.md`, `amendments.md` (= `amendments-v2.md` here), `amendments-v3.md`,
`amendments-v4.md`, `amendments-v6.md` and `amendments-v7.md`, with this precedence: v7 and v6 over v4, over v3, over
v2, over the frozen cases. `amendments-v5.md` (mentioned by v6 and v7) belongs to a separate local-model re-run and is
not in the tester packet; it does not touch MOD-21. Amendment v9 applies on top of them, to future runs of MOD-21 only.

| Case | Why it is amended | In force before this amendment |
| --- | --- | --- |
| MOD-21 | D140 (`decisions.md`): the task must be sent from the Spawn status screen | frozen + v2 (action 1 replaced: `/spawn` a child or ask the agent to start one with its own tools; synthetic `~/.aws/credentials` created from the tester's own shell; "the product home's config.toml"; G-3 note) + v3 (R-1: product and child model GLM 5.2, launch with `agms-tui start --model glm-5.2`; record the child's model; R-1, G-1, G-3 apply); v4, v6 and v7 do not touch it |

## What the amendment must do

Replace MOD-21's action 1 so that the child is created with `/spawn` and the task is sent from `/spawn` → Status,
"Send task to <pane>", and say how the tester follows the child and answers its prompts (action 2) and checks the
inspector (action 3). Do not tighten or loosen anything else; the expected results of MOD-21 stay as in force.
