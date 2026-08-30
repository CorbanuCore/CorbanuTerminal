# PF-33-S03 evidence

## Candidate contract

- Contract: `pf33-destination-policy/v1`.
- Pure source: `codex-rs/network-proxy/src/destination_contract.rs`.
- Executable table/property coverage: `codex-rs/network-proxy/tests/destination_contract.rs`.
- Frozen representative fixture: `contract-v1.json` (SHA-256 `38cdf3cbb68de22597221089c49d3e9595a3e82dccfeda5b809077f5993a8a31`).
- Reviewed source candidate: `e965c522f2eff367586aa03c70426c2cd0a26282`.
- Source SHA-256: `1422f8f2725eb90c4c543fdbb09741f975809b0793c4bb1ae2d0b200d3991aa4`.
- Test SHA-256: `061c94bc56dbe08eecc2c22c42f0ce8da6de4f619e6a8bb9e93ada065d541729`.
- Runtime registration: none. Direct source/scope audit confirms that the module opens no socket and is deliberately absent from `src/lib.rs` and all Cargo/Bazel/runtime registries; the literal-string test is only a regression smoke check.

The contract normalizes bounded HTTP(S) scheme/host/port/method/path inputs; keeps `None`, explicit empty, wildcard-public and exact private-service policy polarities distinct; evaluates complete synthetic DNS answer sets; and re-evaluates redirect targets with downgrade, origin credential and body/method replay checks. Reserved names, private/special addresses, IPv4-mapped IPv6, translation/tunnel IPv6, mixed answers and unapproved private identities fail closed.

## Isolated verification

Run from `codex-rs` in the allocated PF-33 worktree, in the required order:

```text
just fix -p codex-network-proxy
just fmt
just test -p codex-network-proxy
```

Result on macOS arm64 on 2026-08-30 before final review: fix and formatting passed; nextest ran 239 tests across three binaries, 239 passed, zero skipped. Sixteen tests are the standalone destination-contract table/property and executable-fixture suite. `git diff --check` passed.

Integration merge `1b07aef5d1a22aedd6c12140e36beaf89c0eede1` received the reviewed source unchanged. On canonical `main`, the integration owner reran the same required sequence: clippy fix and formatting passed without edits; nextest ran 239 tests across three binaries, 239 passed, zero skipped (one existing test carried nextest's leaky annotation). Both governance checkers and `git diff --check` passed on the completed archive tree.

## Security boundary and handoff

This sprint makes no SSRF-prevention or live-route claim. PF-33-S01/S02 still own real resolver acquisition, complete-answer enforcement, connect-to-approved-address behavior, connected-peer verification, redirect/retry re-resolution and chain limits, pool partitioning/expiry, inherited proxy and `NO_PROXY` resistance, operator-specific translation prefixes, raw socket/UDP/QUIC and alternate-egress controls, and IPC/isolation. Consumer code must use this versioned contract without a permissive fallback. The receiving owner audited the literal scope, archived PF-33-S03 and returned the parallel slot.

The product requirement advanced is **Reconciled security scope — TO BUILD**: “Unknown or unsupported protected paths fail visibly rather than falling back to raw secrets or unscreened execution.”

## Review

- [Codex GPT-5.5 Autoreview](review/codex-gpt-5.5.md): review iterations drove closure of private-service DNS-rebinding/pin-bypass and malformed-host alias defects; final full-candidate verdict is clean at confidence 0.86.
- [Claude Opus 5 Max Computer Use](review/claude-opus-5-max.md): initial `CHANGES REQUIRED` verdict, evidence and integration-owner dispositions retained; corrected-candidate follow-up verdict is `PASS`.
