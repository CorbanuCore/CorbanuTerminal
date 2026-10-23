---
title: "Named accounts (PF-84): flag removal"
status: draft
change_class: product-initiative
priority: P1
owner: "Codex PF-84 lane owner (proposed); Travis Good accepts"
activation_authority: "Travis Good"
activation_basis: "Carried from the completed provider-onboarding plan on 2026-10-14; activation pending a free slot, the test accounts and his decision"
target_release: "TBD"
deadline: "TBD"
created: 2026-10-14
updated: 2026-10-20
---

# Named accounts (PF-84): flag removal

Draft. Predecessor: [unified provider onboarding](../completed/main-2026-10-14-unified-provider-auth.md), closed
2026-10-14 to free a slot for the PF-60 follow-ups. PF-84-S01–S06 are accepted and archived; named accounts stay
behind the default-off `named_accounts` flag. This plan authorizes no implementation until Travis activates it.

## Carried forward (Travis, 2026-10-13, #720)

Before the flag is removed:

1. A live ChatGPT check with a disposable test ChatGPT account (sign-in, refresh written back to that account,
   default login untouched).
2. A live Bedrock check with a test AWS account and a least-privilege profile.
3. The S03 limits #425, #426 and #428 (fixed by #431 and #432) re-verified.
4. The partial #555 legs re-run (ESC-03, REG-01, GH-02, PR-01, ESC-04, ESC-01, the D3 approvals-off leg); REG-02
   decided.
5. Open follow-ups closed or documented: #588, #609, #689 (bug fixes; they need no active slot).
6. An isolated code-blind run from a frozen design on the packaged candidate, then Travis's sign-off.

Also open from the predecessor: PF-58-S01 (draft residual qualification), named human acceptance and release linkage.

Owner intent (Travis, 2026-10-20, in the coordinator's brief for this pass): ship multiple credentials per provider;
turn the flag on in the sprint after the local-model QA. Where a model has several connections, the OpenAI-style one
stays the default.

## Prerequisite status (2026-10-20, against `main` `8508e4d540`)

"Fixed: #N" means fixed by that PR; the PRs were open when this was written (#1069 merged the same day).

| # | Prerequisite | Status | Evidence / what remains |
| --- | --- | --- | --- |
| 1 | Live ChatGPT check | Needs Travis | Needs a disposable ChatGPT test account. Not run. |
| 2 | Live Bedrock check | Needs Travis | Needs a test AWS account and a least-privilege Bedrock profile. Not run. |
| 3 | #425, #426, #428 re-verified | Done by tests; two #426 legs left for item 6 | On current `main`: the #425 thread-start and exec refusals and the #428 D3 tests (`core`, incl. approvals-off refusal), and the TUI's #555 tmux suite pass. #426 had no end-to-end test; `tmux_a_missing_launch_account_exits_before_onboarding` (added with this table) checks that a new session with `--account ghost` or a missing `[provider_accounts]` entry exits with the recovery text before onboarding. #426's `resume --account ghost` leg has a unit test only, and its custom-provider leg none: re-run both in item 6. |
| 4 | Partial #555 legs | Open: re-run in item 6; REG-02 needs Travis | ESC-03 draft, REG-01 flag-off Esc, GH-02 not naming `ghost`, PR-01 `/status`, ESC-04 before-build comparison, ESC-01 terminal restore are evidence gaps, not known defects: re-run them in the code-blind run. D3's approvals-off leg cannot be reached from outside (Aggressive keeps approvals on); unit tests cover it, which Travis would need to accept in place of a re-run. REG-02 (masked key fragment in `login status`) needs a decision. |
| 5a | #689 | Fixed: #1067 | Core-started spawns on a connection with no credential of its own are refused up front. Follow-up #1074 (resumed agents), low. |
| 5b | #588 | Flag source fixed: #1076; rest documented | A fork's own config decides. Still documented: a missing `[provider_accounts]` entry refuses `corbanu fork` at launch; `resume --last` filters by the configured provider (upstream). |
| 5c | #609 | Items 1–4 fixed (#703, #715); row attribution: contributor draft PR #979 | Still documented: `forced_chatgpt_workspace_id` is checked at use; the `/model` OpenAI subtitle is static. `status_accounts()` still lacks tests for command-auth/Bedrock entries and at App level (#979 adds single-connection and shared-entry ones). |
| 5d | #694 (Windows) | Fixed: #1069 | Test-side only: the Windows failure was the test's canary scan reading a held lock file, not the vault. The owner's Windows gate run is still to do before "supported on Windows". |
| 6 | Isolated code-blind run + sign-off | Needs Travis | From a frozen design on the packaged candidate (best run after items 1–2, so it covers them). Draft flag-removal PR #1080 prepared, not merged. |

## Decisions for Travis

- Activation and slot, once the test accounts exist (items 1–2 are blocked on them; mock-only testing was accepted
  for S06 on 2026-10-13).
