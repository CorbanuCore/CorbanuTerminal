# Coordinator answers to the designer's questions (third run, 2026-10-13)

Answers give test-machine facts or say that the owner has not decided; they do not say how the product behaves.

1. Remembered approvals: from confirmation or after the restart? The owner's decision does not say. With pending
   answer (a) at its recommendation, approval behaviour belongs to the restart-only list. Test after the restart (S1),
   as you assumed.
2. Rules from config or rules files: the decision speaks of remembered prefix approvals ("don't ask again"). Test only
   those, as you assumed.
3. Grant once offered for an ordinary write outside the folder: not decided by the owner. Recording it as an advisory
   finding is fine; any write that lands outside the folder is a blocker, as you assumed.
4. Hard link to a file in .codex/.agents: not decided by the owner beyond "a link that points into .codex/.agents".
   Your assumption stands (file unchanged is the blocker; whether the link is created is recorded).
5. "OpenAI-style is the default": the test harness always names a connection when it starts the product, so a launch
   with no connection named is not available in the test machine. Observable: the order and marking in `/model`, and
   the choice a new pane offers.
6. A connection picked in `/model` surviving a restart: not decided by the owner; record only.
7. Home folder as the current folder: not available in the test machine. The harness starts the product only in the
   test project (`$AGMS_RUN/fixture/workspace`), and the account's home also holds the run's own state and the
   product's home, so it cannot serve as a clean "current folder holds the home folder" test.
8. Setup scope (your AGG-17 note): each case group starts from a fresh copy of the test project and a fresh product
   profile, and the `.husky` setup is done only in the groups that run the grant cases (AGG-17/18/22/23/24/33, R17), so
   AGG-06 and AGG-11 run without `core.hooksPath` set.
9. Relay log: the executor cannot read it; the independent evidence check reads it afterwards (as in AGG-R01). It
   records each request's time, path (`/chat/completions` for the OpenAI-style connection, `/messages` for the
   Claude-style one) and size.
