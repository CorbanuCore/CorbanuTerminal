# Review disposition (Opus 5.5 High)

The first pass on `7a5e9d93df` asked for changes. The fixes are in `ceb068042b`. A verification pass by the same reviewer (`verify.md`) returned "approve with fixes", and those fixes are in the commit that follows.

| # | Severity | Disposition |
| - | -------- | ----------- |
| 1 | High | Fixed. Without denied reads, the Sandbox row says approved or allow-listed commands can run outside it. Network then reads "off inside the sandbox, on for commands that run outside it", and Vault reads "readable". The tmux run confirms it: with network off, GLM's escalated `curl` returned 200. |
| 2 | High | Fixed. The row lists the real writable roots from the effective filesystem policy, or says full-disk, no sandbox or external sandbox. |
| 3 | Medium | Fixed. The kinds come from the probes plus every secret-like name in the real environment that survives the policy. Only names are kept. |
| 4 | Medium | Fixed. A role counts as changing children unless it sets only model or instruction keys. `sandbox_workspace_write` and `profile` were added to the Aggressive role check. |
| 5 | Medium | Fixed. The review scrolls with the move keys when it's taller than the pane, and the footer says "↑/↓ scroll". Covered by a snapshot test and the 80x24 tmux capture. |
| 6 | Low | Fixed. The wording is now "login profiles and shell snapshots", and `use_profile` is checked too. |
| 7 | Low | Fixed. Readable roots under `secrets/` count as readable. |
| 8 | Low | Fixed. A dim line says web search, environment, shell and roles are as loaded at launch. |
| 9 | Low | Fixed. No indents below 20 columns, and the values are computed only when the picker is on. |
| 10 | Low | Fixed. Tests for escape, extra roots, named variables, roles, permission requests and the whole Vault string. |
| V1 | Medium | Fixed. When permission-request tools are on and approvals are not `never`, Network and Vault say an approved request can widen them. Test added. |
| V2 | Low | Fixed. Up clamps the stored scroll first. |
| V3 | Low | Fixed; see 8. |
