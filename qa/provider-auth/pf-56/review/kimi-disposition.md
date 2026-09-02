# PF-56 Kimi finding disposition

Status: `CLEAN AFTER DISPOSITION` on candidate
`c1a444f2f882cca9c2739dd27f2a692f284a1e498f299bdb1683a10e06d807a0`.

The user-authorized Kimi 3.0 high review through Vercel was the sole completed
external review. The earlier Claude Fable process failed OAuth before inference
and remains superseded evidence.

| Finding | Reproduction | Final disposition | Verification |
| --- | --- | --- | --- |
| High: startup command authorization is applied after policy status snapshot | Not reproduced. The startup path installs the typed runtime authorization snapshot on the shared status host before constructing `ProviderModelPolicy`. | Rejected as a false positive; no production change for this finding. | Startup/policy focused regressions passed, including authorized, rejected, unchecked, grouped, stale, exact-identity, and redaction cases. |
| High: `OutcomeUnknown + Replace + Configured` can remain unsettled | Reproduced in the provider-auth reducer and effect executor. | Confirmed and fixed. A second correlated timeout reaches typed `Failed`, and the executor returns that correlated terminal settlement without moving a secret into App events or debug output. | `codex-provider-auth` 62/62 plus executor focused tests passed. |
| Medium: deferred Corbanu no-fallback auto-selection uses stale policy | Reproduced by the strengthened no-fallback onboarding journey. | Confirmed and fixed. The shared provider policy is refreshed before exact in-session selection; cancellation still returns without fallback or silent current-provider mutation. | Final true-TMUX deferred/fresh-plan journeys passed on the recorded candidate. |
| Medium: manager eligibility mutation does not update model-picker policy | Reproduced for same-session custom-provider reactivation. | Confirmed and fixed. Successful manager persistence performs bounded idempotent exact-runtime catalog synchronization before refreshing shared policy; manager open reuses the shared status authority. | Model-catalog/manager focused tests and final deactivate/reactivate/restart/request true-TMUX journeys passed. |

Additional qualification exposed an adjacent exact-identity bug not asserted by
the review: login-status OpenAI metadata could be applied while a custom provider
was current. The final repair scopes that metadata to the exact OpenAI runtime.
The adjacent custom-provider/OpenAI identity regression, startup suite, provider
status suite, and final PF-51 journey all pass.

The reviewer found no regex-dependent provider routing and no raw credential
crossing in the inspected boundaries. Final accepted TMUX bundles and source/doc
scans contain no synthetic credential canaries. Failed diagnostic bundles are
retained separately and are not cited as accepted evidence.
