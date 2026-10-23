You are a bug-investigation worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
(binding: keychain rule, disk hygiene, per-fix gate) first.
Problem: with an OpenAI API key on the built-in `openai` provider, Corbanu Terminal does NOT offer the image-generation tool
to the model, so a user who pays for OpenAI API credits can't generate images. Evidence: the PF-60-S05 re-run (PR #362,
qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/rerun-20261009/README.md, AC4 image row):
tool lists in `response.created` for gpt-5.6-luna, gpt-5.6-terra, gpt-6-luna, gpt-5.4-mini and gpt-5.2 contained no image
tool, although `features list` says `image_generation stable true`. Yet
codex-rs/ext/image-generation/src/extension.rs marks it available when `config.model_provider.is_openai()`.
1. Find exactly why the tool isn't offered on the API-key route (model capability metadata in models.json / spec_plan.rs,
   auth-mode gating, exec vs TUI, the images backend endpoint choice, the extension not being installed in `exec`, etc.).
   Compare with upstream openai/codex behaviour for API-key users (OpenAI's Images API works with API keys; check how the
   backend chooses the endpoint for ChatGPT login vs API key). Write the root cause down with file/line evidence.
2. Fix it so API-key users get image generation through OpenAI's Images API (correct endpoint, auth, model, and cost
   accounting via the existing ExtensionAccounting at OpenAI's published image prices), unless there's a real reason it
   can't work — then make the product say so plainly instead of silently omitting the tool. Don't change ChatGPT-login
   behaviour.
3. Also check realtime voice (`realtime_conversation` is "under development, false"): report in one paragraph what it would
   take to enable it for API-key users; do NOT implement realtime in this task.
Regression tests (tool offered on the openai API-key route; request goes to the right endpoint with the key; accounting
records it). Real check: ONE smallest-size image with the OpenAI key (vault label `openai-api-key`, passed as CODEX_API_KEY
on the consuming command; resolve via
`env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper openai-api-key`;
never print it) in a disposable home with CORBANU_TEST_NO_NATIVE_KEYRING=1; confirm the image is saved and `/cost` records it.
Gate: focused tests; Linux clippy -D warnings on the RTX box; one independent Opus 5.5 High review via installed
`corbanu exec` (read-only). Another worker (acct-price1) is fixing OpenAI price data (#361) — coordinate by merging
origin/main often and not editing the price catalogue. PR to main; merge only when ALL checks are green
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). File/close an issue with evidence.
Report as in sec-common.md, with the root cause in plain words for Travis.
