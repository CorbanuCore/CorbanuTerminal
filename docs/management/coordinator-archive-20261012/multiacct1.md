You are the multi-account feasibility worker. Read sec-common.md in /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/
first (binding; in particular the keychain rule: Travis got repeated keychain popups when a worker touched his real login
keychain — avoid that). Question from Travis: can we run tmux workers on a DIFFERENT Anthropic (Claude Plan) account than the
coordinator? Hypothesis to test: a separate CODEX_HOME per account, each with its own selected Claude Plan source.
1. Code analysis first (origin/main, installed `corbanu` version): where is the vault stored and how is its key protected
   (keychain service/account names — are they per CODEX_HOME?), where is the Claude Plan source selection stored, and does
   `internal-claude-oauth-token` resolve strictly from the current CODEX_HOME? Also the "Claude Code login" source with a
   per-home CLAUDE_CONFIG_DIR. Write findings with file:line references.
2. Safe live test using the INSTALLED corbanu, on a throwaway home under .codex-work/multiacct-test/home2:
   - If the vault key for a new home would create/read a keychain item that is distinct per home, proceed; if it would touch
     the item the real home uses, STOP and report instead.
   - Store a FAKE, clearly-invalid Claude token in home2 via the supported flow (non-interactive if possible), select it,
     run one `corbanu exec` with CODEX_HOME=home2 on claude-plan and confirm it fails with an auth error (proving home2's
     token was used), and confirm the real home still works (one tiny `corbanu exec` with the real home) and that its
     /providers or vault metadata (fingerprint, not value) is unchanged before/after.
   - Never print token values. Delete any keychain item you created, list exactly which (service/account) you created and
     removed, and remove the throwaway home.
3. Report: does workaround 1 work, exact steps for Travis to set up a second account (he'll need to run `claude setup-token`
   on the second account), how workers would be launched with it, and caveats (accounting attribution, shared caches,
   keychain prompts). Write to /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/multiacct1.result.md. No PRs needed.

Note from Travis: the eventual feature must support multiple accounts for ANY provider, not just Anthropic. In your report, also note for each provider auth type you find (API-key env/vault label, auth.command, Claude Plan token, Claude Code login, OpenAI/ChatGPT login, AWS profile) whether the per-home workaround applies and what a generic "named accounts per provider" design would need to touch.
