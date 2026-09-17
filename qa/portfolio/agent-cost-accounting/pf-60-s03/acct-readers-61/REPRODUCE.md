# Reproduce acct-readers-61

Use this macOS checkout and read docs/development/test-isolation.md first.
Every path below starts with qa/portfolio/agent-cost-accounting/pf-60-s03/.
Choose new output directories; never overwrite a recorded attempt.

1. Run `python3 acct-boundaries-58/campaign.py NEW-GATES` from the repository
   root with the full prefixed paths. This builds CLI/rmcp prerequisites, runs
   the three guarded just-test lanes with four test threads, and builds the
   developer-accounting CLI. It uses the shared dedicated target at
   acct-activation-33/feature/target. No raw cargo test or broad formatter.
2. Build the QA clock/probe:
   `clang -dynamiclib -o acct-readers-61/tz_clock.dylib acct-readers-61/tz_clock.c`.
   The source includes the existing round-58 clock. Prefix both paths.
3. Run `RUST_LOG=warn uv run --offline --script acct-boundaries-58/boundaries.py
   BINARY NEW-BOUNDARIES --mode collect`. Prefix paths; BINARY is
   acct-activation-33/feature/target/debug/codex. This now requires the per-PID
   timezone probe and asserts requested day plus compact-hour explanation.
4. Audit with `python3 acct-boundaries-58/audit_boundaries.py NEW-BOUNDARIES
   NEW-BOUNDARY-AUDIT.json`, and `python3 acct-readers-61/negative_controls.py
   NEW-BOUNDARIES NEW-NEGATIVES.json`.
5. Run `RUST_LOG=warn uv run --offline --script acct-readers-61/readers.py BINARY
   NEW-READERS --mode collect`. It uses fresh synthetic profiles and loopback
   inference, then preserves exact keys, viewports, selected text, counts and
   declared fault injections. The stale marker is restored; the backend failure
   is cleaned up by restoring an exact fixture-only SQLite backup after the TUI
   exits. No live profile or native credential store is used.
6. Run `python3 acct-readers-61/audit_readers.py NEW-READERS NEW-READER-AUDIT.json`.
   It independently checks exact/rounded monetary amounts, components, rates,
   token metrics, price/usage absences, omission of custom-provider collection,
   stale readback, backend failure and 64×24 rendering.
7. Review reader-findings.md. A successful script run is not a pass for every
   product requirement: missing-price next steps and mixed-provider completeness
   remain disclosed gaps. Independent functional acceptance belongs to the
   manager's later gate.

RUST_LOG=warn quiets uv only; the shared helper sets RUST_LOG=trace for each TUI.
The probe's constructor writes only PID, TZ, fixture time and libc localtime.
It executes inside the actual product process. Separate PID filenames prevent a
child overwriting the parent's observation. A deliberately TZ-stripped invocation
of the exact product is rejected by the same assertion. This is instrumentation
of process-local timezone behavior, not a claim that a local timestamp is
displayed in the UTC-only inspector.

The strengthened boundary audit intentionally cannot qualify historical
round-58 captures as timezone-proven: those runs have no in-process positive
control. Their historical receipts remain untouched; use this new run for the
stronger claim. The prior day headers can still support the new day-binding
counterexamples without changing old capture bytes.
