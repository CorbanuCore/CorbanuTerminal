# Reproduce round 58

Run from the repository root on macOS. Read docs/development/test-isolation.md.
All paths below are under qa/portfolio/agent-cost-accounting/pf-60-s03/.
Choose unused run/output paths; never overwrite a returned attempt.

1. Run `python3 <prefix>/acct-boundaries-58/campaign.py NEW-CAMPAIGN-DIRECTORY`.
   It builds CLI/rmcp prerequisites, runs the three guarded just test lanes with
   NEXTEST_TEST_THREADS=4, then builds the developer-accounting CLI. All use the
   dedicated shared target at acct-activation-33/feature/target.
2. Compile each clock asset with `clang -dynamiclib -o PATH/fixture_clock.dylib
   PATH/fixture_clock.c` for PATH=acct-qualify-55 and acct-boundaries-58.
3. Replay P3 fixtures with `RUST_LOG=warn uv run --offline --script
   <prefix>/acct-qualify-55/qualify.py BINARY NEW-P3-RUN --mode collect`.
4. Replay boundaries with `RUST_LOG=warn uv run --offline --script
   <prefix>/acct-boundaries-58/boundaries.py BINARY NEW-BOUNDARY-RUN --mode collect`.
   BINARY is <prefix>/acct-activation-33/feature/target/debug/codex.
   RUST_LOG=warn quiets uv; the helper explicitly sets trace for the TUI.
5. Audit P3 selections using `python3 <prefix>/acct-qualify-55/audit.py
   NEW-P3-RUN NEW-AUDIT-RECEIPT.json`; audit boundaries using
   `python3 <prefix>/acct-boundaries-58/audit_boundaries.py
   NEW-BOUNDARY-RUN NEW-AUDIT-RECEIPT.json`.
6. Historical UTC verification is `python3
   <prefix>/acct-qualify-55/verify_utc_alignment.py`. It compares regenerated
   round-55 bytes against untouched round-50 bytes; mismatches fail.
   `check_p3.py` records negative controls against p3-run-01 and uses
   temporary synthetic copies for corruptions. Its output uses exclusive create;
   retain the existing receipt and choose a fresh output filename for another run.

Both drivers preserve their source and expanded helpers, binary identity,
emitted counts, chosen clock times, actual keys, selected text, and lossless PTY
streams. Expectation arithmetic is independent of storage: inclusive input minus
cached input, cached input, and output multiply fixed Sol/Terra rates, divided by
one million. Reasoning is an output subset and is not billed a second time.
Expected membership comes from driver-chosen UTC windows and emitted timestamps.
Native thread identities/ancestry and retention status use store read-backs;
this is code-aware evidence, not independent ancestry proof.

The boundary driver varies TZ, executes native thread/delete on its synthetic
child through app-server, and advances time with genuine loopback requests to
invoke normal retention. It never writes accounting attempts, observed usage,
prices, compact rows or checkpoints. Only the P3 replay injects disclosed native
source/edge faults, restoring them afterwards.

Raw .raw/.txt and synthetic profiles stay ignored; lossless .gz captures and
selected JSON are reviewable. Use a fresh fixture for full reproduction; archived
driver sources describe earlier interrupted/failed attempts. No live credentials,
network inference, OS-clock change or machine-wide formatter is required.
