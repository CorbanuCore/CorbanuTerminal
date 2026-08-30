# PF-33-S03 evidence

## Historical candidate contract

- Contract: `pf33-destination-policy/v1`.
- Pure source: `codex-rs/network-proxy/src/destination_contract.rs`.
- Executable table/property coverage: `codex-rs/network-proxy/tests/destination_contract.rs`.
- Frozen representative fixture: `contract-v1.json` (SHA-256 `38cdf3cbb68de22597221089c49d3e9595a3e82dccfeda5b809077f5993a8a31`).
- Reviewed source candidate: `e965c522f2eff367586aa03c70426c2cd0a26282`.
- Source SHA-256: `1422f8f2725eb90c4c543fdbb09741f975809b0793c4bb1ae2d0b200d3991aa4`.
- Test SHA-256: `061c94bc56dbe08eecc2c22c42f0ce8da6de4f619e6a8bb9e93ada065d541729`.
- Runtime registration: none. Direct source/scope audit confirmed that the module opened no socket and was absent from `src/lib.rs` and the runtime graph. The original candidate incorrectly treated absence from the Bazel test inputs as an isolation property; the 2026-08-30 remediation below corrects that build break.

The historical contract normalized bounded HTTP(S) scheme/host/port/method/path inputs; kept unrestricted, explicit empty, wildcard-public and exact private-service policy polarities distinct; evaluated complete synthetic DNS answer sets; and re-evaluated redirect targets with downgrade, origin credential and body/method replay checks. Reserved names, private/special addresses, IPv4-mapped IPv6, translation/tunnel IPv6, mixed answers and unapproved private identities failed closed.

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

## 2026-08-30 post-archive remediation

A trace-backed independent review in tmux + Corbanu Terminal + Claude Opus 5 Max reopened the archived sprint. The original source, test and fixture hashes above remain historical. The repaired candidate:

- replaces the ambiguous `Option<Vec<RuleSpec>>` boundary with non-defaultable `PublicScope::{Unrestricted, Rules}` and makes every fixture case name its public scope explicitly;
- makes `NormalizedDestination` and `DestinationDecision` fields private and exposes read-only accessors, so consumers cannot bypass normalization or forge a policy result;
- declares `src/destination_contract.rs` in `integration_compile_data_extra`, relocates the executable fixture copy under `tests/`, and freezes the package-local copy's SHA-256 in the test;
- keeps `qa/security-levels/sprints/PF-33-S03/contract-v1.json` byte-identical to `codex-rs/network-proxy/tests/contract-v1.json`.

Corrected-candidate identities independently corroborated during the final review:

- Source SHA-256: `38cfbea09fa9fd3231e277ee6d516ce00afd3f9d9e23d6bb74bb6f08e85a9f89`.
- Test SHA-256: `cf27a8b37de2b515e33675539623ada12cd407a86ef8077c705556a028d81608`.
- Both fixture copies SHA-256: `1b05284a2c173bb4436f9eae913e0d47cd2a11a6df4ee7e5d5b9e7fa93d2eb1a`.

Ordered verification on macOS arm64 after remediation:

```text
just fix -p codex-network-proxy                                  PASS
just fmt                                                         PASS
just test -p codex-network-proxy                                 PASS (239/239, 0 skipped; 16 contract tests)
bazel test //codex-rs/network-proxy:network-proxy-destination_contract-test
                                                                 PASS (1/1)
cmp qa/security-levels/sprints/PF-33-S03/contract-v1.json \
    codex-rs/network-proxy/tests/contract-v1.json                PASS
```

The Bazel target now deliberately compiles the pure source as test input; the source remains absent from `src/lib.rs` and the runtime graph. This still makes no live SSRF-prevention claim.

The bounded remediation commit is `80a2469e401066ebaf04d95ba603ab68cb341854`; the sprint was re-archived only after the ordered Rust gate, focused Bazel target, governance checks and same-session `CLEAN` verdict.

## Review

- [Codex GPT-5.5 Autoreview](review/codex-gpt-5.5.md): review iterations drove closure of private-service DNS-rebinding/pin-bypass and malformed-host alias defects; final full-candidate verdict is clean at confidence 0.86.
- [Claude Opus 5 Max Computer Use](review/claude-opus-5-max.md): initial `CHANGES REQUIRED` verdict, evidence and integration-owner dispositions retained; corrected-candidate follow-up verdict is `PASS`.
- [2026-08-30 tmux + Corbanu Terminal + Claude Opus 5 Max review](review/corbanu-tmux-claude-opus-5-max-20260830.md): trace-backed post-archive review, finding dispositions and corrected-candidate verdict.
