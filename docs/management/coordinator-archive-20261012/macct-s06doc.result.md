**PR #424** is merged as `3b211523163c1ceacb8388e5f85585f9659097bd`. I merged it only after every check finished green; the last one was the Ubuntu tmux smoke at about 45 minutes. It was a merge commit with the head pinned to `f1680e617f` and no conflicts with main.

**Checker output** (run on the final tree):
```
$ python3 docs/plans/check.py && python3 docs/sprints/check.py
plans: active 3/3; available slots 0
sprints: current 105; archived 155
```

**What the PR changes:**
- **Decision recorded:** Travis's 2026-10-11 choice of option (a), from his chat with the coordinator, is now decision **D4** in the plan's PF-84 section and its decisions table. It replaces the old wording that sent named OpenAI/AWS accounts to S04, and notes that options (b) and (c) were not chosen.
- **New draft sprint:** `pf-84-s06-aws-and-chatgpt-accounts.md`, unallocated, depending on PF-84-S05, and 96 lines long (the checker's limit is 100).
  - **Slice A (AWS profile accounts):** the account stores only the profile name, which isn't secret, and Bedrock uses it as `aws.profile`. A named account ignores the stored Bedrock key and the AWS environment variables. A missing profile or failed credentials are refused before any request, with no fallback.
  - **Live AWS check:** marked **NOT VERIFIABLE** unless Travis provides a test AWS account with Bedrock access, a minimal-permission profile and a spend cap.
  - **Slice B (ChatGPT/OpenAI sign-in accounts):** each account has its own sign-in store in the vault, and refreshed tokens are saved back to that account only. Login and logout can target one account. Today's `Codex Auth` item or `auth.json` stays in place as the `default` account. No new keychain items are created.
  - **Gates:** the same as the other PF-84 sprints: focused tests, Linux clippy on glitch, tmux, GLM 5.3 Flash runs, two videos, and one Opus 5.5 High review per slice.
- **Tables and navigation:** S06 is added to the plan's sprint table, dependency graph (S05 → S06) and acceptance flows, and to both sprint indexes.

**Open points for Travis** (listed in the S06 record):
1. Keep the existing ChatGPT login where it is, or move it into the vault? I recommend keeping it.
2. If the same ChatGPT user is signed in under two accounts, warn or refuse? I recommend warning.
3. Provide a test AWS profile, or accept that slice A merges without a live AWS test.
4. Allow AWS SSO profiles, stored as profile names only? I recommend yes.

**For macct-lane3:** the S04 record still has the "Pending Travis: named ChatGPT sign-ins and AWS…" item under `Remaining` (lines 73–74). I left it because the brief limited me to S06's record. Now that D4 settles it, that item should be removed in S04's next update.

My worktree is removed. Git refused to delete the local branch `docs/pf-84-s06-draft` with `-d`, and I didn't force it, so it's still there.