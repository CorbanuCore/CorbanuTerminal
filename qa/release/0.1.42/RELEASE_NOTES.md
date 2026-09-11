Corbanu Terminal 0.1.42 fixes wallet startup and model-request recovery, and adds Team Context plus DeepSeek V4.1 Flash to Corbanu API.

- Adds shared collaborator work summaries through `/tasknode team` and `corbanu tasknode team context --json`, with refresh and report freshness indicators.
- Updates the bundled Task Node skill so agents can retrieve Team Context while preserving the active account and sharing permissions.
- Fixes wallet startup in standalone installations where the client looked for the old daemon filename.
- Keeps models in their correct provider tabs, including Claude Plan.
- Improves Campaign Tracker error handling and recovery.
- Adds DeepSeek V4.1 Flash with High reasoning under `/model` → Corbanu API.
- Recovers safely from confirmed released API attempts without the misleading duplicate-request 409.
- Omits the unsupported parallel-tool control on Flash requests while preserving tool use.

The matching gateway fixes are already deployed. Restart Terminal after updating. This release preserves your existing profile and credentials.

The fixes passed focused automated tests and live terminal checks. Cross-platform archives also run packaged wallet-client startup checks before publication. The broader competitor/model benchmark cycle remains incomplete; no benchmark pass is claimed. [Release evidence](https://github.com/CorbanuCore/CorbanuTerminal/blob/rust-v0.1.42/qa/release/0.1.42/RELEASE.md).

After updating, start a new chat and ask: “Refresh my Team Context and tell me what each collaborator is working on.” Team Context summarizes shared rewarded work; it is not a live feed of every unfinished task.
