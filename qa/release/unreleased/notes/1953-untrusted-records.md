## Windows sandbox

Sandbox launches that need integrity or planted-project records now refuse existing records owned by an untrusted account, including files left by older elevated sandbox commands. The elevated watcher no longer silently disables tracking for this refusal. Keep the records for inspection and follow the recovery instructions in `docs/sandbox.md`; changing their owner does not make their contents trustworthy. D599 retains the approved `tsx-*` cache naming policy.
