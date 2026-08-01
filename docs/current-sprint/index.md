# Current Sprint

## PF Terminal 0.1.27

There is one active implementation and release specification:

- [PF Terminal 0.1.27 Canonical Product-Preserving Codex Convergence Spec](PFTERMINAL-0.1.27-SPEC.md)

The dated convergence plan and baseline are historical evidence, not additional specifications
or sources of release authority.

## Security work

Provider API-key containment remains an active security workstream.

The vault already exists and is documented in
[Authentication And Vault](../authentication.md). The current security work is
narrower: agent/pane processes must be able to use provider credentials without
inheriting or exposing raw long-lived secrets.

## Active Security Scope

| Area | Current State | Where To Read |
| ---- | ------------- | ------------- |
| Agent vault access | Active security design for letting agents, subagents, and Claude panes use provider credentials without reading raw vault records or inheriting long-lived API-key environment variables. | [Agent Vault Access](agent-vault-access.md) |

## Reading Path

1. Read [Authentication And Vault](../authentication.md) for the already-shipped
   `/vault` command surface and local credential-store behavior.
2. Read [Agent Vault Access](agent-vault-access.md) for the provider-secret
   containment model for agent and pane execution.

## Boundary

This folder is not a general backlog. Shipped feature documentation belongs in
the [Features](../features/index.md) section, and deferred/non-feature planning
notes belong in [TBD](../TBD/index.md).
