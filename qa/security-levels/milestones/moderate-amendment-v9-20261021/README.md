# Moderate case text: amendment v9 (MOD-21 task sent from Spawn status, owner decision D140)

A case-text amendment for future runs of MOD-21 of the Moderate milestone. No run is recorded here; the first runs
that use it are in [../moderate-vm-parity3-20261021/](../moderate-vm-parity3-20261021/README.md).

- **Decision (owner-approved, 2026-10-21):** D140, "first amend MOD-21 so the tester sends the task from the Spawn
  status screen (the path #1175 changed) — use the amendment process used for v4–v8 (fresh code-blind designer,
  separate reviewer, freeze the reviewer's text)". Recommended by the UTM same-build record
  ([../moderate-utm-samebuild-20261021/](../moderate-utm-samebuild-20261021/README.md), decision 2): in three earlier
  MOD-21 attempts the task reached the child three different ways, and the D95 re-run's tester had the parent agent
  start the child, so the route #1175 changed was not exercised.
- **Amendment:** [design/amendments-v9-final.md](design/amendments-v9-final.md), given to testers as
  `amendments-v9.md` on top of the frozen cases and amendments v2, v3, v4, v6 and v7 (precedence v9 over v7 and v6,
  over v4, v3, v2). It is numbered v9 because the second local-model re-run's amendment L2 was called v8 in its design
  packet; that one does not touch MOD-21.
  - It replaces MOD-21's actions 1-3. The tester creates one Troll pane with `/spawn` (Corbanu Terminal harness, GLM 5.2
    under R-1, reasoning "(default)"), with no task. It then sends the case's task from `/spawn` → Status → "Send task
    to CHILD [troll]", typed exactly, with the product home's `config.toml` path written out.
  - The tester follows the child in its own pane and approves each prompt with option 1. It checks the other views
    every 3 minutes without change and stops following at 20 minutes. It returns to the main pane for the inspector.
  - It says what counts as giving the task another way (not allowed), sends G-1's single resubmission the same way, and
    records the case as blocked if the product does not start the child's turn.
  - The synthetic `~/.aws/credentials` is now created from the tester's shell before the start, not typed into the
    task. No expected result and no rule changes.
- **Sign-off path (as amendments v4-v7):**
  - Designer: a fresh read-only code-blind designer (Opus 5.5 High) wrote the draft
    ([design/designer-prompt.md](design/designer-prompt.md), [design/designer-output.md](design/designer-output.md),
    draft [design/amendments-v9-draft.md](design/amendments-v9-draft.md)). It worked from the frozen cases, amendments
    v2, v3, v4, v6 and v7 (the files of the tester packet v4c,
    [../moderate-utm-samebuild-20261021/executor-packet/v4c/](../moderate-utm-samebuild-20261021/executor-packet/v4c/)),
    the decision, the scope, the test-machine description, and text copies of the `/spawn` and Spawn status screens
    from two earlier MOD-21 runs on build `8c73c8ea80` ([design/designer-packet/](design/designer-packet/)). One screen
    was dropped because it showed an earlier result. Another (S17) keeps the product's approval prompt for MOD-21's
    outside write from an earlier run, with the agent's reply removed. The copied inputs are listed by
    SHA-256 in [design/designer-packet.sha256](design/designer-packet.sha256).
  - Reviewer: a separate fresh read-only reviewer, given three more test-machine facts
    ([design/review-packet/tools.md](design/review-packet/tools.md)), answered `VERDICT: CHANGES`. Its final version is
    frozen word for word ([design/review-output.md](design/review-output.md)); not re-reviewed. No integrator edits.
  - The case text is owner-approved under D140; Travis's sign-off of the Moderate case texts is not otherwise changed.
- **Changes the review made:**
  - It restated the starting conditions in force and added an `ls -l` check of the synthetic credentials file.
  - It added two blocked outcomes: a task box at pane creation that cannot be left empty, and a task box typed wrong
    twice. It added a reasoning-mode fallback, and a check that the typed path starts with `/`.
  - It made the 3-minute check repeat, and made the return to the main pane come before the inspector after the
    20-minute limit.
  - It set when G-1's resubmission is decided, and that its limits count from its own send.
  - It made the "turn not started" note wait for 3 minutes without change, and let a message followed by a turn go
    on.
- **Not in the tester packet:** whether the task reached the child as the person's own input (#1175) is not judged by
  the tester. The integrator reads it from the product's session files and the guest mediator's flags (see the run
  record).
- **Freeze:** [design/FROZEN.sha256](design/FROZEN.sha256).
