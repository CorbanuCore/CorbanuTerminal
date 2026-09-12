# Exact candidates and shortcut

Both packages report **0.1.41**. Version alone does not identify a test build.

| Platform | CLI SHA-256 | Exact executable |
|---|---|---|
| Mac arm64 | `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e` | `/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/macos-candidate-final9-20260910/bin/corbanu` |
| Linux x86_64 | `bc2200d67d273e8555d1824b9cff9f14390276de92c915685f96015525279dc7` | `/home/travis/security-round5/evidence/pf58-final9-20260910/candidate-v2/bin/codex` |

The source is worktree `provider-reauth-health`, branch `feat/provider-reauth-health`,
base HEAD `4f6743b9c`, plus the closeout source hashes in `source-files.sha256`.
Incoming main `295aed26e53b17f919f7199ae1c9748b1b1250ba` changes only planning/docs
and is reconciled into this branch. The latest remote-main check returned that
same SHA. Runtime source was frozen before the final builds; later changes affect
test selectors, evidence, documentation and merge bookkeeping only.

Mac build: Cargo release, `corbanu`, Code Mode host, wallet daemon and ACP binaries;
Developer ID signing and strict verification recorded in `mac/stage.log`.
Linux build: Cargo debug, `codex`, Code Mode host and wallet daemon. The build mirror
is `/home/travis/security-round5/picker-repair-20260910`; checksum comparison against
local `codex-rs/` found no runtime source differences. A test-only formatting
difference was synchronized and its test repeated.

Complete file/link inventories: `mac/package-inventory.json` (13 entries) and
`linux/package-inventory.json` (47 entries). These include unchanged package
resources, not merely the executables. Build caches and packages remain on the
external Mac drive / designated Linux disk.

The inherited Linux package also contains older `matrix/`, `tools-tmux/` and
launcher/package bookkeeping files. Their inventory hashes are not current test
results or current build provenance. Use this packet's `linux/` receipts and
source manifest for the replacement; do not infer qualification from the copied
historical reports inside the package.

## Installed Applications path

`/Applications/Corbanu Terminal Launcher.app` reads the stable executable path
`/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/bin/corbanu` and the
existing `activate.sh`. All four stable binary links now target the Mac package
above. The CLI and Code Mode hashes were verified through those stable links.
The application bundle, credential store and existing user terminal processes
were not changed. New launches use the replacement; running sessions remain old.

Rollback is available: all four previous links targeted
`/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/macos-candidate-picker-final-20260910/bin/`.
That package remains intact. Native Applications-path confirmation is a separate
user check, not inferred from signature validation or direct TMUX launches.
