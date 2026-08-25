# Runtime, permissions, and extensions

## The pain

A capable agent needs file and shell tools, review, external integrations, and
custom workflows, but those surfaces become dangerous when their permissions
or provenance are invisible. Corbanu Terminal keeps the Codex-derived runtime
and exposes its controls directly.

## Product contract

| Field | Value |
| --- | --- |
| Status | **LIVE** |
| Exact product-spec heading | **Shipping MVP — LIVE** |
| Requirement excerpt | “Runtime: Rust, Apache-2.0, Linux/macOS/Windows, the `corbanu` command, and legacy `pfterminal` command and state compatibility.” |
| Related excerpt | “Workspaces: `/panes`, `/agent`, approvals, existing general sandboxing, review, MCP, skills, plugins, apps, connectors, and background terminals.” |

## Start the runtime

Run `corbanu` from the workspace the agent should inspect. The terminal loads
repository instructions, session configuration, the selected model, and the
active permission and sandbox policy.

Corbanu Terminal runs on Linux, macOS, and Windows. The legacy `pfterminal`
command and state paths remain readable for compatibility; `corbanu` is the
primary product command.

## Control what the agent can do

- `/permissions` chooses the active permission posture.
- Sandbox configuration limits filesystem and process access.
- Approval prompts expose sensitive actions before execution.
- `/review` reviews current changes.
- `/status` shows the active session configuration and token state.
- `/diff` shows the workspace diff, including untracked files.

A model's statement that an action is safe does not replace host authorization.

## Extend the runtime

| Surface | Command | Purpose |
| --- | --- | --- |
| MCP | `/mcp` | Inspect configured MCP tools |
| Skills | `/skills` | Browse task-specific instructions |
| Plugins | `/plugins` | Browse installed plugin capabilities |
| Apps | `/apps` | Manage connected app surfaces |
| Hooks | `/hooks` | Inspect and manage lifecycle hooks |
| IDE context | `/ide` | Include current editor selection and open-file context |
| Background terminals | `/ps` | Inspect running background commands |

Extensions inherit the active host permissions. Installing or connecting an
extension does not authorize it to expose secrets, move money, or bypass the
sandbox.

## Related documentation

- [Exec](../exec.md)
- [Execution policy](../execpolicy.md)
- [Sandbox](../sandbox.md)
- [Skills](../skills.md)
- [`/panes` and workspaces](workspaces.md)
