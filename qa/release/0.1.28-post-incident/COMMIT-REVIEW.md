# 0.1.28 post-incident commit review

Date: 2026-08-09

Locked baseline: `rust-v0.1.27` / `827ad686965042a474fe4146471cf71b54ab440d`

Quarantine ref: `quarantine/main-pre-lockdown-20260809` / `4a5dce18d0401a603ef5532cdb7a1934ce43ff97`

## Scope correction

The incident plan says the range contains 37 commits. The tag-to-quarantine audit command,
`git rev-list --count rust-v0.1.27..quarantine/main-pre-lockdown-20260809`, returns **38**.
All 38 commits are reviewed below. This discrepancy is itself evidence that release scope must
always be generated from the exact tag range rather than copied into prose.

Verdicts mean:

- **readmit**: acceptable as an individual cherry-pick after the normal focused checks shown here.
- **readmit-after-fix**: the intent is useful, but the quarantined commit must not be cherry-picked as-is.
- **reject**: do not reintroduce this commit; a replacement, if wanted, starts from the locked baseline.

## Verdicts

| # | Commit | Runtime or repository surface | Verdict | Review basis |
|---:|---|---|---|---|
| 1 | `c56d4c3dc8` | Unix/Windows installers, release asset selection, release CI | **readmit** | Corrects the fork's asset names, repository URLs, state home, and compatibility fallbacks; includes contract tests. It does not implement the deleted-release updater requirements, which remain a separate gap. |
| 2 | `dc15684c44` | Windows installer tests | **readmit** | Adds the missing Windows ZIP-selection coverage without changing shipped behavior. |
| 3 | `4d7d12b427` | Bazel module lock | **readmit** | Mechanical lock refresh for the existing `path-absolutize` dependency. |
| 4 | `9997c15262` | Bazel CI scheduling and no-RBE resource budgets | **readmit** | CI-only resource/scheduling change. No product runtime path is changed. |
| 5 | `02f7fc75b3` | Release and checkout runbooks; worktree guard script | **readmit-after-fix** | The checkout policy switches the canonical checkout to a feature branch and then tries to add that same branch as a second worktree; that is contradictory and fails Git's one-branch-per-worktree rule. The release prose also records stale commit identities. Keep the guard script only after correcting and testing the workflow. |
| 6 | `a85fb3a968` | Config derivation and resumed-thread provider/model pairing | **reject** | It broadens automatic correction from inferred providers to explicitly configured providers and mutates both provider and model. In combination with #83, an explicit gateway + bare Claude pair is silently redirected to direct Anthropic. Explicit routing is an operator decision and must fail closed with a diagnostic, not be rewritten. |
| 7 | `34c3f22d8c` | Bundled PostFiat L1 skill asset | **readmit** | Inert unless selected and covered by skill-catalog tests; no provider or release path change. |
| 8 | `6789541f74` | Archived incident/spec/handoff documentation | **readmit** | Repository-only archival material. Large, but no executable or generated release surface. Readmit separately from runtime changes so it cannot obscure release scope. |
| 9 | `c52db5eff0` | App-server error data and Telegram unmaterialized-thread UX | **readmit-after-fix** | The raw error propagation is useful and tested, but Telegram still recognizes a user state through exact English string matching. Replace the wire string match with a structured app-server error code before readmission. |
| 10 | `0c8ec95cf5` | Model-provider catalog correction | **reject** | Adds a Claude allowlist that maps known non-allowlisted providers to direct `anthropic`. Alone it is conservative for explicit config because 0.1.27 only corrected inferred pairs; #86 makes it destructive for explicit pairs. Unknown custom providers are actually left unchanged, contrary to the incident plan's broader wording. A replacement must preserve explicit pairs and report incompatibility. |
| 11 | `4f0b5df020` | Telegram `/status`, thread token accounting | **readmit** | Focused addition based on app-server thread token usage with formatting/unit coverage. Live staging remains a release gate, not a reason to conflate this with provider routing. |
| 12 | `9d00507174` | New `pfterminal-acp` executable and launcher resolution | **readmit** | The isolated launcher implementation has focused tests. The built candidate's `--version` exited 0 with both resolved-path diagnostics, and a controlled real handoff exited 0 with an empty stdout SHA-256. Treat it as a separate feature PR, never release collateral. |
| 13 | `9efd14e59a` | `peter-evans/create-pull-request` workflow pin | **readmit** | CI action pin only. |
| 14 | `196f104b0f` | `actions/cache/save` workflow pin | **readmit** | CI action pin only. |
| 15 | `60f773282a` | `actions/cache` workflow pin | **readmit** | CI action pin only. |
| 16 | `11e27b4e8d` | `taiki-e/install-action` workflow pin | **readmit** | CI action pin only. |
| 17 | `91d2cac3eb` | `astral-sh/setup-uv` workflow pin | **readmit** | CI action pin only. |
| 18 | `89a3186efb` | `serde_json` 1.0.149 to 1.0.151 | **readmit** | Source audit shows stricter map-key deserialization/error positioning, an unsafe raw-value constructor addition, and mechanical cleanups. No serializer wire-format change was found. Focused provider payload tests are still required. |
| 19 | `a9b344aa1b` | `pkg-config` 0.3.32 to 0.3.33 | **readmit** | Changelog change restores captured stderr to probe errors; build-time only. |
| 20 | `3a0885748d` | `semver` 1.0.27 to 1.0.28 | **readmit** | Source changes are edition/MSRV modernization and equivalent parsing/display/unsafe-pointer rewrites; repo Rust is newer than the new 1.68 MSRV. |
| 21 | `2881363719` | `serde_with` 3.17 to 3.21 | **readmit** | Includes the `KeyValueMap` allocation/panic security fix and documented MSRV increases still below this workspace's toolchain. Requires its focused downstream tests. |
| 22 | `f3c793b119` | `constant_time_eq` 0.3.1 to 0.5.0 | **reject** | This is not a routine bump: 0.4 replaced the constant-time implementations and enabled `std` by default; 0.5 requires Rust 1.95 and follows a yanked 0.4.3 version correction. Crypto-adjacent major changes need an isolated cross-platform/security review, not an unrelated release batch. |
| 23 | `e0efd32b33` | Proof-of-adversarial-review proposal | **readmit-after-fix** | Inert documentation, but it presents external Ambient/Proof-of-Logits properties as facts while also admitting the receipt format is unverified. Mark external claims as unvalidated design assumptions or substantiate them before archival. |
| 24 | `9cea0830f9` | Archived wallet QA evidence | **readmit** | Documentation/evidence only; no wallet runtime change. Keep outside release-critical PRs. |
| 25 | `c231907006` | Canonical UI HTML and binary screenshots | **reject** | Unrelated product-design bundle (including a malformed `<footer>` opening in the HTML) with no PFTerminal runtime or release purpose. It should live in its owning project/archive, not this patch release. |
| 26 | `5d623904c2` | Task Node non-destructive auth state and headless link flow | **readmit-after-fix** | Separates active and pending credentials, validates candidates before promotion, and avoids destructive relink. However, the required ordinary `just test -p codex-tasknode-session` gate is red: 7 pass and 3 vault-heavy tests time out at 60 seconds, including an N=1 retry. The test/storage cost must be fixed or qualified under the normal profile before readmission. |
| 27 | `b6d5460609` | Task Node session expiry handling | **readmit-after-fix** | Expired sessions stop shadowing pending links; invalid/missing expiry remains server-authoritative, and the three new expiry assertions pass. It depends on the red Task Node storage change and cannot be readmitted until that crate's normal focused gate passes. |
| 28 | `5bbb5b636f` | Bundled Task Node skill command selection | **readmit-after-fix** | Correctly identifies stable/debug home isolation, but the instructions say entrypoints always use the default homes and choose only from `CODEX_HOME`; actual entrypoints honor `PFTERMINAL_HOME` / `PFTERMINAL_DEBUG_HOME`. Document override-aware resolution. |
| 29 | `611993d3e3` | Rust formatting for Task Node files | **readmit** | Mechanical formatting only; fold into the Task Node change rather than cherry-pick alone. |
| 30 | `801f9a13f1` | Task Node unused import/redundant clone cleanup | **readmit** | Mechanical, behavior-preserving cleanup; fold into Task Node change. |
| 31 | `cc63f0838d` | Comment spelling | **readmit** | Documentation-only spelling correction; fold into Task Node change. |
| 32 | `598f5059a8` | Codespell config and Windows pseudo-console message | **readmit-after-fix** | The runtime message typo is valid, but skipping the entire `canonical-assets` and `target` names globally can hide source nested under matching paths. Split the typo and use anchored/generated-path exclusions. |
| 33 | `d474ffd3fb` | Codespell exclusions | **readmit-after-fix** | Skipping all `./qa/artifacts` removes QA prose from spelling review. Use narrow binary/generated patterns rather than a whole evidence tree. |
| 34 | `abfc1daac1` | Merge commit for #93 | **reject** | No standalone readmission value; cherry-pick the reviewed constituent Task Node commits only. |
| 35 | `444fc0b94c` | Workspace 0.1.28 version/release preparation | **reject** | Belongs to the yanked release and would recreate its version identity. |
| 36 | `d13e239b96` | 0.1.28 Windows acceptance evidence | **reject** | Evidence is tied to the yanked candidate and cannot qualify a future rebuilt candidate. |
| 37 | `0f01128d77` | Merge commit for release PR #95 | **reject** | No standalone code value and represents the invalid release aggregation. |
| 38 | `4a5dce18d0` | Published 0.1.28 release verification | **reject** | Records a deleted/yanked publication and cannot be reused as future release evidence. |

## Provider-routing root cause

The failure is an interaction, not a defensible single-commit attribution:

1. `0c8ec95cf5` teaches `corrected_catalog_provider` that a bare `claude-*` on a
   known non-allowlisted provider should map to `anthropic`.
2. On the locked 0.1.27 configuration path, correction is only applied when the provider was not explicit.
3. `a85fb3a968` removes that provenance guard and also applies correction while restoring thread settings.
4. The combined candidate therefore discards an explicitly selected gateway/transport and can resolve the
   request against a different credential, endpoint, and model mapping.

The safe replacement design is: preserve explicit provider/model selections; validate them; return a typed,
immediate incompatibility error that names the pair and suggested alternatives. Never silently change vendor,
credential, endpoint, or billable route during config derivation or resume.

## Re-admission order

No quarantined merge or release commit should return. If maintainers choose to proceed, start with isolated
CI/docs/dependency PRs, then Task Node, Telegram, and ACP as separate PRs. Provider routing requires a new
implementation from the locked baseline plus the full live provider matrix. This review does not authorize a
merge, tag, publication, or deployment.
