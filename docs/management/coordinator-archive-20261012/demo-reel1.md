You are the demo-reel worker. Goal (Travis): one coherent demo video of the P0 security-levels work, edited from the
existing sprint demo videos, with a Breeze-voiced narration explaining what the viewer is seeing and how each clip
meets the sprint's acceptance criteria.
Read and follow these skills first: /Users/Neo/.local/share/portableSkills/library/skills/breeze-voices/SKILL.md,
/Users/Neo/.agents/skills/hyperframes/SKILL.md (entry point; it routes to the owning workflow, likely general-video),
and /Users/Neo/.agents/skills/media-use/SKILL.md for media handling. Also read sec-common.md in
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ (binding: no secrets, keychain rule, disk hygiene).
Sources: videos on the GitHub release https://github.com/CorbanuCore/CorbanuTerminal/releases/tag/demos, indexed per sprint in
qa/demos/index/<SPRINT>.md; acceptance criteria in each sprint record (docs/sprints/archive/p0-security-levels/) and
gate records (qa/security-levels/sprints/<SPRINT>/gate.md); task map qa/initiative-control/p0-security-levels/tasknode-task-map.md.
Steps: 1) pick the best 1-3 clips per track (prefer GLM 5.2 runs) for: secretless launch/broker, secret output gate,
URL policy/pinning, protected-mode preflight & migration, taint, memory floors/downgrade/inheritance, /security picker
confirm/cancel/downgrade/restart, inspector, temporary grants, revocation & kill switch, saved-level qualification.
2) write a script: per segment, the acceptance criterion in plain words and what on screen proves it; aim ~8-12 min.
Show the script draft in the work dir. 3) cast a calm, clear Breeze narrator per the skill (audition 2-3, pick one,
save it to the voice library), synthesize narration on the RTX facility. 4) edit: trim clips to the proving moments,
speed up dead time, title cards + lower-third captions naming the sprint and criterion, burn-in subtitles of the
narration, chapter list; 1080p MP4. 5) QA: watch-through checks per the skills (sync, legibility, no secret values
or real home paths visible). 6) upload the final MP4 to the same `demos` release (gh release upload) and add
qa/demos/index/P0-security-reel.md (chapters + link) via a docs-only PR; merge when green
(`gh pr merge N --merge --match-head-commit <sha>`). Work dir: .codex-work/demo-reel-20261008/. Report the link,
length, chapter list, and anything that needs Travis's review.
