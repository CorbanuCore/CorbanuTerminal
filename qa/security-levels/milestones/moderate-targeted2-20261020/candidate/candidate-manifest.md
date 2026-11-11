# Candidate (Moderate second targeted re-run)

The brief: the installed signed build main `8bf182de43` (`INSTALL.json`, integration-build-20260922a), which has #900
and the #941 fix (#949). At the freeze it was:

| Field | Value |
| --- | --- |
| Source | `main` at `8bf182de43f412e1571858ad4b5d747f5420461a` (merge of #871), built 2026-10-20T03:20:00Z ([INSTALL.json](INSTALL.json)) |
| Fixes in it (checked as ancestors of the source commit) | #900 (refused cells for unified-exec refusals, `f715f9455a`), #949 (#941: the approvals-off refusal names the level and how to proceed, `1482a45526`), #962 (#942: no session option for MCP tools that need fresh approval after untrusted content, `a875e4cca2`) |
| Version | 0.1.48, macOS arm64 |
| Entry point | `bin/corbanu`, SHA-256 `95ee54cb7ce1bcd3566da613cb415e8677227cb5e11baac63d5a1bbfecf9e509`, Developer ID signed (Travis Good, K5N2T25C23; `codesign --verify --strict` ok), developer build (`developer_marker: true`) |
| Package | the previous re-run's package (the first run's candidate-v2) with `bin/corbanu` replaced by the build above; the helpers (`codex-code-mode-host`, `corbanu-acp`, `corbanu-walletd`, `codex-path/rg`, resources) are candidate v2's (`df3d8c5d1c`, 2026-10-14). Every file, mode and digest: [package-files.txt](package-files.txt) |
