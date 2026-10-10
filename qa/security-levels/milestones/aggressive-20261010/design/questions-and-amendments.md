# Designer questions, coordinator answers and amendments

Written by the integrator before any case ran. Answers come from the product requirements
(`docs/corbanu-product-spec.md`, **P0 `/security` levels**; `docs/config.md`) and the test environment, not
from implementation. No original expectation is changed: where there is no further requirement the designer's
own assumption stands.

| Q | Topic | Answer |
| --- | --- | --- |
| 1 | Approval shortcuts | No requirement beyond the brief: typed text must never approve. Assumption stands. |
| 2 | "Don't ask again" under Aggressive | No further requirement. Assumption stands. |
| 3 | Reading outside the project | No further requirement; writes are limited to the project folder. Assumption stands. |
| 4 | What applies before restart | The screens are the product's statement. Assumption stands. |
| 5 | Making the broker fail | Test fixture: route `zai-plain-http`, the same Z.AI models with the provider address given as plain HTTP. The product docs state such a URL cannot be brokered and its requests are refused, the same refuse-and-explain path as a broker that cannot start. No user action stops the broker process itself. |
| 6 | Network refusal wording | Requirement: refusals are explained (brief item 7). Assumption stands. |
| 7 | Resume | Assumption stands. |
| 8 | "Managed secrets" | Spec: protected levels keep managed (vault) secrets out of model, env, argv, logs and artifacts. Assumption stands. |
| 9 | Kill switch and model turns | No further requirement. Assumption stands. |
| 10 | Child agents, project config | Child agents: ask the product's agent to spawn one (normal feature). Project config: `.codex/config.toml` inside the trusted project. |
| 11 | Known read-only, untrusted reason | No further requirement. Assumption stands. |
| 12 | Reproducing E2 | Test fixture: for that case group the coordinator puts back the system-wide config file `/etc/codex/config.toml` that was present on the test machine when E2 was captured. |
| 13 | "Come back exactly" | Spec: prior settings are restored exactly. Assumption stands. |

Further test-environment facts given to every executor (neutral navigation, not expectations): extra synthetic
environment variables can be set at launch (`--env NAME=synthetic…`, needed by AGG-15); launch arguments such as
`-c key=value` can be passed to the product (AGG-25); the `zai-anthropic` route can be selected at launch.

## Amendment A01 (integrator, additive)

The P1 plan carries a "verify on the next candidate" row for the `zai-anthropic` route with model `glm-5.2`
(every turn failed for lack of a catalogued max-output limit). The designer's AGG-27 covers `zai-anthropic` with
GLM 5.3 Flash only, so this additive case covers the plan row. It supersedes nothing. GLM 5.2 is used here
only because the row is specifically about that model (recorded reason under the 2026-10-10 amendment).

- **AGG-A01** (blocker). Start: fresh profile, Permissive, launched on route `zai-anthropic`, model `glm-5.2`.
  Actions: ask `Reply with exactly: pong`; then select Aggressive in `/security`, confirm, restart with `r`, ask
  the same. Expected: both turns complete with "pong"; no error about a missing output limit or a refused request.

The plan's other "verify on the next candidate" rows are covered by original cases: typed text cannot approve
(AGG-07), a declined request is not shown as run (AGG-08), "Enable full access?" cancel message (AGG-31).
