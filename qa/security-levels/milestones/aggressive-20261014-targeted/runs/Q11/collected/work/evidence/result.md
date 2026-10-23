# Q11 execution log

Executor role: independent functional executor; not human acceptance.

## Candidate and preflight

- Candidate requested: `/opt/agms/pkg/bin/corbanu`, Corbanu Terminal 0.1.48.
- SHA-256 observed at 2026-10-14 10:12 UTC: `dc61bd6403b99a892b3db4ed59fef22f30a4ff65f1cd2a9ee165e994c2a50483` (matches required).
- Frozen cases SHA-256 observed and matches `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`.
- Amendments/new cases SHA-256 observed and matches `072a8c3fc4910e819fbcf569fd359bfff7521f7b02587f2e272936d311da7523`.
- Coordinator preflight-summary and both preflight files were present. The required self-run `agms-probe all` completed rc=0 at 2026-10-14T10:13:04Z and reported `"ok": true`. Evidence: `preflight-executor.json`, `preflight-launcher.json`, `preflight-tmux-child.json`, `preflight-summary.json`.

## Case AGG-R14 (advisory)

Starting condition setup: in progress.
