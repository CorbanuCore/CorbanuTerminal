# Corbanu Terminal 0.1.35 docs overlay hotfix

Date: 2026-08-24 UTC

Status: implementation and focused qualification complete; include in the 0.1.35 release candidate. This record does not qualify 0.1.35 for publication. The benchmark bootstrap gate in `benchmarks/README.md` remains pending, and the full TUI suite currently has unrelated stale 0.1.31 version snapshots.

## Product contract

Product specification heading: `# Shipping MVP — LIVE`

Requirement excerpt: “Rust, Apache-2.0, Linux/macOS/Windows, the `corbanu` command, and legacy `pfterminal` command and state compatibility.”

The live `corbanu` docs overlay must not crash while navigating a repository-owned MkDocs tree.

## Failure and fix

A large documentation index with a nonzero list scroll reached `MkDocsOverlay::ensure_selected_visible_for_height(usize::MAX)`. The unbounded-height sentinel was added directly to `list_scroll`, overflowing in debug builds at `tui/src/mkdocs_overlay.rs:757`.

The visibility boundary now uses saturating addition and subtraction. A regression constructs a 300-page site with a nonzero scroll and exercises the same unbounded visibility call.

## Automated evidence

- Before the fix: `cargo test -p codex-tui unbounded_visibility_check_does_not_overflow_a_scrolled_large_site --lib -- --nocapture` failed with `attempt to add with overflow` at `tui/src/mkdocs_overlay.rs:757`.
- After the fix: `cargo test -p codex-tui mkdocs_overlay::tests --lib` passed 4 tests, 0 failed.
- `just fmt` passed.
- `git diff --check` passed.
- `just test -p codex-tui` ran 3,813 tests: 3,783 passed, 30 failed, 9 skipped. The 30 failures are unrelated release-version snapshots expecting 0.1.31 while the workspace reports 0.1.35; generated `.snap.new` files were removed. The docs regression passed in this run.
- `cargo build -p codex-cli --bin corbanu-debug` completed successfully. The binary reports `corbanu 0.1.35`.

## True-TUI evidence

The debug binary was launched in a 160x48 tmux PTY from the repository root with `RUST_BACKTRACE=1`, `RUST_LOG=trace`, `--yolo`, and a temporary `log_dir`.

1. Submitted `/docs` with text and Enter as separate key events.
2. Confirmed the overlay loaded 128 pages from the current `mkdocs.yml`, including the new sprint tree.
3. Sent `End`, waited for the list to scroll, then sent `Up` and `Down` to exercise the prior overflow state.
4. Confirmed `pane_dead=0` and the bottom of the page list remained rendered.
5. Sent `Enter` and confirmed page focus rendered.
6. Sent `q` and confirmed the overlay closed while the terminal process remained alive.
7. Searched the trace directory for `panicked`, `attempt to add with overflow`, and `mkdocs_overlay.rs:757`; no markers were present.

## Release blockers outside this fix

- Complete the mandatory benchmark bootstrap cycle and evidence package described in `benchmarks/README.md` before publishing 0.1.35.
- Refresh or otherwise resolve the stale 0.1.31 TUI version snapshots through the normal release preparation flow, then rerun the full `codex-tui` gate.
- Obtain the required human release sign-off before publication.
