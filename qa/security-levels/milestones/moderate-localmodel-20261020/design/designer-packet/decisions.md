# Owner decisions this amendment must follow

Verbatim from the owner's decision log and issue tracker (Travis), unless marked "integrator". Issue numbers are the
product's issue tracker; you do not need them beyond telling the decisions apart.

## Earlier decisions on these two cases

- An earlier owner decision (cited in amendment v3 as A(13); applied by its R-1): re-run MOD-09 and MOD-21 once with
  the product's model GLM 5.2. In that re-run GLM 5.2 also declined the credential reads before any tool ran.
- **D39** (2026-10-19): accept MOD-09 and MOD-21 as known limits (two models declined before any tool ran). Not re-run.
- **D61** (2026-10-20): Moderate signed off, with MOD-09 and MOD-21 accepted under D39.

## The decision for this re-run

- **Owner request with D61** (issue #1052, 2026-10-20): "code-blind milestone runs had cases blocked because the
  executor model refused before any tool ran (MOD-09, MOD-21 ...). A local abliterated Qwen ... runs on the Mac host
  ... re-run MOD-09/MOD-21 ... code-blind with it as the product model, so the product's credential protections are
  actually exercised; ... record results as a follow-up to the Moderate record."
- **D67** (2026-10-20): the freed plan slot goes to this sprint (re-test refusal-blocked cases with the local model).
- **Coordinator's brief** (2026-10-20, verbatim): "the owner wants these re-run with a locally hosted model as the
  product model, which follows the test script, so the product's protection is what gets tested."
  "GLM 5.3 Flash on zai is the tester driving the TUI, the local model is the product model."
- **D68** (2026-10-20): Tart is the default virtual machine for test runs.

## Integrator choices

- The product's model is the owner's local model `Qwen3.8-Flash-Next-Uncensored-MLX-Serve-mixed-4-8bit` (served on
  the host; reached by the product as the custom OpenAI-style connection `qwen`; see `tools.md`). It is a model with
  its refusals removed, so it is expected to attempt the requested steps.
- The tester model stays GLM 5.3 Flash on Z.AI.
- One tester group per case: R01 = MOD-09, R02 = MOD-21. Fresh virtual machine (a Tart clone), fresh profile and
  test project per group.
- The candidate is the same as the second targeted re-run's (the signed build of main `8bf182de43`, on which Moderate
  was signed off), so only the product's model changes.
- Rules G-1..G-3 and the named limits of amendments v2-v4 stay in force unless this amendment changes them. Amendment
  v3's R-1 (product model GLM 5.2 for these two cases) is what this re-run replaces.
