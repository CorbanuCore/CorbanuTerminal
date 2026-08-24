# RW-TMUX-01 verification results
Date: 2026-08-24  
Branch: `codex/tmux-harness-foundation`  
Implementation: `72b72eedc`
Status: completed
## Scope and results
The original package changed only `tui/tests/all.rs`,
`tui/tests/support/{mod,tmux,tmux_tests}.rs`,
`tui/tests/suite/resize_reflow.rs`, and this work-package record and evidence.
A build-infrastructure follow-up changed `core/tests/common/lib.rs` so Apple test
constructors are emitted in the regular text section. No product source,
configuration, protocol, snapshot, CI, or release file changed.
- tmux 3.7c; focused TMUX target: 7/7 passed.
- Full owning `all` integration target: 17/17 passed, two intentional skips.
- Six harness contracts repeated 20 times: zero failures in 36 seconds.
- Migrated workflow repeated 20 times: zero failures in 131 seconds.
- The default-server sentinel survived both sequences; private roots, sockets,
  tmux processes, and child processes were absent afterward, including on panic.
- `just fix -p codex-tui`, `just fmt`, snapshot audit, and `git diff --check` passed.
- Final local autoreview reported no actionable issue.
Early failures sampled a transient pane state before the restored-height resize
event. The semantic stable wait now requires identical matching captures; no
product defect or product-source change remained.

## Build-infrastructure follow-up

The original crate-wide attempt exposed a macOS ARM64 `ld: B/BL out of range`
error while linking the oversized `codex-tui` library test binary. The failing
branch originated in a `core_test_support` constructor body emitted into the
special startup text section. Emitting the three Apple constructor bodies into
the regular `__TEXT,__text` section lets the linker place branch islands while
preserving the constructor registration and all non-Apple behavior.

- `just test -p core_test_support`: 36/36 passed.
- Exact default test profile and default Apple linker, with no `RUSTFLAGS`,
  deployment-target override, or debug-profile override: the full-size
  `codex-tui` library test executable linked and the selected test passed.
- Full `just test -p codex-tui` execution with the default Apple linker:
  3,780 passed and 41 failed among 3,821 executed tests, with 8 additional tests
  skipped.
- The 41 failures comprise 37 stale UI snapshot baselines and four macOS MkDocs
  assertions comparing `/var/...` with its canonical `/private/var/...` path.
  These are pre-existing test debt, not failures in the TMUX harness or linker
  repair. Generated `.snap.new` files were rejected rather than accepted.
