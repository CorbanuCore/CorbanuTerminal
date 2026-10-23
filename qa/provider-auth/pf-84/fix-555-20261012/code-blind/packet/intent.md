# Feature packet: named provider accounts — forks, resume and recovery (Corbanu Terminal)

Corbanu Terminal is a terminal (TUI) coding agent for macOS and Linux. With the `named_accounts` feature on, one model
provider (for example Z.AI) can hold several accounts besides `default`, such as `work` and `personal`. Accounts are
enrolled with `corbanu account add <provider> <name>` (the key is read from stdin and stored in an encrypted vault) and
listed with `corbanu account list`. `[provider_accounts] <provider> = "<name>"` in config.toml picks the account new
sessions use; `--account <name>` (or `<provider>:<name>`) picks one for a session; inside the TUI, `/providers` lets the
user add accounts and pick one. `/status` shows the account on the "Model provider" line and an "Account usage" line
with tokens per account. Each conversation records the provider and account it ran on.

## What the user wants from this change
1. Forks follow the conversation's account. `/fork`, editing an earlier prompt (which forks), a side conversation
   (`/side`), and `corbanu fork <id>` run on the account the source conversation runs on, and `/status` names that
   account. Before, the fork's tokens were charged to another account while `/status` claimed the source's account.
2. Resuming a conversation recorded on a provider other than the home's configured provider (e.g. the home is set up
   for OpenAI, the conversation ran on Z.AI account `work`): it resumes on the recorded provider and account when this
   home has credentials for them. Otherwise it names the provider/account and how to recover, and does not show the
   sign-in (onboarding) screens of the home's provider.
3. If `[provider_accounts]` names an account that does not exist, `corbanu resume` of a conversation recorded on an
   existing account still works on that recorded account. If the recorded account is missing too, it is named with the
   recovery (`corbanu account add …`).
4. Inside the TUI, the missing-account message points to `/providers`; on the command line it keeps pointing to
   `corbanu account add` / `--account`.
5. Esc in the `corbanu resume` picker, in a home without usable default credentials, must not lead to the sign-in
   screens. Inside a running session, Esc in the `/resume` picker returns to the session.

## Constraints
- Precedence for a provider: an explicit `--account` for that provider wins, then the account a resumed conversation
  recorded (or a fork's source conversation / a worker's parent runs on), then `[provider_accounts]`, then `default`.
- With `named_accounts` off, behaviour must be unchanged.
- A missing account is an error, never a silent fallback to `default`. Secrets are never shown.
- Users: developers running the TUI in a terminal, fresh or existing homes, sometimes several providers configured.

## Screens (text captures of the real TUI; `CANDIDATE` = this change, `EARLIER-BUILD` = before it)
01 /fork then /status; 02 /side then /status; 03-04 resume of a Z.AI conversation in an OpenAI-configured home, then
/status; 05 resume with `[provider_accounts] zai = "ghost"` then /status; 06-07 `corbanu resume` picker and Esc in a home
with no default key; 08 Esc in `/resume` inside a session; 09 in-app `/resume` of a `work` conversation then /status;
10 in-app `/resume` of a conversation whose account was removed; 11-15 the same situations on the earlier build.
