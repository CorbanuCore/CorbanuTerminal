You are the accounting demo worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md
(binding: keychain rule, disk hygiene) and the repo demo SOP (qa/demos/README.md, scripts/demo_video.py), then
qa/demos/index/PF-60-S03.md and acct-s03a.result.md / acct-fix2.result.md / acct-fix3.result.md in the workers folder.
The three PF-60-S03 demo videos were recorded at 5ba0c02f6705, before the acceptance fixes (#299, #303, #305, #306, #318,
#322). They will be the source footage for a narrated accounting demo reel, so they must show CURRENT behaviour.
Build current origin/main (record the commit) with developer accounting (and a default build for item 5), disposable
homes, CORBANU_TEST_NO_NATIVE_KEYRING=1, GLM 5.2 on zai with the key only as
`ZAI_API_KEY="$(corbanu vault auth-helper provider/zai_api_key)"` on the consuming command. Record per the SOP:
 1. Re-record the three existing demos (same specs, updated where wording changed).
 2. New: a conversation with missing usage — its page names its own provider's bill as the next step (#288).
 3. New: deleted conversations — delete one with a subagent; the day page discloses "Deleted conversations or subagents
    sent N request attempts" (#286/#303), and delete works with the clock behind (#308) if that can be shown cleanly.
 4. New: a week/month range including today reads "Cost so far" / "In progress" and the retention wording agrees (#289).
 5. New (default build): `/usage requests` says "Per-request cost history is not part of this build" and `/cost` is
    absent (developer-only).
Keep each short (15-45 s), state any seeding on the title card, leak-scan every video (recorder scan + a second scan),
upload to the `demos` release, and replace/extend qa/demos/index/PF-60-S03.md (keep the old rows marked superseded).
Docs-only PR to main, merge when green (`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f).
Don't change PF-60-S03's acceptance state. Clean up build dirs and homes. Report: video links with one line each on what
it proves, the PR number.
