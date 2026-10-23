You are the Task Node worker, round 3. Read tn-sec1.result.md, tn-sec2.result.md, the tasknode-usage skill
(/Volumes/CorbanuDrive/Corbanu/.codex-work/corbanu-terminal/home/skills/.system/tasknode-usage/) and sec-common.md (binding).
Same rules as before: never submit against closed/rewarded tasks; request right-sized new tasks for unmapped work; keep a
mapping; idempotent sends with a tracking tag; Task Node moves stages, we submit evidence.
New merged work to cover: PF-27-S06 Windows broker and secretless launch (#267 #269 #270 #272 #277; note the gate is
incomplete: GLM tmux run and videos pending a Windows machine; say so honestly in the task scope and evidence);
the manager-loop recovery and hardening (#263 #265 #268 #271 #275 #276); the narrated P0 security demo reel (#274).
Request tasks (likely 2-3), wait for generation, write evidence files, add them to the map
qa/initiative-control/p0-security-levels/tasknode-task-map.md (or a sibling map) in a docs-only PR, merge when green.
Do not submit evidence until Travis accepts; prepare submit.sh-style script. Report task ids needing acceptance.
