# Candidate (Moderate targeted re-run)

The brief: install the current signed build from `INSTALL.json` (integration-build-20260922a). At the freeze it was:

| Field | Value |
| --- | --- |
| Source | `main` at `4ef3be846789765e149c75a0cbe9af7c6bec6726` (merge of #894), built 2026-10-16T06:48:15Z ([INSTALL.json](INSTALL.json)) |
| Fixes in it | #813 (#739), #849 (#847), #850 (#845), #851 (#822), #854 (#843), #869 (#838), #870 (#846), #877 (#840, MOD-28 wording), #881 (#839), #883 (#842), #892 (#841), #827 (#766) |
| Not in it | #900 (#873 follow-ups to #851's refusal wording: neutral header, one `└ Refused: <reason>` line, refused cells for unified-exec refusals), merged after this build. The re-run judges only that a refusal nobody was asked about is not labelled "Approval declined" (amendment v3 rule R-2) |
| Version | 0.1.48, macOS arm64 |
| Entry point | `bin/corbanu`, SHA-256 `cb42614c8cc194bf8fc5d598e61e49883384a6747051d92884d5c41f905b0d25`, Developer ID signed (Travis Good, K5N2T25C23), developer build (`developer_marker: true`) |
| Package | the first run's candidate-v2 package with `bin/corbanu` replaced by the build above; the helpers (`codex-code-mode-host`, `corbanu-acp`, `corbanu-walletd`, `codex-path/rg`, resources) are candidate v2's (`df3d8c5d1c`, 2026-10-14). Every file, mode and digest: [package-files.txt](package-files.txt) |
