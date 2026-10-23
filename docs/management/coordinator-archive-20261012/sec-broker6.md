You are the broker lane worker (round 6). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first
(binding, incl. disk hygiene), then sec-broker5.result.md in the same folder (PF-27-S05 slice 1 merged as #229).

Finish PF-27-S05 "Model-client auth in the broker": with the flag on, route every remaining direct-key path through
the broker (web search, image generation, model catalog refresh, realtime websocket, other auth built from the OpenAI
login); decrypt vault-stored keys inside the broker, not Core; strip provider-key env vars from Core's environment
once handed to the broker; broker ChatGPT sign-in tokens; add a test that the raw key is not in Core's memory
(best-effort scan of the process heap/core dump in a test harness) and a test of the non-Unix branch's refusal.
Record this worktree in the plan front matter so the sprint can be `in_progress`, and archive PF-27-S05 when its gate
is complete. Never run live checks against the real Corbanu home: assert CODEX_HOME/CORBANU_HOME point at a disposable
dir before any live run. Per-sprint gate per sec-common.md; Linux clippy on the RTX box. PR to main; merge when green.
Report as in sec-common.md.
