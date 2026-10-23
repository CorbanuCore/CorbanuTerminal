# Corbanu Terminal: the Moderate security level

## Who and why

Corbanu Terminal is an AI terminal: a person chats with a coding/trading agent that can read files and run
commands on their computer. `/security` lets the person choose how locked down that agent is. Three levels:

- **Permissive**: today's behaviour, nothing added.
- **Moderate**: "Add strong protection without making normal agent work painful." The recommended level.
- **Aggressive**: the lockdown posture; every important door stays closed until the person opens it.

The intended Moderate user is an everyday developer who lets the agent read files, run tools, use MCP servers and
look at outside content, and who wants two things without a lockdown: instructions hidden in that content must not
be able to make the agent do something protected on its own, and secrets (credential files, the vault, Corbanu's
own sign-in and settings) must stay away from agent commands and the model. Ordinary work in the project should
feel the same as under Permissive.

## What a person who picks Moderate should experience

1. **Choosing the level.** `/security` shows the level active now and the choices. Picking Moderate shows a review
   first: what Moderate adds and what is not available yet. Esc at any point changes nothing. Enter confirms and
   saves; a key restarts Corbanu Terminal right away (`r`). Only the person, in the terminal UI, can change the
   level: the agent, a prompt, a project's files, tools, MCP servers or retrieved content cannot raise, lower or
   switch it. The level survives quitting and starting again.
2. **When it applies.** Owner decision: Moderate's approval rules apply as soon as the person confirms; the rest
   applies at the next start. The product's user documentation says that at confirmation the sandbox rule, the
   protected files and the untrusted-content rules apply at once in every session, while the model-key broker (and
   the level shown as active) waits for the restart; the owner has been asked to confirm this reading. The screens
   say what applies when. A protection must never be shown as active when it is not.
3. **Normal work stays normal.** Reading, editing and running ordinary commands inside the project works as under
   Permissive. Moderate keeps the person's own approval, sandbox, network and web search settings.
4. **Commands stay in the sandbox.** No agent command runs outside the sandbox under Moderate, even when the person
   approves a request to run outside it (owner decision). Such a prompt says the command stays in the sandbox;
   "retry without sandbox?" is not offered; commands the person allowed earlier with "don't ask again" still skip
   the prompt but stay in the sandbox.
5. **Protected files.** Agent commands cannot read the credential files in the home folder (such as `.ssh`, `.aws`,
   `.gnupg`, `.kube`, `.docker`, `.netrc`, `.npmrc` and `~/Library/Keychains`) or Corbanu's own sign-in, vault,
   config, history and state. They cannot write files that run code later: shell and git start-up files, git hooks
   and git config, the project's `.codex` and `.agents` folders, and Corbanu's own files. The person's fresh
   approval of one command lifts this for that one command, except for Corbanu's runtime files; a remembered or
   automatic approval never does. A project's own `.env` files are not covered by Moderate (Aggressive covers them).
6. **Untrusted content.** Output from tools, commands, MCP servers, the web, other agents and memory reaches the
   model labelled as untrusted data, never as instructions. After such content has entered the session, protected
   actions need the person's fresh approval: the vault, credentials, security settings, disclosure, value transfer,
   running code the person hasn't seen, MCP calls that change or send data, and tools Corbanu hasn't classified.
   Cached, remembered or automatic approvals don't count for them, and with approvals turned off they are refused.
   The prompt says why it is asking.
7. **Model keys.** From the next start, provider API keys are held by a separate credential broker that sends the
   model requests (owner decision: on by default under Moderate; the person's own setting still wins).
8. **Child agents.** Agents started inside Corbanu Terminal (`/spawn` and the agent tools) are allowed and run with
   Moderate enforced (owner decision).
9. **"Don't ask again" rules from Permissive.** Rules a person saved under Permissive keep working under Moderate
   for ordinary commands (owner decision); after untrusted content they do not count for protected actions.
10. **Kill switch and revocation.** From `/security` (key `g`) the person can see grants held now, revoke all
    authority and turn on a kill switch that keeps everything closed, across restarts, until they turn it off there.
    It works under every level.
11. **Inspector.** From `/security` (key `i`) a read-only inspector shows what is actually enforced: the saved level,
    what is active this run, grants, recent denials and anything degraded. Controls that do not exist yet read "not
    available"; things that run outside the sandbox read "not contained".
12. **Refusals are explained.** When something is refused under Moderate, the person can tell that the security level
    caused it and what they can do.
13. **Not available yet** (owner decision: Moderate ships with today's controls; these are shown as "not available"
    and are not tested): financial preview and approvals, isolated retrieval, screened search, brokered browser
    login, quarantine, the untrusted-content detectors and Agent Sweep.
14. **Switching away.** Choosing Permissive or Aggressive from Moderate shows what changes first. Back to Permissive
    takes effect at the restart and Moderate stays on until then; the person's own earlier settings come back.
    Aggressive applies its stricter rules when its preflight passes, the rest at the restart.
15. **Errors and recovery.** Moderate needs Corbanu's untrusted-content labels (a feature setting): without them
    Moderate is not saved and the screen says why. If the model-key broker cannot be used, model requests are refused
    rather than sent directly, and the message says what is involved and how to change it.
16. **What does not change.** Model and provider choice, including both Z.AI connections (`zai` and
    `zai-anthropic`): a normal turn on either should complete. MCP servers, apps, hooks and `!` shell commands run
    outside the sandbox, as under every level (the inspector says "not contained").

## Constraints for this test run

- Platform: macOS on Apple silicon, the packaged Corbanu Terminal candidate. Windows and Linux are not part of this
  run.
- The profile has the security features switched on (they are behind feature flags); the security level itself starts
  at Permissive and is chosen in `/security`.
- Testers drive the real terminal UI with real keys, in an isolated macOS virtual machine, as an ordinary
  unprivileged account with no real credentials. Each test group starts from a fresh profile and a fresh copy of
  the test project. The tester can also create or change files in the project from their own shell, as the person
  would in another terminal window, and can add a small local MCP server to the profile that returns text the tester
  chooses (run with the machine's `python3`).
- The test machine has no internet access: web pages and web search cannot be reached, GitHub cannot be reached,
  and Claude Code is not installed.
- Model: GLM 5.3 Flash through the Z.AI `zai` connection; the `zai-anthropic` connection is also provisioned. The
  test machine routes model traffic through a local relay; that relay's address (127.0.0.1:18443) can appear in
  command errors and is not part of the product. The model key reaches the product from the environment at launch.
  For error cases a test connection `zai-plain-http` is also available: the same models, with the provider address
  given as plain HTTP, which the credential broker cannot accept.
- The product's own model may decline a request by itself. That is the model, not a protection; testers record its
  exact words. Owner rule for judging: if the model declines a step but the product's text still meets the expected
  wording, the step counts as met; otherwise the case is blocked, not passed.
- Test data: a small git project `ledger-notes` (README.md, `src/notes.py` with `list` / `add` commands, `notes.txt`,
  a `.env` holding a synthetic token `NOTES_SYNC_TOKEN=...`, a `.gitignore`), and a file `outside-notes.txt` just
  outside the project folder. It stands in for the person's real trading project. All values are synthetic; there are
  no real accounts, wallets or money, and nothing may be bought, signed or sent.
- A macOS Keychain or other system password prompt must never appear.
- Out of scope for this run, by the owner's scope decision or pending owner decisions (named limits, not tests):
  the real trading project (a stand-in is used), Claude panes, Linux and Windows, and an agent command that itself
  starts another Corbanu Terminal (`corbanu exec` run by an agent command).
- There is no target number of tests: prioritize what an ordinary Moderate user does and the failures that would
  hurt them most.

## Screens from this candidate

See `screens/README.md`.
