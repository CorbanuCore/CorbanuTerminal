You are the Task Node worker, round 4. Read tn-sec1.result.md, tn-sec2.result.md, tn-sec3.result.md and
tn-verify3.result.md in /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/, the round-3 kit in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261007/tn-sec3/ (README, create.sh, gen_records.py, gen_evidence.py,
submit.sh, poll.sh, respond.sh, tn.sh), the tasknode-usage skill
(/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/.system/tasknode-usage/) and sec-common.md (binding).
Same rules as before: never submit against closed/rewarded tasks; request right-sized new tasks for unmapped work; keep a
mapping; idempotent sends with a tracking tag; Task Node moves stages, we submit evidence.
New merged work since round 3 to cover (check each PR is MERGED on main and group sensibly, likely 5-8 tasks):
- P1 Windows security: PF-27-S06 real-Windows gate rerun + defects #294/#295 (#298 #302 #312), PF-27-S07 gate (#316),
  both archived with Travis's acceptance (#329); follow-up fixes #307 (#321), #320 (#327), #300 (#331), #301/#304 (#326),
  #341 Windows sandbox over SSH (#343); Windows clippy clean + CI leg (#317).
- PF-27-S08 Windows broker restricted token (#333) — sprint still in_progress pending Travis's key-path decision; say so
  honestly and scope the task to what shipped.
- Provider keys kept out of model-run commands #310 (#313).
- Accounting: PF-60-S03 acceptance fixes and independent acceptance (#299 #303 #305 #306 #311 #315 #318 #322 #328), archived
  with Travis's acceptance (#335); demo videos (#332) and narrated accounting reel (#336, published on Cloudflare);
  PF-60-S05 collection correctness (#330 #334 #337 #338 #342 #344 #346) with independent acceptance (#355) — S05 awaits
  Travis's acceptance, say so; /cost date hint + plain billed wording (#339).
- CI health: canary SIGSEGV race (#309), tmux smoke caching (#319), postmerge-ci 40-run red fixed (#350).
Request the tasks, wait for generation, generate records and evidence files (with PR links, merge SHAs, demo/video links,
and honest status for anything not yet accepted), add them to a sibling map under qa/initiative-control/ in a docs-only PR,
and merge when ALL checks are green (never with checks pending; `gh pr merge N --merge --match-head-commit <sha>`).
Do not submit evidence until Travis accepts; prepare the submit script. Never print secrets. Report the task ids, titles and
PFT amounts needing Travis's acceptance.
