# Claude Code headless panes

## The pain

Claude sessions are hard to supervise when they live outside the terminal's
pane, credential, and artifact system. Corbanu Terminal keeps provider-backed
Claude Code sessions beside ordinary user and agent panes.

## Product contract

> **Product specification — “Shipping MVP — LIVE”**
>
> “Workspaces: `/panes`, `/agent`, approvals, existing general sandboxing,
> review, MCP, skills, plugins, apps, connectors, and background terminals.”

## Create a Claude pane

Run:

```text
/panes
```

Choose **New Claude Pane**, then select an available provider profile. The pane
appears with user panes and agent panes and can be selected again later.

Supported profiles are derived from configured credentials and installed
provider support, including Claude Plan and provider-backed Claude-compatible
routes. A profile is offered only when its required authentication path is
available.

## What persists

Each Claude pane retains:

- its pane identity and display name;
- provider profile;
- Claude session identifier when available;
- turn status and latest usage summary;
- streamed turn artifacts outside the active chat context; and
- enough state to switch away and resume the pane.

The TUI converts Claude Code stream events into native Corbanu Terminal history
cells and shows bounded progress during a running turn.

## Credential boundary

Corbanu Terminal resolves provider credentials through the host-side vault or
the provider's native login mechanism. Raw provider keys must not be inserted
into prompts, visible chat history, pane transcripts, or model-readable
metadata.

Claude Plan uses the exact long-lived subscription-token or Claude Code login
source selected in `/providers`. See
[Reliable Claude Plan authentication](claude-plan-authentication.md).
The pane plan stores only a deferred auth descriptor. Immediately before Claude
starts, Corbanu resolves the selected source through the same trusted helper as
ordinary Claude Plan requests, removes inherited API-key/cloud-routing
overrides, and supplies the credential only to that child process. The value is
not written to pane arguments, settings, artifacts, audits, or persisted pane
metadata.
Vault-backed profiles use the credential label associated with that provider.

## Failure and recovery

Provider errors, cancellation, and interrupted turns remain visible in the pane
instead of silently discarding the session. Switching panes does not terminate
the retained Claude session. A later prompt can resume a pane whose provider
session remains valid.

## Main implementation

- `codex-rs/tui/src/claude_panes/`
- `codex-rs/tui/src/chatwidget/provider_credentials.rs`
- `codex-rs/tui/src/chatwidget/model_popups.rs`
