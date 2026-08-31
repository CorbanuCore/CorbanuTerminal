---
title: "Reliable Claude subscription authentication"
status: active
change_class: product-initiative
priority: P1
owner: "Jim Ricketts"
parallel_sprint_limit: 1
integration_owner: "Jim Ricketts"
activation_authority: "Travis Good — final product authority"
activation_basis: "Travis Good's 2026-08-30 decision to give reliable Claude subscription authentication its own plan and branch, with the long-lived subscription token as the recommended default and current credential detection plus legacy migration repaired."
target_release: "TBD"
deadline: "TBD"
created: 2026-08-30
updated: 2026-08-31
product_spec:
  file: docs/corbanu-product-spec.md
  heading: "Shipping MVP — LIVE"
  requirement_excerpt: "Encrypted `/vault`, masked entry, metadata-only inspection, and operational credential use without placing raw values in chat."
implementation_worktrees:
  - path: "/Volumes/CorbanuDrive/Corbanu/worktrees/claude-subscription-auth-isolated"
    branch: "feat/claude-subscription-auth-isolated"
    base_commit: "8ae13e168817445205321bae410740cbc3e919b7"
---

# Reliable Claude subscription authentication

Policy: repository-root `AGENTS.md`

Plan lifecycle: `docs/plans/index.md`

## Activation record

| Field | Value |
| --- | --- |
| Status | **Active** |
| Active-plan slot | **2 of 2** |
| Product authority | Travis Good — final product authority |
| Authoritative decision | On 2026-08-30, make reliable Claude subscription authentication a separate plan and branch; recommend a long-lived subscription token while retaining an explicit Claude Code login compatibility choice and repairing current/legacy credential management. |
| Delivery owner | Jim Ricketts |
| Target release | TBD |
| Deadline | TBD |

This activation allocates a plan and branch only. It does not create an
implementation sprint or authorize edits outside a future `ready` or
`in_progress` single-feature sprint.

Coordination commit `f36c28770` is a lifecycle-only mirror of the finalized
PF-13-S06 and PF-41-S03 allocation state from committed main. The isolated
Claude-auth branch does not contain or re-qualify those features' product code
or evidence; their completed/archive claims are valid only when this delivery
is integrated onto the main lineage that already contains that work. The mirror
predates every CSA sprint and remains outside PF-46's literal scope because the
active P0 records are already owned by PF-35's disjoint live scope.

## User pain

Claude Plan works until the shared Claude Code OAuth record loses or rotates its
refresh token, after which Corbanu fails with an internal provider-auth error and
tells the user to log in again. Multiple Claude CLI, Desktop, and Corbanu
processes can compete over the same rotating credential. On macOS, a legacy
credentials file can also mask the current Keychain record. Users cannot see
which source Corbanu selected, why it failed, or how to move to the more stable
subscription-token path without manually managing a secret in their shell.

The result is repeated interruption, ambiguous account state, and a dangerous
temptation to copy credentials into plaintext configuration or chat.

## Product intent and ideal flow

The first time a user selects Claude Plan, Corbanu presents two short choices:

1. **Stable subscription token — Recommended.** Corbanu guides the user through
   Anthropic's `claude setup-token` authorization, stores the resulting token in
   an approved secret store, and uses it for subscription-backed model requests.
2. **Use my Claude Code login.** Corbanu uses the platform-authoritative Claude
   credential store and explains that shared rotating login state may require
   reauthentication when several Claude processes run concurrently.

The chosen method persists and is visible as metadata-only status. Corbanu never
silently switches credential sources, accounts, or billing contexts. Existing
users keep their current behavior until they explicitly accept migration. A
legacy or unhealthy source produces a specific recovery view that identifies
the source, preserves cancel, and offers reconnection, replacement, or a switch
to the recommended method. Successful recovery resumes Claude Plan without
exposing the credential or discarding unrelated provider state.

## Product linkage

| Field | Value |
| --- | --- |
| Exact product-spec heading | **Shipping MVP — LIVE** |
| Requirement excerpt | “Encrypted `/vault`, masked entry, metadata-only inspection, and operational credential use without placing raw values in chat.” |
| Related shipping capability | “OpenAI, Anthropic/Claude Plan, Kimi, Z.AI, DeepSeek, OpenRouter, Ambient, Meta, Baseten, Vercel, Bedrock, Ollama, LM Studio, Corbanu Plan, and custom providers.” |
| Compatibility requirement | **Live MVP versus the P0 security controls** — “Permissive preserves the shipping behavior and does not silently change existing policies.” |
| Product outcome advanced | Claude Plan remains usable across long-running and multi-agent sessions while credentials stay outside model-visible context. |
| North-star criterion advanced | A user can direct an agent through a dependable provider without exposing credentials or needing repeated manual login repair. |

## Scope

### In

- A concise Claude Plan authentication-method choice with the long-lived
  subscription token recommended for new enrollment, reachable directly from
  first-run provider onboarding as an Anthropic account rather than only from
  the later `/providers` command.
- A verified `claude setup-token` handoff that never renders the resulting token
  in chat, ordinary TUI history, logs, crash reports, or command previews.
- Secure Corbanu-managed storage and runtime resolution of the selected
  long-lived token through existing vault/keyring/trusted-helper boundaries.
- A persisted, metadata-only authentication source selection and health model.
- Explicit support for user-supplied `CLAUDE_CODE_OAUTH_TOKEN` without silently
  importing or duplicating it.
- Platform-authoritative Claude Code login discovery: macOS Keychain and
  Linux/Windows `.credentials.json`, including `CLAUDE_CONFIG_DIR` behavior.
- Detection of missing, blank, stale, malformed, ambiguous, and conflicting
  current or legacy credential records without logging their values.
- A reversible, human-confirmed migration and recovery flow for existing users;
  cancel is inert and no legacy credential is deleted automatically.
- Deterministic provider-auth source selection across startup, refresh, 401,
  restart, resume, child agents, and native Claude panes.
- Focused unit/integration coverage, cross-platform qualification, true-TUI
  success/failure/recovery/resume proof, shipped guidance, and human acceptance.

### Out

- Creating, selling, or brokering Anthropic subscriptions.
- Converting a rotating Claude refresh token into a long-lived token without the
  user's Anthropic authorization flow.
- Direct OAuth token exchange against undocumented Anthropic endpoints.
- Silently copying Claude's Keychain/file secrets into Corbanu storage.
- Automatically deleting, rewriting, or garbage-collecting Claude-owned
  credentials or Keychain entries.
- Supporting Claude Desktop/cloud-only features such as Remote Control or
  Claude.ai connectors through the model-request-only long-lived token.
- Changing Anthropic API-key, Bedrock, Vertex, Foundry, gateway, or unrelated
  provider authentication.
- Weakening Moderate or Aggressive credential-broker requirements.

## Invariants

- The raw credential never enters model context, chat, rollout history,
  telemetry, logs, error strings, clipboard automation, or ordinary config.
- The user selects the authentication method; Corbanu persists that choice and
  never opportunistically falls through to another source after failure.
- Existing users do not change source until they explicitly confirm migration.
- A source conflict fails visibly and identifies only source metadata; it never
  guesses which account, organization, or billing context the user intended.
- On macOS, a persisted legacy `.credentials.json` selection remains usable
  only while the matching Keychain item is proven absent by the typed
  `errSecItemNotFound` result. A present, malformed, denied, unavailable, or
  unverified Keychain item fails closed before Claude status or legacy-file
  token resolution, so account verification can never attest one store while
  Corbanu returns another store's credential.
- A blank or missing rotating refresh token is not represented as recoverable
  without reauthorization or a valid independently managed long-lived token.
- Legacy migration is reversible until the new source passes validation and the
  user explicitly chooses whether to remove any Corbanu-owned obsolete state.
- Claude-owned credential stores remain Claude-owned and are not modified by
  Corbanu except through documented Claude CLI flows initiated by the user.
- Provider 401 recovery cannot rotate or invalidate a source that was not the
  source selected for that request.
- Permissive retains its existing behavior for users who decline migration;
  Moderate and Aggressive continue to require their plan-owned trusted
  credential-resolution boundaries.
- Unsupported platforms, Claude versions, stores, or ambiguous records fail
  with a recovery action rather than exposing or silently downgrading a secret.

## Ownership and implementation worktrees

| Owner | Worktree | Branch | Base commit | Scope |
| --- | --- | --- | --- | --- |
| Jim Ricketts | `/Volumes/CorbanuDrive/Corbanu/worktrees/claude-subscription-auth-isolated` | `feat/claude-subscription-auth-isolated` | `8ae13e168817445205321bae410740cbc3e919b7` | Plan integration and all single-lane implementation; exact per-sprint write scopes remain pending. |

Parallel implementation is not enabled. `parallel_sprint_limit: 1` requires one
ready or in-progress sprint at a time, and shared auth/TUI/persistence surfaces
remain with the integration owner.

## Useful code references

| Path or symbol | Why it matters |
| --- | --- |
| `codex-rs/cli/src/claude_oauth.rs` | Current environment, file, Keychain, expiry, lock, and forced-refresh behavior. |
| `codex-rs/cli/src/main.rs::run_internal_claude_oauth_token` | Trusted CLI boundary that emits the provider bearer credential to its parent process. |
| `codex-rs/login/src/auth/external_bearer.rs` | External provider token cache and forced refresh after 401. |
| `codex-rs/model-provider-info/src/lib.rs::create_claude_plan_provider` | Claude Plan provider auth command, refresh interval, and wire configuration. |
| `codex-rs/tui/src/chatwidget/claude_code_login.rs` | Current inline Claude subscription login/status workflow and likely choice/recovery seam. |
| `codex-rs/tui/src/chatwidget/model_popups.rs` | Claude Plan model-selection entry point and explanatory copy. |
| `codex-rs/tui/src/claude_panes/provider.rs` | Native Claude pane profile and current “native auth” contract. |
| `codex-rs/vault/src/lib.rs` and `codex-rs/keyring-store/` | Existing protected storage and platform keyring boundaries to extend rather than duplicate. |
| `codex-rs/core/tests/suite/client.rs` | Existing provider-auth command and 401 refresh integration coverage. |

## Upstream-touch record

| Field | Record |
| --- | --- |
| Baseline | Canonical upstream `https://github.com/openai/codex.git` was fetched as `upstream/main` on 2026-08-30. The fetched upstream tip is `b7cd519c767c8fd4bc3581d9bc92fbab37a768c1`; the verified merge-base with fork base `8ae13e168817445205321bae410740cbc3e919b7` is `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa` (`Ignore symbolic slash-tmp permissions on Windows (#36237)`, 2026-07-30). Candidate SHA is pending. |
| Footprint | Anticipated adapters are the CLI OAuth helper, external bearer auth, provider info, Claude login/model TUI, and their focused tests. Each sprint must replace this anticipation with literal changed files and its single feature owner before readiness. |
| Boundary | Keep provider credential discovery/resolution behind typed product-owned source and status contracts. TUI initiates human choice and displays metadata; it cannot own raw storage or refresh policy. Upstream external-auth interfaces remain thin adapters. |
| Compatibility | Preserve provider wire headers, selected account/source, 401 behavior, config persistence, native pane startup, child use, cancellation, restart, and resume. API-key and non-Claude providers are non-applicable and must remain unchanged. |
| Verification | Final affected crate tests, provider-command 401 integration, source-conflict/adversarial redaction tests, Linux/macOS/Windows store fixtures, true-TUI flows in both default repositories, and release evidence. Exact commands and artifacts are assigned by each sprint. |
| Upgrade handling | Before release, resolve the upstream baseline/candidate, classify every touched adapter as retained/adapted/removed, record conflicts and qualification results, and keep Corbanu policy outside upstream wire types. |

## Sprint execution map

No sprint is created by plan activation. Before implementation, create one
single-feature record under `docs/sprints/current/claude-subscription-auth/`,
record these exact worktree coordinates, set it to `ready` or `in_progress`, and
run `python3 docs/sprints/check.py`.

| Feature ID | Plan feature | Current sprint records | State |
| --- | --- | --- | --- |
| `PF-42` (`CSA-01`) | Typed authentication source, selection, health, and persistence contract | [CSA-01 / PF-42-S01](../../sprints/archive/claude-subscription-auth/pf-42-s01-auth-source-contract.md) | completed and archived |
| `PF-43` (`CSA-02`) | Secure long-lived subscription-token enrollment, storage, validation, replacement, and removal | [CSA-02 / PF-43-S01](../../sprints/archive/claude-subscription-auth/pf-43-s01-managed-token-lifecycle.md) | completed and archived |
| `PF-44` (`CSA-03`) | Platform-authoritative Claude login adapters and deterministic provider resolution | [CSA-03 / PF-44-S01](../../sprints/archive/claude-subscription-auth/pf-44-s01-platform-auth-resolution.md) | completed and archived |
| `PF-45` (`CSA-04`) | Legacy/conflict migration plus failure, recovery, and resume UX | [CSA-04 / PF-45-S01](../../sprints/archive/claude-subscription-auth/pf-45-s01-auth-choice-and-recovery.md) | completed and archived |
| `PF-46` (`CSA-05`) | Final automated, cross-platform-fixture, true-TUI, documentation, and review qualification | [CSA-05 / PF-46-S01](../../sprints/archive/claude-subscription-auth/pf-46-s01-final-qualification.md) | completed and archived; live-account, live-repository, human, and release gates remain plan-level work |
| `PF-47` (`CSA-06`) | First-run Anthropic-account onboarding integrated with the existing explicit Claude authentication choice | [CSA-06 / PF-47-S01](../../sprints/current/claude-subscription-auth/pf-47-s01-first-run-anthropic-account.md) | in progress; corrects the startup integration gap found during human acceptance |

## Hard dependency graph

```text
CSA-01 typed source/selection contract
  ├── CSA-02 managed long-lived token
  └── CSA-03 current Claude store adapters

CSA-02 + CSA-03
  └── CSA-04 migration and recovery UX
        └── CSA-05 final qualification
              └── CSA-06 first-run Anthropic-account onboarding
```

No dependent sprint may become executable before every prerequisite is
completed and archived.

## Acceptance flows

| Flow | Starting state | User action | Expected visible result | Pass criterion |
| --- | --- | --- | --- | --- |
| Recommended success | No Claude Plan method selected | Choose **Stable subscription token — Recommended**, complete authorization, then select a Claude Plan model | Corbanu reports the managed method healthy and the model request succeeds without displaying the token | Token is stored only in an approved secret backend; restart and a real request succeed |
| First-run account enrollment | Fresh Corbanu profile with no provider configured | Choose **Anthropic Claude Account**, then choose the recommended stable token or Claude Code login | Onboarding enters the same Claude authentication flow used by `/providers`, persists `claude-plan` only after a successful choice, and does not require an Anthropic API key | Both account methods, cancel, and restart are proven from the startup screen; API-key enrollment remains a separate explicit option |
| Compatibility success | Healthy platform Claude login | Choose **Use my Claude Code login** | Corbanu reports the platform source and account metadata, then completes a real request | No Corbanu-managed duplicate is created and the selected source persists |
| Existing-user preservation | Existing installation using shared Claude login | Decline the migration offer | Existing source remains selected and usable; prompt can be reopened later | No persistent auth or credential mutation occurs |
| Cancel | Enrollment or migration browser/code flow open | Cancel or press Esc | Flow closes with current source unchanged | Child process exits and no token, partial selection, or store entry remains |
| Enrollment failure | `setup-token` unavailable, times out, returns malformed output, or authorization is denied | Attempt recommended enrollment | Specific safe error and retry/switch actions appear | No output leaks and no unhealthy source is committed |
| Missing refresh token | Selected rotating source contains blank/absent refresh token | Start or retry Claude Plan | Corbanu identifies the selected source as needing reauthorization and offers stable-token setup or Claude login | It does not claim ordinary refresh can recover or invoke an unrelated source |
| Legacy migration | Legacy macOS file or historical source exists beside the current platform source | Review and confirm migration | Source identities and health are shown without values; selected replacement validates before commit | Cancel is inert; no Claude-owned record is deleted or rewritten |
| Conflict | Multiple healthy sources or accounts are detected | Start Claude Plan | Corbanu requires an explicit source/account choice | No priority heuristic silently selects a different identity |
| Revoked/401 recovery | Selected long-lived token is rejected | Retry, replace, or switch method | Corbanu keeps the failed source identified and guides explicit recovery | No fallback rotation occurs; successful replacement resumes the pending user goal safely |
| Restart/resume | A method was selected and validated previously | Restart Corbanu and resume a Claude Plan pane | Same method and metadata return; request succeeds or shows a source-specific recovery state | No raw credential is persisted in rollout state and account/source does not drift |

## Implementation sequence

1. Implement CSA-01 as a product-owned typed source/selection/health contract,
   including persisted choice, metadata-only state, and deterministic conflict
   semantics without changing the existing provider path yet.
2. Implement CSA-02 through the smallest existing vault/keyring/trusted-helper
   seam after verifying the supported `setup-token` CLI contract and output
   handling on all applicable platforms.
3. Implement CSA-03 with platform fixtures and explicit current-versus-legacy
   precedence; keep source discovery separate from mutation or refresh.
4. Implement CSA-04 in the existing Claude login/model-selection TUI seam with
   success, cancel, failure, recovery, and resume behavior.
5. Complete CSA-05 only on the formatted final tree with cross-platform,
   live-repository, true-TUI, human, documentation, upstream, and release proof.
6. Correct the human-acceptance onboarding gap in CSA-06 by routing first-run
   Anthropic-account selection into the already-qualified Claude auth choice,
   then repeat focused, typed-Tmux, and final review qualification.

## Requirement-to-evidence traceability

| Requirement | Owner feature | Required evidence |
| --- | --- | --- |
| Recommended long-lived token choice | CSA-02 / CSA-04 | Enrollment unit tests, masked-output adversarial test, true-TUI primary and cancel flows |
| Explicit Claude Code login alternative | CSA-03 / CSA-04 | Platform status/store fixtures and true-TUI compatibility flow |
| Secure storage and no disclosure | CSA-02 | Vault/keyring contract tests, logs/history/telemetry redaction tests, final artifact inspection |
| No silent source/account switching | CSA-01 / CSA-03 | Conflict, 401, restart, and resume integration tests |
| Reversible legacy migration | CSA-04 | Current/legacy matrix, injected partial failure, cancel, retry, and cleanup proof |
| Cross-platform support | CSA-03 / CSA-05 | macOS Keychain plus Linux/Windows file evidence on final candidate |
| User-visible recovery | CSA-04 / CSA-05 | True-TUI missing-token, revoked-token, replacement, and resumed-request flows |
| First-run Anthropic account | CSA-06 | Onboarding state/snapshot tests plus typed-Tmux recommended, compatibility, cancel, persistence, and restart flows |

## Automated evidence

Run fix and formatting tools before the final affected tests.

| Check | Final-tree command | Result | Artifact |
| --- | --- | --- | --- |
| Plan lifecycle | `python3 docs/plans/check.py` | passed on final documentation tree | PF-46 evidence ledger |
| Sprint lifecycle | `python3 docs/sprints/check.py` | passed before and after PF-46 archive | PF-46 evidence ledger |
| CLI OAuth helper | `CARGO_INCREMENTAL=0 just test -p codex-cli claude_oauth -j 1 --retries 0` | passed 120/120; missing-source recovery and authority drift included | nextest `63905eed-645a-43e8-bf66-7a5b3a842b6f` |
| Vault custody | `CARGO_INCREMENTAL=0 just test -p codex-vault claude_auth -j 1 --retries 0` | passed 21/21; managed token stays zeroizing and generic reveal/export paths remain denied | nextest `0e40f330-2ae9-48ac-bebf-f80427224fa7` |
| External bearer selection | `CARGO_INCREMENTAL=0 just test -p codex-login external_bearer -j 1 --retries 0`; model-provider policy regression; Core provider-auth filter | passed 4/4, 1/1, and 2/2; Claude bypasses the timed cache after a persisted source change while custom providers retain it | nextest `b9fb3e75-7d58-428c-8f0e-996827d7c069`, `d2258b6b-ea30-439a-bb0b-b2a7251e2e16`, `1a46d371-8aea-4c7e-8347-b58cbbb6ae6e` |
| TUI choice/recovery | `CARGO_INCREMENTAL=0 just test -p codex-tui claude_code_login -j 1 --retries 0`; required typed-tmux `tmux_claude_auth` filter | focused 22/22 and typed Tmux 2/2 passed; exact canary checks clean | nextest `f3dc0d72-87b5-42ea-807c-78c12f6a0a9b`, `722fccd6-9297-4bc9-8329-2d53b9d41e19` |
| Adversarial/redaction | Missing, blank, malformed, conflicting, partial-write, timeout, cancellation, oversized output, debug-log, history, and artifact cases | passed affected suites; unterminated login output is bounded before rejection | PF-46 evidence ledger |
| Cross-platform | Platform-store and profile-identity fixtures in the CLI/vault affected suites | macOS Keychain and Linux/Windows credential-file semantics passed on the macOS qualification host | PF-46 evidence ledger; physical Linux/Windows release hosts remain unclaimed |

## True-TUI evidence

Launch through the repository TUI workflow. Send prompt text and Enter as
separate key actions. Corbanu `exec`, menu-only rendering, and mocked login are
supporting evidence only.

| Flow | Candidate binary | Test repo/worktree | Keys/actions | Visible checkpoints | Result | Artifact |
| --- | --- | --- | --- | --- | --- | --- |
| Recommended primary | pending | TensorCash disposable worktree | Choose Claude Plan, recommended method, complete human auth, select model, send a real prompt | Method choice, safe handoff, healthy status, response | pending | pending |
| Compatibility alternative | pending | TensorCash disposable worktree | Choose shared Claude login, select exact source, send a real prompt | Source metadata, warning, response | pending | pending |
| Cancel/failure | pending | Isometric Game disposable worktree | Cancel enrollment; inject unavailable/malformed/blank source | Inert cancel and actionable safe failures | pending | pending |
| Recovery/resume | pending | Both applicable repositories | Reject selected credential, replace/switch explicitly, restart, resume pane | No fallback drift; exact method and pane recover | pending | pending |
| Typed state-machine qualification | `codex-rs/target/debug/corbanu` (`corbanu 0.1.35`, SHA-256 `b709b9eea122b1482cd965de643fbe57b88bdad1a24f217c54b16cb1ddfb65c5`) | Isolated repository and hostile `CODEX_HOME` fixtures at commit `f0d5b0b16` | Text and Enter sent separately; Down, Enter, and Esc sent as typed keys across recommended, compatibility, cancel, setup failure, retry, masked cancel, and restart | Exact selected source remains stable; recommended and compatibility paths persist only after success; harness canary absent from terminal/home/log/artifact surfaces | passed 2/2, retry-disabled, tmux required; compatibility 17.991s, managed 56.285s | nextest `9b9d555f-b3ee-4b7f-8e8a-f7182a33cf8d`; successful run emitted no failure bundle under `.codex-work/claude-subscription-auth/f0d5b0b1-final-qualification/tmux` |

## Live-repository applicability

| Repository | Applicable to this initiative? | Resolved checkout/test worktree | Base commit | Reason or result |
| --- | --- | --- | --- | --- |
| TensorCash | yes | pending environment resolution | pending | Systems/provider workflow proves real Claude Plan enrollment, request, failure, and resumed work in a substantial codebase. |
| Isometric Game | yes | pending environment resolution | pending | Interactive/model-selection workflow proves readable choice, cancel, recovery, and pane resume in a visual codebase. |

## Human acceptance

| Tester | Date | Candidate version/commit | Flow | Result | Evidence |
| --- | --- | --- | --- | --- | --- |
| Named tester pending | pending | pending | Recommended enrollment, compatibility choice, cancel, failure, migration, replacement, restart, and resume | pending | pending |

## Documentation

| Finished-feature doc | Product-spec citation present | Verified candidate |
| --- | --- | --- |
| [Reliable Claude Plan authentication](../../features/claude-plan-authentication.md) | yes — **Shipping MVP — LIVE** vault and multi-provider requirements | verified on PF-46 automated candidate; live Anthropic account acceptance remains pending |

The finished guide must begin with repeated-login pain, explain both choices in
plain language, identify subscription eligibility and token limitations, cover
secure replacement/removal, and describe platform/legacy recovery without
documenting raw-secret inspection commands.

## Performance and benchmark state

This initiative does not change model quality. CSA-05 must record startup,
source-check, and provider-auth latency before release and confirm that no
interactive auth subprocess runs on a healthy steady-state request. The target
release still follows the benchmark cadence and ledger in `benchmarks/README.md`;
due benchmark work is neither added nor waived by this plan.

## Dependencies, decisions, and blockers

| Item | Type | Owner | Needed by | State / decision |
| --- | --- | --- | --- | --- |
| Product authority and WIP slot | product | Travis Good | activation | decided 2026-08-30; slot 2 of 2 allocated |
| Exact upstream Codex baseline | integration | Jim Ricketts | first sprint readiness | resolved 2026-08-30: fetched `upstream/main` `b7cd519c767c8fd4bc3581d9bc92fbab37a768c1`; verified fork merge-base `413492cd6c3a4d4f8dff6f406247ccda5a9d88aa` |
| Current official `claude setup-token` CLI/output contract and supported version floor | external compatibility | Jim Ricketts | CSA-02 readiness | verified 2026-08-30 against `https://code.claude.com/docs/en/authentication`: one-year, model-request-only token for Pro, Max, Team, or Enterprise; command prints but does not save it. Local Claude Code 2.1.92 advertises `setup-token`. |
| Secure storage seam and protected-mode composition | architecture/security | Jim Ricketts | CSA-01/CSA-02 readiness | extend existing vault/keyring/broker boundaries; do not create a parallel plaintext store |
| Long-lived token replacement/revocation semantics | external compatibility | Jim Ricketts | CSA-02/CSA-04 readiness | verified documented replacement requires a new token and restart; Corbanu local removal is explicitly not server revocation |
| Exact platform credential naming and `CLAUDE_CONFIG_DIR` behavior | compatibility | Jim Ricketts | CSA-03 readiness | reverified 2026-08-30 against installed Claude Code 2.1.92 bundle SHA-256 `6b0b860206b3723d70619b84dbf3a53a795d703862aa3b01d58e869685c85362`: macOS uses `Claude Code-credentials` (or `Claude Code-custom-oauth-credentials`) and appends `-{sha256(NFC(config_dir))[..8]}` whenever `CLAUDE_CONFIG_DIR` is set; Linux and Windows use `${CLAUDE_CONFIG_DIR:-~/.claude}/.credentials.json`. |
| Release target and named human tester | release | release owner | release | pending; not fabricated by PF-46 |

## Release linkage

- Release record: `qa/release/<version>/` — target version pending.
- Benchmark tracker row: `benchmarks/README.md` when due for the target release.
- Remaining blockers: named human acceptance with a live eligible Anthropic
  account, TensorCash and Isometric Game live-repository runs, physical
  Linux/Windows release-host confirmation, target release/tag/merge decision,
  release ledger, and any due release benchmark entry.

## Completion

- [x] Product linkage, scope, invariants, and worktree coordinates are current.
- [x] Every planned implementation unit has one stable plan feature ID and hard dependencies.
- [x] Every implementation unit is represented by a valid completed single-feature sprint.
- [x] Required final-tree automated and adversarial evidence passes.
- [x] macOS Keychain plus Linux/Windows credential-file behavior passes in final-candidate platform fixtures.
- [ ] True-TUI and both live-repository workflows pass.
- [x] Automated canary and custody checks find no raw credential in logs, chat, history, telemetry, artifacts, or ordinary config.
- [ ] Named human acceptance passes.
- [x] Finished documentation matches the automated candidate.
- [ ] Upstream baseline/adapter disposition, release record, and due benchmark state are linked.
- [ ] No hard release gate remains pending.
