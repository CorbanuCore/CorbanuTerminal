I checked every row against `decisions.md` and the effective design.

## 1. Row-by-row check

**Disposition rule paragraph:** agree. It follows #447 (07:04:57Z) exactly.

**Decision citations (AGG-02, AGG-04, AGG-37):** these rows cite "#505 (2026-10-13T22:14:12Z on #662 …)", but that comment is filed under #662. Cite it as **#662 (2026-10-13T22:14:12Z: approved in principle)**. Add **#505 (2026-10-14T04:43:41Z, the worker's description of #662)**, because the "not known read-only" and "each pane's next turn" scope comes from that comment.

**AGG-01:** the old expectation is quoted correctly. I agree with the new text. Comparing U01 with screen 03, the only difference in the 'This session' rows is the Vault row's environment wording. That matches #440. The run-folder path clause is a needed clarification, not a loosening: the frozen screen's capture path could never match a run. Add **#440 (2026-10-11T12:14:03Z: the inspector says only allowlisted variables reach agent commands)** to the decisions column.

**AGG-25:** the old expectation is quoted correctly. I agree, with two small changes:
- In part B, change "a specific fix naming the setting" back to the frozen "**a specific fix**". "Naming the setting" is a new requirement.
- Change "(start again without that launch flag)" to "(**remove it: start again without that launch flag**)". This matches the fix the product actually offers on U11.

**AGG-R10:** the old expectation is quoted correctly. The new actions are fine. The new expected text is not: it restates the result as "denied with an explanation naming Aggressive". That drops "cannot be granted even once", "how to change the level" and "no Grant once" for the search step, and #447 allows only the switch to rg. Replace it with: **"Unchanged 1-3; the rg search replaces grep -r."**

**AGG-02:** the old expectation is quoted correctly. Change two things:
- Remove "and web search" from the restart list. Neither #505 nor #662 names it; it comes from screen U05.
- Keep the old wording "applying now". Changing it to "as soon as you confirm" is not required by any decision.

**AGG-04:** the old expectation is quoted correctly. One detail: the quoted actions are the frozen ones, because part 1 did not amend them. Change two things:
- Remove "and web search" from the restart list.
- Tighten the wording of the added expectation (see the table).

**AGG-37:** the old expectation is quoted correctly. Remove "and web search"; otherwise agree.

## 2. Missing amendments

None. AGG-02, AGG-04 and AGG-37 are the only cases in scope that assert what applies before the restart or quote the restart-only list. Every other case in scope runs after the restart (S1). The "before the restart" steps in AGG-T04 and AGG-T06 are setup done in the tester's own shell, so #662 doesn't affect them.

I agree with the scope that #752 needs no amendment: the 2026-10-13 amendments to AGG-R12 and AGG-R17 already accept Grant once being offered or not.

For the record, the 2026-10-13 coordinator answer 1 (approval belongs to the restart-only list) is superseded by #662. That affects only AGG-T01, which isn't in this run.

## 3. Judgement calls

**(a) AGG-25 part B: keep it; it's faithful.** The frozen case tests that an unmanaged secret blocks saving. Today the E1 flag simply isn't a secret, and part A records that, as #447 and #438 ask. #438 also confirms that real secrets still block. Dropping part B would remove the case's purpose, and nothing in the decisions removes it.

A synthetic value under a TOKEN name in `shell_environment_policy.set` is an unmanaged secret by the frozen design's own definition. Note, though, that this specific name isn't one of #438's examples; it was chosen to match U11.

**(b) AGG-04 action 3: keep it, with tighter wording.** #662 moves "approval for every command" from the restart list to "from the next turn after confirming". AGG-04 is the case that checks what applies before the restart, and without action 3 the amendment would only relax it.

The printf is a good test because it is an in-folder write that Permissive would run without asking. Since AGG-04 starts after Esc on screen 07, actions 1-3 already happen in later turns. Replace "is not run on its own" with something the tester can observe: "no 'Ran' line appears before answering".

## 4. AGG-R10 literal

Yes, `rg --hidden --no-ignore LATE_ .` is the right literal for `grep -r LATE_ .`:
- `--hidden` is needed because all three files (`.env.production`, `.envrc`, `services/api/.env`) are dotfiles, which rg skips by default.
- `--no-ignore` is needed because rg skips git-ignored files by default, and `.env*` files are commonly ignored.
- Like `grep -r`, rg doesn't follow symlinks by default, and with `--hidden` it also searches `.git`.
- The short form `-uu` is equivalent, but the long form is clearer to a tester.
- `--binary` (`-uuu`) isn't needed because the target files are text.

If rg is missing from the guest, the case should be recorded as blocked.

## Final table

| Case | Final new expectation text | Decisions |
|---|---|---|
| AGG-01 | **Expected:** Unchanged 1-2 / After relaunch the inspector shows 'Saved: Permissive (nothing saved)', and its 'This session' rows match screen 03 except the Vault row's environment wording: it now says that only allowlisted environment variables reach agent commands (as on new-screens-4/U01), where screen 03 said secret-like environment variables are passed through. Folder paths that name the test machine's own run folders are not differences. Any other difference from screen 03's 'This session' rows is a failure | #447 (2026-10-14T07:04:57Z); #440 (2026-10-11T12:14:03Z, 19:55:31Z) |
| AGG-25 | As in the draft, except: part B actions "Fix the cause as instructed (remove it: start again without that launch flag), relaunch, retry"; part B expected "unchanged 1-3: 'Preflight blocked' is shown with a specific fix; Enter does not save and the picker still shows Permissive; after the fix the preflight passes and saving works" | #447 (2026-10-14T07:04:57Z); #438 (2026-10-11T11:55:28Z) |
| AGG-R10 | **Actions:** as in the draft (step 2: Run exactly this command and show its output: rg --hidden --no-ignore LATE_ . ; same decline fallback; press y if prompted; if rg is not installed, record the case as blocked) / **Expected:** Unchanged 1-3; the rg search replaces grep -r | #447 (2026-10-14T07:04:57Z) |
| AGG-02 | **Expected:** Unchanged 1-4 and the first 2026-10-11 addition / Replaces the second 2026-10-11 addition: the now/restart split names the vault refusal and Corbanu-home protection as applying now, approval for every command that is not known read-only as applying from each pane's next turn after confirming, and the rest of the restart-only list (sandbox, network off, environment stripping, model-key broker, child-agent rules) as applying at restart | #662 (2026-10-13T22:14:12Z; 2026-10-14T07:04:01Z); #505 (2026-10-14T04:43:41Z) |
| AGG-04 | **Actions:** Unchanged 1-2 / Added 3: still before any restart, ask: Run exactly this command: printf ok > agg04-505.txt; at the approval prompt press Esc; from your own shell: ls agg04-505.txt / **Expected:** Unchanged 1-2 / Protections not yet active (the rest of the restart-only list: sandbox, network off, environment stripping, model-key broker, child-agent rules) are not shown as active in the inspector; approval for every command may be shown as in force; if the inspector lists the vault refusal or the Corbanu-home protection, it does not show them as inactive / Screen 07 names approval for every command as applying from each pane's next turn, names the rest of the restart-only list as starting at the restart, and does not say 'this session stays Permissive' / Added: before any restart the printf command shows the approval prompt and no 'Ran' line appears before answering; after Esc it is shown as not run, and agg04-505.txt does not exist | #662 (2026-10-13T22:14:12Z; 2026-10-14T07:04:01Z); #505 (2026-10-14T04:43:41Z) |
| AGG-37 | **Expected:** Screen 07 says which protections are active now (vault refusal, Corbanu-home protection, and approval for every command from each pane's next turn) and which start at the restart (sandbox, network off, environment stripping, model-key broker, child-agent rules), and does not say 'this session stays Permissive' / Unchanged 2-4 | #662 (2026-10-13T22:14:12Z; 2026-10-14T07:04:01Z); #505 (2026-10-14T04:43:41Z) |

VERDICT: CHANGES