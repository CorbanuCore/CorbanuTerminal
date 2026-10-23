You are the accounting demo-reel worker. Goal (Travis): one coherent, narrated demo video of the agent cost accounting
work (/cost), edited from the existing accounting demo videos, in the same house style as the P0 security reel.
First read demo-reel1.md and demo-reel1.result.md in /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/ (how the
P0 security reel was made and what Travis reviewed), sec-common.md there (binding: no secrets, keychain rule, disk
hygiene), and follow these skills: /Users/Neo/.local/share/portableSkills/library/skills/breeze-voices/SKILL.md,
/Users/Neo/.agents/skills/hyperframes/SKILL.md (entry point; routes to general-video), /Users/Neo/.agents/skills/media-use/SKILL.md.
Sources: wait until PR #332 is merged (poll `gh pr view 332 --json state` every 5 min, up to 2 h), then use
qa/demos/index/PF-60-S03.md (the current-main rows, not the superseded ones) plus any earlier accounting sprint demo
indexes (qa/demos/index/PF-60-*.md, PF-6x) that still show current behaviour; videos are on the `demos` release.
Acceptance criteria: the PF-60 sprint records (docs/sprints/**/pf-60-*.md) and the independent acceptance evidence in
qa/portfolio/agent-cost-accounting/pf-60-s03/independent-acceptance-20261008/ (README + rerun*/README).
Steps: 1) pick clips per topic: what /cost shows for a conversation and a request (drill-down, Back/Esc); history (a past
day, ranges by day/week, "cost so far" for ranges that include today, retention wording); robustness (concurrent exec
runs, clock behind the ledger, deleted conversations still counted); honest gaps (missing usage -> next step naming its
own provider's bill; no price); developer-only (default build has no /cost). 2) script: per segment, the acceptance
criterion in plain words and what on screen proves it; ~4-7 min; save the draft in the work dir. Say clearly that /cost
is a developer-only feature today, and that a follow-up sprint (PF-60-S05) will fix how subscription vs pay-per-use work
is recorded (don't claim it's done). 3) reuse the P0 reel's approved Breeze narrator voice from the voice library
(synthesize on the RTX facility). 4) edit as before: trim to proving moments, speed badges for dead time, side panel with
sprint + criterion, amber proof boxes, burned-in subtitles, embedded chapters, 1080p MP4. 5) QA as before (sync, STT
completeness, highlight placement, no key-shaped strings or /Users/ paths). 6) upload the MP4 + .srt to the `demos`
release, add qa/demos/index/accounting-reel.md (chapters + source clips) in a docs-only PR; merge when green
(`gh pr merge N --merge --match-head-commit <sha>`; never force-push; never rm -f). Do NOT publish to Cloudflare (that
waits for Travis's review). Work dir: .codex-work/accounting-reel-20261009/. Report: link, length, chapters, anything for
Travis to review.
