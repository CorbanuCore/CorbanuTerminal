# PF-56 Kimi/Vercel runtime evidence

- Date: 2026-09-02 UTC
- TMUX session/socket: `pf56-kimi-vercel-review`
- Pane/PID: `%16` / `3757670`
- Corbanu Terminal: `0.1.35`
- Binary SHA-256: `d5c51203779224704487a6596213062b77a15db6e0b55172adfe71cae2dba944`
- Binary mtime: `2026-09-02 19:52:42.853149984 +0000`
- Binary size: `1379543376`
- Conversation: `01a063c9-1fee-7703-8a47-94950f0a152f`
- Requested and traced provider: `vercel` / `Vercel`
- Requested and traced model/slug: `moonshotai/kimi-k3`
- Requested effort: `high`
- Sandbox/approval: `read-only` / `never`
- Fallback or substitution: none
- Reviewer processes launched: one Kimi Corbanu Terminal process

The TUI footer used the catalog display label `OpenRouter Kimi K3`, while the
authoritative conversation-start trace recorded `provider_name=Vercel`, model
and slug `moonshotai/kimi-k3`, and reasoning effort `high`. Both observations
are retained rather than rewriting the visible evidence.

The controller pasted the prepared Markdown brief buffer once. Corbanu treated
newline-delimited buffer fragments as queued follow-up inputs. The first heading
caused Kimi to locate and read the complete scoped `kimi-brief.md`; it completed
the requested review and emitted the preserved final report after 23m 11s.
Queued fragments then began an unintended follow-up exchange. The controller
sent Escape/Ctrl-C and terminated the exact reviewer PID only after the final
report was complete, preventing further review work. No second reviewer process,
model substitution, or fallback was launched.

