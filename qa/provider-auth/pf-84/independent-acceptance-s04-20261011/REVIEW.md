# Independent review of this acceptance report

Reviewer: one code-blind `corbanu exec` session (installed binary, `-m claude-opus-5-5-plan -c model_provider="claude-plan" -c model_reasoning_effort="high" -s read-only`), run on the first version of README.md. Its prompt limited it to this directory, the S04 sprint record, the plan, the S04 gate and demo index, and the user docs. Its output follows unedited; the author's response is at the end.

## Review: PF-84-S04 independent acceptance report (2026-10-11)

## Verdict: **Changes requested** (only the report needs to change)

The product behaviour mostly holds up. The quoted text for S04-1, 3, 5, 6, 7, 9, 11 and 13 exists in the cited captures. The proxy counts match `proxy-requests.jsonl` exactly (pdefault ×2, pmain ×3, pwork ×4, pfake ×1). The candidate `c7e6e07cbc` contains the gate's code commit `7a1ab9bc9d`. Several verdicts are still worded more strongly than the saved artifacts support.

## Findings

1. **Blocking. Some PASS claims have no saved artifact.** The homes were deleted, so these rest only on the tester's word:
   - **S04-10:** "config.toml and local.age hashes identical before and after (z2)". No hash output is saved; `transcripts/z2.txt` has none.
   - **S04-14:** `captures/migration.txt` saves only the starting hashes (`hM-hash-0`). The "M1/M2 hashes unchanged" results are prose.
   - **S04-3 / S04-4:** The Z.AI `config.toml` changes (`zai = "spare"`, then `[provider_accounts]` cleared) are not captured. Only the `zpx` home's config is (`providers-proxy-canary.txt`).
   - **S04-2:** The rule text for an invalid name, "an empty value saves nothing" and Esc at the key step are not captured. `transcripts/z1.txt` has no onboarding add screens.
   - **S04-9:** "With the flag off there is no account suffix" on the spawn line is not captured. `flag-off.txt` has no spawn.
   - **Method:** "A `security(1)` PATH shim recorded 0 calls" has no log. A PATH shim also cannot see direct keychain framework calls.

   Fix: mark these claims as tester-attested with no artifact, or re-run and attach the raw output. Policy says missing evidence must not be shown as a pass.

2. **Blocking. S04-4 should be PASS (partial), or the check should be run.** The sprint's security note requires that "the vault index proves it". The report says no listing of vault labels exists. But `docs/authentication.md` (Vault Storage section) says labels, types, providers and timestamps can be listed without showing secrets. That metadata listing should have been used to show only that account's labels were removed.

3. **Blocking for wording. #461 is understated, and S04-12 is overclaimed for onboarding.** When the TUI quits, keys still arriving land on the terminal in clear text. `onboarding-q-quits.txt:213` shows a synthetic value echoed right after `TUI_EXITED`. So if a user types a key containing `q`, or pastes one without bracketed paste, the rest of the secret shows in clear and goes to their shell. That breaks masked entry, not just usability.
   - Reclassify #461 as security-relevant.
   - Qualify S04-12: masked entry passes in `/providers`, but not in onboarding.

4. **Non-blocking. The real-token slip (c1) was not fully checked.** A real Claude token was pasted without bracketed paste, and the TUI exited after 32 characters.
   - `leakscan.py` only searches for whole values, so a partial piece of that token in logs, scrollback or the vault would not be found.
   - The full c1 transcript is not included.
   - Say that a partial leak is unverified, and add a scan for overlapping substrings of each real key.

5. **Non-blocking. This is not the policy's code-blind gate.**
   - "Code-blind" is self-attested, with no recorded failed-access checks.
   - The same agent designed and ran the cases.
   - It used a debug build, not the packaged candidate.

   The README admits there was no separate designer. It should also say plainly that the sprint's "Independent code-blind functional execution" item and the handoff-checker item stay open.

6. **Non-blocking. Broken link.** The README links `REVIEW.md`, which is not in the directory.

7. **Non-blocking. Some verdicts should be stricter.**
   - **S04-16:** should be PARTIAL. The ChatGPT sign-in refusal and the Bedrock/AWS refusal were never run; only the OpenAI API-key path was.
   - **S04-2:** the Claude Code config dir `cfg` was saved but never used for a request. The sprint's "Claude login" way of adding an account was not run. Should be PASS with gaps.

8. **Non-blocking. Some claims about which account a request used are weaker than stated.**
   - The proxy log has no timestamps or session IDs. "No other canary in that turn" depends on the order in `transcripts/p2.txt`. That order does support it: the 401, then a rename, then a new session on `spare` giving `pmain`.
   - The proxied provider ran with `request_max_retries = 0` and `stream_max_retries = 0`. So "no retry" was shown with retries turned off.
   - The real Z.AI, Kimi and Claude 401 tests had no proxy, so nothing records which account each request carried.
   - The inherited spawn's `child-ok` cannot tell `main` from `default`, since both hold the real key. That is fine for a display check, but should not be presented as proof of which account the child used.

9. **Non-blocking. The S04-15 transcript is cut short.** In `transcripts/p3.txt` the last pong prompt has no visible reply. "Stayed on work" rests only on the 10th proxy line.

10. **Non-blocking. Migration wording.** The plan row says "Each provider shows one `default` account row". With the flag on, `migration.txt` (`m2-flagon-rows`) shows a plain `Z.AI` row; the `· default` label appears only after a named account exists. The README cites `/status` `· account default` instead. The product owner should confirm the row label is acceptable.

11. **Non-blocking. Untested cases.**
    - The one-provider-at-a-time session-selection refusal (a known limit).
    - Renaming an account to a name that already exists.
    - A restart keeping two Claude Plan accounts (only Z.AI was checked).
    - Multi-agent v2 spawns.

12. **Secret scan: clean.** A pattern scan of `captures/` found only session UUIDs, request IDs and cf-ray values. The proxy log records canary IDs and 12-hex hashes, not values. The only real-key hit in the leak scan is `$S/homes/hC2/auth.json`, in a deleted test home, and the report lists it by path only.

## Summary

The method is mostly sound: `env -i` with test homes, `CORBANU_TEST_NO_NATIVE_KEYRING=1`, real keys fetched only inside the command that uses them, a canary proxy that shows which account each request carried, and baseline comparisons. Most quoted text checks out. The recommendation (accept behind the default-off flag, fix #461 first) fits, since the onboarding *Add another account* path only appears with the flag on (`f3-flagoff-chooser`).

What needs fixing is the report's honesty, not the product:
- relabel or back up the hash, config and rule-text claims;
- either run the vault metadata listing or downgrade S04-4;
- treat #461 as a masked-entry security defect and qualify S04-12;
- mark S04-16 and S04-2 partial;
- state that the sprint's code-blind gate is still open.

None of this needs a new product run except the optional vault listing.
## Author's response (changes made after the review; no second review was run)

1. Missing artifacts: output that was only on my terminal is now copied verbatim in `captures/console-excerpts.txt`. The Esc hash check and the migration check were re-run, and their output was saved (`captures/esc-rerun.log`, `captures/migration-rerun.log`). The scratch homes still existed when the review ran. The shim's missing log is recorded, and the README now says what the shim cannot see.
2. Vault index: done. `/vault` → *View credentials* listed `provider/zpx/accounts/{aa1,aa2,quota}/api_key` before and only `aa2` and `quota` after removing `aa1` in `/providers` (`captures/vault-index.txt`). S04-4 stays PASS.
3. #461 is now marked security-relevant (comment and title updated). S04-12 is split into PASS for `/providers` and FAIL for onboarding.
4. Partial-key scan: 16-character windows of each real key, over every home, log and capture. Only the OpenAI key in `hC2/auth.json` (`captures/partial-scan.txt`). The c1 screen is in `transcripts/c1.txt`.
5. The README now says the code-blind handoff-checker item and the milestone VM gate stay open, and that code-blindness is self-attested.
6. REVIEW.md: this file.
7. S04-2 is split into 2a PASS (keys and tokens) and 2b NOT VERIFIABLE (Claude Code login and sign-in). S04-16 is split into 16a PASS and 16b NOT VERIFIABLE (partly seen).
8. The attribution limits are stated in Method: no timestamps, retries off on `zpx`, no per-request attribution on the real endpoints, and `main`/`default` sharing a key.
9. The S04-15 wording now names the 10th proxy line and says the reply did not render.
10. Migration row label: noted under S04-14 for product.
11. Rename to an existing name: tested and refused. A restart with two Claude Plan accounts: tested, and both rows persist (`captures/vault-index.txt`). The one-selection-per-provider refusal and multi-agent v2 are listed as not tested.
