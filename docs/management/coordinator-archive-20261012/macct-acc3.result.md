Every S03 criterion and the three S02 re-checks were run on current `origin/main` (`ee530a0e26e21dac65e0bb9d946e5549c0892806`) with real Z.AI, Kimi, OpenAI and Claude Plan. S03 is 15 PASS, 1 FAIL, 1 NOT VERIFIABLE, and all three S02 re-checks now pass. Evidence is merged in PR #429 (all checks green): `qa/provider-auth/pf-84/independent-acceptance-s03-20261010/` (README.md, REVIEW.md, captures/, scripts/).

This was source-blind (I didn't read the code), not the full isolated code-blind run, so the sprints' handoff-checker box stays unticked. To tell accounts apart I gave each test account a dummy key and routed requests through a small local proxy that records which key arrived; it forwards the real-account keys to Z.AI and rejects the dummy ones with a 401.

**S03**
| # | Verdict | Evidence |
|---|---|---|
| S03-1 Children inherit the account | PASS | A child spawned without an account (explorer role child too) inherits `main` and answers, even though the default key is invalid. A TUI `/spawn` worker's requests all carried `work`. Nazgul/Orc role children weren't run. |
| S03-1b `thread/spawnAgent` workers | NOT VERIFIABLE | I couldn't find a user-facing way to reach it without reading code. |
| S03-2a Only configured accounts allowed | PASS | `ghost` and an account configured only for Kimi are refused, and the refusal lists the configured names. |
| S03-2b Spawn result shows the account | PASS | The result includes `"account":"work"`. The TUI spawn line doesn't show it (a known limit). |
| S03-2c Approval under Aggressive | PASS, with #428 | The TUI asks "Allow the spawned agent to run on account `work`…". Allow runs the child; Cancel runs nothing. Unreadable or tampered stored levels also ask. One gap: in the "Aggressive enforced, boundary unverified" state the switch isn't gated. |
| S03-2d Refused with approvals off | PASS | `exec` under Aggressive is refused with "…approvals are off". |
| S03-3a `--account` on exec, TUI and resume | PASS | Z.AI, Kimi and Claude Plan: `main`/`real` answer, `fake` gets a 401. Claude Code login-folder accounts had no positive test. |
| S03-3b Missing accounts refused at start | **FAIL** | `--account kimi-code:ghost` on a Z.AI session isn't refused and runs on Z.AI's default key (#425). TUI `--account ghost` shows a default-key setup screen that loops and never names the account; nothing is saved or sent (#426). Exec, resume, spawn and workers do refuse correctly. |
| S03-4 Resume uses the recorded account | PASS | A resume with no `--account` uses the recorded `fake` and gets a 401, even with a valid key in the environment. A removed account blocks with "add it with…". `--account main` overrides. The TUI resume hint is wrong (#427). |
| S03-5 TUI worker preflight checks the worker's account | PASS | The credential command is called with `work`. After removing `work`, the worker is refused and the command never runs. |
| S03-6 Precedence | PASS | explicit > recorded > config > default all held. |
| S03-7 Out-of-process worker on another account | PASS | The coordinator stayed on `default`; workers carried `work`, `fake` (one request, no retry) and `main` on resume; `ghost` sent nothing. |
| S03-8 Flag off | PASS | A named `--account` is an error, `--account default` works, nothing else changes. |
| S03-9 Names only, never values | PASS | 0 of 23 request bodies contained any key. Child processes get account names only. The leak scan's only real-key hit is today's OpenAI `auth.json` in a scratch home. |
| S03-10 No retry on another account | PASS | A `fake` child gets a 401 even when both its parent's account and the default key are valid. |
| S03-11 Existing single-account home untouched | PASS | Config and vault hashes didn't change for homes made by a pre-PF-84 build (Z.AI and OpenAI gpt-5.4). Named OpenAI accounts are refused (moved to S04). |
| S03-12 Second provider and Claude token helper | PASS | Kimi works the same way; the Claude token helper receives the account as an argument. |

**S02 re-checks** (same steps as 2026-10-10)
- **S02-3b: PASS.** With an older `corbanu` on PATH, a named Claude account is now refused ("…predates named accounts…") instead of falling back to the default.
- **S02-9b: PASS.** With the flag off, the internal token command refuses named accounts. One change for Travis to note: a stray internal account variable is now refused instead of ignored (it fails closed).
- **S02-14: PASS.** A 401 on a named account now names it and says how to replace its key, with one request and no retry.

**Issues filed:**
- #425: an `--account` naming another provider's missing account is silently ignored.
- #426: TUI setup loop for a missing account.
- #427 (low): two wrong messages on resume.
- #428: a product decision on whether the D3 approval should follow the level the TUI displays.

The one code-blind Opus reviewer returned ACCEPT WITH CHANGES; all 12 findings are answered in REVIEW.md, and the README was revised to match.

**For Travis:**
- **S01:** accept. #418 was closed by #422 but I didn't re-run it.
- **S02:** accept behind the flag. The three failed items now pass, and S02-15 stays open until someone names a sibling route pair.
- **S03:** accept behind the flag, with limits. Fix #425 and #426, and decide #428, before removing the flag. None of them leaks a key or moves a request to another provider, but they break "a missing account is an error". Choosing an account, inheriting it, workers, resume, the approval prompt and per-account isolation all work.

Scratch homes (including the one holding the OpenAI key), the proxy, the tmux server, both build targets and both worktrees are deleted.