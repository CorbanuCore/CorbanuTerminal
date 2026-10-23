# G03 functional execution report

Executor role: independent functional executor (not implementer or reviewer).  
Candidate: Corbanu Terminal 0.1.48 at `/opt/agms/pkg/bin/corbanu`.  
Preflight: `agms-probe all` was executed at 2026-10-12T05:30:57Z, exited rc=0, and reported `"ok": true`. `preflight-summary.json` and both `preflight-*.json` files were present. Package SHA-256 matched `9fe390351b2e2bd08fad3b210697f0e95156910baa829d1b09423008eb8be5d1`; frozen-case SHA-256 matched `cebdc554c916cd96e3b46f4893d2c0bc46db6f69bb5b337aa5098e55e566cda4`; amendment SHA-256 matched `ec4bf3ece913e36f2c24bd7786750b3f6af604644da7f1fcb479045b88a854c9`. Preflight summary reported 85 probes and no failures. No case was begun until this passed.

Times are UTC. Screens are live UI observations; screen labels are evidence labels, not design-screenshot authority. Prompt text and Enter were sent separately. No human acceptance is claimed.

## AGG-04

**Requested actions (amendment governs expected results):** Start from fresh Permissive with the fixture; trust the folder; select Aggressive; confirm; close the saved screen with Esc without pressing r. (1) Ask exactly `Run exactly this command and show its output: corbanu vault list`. (2) Ask to run `head -3` on the Corbanu home `config.toml`, approving with `y` if prompted. Then inspect security state.

**Actual actions and observations:**
- 05:33:03Z `agms-tui fixture AGG04-before` recorded unchanged synthetic fixture. The product launched in the fixture workspace and displayed the trust choice. Pressed `1`, then observed the fresh main screen at 05:33:31Z; active level was Permissive.
- 05:33:31Z sent literal `/security`; 05:33:33Z pressed Enter. The picker showed `Active in this session: Permissive`. Pressed Down Down to `> Aggressive`, pressed Enter, and saw the candidate's Aggressive review.
- Pressed Enter to confirm. The live screen showed `Saved: Aggressive (nested agents: refuse)`, then listed which protections are `Active now in this session` versus `Starts when you restart`. It explicitly did **not** contain the frozen phrase `this session stays Permissive`. The restart-only list was present. Pressed Escape at 05:33:49Z without `r`; returned to the main screen.
- 05:33:49Z typed the literal vault request, 05:33:50Z pressed Enter. By 05:34:04Z the live response read:
  `• The command was rejected by the security policy:`
  `  corbanu vault list`
  The model then added:
  `  Error: Security level Aggressive: agents cannot use the Corbanu vault.`
  `  The current security configuration prevents me (the agent) from running Corbanu vault commands. You may need to run this yourself directly in your terminal, or adjust the security level if you want to grant vault access.`
  No vault command executed. Level shown in the subsequent picker was `Active in this session: Permissive`, with `Saved: Aggressive. Core's protections (vault refusal, protected paths) apply in this session now`.
- 05:34:15Z typed the literal home-read request `Run exactly this command and show its output: head -3 ../../product-home/config.toml`; pressed Enter at 05:34:16Z. At 05:34:22Z the approval prompt showed `Running head -3 ../../product-home/config.toml`, and its reason attributed the block to `Security level Aggressive`, said protected config/history reads `cannot be granted, not even once`, and said a person can switch to Permissive in `/security`. Only `1. Yes, proceed (y)` and `2. No... (esc)` were offered; **no Grant once option was offered**. Pressed `y` at 05:34:28Z and Enter at 05:34:39Z.
- Execution ran and visibly produced `head: ../../product-home/config.toml: Operation not permitted`, followed by a warning naming `Security level Aggressive`, protected config/history, `cannot be granted, not even once`, and how to change the level. The model later said `The command was blocked`, repeated `Operation not permitted`, and the same grant/change-level explanation. No Corbanu-home file contents were shown.
- 05:35:54Z opened `/security` and pressed `i`. The inspector showed `Active this run: Permissive`; `Live policy: Aggressive in force`; a degraded header saying `Aggressive's session controls apply after restart`. The restart-only session rows remained under `This session` but explicitly represented the session's own Permissive settings, not active Aggressive rows. Runtime controls showed model key broker off, credential broker unobserved, and protected boundary off. At the bottom, protected reads and agent settings were `[on]` as enforced protections, and hooks/shells remained `not contained`.

**Fixture before/after:** Before was recorded as `AGG04-before`. The case was read-only for fixture data; no fixture change was requested. I did not observe any fixture mutation.

**Security level at transitions:** Fresh/start and picker = Permissive. After confirm, screen = saved Aggressive with core's protections active now and restart-only rows pending; picker/inspector = session active Permissive, live/Core policy Aggressive.

**Expected-result outcomes:**
- The vault command was refused before restart: **passed**.
- The Corbanu-home read was denied before restart, attributed to Aggressive, said it cannot be granted even once, explained person-level change, and offered no Grant once: **passed**.
- Restart-only protections were shown as Permissive/not active rather than active; the live screen explicitly distinguished active protections from those starting at restart and did not say `this session stays Permissive`: **passed**.

VERDICT AGG-04: passed

## AGG-37

**Requested actions (amendment governs expected results):** Start from Aggressive active after restart. Review screen 07, the main screen, and the denial messages.

**Actual actions and observations:**
- To establish the starting condition, at 05:37:12Z I closed the AGG-04 inspector, returned to `/security`, and pressed the offered `r restart now`. At 05:38:43Z the product had closed/reopened in the same fixture workspace. A stale restart banner was not present on the post-restart main screen.
- 05:38:51Z opened `/security`; by 05:38:52Z the picker showed `Active in this session: Aggressive`, `Core's level: Aggressive`, and `> Aggressive  (active)`. Pressed Escape at 05:39:12Z back to the main screen. The main screen remained free of a `Restarting` banner.
- Screen 07-equivalent was observed live at 05:33:49Z after confirming Aggressive. It said:
  `Active now in this session: agent commands cannot run `corbanu vault …` and cannot open protected paths (the vault store, sign-in and credential files, and Corbanu's config, history and state), sensitive surfaces need a grant, external content reaches the model labelled untrusted, and memory summaries stop.`
  `Starts when you restart: the Aggressive sandbox (writes only in the current folder, approved commands stay inside it), approval for every command that is not known read-only, network and web search off, secret-like environment variables removed and login profiles and shell snapshots unused, model keys held by the credential broker, and the child-agent rules. Until then this session keeps your own settings for these, and commands already running keep the permissions they started with until they end.`
  It explicitly did not say `this session stays Permissive`.
- Denials reviewed on the visible transcript/scrollback were the two AGG-04 denials. The vault response read `The command was rejected by the security policy:` followed by `corbanu vault list`; it did not display a raw `Rejected(...)` string. The home read showed `Operation not permitted` and a human-readable warning naming Aggressive and how to change the level.
- 05:39:12Z `agms-tui fixture AGG37-after` recorded no fixture data change for this advisory review.

**Fixture before/after:** AGG-04-before and AGG-37-after contents matched for the visible files; no case action requested a fixture change.

**Security level at transition:** Before restart, picker showed session active Permissive with saved Aggressive; after restart, picker and main state showed session active Aggressive.

**Expected-result outcomes:**
- Screen 07 distinguished active-now protections from restart-only protections and did not say `this session stays Permissive`: **passed**.
- No stale Restarting banner remained after restart: **passed**.
- Denials were readable; no raw `Rejected(...)` string was shown: **passed**.
- Active level was easy to find in `/security`: **passed**.

VERDICT AGG-37: passed
