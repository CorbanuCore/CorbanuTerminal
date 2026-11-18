# Extended current-main TUI preflight

These are harness preflight results, **not release-candidate qualification**.
No additional build, provider credential, model-server change, UAC action or
release workflow was needed. The final candidate requires fresh profiles and
new source/binary provenance.

## Navigation and approval controls

The macOS and Linux harnesses each pass 12 checks: `/status`, `/model`,
`/providers`, local-fixture `/docs`, the read-only `/security` view, slash list,
ordinary typing leaves a synthetic approval open, explicit cancellation,
first response, cancellation followed by recovery, two persisted completed
turns, and a persisted abort. The approval comes from the debug
`/test-approval` fixture; no real file edit is authorized by this case.

- macOS: [passing replay](preflight/extended-macos.json), source
  `5fd571822bef255237f963c7bc18286a9110f58b`, binary SHA-256
  `53f77726294ee6323aa565ae1400ce4cf53a0e15ab305be9bb10ca3bb812cb57`.
  An initial Escape-only dismissal left the approval open and blocked the
  following prompt. Its [failed result](preflight/extended-macos-attempt1.json)
  is retained. The corrected replay deliberately selects the cancellation
  option with arrows and confirms with Enter.
- Linux: [passing replay](preflight/extended-linux.json), source
  `d3fe32513ee65deb397fa228c786212fb5afacd3`, binary SHA-256
  `5f436c90c201f50040d339f7e0b68863307401163031fd40a73009a08e9535bd`.

macOS uses the existing outer Seatbelt denial of the operator home and
Keychain services. Linux uses a non-root, no-network Docker container with a
read-only root, capped tmpfs/memory/PIDs and only the worker fixture writable.
Mock responses do not qualify live provider behavior or tool sandboxing.
The later native Windows replay, with measured Medium integrity and protected
AppData/workspace ACLs, passes the same 12 checks plus native readiness (13).
Raw result: `$W/astra-rel149-windows-smoke-protected-acl-result.json`;
remote fixture: `C:\CorbanuQA\astra-rel149\acl-parent-smoke-medium-protected-acl\fixture`.

## Security choice, restart and recovery

[macOS](preflight/security-macos.json) and
[Linux](preflight/security-linux.json) each pass seven checks:

1. Moderate cannot be saved while `source_envelopes` is disabled.
2. With the prerequisite enabled, the picker saves Moderate.
3. Restart shows Moderate as active.
4. A deliberately malformed level in the private fixture enforces Aggressive.
5. The picker repairs the invalid state to Moderate.
6. Restart keeps the repaired Moderate choice.
7. The original config file remains unchanged.

This tests the UI and stored state, with no provider calls or agent commands.
It does not qualify credential brokering or sandbox enforcement. The original
macOS attempts are preserved: one [omitted the prerequisite](preflight/security-macos-attempt1.json),
and the next [expected the wrong saved-screen wording](preflight/security-macos-attempt2.json).
The corrected harness tests the prerequisite refusal explicitly and matches
`Saved: Moderate` before checking the stored value independently.

## Windows limitation

The [native Windows attempt](preflight/security-windows-unavailable.json)
measured Medium integrity (`S-1-16-8192`) and both administrator groups deny-only.
It verified that LocalAppData resolves inside the disposable OS profile.
The TUI refused Moderate because the selected non-admin Windows sandbox cannot
provide the required protected-file boundary; the UI requires the default
Windows sandbox and administrator setup. No level was saved by that action.

The full Windows security-state/restart/recovery case is **unavailable in this
lane**, not passed. No UAC or default-sandbox setup was attempted. This expected
prerequisite refusal is separate from the [#2584 tool-execution failure](WINDOWS-MEDIUM-FINDING.md),
which remains a release gate via PR #2586. The separate corrected-permission
[Medium live tool replay](preflight/live-windows-medium-profile-acl.json) passes
all four checks in 17.12 seconds; see that finding record for ACL evidence.

## Raw evidence and reuse

- macOS navigation: `.codex-work/astra-rel149-macos-extended-preflight{,2}/`.
- macOS security: `.codex-work/astra-rel149-security-preflight{,2,3}/`.
- Linux: `glitch:~/corbanu-rtx/astra-rel149/linux-extended-preflight/` and
  `linux-security-preflight/`.
- Windows: `C:\CorbanuQA\astra-rel149\security-medium-preflight\`;
  local capture `$W/astra-rel149-windows-security-failure.txt`.
- Parameterized replay scripts and preserved earlier versions:
  `$W/astra-rel149-final-*-smoke.py`, `astra-rel149-security-smoke.py`,
  `astra-rel149-linux-security-smoke.py`, `astra-rel149-windows-security-smoke.py`
  and `astra-rel149-final-medium.ps1`.

The final Windows Medium tool replay must pass after the accepted #2584 fix
merges. These additional preflights do not satisfy that gate.
