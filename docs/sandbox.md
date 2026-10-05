## Sandbox & approvals

For information about Codex sandboxing and approvals, see [this documentation](https://developers.openai.com/codex/security).

## Windows non-admin sandbox

On Windows, Corbanu Terminal can sandbox commands in two ways. The **default
sandbox** needs a one-time Administrator setup. The **non-admin sandbox**
(also called the unelevated or legacy sandbox) needs no setup, but it protects
less.

**What the non-admin sandbox does not stop.** A sandboxed command can delete
or move files and folders outside your workspace when they sit in a folder you
own, which includes most of your user profile. This applies in both Read Only
and workspace-write modes. Creating or changing files outside your workspace is
still blocked. The default sandbox does not have this gap.

**When you will see a warning.** If your sandboxed commands run in the
non-admin sandbox, Corbanu Terminal tells you once per session:

- in the TUI, as a warning at startup and after you choose the non-admin
  sandbox during setup; the setup screens and `/permissions` also describe the
  risk;
- in `corbanu exec`, as a `warning:` line on stderr;
- in `corbanu sandbox`, as a `warning:` line on stderr before the command runs;
- in app-server clients, as a configuration warning.

The warning never blocks a command.

**How to switch to the default sandbox.**

- In the TUI, run `/setup-default-sandbox` and approve the Administrator prompt.
  Corbanu Terminal saves the choice for later sessions.
- Or set the mode in `config.toml` in your Corbanu home folder:

  ```toml
  [windows]
  sandbox = "elevated"
  ```

  You can also pass `-c windows.sandbox=elevated` for a single run. If setup
  has not run yet, the first sandboxed command asks for Administrator approval
  once.

Your organization's managed requirements may allow only the non-admin sandbox.
In that case the warning still appears so you know the limits.

This page documents the shipped capability "existing general sandboxing" from
the [product specification](corbanu-product-spec.md) heading "Shipping MVP —
LIVE": "The shipping MVP already has a wallet, vault, scoped signing, approvals,
and general workspace sandboxing."
