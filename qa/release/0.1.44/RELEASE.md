# Corbanu Terminal 0.1.44 — authorized release

## Authority and scope

On 2026-09-22, after the Sol access fix and cancellation-recovery limitation were reported, the requesting repository operator said: “alright looks good. please publish a release”. This is explicit human release authorization under the root AGENTS.md release gate. Codex executes the publication; the requesting operator is release owner. The statement is not represented as exhaustive cross-platform acceptance.

Classification: release packaging of bounded multi-provider catalog/routing/compatibility repairs. Product authority: **Shipping MVP — LIVE / Multi-provider inference**, “OpenAI, Anthropic/Claude Plan … Z.AI, DeepSeek, OpenRouter … and custom providers.” No new trading, credential-storage, payment, or protected-data boundary. No product-initiative plan/sprint is opened for this packaging work.

Branch: release/corbanu-0.1.44. Worktree: /home/pfrpc/corbanu-debug-main-20260922 (mounted scratch worktree). Implementation commit e596867177 is based on main 0f18b63404. Merge 4b6bcd7f05 includes latest-main c9f358c0a7 without discarding its conservative Sol billing metadata. Merge 6194ccc427 includes published rust-v0.1.43, whose provider-credential replacement fixes were still absent from main (PR122 open). This prevents regression from the current public release. Versioned release increases workspace version to 0.1.44; only 150 local workspace package versions change in the lockfile, with no external dependency changes.

## Included changes and evidence

See [release notes](RELEASE_NOTES.md). Detailed implementation/test records: [model and provider work](../../reliability/2026-09-22-opus55-plan-debug/PROVIDER_REFRESH_VERIFICATION.md), [Opus Plan](../../reliability/2026-09-22-opus55-plan-debug/VERIFICATION.md), [selector](../../reliability/2026-09-22-opus55-plan-debug/SELECTOR_VERIFICATION.md), and [Sol compatibility diagnosis](../../reliability/2026-09-22-opus55-plan-debug/SOL_COMPATIBILITY.md). Earlier selector-only account rejections in those historical records are superseded by the controlled version-header diagnosis and successful Sol access checks.

Pre-version-bump local candidate: 419/419 protocol/model/provider tests, 3/3 focused core tests, and 10/10 focused TUI/crew tests passed. Live TUI used the existing normal profile: Sol selection, response, fresh-fixture shell read, explicit recovery after an invalid-model failure, and normal restart succeeded. Same account/model/body with version 0.153.0 failed; version 0.156.0 succeeded. Luna/Astra and Z.AI request regressions passed. Profile config, provider eligibility, and stable launcher/binary hashes stayed unchanged. These results belong to the recorded debug candidate, not yet-built cross-platform release binaries.

## Disclosed gaps and failure

- Post-cancel recovery FAILED: after cancelling a long streamed response, a new prompt stayed Working for 92s; a second attempt also stalled. Fresh sessions succeeded. Independent review and functional handoff checker remain non-green. The release instruction followed disclosure of this failure; publication does not reclassify it as passed.
- Same-session TUI cross-provider switching was not repeated in the Sol repair pass; separate provider CLI checks passed. Tool test used user-requested YOLO mode, not an interactive approval dialog.
- Full release-version TUI qualification in both TensorCash and Isometric Game is not performed in this release turn; historical checks are not relabeled as new passes.
- The competitor/model benchmark cycle remains incomplete. The earlier requested Corbanu/Hermes experiment remains paused; no leaked task, incomplete trial, or inference readiness probe is called a benchmark win.
- FlashX API integration remains unimplemented and unauthorized; no FlashX Plan support is claimed.
- Cross-platform human interactive acceptance and a full workspace test suite are not claimed. Workflow package smoke tests are separate from interactive qualification.

Release-tree preparation: `just fmt`, `git diff --check`, five Unix installer-contract tests, portable-skill parity (25 files), and `just bazel-lock-update` passed. Bazel emitted existing direct-dependency version warnings; MODULE.bazel.lock did not change. Final versioned focused Rust tests are running at dispatch and are not preclaimed passed.

Proceed with the existing cross-platform release workflow under explicit authorization, recording its actual outcome. No version-tag reuse or previous artifact reuse is intended. Public publication is confirmed only when GitHub shows the completed release and its assets.
