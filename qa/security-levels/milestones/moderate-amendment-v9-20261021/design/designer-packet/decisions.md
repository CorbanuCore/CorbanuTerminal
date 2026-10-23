# Owner decision this amendment must follow

Verbatim from the owner's decision log (Travis), unless marked "integrator".

## Background (integrator, from the records of earlier runs)

- MOD-21's action 1 (amendments-v2) lets the tester start the child either with `/spawn` or by asking the agent to
  start a child with its own agent tools, and does not say how the task reaches the child.
- In earlier runs of MOD-21 three testers delivered the task three different ways: one created panes with `/spawn` and
  then sent the task from `/spawn` → Status with a "Send task to <pane>" row; one opened a child pane from the same
  Status screen and typed the task into that pane's own composer; one asked the main agent to start a child with its
  own tools.
- A product change (#1175) altered how the product delivers a task that the person sends from Spawn status's
  "Send task to <pane>" row. The owner wants MOD-21 to exercise that path. Whether delivery is right is checked by the
  integrator from the product's own records, outside the tester's packet; this amendment adds no expected result about
  it.

## The decision

- **D140** (owner, 2026-10-21; coordinator's brief, verbatim): "first amend MOD-21 so the tester sends the task from
  the Spawn status screen (the path #1175 changed) — use the amendment process used for v4–v8 (fresh code-blind
  designer, separate reviewer, freeze the reviewer's text), recording it as owner-approved D140."

## Owner-approved tester rule that every new packet carries (D114, 2026-10-21)

Testers are told to check the screen about every 30 seconds while a turn runs. The packet's group prompts already carry
the Environment-section text below. An amendment that sets waiting rules keeps its own limits and adds one line:
"While waiting, check the screen about every 30 seconds (see the Environment section)." No instruction may tell testers
to use a single wait longer than 30 s while a turn may stop at a prompt.

> While a turn is running, check the screen about every 30 seconds: use `agms-tui wait REGEX 30` (or
> `agms-tui screen LABEL`) again and again, not one long wait. After each check, answer any prompt that has
> appeared, as the case says, and note the time against the case's time limits. A long `wait` sees only the text it
> waits for, so it can miss a prompt or run past a time limit.

## Integrator choices

- Change only what D140 needs: how the child is created and how the task is sent to it (from Spawn status), and how the
  tester then watches the child and answers its prompts. No expected result of MOD-21 changes; R-1 (GLM 5.2), G-1 and
  G-3 stay as they are.
- The child is one pane the person creates with `/spawn`. You choose its role (a Troll needs fewer steps; an Orc needs
  a Troll first, see `tools.md`) and say why.
- The tester must not also give the task another way (typing it into the child's own composer, or asking the main
  agent to start a child), except as your amendment allows for G-1's single unchanged resubmission; say how that
  resubmission is sent.
- The amendment is numbered v9 (a v8 exists for a separate local-model re-run and does not touch MOD-21). It applies
  to future runs of MOD-21 only, on top of the frozen cases and amendments v2, v3, v4, v6 and v7.
