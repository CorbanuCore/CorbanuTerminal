# Corbanu Terminal 0.1.42

The requesting repository operator instructed a new release on 2026-09-10 to ship the fixes previously available in `corbanu-debug`. Codex is executing the authorized release. Classification: release of the user-authorized catalog integration, bounded reliability repairs, and completed Team Context initiative PF-76. No trading behavior is introduced.

Worktree: `/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-wallet-launch-hotfix`; release branch `release/corbanu-0.1.42`. Implementation head `401ddcc206`, based on public 0.1.41 (`739897fd64`), with the documentation-only main updates through `6ca801add7` merged before packaging. The canonical dirty checkout is preserved.

Included changes:

- `9677aef4e2`: packaged wallet daemon discovery and production-client startup probe.
- `9a9922c626`: Campaign Tracker recovery and provider-tab filtering.
- `91995c339c`: Corbanu API Flash catalog integration.
- `c6b48e6b36`: Flash capability correction and released-attempt recovery.
- `401ddcc206`: review follow-up, retry-status/body regressions and valid incident JSON.

Product specification **Shipping MVP — LIVE**, **Multi-provider inference**, **Task Node and identity**, and **Named profiles** cover the existing workflows. **Corbanu API — TO BUILD**: “The customer-facing model catalog uses Corbanu identities and displays Corbanu prices”; “Reservations, settlement, idempotency, and insufficient-balance enforcement remain atomic and server-authoritative.” The new Flash catalog entry was explicitly requested by the operator; its supporting gateway is already deployed as release 35.

Implementation evidence: [Flash review follow-up](../../reliability/2026-09-10-flash-review-followup.md), [Flash recovery](../../reliability/2026-09-10-deepseek-flash-recovery.md), and the wallet/Campaign Tracker repair records in `qa/reliability/`. Most recent verification: 256 focused Rust tests, 91 gateway tests and 16 isolated PostgreSQL tests passed. A real keyboard-driven terminal run passed preparation failure → retry → real tool use → final response → restart, with no unsupported parallel control. Synthetic billing matched actual upstream receipts.

The final versioned tests passed: all 189 `codex-api` library tests, including the 80-case released-attempt matrix, plus 7 selected core/catalog tests. An exploratory `codex-client --lib` invocation selected zero tests and is not counted as a pass; the retry boundary lives in `codex-api` and was tested there.

The versioned Linux debug build reports `corbanu 0.1.42`. Its packaged production wallet-client probe passed (`wallet-client-package-ok`). Both default repositories passed keyboard-driven TUI checks on the versioned binary: picker cancel/select, a confirmed released 503 retry with a fresh ID, actual `pwd` execution in each disposable worktree, final response and selection persistence after restart. These release checks use synthetic inference and protocol failures, supplementing the separately recorded live gateway test. Repository paths and base commits are in [pty-results.json](pty-results.json); neither test worktree changed.

Formatting, installer contract tests (5), portable skill checks (25 files), workflow YAML validation and Bazel lock refresh passed; `MODULE.bazel.lock` is unchanged. The version bump changed 148 workspace package versions and no external dependency entries. The sprint checker still reports pre-existing plan-link and duplicate-ID errors from the current documentation tree; [its output](sprint-check.txt) is retained without claiming a pass.

Release qualification and artifacts are being collected under `/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-0.1.42-qa`. The cross-platform workflow builds all five platform archives, both macOS installers and checksum manifests, and automatically publishes only after its packaging checks pass. Publication is not yet claimed.

The full competitive and coding-model benchmark cycle is incomplete; the counter is not reset. Cross-platform interactive human acceptance is not available. These missing artifacts are disclosed under the operator's release instruction, consistent with the root release policy. The benchmark disclosure is [here](benchmarks/README.md).

The first publish run (34542400957) compiled the ARM Linux application but its separately selected wallet probe failed because selecting only the daemon omitted Core's platform-specific vendored OpenSSL feature. The probe build now selects Core alongside the daemon on each platform, preserving the package build's TLS feature configuration without changing application code or dependencies. `cargo tree --target aarch64-unknown-linux-musl -p codex-core -p codex-wallet-daemon -e features -i openssl-sys` confirms the vendored feature. The release workflow is rerun after the same package-selection command is verified locally.


On 2026-09-11 the operator explicitly requested the release run after Team Context qualification. The replacement release is assembled in `/mnt/HC_Volume_101713660/pfrpc/scratch/corbanu-0.1.42-release-recovery`, branch `release/corbanu-0.1.42`, merging Team Context head `30436889d5` with the wallet probe build correction. Application and embedded-skill sources are byte-identical to that tested debug head; only release workflow and evidence differ. All five platform packages will be rebuilt from the replacement commit, without artifact reuse.

Included Team Context work: `721d6f974d`, `fa9094117c`, and `30436889d5`, linked to completed [PF-76-S01](../../../docs/sprints/archive/p0-security-levels/pf-76-s01-team-context.md). Product authority: **Shipping MVP — LIVE / Task Node and identity**, “Tasks, evidence, verification, rewards, balances, chat, context, linked identity”. This adds the existing permission-filtered shared report to `/tasknode team`, the CLI, and embedded agent guidance. The backend is already deployed as Task Node web release 737. [Team Context evidence](../../reliability/team-context-2026-09-11/README.md) records 24 focused Rust tests, backend database tests and final keyboard workflows in both default repositories, including helper resolution against an older installed release. Synthetic reports were used; a live end-user collaborator read and model-driven conversation are not claimed. Named-human feature acceptance remains unrecorded.

The corrected wallet probe build passed locally in 14.43 seconds. The ARM feature-tree check confirms vendored OpenSSL; cross-platform execution remains the responsibility of the replacement workflow. Existing benchmark and sprint-check disclosures above still apply.
