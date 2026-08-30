# Claude Opus 5.0 Max review — PF-13 integrated repair

## Review identity

- Reviewed commit: `be8153f2e29c360d83776441aed50deb204eafa7`
- Parent: `09dbd3bbc8688574dd4c1350149e162b5d4f3216`
- Review scope: `claude-integrated-repair-review-scope.md`
- Harness: TMUX + Corbanu Terminal `0.1.35`
- Runtime: `claude-opus-5-plan`, provider `claude-plan`, reasoning effort `max`
- Sandbox/approval posture: read-only / never
- Raw external pane capture: `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/tmux-artifacts/pf13-opus-max-review/pane.txt`
- Raw capture SHA-256: `22a0745e7eef4a01e3d0a40ea12bb88f28c1958c6e09178d71139be2efc935ce`

The provider stream disconnected once while the review was running. Corbanu
Terminal retried the same Claude Opus 5.0 Max runtime and the review completed;
no fallback model was used. The reviewer also detected that the original scope
record had the correct abbreviated parent but an invalid expanded SHA. The
scope record now carries the actual parent shown above; its recorded
format-patch SHA-256 remains unchanged.

## Findings and dispositions

1. **P2 — accepted: Bash credential-name filtering depended on `tr`.**
   `bash_snapshot_script` sources `.bashrc` before filtering exports, so a
   shadowed or unavailable external `tr` could make the sensitive-name check
   fail open and persist a secret-bearing export. The repair uses Bash 3.2's
   builtin `nocasematch` with `[[ ... =~ ... ]]` and adds a regression whose
   `.bashrc` shadows `tr`.
2. **P2 — accepted: MCP reviewer test observed a scheduling flag, not the
   published runtime.** The test now awaits `refresh_mcp_if_dirty` and compares
   the approvals reviewer in both session configuration and the published MCP
   runtime binding.
3. **P3 — rejected after reproduction: restore `agent_message` wire matchers in
   the plaintext multi-agent resume fixture.** Adding the proposed matchers
   caused the exact test to time out twice. Core retains the typed
   `AgentMessage` in durable history, but `client.rs` deliberately adapts
   plaintext agent messages for providers that do not support the native item.
   This fixture selects provider id `mock`, so requiring an outbound
   `agent_message` contradicts the provider-adaptation contract. The existing
   role-model and content assertions remain the correct wire-level proof.
4. **P3 — accepted: direct regression coverage for the two lifecycle
   repairs.** Added exact policy-snapshot equality across repeated root binding,
   positive inheritance for an unbound auxiliary agent, and a twice-repeated
   shutdown interrupt proving truthful `Shutdown` status, no admitted
   inter-agent audit message, and preserved metadata.

The non-blocking compaction-comment observation is outside the credential and
lifecycle repair and requires no change.

## Focused repair qualification

After `just fmt`, the focused nextest run
`a9cc52f9-21ca-4cf1-8a44-773a8a7d0889` passed all six selected tests with no
retry:

- Bash secret filtering, including the shadowed-`tr` regression
- deterministic MCP reviewer publication
- root binding preservation and auxiliary inheritance
- idempotent shutdown interrupt identity preservation
- cold multi-agent resume on the provider-adapted plaintext path

The first attempted run is retained as diagnostic evidence only: the five
accepted-finding tests passed, while the over-broad P3 matcher timed out on both
attempts. No production failure was observed.

## Closeout gate

The first repair re-review used the same TMUX + Corbanu Terminal + Claude Opus
5.0 Max harness against immutable commit `ba17fa0da`. Its external pane capture
is
`/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-foundation-platform/tmux-artifacts/pf13-opus-max-rereview/pane.txt`,
SHA-256
`37f2f69a29011f133d63d9955569858a38156f207da1db9bc8c22c8f254e856a`.
It found two actionable issues, both accepted:

1. The MCP test's explicit refresh could perform the production work whose
   ordering the test must verify. The test now first proves the user-turn path
   cleared the pending state, then observes the published binding without
   mutating it.
2. A `.bashrc` function could shadow `shopt` just as it could shadow `tr`.
   Instead of relying on either command lookup or mutable shell options, Rust
   now expands the exclusion expression into an ASCII case-insensitive regex
   before the script runs. The regression shadows both helpers.

Focused nextest run `627c3d10-e746-4885-a25b-829cfa58abc9` passed all five
accepted-finding tests without retry after the second repair. The repair still
requires a clean independent re-review through the same mandated harness. The
sprint must not be archived until that re-review and the affected final
qualification are clean.
