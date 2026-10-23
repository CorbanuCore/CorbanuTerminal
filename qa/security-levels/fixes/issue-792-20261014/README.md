# #792: the inspector's rows before the restart (AGG-04)

Replay of the record's steps (AGG-04, Q02b) on PR #799's head `409064da15`, in tmux with a
disposable home and GLM 5.3 Flash on zai (`qa/demos/specs/issue792-inspector-vault-before-restart.toml`):
under Permissive, confirm Aggressive in `/security`, Esc on the saved screen (no restart),
`/security`, `i`.

- `before-U07-inspector-before-restart.txt`: the candidate `4440d2d4ce` (copied from the targeted
  run's review packet). The Vault row said the vault store and sign-in file are readable; the
  Sandbox and Network rows said approved commands can run outside the sandbox.
- `after-inspector-before-restart.txt`: this fix. Sandbox: approved and allow-listed commands stay
  inside the sandbox under Aggressive; Network: off for agent commands; Vault: unreadable
  (Aggressive, also when you approve a command), and direct `corbanu vault …` commands are refused.

Video: [mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-792-issue792-inspector-vault-before-restart-409064da15b4-2026-10-14.mp4).
