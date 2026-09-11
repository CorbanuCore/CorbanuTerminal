# Mac Keychain repeated-prompt repair — 10 September 2026

**Superseded candidate:** the user subsequently reported two prompts and a
broken model catalog on the 16:55 package below. The
[picker/health repair](picker-repair-20260910.md) now identifies the installed
signed replacement, passing Mac/Linux package tests and two unattended
existing-profile Mac menu launches. The historical results below are preserved,
not silently upgraded to human acceptance.

PF-58-S01 follow-up, authorized by the user after the native 0.1.41 candidate
repeatedly requested access to `Codex Auth`. Product: **Shipping MVP — LIVE**,
“Encrypted `/vault`, masked entry, metadata-only inspection, and operational
credential use without placing raw values in chat.” Worktree:
`/Volumes/CorbanuDrive/Corbanu/worktrees/provider-reauth-health`, branch
`feat/provider-reauth-health`, starting HEAD `e8e9b92ae012045358d7ae0916ed092869bb80ad`
plus the pre-existing, preserved September 8 repair changes.

## Failure and repair

**Human acceptance failed at 16:47 on September 10.** The user reported at
least four more prompts after entering the password and selecting Always Allow.
The running image was the intended ad-hoc-signed replacement, not an old build.
Metadata-only logs identified the vault item (`codex` service, `secrets|…`
account): one load succeeded, subsequent same-process loads failed, then a later
load succeeded. The generic native error does not prove an incorrect password.
The automated checks below did not establish a reliable real-account experience.

### Developer ID follow-up (16:55)

With explicit user approval, staged a separate copy and signed Corbanu and its
three helpers with the existing **Developer ID Application: Travis Good
(K5N2T25C23)** certificate. This is bounded local packaging/validation within
PF-58, not a release or a change to credential permissions. No private key was
exported and no Keychain ACL was edited. Vendor resources and source code were
unchanged. The old package and running user session were preserved.

Current package:
`/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/macos-candidate-developer-id-20260910`.
Version 0.1.41; CLI SHA-256:
`b897e1e5e963b0a4e43876a1b53e03c12133c629e732119bb23df9cd12db9131`.
Stable identifiers are `com.corbanu.corbanu`,
`com.corbanu.codex-code-mode-host`, `com.corbanu.corbanu-walletd`, and
`com.corbanu.corbanu-acp`. All four signatures pass strict verification, have
Apple timestamps and team-based designated requirements rather than ad-hoc
build hashes. Preserve these identifiers/team when signing future candidates.
No hardened-runtime flags or entitlements were introduced; notarization and
Gatekeeper distribution qualification are not claimed.

Signing recipe (existing identity only; never export its key):
`codesign --force --sign DB43D92853B5EF2BCE87DA9B2F28360DC44BBEBD --identifier com.corbanu.<binary-name> --timestamp <staged-binary>`.
Sign copies before changing the launcher, then verify and rerun package QA.

The signed final package passed the true-TMUX smoke with Ctrl+C: four actual
Code Mode/shell/MCP round trips, three concurrent processes, same-home restart,
stream cancellation/recovery, inert permissions/security cancellation, and
40-column Claude guidance. Artifacts are
`evidence/macos-keychain-20260910/tmux-developer-id/result.json`,
`developer-id-signatures.txt`, and `developer-id-package.sha256` in the same
evidence directory. The Applications launcher's four links now target this
signed package. Existing windows continue to run the older image.

**Real-account acceptance is still required for this signed candidate.** Stable
signing addresses a build-identity problem, but the exact cause of the user's
four dialogs has not been conclusively established. The first transition may
require approval for existing items. Do not claim the prompt loop fixed until
the user tests startup, provider use, and reopening the same signed package.
The Escape baseline failure and other human/live-repository gates remain open.

### Earlier prompt-budget implementation and evidence

The previous native keyring wrapper serialized operations and bounded hangs,
but every completed call permitted another native permission dialog. Denial,
cancel, and one-time Allow could therefore be followed by another prompt.
Ad-hoc build signing may require fresh permission after replacement, but is not
proven to be the cause of this user's repeated prompts.

The Mac wrapper now consumes one interactive-operation allowance per exact
service/account pair per process. Later loads, saves and deletes consult the
actual Keychain with optional native UI suppressed. No credential value is
cached, so updates/deletions are observed. Failures explain that repeated
prompts are suppressed and instruct the user to unlock/restart and choose
Always Allow if they trust the build. Restart deliberately resets the budget.
Separate credentials can each require their own initial permission.

The native process-level interaction flag is restored by an RAII guard while
the existing serialized worker permit is still held, including panic and
late-worker completion. The code never forces interaction on, changes an ACL,
unlocks a keychain, or adds a storage fallback. Existing fallback behavior is
unchanged. Linux/Windows retain their original native behavior.

The `keyring` dependency uses the legacy file-based Keychain API. The scoped
Security.framework interaction setting is also used for that limitation in
[Chromium's native implementation](https://chromium.googlesource.com/chromium/src/crypto/+/refs/heads/main/apple/scoped_keychain_user_interaction_allowed.cc).

## Verification

- `just fmt` completed; no dependency changes.
- `cargo clippy --locked -p codex-keyring-store --all-targets -- -D warnings` passed.
- `just test -p codex-keyring-store`: 8/8 passed (2 opt-in native tests excluded).
- `just test -p codex-keyring-store --run-ignored ignored-only`: 2/2 passed.
- Native freshness test: own synthetic item, 20 reads, replacement, fresh read,
  deletion, absence and idempotent deletion. No cached stale value returned.
- Native denial test: own synthetic item trusted only to `/usr/bin/security`,
  20 denied reads through the production UI-suppression wrapper; no repeated
  dialogs and no synthetic credential in the error. Both exact test items
  removed afterward. No real credential values accessed by these tests.
- Unit/native-state tests: 32 concurrent requests get one interactive permit;
  credentials isolated; restart state resets; errors and panic restore native
  UI state; existing timeout/serialization tests still pass.

Native release build completed in 9m 05s (not an interference-controlled
benchmark). Canonical package layout and strict signatures verified. All four
binaries rebuilt; pinned ripgrep/zsh and Telegram resources retained.

Package: `/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/macos-candidate-keychain-20260910`.
Version: 0.1.41; CLI SHA-256:
`573c871d1b738ea31a072da42b59b31c66bd27ade70fec5b0cf5fd91887b01ff`.
Source hashes, package hashes and native/unit logs are under
`evidence/macos-keychain-20260910/`.

Actual-package TMUX passed with **Ctrl+C** cancellation: four actual host/shell/MCP
round trips, three concurrent processes, same-home restart, stream recovery,
inert permission/security cancellation and 40-column Claude token guidance.
See `evidence/macos-keychain-20260910/tmux-final/result.json` (final post-format
rerun) and the earlier `tmux-ctrl-c/result.json`. Both passed on the same hash;
the eight focused and two native Keychain tests were also rerun after formatting.

**Escape remains a reproduced baseline failure**, not a passed check. The first
run queued a follow-up while still streaming. The harness now waits for actual
stream output and a positive `Conversation interrupted` event rather than an
old/missing footer. Both the replacement and untouched September 8 binary still
failed that Escape check. Preserve `tmux/`, `tmux-confirmed-cancel/` and
`tmux-old-baseline/` as failure evidence. The alternate cancel key is explicit
in the passing report; it does not convert the Escape-specific human check into
a pass. Check 11 and the coverage manifest now show this blocker.

This ad-hoc package has been superseded in the launcher by the Developer ID
copy above. Its real-user no-loop check failed; both packages are retained.
Do not treat these native API tests as a completed real-account GUI test.
No additional independent review, main merge or release is claimed.

`docs/sprints/check.py` has pre-existing unrelated PF-43/PF-44/PF-45 duplicate
IDs, missing feature links and duplicate execution-order findings. PF-58's
unchecked verification ledger was corrected; unrelated plan records were not
rewritten as part of this repair.
