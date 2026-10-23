You are an accounting price-data worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
(binding: keychain rule, disk hygiene, per-fix gate) first.
Fix GitHub issue #361 (`gh issue view 361`): with an OpenAI API key (built-in `openai`, pay per use), `/cost` estimates use
native-catalogue rates that differ from OpenAI's published price page (gpt-5.6-luna ~5x too high, gpt-5.6-terra 25% too
high), cache writes have no rate although OpenAI reports and bills them (`input_tokens_details.cache_write_tokens`), and
gpt-5.4-mini, gpt-5.2, gpt-6-luna have no price at all. Evidence:
qa/portfolio/agent-cost-accounting/pf-60-s05/independent-acceptance-20261009/rerun-20261009/ (PR #362).
Required end state:
- Read the current official prices yourself from OpenAI's pricing page (https://developers.openai.com/api/docs/pricing and
  the model pages) and record the source + retrieval date. Correct every OpenAI API-key rate in the price catalogue the
  accounting uses (input, cached input, cache write, output; Standard tier; keep Priority/Flex either correct or explicitly
  "no price" — never a wrong price), and add the missing models that OpenAI publishes.
- Cache writes are priced and included in estimates for every model where OpenAI bills them.
- Prices are versioned/dated per the ledger's existing price-record rules (S05 made the format versioned); history is
  never re-priced. Check how other providers' prices are sourced and whether there's a refresh tool; if the catalogue is
  generated, fix the generator/source, not just the output.
- While there, audit the other built-in pay-per-use routes the defaults table lists (zai, deepseek, openrouter-pass-through,
  anthropic API key, moonshot/kimi platform, bigmodel) against their official price pages and fix any mismatch the same way,
  or list them in the PR if out of scope.
- Regression tests: the exact recompute from the re-run (`captures/mac/mac-r1-t1-conv.txt`: published-rate total
  $0.00625965) and a cache-write case.
- Real check: one small `gpt-5.6-luna` request with the OpenAI key (vault label `openai-api-key`, passed as CODEX_API_KEY on
  the consuming command; resolve it via
  `env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper openai-api-key`
  and never print it) in a disposable home with a developer-accounting build: `/cost` must equal your recompute from the
  provider-reported usage to the micro-dollar.
Gate: focused tests with and without developer-accounting; Linux clippy -D warnings on the RTX box (ssh ambient@100.99.88.49);
one independent Opus 5.5 High review via installed `corbanu exec` (read-only). New worktree off origin/main under
/Volumes/CorbanuDrive/Corbanu/worktrees/. PR to main; merge only when ALL checks are green, never with checks pending
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Close #361 with evidence. Remove build
targets when done. Report as in sec-common.md.
