<!-- Reviewer: installed `corbanu exec -m claude-opus-5-5-plan -c model_provider="claude-plan" -c model_reasoning_effort="high" -s read-only`, run once, code-blind (told not to read codex-rs/**, top-level scripts/**, PR diffs or .codex-work). The prompt is in the PR description. The output is saved verbatim below; executor dispositions follow. -->

**Verdict: ACCEPT WITH CHANGES.** Every PASS, FAIL and NOT VERIFIABLE verdict holds. Some evidence is weaker than the README claims, a few citations need fixing, and two observations (the `cfgacct` reconnect loop and the ignored `security_level.toml`) were never dispositioned.

I worked read-only and kept to the allowed files. I did not open `codex-rs/**`, top-level `scripts/**`, PR diffs or `.codex-work`.

## Findings

1. **Medium: two accounts often share a key, so the result can't show which one was used.** The exact per-account check (the canary proxy) only covers the custom providers `zpx`/`zcmd`. The built-in `zai` runs only show whether the key was valid:
   - In home hZ, `main` and the env `default` hold the same real key. So the pongs in X1, X4, X7, R2 and R10 can't show that `main` was used rather than `default`.
   - In home hS, `fake` and `default` are both invalid. So the 401s in SP3, SP7, t2, t2b and the S02-14 capture can't tell `fake` from `default`.
   - In hS, `main` and `work` share the real key. So SP2, SP8, SP13 and the TUI D3 "Allow once" child can't tell `work` from the parent's `main`.

   For an in-process spawn with an explicit override to another valid account, the only evidence of which account ran is the product's own rollout field (`provider_account`). The negative cases do show that the override changes the credential, and the out-of-process and TUI-worker paths are proven by the proxy. Qualify S03-1, S03-2b and S03-2c as "rollout-reported, not proxy-proven", or rerun with different real keys for `main` and `work`.

2. **Medium: the README over-states the `cfgacct` result (S03-3a, S03-12).** With the candidate first on PATH, `--account cfgacct` (the Claude Code login-folder kind) shows `Reconnecting... 1/5 … 3/5` and exits 1, after 27 token-command calls. It never shows a 401 or recovery text, but the README says "`cfgacct` rejected". It is only refused in the old-build (version-skew) run. No real Claude Code login-folder account was tested, so that kind has no positive end-to-end proof. Reword this, and either file an issue or explain it.

3. **Medium-low: D3 observation left unresolved.** `s03-spawn-exec-d3.txt` shows that `security_level.toml` set to `aggressive` was ignored: the run recorded `permissive`, and the switch to `work` ran with no approval. Other docs treat that file as enforced state. This could mean D3 does not use "the level in force" when Aggressive was set through `/security`. It needs an owner decision (expected or bug). Two gate items were also never exercised: "unreadable policy counts as Aggressive" and a mid-session raise of the level.

4. **Low: two #426 claims have no capture.**
   - The README says TUI resume `--account ghost` behaves the same. That run (`t4d`) is marked superseded, and `p3-ghost` has no screen and covers the custom provider.
   - "It sends nothing" was not checked on the network: the built-in `zai` ghost run did not go through the proxy. Only "the vault is unchanged" is shown.

5. **Low: plan row "Spawn event shows `work`" is only partly met.** The account appears only in the tool result. In the TUI, the spawn cell shows no account, and the D3 run's footer reads `via zai default … Main [default]` while the session is on `main`. That may be misleading or may be an unrelated label; it should be checked. Record this as partial and as a follow-up, not as a plain PASS for the plan row.

6. **Low: SP11 and SP12 under real Aggressive (`d3-config-level`) don't show the child finishing.** SP11 has no `wait_agent` result, and SP12 shows "Not run … approvals are off" before "child-ok". The claim that these spawns are not prompted still holds, but the "child-ok" replies are not evidence that the child ran.

7. **Low: citations to fix.**
   - S02-14 and the TUI `fake` part of S03-3a cite hS captures, where `default` is also invalid. The proof of "no retry" is the proxy run p2 (exactly one `fake` request); cite it.
   - R7 is labelled "recorded fake", but R2 had already re-recorded the thread as `main`, which is why the error names `zai:main`. #427 still stands.

8. **Low: SP9 claim is inferred.** "The spawn tool offers no `account`" with the flag off comes only from the model leaving the argument out. The proxy's schema check only covered `zpx`, which has no spawn tool.

9. **Low: proxy tally doesn't add up.** The summary counts 23 requests. 8 of them (5 `work`, 3 `default`) don't match any step cited in the captures. Map them, probably to the hW coordinator. There were no unknown or missing bearers, which is the result that matters.

10. **Low: S02-9b has a behaviour change that needs sign-off.** With the flag off, a stray `CORBANU_PROVIDER_ACCOUNT` now gives an error where today's code ignores it. That is not literally "every code path is today's". It fails closed, but product should record that it accepts this.

11. **Low: the record doesn't trace back cleanly.**
   - The candidate SHA `ee530a0e26dac65e0bb9d946e5549c0892806` is 37 hex characters; a full SHA is 40.
   - `REVIEW.md` is linked but does not exist.

12. **Coverage gaps, all disclosed or minor.**
   - S03-1b (`thread/spawnAgent`) is NOT VERIFIABLE. I agree, and it is a sprint Done item.
   - Of the role children the gate names, only explorer and Troll were run; Nazgul and Orc were not.
   - There was no separate test designer and no handoff-checker run. The sprint's code-blind design-and-execution item in the Remaining list must stay unchecked.
   - The sprint says it advances a plan row called "CLI account", but no such row exists in the plan's acceptance table. That is a fault in the sprint record, not in this run.

## Rows I confirmed against the captures

- **Confirmed as stated:** S03-2a, S03-2d, S03-4, S03-5, S03-6 (beating the config is proven; the `main` attribution comes from the proxy runs W5 and p4c), S03-7, S03-10 (through the proxy runs W2 and p2) and S03-11.
- **S03-3b FAIL:** justified by X12, K9, K12 and the ghost capture.
- **S03-1b NOT VERIFIABLE:** agreed.
- **Confirmed with caveats:**
  - S03-1, S03-2b and S03-2c: see findings 1, 5 and 6.
  - S03-3a: see finding 2.
  - S03-8: see finding 8.
  - S03-9: proxy and argv evidence; see the Secrets section.
  - S03-12: see finding 2.
- **S02 re-checks:**
  - S02-3b PASS: a 401 with the candidate on PATH, and an explicit refusal with the old build on PATH, while the real default token was in the env.
  - S02-9b PASS: see finding 10.
  - S02-14 PASS: the message names the account and gives recovery text. "No retry" needs the p2 citation (finding 7).

**Method.** The proxy design works:
- Each account has its own canary.
- Any bearer the proxy doesn't recognise is logged as UNKNOWN or NONE.
- Request bodies are checked for both the canaries and the real key.

It would catch a silent fallback on `zpx`/`zcmd`. The weak spot is the built-in-provider runs in finding 1.

**Issues filed.**
- #425 is justified (X12, K9, K12).
- #426 is justified, without the resume and "sends nothing" claims (finding 4).
- #427 is justified (p4 shows the old default-credential hint; R7 blames `--account`).

The recommendation (limited acceptance behind the flag; fix #425 and #426 before the flag is removed) matches the verdicts. Add findings 2 and 3 as dispositions or issues.

## Secrets

- **Captures and scripts:** a pattern search (long base64/hex strings, `sk-…`, `eyJ…`, `Bearer`) found no real keys. Only request ids, cf-ray values and session ids came up.
- **Real keys** are fetched only by the helper inside the command that uses them. The proxy logs canary ids and 12-hex fingerprints, never values.
- **Synthetic fixtures** appear in plain text in the scripts (`sk-ant-`+`oat01-cfgdir-canary-s03`, `envtok-canary-s03`). They are not real secrets, but they are written out. The random canary and `fake-*` values are never printed.
- **Leak scan gaps:**
  - It never searched for the random `fake-zai-*` and `fake-kimi-*` keys or `envtok-canary-s03`.
  - Its two scan folders are not named.

  So "synthetic values appear only in my fixture files" is only partly supported.
- **Known real-key hit:** the scan found the real OpenAI key in `hO/auth.json`. That is where the pre-PF-84 build stores it, the home was a scratch home, and the README says scratch homes were deleted.
- `env.sh` contains a personal path, `/Users/Neo/.local/bin/corbanu.bak-20261010`. It isn't a secret, but it shouldn't be reused as a policy path.
---

## Executor dispositions (added after the review; README and captures revised accordingly)

1. **Accepted.** I have one Z.AI key, so valid-to-valid attribution for in-process children cannot be proven with
   it. The README Method now says which account a valid in-process child used comes from the rollout's
   `provider_account`. Proxy proof exists for out-of-process workers and TUI workers. I re-ran SP3 and SP7 with a
   valid default and the parent on the real `main`; the `fake` child still gets a 401, so the override really
   switches to `fake` ([capture](captures/s03-spawn-exec-fake-valid-default.txt)). The TUI `fake` and S02-14 rows
   now cite the canary run p2.
2. **Accepted.** I re-ran `cfgacct` to the end. It fails closed after 5 reconnects and 27 token-command calls with
   "Claude Code OAuth refresh token is missing. Run `claude /login` again", because my synthetic folder has no
   refresh token. The README row now says this and that the Claude Code login-folder kind has no positive proof.
   Not filed: it fails closed with recovery text, and the retry count is the #419 polish theme.
3. **Investigated.** The ignored `security_level.toml` was my own hand-written state. Without the preflight
   features, core stays Permissive; `/security` says so. Re-tested in the TUI ([states](captures/s03-tui-d3-security-states.txt)):
   - Aggressive saved through `/security` (preflight passed): D3 asks.
   - Unreadable `security_level.toml`: D3 asks.
   - Missing `security_level.toml` with the Aggressive rules file present: D3 asks.

   In the "Aggressive enforced, boundary unverified" state (preflight features off), the TUI shows Aggressive while
   core records Permissive, and D3 does not ask. Filed as a product decision:
   [#428](https://github.com/CorbanuCore/CorbanuTerminal/issues/428). A mid-session raise of the level was not run.
4. **Accepted.** The ghost capture now includes the TUI resume run (t4d) and the custom-provider run p3 (proxy: 0
   requests). The README says "never reaches the chat; nothing saved" for `zai`.
5. **Accepted.** S03-2b says the account appears only in the tool result (the plan row is partly met). `Main
   [default]` in the footer is the agent pane's label (the main thread's name), not the account.
6. **Accepted.** The SP11/SP12 wording now says the children were created, and their replies do not prove they
   finished.
7. **Accepted.** Citations fixed. R7 now says the recorded account is `zai:main`, re-recorded by R2.
8. **Accepted.** The SP9 claim is now worded as inferred.
9. **Accepted.** All 23 proxy rows are now mapped in the [proxy summary](captures/s03-proxy-summary.txt). Rows
   17–23 are home hW runs, which all used `--account work`.
10. **Accepted.** Added to the S02 recommendation for Travis.
11. **Accepted.** I had mistyped the SHA; it is now the full `ee530a0e26e21dac65e0bb9d946e5549c0892806`. This
    REVIEW.md is that link.
12. **Agreed.** Nazgul/Orc role children are noted as not run, and the sprint's code-blind item stays unchecked.

Secrets: the leak scan now names its roots and also searches for the synthetic env token. It finds that token only
in my script. The random `fake-*` keys were piped straight into `account add` and never written anywhere else, so
they cannot be searched for.
