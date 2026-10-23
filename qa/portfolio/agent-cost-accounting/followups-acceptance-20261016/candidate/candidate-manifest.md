# Candidate

- Source: `origin/main` `395ac0adcd` (Merge #861), including PF-60-S06 (#793), S07 (#812), S08 (#821), S09 (#825) and the
  Moonshot zero-count fix #859. Not included: #882 (#824, realtime), unmerged when the design was frozen.
- Build: the coordinator's signed developer build (`dev-build.sh`, `codex-core/developer-accounting` and
  `codex-tui/developer-accounting`), installed 2026-10-16T03:08:51Z ([INSTALL.json](INSTALL.json)), at this worker's
  request (`pf60-s10.request.md` / `.grant.md`), because the previously installed build (`c7de27594e`) lacked #859.
- `bin/corbanu`: 0.1.48, SHA-256 `5b805b9c244e55c12c6f61fc3d31bddd760a415554cbabff8cbcb4c3ba0531a1`, Developer ID
  Application: Travis Good (K5N2T25C23), hardened runtime ([codesign.txt](codesign.txt)); developer-accounting marker
  present ([developer-marker-count.txt](developer-marker-count.txt)).
- Staged package (read-only `/opt/agms/pkg` in the guest): `bin/corbanu`, `bin/codex-code-mode-host` (needed by the image
  tool), `codex-resources/`, `codex-package.json` ([package-files.txt](package-files.txt)); packet digests
  [packet-files.txt](packet-files.txt); [build-head.txt](build-head.txt).
