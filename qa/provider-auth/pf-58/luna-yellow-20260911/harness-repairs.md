# Two-lane pilot repairs — 11 September 2026

Routine QA/process experiment; no product or launcher code changed. Product
context: **Shipping MVP — LIVE**, “Multi-provider inference” and “Encrypted
`/vault`, masked entry, metadata-only inspection.” This does not adopt the pilot
as canonical policy, rebase the branch, or assert release readiness.

## Execution changes

- Start credential setup/recovery at `/providers`; `/model` is for model choice.
- Preserve complete literal test prompts, sending text and Enter separately.
- Inspect prerequisites before exploration; record missing fixtures promptly.
- Save progress and a blocked report before optional observations where a
  required prerequisite is absent.
- Preserve original attempts and distinct replay records.
- Future scratch profiles map `work/evidence` to their own evidence directory
  so relative artifact writes cannot silently strand the final report. This
  remains inside the same filesystem sandbox; existing artifacts are untouched.
- Treat process timeouts as timeouts even when a partial report exists. Do not
  show them as pending or passes; do not link to nonexistent final reports.
- Run desktop launch tests through a separate fresh-context Luna Max agent
  using Computer Use. That agent has instruction-only code blindness. It does
  not inherit the CLI lane's filesystem-isolation assurance.

The frozen human expectations are unchanged. Replays are for cases 5 and 15;
the new native attempt covers cases 1 and 12. No extra independent review was
commissioned; these are user-authorized test execution runs.

## Remaining fixture boundaries

| Condition | Current fixture | Disposition |
| --- | --- | --- |
| OpenAI and Claude ordinary requests | Private existing-profile clones | Exercise live through TUI; do not assume validity |
| API-key instructions | Unconfigured provider entry screen | Can inspect and cancel without a credential |
| Expired OpenAI token and successful reauthentication | Existing credential currently worked; no expired fixture | Full case 24 remains blocked; no new browser login requested |
| Environment, external command/AWS, and local model recovery | Not provisioned in these clones | Full case 26 remains blocked; no claim from unrelated account menus |
| Applications launcher/Desktop 3 | Real Mac UI, new native lane | Await actual native observations |
| Native Keychain prompts | CLI clones deliberately avoid native Keychain | Cannot infer a native pass from CLI results |

## Checks

Five focused collector regression checks pass: preserving original/replay
identities, timeout precedence, failed-process handling, complete result
preservation, and basic credential/code/email redaction. Coordinator scripts
and tests are in the external-drive pilot directory
`/Volumes/CorbanuDrive/Corbanu/.codex-work/provider-reauth-health/luna-yellow-20260911/`.
This is not an independent evidence review or a product regression suite pass.

Live profiles are tested with warning-level logs, not full tracing; the exact
existing signed package is used, not rebuilt with `just codex`. These safety and
candidate-provenance constraints override the generic TUI skill defaults.
