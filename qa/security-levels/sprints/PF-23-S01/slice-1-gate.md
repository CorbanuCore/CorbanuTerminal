# PF-23-S01 slice 1: per-sprint gate (2026-10-06)

- **Branch:** `feat/pf-23-s01-moderate-ingress`, off main at `55339d5b24` (PF-30-S03 merged as #212).
- **Scope:** behind the default-off `source_envelopes` feature, Moderate and Aggressive only. Flag off,
  Permissive and sessions without untrusted content are unchanged: every new check returns before classifying.

## What ships

| Piece | Behaviour |
| --- | --- |
| Route matrix | `protected_surface::coverage` lists every tool route by who provides it (`CoreToolRuntime::tool_origin`: built-in, extension, MCP, client) and how a protected action on it is checked: the PF-30-S03 approval seam (shell, exec, patch, structured edits), its own check (MCP calls, `write_stdin`), nested calls only (code mode), child agents, policy requests, or no protected effect. Anything else (client tools, unlisted extension tools, unknown built-ins) is unclassified and, after untrusted content, needs a fresh human answer at the dispatch boundary; refused under `never` and with the kill switch. `request_permissions` asks there when an automatic reviewer would answer it; `request_plugin_install` always. A test builds every built-in tool with all features on and fails if one is unclassified. |
| MCP calls | Protected after untrusted content when the tool may change or send data (not read-only and destructive/open-world or unknown: Disclosure), when its name moves value (`transfer`, `swap`, `send_sol`, ...; a read verb such as `get_swap_quote` is not, unless joined to an act), or when an argument reaches a protected path or a command-like key holds a protected command. The tool's own annotations can only add protection. The check runs before remembered approvals, permission hooks, auto-approve and the automatic reviewer; the question (Allow once / Cancel) shows the arguments and uses an id the delegate's reviewer never answers. |
| Typing into a process | `write_stdin` text is judged together with what was typed into that process since untrusted content (so `corban` + `u vault list` is one command), as input to the program it goes to: a plain REPL's code, otherwise shell input plus any interpreter named in the command. Line-editing keys, escape sequences and history re-execution (`!!`, `^a^b`, `fc`, `r`) are unreadable, as is more than 16 KiB; after a lone Ctrl-C the next text is also judged on its own. Writes to one process are serialized; approvals clear the kept text. |
| Code mode | A cell reads nested tool results before they are recorded, so each result raises the taint generation; the cell's later calls are post-taint. The cell has no host access of its own. |
| Outbound and value | Two new kinds in the shell classifier: Disclosure (a file, stdin or text the host cannot see sent to another machine by curl, wget, httpie, nc/socat, scp/rsync, `gh gist create`, mail; literal request bodies and requests to loopback with no proxy or redirect stay quiet) and value transfer (solana, spl-token, cast, bitcoin-cli, electrum, sui, aptos, near). Judged at the command word and inside exec-style wrappers (`find -exec`, `proxychains`, `npx`, `uv run`, `op run --`); more than 64 possible start positions fail closed. `~/.curlrc` and `~/.wgetrc` count as credential files. |
| Audit | Each decision logs `route` (`mcp_tool`, `write_stdin`, `unclassified_tool`, `policy_request`), kind, taint generation, outcome and wait time, never arguments. |

## Gate results

- **Formatting and lint:** `just fmt`, `just fix -p codex-core` clean; Linux clippy (`cargo clippy -p codex-core
  --all-targets -- -D warnings`) clean on the RTX box at each review commit, last at `eac4be8903`.
- **Focused tests:** `just test -p codex-core pf_23_s01` (unit: matrix incl. every built-in with all features,
  MCP classification with paraphrases and adjacent read names, typed input, the typed-input window, outbound and
  value-transfer cases from three review rounds, bounded wrapper positions; suite: protected MCP call asks /
  cancel / refused under `never` / ordinary and Permissive unchanged, typing a vault command asks / refused /
  split across two writes refused / ordinary typing unchanged, a code-mode cell's later vault call refused while
  Permissive runs it). `pf_30_s0` still passes 102.
- **Full crate:** `just test -p codex-core` at `8037f122a9` and again at `eac4be8903`: 3,841 of 3,844 pass; the
  3 failures are the known
  baselines (`skills_append_to_developer_message`,
  `skills_use_aliases_in_developer_message_under_budget_pressure`,
  `remote_compact_trim_estimate_uses_session_base_instructions`).
- **GLM 5.2 tmux runs and videos** at `eac4be8903`, in [qa/demos/index/PF-23-S01.md](../../../demos/index/PF-23-S01.md):
  a code-mode MCP call with `~/.docker/config.json` asks (Security level question with the arguments); an
  ordinary code-mode MCP call runs unasked; typing `corbanu vault --help` into a running bash asks; one code-mode
  script that reads notes.txt then runs the vault command is refused with approvals off; `curl -T notes.txt` to
  a remote host and `solana transfer` each stop at an approval naming the kind.
  GLM refused earlier takes itself (echoing `~/.ssh/id_ed25519`; twice the upload to an `example.com` host,
  calling it exfiltration); one take ended with the model paraphrasing the refusal past the wait pattern. The
  specs were reworded, not the product.
- **Independent review:** Opus 5.5 High, read-only through `corbanu exec`, six rounds in
  `.codex-work/workers-20261002/pf23s01-review{1..6}/`. Rounds 1-4 CHANGES REQUESTED (split and edited typing,
  interrupts, parallel writes, outbound wrappers/destinations/curl forms, read-named MCP tools, shells behind
  interpreters, quadratic wrapper cost); round 5 APPROVE at `8037f122a9`; round 6 APPROVE of the follow-up for its
  Lows 1 and 3 at `eac4be8903`.

## Known limits

- Lexical net: unknown exec wrappers, run-time strings, build tools and other forms the PF-30-S03 list names;
  REPL forms still read as plain (`python3 -ic'...'`, `perl -E...`, `node --import=data:...`). OS-level denial
  of home and credential reads is slice 3.
- Parallel code-mode results make an open approval stale (refused); a second question in one turn replaces the
  first (existing per-turn slot). `web.run` fetches model-chosen URLs. Contents of `~/.curlrc` used without flags.
- Over-prompting after untrusted content (asks, never runs unasked): `$` sigils and template literals typed into a
  Perl, Ruby or Node REPL; lines typed into a REPL that is not plain (`uv run python`, `python manage.py shell`,
  `cd app && node`) are also judged as shell input (`!ok`, `"""`); a wrapped command with more than 63 arguments
  (`npx prettier --write` on many files); 16 KiB typed into one process asks once.
- **Product finding:** the Chat Completions and Anthropic wires drop namespace tools
  (`core/src/client.rs`, `tool_spec_to_chat_tool` and its Anthropic twin), so GLM 5.2 never sees MCP tools
  directly; the MCP videos reach them through code mode.
