# #739: AGG-T01/T02 replay under Permissive with protected launch

Owner decision (Travis, 2026-10-14, security sign-off for option B): with the
security features on, Permissive stops offering runs outside the sandbox that
protected launch refuses, says so plainly, and explains remembered rules.

- Candidate: debug `corbanu` from PR #813 at `742b6e348b` (the last screen,
  `T02-step1-review-final-head`, at `643a2bdc91`, whose only change after
  `742b6e348b` is that review row).
- Model: GLM 5.3 Flash on `zai`. Disposable home with the "Aggressive ships"
  milestone flags (`security_levels`, `protected_mode_preflight`,
  `secretless_agent_launch`, `isolated_credential_broker`, `secret_output_gate`,
  `source_envelopes`, `url_destination_policy`, `contained_external_agents`),
  starting at Permissive; no native keyring. The fixture (`ledger-notes`
  project, `../outside-notes.txt` with one line) is outside the temporary
  folders, as in the VM run.
- Screens are in `captures/` (run directory shown as `<run>`). Times are UTC on
  2026-10-15.

## AGG-T01

| Step | Observed | Capture |
| --- | --- | --- |
| S0 | N = 1. `/security`: Active Permissive; the summary says protected launch is on and agent commands never run outside the sandbox, so that isn't offered. | `T01-S0-security` |
| 1 | No approval prompt. The command ran in the sandbox: `Operation not permitted`. The model did not ask to leave the sandbox. | `T01-step1` |
| 2 | Not exercisable: no prompt, so no "don't ask again" option (the intended change). | — |
| 3 | No prompt; same sandbox denial. | `T01-step3` |
| extra | Asked explicitly for `require_escalated`: no prompt; the warning "Not run: this command asked to run outside the OS sandbox, and that isn't possible here…" is shown to the person. | `T01-escalation-probe` |
| 4 | Aggressive review: "Sandbox now: … with protected launch (secretless_agent_launch) no command runs outside the sandbox, so none is offered"; vault row unreadable (secretless_agent_launch). Saved, `r`, Active Aggressive. | `T01-step4-*` |
| 5–6 | Aggressive prompt appears, no Ran/approved line before answering; Esc: canceled / Did not run / Approval declined. No suspended-rule note: no rule was saved at Permissive. | `T01-step5-aggr-prompt`, `T01-step6-esc` |
| 7 | After `/quit` and relaunch: the same prompt and Esc result. | `T01-step7-*` |
| 8 | `wc -l` = 1 (N). | — |

## AGG-T02

| Step | Observed | Capture |
| --- | --- | --- |
| 1 | Permissive review lists the removed Aggressive rows, keeps Corbanu's own files unreadable under protected launch, and says no approval to run outside the sandbox is offered. Saved; "Restart now" not available (broker holds the key), so `/quit` and relaunch; Active Permissive. | `T02-step1-*`, `T02-permissive-active` |
| 2 | No prompt; sandbox denial; the model says protected launch keeps commands in the sandbox. | `T02-step2` |
| 3 | `wc -l` = 1. | — |
| extra | With a remembered allow rule for `touch` (seeded in the home, then relaunched): no prompt; the note "A remembered approval or allow rule would run this command outside the OS sandbox, which isn't possible here…" once; `Operation not permitted`. Again: no second note. `../outside-touched.txt` not created. | `T02b-rule-1`, `T02b-rule-2` |

## Expected-result amendments these cases need

AGG-T01 and AGG-T02 expect the Permissive writes outside the project to run
(N+2, N+3 lines). Under option B they can't: with protected launch on, nothing
runs outside the sandbox at Permissive. The cases should expect no
out-of-sandbox approval at Permissive, the plain refusal or sandbox denial, the
file unchanged (N), and, for a remembered rule, the note. The #626 part
(remembered approvals suspended under Aggressive) needs a rule saved under a
less strict level, which protected launch at Permissive no longer creates for
these commands.
