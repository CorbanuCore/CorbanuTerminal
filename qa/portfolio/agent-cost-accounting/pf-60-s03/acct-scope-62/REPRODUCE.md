# Reproduce acct-scope-62

Read `docs/development/test-isolation.md` first. Run from this checkout; prefix
all paths below with `qa/portfolio/agent-cost-accounting/pf-60-s03/`. Choose new
output names; existing attempts and receipts are never overwritten.

1. `python3 acct-boundaries-58/campaign.py acct-scope-62/NEW-GATES`
   builds CLI/rmcp prerequisites, runs the three guarded lanes from `codex-rs`
   with `NEXTEST_TEST_THREADS=4`, and builds the developer-accounting CLI. Its
   shared dedicated `CARGO_TARGET_DIR` is `acct-activation-33/feature/target`.
2. `python3 acct-scope-62/coverage_controls.py acct-scope-62/NEW-CONTROLS`
   re-audits round-61 final captures; removes exact amounts from all 15 expected
   pages and displayed amounts from the 13 priced/unknown pages. It evaluates
   the frozen base auditor on each same mutation and the revised auditor, records
   exact errors and receipt existence, and requires 28 new rejections. The old
   auditor accepts 26; its two narrow-overview cases already reject. Mutations
   are labelled counterexamples, never attributed to the product.
3. `python3 acct-readers-61/negative_controls.py acct-readers-61/boundary-run-02
   acct-scope-62/NEW-TZ.json` reproduces the six negative controls, including a
   new TZ-stripped invocation of the built CLI. The four positive timezone
   controls are checked from the preserved round-61 run, not claimed as rerun.
   The TZ case deliberately omits the unevaluated `old_check_passes` field.
4. `RUST_LOG=warn uv run --offline --script acct-scope-62/scope_repro.py BINARY
   acct-scope-62/NEW-SCOPE --mode collect`, where BINARY is
   `acct-activation-33/feature/target/debug/codex`, collects four priced requests
   across three roots, opens a fresh-session day inspector, preserves store
   readback before/during, and reopens each contributing root and attempt.
   It reuses the existing round-61 clock dylib and helpers. All profile state,
   config, prices and loopback emissions are synthetic; no live profile is used.
5. `python3 acct-scope-62/audit_scope.py acct-scope-62/NEW-SCOPE
   acct-scope-62/NEW-SCOPE-AUDIT.json` independently binds store contributions,
   quote evidence, reported usage, frozen rates and all seven priced pages.
6. Read `reader-findings.md` and `residuals.md`. Successful QA assertions prove
   observations and arithmetic; they do not resolve the P2 product issue or
   qualify the blocked two-collected-provider and measured-dollar workflows.

No production changes, workspace formatting, raw cargo tests, push or acceptance
approval. `finalize.py` writes this allocation's evidence indexes and compressed
logs after the recorded runs; its recorded input paths are intentionally fixed.
