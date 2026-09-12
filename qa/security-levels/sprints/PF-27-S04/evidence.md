# PF-27-S04 candidate evidence

## Scope and disposition

- Owner: `/root/pf27_isolated_broker`
- Branch: `feat/p0-security-isolated-broker`
- Allocation commit: `7fca549f731d95e7c8a63a93cd2aae6daa6fb6b3`
- Implementation candidate: `75bce53ef`
- Opus remediation candidate: `2197f833a`
- Disposition: scoped implementation candidate. It does **not** claim
  protected-mode eligibility or completion. Shared registration, qualified
  OS-service activation, and measured Linux, macOS, and Windows qualification
  remain integration gates.

## Implemented leaf contract

- Authenticated, length-bounded HMAC-SHA256 frames bind controller, worker,
  session, task, run, generation, sequence, and an OS-observed peer.
- The only operation is the PF-13 exact `POST https://api.openai.com:443/v1/*`
  adapter with visible-ASCII origin-form paths. Opaque credential references
  are canonical SHA-256 identifiers; there is no generic resolve-to-string API.
- A PF-27-S03 `ProtectedModeAuthorization` is mandatory for construction.
  Invalid or unavailable platform evidence returns typed unavailable, and the
  broker binary refuses ordinary same-user launch with exit code 78.
- Run replacement, session cancellation, credential revocation, concurrent
  upload cancellation, broker restart, wrong peers, cross-run theft, replay,
  forged references, expiry, bounded retained run generations, and resource
  exhaustion fail closed.
- Dispatch reserves PF-41 durable intent before backend execution and commits a
  terminal outcome afterward. The intent includes the exact path. Ambiguous
  terminal commit invalidates the session.
- The Core client creates authenticated monotonically sequenced frames and
  closes an ambiguously delivered channel instead of rolling back or advancing
  its sequence. The Core config adapter binds the complete capability authority
  to the network route.
- The network-proxy isolated route replaces raw OpenAI environment material
  with an opaque dummy, refuses header injection, and sends exact-host typed
  dispatches through the broker for fresh calls. Plaintext denial and MITM
  authorization-hook collision predicates cover both scoped and isolated routes.
- The Vault backend resolves raw material only inside the constrained broker's
  typed transport callback, after exact authorized method, destination, and
  path comparison; opaque-reference lookup is bounded and exact.

## Verification

All Cargo, target, Bazel, temporary, trace, and log paths were rooted under
`/Volumes/CorbanuDrive/Corbanu/.codex-work/pf27-isolated-broker/`.

| Check | Result |
| --- | --- |
| final `cargo test --manifest-path codex-rs/secret-broker/Cargo.toml` | 30/30 passed after formatting |
| final focused secret-broker `pf_27_s01` filter | 14/14 passed |
| `cargo clippy --manifest-path codex-rs/secret-broker/Cargo.toml --all-targets -- -D warnings` | passed |
| Bazel `//codex-rs/secret-broker:secret-broker-unit-tests` | passed, 30 tests |
| focused network-proxy `credential_broker` | 14/14 passed |
| focused Vault `capability` | 5/5 passed |
| full `codex-security-audit` | 44/44 passed (2 pre-existing leaky annotations) |
| full `codex-network-proxy` | 239/239 passed |
| full `codex-vault` | 33/33 passed (10 slow, 1 pre-existing leaky annotation) |
| focused Core broker client/config under temporary registration | 4/4 passed, including close-on-drop |
| focused network-proxy isolated route under temporary registration | 1/1 passed |
| focused Vault broker backend under temporary registration | 1/1 passed |
| full network-proxy with isolated adapter | 240/240 passed (2 pre-existing leaky annotations) |
| full Vault with broker backend | 34/34 passed |
| affected secret-broker/network-proxy/Vault clippy (`--no-deps -D warnings`) | passed |
| Core clippy (`--no-deps -D warnings`) | PF-27 leaves clean; command stops on inherited `core/src/session/output_text_stream.rs:33` `uninlined_format_args`, unchanged from allocation commit |
| platform probe self-test | 8/8 contract regressions passed |
| platform probe Python tests | 7/7 passed |
| plan/sprint governance tests | 23/23 passed; both checkers passed |
| `just fix -p codex-secret-broker && just fmt` | passed |
| standalone unqualified broker launch | exited 78 with typed qualified-service-unavailable message |
| shared registration and lock restoration | no diff from allocation commit for the six integration-owned paths |
| remediation secret-broker full suite | 31/31 passed |
| remediation Core client focus under temporary registration | 4/4 passed |
| remediation network-proxy full suite | 240/240 passed |
| remediation Vault full suite | 35/35 passed |
| remediation Vault authority focus | 2/2 passed; wrong method, destination, and path reached no transport call |
| remediation secret/network/Vault Clippy (`--all-targets --no-deps -D warnings`) | passed |
| remediation Bazel secret-broker unit target under the documented temporary lock registration | passed, 1 target |

The adapter tests used only the exact temporary registrations documented in
`integration-handoff.md`. Those changes and `codex-rs/Cargo.lock` were restored
before handback. The Core clippy diagnostic is outside this sprint's scope;
`core/src/session/output_text_stream.rs` has no diff from allocation commit
`7fca549f731d95e7c8a63a93cd2aae6daa6fb6b3`.

## Human-observable and independent review evidence

- Supporting TMUX smoke: passed in session `pf27-isolated-smoke-2` using the
  CorbanuDrive build of Corbanu Terminal v0.1.35, `read-only` sandbox,
  `never` approval policy, `claude-opus-5-plan`, and `max` effort. `/status`
  reported the expected worktree, Claude Plan provider, connected account,
  read-only/never permissions, and session
  `01a059c0-263e-7f53-9e5a-6a9a2f0efebb`.
  - pane capture SHA-256:
    `8a9a0c82e1616d962d1d9e305cf77bc23dc917772c0c971df2f1b6e141514169`
  - trace SHA-256:
    `35c4b1af3f6f28c2b7bcb6c6b486eff999ae44903fd4261436ef8a690e04ebb9`
- Required Claude Opus 5 Max read-only review was attempted through Corbanu
  Terminal in TMUX session `pf27-isolated-opus5-review`, with the same
  read-only/never constraints and exact model/effort. The provider failed
  closed before inference because current Claude Code credentials were absent
  from both `/Users/Neo/.claude/.credentials.json` and the macOS Keychain.
  No review verdict exists and no clean-review claim is made. The live TMUX
  session is preserved for continuation after authentication.
  - prompt SHA-256:
    `8b32361cb4ad4e2bf8989fd0794ccd8435e19310b4ca7da8f2734a851fcc08b9`
  - auth-blocked pane SHA-256:
    `82a7209ed15b2e5f996aa7e4bae6d8d96dafbab82c156fc2e51e9bf2a57fed0a`
  - review trace SHA-256:
    `602a7af9065ec0ffc4472ad2c080ccdce8e61d1753e3945ed0a70bb45848b45d`
- A later token-authenticated read-only review completed in Corbanu Terminal
  through TMUX session `pf27-opus-long-token` on socket
  `pf27-long-token-review`, against exact range `7fca549f7..c80768e0b`.
  The footer reported Claude Opus 5 Plan at Max. The verdict had no P0, one P1,
  five P2 findings, and one related P3 predicate; an exact-pattern token-leak
  scan of the captures was false.
  - prompt:
    `/Volumes/CorbanuDrive/Corbanu/.codex-work/claude-auth-review-runtime/pf27-review-prompt.md`
  - prompt SHA-256:
    `1dbf6d4f94414f25c55fc201779862d3da1e846ed9f0b4a0ae4d9e96ed808eee`
  - transcript (29,575 bytes):
    `/Volumes/CorbanuDrive/Corbanu/.codex-work/claude-auth-review-runtime/pf27-opus5-max-review.txt`
  - transcript SHA-256:
    `121a2b3bad65c7533051d1ce10dde6195cb225ed9ac14cbb9581de2271642df0`

## Review remediation disposition

All actionable findings were independently confirmed and fixed in
`2197f833a`:

1. **P1 Vault authority binding:** the backend now compares the capability's
   exact method, HTTPS destination host/port, and path before Vault resolution.
2. **P2 audit completeness:** durable intent now includes the validated exact
   path.
3. **P2 retained-run bound:** generation tombstones are fail-closed bounded at
   eight per configured session slot and are never evicted into stale reuse.
4. **P2 path grammar:** controls, whitespace, non-ASCII, and backslashes are
   rejected before authentication/transport.
5. **P2 ambiguous sequence:** the client advances only after a receipt and
   closes on any transport result whose consumption status is ambiguous; it
   never blindly retries a possibly consumed sequence.
6. **P2 plaintext-route parity and related P3 MITM collision:** the existing
   runtime predicates now recognize either protected OpenAI route.

The fresh post-remediation review ran read-only in the same authenticated
Corbanu Terminal/TMUX session against `c80768e0b..2197f833a`. Claude Opus 5
Plan at Max independently rechecked every remediation and reported exactly
`NO ACTIONABLE P0/P1/P2 FINDINGS`. Its two non-blocking observations were an
out-of-contract transport panic remaining fail-closed and deliberate channel
teardown after indistinguishable in-band/transport errors. A token-pattern
scan of both post-remediation artifacts found no matches.

- prompt:
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/pf27-isolated-broker/review/opus5-max-post-remediation-prompt.md`
- prompt SHA-256:
  `d86732776d5f3b766bd14cedab36f866160fc1308e57034a6a2d0afdcb0fc1d0`
- transcript (43,661 bytes):
  `/Volumes/CorbanuDrive/Corbanu/.codex-work/pf27-isolated-broker/review/opus5-max-post-remediation-transcript.txt`
- transcript SHA-256:
  `101a645e661426a52a4acf852f6ba45919f8b173391d97b467724cc15a02fce4`

## Remaining all-OS and integration gates

1. Integration owner applies the serialized registration/activation handoff in
   `integration-handoff.md` after rebasing over PF-22-S02.
2. Re-run focused Core broker-client tests and all affected suites after the
   serialized integration, then run a combined-tree Opus 5 Max review.
3. Qualify actual Linux dedicated-UID/service, macOS launchd/XPC helper, and
   Windows service SID/AppContainer launches. Linux/Windows testing is deferred
   until the user confirms the tailnet switch.
4. Re-run PF-26 final-candidate and both-live-repository qualification before
   any release-complete claim.
