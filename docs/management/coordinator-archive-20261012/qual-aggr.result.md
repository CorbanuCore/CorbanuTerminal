The "Aggressive ships" milestone run is finished, but the milestone did not pass: 17 of 39 cases passed, 15 failed and 7 were blocked. The record is merged into main and is ready for your sign-off decision. I have not marked the milestone passed.

**What shipped**
- PR #447 (docs only) is merged as `e311a44eeb`, with all checks green. The record is in `qa/security-levels/milestones/aggressive-20261010/`. I also refreshed the milestone, PF-83 and sign-off rows in the P1 plan.
- I filed defects #434–#445 and fixed no code.
- The candidate is `origin/main` at `9ead7a5755` (after #413), built as a packaged macOS arm64 0.1.48 (`bin/corbanu` `181ad029…2a4a`). It is ad-hoc signed, not notarized.

**How it was run**
- **Design:** a fresh Opus 5.5 High agent saw only a brief plus 31 real screens from the candidate. Its isolation was instruction-only, and it read only the packet. Its 38 cases, plus one I added (A01, `zai-anthropic` with `glm-5.2`), were frozen and pushed at `d566fb4e06` before any execution.
- **VM:** I reused "macOS 3" and kept "macOS 2" stopped.
  - The guest was erased, provisioned and restored by script.
  - I set aside a leftover `/etc/codex/config.toml` from a September harness and fixed a guest clock that was 15 hours slow; both are restored.
  - Both VMs are stopped now.
- **Isolation:**
  - Each executor ran as a hidden account with no password, admin or SSH access, one lane at a time.
  - Network was limited to a loopback relay that holds the Z.AI key (from the same guest file as increments 33/34). Executors never saw the key.
  - Every run passed an 85-probe isolation check.
  - The native-prompt guard ran 23,429 checks with zero prompts.
- **GLM 5.3 Flash:** it needed only one new path on the relay's allowlist. The guest firewall entry for `api.z.ai` was unchanged.
- **Runs:** 21 groups plus 9 replays, each with a fresh executor; earlier attempts are kept. GLM 5.2 was used only for A01 and for the grant-case replays, after Flash refused three times; both reasons are recorded.
- **Evidence check:** a separate fresh agent each round. Rounds 1–5 failed and I corrected each one; round 6 passed. That used the review budget plus two extensions I authorized under your September 12 delegation (7 of 5); please confirm or overrule them. `check.py` reports BLOCKED, as it must while any case is unresolved.

**Passed (17)**
- Picker, review, confirm, restart and persistence; Moderate shows "not available yet"; pre-Aggressive conversations can't be resumed.
- Typed text never approves an open prompt (#187 fix confirmed), and a declined command shows "Did not run".
- Protected startup and git files can't be written; agents can't use the vault.
- Preflight-blocked and recovery flow; the broker refuses an address it can't handle and says why.
- Both Z.AI routes keep working; the "Enable full access?" cancel message appears; `glm-5.2` on `zai-anthropic` works; the warning about an extra system config file appears; hooks are listed as not sandboxed.

All four "verify on the next candidate" rows in the plan pass on this candidate.

**Failed (15)**
- **Secret leak (#434):** a vault-stored value was shown verbatim and reached the model's context under Aggressive. Caveat: it was stored mid-session and not retested after a restart.
- **Kill switch (#445):** with it on, a normal model turn was still answered, although the screen says the credential broker stays closed.
- **Before restart (#443):** right after confirming Aggressive, `corbanu vault list` still ran and the Corbanu home was listable. One agent command ran without ever appearing on screen.
- **Return to Permissive (#436):** using "restart now" loses the provider key from the environment, and the user lands on API-key onboarding.
- **Protections held but weren't explained (#439, #441):** denials show as raw OS errors with no mention of the security level, and nested `corbanu exec`/`review` are offered for approval, then refuse.
- **Inspector (#437):** a declined approval isn't listed as a recent denial, "every row verified" appears next to unobserved items, and after a session's first turn the rows read looser than the behaviour actually is.
- **Other:** ordinary environment variables are removed too, not just secret-like ones (#440); a false-positive preflight block on `cli_auth_credentials_store` (#438); polish items (#444).

**Blocked (7)**
- **Grant once, revoke all, 10-minute expiry:** never reached. Both GLM models refused to try `cat .env`, echoing the product's own "non-escalatable" wording (#442).
- **Child agent:** behaviour after approving a command wasn't tested, and the spawn failed on replay (#435).
- **Two cases** each had one expected result left untested because the model declined the step.

**Known limits:** this covers macOS only; there is no Windows or Linux code-blind run, and the PF-13-S07 Linux canary rerun is still open. The Isometric Game repo wasn't accessible to this account, so a synthetic project stood in. Paste was simulated by typing, and the relay doesn't log request bodies.

**Decisions I need from you:**
1. **Sign-off:** accept with named limits, or hold until fixes. I recommend holding until at least #434, #445 and #436 are fixed and the failed and blocked cases are re-run.
2. **Before-restart window (#443):** should vault refusal and home denial start immediately on confirm, or only at restart with clearer screen text?
3. **Protected-read grants (#442):** should reads of credential files like `.env` ever be grantable?