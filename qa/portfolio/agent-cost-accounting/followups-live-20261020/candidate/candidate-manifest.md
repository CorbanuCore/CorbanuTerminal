# Candidate

- Source: `origin/main` `78dd811d6e`, which includes the #898 fix (#917, `/review` cost), #924 (#889 item 1) and the
  other decided fixes listed in [INSTALL.json](INSTALL.json) (`what_is_new`), and the realtime recording from #882 (#824).
- Build: the coordinator's signed developer build (`codex-core/developer-accounting` and `codex-tui/developer-accounting`),
  built 2026-10-19T18:45:00Z ([INSTALL.json](INSTALL.json)); staged unchanged for the guest at 02:02:11Z
  ([build-head.txt](build-head.txt)).
- `bin/corbanu`: 0.1.48, SHA-256 `e72003067f9316f2805b4344041d111409611c6d0730fe268c46bf95225bdaa3`, team
  K5N2T25C23, hardened runtime ([codesign.txt](codesign.txt); identity "Developer ID Application: Travis Good" per
  INSTALL.json); developer-accounting marker
  present ([developer-marker-count.txt](developer-marker-count.txt)).
- Staged package (read-only `/opt/agms/pkg` in the guest, copied in over SSH): [package-files.txt](package-files.txt);
  executor packet digests [packet-files.txt](packet-files.txt) and [packet-build.txt](packet-build.txt).
