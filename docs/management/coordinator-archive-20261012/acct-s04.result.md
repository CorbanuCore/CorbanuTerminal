I finished PF-60-S04 and set the sprint to `ready` on main; it is not archived. All four PRs merged with every check green.

My own qualification on one recorded binary passed every cost flow. Next come the independent code-blind acceptance and your acceptance.

**What Travis must decide or accept**
1. **Accept PF-60-S04.** Do this after the code-blind acceptance.
2. **Lift waiver (a).** I recommend yes. On the recorded binary, 13 real OpenAI `gpt-5.4` requests on the API key cost $0.270659, and my recomputation from OpenAI's own usage reports at published prices gives $0.2706590. They match to the micro-dollar.
3. **Approve the narrator voice in the new reel.** It's the same unapproved voice as the earlier reels. The reel is built but not published; the coordinator publishes after you review it.
4. **Accept the remaining gaps as known limitations:**
   - A guardian review of a `/side` conversation, and sub-agents started from one, are still not recorded. Nothing is put under the wrong conversation.
   - The `side:` label also covers any other temporary fork of a saved conversation, not only `/side`.
   - The #368 price items (OpenRouter, Moonshot/BigModel, gpt-6.1-sol, the Sol promotion ending 2026-11-21, DeepSeek holidays) are recorded only, because this sprint excludes price changes.
   - Still untestable, as in S05: ChatGPT login, image generation, realtime. TensorCash was unreachable from these machines.

**PRs**

| PR | What it does | Review |
|---|---|---|
| #382 | Starts the sprint and folds in the four hand-over items | — |
| #383 | `/side` conversations and memory consolidation are now recorded under the conversation they belong to | Round 1 asked for changes (the owner was set too late); round 2 approved |
| #385 | A request with no price shows one line, "Price: none recorded — this request records only its billing basis (…)", instead of a "Price source" | Round 1 asked for changes (it misread plan-rate records as unpriced); round 2 approved |
| #393 | Qualification record, evidence, demo index, sprint set to `ready` | — |

The full record is `qa/portfolio/agent-cost-accounting/qualification.md`.

**Tests**
- **Focused tests on macOS:**
  - core: 329 with the developer-accounting feature, 324 without;
  - memories: 48;
  - tui: 126 with the feature, 113 without.
- **Linux clippy `-D warnings`** on the RTX box: 0 warnings on every slice's final code, with and without the feature.
- **The sprint's tmux suite:** 71 of 71 passed on the final commit. `just` isn't installed on RTX, so I ran the command it wraps directly.

**Qualification**

The recorded binary is main `1e9cd464b9`, a Linux debug build (sha256 `cb378c27…a1d58`). I drove the real TUI through tmux with real keys, in throwaway homes kept off the keychain.

- **Three providers, parent and children:**
  - An OpenAI parent with children on Kimi Code, Claude Plan and DeepSeek: 11 requests, each counted once. Subscription work is never shown as money spent.
  - A GLM 5.2 parent on Z.AI with Kimi and Claude children. This also served as the GLM tmux run the gate asks for.
- **The two fixes, seen live:**
  - `/side` requests appear under the parent conversation.
  - Memory consolidation's 6 requests (about 89k tokens of the Kimi Code subscription) appear under the conversation that started it.
  - Requests with no price show the new single line.
- **Edge cases:**
  - A cancelled request shows "usage incomplete" and the total reads "at least".
  - Priority-tier requests show "no price available", never a guessed price or zero.
  - A retry forced by a fake server error counts as 1 request with 2 linked attempts; tokens are counted once.
  - After reopening, and after killing the process and restarting, the earlier requests are identical.
  - Past days open and date ranges group correctly; impossible dates (future day, reversed or future-start range, malformed date) are each refused with their reason.
- **Reconciliation:** every pay-per-use figure matches my recomputation exactly (OpenAI, GLM, DeepSeek). The synthetic past days match their expected totals, and the existing contract test suites pass (48 and 6/6).
- **Key scan:** 0 hits in everything committed and in the demo recordings. The only hit was in a throwaway smoke home from before #381 merged, and it's deleted.

**Things you should know**
- **Reopening costs money.** Every reopen of an OpenAI conversation sends a paid warm-up request (about $0.025 on `gpt-5.4`). It's recorded correctly, but users may not expect a charge for reopening.
- **Esc Esc forks.** Pressing Esc twice on an empty input box edits the previous message and starts a new conversation. My test driver hit this once; its cost went to the new conversation, correctly.
- **Wording nit:** with a basis set in the user's config, the new line reads "(pay per use (set in your config))", with nested parentheses.
- **Z.AI outage:** Z.AI briefly refused requests for "insufficient balance" (code 1113) on both routes, then worked again. While it refused, `/cost` showed the denied requests honestly as incomplete, never as zero. That observation is from an interim build, not the recorded binary.
- **Spend:** about $0.50 on the OpenAI key, $0.03 on Z.AI and under $0.01 on DeepSeek; Kimi and Claude ran on their subscriptions.

**Videos**
- **Five demos**, published and leak-scanned, indexed in `qa/demos/index/PF-60-S04.md`: the side conversation, a request with no price, a parent with children, reopening, and date limits.
- **Narrated reel:** `.codex-work/accounting-reel-s04-20261010/out/agent-cost-accounting-s04-reel-2026-10-10.mp4` (4:39, 1080p, chapters, with an `.srt`), not published.

**Cleanup**
- **Removed:** my worktree, my build folder (80 GB) and everything I created on the RTX box.
- **Left in place:** a few merged local branches that a normal delete refused. I didn't force them.
- **Partial build in a shared folder:** a quoting mistake in one of my commands started a test build in `.codex-work/corbanu-terminal/target`. I stopped it, but its partial output is still there because that folder isn't mine to delete.