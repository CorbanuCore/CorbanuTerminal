# Owner decision this amendment must follow

Verbatim from the owner's decision log (Travis), unless marked "integrator".

## Background (integrator, from the record of the Tart parity run of 2026-10-20)

In the Tart parity run, three cases were blocked because the AI tester misread the case text; a replay with a fresh
tester passed each of them:
- **MOD-12:** the tester sent the case's literal "Call get_note on the inject server" instead of using the notes
  server's `read_shared_note` tool that amendments-v2 maps it to, so no MCP tool call was made.
- **MOD-23:** the tester's file tool wrote `source_envelopes = false` to `$AGMS_RUN/work/product-home/config.toml` (a
  path under its working folder), not to the product's home `$AGMS_RUN/product-home/config.toml`, so the starting
  condition (labels off) was never met.
- **MOD-29:** run in the same group after another case, the tester "reset" the profile by emptying
  `$AGMS_RUN/product-home`, which deleted the supplied `config.toml`; `/security` then opened read-only and Moderate
  could not be reached.

## The decision

- **D97** (2026-10-20; the owner approved the recommendation; coordinator's brief, verbatim): "fix tester-confusing
  wording in MOD-12 (MCP tool naming), MOD-23 (which config file to edit), MOD-29 (how to reset the profile); and add
  to the parity method doc that parity runs use the same build on both machines." (The parity method part is not a
  case text and is handled outside this amendment.)

## Integrator choices

- Wording only: say exactly which tool to call (MOD-12), which file to edit and how (MOD-23), and how to reset the
  profile without losing the supplied `config.toml` (MOD-29). No expected result changes.
- Amendment v7 touches MOD-12, MOD-23 and MOD-29 only. A separate amendment v6 (D96) changes MOD-22; the two do not
  overlap and neither depends on the other.
