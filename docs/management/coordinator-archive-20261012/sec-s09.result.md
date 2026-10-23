PF-27-S09 is finished. Both slices are merged with every CI check green, and the sprint is marked `ready` for your acceptance (not archived). On Windows, with `broker_model_auth` on, Core's model requests now go through the broker over its named pipes, with the server process-id check, and Core holds only references to the keys. The flag is still off by default; Permissive, macOS and Linux are unchanged.

## What shipped
- **#363, the broker side:**
  - **Env keys:** when Core hands a provider key to the broker, the key is overwritten in Core's environment and in the C runtime's copies. It is also overwritten in the old launch environment block, which Windows never frees after moving the environment.
  - **Stored keys:** the broker reads them itself, taking the vault key from Credential Manager under its own token (your option (c)). That token can't open the vault's lock file for writing, so Core creates the file first and the broker locks it read-only.
  - **TLS fix:** the broker trusted no TLS root at all, so every brokered HTTPS request failed. Its token can't open the Windows root certificate store for writing; it now opens the store read-only.
- **#379, Core:** on Windows, Core now starts the broker, hands it the env keys and sends every model request to it through the checked pipe. With no broker running, requests are refused, never sent direct.

## Gate
- **Real Windows machine, GLM 5.2, normal (medium) session and over SSH:**
  - Answers came through the broker with both an env key and a vault key.
  - In the console session the vault key was in Credential Manager and the broker read it.
  - Over SSH, the elevated sandbox ran `whoami`.
- **Tests:** focused tests pass in both sessions. The broker tests also passed 5 out of 5 times in parallel, the way the credential canary runs them.
- **Clippy `-D warnings`:** clean on Windows and Linux (RTX box).
- **CI found two key copies the real machine never showed.** The credential canary found one; the normal-session security probes found the other, in the old environment block. Both are fixed and pinned by tests.
- **Reviews (Opus 5.5 High):** slice 1 was approved, plus three follow-up reviews of later fixes, all approved. Slice 2 was approved.
- **Videos:** three short demos, checked for leaked keys, uploaded to `demos` and listed in `qa/demos/index/PF-27-S09.md`: env key in a normal session, vault key in a normal session, and over SSH with the elevated sandbox.

## Known limits for you
- **Memory scan scope:** the scan covers the key hand-over, not a full Core start-up. If other Core code keeps its own copy of the key, the copy is overwritten on purpose.
- **Key overwrite is best effort:** copies outside the heaps and the launch environment block, such as on a thread stack, aren't found. It adds about 140 ms per key at session start in a debug build.
- **No Credential Manager over SSH:** in SSH sessions the vault keeps its key in a file in the profile, which was already the case before this sprint. The broker reads that file the same way.
- **Your other saved credentials:** the broker's token can read and write your other saved Windows credentials. This is the accepted consequence of option (c).
- **Sign-in refresh:** replacing the broker's copy when a ChatGPT sign-in refreshes is covered by unit tests only.
- **Missing headers:** brokered requests on Windows don't carry the usual tracing headers or the request debug log.
- **Sandbox in a normal session:** the elevated sandbox needs a one-time admin setup, so the normal-session demos used the non-admin sandbox.

## Cleanup
- **Windows:** removed `C:\CorbanuQA\s09`, the Z.AI key file, the Credential Manager entry from my runs, and all my scheduled tasks. I created no local users, and `C:\CorbanuQA\target` is untouched. My elevated runs over SSH did the sandbox setup, which may have reset the machine-wide sandbox passwords.
- **RTX box:** removed `~/corbanu-rtx/s09*`.
- **Mac:** removed my worktrees, local branches and build directory. I kept the review outputs and the 14 MB of run artifacts in `workers-20261002/win-gate-s09/`.

The full report is in `.codex-work/workers-20261002/sec-s09.result.md`.