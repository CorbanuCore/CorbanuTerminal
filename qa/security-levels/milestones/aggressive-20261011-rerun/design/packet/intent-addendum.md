# Aggressive security level: owner decisions of 2026-10-11 (addendum to the earlier brief)

The product owner made these decisions after the first test run. They change or add to what a person who picks
Aggressive should experience. The earlier brief (`intent.md`) still holds where these do not change it.

## Changed or new behaviour

1. **Protected reads can never be granted.** Under Aggressive, agent commands can never read protected credential
   files (a project's `.env`, SSH, cloud and CLI tokens) or Corbanu's own sign-in, vault, settings and history, not
   even with "Grant once". When such a read is denied, the person is told right there that the security level
   blocked it, that it cannot be granted, and how they could change it (switch the level in `/security`). The review
   and inspector say the same. "Grant once" stays available for other approval prompts.
2. **When protections start.** Some protections apply the moment the person confirms Aggressive (for example agent
   use of the `corbanu vault` command is refused and Corbanu's home is protected); the rest start at the next start
   of Corbanu Terminal. The confirmation screen names which ones wait for the restart (sandbox, approval for every
   command, network off, environment stripping, model-key broker, child-agent rules). It must not claim the session
   "stays Permissive" while some protections are already active.
3. **Corbanu's home cannot be listed.** Agent commands cannot list the contents of Corbanu's home folder (not just
   read the protected files in it), from the moment of confirmation and after a restart. The denial is explained the
   same way as in 1. Corbanu Terminal itself keeps working normally.
4. **Credential files created later are protected too.** A `.env` file (also `.env.*` and `.envrc`) that appears
   after the Aggressive session started, for example one the person creates in the project while the session runs,
   is as unreadable to agent commands as one that existed at start. The screens describe these files as including
   ones created later.
5. **Agent settings folders stay read-only.** The project's own `.codex` and `.agents` folders are always read-only
   to agent commands under Aggressive; "Grant once" does not make them writable.
6. **Kill switch stops work.** With the kill switch on, nothing is sent to the model: a new message is refused and
   not kept, with a message naming the kill switch and how to turn it off. Turning the kill switch on also
   **interrupts** whatever is running at that moment, in every pane: a reply that is streaming is cancelled, a
   command that is running is stopped, and a command waiting for approval never runs; the pane says it was stopped by
   the kill switch. This includes Claude panes (panes running Claude Code inside Corbanu Terminal). Messages that
   other agents send while the switch is on wait and are delivered after it is turned off.
7. **Going back to Permissive.** Under Aggressive the provider key is handed to a separate credential broker and is
   not handed back. So when the key came from the environment, "restart now" is not offered for the return to
   Permissive; the screen says to quit and start Corbanu Terminal again from the shell.
8. **Environment.** Under Aggressive, agent commands get only an allowlist of environment variables; ordinary
   variables outside it are removed too, and the review says so.

## Constraints for the new cases

- Same machine, candidate kind, profile, fixture, model and relay as `intent.md`, with a newer candidate build.
- Claude panes need Claude Code installed in the test machine; whether it is available is stated in
  `claude-availability.txt`. If it is not, cases that need a Claude pane will be recorded as blocked.
- The tester can create or change files in the project from outside the product with their own shell, as the
  person would in another terminal window.
