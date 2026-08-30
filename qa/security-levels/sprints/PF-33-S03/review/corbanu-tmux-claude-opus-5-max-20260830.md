# PF-33-S03 post-archive independent review — 2026-08-30

## Runtime attestation

- Terminal: actual rebased `target/debug/corbanu` TUI inside tmux session `pf-browser-opus5-main-20260830` (`terminal.type=tmux/3.7c`).
- Conversation: `01a051a8-4a83-7383-b383-b9724878e674`.
- Rollout: `/Users/Neo/.corbanu/sessions/2026/08/30/rollout-2026-08-30T00-52-58-01a051a8-4a83-7383-b383-b9724878e674.jsonl`.
- Trace: `/Volumes/CorbanuDrive/Corbanu/.codex-work/p0-security-browser-retrieval/tmux-opus5-main-20260830/logs-final/codex-tui.log`.
- Provider/model/effort: `Claude Plan` / requested `claude-opus-5-plan` / `max`; the provider response reported `claude-opus-5`. The trace records both identities, `approval_policy=never`, and `sandbox_policy=read-only`.
- Initial combined-review prompt SHA-256: `126df8815ecbe0c0fa8839fb23a987b581f191f993b5732fbca9deee447922e8`.
- The rendered initial response accidentally cited the prior conversation ending `01a051a7`; the submission, response events and runtime trace for this evaluation are the `01a051a8` artifacts above.

## Initial verdict

`CHANGES REQUIRED` after approximately 13 minutes of read-only local inspection.

PF-33-S03 findings:

1. **P1 — Bazel compile inputs undeclared.** The integration test read `../src/destination_contract.rs` and a repository-root QA fixture at compile time, while `BUILD.bazel` declared neither input. The generated Bazel test would fail even though Cargo passed.
2. **P3 — Public scope boundary permissive by construction.** `Option<Vec<RuleSpec>>` used `None` for intentional unrestricted public scope, indistinguishable at the contract boundary from a loader's absent or degraded configuration.
3. **P3 — Security-sensitive outputs forgeable.** Public fields let consumers directly construct an unnormalized `NormalizedDestination` or fabricate a `DestinationDecision`.

The combined review also found the PF-31-S04 runtime-image pin drift gap. It was resolved and independently re-reviewed to `CLEAN` before this sprint was reopened; its full record is in `qa/security-levels/sprints/PF-31-S04/review/corbanu-tmux-claude-opus-5-max-20260830.md`.

## Dispositions

- **Bazel input finding:** closed by declaring `src/destination_contract.rs` through `integration_compile_data_extra`, keeping the executable fixture under `network-proxy/tests/`, freezing its hash, and retaining a byte-identical QA evidence copy. Focused Bazel target: `PASS` (1/1).
- **Public-scope finding:** closed with explicit, non-defaultable `PublicScope::{Unrestricted, Rules}` at the `PolicySpec` boundary. `Rules([])` remains explicit deny-all; each fixture case now names the scope.
- **Forgeability finding:** closed by private fields and read-only accessors on both `NormalizedDestination` and `DestinationDecision`; construction remains internal to normalization/evaluation.

## Corrected-candidate verdict

First follow-up prompt SHA-256: `a63a3fe0f27f38c370b20b5fb9dc0480dbd79ab31c53aefcb3ed98682f018a01`.

Verdict after 5 minutes 26 seconds: `CHANGES REQUIRED`. The reviewer independently confirmed all three substantive PF-33 findings closed, recomputed the three current hashes, verified byte equality and Bazel target shape, and found one remaining P3 evidence contradiction: the checked sprint ledger still said the source was absent from “manifests” after `BUILD.bazel` deliberately declared it as test compile data.

Disposition: corrected the checked sprint item to state that the pure source remains absent from `src/lib.rs`, the module tree and runtime graph, while the Bazel integration target declares it only as test compile data.

Final follow-up prompt SHA-256: `2b18ff503a613ade2adb882b358c01303adf3e244470fedc2026b3700eea7687`.

The same tmux/Corbanu Terminal/Claude Opus 5 Max conversation recomputed the source/test/fixture hashes, confirmed that the repair touched only the ledger and evidence, and verified the full working-tree consistency. It marked the ledger contradiction and all three original PF-33 findings closed, reported no actionable P0-P3 finding, and ended with the standalone verdict `CLEAN` after 1 minute 13 seconds.

The integration owner separately completed the writable final-tree gates. The bounded remediation commit is `80a2469e401066ebaf04d95ba603ab68cb341854`.
