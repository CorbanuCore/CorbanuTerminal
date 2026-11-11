# Moderate case text: amendment v7 (tester wording in MOD-12, MOD-23, MOD-29; owner decision D97)

A case-text amendment for future runs of MOD-12, MOD-23 and MOD-29 of the Moderate milestone, and the parity-method
rule of the same decision. No run is recorded here.

- **Decision (owner-approved, 2026-10-20):** D97, "fix tester-confusing wording in MOD-12 (MCP tool naming), MOD-23
  (which config file to edit), MOD-29 (how to reset the profile); and add to the parity method doc that parity runs use
  the same build on both machines." Recommended by the Tart parity run
  ([record](../moderate-tart-parity-20261020/README.md)): each of the three cases was blocked once by a tester
  misreading the text (MOD-12 called the case's `get_note`/`inject` names literally; MOD-23 edited
  `$AGMS_RUN/work/product-home/config.toml` instead of the product's home; MOD-29 emptied the product home and deleted
  the supplied `config.toml`), and a fresh tester passed each.
- **Amendment:** [design/amendments-v7-final.md](design/amendments-v7-final.md), given to testers as
  `amendments-v7.md` on top of the frozen cases and amendments v2-v5 (precedence v7 over v5 over v4 over v3 over v2).
  It adds the product-home path rules and a profile reset (PR-1..PR-4: stop, record, empty the home and copy back the
  supplied `config.toml` from `/opt/agms/etc/product-config.toml`, check), uses it for a fresh profile in all three
  cases, names the notes server's `read_shared_note` tool in MOD-12's request, gives MOD-23 one exact edit of
  `source_envelopes` in `$AGMS_RUN/product-home/config.toml` with a check, and adds a picker check before Moderate in
  MOD-29. Wording and setup only: no expected result and no rule changes.
- **Sign-off path (as amendments v4 and v5):** a fresh read-only code-blind designer (Opus 5.5 High) wrote the draft
  from the frozen cases, amendments v2-v5, the decision, the scope and the test-machine description
  ([design/designer-prompt.md](design/designer-prompt.md), [design/designer-output.md](design/designer-output.md),
  draft [design/amendments-v7-draft.md](design/amendments-v7-draft.md); the new packet files are in
  [design/designer-packet/](design/designer-packet/), the copied inputs are listed by SHA-256 in
  [design/designer-packet.sha256](design/designer-packet.sha256); the frozen cases and amendments v2-v5 are the files of
  [moderate-localmodel-20261020/design/designer-packet/](../moderate-localmodel-20261020/design/designer-packet/) plus
  its `amendments-v5-final.md`). A separate fresh read-only reviewer, given two more test-machine facts
  ([design/review-packet/tools.md](design/review-packet/tools.md)), answered `VERDICT: CHANGES`; its final version is
  frozen word for word ([design/review-output.md](design/review-output.md)); not re-reviewed. No integrator edits. The
  case text is owner-approved under D97; Travis's sign-off of the Moderate case texts is not otherwise changed.
- **Beyond the three named fixes:** the reviewer also gave MOD-12 the same fresh-profile reset (so it cannot hit the
  MOD-29 failure when it runs after another case in a group). It is setup wording only and kept as frozen; the
  reviewer noted it can be dropped to keep strictly to D97's MOD-12 item.
- **Parity method:** the same-build rule is in [../vm-parity-method.md](../vm-parity-method.md). The Tart run's
  predeclared method stays frozen with that run.
- **Freeze:** [design/FROZEN.sha256](design/FROZEN.sha256).
- **Separate:** amendment v6 (D96: MOD-22) is its own record; the two do not overlap.
