You are the broker lane worker (round 6, continuation). The previous worker was stopped by the coordinator because
its live/demo runs triggered repeated macOS login-keychain password popups for Travis. Read
/Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md (binding; note the new keychain rule), its brief
sec-broker6.md, and skim the tail of sec-broker6.log (very long; use rg/tail) to recover state. Its worktree
/Volumes/CorbanuDrive/Corbanu/worktrees/sec-broker6-20261006 has uncommitted edits in progress: keep them.

First, before any further live run:
1. Find why a disposable-profile run reached the real login keychain: the vault (and any other OS-keyring user: login
   token storage, broker key storage) must honour CORBANU_TEST_NO_NATIVE_KEYRING (today only claude_oauth checks it),
   falling back to a file-backed store inside the disposable home. Fix it in its own small PR with tests, and make
   scripts/demo_video.py and qa helpers (e.g. qa/security-levels/sprints/PF-27-S05/vault-seed-launch.sh) refuse to run
   unless the variable is set. Debug/test builds only: release behaviour must not change.
2. Record in the PR which keychain items the earlier runs may have created (service/account names), so Travis can
   delete them in Keychain Access; don't delete or read keychain items yourself.
Then continue finishing PF-27-S05 per sec-broker6.md. Report as in sec-common.md.
