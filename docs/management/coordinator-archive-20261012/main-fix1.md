You are a CI-repair worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; Linux
work on glitch, `ssh glitch`). Main's postmerge-ci is RED since PR #453 (#443: under Aggressive, agent `corbanu vault ...`
commands are refused from the moment Aggressive is confirmed) merged: the test
`suite::provenance::pf_30_s03_vault_command_after_untrusted_content_needs_the_human` (codex-rs/core/tests/suite/provenance.rs
~line 630) fails with "the vault command asks the human", because under Aggressive the vault command is now refused before
the PF-30-S03 post-taint prompt. Read PR #453, its last comment, issue #443 with Travis's decision comment ("protections apply
immediately; vault refusal must hold from the moment of confirmation"), and the PF-30-S03 archived sprint record.
Fix the TEST (not the product) so it asserts the correct behaviour per level: under Aggressive the vault command is refused
outright (no human prompt; nothing runs), and under the level(s) where PF-30-S03's post-taint human prompt still applies,
keep asserting the prompt — split into two tests if clearer. Do not weaken either guarantee. Also run the full
`just test -p codex-core` on glitch (Linux; that's where postmerge failed) to check nothing else on main is red after the
day's Aggressive merges (#448-#459, #464), and fix any other test that fails for the same reason (expectation predates an
intended, decided behaviour change), filing an issue for anything that looks like a real product bug instead. Check the latest
postmerge-ci run on main (`gh run list --workflow postmerge-ci.yml`) for other failing jobs too. Gate: focused tests; Linux
clippy -D warnings on glitch; PR to main; merge only when ALL checks are green, never with checks pending
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Then confirm the next postmerge-ci on main
passes (or report what still fails). Remove build targets when done. Report as in sec-common.md.
