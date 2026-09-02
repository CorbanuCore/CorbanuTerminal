# PF-56 Kimi finding disposition

Status: `NOT CLEAN`; production repair is blocked on an exact PF-56 scope
expansion. No production file was edited during disposition.

| Finding | Initial disposition | Required next action |
| --- | --- | --- |
| High: startup command authorization is applied after the policy snapshots statuses | Accepted pending focused reproduction | Add the typed authorization before/snapshot-refresh regression and repair the shared policy construction boundary. |
| High: `OutcomeUnknown + Replace + Configured` can remain unsettled | Accepted pending focused reducer reproduction | Add a deterministic reducer regression and a bounded terminal transition; preserve correlation and custody. |
| Medium: deferred Corbanu no-fallback auto-selection uses stale policy | Accepted pending integrated reproduction | Refresh the shared policy before exact in-session selection and add no-fallback TMUX coverage. |
| Medium: manager eligibility mutation does not update model-picker policy | Accepted pending focused/integrated reproduction | Refresh the shared model policy after applied eligibility mutation and prove deactivate/reactivate visibility in one session. |

Exact production paths named by the findings and outside the current PF-56
write scope are:

- `codex-rs/tui/src/startup_provider.rs`
- `codex-rs/tui/src/chatwidget/provider_model_policy.rs`
- `codex-rs/provider-auth/src/auth_flow.rs`
- `codex-rs/provider-auth/src/auth_flow_tests.rs`
- `codex-rs/tui/src/app/event_dispatch.rs`
- `codex-rs/tui/src/app/provider_management_status.rs`
- corresponding already-scoped PF-56 TMUX suite files for integrated regressions

The repair should be authorized as one bounded final-review remediation scope.
The reviewer explicitly found no regex-dependent provider path or raw credential
crossing in the inspected boundaries; that positive observation does not waive
the four actionable state-convergence findings.

