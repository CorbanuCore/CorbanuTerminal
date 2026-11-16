## Resume and recovery

When `resume --last` finds no resumable conversation, the TUI and `exec` now explain the lookup scope, including the model provider when filtered, and say that a new conversation is starting. `exec` also explains missing conversation names and other fresh-start fallbacks on stderr, keeping JSON output unchanged.
