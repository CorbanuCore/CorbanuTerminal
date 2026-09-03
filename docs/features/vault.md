# /vault and credentials

## The pain

Service credentials should be usable by agents without ever being pasted into
chat, prompts, transcripts, logs, or source files. Corbanu Terminal provides an
encrypted credential store with masked entry and metadata-only inspection.

## Product contract

| Field | Value |
| --- | --- |
| Status | **LIVE** |
| Exact product-spec heading | **Shipping MVP — LIVE** |
| Requirement excerpt | “Vault and credentials: Encrypted `/vault`, masked entry, metadata-only inspection, and operational credential use without placing raw values in chat.” |

## Open the vault

Run:

```text
/vault
```

The action menu provides:

| Action | What it does |
| --- | --- |
| **Add credential** | Prompts for a label and opens masked secret entry. |
| **View credentials** | Lists labels, types, providers, and timestamps without revealing values. |
| **Copy secret** | Copies a selected secret through the secure UI without placing it in chat. |
| **Delete credential** | Removes a selected credential after explicit confirmation. |

For repeatable operations, use:

```text
/vault list
/vault show <label>
/vault credential add
/vault credential delete <label>
```

`/vault credential add` intentionally accepts no inline secret. Enter the
value only in the masked modal. `/vault show <label>` returns metadata, never
the raw value.

## Use a stored credential

Agents and automation refer to a credential by its vault label. When a command
needs the value, retrieve it only at execution time and attach it directly to
the consuming command:

```bash
CREDENTIAL="$(corbanu vault auth-helper <label>)" your-command
```

Never run the helper by itself, print its expansion, or place the result in
chat. Provider keys entered through onboarding or `/providers` are also stored
in this vault.

## What belongs in the vault

The vault supports operational service credentials, including:

- inference-provider API keys;
- bearer, basic-auth, and OAuth credentials;
- RPC and exchange credentials;
- deployment credentials; and
- other manually labeled service secrets.

Seed phrases, cryptocurrency private keys, and keystores do not use the generic
credential flow. Those require an explicit user-controlled wallet or signing
workflow.

## Know which surface to use

| Surface | Use it for |
| --- | --- |
| `/vault` | Add, inspect, copy, use, or delete service credentials by label. |
| `/providers` | See provider authentication status and enter supported provider credentials. |
| `/model` | Choose the active provider, model, and reasoning effort after authentication. |
| `/wallet` | Manage signing custody, balances, backup/restore, and Corbanu API linkage. |

The vault and wallet are separate security boundaries. Deleting a vault
credential can make its provider or service unavailable, but it does not delete
the Solana wallet. Disconnecting Corbanu API removes its local API credential
without deleting the wallet or changing its server-authoritative dollar balance.

## Storage and security boundary

Vault values are encrypted at rest in the Corbanu Terminal home. The OS keyring
stores the vault passphrase where available; keyring-less systems use a local
permissions-restricted fallback. Metadata remains inspectable so users can
manage credentials without exposing their values.

See [Authentication and Vault](../authentication.md) for provider-key labels,
account login, environment-variable fallback, migration behavior, and logout
semantics.
