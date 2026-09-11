# Final native package — September 10, 17:52 Phoenix

Package: `/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/macos-candidate-picker-final-20260910`.
Version: `corbanu 0.1.41`. Source: [source identity](source-identity.md).
Built natively with the pinned toolchain and existing external-drive target;
all four release binaries were copied, stripped with `strip -S`, signed, and
strictly verified before qualification. No binary was substituted after testing.

| Binary | SHA-256 |
| --- | --- |
| corbanu | 8275923c4ee9c0bfc7e52109f742564c5a429631d765e15b6f0643b4cb7d3669 |
| codex-code-mode-host | dc9d05cff406d18f1f6720b777a776d74f949b49c6df9d8a7c0bd654187b1fcf |
| corbanu-walletd | 1fde40dd99cb84a466441449432fc34c20dc0a73f05d28fc670a4ad0fd0afec6 |
| corbanu-acp | 79194a502ee36c3792518c928fb1ece561de9308add644149b909e7ed5ff710e |

Signing authority: Developer ID Application: Travis Good (K5N2T25C23), followed
by Developer ID Certification Authority and Apple Root CA. Identifiers are
`com.corbanu.<binary-name>`, unchanged from the previous signed candidate.
CLI timestamp: September 10, 2026 at 17:52:20 Phoenix. CLI CDHash:
`5990fe3e94d7277479798f5854096aeddb7f0ddd`.
All four pass `codesign --verify --strict --verbose=2` and their designated
requirements. No Keychain ACL, hardened-runtime entitlement, private key,
notarization or credential-store migration was changed.

The previous `macos-candidate-developer-id-20260910` and intermediate packages
remain available. After final package tests passed, the four stable launcher
symlinks were switched to this package and the CLI hash/version rechecked through
the launcher path. Existing windows and the Applications app bundle are preserved.

## Existing-profile verification

`live-mac-delivery/result.json`: two launches, PIDs 63655 and 64038, reached the
provider and Claude model menus in 3.11 and 1.68 seconds. Each displayed all three
subscription models, exactly one current entry, and Anthropic in management.
The script entered no password, sent no model request, and verified unchanged
configuration bytes. Both test processes exited normally.

Metadata-only SQLite log inspection, restricted to these PIDs and
`codex_keyring_store`, found seven successful loads per process: three for
`Codex Auth` and four for `codex` vault metadata/operations. No failed load was
recorded. Account identifiers and credentials are intentionally omitted here.
This is evidence of unattended completion on the existing profile, not visual
certification of every possible native dialog or a live-provider request.
Separate credential items may require separate initial approval on a new
installation. The original user's two dialog identities remain unconfirmed.

Final synthetic package results and remaining acceptance gates are linked from
the [repair record](../../picker-repair-20260910.md). Human acceptance remains false.
