# Provider UX parity audit

Date: 2026-09-04 UTC

Candidate: uncommitted correction on `integration/reconcile-release-0.1.37`,
based on `f03e95f7a65609bb442764d6306682d5fe43f6bb`

Release authorization: none. This is implementation and qualification evidence,
not permission to tag or publish a release.

## Product linkage

- Exact product-spec heading: **Shipping MVP — LIVE**
- Requirement excerpt: “Multi-provider inference — OpenAI,
  Anthropic/Claude Plan, Kimi, Z.AI, DeepSeek, OpenRouter, Ambient, Meta,
  Baseten, Vercel, Bedrock, Ollama, LM Studio, Corbanu Plan, and custom
  providers.”
- Requirement excerpt: “Encrypted `/vault`, masked entry, metadata-only
  inspection, and operational credential use without placing raw values in
  chat.”
- Active-plan invariant: “Onboarding and `/providers` consume the same catalog
  and status service and dispatch the same typed auth-flow controller actions.”
- Active-plan invariant: “Host renderers may differ, but authentication success,
  failure, cancellation, recovery, timeout, stale-result rejection, and
  persistence semantics may not.”

## Initial instruction correction

The `/providers` Claude Account route now presents the established Claude Plan
method choice and managed-token instructions. The long-lived-token choice is
recommended, explains that `claude setup-token` must be run in a private
terminal, identifies the approximately one-year lifetime and eligible account
types, and retains the no-fallback/account-change-on-success guidance. The
masked token-entry screen now also tells the user to run `claude setup-token`.

The established Claude screen and the shared provider screen obtain this copy
from one presentation module, preventing the same strings from drifting again.
Two rendered snapshots protect the method picker and masked token entry. The
PF-52 true-TMUX journey now requires the instructions to be visibly present.

## Original parity matrix (before the combined correction)

| Surface or behavior | Evidence | Result |
| --- | --- | --- |
| Shared built-in and custom provider catalog/status | PF-54 true-TMUX catalog/status journey | pass |
| API-key masked setup, success, recovery, and cancel | PF-50 true-TMUX journey | pass, copy gap below |
| OpenAI account start, device-code challenge, cancel, and retry | PF-51 true-TMUX journey | pass, affordance gap below |
| Claude recovery, cancel, retry, managed-token instructions, and secret redaction | Updated PF-52 true-TMUX journey plus two snapshots | pass |
| Non-current deactivate/reactivate, model-picker eligibility, restart, and credential reuse | PF-54 true-TMUX journey | pass |
| Current-provider deactivation cancellation | PF-54 true-TMUX journey | pass |
| Exact replacement before current-provider deactivation and restart | PF-54 true-TMUX journey | pass |
| Environment-backed provider copy, deactivate/reactivate, and no deletion | PF-54 true-TMUX journey | pass |
| Multi-provider onboarding, first-provider default, restart, and request | Existing final-tree PF-53 evidence | pass on candidate base; initial copy-only correction did not touch this flow |
| Deferred Corbanu success/cancel/only-provider return behavior | Existing final-tree PF-53 evidence | pass on candidate base; initial copy-only correction did not touch this flow |

## Follow-up corrections (all six authorized together)

The findings below describe the original regression and its correction. Final
qualification results are recorded at the end; the earlier matrix above is
historical evidence, not a substitute for the new run.

### 1. Account-auth failures lose their visible reason — high priority

`ProviderAccountAuthHost` collapses typed OpenAI and Claude `Failed`, `Blocked`,
and OpenAI `RecoveryRequired` snapshots into a reason-free `Failed`
presentation. Both the onboarding host and `/providers` then dismiss or redraw
without rendering the reason or a recovery action. This conflicts with the
active-plan requirement that unsupported or ambiguous auth sources fail
visibly with recovery guidance.

Implemented: allowlisted messages preserve each typed failure/blocked/recovery
reason without displaying raw backend errors. Settled failures expose Retry
and Back; Escape returns to Providers. OpenAI retries receive a fresh attempt
id and do not bypass in-flight outcome reconciliation. Claude Code retry
resubmits its login action. The failure header wraps its complete guidance.
Deterministic TMUX cases now reject OpenAI login start and a synthetic Claude
token, then exercise recovery with actual keypresses.

### 2. Masked secret-entry guidance can be clipped — medium priority

`VaultSecretEntryView::desired_height` sizes the empty text area without
accounting for a wrapped placeholder. At narrower terminal widths, the new
Claude guidance visibly stops after the first row. The essential
`claude setup-token` instruction remains visible, but the privacy assurance can
be truncated.

Implemented: wrapped placeholder rows participate in height calculation,
remain capped at ten rows, and rendering is intersected with the viewport.
Snapshots cover 40, 76, and 120 columns and require the privacy instruction's
last words to remain visible.

### 3. OpenAI challenge loses established browser affordances — medium priority

The shared OpenAI challenge prints `Open:` and the device code but does not mark
the URL as an OSC 8 hyperlink. It also omits the established remote/headless
guidance that explains when to use device-code login. Authentication semantics
and cancellation pass; discoverability and clickability do not have UX parity.

Implemented: a wrapped challenge header marks the full URL as one OSC 8
hyperlink and shares browser/device guidance with onboarding. A narrow
snapshot asserts the full hyperlink target; TMUX checks remote/headless copy.

### 4. API-key entry copy is less specific than established onboarding — medium priority

The shared onboarding and `/providers` masked entry says only “Stored through
the selected provider credential backend.” Established onboarding says the key
is stored in the vault, while the retired credentials screen named the exact
environment variable and explicitly said the key is not stored in chat. The
storage behavior is correct, but the user receives weaker custody and recovery
information.

Implemented: onboarding and both provider entry routes derive guidance from
the same typed storage descriptor. Environment-backed keys name the exact
variable, encrypted vault, no-chat policy, and environment precedence. OpenAI
API keys correctly describe the configured OpenAI credential store rather
than incorrectly promising vault storage.

### 5. Onboarding still owns duplicate provider-auth presentation copy — medium priority

The fresh-install onboarding renderer hard-codes its own Claude method picker,
managed-token screen, API-key screen, and browser guidance. It currently gives
enough information to complete those flows, but it does not consume the new
shared Claude presentation module. This duplication is the same class of drift
that caused the `/providers` regression.

Implemented: onboarding retains its layout but consumes shared Claude labels,
method descriptions, token instructions, API-key guidance, browser/device
guidance, and safe account-failure messages. No credential backend is replaced.

### 6. macOS TMUX fixtures are not isolated from the login keychain — high priority

True-TMUX fixtures use temporary `CODEX_HOME`, `CORBANU_HOME`, and
`PFTERMINAL_HOME` directories, but spawned test binaries can still query the
real macOS login keychain. Repeated provider-management runs produced
interactive keychain prompts, and interrupted runs left multiple TMUX servers
and child `codex` processes alive. Rebuilding and ad-hoc signing the fixture
binary also changes the executable identity macOS uses when applying an
“Always Allow” decision.

Implemented: harness commands enable `CORBANU_TEST_NO_NATIVE_KEYRING=1` for
debug fixtures. Native keyring load/save/delete and implicit Claude macOS
keychain probes stop before accessing the OS; existing profile-local fallback
storage remains available. Release builds ignore this test override. Tests
verify all three native operations are rejected in an isolated subprocess.
A pipe-EOF watchdog cleans up each private TMUX server when a test process is
killed; normal cleanup also terminates verified pane process groups. Eight
harness tests pass, including a SIGKILLed parent with a pane that ignores HUP
and TERM. This does not disable or weaken the user's production keychain ACLs;
a newly built executable may still legitimately require a first approval.

## Explicit non-finding

Managed credential deletion is not present in `/providers`. PF-54 explicitly
excluded credential deletion and requires deactivation to retain credentials,
so this audit does not classify its absence as a regression. If deletion is
desired, it needs a separate product decision and destructive-action design.

## Additional pre-existing privacy observation

During failure diagnosis, `TextArea` debug logging recorded individual
`KeyEvent::Char` values from a synthetic canary. Matching/redacting the complete
canary string does not prove those individual-character logs safe. No real
credential was entered in these fixtures. This logging predates this correction
and needs a separately scoped protected-data-disclosure fix under the repository
policy. Until then, do not enable debug/trace key-event logging while entering
real credentials; human acceptance should use normal launcher logging.

## Additional installed-app observation

The final Apps-launcher smoke created Terminal window 10763, titled
`Corbanu — provider UX retest`, using the verified 0.1.38 executable. It stopped
at an update prompt offering 0.1.37. Inspection of
`tui/src/updates.rs::revalidated_upgrade_version` shows the pre-existing recall
heuristic treats a missing installed-version release tag as a recalled release,
even for this unpublished candidate. This is not a stale-cache explanation.
The six provider fixes do not change that heuristic. Choose **2. Skip**, not
Update now, to preserve the acceptance candidate. Computer Use denied Terminal
access, so the prompt's dismissal and subsequent human acceptance remain for
the user; launch is verified, but reaching the interactive chat is not claimed.

## Commands and artifacts

- Rendered snapshots: `codex-rs/tui/src/chatwidget/snapshots/`
- TMUX artifacts: `/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-parity-success-artifacts/`
- Focused snapshots: `just test -p codex-tui -E 'test(shared_claude)'`
- Provider-management matrix:
  `CORBANU_TMUX_REQUIRED=1 just test -p codex-tui --test all -E 'test(/provider_management::/)'`

The corrected PF-52 journey passed independently. A fresh-binary matrix run
passed the other nine provider-management tests, while PF-52 timed out under
parallel contention; the run was stopped after the macOS keychain-isolation
defect above surfaced. No production credential was deleted or modified.

## Combined correction qualification

- `just fix -p codex-cli -p codex-tui -p codex-keyring-store -p codex-provider-auth`,
  `just fmt`, and `git diff --check` passed.
- The final focused unit/component run passed **253/253** tests (provider-auth,
  keyring isolation, CLI Claude OAuth, onboarding, provider presentation, and
  masked entry). Snapshots were inspected and accepted; the final run did not
  use automatic snapshot updates.
- The harness cleanup run passed **8/8**, including forced parent termination,
  panic unwind, server isolation, and artifact redaction.
- An initial TMUX run waited for the retired `Paste or type your API key below.`
  wording after the new shared guidance was already visible. A diagnostic run
  captured this mismatch; onboarding and convergence assertions now require
  the new no-chat privacy guidance. No production credential was involved.
- Post-review UI qualification passed **50/50**, including the new numbered
  device-code snapshot, without snapshot-update flags.
- Fable 5.1 Max through Corbanu/TMUX: full review found no correctness blockers
  and two P3 issues (device-code numbering and stale release wording). Both
  were fixed. A follow-up raced snapshot generation and reported a missing
  artifact; the accepted snapshot and default test run resolved that finding.
  The final artifact-resolution review exited **0**, with no actionable
  findings (`provider-ux-review/final.txt`). Subsequent testing found and separately
  reviewed the submission-view correction below.
- The native test installation now includes both wallet-daemon names next to
  the terminal executable; the first expanded run had omitted this prerequisite.
- Native retest build: the final submission-view correction built successfully
  in 7m 37s; `--version` reports `0.1.38`, and `codesign --verify --verbose`
  passes. The stable launcher link is unchanged. The release record contains
  the verified executable hash. No production credentials or keychain ACLs
  were changed. This build timing was not a controlled benchmark.
- The expanded matrix passed 32/33 cases initially. The remaining Claude case
  revealed that a whitespace-free dummy token was valid under the existing
  local format check, and `SetManagedToken` reopened a blank entry over successful
  completion. Presentation now opens entry only when `has_input` is false.
  The corrected case uses embedded whitespace for rejection, then also checks
  successful local storage returns to the manager without a ghost form.
  The final targeted case passed without retries in 31.490 seconds (nextest
  `a9cfce64-49c5-43ea-8396-6e6f40ccccca`). All 33 selected cases therefore have
  passing coverage across the matrix and targeted reruns, not one clean matrix
  run. The environment-convergence case needed one input-timing retry in the
  matrix and also passed a separate no-retry run in 31.299 seconds.
  The human checklist uses a spaced invalid value so it cannot overwrite a
  saved token.
- The final submission correction passed 38/38 focused UI tests; nextest
  flagged one animation test as leaky during that run. The same test passed
  independently without a leak warning or retries (nextest
  `276143a7-b8e9-4055-9800-6c55f6f51f17`, 0.216 seconds).
  Fable 5.1 Max reviewed the submission guard, regression coverage, and safe
  human-test instructions through Corbanu/TMUX with no findings and exit 0
  (`provider-ux-review/submission.txt`).

Local logs are under `/Volumes/CorbanuDrive/Corbanu/.codex-work/`:
`provider-parity-final-tests.log`, `provider-parity-cleanup-tests.log`,
`provider-parity-tmux-complete.log`, `provider-parity-submission-tmux.log`,
`provider-parity-submission-unit.log`, `provider-parity-tmux-artifacts/`, and
`provider-ux-review/`. They are operational artifacts, not committed secrets.

Human acceptance remains pending. These bounded fixes restore the existing
Shipping MVP behavior; they do not create a new sprint or authorize release.
The candidate's cross-platform/live-repository and benchmark gates remain as
disclosed in the release-candidate record.
