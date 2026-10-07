# Review disposition (Opus 5.5 High, request changes)

1. **High, bridge could send the key to any host:** fixed. The passthrough bridge forwards only `/v1/messages` and `/v1/messages/count_tokens` (plain query only) and builds the URL on the base URL's own scheme, host and port; anything else gets 404. Origin-form targets must start with `/`. This also hardens the existing Vercel and Claude Plan bridges. Test: `passthrough_bridge_forwards_only_messages_routes_to_its_own_host` (`@host`, `.host`, `:port`, `//host`, `..`, `@` in the query, fragments, other routes, absolute form with userinfo).
2. **Medium, other panes' transcripts readable:** fixed. A contained launch denies reading `CODEX_HOME/panes`; Claude Code gets its settings from a read-only `settings.json` in its own state folder. Test in `base_profile_*`; probe in the tmux runs.
3. **Low/Medium, key fetched before the launch is checked:** fixed. `contain()` now runs right after the level check, before the vault, the Claude Plan helper or the bridge.
4. **Low, non-UTF-8 variable panics:** fixed (`vars_os`, non-UTF-8 entries dropped).
5. **Low, feature read once at start:** recorded. The feature takes effect at start, like the other security features; README says so.
6. **Info, absolute-form handling on uncontained bridges:** intended; those bridges get the stricter checks of finding 1 too.
7. **Test gaps:** hostile targets (finding 1) and the profile's denials (finding 2) are now tested. `contain()` with an armed contract is covered by the core test of `protect_external_launch` and by the macOS and Linux tmux runs, not by a TUI unit test: the contract is a process-wide `OnceLock` in core. Recorded.

## Second round (approve with fixes)

1. **Medium, sibling panes' Claude state readable:** fixed. The folder holding every pane's state for this Corbanu home is denied; the pane's own folder, more specific, stays writable. Test in `base_profile_*`; probe 7 in the tmux runs.
2. **Low, evidence:** the macOS and Linux runs were recorded again on the final tree with probes 6 (`panes`) and 7 (sibling state). The hostile-target cases are covered by the bridge unit test, not a raw request in the runs.
3. **Low, demo script:** fixed. Each step of the vault popup must be on screen, or the script stops; the key is only pasted into the masked field. The candidate keeps `ZAI_API_KEY`: the spec's main model uses it.
4. **Info, settings write:** fixed; the file is written to a temporary file in the folder and renamed over `settings.json`.
