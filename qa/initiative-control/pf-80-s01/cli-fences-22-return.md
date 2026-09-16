# RETURN — tn-cli-fences-22

Allocation digest: `4faca8af2cae6f6c8b4f717ebf1beadf474812ac4a1accf5619bd5ae1acfe5ea`; claim `cfc352d9-e4cf-49f5-b708-be7464d2683a`; runtime gpt-6-astra/high.
Base verified clean: `5a11045e742e67247ef1c672484821f5f8415e77`; brief SHA-256 verified with shasum: `143fb42c22286b15062822b095f8961a8c373be8f7c3cf8e97c77b578dbf37cc`.
Worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/tasknode-cli-fences-20260916`.
Class: bounded correction within active plan `initiative-delivery-control.md`, PF-80-S01 in_progress. Product heading **Internal delivery control — TO BUILD**: “Task Node receives only explicitly mapped, supported progress”.
Tick counts invalid records in the loop and appends the inspection sentence once afterward, preserving the preceding lock/rejection error.
New two-record regression checks exact combined text and one occurrence for no preceding error, report rejection, and queue-lock refusal.
New publication regression exercises **100 invalid records**, one receipt-backed uncertain record and a lock refusal, real publish/collect/render and the 2000-character safe_text guard; only source checkers and enqueue/initial run collection are stubbed.
That fixture measured 7224 characters before the fix; afterward the size assertion, real collector retention and rendered single corruption notice/count/lock/uncertainty all pass, with no unknown-delivery fallback.
Successful flush removes `error`. The same line fixes explicit uncertain retries and transient failures; coverage extends the preserved-explanation retry test through delivered/4 and adds actual synthetic 503/backoff-expiry/success through delivered/2.
All previous tests retained. Four regression methods ran against unchanged implementation: six failing assertions/subtests, zero errors; all four passed after the repair.
Recorded, not fixed: receipt presence can override validation of damaged record status, so receipt-derived status can mask field corruption; file-level damage still reaches invalid handling.
Recorded, not fixed: tick lock refusal and argparse usage errors both exit 2; their diagnostic wording distinguishes them.
Recorded, not fixed: activation starts the publish service with check=True after swapping the source pointer, so transient contention can abort installation after that swap. Whether activation should tolerate a transient publish-service failure is a real question for the release owner.
Isolation guidance read before tests. Fresh disposable root R: `qa/initiative-control/pf-80-s01/.tn-cli-fences-22.ok0wyB`; no pre-existing venv reused.
Environment E: `env -i PATH=/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin HOME=<absolute-R> TMPDIR=/private/tmp PYTHONDONTWRITEBYTECODE=1 CODEX_HOME=<absolute-R>/profile CORBANU_HOME=<absolute-R>/profile PFTERMINAL_HOME=<absolute-R>/profile CORBANU_TEST_DISABLE_NATIVE_KEYRING=1 PYTHONPATH=<worktree>/scripts/initiative_control`.
Built with `E /opt/homebrew/bin/python3 -m venv <absolute-R>/venv`; installed with `E <absolute-R>/venv/bin/python -m pip install --no-deps -r scripts/initiative_control/requirements.txt`.
Installed application packages only: markdown-it-py==3.0.0, mdurl==0.1.2, slack-sdk==3.44.1; bootstrap pip==26.0.1.
Final gate command: `E <absolute-R>/venv/bin/python -m unittest discover -s scripts/initiative_control -p 'test_*.py' -v`: **709 tests, 1 failure, 1 error**, 484.523s, exit 1; all **100** Task Node/tick/preparation/control tests passed. Full-suite gate not satisfied.
Remaining failure/error names:
- `test_owner_tmux.TmuxTests.test_bridge_incomplete_and_malformed_rollouts_never_issue_evidence (mode='partial')`
- `test_owner_tmux.TmuxTests.test_bridge_incomplete_and_malformed_rollouts_never_issue_evidence (mode='malformed')`
Raw logs preserved as cli-fences-22-{before,after,suite-final}.log.gz. Uncompressed SHA-256 respectively: `4809b7f31c7088852a732b60c71a4d297460c4cf209bd781a83ef7cde414897c`, `aaacd7a15d63ca5bd455cc843e43c1cc213c380b78fae45cc2bdbd8dd535ce46`, `f3c52d80dc55c131fef28fef4125bcd357b3cb84251a9da17e1e9e8619ff65d8`. Disposable root removed after preserving logs.
Sprint checker passed (115 current/127 archived); git diff --check passed. Test changes: 81 added/1 deleted = **82**; implementation: 4 added/2 deleted = **6**; this receipt: **27** added; total outside tests **33** text lines, plus three binary gzip logs. Only assigned paths changed.
No brief finding was contradicted. No live profile, native credential prompt, real credential read, live posting, push, Rust test or workspace formatter.
This is worker implementation evidence; independent functional/TUI acceptance, human-test readiness and release qualification remain manager-owned and are not claimed here.
