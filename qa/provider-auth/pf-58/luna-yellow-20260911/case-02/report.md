# Case 2 — Verify startup identity and remembered setup

Status: **BLOCKED**

## Summary

The supplied candidate reached a responsive chat composer after the visible directory-trust prompt. It started on `Claude Fable 5.1 Plan` through `Claude Plan`; `/status` confirmed the same provider/model and a connected account. No provider onboarding, `codex_apps` startup warning, MCP interruption, UI stall, or update-to-0.1.37 message was observed. The case remains blocked because the harness disables startup update checks and the allowed visible startup/status flow did not provide an unambiguous direct check for Code Mode availability.

## Observed checks

| Check | Result | Evidence |
|---|---|---|
| Startup reached chat without forcing `/providers` | Passed | [01_startup_composer.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-02/evidence/01_startup_composer.txt) |
| Current provider/model visible and consistent with startup selection | Passed | [02_status.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-02/evidence/02_status.txt) |
| Code Mode available | Blocked / unconfirmed | [03_visible_slash_commands.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-02/evidence/03_visible_slash_commands.txt) |
| No `codex_apps` startup warning | Passed | [01_startup_composer.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-02/evidence/01_startup_composer.txt) |
| No MCP startup interruption | Passed | [01_startup_composer.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-02/evidence/01_startup_composer.txt), [02_status.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-02/evidence/02_status.txt) |
| No update offer to 0.1.37 | Blocked / untestable in this harness | [01_startup_composer.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-02/evidence/01_startup_composer.txt) |
| UI remained responsive | Passed | [02_status.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-02/evidence/02_status.txt), [04_clean_composer.txt](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-02/evidence/04_clean_composer.txt) |

No failed observable expectation was seen.

## Actions

The exact supplied launch command was used in the dedicated `target` TMUX session. I selected the visible `1. Yes, continue` directory-trust option with Enter, waited for the composer, and submitted `/status` followed by Enter. For the Code Mode check only, I submitted read-only `/help` (the UI reported it as unrecognized), then typed `/` as directed by the visible hint; no configuration command was submitted. I cleared the composer with Ctrl-C and exited only the target session with Ctrl-D. The key/action history is in [actions.log](/Volumes/CorbanuDrive/Corbanu/.codex-work/luna-0911/case-02/evidence/actions.log).

## Blockers and limitations

- The harness states that startup update checks are disabled in the cloned profile. Therefore, the absence of an update notice cannot establish the required version-comparison behavior.
- The startup/status UI did not expose a definitive Code Mode availability checkpoint. The visible slash-command list did not contain a Code Mode entry, but that alone does not prove that the Code Mode host is missing.
- No model request, provider configuration, credential inspection, browser/native access, or external service interaction was performed. The connected-account line is UI metadata only.
- This execution is bounded to the supplied macOS arm64 candidate; no Linux or other-platform result is inferred.

## Test-instruction ambiguity

The instruction requires “Code Mode is available” but does not specify an allowed visible control or observable checkpoint for verifying it. It also provides no before-run UI baseline for independently proving the word “remembered”; the run only observes the preconfigured provider/model shown at startup and by `/status`.

Candidate SHA-256: `4a8eba7b10199ea49aee42a720687b63cb4b1b2194c2ce164d1b9f6f88ee510e`

