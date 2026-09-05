# Corbanu Terminal 0.1.38 integration candidate

## Latest repair — explicit Astra subagents, September 5

Source `51185a24d404f98ce7f0dd5fc67e516deabee000` separates explicit runtime
discovery from automatic-allocation economics. Astra is now advertised as an
explicit choice without inventing prices or changing spawn authorization.
183 focused Core tests and 22 harness tests pass; an intermittent existing
resume-fixture timeout on another run is disclosed in the evidence.
Four actual Astra children passed tool work, cancellation/recovery and same-child
cold resume across both default repositories: 20 child responses and 12 paired
tool calls. `corbanu-debug` uses the verified installed build; the fresh human
session is `corbanu-astra-agents`. [Exact evidence and limits](astra-subagents.md).
No release was published; older user sessions are unchanged.

## Historical repair — native Luna/Kimi subagents, September 5

Runtime source `e7cdb94359a7bdedeb6b0abdf2f17f09823d08e1` repairs the mixed-model
catalog filter, V2 child inheritance and exact OpenAI-runtime adapter dead end.
180 selected Core tests and 19 harness tests pass. Both real child runtimes pass
tool-backed repository tasks, parent cancellation/recovery and cold resume with
the original child IDs. `corbanu-debug` now uses this verified build;
`corbanu-agents` is the fresh human session, with the old session preserved.
[Exact evidence and limits](subagent-runtime.md). No release was published.

## Historical repair — live GPT-6 Astra, September 5

Runtime source `6df2f2e2ed545506057e9e1aa7a76b9375aaea73`, harness/source tree
`f848954d7739da8eaa4962f0866c612dedbaf5bb`. Fixed the real newer-client rejection
and native Astra runtime metadata. **445 selected Rust/TUI tests and 12 harness
tests pass**. The exact installed debug binary passed actual Astra file/tool,
cancel/recovery, restart and same-thread resume in both TensorCash and Isometric
Game: 18 responses and 12 paired tool calls. `corbanu-debug --yolo` and the
separate `corbanu-test` session use that binary and the approved normal profile.
[Exact evidence and limits](astra-runtime.md). No release was published; human
follow-up, cross-platform qualification and competitor benchmarks remain separate.

## Historical selector addition — GPT-6 Astra, September 5

Selector-only implementation source: `6b17a2630f31f5447d2c53fa8f6a29b60407b42a`
on `integration/reconcile-release-0.1.38`. GPT-6 Astra is selectable under
OpenAI with its supported reasoning levels; existing Sol/current defaults and
automatic allocation remain unchanged. Final affected tests passed **87/87**,
including real-TMUX cancellation, selection, restart and loopback Responses
request checks. Manual `just codex` selection also passed.

[Astra selector evidence](astra-selector.md) records exact sources, binaries,
commands, account-access limitations and test-fixture corrections. This is not
live Astra inference or new cross-platform/release qualification. No release
workflow, tag or publication was started by this addition.

## Latest reconciliation — September 5

Reconciled baseline source: `c37eb277d9f83ebcabe89e41cc81b9d3e92797a2` on
`integration/reconcile-release-0.1.38`. This preserves Travis/IridiumMaster's
`07791288b6` integration tip and the two PF-57-S02 repair commits. Fork `main`
through `6dd9ad646b` is already included. Claude token/login improvements and
Ambient's GLM-only selection remain intact.

Combined Linux evidence: 625 selected tests, 16 real-TMUX application/harness
tests and a manual `just codex` flow passed. Astra's bounded combined-source
review found no new P0/P1. [Exact evidence and limits](travis-reconciliation.md)
include source/build identities and the unresolved debug credential-logging
concern. No tag or release was published by this reconciliation.

The sections below retain historical candidate evidence. Their build hashes,
platform passes and publication statements are scoped to their original runs;
they are not new qualification of this combined source.

## Original candidate record

Date: 2026-09-03 UTC

Branch: `integration/reconcile-release-0.1.37`

Release target: untagged `0.1.38` development candidate

Release authorization: none. This record covers line reconciliation and
qualification only; it does not authorize a tag or publication.

## Included scope

- Full history from published `rust-v0.1.37`, including Task Node profile
  isolation, Corbanu API wallet balance, wallet-funded models, and Fable 5.1.
- Current `main` through `6dd9ad646beb4a7407521439411f436f21ea4af1`,
  including PF-48–PF-57 unified provider authentication, immediate credential
  activation, atomic vault storage, and subsequent security integration.
- Semantic conflict resolution that retains the unified provider/vault
  lifecycle, the arbitrary-balance wallet surface, both Fable 5/5.1 routes,
  and the Corbanu API Kimi Messages route.

## Qualification evidence

- Merge commit `aeaabfeaae1c2595a80c7118926e855de4ad6537` preserves both
  exact histories. Its main-line parent is
  `64b12a2e42a73ed17fa9186cf590380d77e27ad6` (including main through
  `6dd9ad646beb4a7407521439411f436f21ea4af1`), and
  `rust-v0.1.37` (`43ff201aabb633a8b6a1a10ea2b1544d92ad3902`) are both
  ancestors of the candidate.
- Semantic reconciliation routes unified onboarding's deferred Corbanu choice
  into the release line's wallet-funded Corbanu API balance/key flow. New Plan
  sales and legacy entitlement UI remain retired; persisted compatibility IDs
  remain stable.
- `just fix`, `just fmt`, `git diff --check`, plan/sprint governance checks,
  and the portable-skills mirror check pass.
- Focused final-tree suites: provider-auth 64/64; Corbanu API UI 10/10;
  provider credentials 12/12; wallet unlock 7/7; onboarding 72/72; wallet menu
  25/25; provider manager 8/8.
- Focused provider/stream core: 31/31, nextest run
  `75e64c46-ee5b-4ff2-920f-c0b16d9cfd5e`.
- True-TMUX: multi-provider onboarding 11/11 on the formatted final source,
  including deferred-passcode cancellation/re-entry and the one-time API-key
  create/store/reveal/activate path without restart; plus the exact PF-52
  Claude recovery journey 1/1.
- Independent TMUX + Corbanu Terminal + Claude Fable 5.1 Max review converged
  to `FINAL_REVIEW: CLEAN`; findings and fixes are recorded in
  `autoreview-fable.md`.
- Native macOS release build: pass at `3f64f36c6383005a3c71fad9fc032de8698a7b85`;
  `corbanu --version` reports `0.1.38`. The successful resumed build took
  318.93 seconds after the first build exposed and stopped on the corrected
  macOS-only compile defect.
- Fresh detached-worktree Linux release build on the remote builder: pass at
  the same exact commit in 377.26 seconds; `corbanu --version` reports
  `0.1.38`. The clean detached source path required new Cargo fingerprints, so
  this is release evidence rather than a warm incremental benchmark.
- macOS SHA-256: `corbanu`
  `279e671e96f75751bb91df3b86f563c8b55a28fe6bde160a635258b1caf21251`;
  `corbanu-acp`
  `1aa66623c5c465b25c86f8a2402752854f407d3dec347e720d3197d1c4d6d73d`;
  `corbanu-walletd`
  `5874fbc9cea2b03a12615a403873b82769204f9cf3cc09fc7617226e7152923d`;
  `pfterminal-walletd`
  `ef09ab6bd46c42b7d79723c978845151191b90a59021749849ce7f4abcce63ce`;
  `codex-code-mode-host`
  `86da26851055ec156a51d74ddc2560326a661034a4bdd568a44f2f90c820ab68`.
- Linux SHA-256: `corbanu`
  `241d9fa818d32f40b347f67c2885ac5d8f6dfbedd87f19a4ae10cdea3aa31df7`;
  `corbanu-acp`
  `2ee62c018ac244b123a3c3d8cbff7c6486a5e0b1a13cda3784d275d34b7b4a91`;
  `corbanu-walletd`
  `17225e0aa90fbd8f25d824aef01f9d4c0f9c9021b7c5079d92f98d130e9a15a2`;
  `pfterminal-walletd`
  `3596b6b67cbbb90935dadce0a42c72979c4304c10c4c4b66219f5f0e261f53e4`;
  `codex-code-mode-host`
  `a3642e3dbf416d1229c34e1330c3f3804c1ea4ccbdc8ed956d4f8bb0593e5365`.
- Installed-app smoke: pass. `/Applications/Corbanu Terminal Launcher.app`
  resolved the stable `bin/corbanu` and `bin/codex-code-mode-host` links to
  the freshly built release artifacts, started the `0.1.38` binary, and placed
  its window through the launcher. The window was then brought to the
  foreground for human acceptance.
- Strict MkDocs is still blocked by 19 pre-existing excluded/missing-link
  warnings outside this candidate's changed pages; the non-strict site build
  completes and changed pages introduce no new warning.
- Human acceptance remains pending.
- Provider UX parity follow-up: the `/providers` Claude managed-token route now
  restores the established `claude setup-token` instructions from one shared
  presentation source, with rendered snapshots and true-TMUX coverage. The
  broader audit and the subsequent six-finding correction are recorded in
  [provider-ux-parity-audit.md](provider-ux-parity-audit.md).
- Follow-up final-tree checks: `just fix -p codex-tui`, `just fmt`, and
  `git diff --check` pass. Both shared-Claude presentation snapshots pass
  (nextest run `4e31e145-3a4c-4229-b261-4dd6292975c2`), and the corrected PF-52
  true-TMUX journey passes independently (nextest run
  `0c78c3bd-b6fd-4e56-b206-668ac1fa20d0`).
- A broader fresh-binary provider-management run passed the other nine tests;
  PF-52 timed out under parallel contention despite its independent pass. The
  matrix rerun exposed a separate macOS fixture/keychain isolation defect, so
  local true-TMUX launches were stopped and leaked test process trees removed.
  No production credential was deleted or modified.

### Combined provider UX correction, September 4

- All six audited findings are implemented: typed visible failure/recovery,
  wrapped masked-entry guidance, clickable OpenAI challenges, shared API-key
  custody/privacy instructions, shared onboarding copy, and isolated/cleaned-up
  macOS TMUX fixtures.
- Focused unit/component qualification: **253/253 passed**. Harness cleanup:
  **8/8 passed**, including forced parent termination and a signal-resistant
  pane. Formatting, scoped fix, and diff checks passed.
- Fable 5.1 Max via Corbanu/TMUX reviewed the combined patch: no correctness
  blockers; two P3 findings covered headless step numbering and stale release
  wording. Both are corrected. Post-review UI checks passed **50/50** without
  snapshot updates; the final review resolution exited **0** with no actionable
  findings after verifying the accepted snapshot and passing test evidence.
- The final native Mac release build completed and passed `--version` and
  `codesign --verify --verbose`. The stable Apps launcher still resolves
  `bin/corbanu` to `../target/release/corbanu`; the wallet daemon is installed
  alongside the canonical executable. SHA-256 at that qualification:
  `corbanu` = `0a9adad86c75de8200cebe862a6124fdbba46c0be6b6c554172f50bbc5f8e36e`;
  `pfterminal-walletd` = `684fbe986ba6c3d4cee4c7a3b1e6ea7fc3c82eebcadb01312b155f566656552b`.
- Expanded TMUX qualification: 32/33 passed in the matrix, with the remaining
  Claude submission/recovery case passing a final no-retry rerun after fixing
  a blank form reopening on submission. All 33 selected cases have passing
  coverage across these runs, not one clean matrix run. An environment-case
  input-timing retry also passed a separate no-retry run. Earlier obsolete test
  wording and missing wallet-daemon fixture prerequisites were corrected.
- The final submission correction passed 38/38 focused UI tests and Fable 5.1
  Max review through Corbanu/TMUX (no findings, exit 0). The human negative
  token test now requires embedded spaces to guarantee rejection before storage.
- A pre-existing debug key-event logging privacy concern was found using
  synthetic credentials and is documented in the parity audit. Do not enable
  debug/trace logging while entering real credentials. Its protected-data fix
  remains separately scoped; no production credential was used in these tests.
- `humanTest.html` now includes checks 13–15 for the corrected flows. Named-human
  acceptance is still pending. Nothing has been merged or published by this
  correction.
- Latest installed-app smoke created the new `Corbanu — provider UX retest`
  window with the verified binary. Startup stopped at the existing release-
  recall heuristic's erroneous 0.1.38 → 0.1.37 prompt. Choose **2. Skip**;
  do not downgrade. Computer Use denied Terminal access, so reaching chat after
  that prompt has not been verified in this launch. See the parity audit.

## Disclosed incomplete gates

The subsequent Ambient GLM-only catalog correction and its newer verified
native executable hash are recorded in [ambient-glm-only.md](ambient-glm-only.md).
It passed 63 model-manager tests, 14 focused UI tests, a true-TMUX picker
journey, and final Fable 5.1 Max review. It does not complete the release gates
below or change other providers' model catalogs.

- No named-human acceptance has yet been recorded for the combined candidate.
- The benchmark bootstrap cycle is due and has not been run.
- Cross-platform and live-repository release qualification remain pending.

These incomplete gates block publication absent explicit release authority;
they do not block development integration or local human acceptance testing.
