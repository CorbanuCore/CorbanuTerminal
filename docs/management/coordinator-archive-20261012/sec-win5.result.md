I merged #307. The #301/#304 fix is up as draft PR #326 and isn't merged: fixing #304 the obvious way opens a new way to strip a secret's deny entry, and closing it needs a design decision from you. #300 is untouched.

**#307 (security): merged as PR #321 (`9810e07e`), issue closed with evidence**
- **Fix:** Core never marks the launcher's pipe ends inheritable in its own handle table. It copies them into a holder process that never runs and starts the launcher as that holder's child, so nothing else Core starts can pick them up.
- **Regression test** (300 launcher starts raced against ordinary child processes, counting children that held a launcher pipe):

| | Elevated | Normal session (medium) |
|---|---|---|
| Real Windows 11, before | 55 of 687 (fail) | 60 of 685 (fail) |
| Real Windows 11, after | 0 of 880 | 0 of 879 |
| CI (`windows-security-probes`), after | 0 of 1683 | 0 of 1635 |

- The #295 probe still starts processes through the launcher in a normal session.
- **Gate:** Windows and Linux clippy clean; CI all green; Opus review APPROVE (its low findings are fixed).
- **New issue #320:** `spawn_protected` (the broker's start) has the same race.

**#304 + #301 (one PR, they touch the same code): draft PR #326, not merged**
- **What's in it:**
  - Removal deletes exactly the deny entry the sync added. Windows splits that entry in two on directories; both parts go.
  - The sync only records entries it added itself, tied to the specific file or folder, and won't follow links or hard links when removing.
  - Its record moved to `.sandbox-secrets`, which sandboxed commands can't write; the old file in `.sandbox` was writable by them.
  - #301: while any session holds the armed lock, nothing is removed.
- **Tests on the real machine, elevated and normal session:** 10 tests. 7 fail on main; all pass with the fix (two junction steps skip in the normal session because this machine won't follow a junction a non-admin created). With only the #304 half applied, the two-session #301 test fails with "removed under an armed session". The core test using a real armed contract fails on main and passes after. Windows and Linux clippy clean; CI runs everything elevated and in a normal session.
- **Review:** four Opus rounds, all REQUEST_CHANGES. Every finding up to round 4 is fixed except one (S1). Because of that, #301 and #304 are still open.

**Decision needed: should removal be driven by the configured rules?**
- **S1, the problem:** a leftover sandboxed process can hold a folder open so the glob scan silently misses a secret inside it. The next sync then treats that secret as no longer listed and removes its deny entry. On main this was harmless only because removal never worked. The sync gets only expanded paths, so it can't tell "you deleted the rule" from "the scan didn't see it".
- **Options:**
  1. **Remove by rule (recommended):** pass the configured deny-read rules alongside their expanded paths, and remove an entry only when its rule is gone from config. That touches the sandboxing crate, core `exec.rs`, the windows-sandbox backends and the elevated setup payload. Roughly a day plus review.
  2. **Exact paths only:** the same plumbing with a "came from a glob" flag; glob-matched entries stay permanent as today.
  3. **Keep denies permanent (main's behaviour):** merge only the safe parts (exact matching, ownership record, #301 lock) with removal switched off. #304 then stays open as a known limit.

Separately, filed as #323: two sessions using different `CODEX_HOME`s, or two flag-off sessions on the same one with different deny lists, can still remove each other's entries.

**Notes**
- Five `windows-sandbox` tests fail on the QA machine on main too (`CreateProcessWithLogonW failed: 2`); it's that machine, not CI.
- One existing core symlink test needs a privilege a normal session doesn't have.

**Cleanup:** the temporary user `w5logon` is removed, no scheduled tasks were created, and no test folders are left in Temp. I deleted `C:\CorbanuQA\win5\target`; `C:\CorbanuQA\target` is untouched. The #307 worktree and its Mac build folder are gone. I kept the #326 worktree, the RTX clone `~/corbanu-rtx/sec-win5` with its build folder, and `C:\CorbanuQA\win5` (source, logs, scripts) for the next round. Review outputs are in `.codex-work/workers-20261002/sec-win5-review-307/` and `sec-win5-review-30x/`.