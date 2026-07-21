---
name: pfterminal-help
description: Help users operate PfTerminal's product-specific features and slash commands. Use when a user asks how to configure providers or credentials, use the encrypted vault, create or restore a Solana wallet, buy or manage a PfTerminal Plan, rent GPUs through Vast.ai or RunPod, choose rented models, spawn or supervise agent hierarchies, navigate panes, use Task Node, or troubleshoot PfTerminal state and workflows. Focus on what to do in the TUI and what each action means, not source-code implementation.
---

# PfTerminal Help

Help the user complete their task in the TUI with the fewest safe steps. Prefer the interactive menu for discovery and inline subcommands for repeatable actions. Explain costs, credential exposure, custody, or cleanup before the user confirms an action that changes them.

Do not turn product help into a code tour. Do not ask the user to paste API keys, wallet recovery material, passcodes, grants, or tokens into chat. Direct secret entry to PfTerminal's masked views.

## Route the request

| User goal | Start here | Key distinction |
| --- | --- | --- |
| Add or replace an inference credential | `/providers` | Provider access and model selection are separate; use `/model` afterward. |
| Store or inspect any secret | `/vault` | The vault manages credentials; `/providers` is its provider-focused front door. |
| Hold SOL/USDC or buy a PfTerminal Plan | `/wallet` | Wallet custody and the local plan credential are separate state. |
| Rent model-serving GPUs | `/gpu` | PfTerminal controls a third-party rental; the marketplace account is billed. |
| Create a Nazgul/Troll/Orc hierarchy | `/spawn` | Spawn creates roles and reporting relationships. |
| Attach ongoing supervision to existing work | `/orchestrate` | Orchestration manages a persistent assignment; it does not create the crew. |
| Switch among PfTerminal, Claude, or agent panes | `/panes` | Use `/agent` for native agent threads and `/panes` for terminal panes. |
| Use Post Fiat Task Node | `/tasknode` | Use the dedicated Task Node skill for submission and verification workflows. |
| Diagnose current state | See the status map below | Read the authoritative surface before proposing resets or reconfiguration. |

## Providers and models

Use `/providers` to view sign-in and credential status for account plans, PfTerminal Plan, and supported API-key providers. Selecting an API-key provider opens masked entry and stores the key in the encrypted vault. Environment-provided keys may appear as available without being copied into the vault.

Use `/model` to choose the provider, model, and reasoning effort for the session. Adding a credential does not automatically select that provider. If a model is unavailable, check `/providers` for authentication and `/model` for catalog visibility before changing files or clearing state.

Use `/usage` for the selected account or plan's allowance and reset information. Use `/status` for the active session, model, permissions, and token configuration.

## Vault

Use bare `/vault` for the action menu:

- **Add credential** opens masked label and secret entry.
- **View credentials** shows labels and metadata without exposing values.
- **Copy secret** uses the clipboard without printing the secret into chat.
- `/vault list` and `/vault show <label>` are safe metadata views.
- `/vault credential add` must have no inline secret; PfTerminal opens the secure modal.
- Delete only when the user intends to remove that stored credential. Explain which provider or workflow will lose access.

The vault and wallet are different. The vault stores service credentials; `/wallet` manages a signing key, on-chain balances, and PfTerminal Plan linkage.

## Wallet and PfTerminal Plan

`/wallet` manages a local Solana-mainnet wallet holding SOL and canonical USDC. Keep some SOL for transaction fees unless the checkout explicitly says fees are sponsored.

For a new user:

1. Choose **Create wallet** or **Restore wallet**.
2. Save the recovery material from the secure view and set a passcode.
3. Use **Receive** to fund the shown address with the correct network assets.
4. Choose **Buy PfTerminal Plan**, unlock the wallet, review the tier and exact USDC payment, then confirm.
5. After activation, use `/model` to select a model offered through PfTerminal Plan.

Explain wallet actions precisely:

- **Unlock** grants signing capability only to the current TUI for one action or the selected duration. **Lock** revokes it.
- **Plan details** shows prepaid tier, token use, limits, reset dates, and queued periods without crowding the wallet summary.
- **Upgrade** purchases the period shown by the confirmation screen; do not imply an immediate tier change when the UI shows a future period.
- **Recover existing plan** signs an ownership proof and sends no USDC. Use it only for a wallet that previously purchased a plan.
- **Disconnect PfTerminal Plan** removes the local plan credential but keeps the wallet and paid period.
- **Remove wallet from this device** removes local custody and the local plan credential; it does not move on-chain funds or cancel a paid period.
- **Back up recovery material** requires the wallet passcode and uses the secure view. Never request or repeat the recovery material in chat.

Treat a durable payment receipt or refreshed plan state as confirmation. If settlement is ambiguous, refresh `/wallet` and inspect the latest receipt before suggesting another payment.

## GPU rentals

`/gpu` rents third-party capacity; PfTerminal does not provide a free GPU pool. The user needs a funded Vast.ai or RunPod account and its API key. For Vast.ai, adding the API key authorizes PfTerminal to use that marketplace account; it does not add credit to the Vast balance. Configure the key in `/gpu`'s masked credential flow, then choose a verified recipe.

The rental flow asks separately for:

1. Maximum hourly USD price.
2. Maximum total USD spend.
3. Duration in whole minutes; setup, model download, and loading consume this time.

PfTerminal searches compatible offers and shows a final billable confirmation. Provisioning may pass through hardware checks, runtime setup/build, model download, artifact verification, loading, and endpoint qualification. Only `READY` means the rented model is available in `/model`.

Use `/gpu` as the authoritative cross-process view of active or potentially billable rentals and estimated spend. Distinguish cleanup actions:

- **Stop serving** removes the endpoint from model selection but provider billing continues.
- **Terminate rental** requests provider cleanup. Treat billing as unresolved until PfTerminal reports provider-confirmed termination.

Never advise exiting PfTerminal as a substitute for terminating a rental.

## Agent hierarchy and panes

Use `/spawn` for the guided role picker or `/spawn status` for the live hierarchy. Targeted entry points are `/spawn nazgul`, `/spawn troll`, and `/spawn orc`.

- **Nazgul** is the root supervisor that converts the user's objective into coordinated work.
- **Troll** manages delegated execution and reports upward.
- **Orc** performs bounded implementation or investigation work and reports results upward.

Do not invent staffing bans, evidence rituals, deadlines, or domain-specific workflows. Role prompts establish general behavior; the user's objective defines the work. Do not treat context compaction as a dead agent. When capacity appears occupied, inspect `/spawn status`, `/agent`, and `/panes` before recommending replacement.

Use `/orchestrate` to attach a persistent supervisory assignment to an existing worker or pane. Bare `/orchestrate` opens the guided flow; `/orchestrate status` shows assignments. Help with attach, pause, resume, extend, test/fire, and detach through the menu unless the user specifically wants inline syntax.

Use `/panes` to switch among PfTerminal, Claude Code, and spawned panes. Use `/agent` to switch the active native agent thread. Use `/ps` to inspect background terminals and `/stop` only when the user intends to stop all of them.

## Other PfTerminal additions

- `/tasknode`: link an account; inspect tasks, verification, rewards, and balances; or open Task Node chat/context. Invoke the Task Node skill for transaction or evidence details.
- `/docs`: browse bundled MkDocs documentation inside the terminal.
- `/providers`: manage provider sign-ins and stored API keys.
- `/panes`: navigate product and hierarchy panes.

## Troubleshoot from authoritative state

- Session/model/permissions: `/status`
- Provider allowance and resets: `/usage`
- Authentication: `/providers`
- Stored credentials: `/vault`
- Wallet balances, plan, usage, and receipts: `/wallet`
- Rentals, readiness, spend, and termination: `/gpu`
- Crew and dispatch state: `/spawn status`
- Persistent supervisory assignments: `/orchestrate status`
- Pane location: `/panes` or `/agent`
- Background terminals: `/ps`

Quote the exact visible error when explaining a failure. Recommend the smallest reversible next action. Do not start with cache deletion, state deletion, credential replacement, wallet restoration, rental recreation, or session abandonment unless the authoritative surface proves it is needed.
