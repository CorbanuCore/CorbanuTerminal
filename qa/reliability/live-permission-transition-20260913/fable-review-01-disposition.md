# Fable review01 disposition

Fresh Fable5.1High through Corbanu review wrapper in TMUX
`pf83-review-rsslte:review`, exact source-only clone
`/private/tmp/pf83-review.RsSlTe/repo`. Completed helper exit1 at
2026-09-14T06:40:21Z; structured output has two P2 findings, not a clean review.
Raw result files preserved alongside this disposition. Model request
claude-fable-5-1-plan; provider reports claude-fable-5-1; effort high.
Reviewed patch e36396dcd62ad8daf59832302ea69f10e24770ddab3d44ce1b99122ed69cf96b.
Review spend: original independent design1 + code review01 =2 of5.
One corrected-tree review and separate evidence check remain budgeted.

## Accepted introduced issues, same scope

1. Confirmed selection is not carried into fresh sessions. Parent read
permission_confirmation.rs, config_persistence.rs, thread_settings.rs and
fresh_session_config(): active-thread selection returns before updating App
defaults/runtime overrides; reply currently only persists reviewer; fresh
session clones stale App config. Concrete downgrade-then-/new can re-enable
old Full Access. Repair confirmed effective-state propagation while rejecting
failed/uncertain/stale selection state; never overwrite a newer observation
with stale requested settings. Cover both directions, built-in/named profiles,
and notification/reply ordering rather than merely adding optimistic mutation.

2. Held initial input lacks a completion path. Parent read profile-selection
callers, confirmation event handler and submit_initial_user_message_if_pending.
The active-thread asynchronous path returns false and no completion calls the
existing held-input release/recovery. Preserve the message and explicitly
resume only after valid successful confirmation when normal gates allow;
on failure/uncertainty restore or hold it visibly. Do NOT blindly follow the
reviewer's suggested submit-after-any-outcome: that could execute with the
wrong authority. Preserve attachments, mentions, queues and consent.

Both are in-scope blockers introduced in the allocated TUI confirmation flow.
No Core or new API change is authorized. Fresh Astra High repair worker receives
cycle1; code scope gains only existing chatwidget/input_restore.rs if required
for non-submitting recovery. Manager approves <=750 non-test/<=1650 total
hand-authored source/test changed lines for these fixes+regressions, preserving
old1250 baseline and generated accounting. This explicit exception does not
authorize refactors. Freeze a v2 patch/manifest/log without overwriting v1;
rerun affected tests and Fable review. After two repair cycles reclassify.

## Separate qualification preparation

Linnaeus returned and was closed. Uninstalled v1 macOS package exists at
`.codex-work/pf83-preparation.vwQlK8`; receipt records exact hashes. It is held,
not accepted and must be rebuilt after source correction. Synthetic native
preflight denied source/auth reads but FAILED network and cross-run IPC denial.
No independent functional executor was launched; no functional pass inferred.
Native confinement/mediated PTY remains an infrastructure prerequisite. Do not
reuse the old pilot as qualified or bypass its failed boundaries. Current app,
profile, launcher and canonical integration remain untouched.
