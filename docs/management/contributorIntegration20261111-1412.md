# Contributor integration — November 11, 14:12 UTC

**The approved main contributor queue is integrated. The security follow-up and larger program are not finished.** This checkpoint updates the [13:20 snapshot](contributorIntegration20261111-1320.md). See [tracker #2106](https://github.com/CorbanuCore/CorbanuTerminal/issues/2106) for subsequent results.

## What is integrated

The 13:20 snapshot records 32 explicit PR merges, plus #1957 incorporated through #1959 ancestry, through main `493f370ccf`. This includes provider Stage A, compatible reasoning/TPS work, Windows evidence tooling, classifier fixes, rustix and axum updates. Each passed its applicable pre-merge checks and exact-head independent review.

Named thanks and factual contribution credit remain on the contributor threads: **citadelculture**, **local-maxi**, **goodalexander** and **bradsbots**. No external Task Node reward or completion is claimed.

## New security follow-up

Finalizing the documentation exposed four new Hickory advisories in cargo-deny. [#2127](https://github.com/CorbanuCore/CorbanuTerminal/issues/2127) records the dependency chain and repair.

### Main candidate: #2128

[#2128](https://github.com/CorbanuCore/CorbanuTerminal/pull/2128), head `504ebb2ce9`, preserves Rama's alpha interfaces through a small vendored DNS adapter using patched Hickory 0.26.3. Original licenses and attribution are retained. No new audit exception, resolver fallback, or network-authorization change was introduced.

Initial independent review caught three P2 findings: unrelated DNS answer owners could be accepted, CNAME targets in additional records could be lost, and vendor-only edits missed targeted CI tests. Reproductions and repairs are committed. Four initial Astra High review lenses plus one coordinator-authorized repair review are complete and clean at this exact head.

Evidence:

- Full isolated proxy tests: **336 passed, zero skipped**, including ten DNS regressions. One existing browser-policy test received a nextest LEAK annotation; its separate fresh replay passed without the annotation.
- Linux CI-selection regressions: **33 passed**.
- Full cargo-deny categories, actual Bazel adapter build, scoped Clippy, formatting, spelling and release-fragment checks passed.
- Failed attempts remain recorded, including old-code reproductions and a foreign-class fixture corrected to expect Hickory's intentional rejection.

**Not merged:** CI is blocked by the runner DNS failures below. The approval does not waive all-green or current-base validation.

### Upstream-sync candidate: #2131

[#2131](https://github.com/CorbanuCore/CorbanuTerminal/pull/2131) is a **draft**, stacked on #2124, at `f7232831a5`. It preserves sync-specific modules, test dependencies and the absence of an already-removed proc-macro exception. All nine vendored Rust files match the reviewed main adapter after identical formatter normalization; provenance explicitly records the sync-only formatting.

Final isolated proxy-library tests: **451 passed; 11 pre-existing S3b credential-engine tests remain ignored**. Cargo-deny, locked metadata, scoped Clippy, Bazel lock update and an actual Bazel adapter build passed. Subsequent completion: the separate coordinator-authorized integration review is clean, and the exact-tree Linux selector replay passed **27 tests**. Neither is whole-tree approval.

**Do not merge the draft:** the underlying sync tree still contains unresolved Rust conflicts and fails whole-tree formatting/build gates. Focused evidence is not full-tree approval.

## CI infrastructure blocker

Main's post-merge run [43027634595](https://github.com/CorbanuCore/CorbanuTerminal/actions/runs/43027634595) failed all four Linux shards while installing packages inside Docker: Ubuntu package-server DNS resolution failed before tests. One bounded failed-job rerun also failed. The failure is tracked in [#2130](https://github.com/CorbanuCore/CorbanuTerminal/issues/2130).

The same package-installation DNS failure affects multiple Linux jobs on #2128. Inspected failures precede product tests; they are not evidence that the repaired product resolver failed. Do not weaken checks, change product DNS policy, or transfer an older green-main result to these commits.

## Larger follow-ups still open

- #2119, #2120, #2123 and #2124: focused upstream MCP/CI repairs remain held by the underlying sync tree. #2101's contributor regressions pass, but that does not finish its broader integration.
- #2121: formatter draft preserves the 362 restored evidence files and original checksum manifests. Its scoped review and Linux checks do not waive whole-tree gates.
- #2122: CLI-helper draft remains held until the upstream CLI is resolved.
- PF-23-S04: independent functional acceptance and the demo reel remain unfinished.
- PF-84-S07 / #88 Stage B: credential-preservation contract and shared-owner/shared-file reactivation gates remain open.
- #1958: WinGet publisher hold remains; no publisher ownership is invented.
- No binary was installed, no release was published, and no independent functional or demo completion is claimed.

The DNS change is an internal bounded security repair under **Shipping MVP — LIVE / Runtime**, whose requirement includes “Rust, … Linux/macOS/Windows.” It adds no TUI interaction; that does not waive qualification of the separately changed product features.

## Owner decisions and actions

1. **88-B:** Recommend failing closed and requesting explicit sign-in when damaged/deleted external Claude credentials cannot be identified safely. The original automatic-preservation contract has not been waived.
2. **New #2114:** Recommend accepting **citadelculture**'s socket-path fixture fix. Its 153 tests, independent review and CI passed; it still needs owner acceptance.
3. **Optional execution prerequisite:** Restore Claude Opus 5.5 High access for independent functional execution and demos. Coordinator implementation and Astra High code review are not substitutes for that independent execution.

Repeat these unanswered items in five-minute coordination updates. During 01:30–09:30 PST, continue useful work that needs no new owner decision. This checkpoint does not promise unattended continuation.
