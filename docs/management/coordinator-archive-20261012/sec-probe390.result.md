#390 is fixed and merged as PR #399 (merge commit `ad33815766`), with all 31 checks green and 9 skipped. The issue closed when the PR merged, and I added an evidence comment.

**The case in the issue (squat before the broker starts):** if another process creates the broker's pipe names first, using `FILE_FLAG_FIRST_PIPE_INSTANCE` or a limit of one instance, the broker now reports that its names are taken and exits. Core refuses it in about 10 ms and shows: "another process holds or serves the credential broker's named pipe (possible pipe squatting); it was refused and nothing was sent to it". The squatter gets no connection, and Core never falls back to sending requests directly.

**A worse problem I found and fixed:** Windows reports the creator of a pipe's *first* instance as the server of every instance. So a same-user process that added its own instance to a running broker's pipe passed Core's process-id check. It could have received the control message that carries the channel key, or Core's model requests.
- **Control pipe:** it now allows only one instance, so nobody can add one. Foreign clients, including one that connects and closes straight away, are dropped without being read, and Core still connects.
- **Data pipe:** for each connection, Core sends a random 32-byte challenge. It sends its request only after the server answers with a proof keyed to the channel key and both process ids. An added instance receives the challenge and nothing else.
- **After a broker dies or restarts:** a process that grabs the freed names is refused by the process-id check. Core keeps a handle to the broker process, so its id can't be reused.

**Gate:**
- **Real Windows 11 machine, elevated over SSH and in a normal session:**
  - The network-proxy and secret-broker suites pass in both sessions (329/329 elevated), and so do Core's `model_broker_auth::` tests.
  - In the restart race, 10 of 10 requests got through while the squatter received only challenge bytes.
  - Live GLM 5.3 Flash turns answered through the broker in both sessions.
- **`windows-security-probes` CI job:** runs the probes elevated and in the normal session, and checks that they exist.
- **Clippy with `-D warnings`:** clean on Windows and on Linux (RTX box).
- **Review:** Opus 5.5 High, read-only, three rounds. The first two asked for changes, mainly a way one foreign client could kill the broker before Core connected, and a demo squatter that touched other runs' brokers. I fixed everything; the third round approved with small nits, which are also fixed.
- **Demo video:** [mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-390-sec390-win-pipe-squat-refused-04d1324b99e9-2026-10-10.mp4), 42 s, checked for leaked keys. The squatter can't add a control-pipe instance, and its 200 data-pipe instances get 150 connections carrying only challenges, no request or frame. The turn then fails with the squatting error.

**Still open (for #391, `broker_model_auth` on by default for Aggressive):** a same-user process can still block model turns, though it learns nothing. Clients reach the longest-waiting instance first, so enough added data-pipe instances make every turn fail with the squatting error. A process that holds the single control instance before Core connects stops the broker from starting. Both are denial of service, the risk PF-27-S06 already accepted, and they're listed in `qa/security/issue-390/README.md`.

**Cleanup:**
- **Windows:** I removed `C:\CorbanuQA\p390` (including its build target), the Z.AI key file and my scheduled tasks. `C:\CorbanuQA\target` is untouched.
- **RTX box:** removed my worktree, build target and scripts.
- **Mac:** removed my build directory, worktree and branches.
- **Old branch:** the earlier run's work-in-progress branch was never reviewed. The clean work went into a new branch, and I deleted the old one from GitHub without force-pushing anything.

The full report is in `.codex-work/workers-20261002/sec-probe390.result.md`.