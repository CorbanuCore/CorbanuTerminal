# RW-TMUX-01 verification results
Date: 2026-08-24  
Branch: `codex/tmux-harness-foundation`  
Implementation: `72b72eedc`
Status: completed
## Scope and results
Changed only `tui/tests/all.rs`, `tui/tests/support/{mod,tmux,tmux_tests}.rs`,
`tui/tests/suite/resize_reflow.rs`, and this work-package record and evidence.
No product source, configuration, protocol, snapshot, CI, or release file changed.
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
product defect or product-source change remained. `just test -p codex-tui` was
also attempted twice, but this host's oversized library test binary fails before
execution with macOS ARM64 `ld: B/BL out of range`. The separately linked,
affected `all` integration target passes in full.
