# Aggressive security level: owner decisions of 2026-10-13 (second addendum)

The product owner made these decisions after the second test run. They change or add to what a person who picks
Aggressive should experience. `intent.md` and `intent-addendum-20261011.md` still hold where these do not change
them.

## Changed or new behaviour

1. **Grant once works only inside the current folder.** Under Aggressive, agent commands can write only inside the
   current project folder. Some files there are protected even so, because they run code later; "Grant once" lets
   one command write such a file, once. The person's user documentation names, as examples: a git hooks folder set
   with `core.hooksPath` (such as Husky's `.husky`) or a git folder with another name; and, only when the current
   folder holds the person's home folder, shell and git start-up files such as `.zshrc`. The project's own `.git`
   folder, `.codex` and `.agents` stay read-only even with Grant once.
2. **No Grant once for a start-up file outside the current folder.** A command that writes one of the person's
   start-up files outside the current folder (for example `~/.zshrc` while working in a project) is not offered
   "Grant once". The denial explains why: the security level, that the file is a start-up file outside the current
   folder, and that this cannot be granted, not even once.
3. **Remembered approvals from a less strict level are suspended.** On an approval prompt a person can choose "Yes,
   and don't ask again for commands that start with ...". If that was chosen while a less strict level (Permissive)
   was active, it must not auto-approve anything while Aggressive is active: the person is asked again. The
   remembered approval is suspended, not deleted.
4. **No links into the agent settings folders.** Under Aggressive, an agent command that would create a link
   pointing into the project's `.codex` or `.agents` folder is refused, and nothing is created.
5. **Choosing the connection in `/model`.** When a model is offered over more than one connection (GLM 5.3 Flash
   over Z.AI's OpenAI-style connection `zai` and its Claude-style connection `zai-anthropic`), `/model` lists it
   once per connection and picking one switches the session to it. The OpenAI-style connection is the default.
   This works at every security level.

## Constraints for the new cases

- Same machine, candidate kind, profile, fixture, model and relay as `intent.md`, with a newer candidate build.
- The tester can create or change files in the project, and run `git config` there, from outside the product with
  their own shell, as the person would in another terminal window.
- The product's `~` in the test machine is the tester account's own home folder, which is wiped after each run.
- Claude Code: see `claude-availability.txt`.
