# tmux run on GLM 5.2 (`-m glm-5.2 -c model_provider="zai"`)

- **Candidate:** `10ac64a019`.
- **Profiles:** three disposable profiles, each with a trusted workspace.
- **Keys:** `/secur` typed, then Enter, then Esc.

| Profile | Popup description (`*-popup.txt`) | Enter opens (`*-opened.txt`) |
| --- | --- | --- |
| `off`: flag absent | explore security profiles and protection readiness (read only) | Security profiles — read only |
| `on`: `security_levels = true` | choose a security level; takes effect when you restart | Security level picker, Permissive active |
| `stored`: flag absent, Aggressive stored | choose a security level; takes effect when you restart | Security level picker, Aggressive active |

No model turn: an agent cannot run slash commands, so the description is only reachable through keys.
