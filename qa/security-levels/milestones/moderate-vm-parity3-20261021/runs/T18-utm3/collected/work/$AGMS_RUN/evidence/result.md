# MOD-22 functional execution report

## Identity and isolation preflight

- UTC executed: 2026-10-21.
- Candidate expected and observed SHA-256: `0de41e09da09c934fcf6db917e95a9948e107a918b02d804b1c0fa2d263cb5d7`; `/opt/agms/pkg/bin/corbanu --version` observed via preflight as `corbanu 0.1.48`.
- Frozen packet SHA-256 observed: `64ecf9c951f89d645e0b2eb3243dd422c80436f24fadb4d9283a5daa97239c23`.
- Amendment packet SHA-256 values observed: v2 `203685710b74d3cf864ed00e35a5c6e30e10296ea82ec60ab707b09b53da02d1`; v3 `8ebf87cbddcfa2f67d7399aee4ff06ee7304b6835059f7fde2a31ed8db7bd4d3`; v4 `b378112f9cb0ad8761ec0be500238e2565f842c4363785c1c9309fcef879f0ae`; v6 `073ff48b4438b3b7e15726548b8198418c90ce88a401f81291b1e33530d9addf`; v7 `ff15edf6426b9af12031e4668af0173776d110a08b909b842b5044991c37dfdb`; v9 `d7da473dfa0338d11d43c58bcd3e891dd9869e70d7aa6afa080e1dd4351c0857`.
- `agms-probe all > $AGMS_RUN/evidence/preflight-executor.json; echo rc=0` completed at 19:11:24Z with rc=0 and top-level `"ok": true`.
- Coordinator supplied `preflight-summary.json` has `launcher_ok`, `tmux_child_ok`, `pty_input_echoed` and top-level `ok` all true, `failed: []`, `probe_count: 99`. Both `preflight-launcher.json` and `preflight-tmux-child.json` have top-level `"ok": true`.

## Case MOD-22

### Starting conditions

- Requested: fresh product profile and fresh `ledger-notes` fixture; baseline at Permissive before choosing Moderate; then Moderate (restarted). The group note requires MOD-07 step 1 before MOD-07 steps 2–3 for the pre-quit portion; no other MOD-07 step.
- Profile PR-4 (19:11:59Z): `ls -A "$AGMS_RUN/product-home"` printed only `config.toml`; `cmp /opt/agms/etc/product-config.toml "$AGMS_RUN/product-home/config.toml" && echo SAME` printed `SAME`.
- Workspace initial `git status --short` was empty (19:11:59Z).
- Fixture state before: recorded as `initial` by `agms-tui fixture` at 19:12:02Z below.
- Timebase: UTC recorded with `date -u` at each key transition.

### Actions and observations

<!-- live entries appended during execution -->

### Expected-result outcomes

- In progress.

### Verdict

- In progress.
