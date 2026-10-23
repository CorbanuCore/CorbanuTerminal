# Moderate picker sprint: tmux check (2026-10-14)

This is the manual tmux check of the Moderate picker sprint (#750, #747, #749): choosing Moderate in
`/security`, confirming, restarting and going back to Permissive. The candidate was a debug build of
`mod/750-moderate-picker` at 88c131618e, stacked on #764.

Later review fixes changed two things. On these screens they changed only wording: the Moderate effects line, the
Nested agents row and the vault row. They also changed behaviour on paths this check doesn't exercise: Moderate kept
by config or by Core's record, `/status`, and the saved screen when a level is lowered before restarting. Unit and
snapshot tests and the videos listed in `qa/demos/index/issue-750.md` and `issue-749.md` (final build) cover those
paths; these captures don't.

**Setup:** a disposable home and `HOME` (`<run>`), `CORBANU_TEST_NO_NATIVE_KEYRING=1`, GLM 5.3 Flash on `zai` (key
from the vault at launch), the security features on (`security_levels`, `protected_mode_preflight`,
`secretless_agent_launch`, `isolated_credential_broker`, `secret_output_gate`, `source_envelopes`,
`url_destination_policy`), `approval_policy = "on-request"`, and no `[security]` level in config.

| Capture | Shows |
| --- | --- |
| `01-picker` | Moderate is offered, not "not available yet" |
| `02-review-moderate-top` | The review: what Moderate adds, what's not available, and when it applies (Core's rules once confirmed, the broker at restart) |
| `03-saved-moderate` | Saved. Core's Moderate rules are active now, and `r` restarts |
| `04`, `05` | #747, before the restart: an approved `require_escalated` write to `../outside-notes.txt`. The prompt says the write stays inside the sandbox. It is refused with `Operation not permitted` (not "Protected launch refused") and the file isn't written |
| `06`, `07` | D5: a nested `corbanu exec` started by an agent command stops at start because it can't read Corbanu's config (#766) |
| `08`–`10` | Before `r` (`08`): "Active in this session: Permissive", with "Saved: Moderate. Core's Moderate rules apply in this session now". After the restart (`10`): "Active in this session: Moderate" |
| `11`, `12` | #749: the inspector under Moderate |
| `13`, `14` | Back to Permissive: what's removed, saved, Core stays Moderate until the restart, and "restart now" is unavailable because the broker took the provider key (#436) |
| `15` | After `/quit` and a new start: Permissive active, Core Permissive |
