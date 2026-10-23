# Review disposition (Opus 5.5 High, approve with fixes)

1. Root skips: the refusal is now a pure function, `standalone_refusal_for`, unit-tested in `codex-security-level` for every kind in both modes (runs as root). The integration tests print a note when they skip. Fixed.
2. No `codex-app-server`/`codex-mcp-server` integration test: covered by the unit test of the shared function (Host kind) and the identical call site; recorded, not added.
3. Weak "no origin" test and no crate tests: the crate now has its own tests (standalone refusals, `decide` unchanged). The remaining detection tests stay in `codex-tui`, where they exercise the re-exports; recorded.
4. Contradictory README sentence: reworded. Fixed.
5. Info: `corbanu stdio-to-uds` is now a host (refused nested), with a test. "Standalone binaries started by a person do not apply a stored level" and "older binaries on PATH" are recorded under Limits.
