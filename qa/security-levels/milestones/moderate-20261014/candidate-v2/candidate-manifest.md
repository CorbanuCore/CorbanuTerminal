# Candidate manifest v2 (Moderate, first run): the build that was executed

Owner decision 1 (go file, 2026-10-14, recorded on #750): run on a build rebuilt from current `origin/main` (option b).
The design was frozen against candidate v1 ([../candidate/candidate-manifest.md](../candidate/candidate-manifest.md));
its design screens come from v1, a named limit of this record.

| Field | Value |
| --- | --- |
| Source | `origin/main` at `df3d8c5d1c521228a8dbb1035831af572afe1284` (merge of #806), clean detached worktree |
| Changes since v1 (`899cf3e62a`) | #797 (#787: keys typed before an approval was shown never answer it), #798 (#789: no Grant once for waits and printing `sed`, also on child-agent prompts), #799 (#792: the inspector's Vault row before the restart), #806 (proxy expiry), #795 (rustls 0.23.46), and test/CI/docs-only merges #793, #794, #800, #801, #803, #804, #805 |
| Version | 0.1.48 |
| Build | same script and flags as v1 ([build.sh](build.sh)), same target directory; finished 2026-10-14 23:31Z ([build-head.txt](build-head.txt)). A first attempt was stopped when its launching shell closed, before compiling anything |
| Platform | macOS arm64 (`aarch64-apple-darwin`) |
| Entry point | `bin/corbanu`, SHA-256 `abd3e6adbf44f234e12fb6a9654d15a7f6b7b6c20ecd61af2ef8e1944ffbb24a` |
| Package | every file, mode and digest in [package-files.txt](package-files.txt); archive SHA-256 `14fc35fa2803ba964e9d5b1a7a40dc86acb307ff53110dfa25dc2d4941da11c9` (kept outside the repository) |
| Signing | ad-hoc linker signature only; not Developer ID signed or notarized |
| Executor packet | v2 ([../executor-packet/v2/](../executor-packet/v2/)): v1 with only the expected candidate SHA-256 changed in each group prompt ([../common/packet-v2-diff.txt](../common/packet-v2-diff.txt)); cases, amendments, navigation and screens are the frozen v1 files |
