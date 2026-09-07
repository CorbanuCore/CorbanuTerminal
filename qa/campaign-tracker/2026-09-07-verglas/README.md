# Verglas live rollout — 2026-09-07

Release work authorized by the user's request to deploy Campaign Tracker onto the Verglas isometric game and verify it at length.

Product reference: **Campaign Tracker — LOCAL PILOT CANDIDATE**, “preserve attributable user prompts and compact GLM 5.3 Flash summaries in a bounded database”. PF-45 implementation is completed in archived PF-45-S01; this record packages and qualifies that candidate.

Exact workspace: `/home/pfrpc/repos/goodalexander/verglas`, base `4346ee6ab18482110401a153b29b64a9bd4470e8`, existing dirty movement work preserved. No Git remote is configured. Live player: http://178.156.143.199:8444/runtime/player.html.

Backend baseline: Fly tasknodeofficial-dev v723, image digest `sha256:2e09c4862cf49bc610045a75e514cb88eb04f436db645a0a8dadf2983727fefd`. Authenticated tracker status returned 404. Encryption key absent. Rollout overlays only six tracker modules, migration140, and minimal live route integration. Existing production source is the integration baseline; unrelated local dirty changes are excluded. Production deployment preflight, migration registration and background guard remain in the deployment wrapper.

Terminal candidate: previous qualification SHA256 `8e6b66e67c1156f6faa2f0d1c366c03bfb6ff601bf9d709b7169ded8b67cb333`. Prior local test evidence is in ../2026-09-06/README.md. Dedicated evidence stays on the mounted data volume at `/mnt/HC_Volume_101713660/pfrpc/scratch/campaign-tracker-verglas-20260907`.

Status: deployed and live-qualified. No third-party enrollment or sharing was performed. TensorCash qualification, competitor benchmarks, long-term backup restore and human pilot acceptance remain unqualified; explicit user deployment authority applies.

## Final deployed identities

- Task Node release v726, image `deployment-01M1WV3KP4A61Y7X9Y14ZQX270`, digest `sha256:92280209908779fd23b56dfae36c2186ad2395e201662187d231c9f8d75909db`.
- Installed stripped debug copy: SHA256 `e61eacd61d7e4064bb08b9d05d7b2703af6f1fcf1ce94be0e0d15b2be0c7eb85`, 529 MiB, on the data volume. `corbanu-debug` now uses it with the user's normal Corbanu home. User model preferences were not changed; QA used per-command Flash overrides.
- Final live model module matches local SHA256 `bd87a9c172b70d7126856fe8a5cc2ac9255650d08bf1a3968362b0d0044dbc8f`.

## Real qualification

Actual PTY keys ran `corbanu-debug --yolo` in the live Verglas workspace with `RUST_LOG=trace`, a dedicated data-volume log directory, and the real Corbanu gateway model `corbanu/glm-5.3-flash`. Recording was enabled at 02:17:58 UTC; the last model turn completed at 02:31:50, with final replay/UI checks afterward. This was about 15 minutes of live qualification, not an overnight or account-volume soak.

- 78/78 browser assertions passed: one baseline 11-check acceptance round, the 12-check product suite, and five further 11-check acceptance rounds. The repeated sequence took 186 seconds and included five real-time 30-second gameplay soaks. The Node locomotion unit suite also passed. Screenshots visually loaded the expected Verglas scene; no claim is made that prior sliding/stiffness concerns have been resolved.
- Real model turns inspected the repository, ran product/locomotion tests, ran the repeated browser battery, exercised an intentional failure, and verified the live game after a cold restart. No source edits were requested or observed; Git status before/after matched.
- Final ledger: 26 events, five human prompts, ten tool actions, five Flash output summaries, five completed root turns and one goal-cleared status event on resume. Agent runtime 223,951 ms; this is not employee working time. Zero pending summaries/uploads, zero capture gaps, one correctly recorded exit-code-7 failure, zero verified task completions.
- All 26 events replayed across pagination. Original prompts were preserved; assistant output content is empty in durable records and replaced by the pinned Flash summary. Attribution, timestamps, workspace, repository metadata and goal flags passed assertions. Commands executed in the artifact directory correctly record that working directory separately from the enrolled Verglas workspace.
- Anonymous access returned 401. History access for an ungranted account returned 403. No real collaborator's permissions were modified.
- Paused recording through actual `/tasknode` controls, ran a real model response, and proved the paused prompt was absent with the ledger unchanged at 21 events. Re-enabled through the menu, exited, resumed the same conversation in a fresh process, retained all original IDs and captured a new real turn successfully.
- Live review found the task candidate query omitted proposed/submitted states. The bounded fix uses Task Node's canonical outstanding classification. Regression checks cover inclusion and exclusion; the final real summary saw the outstanding candidates and correctly declined to map unrelated Verglas work.
- Final Task Node lint, tracker contract smoke, 38 PostgreSQL checks, migration discovery, production preflight and all nine background-role guards passed. Final local/live model-module hashes match. The terminal code was unchanged from the previously formatted and tested binary; its installed stripped copy was the executable used for all real PTY work.

## Evidence and storage

`operator.mjs verify` checks production replay, attribution, preserved prompts, compact pinned summaries and access denial. `verification.json`, `activity.json`, `metrics.json`, `status.json`, eleven PTY captures, individual browser results/screenshots, and the final deployment log are in the dedicated data-volume directory. Source summaries and database exports are operator evidence, not public game assets.

Root free space remained approximately 375 GiB. The encrypted local tracker store was 28 KiB during the run. Final database payload was 45,242 bytes at 26 events; seven tracker tables including indexes occupied approximately 336 KiB at the earlier size probe. The QA terminal was stopped and traces compressed. All retained QA artifacts total 7.9 MiB and have private filesystem permissions; no verbose QA process remains running. The user's live game server remains on port8444, and workspace recording remains enabled for future updated debug-terminal sessions.

See the [release record](../../release/20260907-campaign-tracker-verglas/RELEASE-CANDIDATE.md) for exact packaging and rollback information.
