# Owner decisions (Travis, 2026-10-13), verbatim from the issues' decision comments

Extracted by the integrator for the third run; only text that states an owner decision. Implementation and evidence text is left out. The decisions of 2026-10-11 (second run) still apply: `owner-decisions-20261011.md`.

## #447 (2026-10-13T08:48:32Z): qa: milestone "Aggressive ships" code-blind VM run record (not passed; sign-off requested)
https://github.com/CorbanuCore/CorbanuTerminal/pull/447#issuecomment-6141853017

Owner decision (Travis, 2026-10-13) on the Aggressive milestone after re-run #628 (41/10/6 of 57): not signed off; fix the defects, then a third isolated code-blind run.

## #620 (2026-10-13T08:48:30Z): Aggressive (AGG-18, AGG-R17): "Grant once" for a ~/.zshrc append is accepted but the write still fails with "operation not permitted"
https://github.com/CorbanuCore/CorbanuTerminal/issues/620#issuecomment-6141852342

Owner decision (Travis, 2026-10-13): keep option B as merged in #637 (Grant once is not offered for a start-up-file write outside the current folder; the denial explains why). The next milestone run exercises grant/revoke/expiry on an in-folder file instead of ~/.zshrc.

## #626 (2026-10-13T08:48:25Z): Aggressive (AGG-33 replay): a "don't ask again" prefix rule approved under Permissive still auto-approves after switching to Aggressive
https://github.com/CorbanuCore/CorbanuTerminal/issues/626#issuecomment-6141850904

Owner decision (Travis, 2026-10-13): suspend remembered prefix approvals from a less strict level while Aggressive is active (they must not auto-approve under Aggressive). Implementing.

## #623 (2026-10-13T08:48:27Z): Aggressive (AGG-R13): agent can create a symlink to .agents inside the project (write through it is refused)
https://github.com/CorbanuCore/CorbanuTerminal/issues/623#issuecomment-6141851400

Owner decision (Travis, 2026-10-13): under Aggressive, refuse creating a link that points into .codex/.agents (nothing is created), consistent with '.codex/.agents always read-only'. Implementing.

## #620 (2026-10-13T07:16:06Z, only the options paragraph the decision above refers to)
https://github.com/CorbanuCore/CorbanuTerminal/issues/620#issuecomment-6140376906

**Decision for Travis:** Option A, letting Grant once open that exact file outside the folder for one run, would move Aggressive's sandbox boundary and needs security sign-off. Option B is merged: Grant once never writes outside the current folder. Recommendation: B. The milestone's grant, revoke and expiry cases (AGG-17/18/23/33) should then use a start-up-file or git-config write inside the current folder.

## #653 (issue body, owner decision 2026-10-13): /model: choose the connection (route) when a model is served by more than one provider route
https://github.com/CorbanuCore/CorbanuTerminal/issues/653

Owner decision (Travis, 2026-10-13), #625 follow-up: option C.
- Now (B): AGG-27 in the next Aggressive milestone run switches route by relaunching on each route; document that /model lists models, not routes.
- Then (A): /model offers the connection when a model is available on several routes (e.g. GLM 5.3 Flash on zai and zai-anthropic); picking one switches the session.
- Default rule (binding for this and anything similar): when a model is available on more than one connection, the OpenAI-style connection is the default (we are a Codex fork), e.g. zai rather than zai-anthropic.

## Coordinator brief for the third run (2026-10-13), restating the decisions

> Owner decisions since the second run (Travis, 2026-10-13; read the decision comments on #447, #620, #626, #623, #653): Grant once is NOT offered for a start-up-file write outside the current folder, and the denial explains why (#620 option B) — exercise grant/revoke/expiry cases (AGG-17/18/23/33) on an in-folder file (e.g. a git config or start-up file inside the project) instead of ~/.zshrc; remembered prefix approvals from a less strict level are suspended under Aggressive (#626); creating a link into .codex/.agents is refused (#623); AGG-27 can run as designed because /model now offers both Z.AI connections (#653/#664; the OpenAI-style connection is the default).

## Pending at drafting time (answers arrive in the go file; filled in before freezing)

- (a) #505/#662 timing, as put to the owner: "approvals at confirmation before the third run? Option A: finish #662 and
  amend the milestone screen (approval for every command would then apply from confirmation instead of at the restart).
  Option B: keep the restart-only list for the third run." Recommendation: B (frozen design unchanged).
- (b) AGG-32, as put to the owner: "should a readable extra config file still mark the setup 'not clean' at startup? The
  startup warning no longer appears. Option A: restore the warning. Option B: amend AGG-32 to match today's behaviour."
  Recommendation: B (no startup "not clean" warning for a readable extra config layer).

## Answers to the pending questions (2026-10-13T17:10Z), verbatim

### #505 (2026-10-13T17:10:33Z)
https://github.com/CorbanuCore/CorbanuTerminal/issues/505#issuecomment-6150994002

Owner decision (Travis, 2026-10-13): option B — the third Aggressive milestone run keeps the restart-only list as designed; #662 (approvals at confirmation) is finished after that run.

### #447 (2026-10-13T17:10:35Z)
https://github.com/CorbanuCore/CorbanuTerminal/pull/447#issuecomment-6150994938

Owner decision (Travis, 2026-10-13) for the third run: (1) #505 restart-only list stays as designed; (2) AGG-32 is amended to today's behaviour (no startup 'not clean' warning for a readable extra config layer).
