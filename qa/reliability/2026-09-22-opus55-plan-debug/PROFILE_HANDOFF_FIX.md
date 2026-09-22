# Real debug profile handoff correction

The initial handoff verified an isolated QA profile but preserved the user's actual debug config on OpenRouter (`stealth/ox-alpha`). Consequently the bare launcher prompted for an OpenRouter key. That handoff was incomplete.

Corrected `/home/pfrpc/.corbanu-debug/config.toml` to provider `claude-plan`, model `claude-opus-5-5-plan`, retaining High effort and project settings. Backed up the old config under the evidence backups directory. In the actual debug profile, explicitly selected the existing Claude Code login through `/providers` to resolve ambiguous credential sources; no raw credentials copied. Launched the public debug wrapper with CODEX_HOME unset, as a normal shell would, and successfully received `PLAN_READY` from Opus 5.5 Plan. Evidence: `/home/pfrpc/corbanu-debug-evidence/real-debug-profile-plan-ready.txt`.

The original debug config hash now intentionally differs. Stable launcher, stable binary, and stable config still match baseline hashes. No source/runtime change or rebuild was needed. User must exit the already-open onboarding screen and relaunch so its old in-memory OpenRouter selection is discarded.
