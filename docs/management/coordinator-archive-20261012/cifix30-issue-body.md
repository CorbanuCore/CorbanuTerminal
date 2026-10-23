On Linux, depending on what the user's home contains, every sandboxed shell command fails under Moderate/Aggressive with `source_envelopes` on. bubblewrap fails before the command starts.

## Repro (glitch, Ubuntu 24.04, bubblewrap 0.9.0)
`suite::provenance::pf_30_s01_source_envelopes_label_tool_output_and_keep_human_prompt` (codex-core), run with `HOME` set to a disposable directory:

| `HOME` contains | result |
|---|---|
| `.config/` | pass |
| `.config/` and `.claude/` (no `.credentials.json`) | `printf` fails: `bwrap: Can't create file at $HOME/.claude/.credentials.json: Read-only file system` |
| nothing | `printf` fails: `bwrap: Can't bind mount /bindfile… on /newroot$HOME/.config: … No such file or directory` |

The same failure happens with the real home on glitch, which has `~/.claude/` but no `.credentials.json`. It also happens at `c7e6e07cbc`, before the 2026-10-11 Aggressive merges, so it isn't new today. GitHub runners pass because their home has `.config` and no `.claude`.

## Why it matters
Claude Code users on Linux often have a `~/.claude/` without `.credentials.json` (API-key login, or not signed in yet). For them, every agent command fails at these levels. The model sees exit code 1 with a bwrap error.

## Likely cause
`USER_CREDENTIALS` in `core/src/security/protected_surface/read_denials.rs` adds `.claude/.credentials.json` with `push_fixed`, which becomes `skip_missing_path` deny entries. On Linux, `append_unreadable_root_args` in `linux-sandbox/src/bwrap.rs` masks a missing path by creating a mount target at its first missing component. That fails when the parent folder is already mounted read-only. The `.config` case looks like the same path through a missing parent. This is related to #348, where the placeholder is created on the host; here it can't be created at all.

Proposal: skip masking a missing denied path when its parent can't gain it inside the sandbox (read-only parent). Masking only needs to block creating it there, and a read-only parent already does that. Alternatively, mask the nearest existing writable ancestor.

Found while fixing postmerge-ci (PR #471); left unfixed because it's a product change.
