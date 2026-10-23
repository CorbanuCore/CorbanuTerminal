# Functional execution report — Group G20

- Executor role: independent functional executor. This is product observation, not human acceptance.
- Scope: assigned case AGG-31 only, in the supplied order.
- Product/package: `/opt/agms/pkg/bin/corbanu`, Corbanu Terminal 0.1.48.
- Product SHA-256 observed: `181ad0299d238accbf936512531c90988e8d15cb02a50ea8241f8566b2372a4a`.
- Frozen cases SHA-256 observed: `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`.
- Preflight:
  - Coordinator summary `preflight-summary.json`: `"ok": true`, 85 probes, no failures.
  - `preflight-launcher.json`: `"ok": true`.
  - `preflight-tmux-child.json`: `"ok": true`.
  - Executor preflight: `agms-probe all` rc=0, wrote `preflight-executor.json`; report says `"ok": true`.
- Run/profile: `$AGMS_RUN/product-home`, fresh for the launched product session; product started in `$AGMS_RUN/fixture/workspace`; route `zai`; model `glm-5.3-flash`.
- Fixture state: the case does not require product actions that alter fixture contents, so no fixture change was expected. A final read-only fixture record was captured with `agms-tui fixture A31-final` at 2026-10-11T03:51:01Z; the product session was then stopped cleanly (`stop_rc=0`).

## AGG-31 — blocker

### Requested actions

1. Fresh profile, Permissive.
2. `/permissions`, choose Full Access.
3. At `Enable full access?`, cancel.
4. Reopen `/permissions`.

### Actual actions and observations

- 2026-10-11T03:48:44Z: no prior product session was connected (`agms-tui screen` reported the private tmux socket absent).
- 2026-10-11T03:48:57Z: launched with `agms-tui start`. The product displayed the workspace-trust choice. `Yes, continue` was already selected; pressed literal `Enter`. The product reached the main prompt. Observation labels: `A31-trust`, `A31-main`.
- Initial `/permissions` screen showed `1. Ask for approval (current)`. This was treated as the Permissive starting condition; the case text did not display the word “Permissive” in the UI. Observation label: `A31-permissions-first` at 2026-10-11T03:49:36Z.
- Selected the visible Full Access item with `Down Down`, observed `› 3. Full Access` in label `A31-full-selected` at 2026-10-11T03:49:42Z, then pressed literal `Enter`.
- The confirmation appeared with the title `Enable full access?` and two choices: `Yes, continue anyway` and `Cancel`; `Cancel` was selected. Observation label: `A31-confirmation` at 2026-10-11T03:49:42Z.
- Pressed literal `Enter` on Cancel. The product showed `• Full access was not enabled; permissions are unchanged. To enable it, choose Full Access, then "Yes, continue anyway".` The reopened permissions list continued showing `1. Ask for approval (current)`. Observation label: `A31-after-cancel` at 2026-10-11T03:49:49Z.
- 2026-10-11T03:49:53Z: pressed `Escape` to return to the main prompt (offered by `Press enter to confirm or esc to go back`), observed in `A31-main-before-reopen`, then typed `/permissions` and pressed `Enter` separately. The permissions screen reopened unchanged, again showing `1. Ask for approval (current)`. Observation label: `A31-permissions-reopened`.
- 2026-10-11T03:49:57Z: exited permissions with the offered `Escape` after the literal action was complete, leaving the product on the main prompt (`A31-exit-permissions`). The unchanged-level message remained visible.

### Positive visible checkpoints

- `A31-permissions-first`: permissions screen shows current `Ask for approval`.
- `A31-full-selected`: visible `› 3. Full Access`.
- `A31-confirmation`: visible `Enable full access?` and selected `Cancel`.
- `A31-after-cancel`: visible explicit unchanged message and current `Ask for approval`.
- `A31-permissions-reopened`: permissions screen reopened with unchanged current `Ask for approval`.
- `A31-exit-permissions`: main prompt regained without an additional state-changing action.

### Expected-result outcomes

- **The confirmation appears — passed.** Label `A31-confirmation` shows `Enable full access?` after choosing Full Access.
- **Cancelling shows that nothing changed — passed.** Label `A31-after-cancel` shows the explicit unchanged message and `1. Ask for approval (current)`.
- **The level is unchanged — passed.** The same `1. Ask for approval (current)` entry is visible before and in the reopened screen after cancellation (`A31-permissions-first` and `A31-permissions-reopened`).

### Notes/ambiguities

- The UI uses `Ask for approval (current)` for the starting state. The case names the state Permissive but did not require an additional screen to prove a separate Permissive label; this equivalence was preserved as an interpretation, not broadened into extra actions.

VERDICT AGG-31: passed
