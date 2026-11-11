# Contributor integration checkpoint — November 11, 2026

> Historical 06:28 UTC snapshot. See the [10:26 UTC integration update](contributorIntegration20261111-1026.md) for subsequent merges and remaining gates, and [tracker #2106](https://github.com/CorbanuCore/CorbanuTerminal/issues/2106) for later changes.

Status at **06:28 UTC**. This is an implementation/qualification handoff, not a release or completed-feature announcement. Travis accepted the comparative assessment of 19 human and eight Dependabot PRs and requested revisions, incorporation and named thanks. [Tracking issue #2106](https://github.com/CorbanuCore/CorbanuTerminal/issues/2106) records current outcomes and review-budget grants.

**No contributor PR has merged in this round.** All 19 human contributors' PR threads received named acknowledgments and their specific dispositions. Original author commits were retained through normal merge/fast-forward updates; no force pushes. Only #28 has been closed, as superseded, not merged. No binary was installed or released.

## Merge gate and first dependency

Main was last verified at `152f62bf6e305d74c4deab00013520bb75d3636e`; sync target at `29909c645e0d921dc34c4af73857e5388331f321`.

[Audit repair #2105](https://github.com/CorbanuCore/CorbanuTerminal/pull/2105), exact head `2cfd034860f792d0c3605b40437a3e21e29b361e`, has independent approval and passing local cargo-deny/Bazel checks. Two Linux V8 jobs still run; the other two completed successfully. Main-target prepared branches include this repair. Merge it first only after every check finishes green/skipped; never bypass pending CI.

The earlier owner-reported provider/model-picker repair [#2103](https://github.com/CorbanuCore/CorbanuTerminal/pull/2103) is also reviewed/tested but awaiting CI. Its candidate is not installed. Do not infer a new release or installation from these integration records.

## Prepared and independently reviewed heads

Heads below were pushed; their PRs remain open and subject to current checks, exact-head security approval verification and combined-tree qualification. Passing static review is not execution, human acceptance or release qualification.

| PR                                                                        | Contributor                              | Prepared head | Evidence / scope                                                                                      |
| ------------------------------------------------------------------------- | ---------------------------------------- | ------------- | ----------------------------------------------------------------------------------------------------- |
| [#1380](https://github.com/CorbanuCore/CorbanuTerminal/pull/1380)         | citadelculture                           | `c1be76afc5`  | Corrected Join-Path/GetFullPath guidance; textual prefix is not containment proof.                    |
| [#1370](https://github.com/CorbanuCore/CorbanuTerminal/pull/1370)         | citadelculture                           | `967155b6ae`  | Linux Bash 5.2: six prerequisite and eight changed-path tests pass. Mac Bash 3.2 correctly rejected.  |
| [#1385](https://github.com/CorbanuCore/CorbanuTerminal/pull/1385)         | citadelculture                           | `8f581f16ed`  | Temporary-directory ownership; 44 instruction-file tests pass.                                        |
| [#1392](https://github.com/CorbanuCore/CorbanuTerminal/pull/1392)         | citadelculture                           | `e3747beddb`  | Persisted cooldown controls; 15 tests pass, one leak flag followed by a clean isolated replay.        |
| [#2073](https://github.com/CorbanuCore/CorbanuTerminal/pull/2073)         | local-maxi; shared citadelculture credit | `79b29c951d`  | Production-path polling and ownership-aware regressions; native CI remains a gate.                    |
| [#1383](https://github.com/CorbanuCore/CorbanuTerminal/pull/1383)         | citadelculture                           | `8b2bcf8147`  | Task-local cancellation scope coverage; do not restore obsolete sync hooks for this test.             |
| [#1384](https://github.com/CorbanuCore/CorbanuTerminal/pull/1384)         | citadelculture                           | `6bc1405a3f`  | Pending-record cardinality; named Linux execution still needs confirmation.                           |
| [#1393](https://github.com/CorbanuCore/CorbanuTerminal/pull/1393)         | citadelculture                           | `25c81be5b2`  | Enables Windows fixture regression; native evidence remains required.                                 |
| [#1348](https://github.com/CorbanuCore/CorbanuTerminal/pull/1348)         | citadelculture                           | `73dc9a4307`  | Ambiguous colon-space parsing restricted to home listing; 60 protected-read tests pass.               |
| [#1246](https://github.com/CorbanuCore/CorbanuTerminal/pull/1246)         | citadelculture                           | `792f7161cb`  | Cached statx identity retains newer mount-scan safety ordering; native Linux evidence pending.        |
| [#135](https://github.com/CorbanuCore/CorbanuTerminal/pull/135)           | citadelculture                           | `dad50ef2b9`  | Grok remains manually selectable but allocation-disabled; 80 catalog tests pass.                      |
| [#983](https://github.com/CorbanuCore/CorbanuTerminal/pull/983)           | citadelculture                           | `35bca47fb5`  | Serialized omitted-tool notice with enqueue acknowledgment; all 11 focused tests pass.                |
| [#1394](https://github.com/CorbanuCore/CorbanuTerminal/pull/1394)         | citadelculture                           | `870044b8f3`  | **Stage 1 only:** 176-line parser/test stage, eight tests pass. Workflow/wrapper untouched.           |
| [#1055](https://github.com/CorbanuCore/CorbanuTerminal/pull/1055)         | citadelculture                           | `0188ed5752`  | Current-depth membership cache; all 402 classifier tests pass after instrumentation repair.           |
| [#1962](https://github.com/CorbanuCore/CorbanuTerminal/pull/1962)         | Dependabot                               | `2d80585e6f`  | Optional-helper panic repaired; all 92 adapter/apply-patch tests pass.                                |
| [#1960](https://github.com/CorbanuCore/CorbanuTerminal/pull/1960)         | Dependabot                               | `71f14dfcd0`  | Minimal socket2 update with aligned locks; 20 socket/escalation tests pass.                           |
| [#1961](https://github.com/CorbanuCore/CorbanuTerminal/pull/1961)         | Dependabot                               | `a8989da5ad`  | Aligned Axum locks retain WebSocket forks; 168 focused macOS transport/TCP/exec/code-mode tests pass. |
| [#1956](https://github.com/CorbanuCore/CorbanuTerminal/pull/1956)         | Dependabot                               | `949554a0b5`  | Cache/save pin; actionlint passes.                                                                    |
| [#1959](https://github.com/CorbanuCore/CorbanuTerminal/pull/1959) + #1957 | Dependabot                               | `b68d420cf8`  | Both CodeQL commits preserved; actionlint passes, actual CodeQL runtime qualification outstanding.    |

### Failures and corrections retained

- **#1962:** independent review found assert_cmd 2.2's missing-helper panic bypassed the adapter's fallible behavior. The coordinator authorized one repair review; the revised adapter preserves explicit Cargo/Bazel lookup and fallible legacy fallback. Do not reset historical review usage.
- **#1055:** the first adapted classifier run passed 401/402 and flagged one leak; its new loop omitted main's required test-only work counter. The maintainer added `work::step()` and authorized one narrow delta review. Fresh final execution passed 402/402 without a leak; the previous leak case also passed alone. Depth 32, skipped-substitution handling and all three readings remain unchanged.
- **#1055 measurement:** 2,000 depth-32 membership repetitions took 2.286 seconds uncached versus 25.57 milliseconds cached in a debug microbenchmark. This includes registration, but preallocates and includes the final outer prefix. It is **not** an exact production-close replay or an end-to-end parser/product latency claim.
- **#1961:** the first transport run passed 150/152; two synthetic Unix-socket paths exceeded macOS `SUN_LEN`. An unchanged-head fresh run with a private short temporary parent passed 152/152, retaining the isolation wrapper and native-keyring denial. Seven actual app-server TCP/auth cases, three exec-server cases and six code-mode WebSocket cases also passed. [#2108](https://github.com/CorbanuCore/CorbanuTerminal/issues/2108) tracks the default fixture-path reliability gap. No failure log was overwritten or relabeled a pass.

## Sync contributor #2101 — not qualified

[bradsbots' MCP port](https://github.com/CorbanuCore/CorbanuTerminal/pull/2101) is revised at `6f605e4d20a371cde25d827ce74bca64bd25ae7a`. Pending startup compares effective discovery budgets; ready clients remain reusable. Three new regressions pass; exact-head code review is positive.

Broader isolated MCP/RMCP testing ran 696 cases: 687 passed, nine failed, nine additional cases skipped. All nine failures reproduced on unchanged sync: helper/remote prerequisites, native-OAuth fixture/guard failures and an HTTP-401 result-shape mismatch. They are not waived. CI also reports the three OpenSSL advisories and four repository-check inventory/grouping failures. [The sync tracker](https://github.com/CorbanuCore/CorbanuTerminal/issues/1532#issuecomment-6788616303) records the blockers separately from the contributor patch.

S00 archival used its explicit November 7 exit record, not a new qualification claim. S01's narrow MCP allocation is reconciled; a nonblocking portfolio-summary wording update remains. No installed-sync or milestone sign-off is claimed.

## Remaining work and dispositions

- **#1837:** reconcile chroot classification with newer finder/sudo and brace protections under the appropriate active security sprint; qualify before incorporation.
- **#1394:** bounded file ingestion, BEP binding and expanded Windows target-matrix activation remain. Parser stage 1 is not the finished evidence collector.
- **#88:** split already-landed installer work; repair timeout restoration, metadata-before-error retry and overbroad entitlement classification without losing current named-account/Keychain boundaries.
- **#28:** closed as superseded with detailed reasons and named thanks to **goodalexander**. Existing cache-write and exact-decimal accounting replace its broad bundle; its cumulative merger also double-counts and its wire schema is stale. The useful reasoning/thinking decoder is preserved as [#2107](https://github.com/CorbanuCore/CorbanuTerminal/issues/2107), not implemented or merged yet.
- **#2076:** close only after #2073 lands, explaining the selected stronger coverage and retaining citadelculture's shared credit.
- **#1957:** close only after the paired #1959 integration; retain both original commits.
- **#1955:** qualify the release-action major upgrade without publishing a release merely to test it.
- **#1958:** hold the inherited WinGet publisher, which still targets OpenAI.Codex/openai-oss-forks, until intentionally adapted or removed.
- Merge independent, fully qualified PRs one at a time with exact-head guards. Refresh/retest overlapping files and combined dependency locks; a green old-base run does not qualify an untested composition.

## Operational handoff

The requested Opus 5.5 High execution provider returned a long cooldown. The coordinator implemented directly; no replacement execution-worker model was selected. Independent reviews requested the owner-approved Astra High runtime. No new owner decision is needed for this scoped queue.

Local evidence and the exact-head/review ledger live in the coordinator's `contributor-integration-20261111.md`, `contributor-integration-*-reviews.md` and named test logs under `.codex-work/`. Preserve failed attempts and replay distinctions. The persistent goal tool reports **blocked**, not active; do not claim an automatic background continuation or change goal status merely to pause/resume it.

The newer contributor instructions do not approve unrelated outstanding program decisions. Keep the broader security/sync release gates and existing owner reservations. Final merge/closure comments must name contributors and give factual Task Node provenance; do not invent task IDs, payment claims or completed integration.
