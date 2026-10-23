PR #528 is merged into main (merge commit `5b2dfbbada`), with all 33 checks passing and none pending. Release builds now show each account's token totals in `/status` for the current provider, with no prices or cost records. With `named_accounts` off, `/status` is unchanged. It agrees exactly with `/cost` in developer-accounting builds.

**What it does**
- **One account:** a single line, e.g. `work · 35,100 input · 12,000 cached input · 6,247 output (300 reasoning)`, or `no usage yet` before the first reply.
- **Several accounts:** a short aligned table with the session's account marked `(active)`.
- **Names only:** accounts appear by name, never with key material or fingerprints.
- **Spawned agents:** an agent on a different account counts toward that account.
- **Other work in the session:** automatic reviews, `/review` and memory extraction count too, because `/cost` records them under the conversation.
- **Resume:** a resumed conversation shows its earlier counts again, including its spawned agents.
- **Storage:** counts live in memory and in the existing session record. There is no new database.

**Gate evidence** (`qa/provider-auth/pf-84/s05-status-tokens-gate.md`)
- **Snapshot tests:** cover flag off, no usage yet, one account and several accounts. A separate test shows that with the flag off, `/status` is byte-for-byte the same with or without recorded usage.
- **Agreement test:** two turns on a named account match the ledger `/cost` reads, for input, cached, output, reasoning and total.
- **`just test`, without developer-accounting:**
  - Every focused test of mine passes.
  - The TUI set has four failures, all from this machine:
    - Three are the "path must be shorter than SUN_LEN" socket-path error seen in S05.
    - One `claude_auth` tmux test times out because the long worktree path cuts off the status line it waits for.
  - Other suites: app-server-protocol 287/287, app-server 80/80, thread-store and migration 307/307.
- **`just test`, with developer-accounting:**
  - Core: one failure, `request_user_input_interrupt_emits_deferred_token_count`. It fails the same way on unmodified main.
  - TUI: the same four failures, plus the `cost_scope` tmux test. That test typed `/cost` before its session had started, the same flake S05 saw.
- **Clippy `-D warnings`:** clean on macOS and on glitch (`-j10`), with and without developer-accounting, including after merging main.
- **tmux check (GLM 5.3 Flash, disposable home, keyring isolation on):** two named accounts, `work` and `personal`, both enrolled with the same `provider/zai_api_key`. The session ran on `work` and a spawned agent on `personal`.
  - **Build without developer-accounting:** `/status` equals the provider-reported usage in each conversation's saved session log.
  - **Developer-accounting build:** `/cost` at the same moment showed work 30,253 tokens (14,589 + 15,552 + 112) and personal 4,439 tokens (3,847 + 576 + 16), equal to the `/status` rows.
  - **Resume:** the same table came back after resuming.
- **Video** (39 s, leak scan 0 hits): [pf84-status-account-tokens](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s05-pf84-status-account-tokens-20528c9d785e-2026-10-11.mp4). It shows the session on `default` and a spawned agent on `work`. I couldn't record it with two named accounts, because starting with `--account` races the account enrolment.
- **Opus 5.5 High review:**
  - Round 1 asked for changes. With the flag off, a resumed conversation still sent per-account usage. Reviews, `/review` and memory extraction were missing from `/status`.
  - Round 2 asked for fresh evidence on the fixed code.
  - Round 3 approved. I then fixed its one doc-comment nit.
- **CI:** one ubuntu security-test failure was a known race that open PR #541 fixes; it passed when rerun.
- **Docs:** `docs/provider-accounts.md` has a new section; the S05 sprint doc has its follow-up line.
- **Clean-up:** I removed my worktree, the local and remote branches, my build folder, the disposable homes and copied binaries, and my files on glitch.

**Still open**
- **Not shown in `/status`:** background memory consolidation, which `/cost` records under the conversation.
- **`/cost` is per UTC day** while `/status` covers the session, so they match only for a session within one day.
- **Rare gap:** a side conversation discarded before it ever reported can lose a request that finishes in that moment.
- **Missing test:** round 3 suggested a test that review usage is sent on the active turn. I didn't add it.
- **Bug outside this change:** resuming a conversation recorded on a named account without `--account`, in a home with no default key, opens the API-key screen first. It's in the S03/S04 code.
- **Not done yet:** the independent acceptance run for S05.