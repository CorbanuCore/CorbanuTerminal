# PF-27-S01 — shared security integration contracts

Date: 2026-08-27. Status: **completed and archived**. This accepts PF-27's shared
contracts, not downstream native consumers or a product release.

Product initiative under **P0 `/security` levels**: “Permissive preserves the
shipping behavior and does not silently change existing policies.” Plan:
`docs/plans/active/p0-security-levels.md`; accountable owner: Jim Ricketts.

The [initial slice](initial-slice.md) preserves baseline ancestry, dependency
commits, early tests, and its limited review. The full consumer handoff is in
[consumer-contracts.md](consumer-contracts.md); synthetic native-adapter fixture
definitions are in [adapter-conformance.json](adapter-conformance.json).

Worktree: `/Users/travisgood/Documents/ChatGPT/corbanu-pf27-s01`.
Branch: `codex/pf-27-shared-security-contracts`.
Base: `ea7d4bec720098f6e0994fcfcc59e272108f7e70`.
Final code candidate: `faa8ed6d39bf30db1b2fe982a69661a108e00a71`.
Package version: 0.1.35. Subsequent commits contain only evidence/navigation.
All work remains local; no push or release was requested or performed.

## Delivered and upstream disposition

- Versioned inspector facts separate requested/effective levels and independent
  browser, content-firewall, confidentiality, and protected-action health.
  Unimplemented controls report unavailable, including at stronger levels.
- Host-issued source envelopes bind exact content bytes; bounded sticky taint
  covers every declared source kind and fails closed on unknown ancestry.
- Immutable actions and grant checks bind the native identity/context and live
  authority epoch. Core adds a fresh runtime nonce to its existing policy state;
  old confirmations fail after restart even when persisted counters match.
- Dedicated protocol intents and observational TUI state cannot authenticate
  themselves as human or mutate policy. The existing trusted Core controller
  owns non-serializable confirmation capabilities; consumers must still perform
  the atomic final check/mutation under its authority guard.
- Shared module registrations reserve disjoint consumer files. No provider tool
  schema, native agent scheduler, persisted format, browser backend, or live UI
  command is added. The only new dependency edge is protocol → existing policy.
- Seven synthetic conformance definitions cover dispatch, provider wire,
  provenance, lineage, post-read/revocation, cancel/resume, and independent health.
  Every contract-test reference resolves to a passing JUnit case; actual native
  consumer assertions remain pending, never silently counted as a pass.

Upstream remains `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa`, incorporated by
`45a60f03d2f6c041d284b41cc3f33c416d9eeed1`. Verified ancestry and exact dependency
commits for PF-17/19/20/22 are preserved in [initial-slice.md](initial-slice.md).
Retain the product-owned contracts and module registrations across upgrades;
adapt only thin native hooks and the existing Core authority snapshot. This is
not an upstream upgrade or integrated upstream qualification.

Implementation stages: `6ad6a299a` inspector/allocation, `5cfc9e021` provenance,
`21ff1f139` action/epoch binding, `53cbe757c` protocol, `7094947a0` Core,
`efe04f08a` browser module reservation, `a3fab3a71` consumer/review handoff,
and `faa8ed6d3` TUI state. Complex code stages stay below 500 changed lines;
the inspector stage also carries allocation/historical evidence (649 total).

## Final-tree verification

Environment: macOS 15.6.1 (24G90), arm64; Rust/Cargo 1.95.0 from `codex-rs`,
cargo-nextest 0.9.143, just 1.58.0. All six Rust commands below were rerun after
the final code commit. The 34 changed source/manifest fingerprints are recorded
in [code-sha256.txt](code-sha256.txt) and match the committed candidate.

Before the final tests, from `codex-rs`:

```sh
just fix -p codex-security-policy -p codex-protocol -p codex-core -p codex-tui -p codex-network-proxy --profile dev-small
just fmt
just fmt-check
```

All exited 0. New panic-based adapter assertions were replaced with fail-closed
errors before final fixes/tests; remaining warnings are pre-existing dead-code
and unrelated TUI-test lint warnings. `just bazel-lock-update` from the repository
root passed; no `MODULE.bazel.lock` change was generated. No new external package
or browser installation was required.

Each command below runs from `codex-rs` with **`--cargo-profile ci-test`** appended.
JUnit reports were copied from `target/nextest/local/junit.xml` after each run.

| Command | Result | Final artifact / run ID |
| --- | --- | --- |
| `just test -p codex-security-policy` | 39 passed, 0 failed | [policy-junit.xml](policy-junit.xml); `3e41557a-c5cc-42a1-a30f-042aae31d7b2` |
| `just test -p codex-protocol` | 281 passed, 0 failed | [protocol-junit.xml](protocol-junit.xml); `a2395918-ae8e-48ac-9576-b0e2133119fd` |
| `just test -p codex-network-proxy --retries 0` | 214 passed, 0 failed, retries disabled | [network-proxy-junit.xml](network-proxy-junit.xml); `6db3f7d5-704a-41c8-8d3f-ac30b2b67e9b` |
| `just test -p codex-core --lib security::` | 26 passed; 2,306 outside filter | [core-security-junit.xml](core-security-junit.xml); `16668f72-6b7c-4a83-9d4b-f44ddaace6fe` |
| `just test -p codex-core --lib security_inheritance` | 3 passed; 2,329 outside filter | [core-inheritance-junit.xml](core-inheritance-junit.xml); `15cb1132-34ca-4b9f-b29a-5629051a4355` |
| `just test -p codex-tui --lib security::` | 2 passed; 3,810 outside filter | [tui-security-junit.xml](tui-security-junit.xml); `0596e24d-2259-40cb-b56a-7762934baa9f` |

There are 32 new contract/adapter tests across policy, protocol, Core and TUI.
The inherited Core selection includes the actual native child-creation path and
existing credential-capability/canary tests. The two Core selections overlap in
two tests; their counts must not be presented as distinct coverage. JUnit omits
filtered-out cases, so its zero skipped count is not a full-Core/TUI pass.

Supporting checks: plan/sprint validators, 19 sprint-validator unit tests,
fixture JSON/reference validation, source fingerprint validation, and
`git diff --check` pass. Closure leaves 24 current and 85 archived sprints.
Earlier development failures were a malformed JSON test fixture and a test
assuming native `Op` is deserializable; both were corrected before final runs.
The initial inspector serde issue and its regression are preserved in the
historical record. No failed run is relabeled a pass.

The first commit-bound network run (`8237a24d-2e23-4cd4-ac8f-6230f611d64f`)
passed with one configured retry in the unchanged DNS-sensitive
`host_blocked_subdomain_wildcards_exclude_apex` test: it first observed
`NotAllowedLocal` instead of `NotAllowed`. The existing local-address guard also
returns that reason on DNS failure/timeout; the precise transient trigger was
not captured. [The retry report](network-proxy-retry-junit.xml) preserves the
failed attempt. The entire suite was then rerun at the same candidate with
`--retries 0`: all 214 passed. No unrelated runtime/test fix was made.

## Review scope freeze

The user requested “please finish PF-27.” Review the full local diff from the
base above, including new files. Owners: existing policy crate, dedicated
protocol security module, Core security boundary, observational TUI state, and
comment-only module reservations. No browser, firewall, enforcement, provider,
transport, installer, persisted-schema or live UI consumer is implemented.
Seven new implementation modules total approximately 889 non-test Rust lines,
plus small registrations and Core epoch binding; tests are sibling modules.
Stage local commits by inspector, provenance, action binding, protocol, Core and
UI/registration boundaries. Scope expansion for unrelated findings is forbidden.

The initial review artifacts are historical, not a full-sprint review. Full
Autoreview used the default Codex `gpt-5.5` engine and exited 0 with no findings:
[text](completion-review.txt), [structured result](completion-review.json).
Command: `python3 /Users/travisgood/.codex/skills/autoreview/scripts/autoreview
--mode local`, with the scope prompt above and outputs in this directory. The
helper lacks executable permission, so Python invoked it without changing
permissions or substituting engines. No nested review was run. The only code
edit after bundling was correcting a numeric argument's parameter-name comment;
no executable logic changed, and final formatting/tests include that correction.

## Non-applicability and next gates

No actual-key TUI or TensorCash/Isometric workflow applies: this sprint exposes
no user interaction and activates no runtime consumer. Those proofs are required
in the downstream interactive/integration sprints and again for release. This
record does not claim full Core/workspace execution, Linux/Windows qualification,
browser containment, active injection prevention, or a reconnect transport fix.
Human acceptance, independent security acceptance, integrated final-tree proof,
finished user docs, and any due benchmark remain release gates; no release is
proposed. Unfinished behavior stays in plans/sprints, not shipped feature docs.

PF-13's isolated worktree was verified clean at
`ea7d4bec720098f6e0994fcfcc59e272108f7e70`; its Windows and other qualification
requirements are unchanged. PF-26-S01 is the next harness workstream. PF-24-S01
is dependency-eligible too. PF-30-S01 may run alongside PF-29 after the harness
dependency completes and disjoint allocations pass validation. None was activated
by closing PF-27.
