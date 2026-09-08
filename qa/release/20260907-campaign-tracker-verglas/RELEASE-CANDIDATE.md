# Campaign Tracker Verglas debug rollout

Explicit user deployment authorization: 2026-09-07. This is an installed debug candidate and production Task Node backend rollout, not a public multi-platform release.

Product: **Campaign Tracker — LOCAL PILOT CANDIDATE**, “preserve attributable user prompts and compact GLM 5.3 Flash summaries in a bounded database”. Active plan: P0 security levels PF-45. Implementation: archived PF-45-S01. This release packages the previously qualified terminal candidate. A bounded backend mapping fix replaces an incomplete status list with Task Node's canonical outstanding-task classification; proposed and submitted work now joins accepted work as candidates. Regression checks cover canonical inclusion and exclusion. Tracker-owned lint findings were corrected without behavior changes.

Corbanu worktree `/home/pfrpc/repos/CorbanuTerminal`, branch `fix/tasknode-profile-isolation`, base `ec549c0c687f50a682487e9d68289c05557ff579`, dirty candidate. Version label 0.1.36. Installed immutable executable `/mnt/HC_Volume_101713660/pfrpc/corbanu-debug/20260907-campaign-tracker/bin/corbanu`, SHA256 `e61eacd61d7e4064bb08b9d05d7b2703af6f1fcf1ce94be0e0d15b2be0c7eb85`. Debug symbols were stripped from a copy of the previously qualified binary; Cargo target was preserved. Installed launcher `/home/pfrpc/.local/bin/corbanu-debug`; previous launcher retained with evidence.

Final backend release v726: image `registry.fly.io/tasknodeofficial-dev:deployment-01M1WV3KP4A61Y7X9Y14ZQX270`, digest `sha256:92280209908779fd23b56dfae36c2186ad2395e201662187d231c9f8d75909db`. Live model-module SHA256 `bd87a9c172b70d7126856fe8a5cc2ac9255650d08bf1a3968362b0d0044dbc8f` matches the tested local file. Overlay derives from exact v723 production digest; adds only tracker files, migration140 and minimal existing route hooks. Deployment-managed encryption key staged once and applied by rollout. Migration/preflight/build/19-machine rollout and all nine background role guards passed. Live status and enrollment succeeded.

Exact visual test workspace is the user-requested `/home/pfrpc/repos/goodalexander/verglas`, base `4346ee6ab18482110401a153b29b64a9bd4470e8`, no Git remote configured. Dirty game work preserved. No third-party enrollment or sharing performed.

Live evidence: [Verglas qualification](../../campaign-tracker/2026-09-07-verglas/README.md). Prior affected Rust tests, database tests, formatting, fixture recovery and real Flash routing: [September 6 qualification](../../campaign-tracker/2026-09-06/README.md).

TensorCash qualification, full competitor benchmarks, production backup restore, account-volume throughput and separate named-human visual acceptance are missing. The explicit deployment instruction applies; these are not claimed as passes. Existing game sliding/stiffness concerns remain separate from functional browser results.

Rollback: restore the saved debug launcher. Backend rollback uses the recorded v723 image; retain the encryption secret and tracker tables so recorded history remains recoverable. No rollback executed.
