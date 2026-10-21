# Scope of amendment v7

34 cases were designed (MOD-01..33 in `design-cases.md`, MOD-34 in `amendments-v2.md`). Amendments v2-v5 are in force
(precedence: v5 over v4 over v3 over v2 over the frozen cases). Amendment v7 applies on top of them, to future runs of
the three cases below only.

| Case | Why it is amended (D97, `decisions.md`) | In force before this amendment |
| --- | --- | --- |
| MOD-12 | MCP tool naming: the tester called the case's `get_note`/`inject` literally | frozen + v2 (the notes server's read_shared_note is the case's inject/get_note; added .ssh step) |
| MOD-23 | Which config file to edit for "`source_envelopes` is off" | frozen only |
| MOD-29 | How to reset the profile | frozen + v3 (actions 2-3, the relaunch, the `.env` expected result) |

## What the amendment must do

Make the wording unambiguous for a tester who has only the tester packet. Do not tighten or loosen anything; the
expected results stay as in force.
