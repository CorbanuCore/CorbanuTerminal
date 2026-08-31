# Reliable Claude Plan authentication

## The pain

Claude Code login credentials rotate. A missing or blank refresh token can
interrupt Corbanu with a provider-auth failure, especially when several Claude
processes share the same login. Corbanu Terminal therefore makes the credential
source explicit and recommends Anthropic's longer-lived subscription-token
flow for stable model requests.

## Choose an authentication method

Run `/providers`, select **Provider: Claude Code Plan**, and choose one method:

| Method | When to use it | What Corbanu stores |
| --- | --- | --- |
| **Long-lived subscription token (Recommended)** | You want fewer rotating-login interruptions. Available through `claude setup-token` for Pro, Max, Team, and Enterprise subscriptions. | The token in Corbanu's encrypted vault plus metadata naming this exact source. |
| **Claude Code login** | You prefer to share Claude Code's normal account login and accept that reauthorization may be needed more often. | Metadata selecting Claude Code's platform-owned credential; Corbanu does not copy that credential into its vault. |

The first option is selected by default. Press Esc to leave the existing method
unchanged. Corbanu changes the selected account and billing path only after the
new method succeeds, and it does not silently fall back to the other method.

## Recommended long-lived token

Claude Code must be installed. When you choose the recommended method, Corbanu
opens masked token entry and asks you to run this command in a separate private
terminal:

```text
claude setup-token
```

Complete Anthropic's authorization there. Claude Code displays the resulting
token in that private terminal; Corbanu does not execute or capture the
token-producing command. Paste the token into Corbanu's masked entry view and
press Enter. Corbanu encrypts it in the existing vault
and selects it as one transaction. If saving fails, the previous token,
metadata, and selected method are restored.

If setup is no longer making progress, cancel Claude Code with Ctrl-C in the
private terminal. Esc cancels Corbanu's masked handoff without changing the
previous method. Command failures remain outside Corbanu's terminal history;
rerun the command or choose the compatibility method explicitly.

The token is currently intended for Claude model requests, lasts approximately
one year, and is not an Anthropic API key. It does not add Claude Desktop or
Claude.ai cloud-only capabilities. Anthropic can change eligibility or lifetime;
follow the current
[Claude Code authentication guidance](https://code.claude.com/docs/en/authentication)
when generating a replacement.

### Replace or remove it

To replace the token, open **Provider: Claude Code Plan**, choose the recommended
method, run `claude setup-token` again, and save the new token in the masked
view. A successful save replaces the previous Corbanu-managed copy atomically.

To switch away, first choose **Claude Code login** successfully. You may then
delete the old managed copy through `/vault` using the label
`provider/claude-code-oauth-token`. Deletion is local only; it does not revoke a
token at Anthropic. Generic reveal, export, copy, replace, and programmatic
vault access to this provider-managed label are blocked, so inspection remains
metadata-only and replacement always uses the validated transactional flow.

## Claude Code login compatibility

If Claude Code is already signed in to a Claude subscription, Corbanu verifies
both that status and the exact platform record's refreshability before selecting
it without starting another login. Otherwise, it starts the normal Claude Code
browser login. Authorization codes are entered only in a masked view. A status
response cannot replace the previous Corbanu selection when the authoritative
record is missing, malformed, or lacks its refresh token or required scopes.

Corbanu follows Claude Code's current platform ownership:

| Platform | Authoritative login store |
| --- | --- |
| macOS | Keychain service `Claude Code-credentials`, or `Claude Code-custom-oauth-credentials` for custom OAuth. When `CLAUDE_CONFIG_DIR` is set, Claude Code appends the first eight hex characters of the normalized directory's SHA-256 digest. Corbanu persists that exact service identity. |
| Linux and Windows | `${CLAUDE_CONFIG_DIR:-~/.claude}/.credentials.json`; Corbanu persists a digest of the exact absolute normalized profile path so changing `CLAUDE_CONFIG_DIR` fails closed. |

On macOS, the current Keychain record wins when it exists. If that Keychain
record is absent and the same profile has a legacy `.credentials.json`,
Corbanu can preserve or explicitly select that exact file identity; it does not
copy or delete the record. `CLAUDE_CONFIG_DIR` selects both a distinct hashed
Keychain service and a distinct file-profile digest. After a method is saved,
Corbanu verifies that exact store identity and never falls through to the other
account.

Existing installations without a saved Corbanu choice retain their historical
behavior: a nonblank `CLAUDE_CODE_OAUTH_TOKEN` is used first, otherwise the
current platform store is used, including a macOS file-only legacy profile when
the corresponding Keychain record is absent. Once you successfully choose a
method in `/providers`, that exact source is persisted; a failure never falls
through to the environment or another store.

## Failure and recovery

The Claude Plan row in `/providers` reports the selected source without showing
its value. Missing managed tokens, missing environment values, blank refresh
tokens, malformed credentials, unavailable stores, and ambiguous state lead to
a source-specific recovery view.

Recovery offers four inert-by-default actions: retry long-lived setup, choose
an authentication method, explicitly resume a currently nonblank legacy
`CLAUDE_CODE_OAUTH_TOKEN`, or keep the current method. The legacy action does
not alter any credential: it persists the environment variable as the exact
selected source, so a later missing value fails closed instead of falling
through to a Claude Code login. Esc also keeps the current method. After a
successful recovery, retry the interrupted request or reselect the Claude Plan
model. Restarting Corbanu preserves the exact selected source; it does not
persist the raw token in chat or session history.

For a broken Claude Code login, run `claude auth login` (or `/login` inside an
interactive Claude Code session), then return to `/providers` and explicitly
choose **Claude Code login** again. A missing or blank refresh token requires
reauthorization even while the current access token has not yet expired;
Corbanu does not represent a source guaranteed to fail at its next refresh as
healthy.

## Security boundary

- Never paste a token into chat, a slash command, config, shell history, or a
  support transcript.
- Corbanu's masked entry, encrypted vault, and provider-auth helper are the only
  managed-token path.
- `/providers` and `/vault show` expose metadata only for the managed token.
- OpenAI account state, Anthropic API keys, Bedrock/Vertex credentials, and
  unrelated vault entries are not changed by this flow.

## Related documentation

- [`/providers`, account login, and `/model`](model-providers.md)
- [`/vault` and credentials](vault.md)
- [Authentication and account setup](../authentication.md)
- [Claude headless panes](claude-headless-panes.md)
