# Exact staged packages — 10 September repair

Version labels are both 0.1.41; hashes distinguish these from earlier candidates.
Built from the PF-58 worktree at `4f6743b9c` plus this repair's uncommitted source
changes. Post-build changes are test/qualification documentation only.

| Platform | Executable | SHA-256 |
|---|---|---|
| Mac arm64 | `/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/macos-candidate-blind-repairs-20260910/bin/corbanu` | `865731588b5fd0447b0d7f60ad04ca4a544a8ab39a8ae48eb2764f37c90abb4d` |
| Mac helper | `bin/codex-code-mode-host` | `fbe01eb7679b95d8a1b207de72d1bd29d6a93cf1cc61a88a7496f6c58e1a7422` |
| Linux x86_64 | `/home/travis/security-round5/evidence/blind-repairs-20260910/candidate/bin/codex` | `aef2b6d258bd31645660f3d701abd7e9460d246da38e08b15c086e2ea1069872` |
| Linux helper | `bin/codex-code-mode-host` | `8a541f36059090b84d26c13476e360d85ccdb1dc0bf39b5a68b2203668007ae8` |

Matching wallet helpers and package resources were staged; Mac also includes ACP.
Linux executable hash list (not a full resource inventory): `linux/candidate.sha256`.
Mac signing/build receipts: `mac/mac-stage.log`
and `mac/mac-build.log`. Mac signatures were strictly verified. Signing is not
proof of native consent behavior, notarization, or human acceptance.

The Applications shortcut is **not updated**; existing user windows are untouched.
These packages are qualification candidates, not a release or unqualified human
handoff. Diagnostic binaries used while investigating the test fixture were never
copied into either package.
