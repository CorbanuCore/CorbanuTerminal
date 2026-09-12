# PF13 human acceptance and main/release reconciliation

## Authority and scope

On September 11, 2026, Travis reported: “I've done the human testing on pf13.”
He authorized rebasing/reconciling main and the actual release line, merging to
main, and resuming this workstream. Earlier he explicitly confirmed live
messages and provider switching without Keychain prompts on the replacement.

This records human acceptance of the presented candidate, not a retroactive
pass for each frozen automated case. The Mac candidate was 0.1.41, SHA-256
`4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`.
The [original candidate manifest](../finish-20260910/candidate-manifest.md),
[nine-review evidence](../finish-20260910/dispositions.md), and
[Luna execution outcomes](../luna-yellow-20260911/report.html) remain intact.
No human checkbox is automatically selected. The previously reported desktop
placement failure is historical evidence, not independently disproved by the
general sign-off. Task Node wallet backups are not Solana `/wallet` recovery
material; that separate compatibility/UX observation is not changed here.

Change class: bounded integration of already-authorized, implemented work;
routine evidence and ledger reconciliation. Product heading **Shipping MVP —
LIVE**, **Multi-provider inference**, **Vault and credentials**, **Named profiles**:
“Encrypted `/vault`, masked entry, metadata-only inspection, and operational
credential use without placing raw values in chat.” Security linkage: **P0
`/security` levels**, “Existing approval, sandbox, vault, wallet, tool, network,
and agent policies are unchanged.” No new protected activation or release tag.

## Source reconciliation

- Accepted branch checkpoint: `f23381303`, backed up and pushed as
  `backup/pf13-human-accepted-20260911`; original runtime `eb7af932b`.
- Incoming main: `7bb0697cf8a5e88fc50bc232b65fea8d77f6da4a`.
- Latest published tag **rust-v0.1.42** and its actual branch
  **release/corbanu-0.1.42** both resolve to
  `5f3a0ad7d72ad14349daccd59592904e6d016427`.
- Integration checkout: `/Volumes/CorbanuDrive/Corbanu/worktrees/pf13-main-release-20260911`,
  branch `integrate/pf13-main-release-20260911`.
- Rebase used `--rebase-merges`. Recreating the historical `bf788d3d3` merge
  omitted its manual source/evidence amendments, so their exact accepted
  contents were restored for paths untouched by incoming main. The empty
  ancestry-only `eb7af932b` merge was skipped. At rebased `c19c328da`, the
  entire `codex-rs` tree was byte-identical to the accepted checkpoint.
- Merge `6dac5a88f` then includes the actual release ancestry: Team Context,
  wallet package startup/probe, provider tabs, Campaign Tracker recovery,
  Flash catalog/capability and released-attempt recovery fixes.
- Both model-catalog regression sets are retained with the new explicit
  current-provider argument. The first compile found one additional old
  two-argument test call; it was corrected before final tests.
- Release package-local canonical/legacy wallet-daemon resolution supersedes
  executable-brand guessing. The branch's canonical executable resolution and
  provider reauth behavior remain. The added protected-state workspace crate
  inherits 0.1.42; its lock entry was reconciled without external dependency changes.

## Governance and qualification limits

Main's three active workstreams remain: security, accounting, delivery control.
PF35 stays an external draft reservation; no classifier gate is bypassed.
PF58 implementation/human acceptance is preserved, while its unclosed automated
qualification remains draft under main's proposed provider plan, not a fourth
active initiative or a false completed archive.

Main already used PF-76-S01 for profile persistence, while the release reused
that ID for Team Context. The coordinator reserved PF82; the Team Context
archive is mechanically renamed PF-82-S01, preserving its released ID and
unchanged qualification evidence. No feature is added by this identity repair.

PF58's nine approved reviews remain exhausted. No tenth review was performed.
The separate broker service checkpoint `cd7457da7` is not part of this accepted
merge: its external service-stage review and native setup gates remain separate.
No accounts, ACLs, system services, real Vault data or user credentials are changed.

## Combined-tree verification

RTX source checkout: `/home/travis/worktrees/pf13-main-release-20260911`.
Evidence root: `/home/travis/security-round5/evidence/pf13-main-release-20260911`.
Build uses the existing shared lock and target, eight jobs and a fresh on-disk
temporary directory. Existing dirty remote checkouts and local installed Mac
candidate are untouched. Results and candidate hashes are appended after the
combined run; previous 0.1.41 acceptance is not new 0.1.42 acceptance.

Local checks so far: plan/sprint checkers pass (3 active plans, 115 current and
125 archived records); their 27 regression tests pass; portable skills match
all 25 files; five Luna-result collector tests pass. Secret scan of new text
evidence found only the explicit synthetic redaction-test string, not a live
credential. Tracked Python bytecode is absent.

Integration-harness repairs: the release-side picker regression now sets both
its Claude model and explicit current provider, instead of relying on the old
model-name inference. Preflight recognizes the released canonical wallet-daemon
name and the supported legacy fallback (11 gate tests pass). The Mac packaging
test incorrectly expected ELF `.debug` files from native Mach-O output and
deleted the temporary object needed by `dsymutil`; it now retains that object
and checks actual dSYM payloads on Mac. Alias assertions canonicalize both
sides. All 28 packaging tests pass locally using CorbanuDrive temporary storage.
These are test/fixture corrections, not changes to wallet access or model routing.

The initial RTX wallet run used an overlong temporary path that exceeded Unix
socket limits (8 failures). A short temporary root on the same physical RTX disk
produced 13/13 passes; the original failure log remains retained. No production
socket path or user home was changed to make the fixture pass.

The release's incomplete competitor benchmark and historical live/platform
limitations remain disclosed in `qa/release/0.1.42/RELEASE.md`. This merge does
not issue or claim a newly qualified cross-platform release.
