# RETURN — pf83-package-67

Action `pf83-package-67`; allocation digest `66f7cd159e69f961e7e2424f667e87dbbd19e4b2e65ea460d3ab18551ea6aa5e`; claim `53f2f7ab-8afd-42cc-a266-81eb248cc186`; runtime `gpt-6-astra high`.
Read the brief first and verified SHA-256 with `shasum -a 256`: `959b42f8be2a51338bf43ae681b743f3b2a128170186ba83f5732c1c901d219e`.
Initial clean HEAD/base: `ae5981d2d7d762dfd6d54e0ac0967729bf88f082`; tree `834f57388066d33c27fdee6e3b234dec7fa0b0e6`.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/pf83-rebind-20260916` (denoted `R` below).

**Built and attested package complete. Live harness pin update remains unapplied because its file is outside the explicit writable scope.** The prepared pin patch passes applicability and the unchanged package verifier. The live harness still rejects this package with `source pin mismatch`; no claim that dispatch is enabled or that this assignment is fully complete.

Routine packaging/evidence correction supporting active `p0-security-levels.md`, sprint `PF-83-S01` (`in_progress`), product heading **Permission selection confirmation — TO BUILD**, excerpt “A submitted selection is not a confirmed change.” Used the Corbanu Terminal development skill. No production or Rust edits. No formatting/fix tool, guest contact/staging, packaged execution, functional case, live-profile/credential read, subagent, commit or push. This is an internal build stage, not a functional/human-test/release handoff; the existing independent execution/evidence gates remain open and no integrator acceptance or human sign-off is invented.

## Corrected claims quoted at their locations

[Runbook headline](../pf83-packaged-63-runbook.md):

> **F04 post-Applied authority investigation: executor continuation refused. No separate F05–F09 refusal is established; acceptance of narrower F06–F09 partial attempts is unknown. F05's central withheld-approval question cannot be removed without removing the case.**

[Runbook refusal paragraph](../pf83-packaged-63-runbook.md):

> **The canonical sprint Remaining item at `docs/sprints/current/p0-security-levels/pf-83-s01-permission-confirmation.md:75` still overgeneralizes this as refusal of F05–F11. It is outside this allocation's writable scope and has not been corrected; the manager must apply the prepared `pf83-package-67/sprint-refusal-scope.patch`.**

The [sprint patch](sprint-refusal-scope.patch) passes `git apply --check` and is **not applied**. This follows the review's explicit alternative of disclosing the uncorrected ledger in the runbook. The [original F04 receipt](../pf83-f04-authority-20260916.md) is corrected where its inference appeared:

> The earlier inference that it blocked F05 through F11 was broader than the recorded evidence: no separate refusal for those cases is established.

Its second request now says:

> A disposition for the refused F04 continuation and admission of each remaining case at its actual scope. This receipt does not establish a general class refusal or retire F05–F11.

[Runbook packet inventory](../pf83-packaged-63-runbook.md):

> `fixtures.py:MAP` is many-to-many: `G03-FR` also maps to F04, `G03-RF` to F03, `G12`/`G13` to F02, and `G08`/`G17` to F10. The TSV's `all_mapped_cases` column preserves these shared attributions.

The [TSV](../pf83-packaged-63-case-inputs.tsv) now names its first column `projected_case` and records every shared case, derived from the actual map. All **288** path/hash/attribution rows verify: F05 **8**, F06 **8**, F07 **16**, F08 **40**, F09 **216**. The historical round-63 return's matching claim is corrected too, with its historical test/build observations explicitly dated to that round. G17/F10 was additionally found by the new map check; the review's examples omitted it.

[Runbook staging paragraph](../pf83-packaged-63-runbook.md):

> The adapter also hardcodes `host="agent@192.168.64.3"` and `identity_file="/private/tmp/fmgr.Q1SIYZ/pf83-vm/id_ed25519"` at lines 89–90. It will unconditionally select those settings; a successor must bind the adapter to a freshly verified guest identity and approved identity-file location before staging, without reading/copying credential contents.

[Runbook build paragraph](../pf83-packaged-63-runbook.md):

> It hardcodes source `H/increment-04/source`, the old `e3bd579bf...` pin and output confinement to `H/increment-26`; **do not run it unchanged**.

The successor now provides a concrete pinned source checkout and output, recorded below. No credential file was opened.

## Built package and commands

`P` = `R/qa/initiative-control/management-bootstrap/pf83-package-67`.
Package: [`P/artifacts/package/`](artifacts/package/).
Source: `P/artifacts/source`, clean detached clone at the exact assigned commit/tree, unchanged before/after the build.
Manifest: [package-manifest.json](package-manifest.json), exact byte copy of `P/artifacts/package-manifest.json`.
Attestation: [build-attestation.json](build-attestation.json), exact byte copy of the build's attestation.

- Manifest SHA-256: `32529d0a894503c021ff756ebc27a75555ab05ed2ec63dc47dc0d9d7bb878024`.
- Attestation SHA-256: `349ecadeae3d1d57125acc012103aa6bb92acf1eec42678669a817aaec162d53`.
- Source commit: `ae5981d2d7d762dfd6d54e0ac0967729bf88f082`.
- Source tree: `834f57388066d33c27fdee6e3b234dec7fa0b0e6`.

| Binary | SHA-256 |
| --- | --- |
| `codex-code-mode-host` | `ae25bd69782ecf167a2588d0b8368b53b9bae68d9ec2bed200f55e6598ef810c` |
| `corbanu` | `483031eee7d1b4f978aa782b89164e4072f1533f2eea72a02e1428857254d6de` |
| `corbanu-acp` | `36a35e03f3816bb067867eb46933da18c25662a2be81cd6bb138248b687ccea5` |
| `corbanu-walletd` | `3ec7539bcb6bdb0543e4c933f059a624b411dc5cc3895c730ed101a19e19066d` |

Exactly these four regular files, no symlinks/extras, each mode `0555`; directory `0555`, original manifest `0444`. `shasum -a 256 -c ../../binary-digests.sha256` from the package directory exited **0**, four **OK** results. [Static inspection](binary-inspection.json) confirms four Mach-O arm64 executables and only system dylib/framework dependencies in `otool -L`; all eight inspection commands exited **0**. No binary was launched. The previously missing code-mode host is present. This is a local qualification package, not a notarized release; optional bundled rg/patched zsh are absent and future guest system-tool controls remain required.

Exact outer build command from `R`, exit **0**:

```sh
python3 -B qa/initiative-control/management-bootstrap/pf83-package-67/build_native.py > qa/initiative-control/management-bootstrap/pf83-package-67/build-driver.log 2>&1
```

Exact Cargo command from `P/artifacts/source/codex-rs`, exit **0**:

```sh
/Users/Neo/.rustup/toolchains/1.95.0-aarch64-apple-darwin/bin/cargo build --release --locked --bin corbanu --bin corbanu-acp --bin corbanu-walletd --bin codex-code-mode-host
```

This is the harness increment-26 pinned-source recipe's same four-binary command and Rust 1.95.0 toolchain. [Successor script](build_native.py) confines writes to this allocation, clones the assigned source, and clone-copies only registry/git/target caches from harness `increment-27/build`. It does not copy profiles/credentials. [Preparation commands](preparation-commands.json) preserve the exact five clone/checkout/cache-copy commands, each exit **0**. The attestation records allowlisted environment, toolchain paths/hashes/versions, recipe/script hashes, cache provenance, timestamps, and clean source identities. Cached build, not a from-scratch reproducibility claim.

## Harness pin and frozen-surface comparison

[Proposed live pin patch](harness-candidate-pin.patch) changes only `fixtures.py:CANDIDATE` commit/tree to the built identities above. [Candidate binding](candidate-pin.json) includes the manifest digest and old/proposed fixture hashes. Applicability check exited **0**. Running the unmodified verifier logic with the proposed pin accepted the full inventory/hash/mode/entrypoint checks; the live stale pin rejected it. **The live harness was not edited.**

The writable list excludes `/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915/fixtures.py`. The adapter also confines attestation/package/manifest paths to that harness, so the manager must perform a digest-verified host-side handoff there and apply the patch in an allocation covering those paths. No guest staging is needed for that handoff. Do not disable confinement or silently alter the old normalized packets.

[Source comparison](production-comparison.json): **34/34** production/contract file digests match the previous freeze, this checkout, and the actual built source. **Differing production files: none.** The five changed files between the old runbook pin `c2505dd1f...` and assigned `ae5981d2...` are QA-only. The freeze recorded source digests, not executable digests; the four new executable hashes are not claimed to equal source hashes or an absent previous-round binary baseline.

## Required regression gate

Read `docs/development/test-isolation.md` before tests. From this checkout's `codex-rs`, shared dedicated `CARGO_TARGET_DIR=P/artifacts/test-target`, `NEXTEST_TEST_THREADS=4`, `INSTA_UPDATE=no`, isolated build home/cache/temp and no inherited profile aliases/inference credentials. The guarded `just test` supplies disposable profiles and debug native-keyring denial. No raw Cargo test/nextest invocation, live profile or native prompt.

Prerequisites first: `cargo build --locked --offline -p codex-cli -p codex-rmcp-client -p codex-code-mode-host --bins`, exit **0**. Cache seed copy from the existing dedicated PF-83 target also exited **0**; its new copy is inside this allocation. [Runner](validate.py) records exact argv/environment/exit statuses in `artifacts/test-commands.json`.

| Exact command | Run / passed / failed / skipped | Exit | Run ID |
| --- | --- | --- | --- |
| `just test -p codex-app-server thread_settings --locked --offline --retries 0` | **26 / 26 / 0 / 1082** | **0** | `166f6cee-1622-43bb-a889-1900b61e6758` |
| `just test -p codex-tui permission_confirmation --locked --offline --retries 0` | **12 / 12 / 0 / 4161** | **0** | `0bf5c957-c5d3-4012-8df5-00be8f84192e` |

**38 passed. Exact failure names: none.** No retries, timeouts, leaks or zero-test lanes. One TUI test was classified **SLOW** and then passed in **40.715s**: `app::permission_confirmation::tests::permission_confirmation_f10_request_does_not_optimistically_apply_or_persist`. Lane execution durations were 31.968s and 40.738s respectively, excluding compilation. [Exact test commands/environment](test-commands.json) and [raw log digests](log-digests.json) preserve the evidence; build warnings remain in the raw logs.

## Checks, changed lines and discrepancies

[Verification attempts](verification-attempts.md) preserve the initial G17 attribution-check failure and correction, successful package/pin checks, and initial context-free sprint-patch failure followed by corrected applicability. No failure is relabeled a pass. Manifest/attestation copies are exact; source/packet/package checks do not execute a functional case.

Changed lines: original F04 receipt **+11/-7**; packet TSV **+289/-289** (mechanical shared-attribution column); historical round-63 return **+4/-2**; runbook **+11/-11**. New `pf83-package-67/` evidence comprises **19 files / +1067 lines**, including this 109-line return, four reproducible build/verification drivers, the two unapplied patches, and generated manifests/digest/command inventories. Total **+1382/-309 = 1691 changed lines**; machine inventories and evidence account for the size. Ignored binaries, caches and raw logs are excluded from that source-diff count and stay under the same prefix. **Zero Rust lines changed.** Final `git diff --check` and both patch applicability checks pass; sprint checker passes **115 current / 127 archived**. HEAD remains the assigned base and final status contains only the four corrected QA records plus this new evidence directory.

Brief discrepancies: the live harness update and canonical sprint correction are requested outside the frozen writable list. The sprint finding is handled by the review's permitted explicit-disclosure alternative plus an unapplied patch; the harness update remains genuinely incomplete. The review's many-to-many examples also omit G17/F10. “Matches the frozen production surface” is a source-byte comparison, not equality between source hashes and executable hashes. Built package identity is now available; downstream dispatch, packet restriction, fresh isolation, guest compatibility/PTY controls and independent functional acceptance remain unexecuted, as required by this round.
