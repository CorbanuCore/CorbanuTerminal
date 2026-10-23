**Verdict: fail.** Most of the record holds up, but four items must be corrected, and ACC-15's stated reason for being blocked is wrong.

## 1. Order and freeze
- **Holds:**
  - `design/FROZEN.sha256` is stamped 02:53:41Z. The first VM start was 03:10:03Z (`common/vm-start.txt`); the second attempt booted at 03:24.
  - The candidate was installed at 03:08:51Z (`candidate/INSTALL.json`).
  - The v1 packet matches the frozen design byte for byte: `design-cases.md` = `designer-output.md`, plus `amendments.md` and `answers.md`. All 28 screens match.
  - `tools/packet-v2-diff.txt` reproduces exactly. v2 adds only the navigation note and the replay prompts C2 and D2 (D2 was never run). v3 adds only `E2.md`. `C2.md` and `E2.md` differ from C and E only in the assignment header and group note.
  - `amendments-final.md` is identical to the reviewer's amendments in `review-output.md` (from "### Amendments" onward).
  - Every `path`/`sha256` reference in `results.json` verifies.
- **Defect:** `tools/make_packet.py` no longer matches `FROZEN.sha256`: its hash is now 036a3c8c…, the frozen hash is 5863196e…. A `NAV_NOTE` hook was added after the freeze to build v2. This isn't disclosed in the README or in `effective-policy.txt`.

## 2. Cases
**Supported as written:**
- **ACC-06 (failed, #898):** `runs/A/ledger-attempts.txt` has six requests under thread 01a142f0-c302 with plain turn IDs (04:20:37–04:21:29Z). `/cost` showed "No other conversation" until 04:25:44Z. After the control exec, ACC-07 shows exactly 1 other conversation with 1 request, so the review requests really are shown nowhere.
- **ACC-04** (blocked): the model declined verbatim.
- **ACC-22** (blocked): the product said the priority tier is "not advertised" and omitted it.
- **ACC-35** (blocked): `d3.txt` says NOT GRANTED.
- **ACC-37:** all branches were observed — (a) OpenRouter GLM, (b) Vercel and Corbanu API, (c) mimo and uncatalogued.
- **ACC-16:** the ledger shows exactly one `prewarm:reopen` for the edit, in new conversation 01a14389 at 07:07:32Z. Passing it means reading the frozen "+1" as applying to that new conversation; that interpretation is disclosed.
- **ACC-14** (passed despite the skipped CHARLIE step) is defensible: the expected results don't depend on CHARLIE. However, see ACC-18 below.
- **ACC-34** (passed): the amendment makes the integrator's real-key scan binding. The C2/E2 "hits" came from case-insensitive matching, not exact matches as "literal" suggests.
- **The C2 and E2 replays were legitimate:** fresh agent IDs, profiles wiped first, first attempts kept, and both reasons are borne out by the first-attempt reports (`runs/C/.../result.md` and `runs/E/.../result.md`).

**Required corrections:**
- **ACC-15:** "the executor did not reach that line" is false. `runs/C2/collected/evidence/pty.raw.gz` shows the line "Turn: side:01a14382-9079-…" (pty offsets about 1,632,506–1,637,929). It was rendered on Request 8's technical page while the executor pressed Down about 40 times between 07:03:26Z and 07:03:34Z (`actions.jsonl`). It matches the ledger line `side:01a14382-9079-…`; it just isn't in a captured screen. Either pass the case on the pty evidence, or keep it blocked under the screens-only rule with the reason corrected. The README text needs the same fix.
- **ACC-18:** there are two unopenable conversations, not one "accidental fork":
  - 01a14387-6354… is the accidental fork.
  - 01a14381-44d6… is ACC-14's starting conversation, left without its CHARLIE turn. It holds only a warm-up request.
  - Each holds one warm-up request (ledger). Every view's counts show 5 other conversations against 3 that could be opened.
  - So the ACC-14 deviation did affect ACC-18, and this should be said.
  - Reviewers should also know: a conversation holding only a warm-up is billed but cannot be opened from `/resume`.
- **ACC-36:** the step "confirm the conversation is ALPHA" was not met. `resume --last` opened the edit branch 01a14389, and the executor marked this inconclusive. The disposition silently omits it.
- **Priorities:**
  - ACC-17 should be "blocker (reviewer part); sub-agent part advisory" (amendment 14).
  - ACC-34 should be advisory (amendment 21). `dispositions.md` contradicts itself here: the table says blocker, the summary says advisory.
  - ACC-35 is "optional (D3)" in the amendment, not advisory.

## 3. Isolation
**Holds:**
- **Receipts:** all 64 isolation receipts are bound to the right run, case, design hash (e6ac8f…) and candidate hash (5b805b…). Each run's probes pass 105/105, and each executor ran `agms-probe all` itself.
- **Gate:**
  - Attempt 1 failed on a read-only `config.toml`.
  - Attempt 2 failed because the probe counted the `CORBANU_TEST_NO_NATIVE_KEYRING` switch as a credential name; `ZAI_API_KEY` was the synthetic placeholder.
  - Attempt 3 passed, with the smoke turn `AGMS-SMOKE-OK`.
- **Guard:** 0 SecurityAgent prompts in 16,192 checks.
- **Relay:** real-host paths were allowlisted and other paths denied (`denied_path`). There were only 2 Moonshot POSTs against the cap of 12, and no cap denials.
- **Guest network change and restore:** `provision.sh` added Moonshot's addresses; `restore-end.out` shows the original anchor and hosts file back. The verification boot (08:06:31–45Z) then shows no hosts entry, accounts, `/opt/agms` or campaign keys. The VM stopped at 08:06:47Z and the lock was deleted at 08:06:54Z.

**Gaps:**
- Four provisioning runs (`provision-1` to `-4`) are unexplained.
- The anchor-address check in `vm-state-end.txt` is an unlabelled "0".
- The widened port-443 rule isn't scoped to one user: it applies to every guest account except the blocked executor accounts. The README says "for the coordinator account".
- `agms-product.v1-as-used-by-A-to-E` was also used by F.

## 4. README totals and claims
- **Totals:** 32/1/4 match `results.json`, `dispositions.md` and `tools/final.json`. The summaries are identical.
- **Spend checks:**
  - Moonshot: 0.036213 + 0.01099135 ≈ $0.047.
  - Z.AI: about $3.16, from 1,335,657 uncached input, 94,087,168 cached input and 273,365 output tokens.
  - Z.AI requests: there were 1,132 (1,131 on GLM 5.3 Flash plus one GLM 5.2 review), not 1,131.
  - OpenAI: the ~$0.26 excludes the unpriced Codex Auto Review requests.
- **Key scan:** only the run-folder scan has raw output (`keyscan-run-folder.txt`). The scan of the record itself is asserted only in a note.
- **Passed-by-sprint list:** ACC-01, ACC-29 and ACC-34 are missing from it.
- **ACC-15 line:** the README says "the executor didn't reach" the `side:` label; this must be corrected as above.

**Required corrections:**
1. ACC-15: correct the reason (and the README), or pass it on the pty evidence.
2. ACC-18: two residual conversations, including the one the ACC-14 deviation left behind.
3. ACC-36: disclose that the opened conversation was not ALPHA.
4. Priorities for ACC-17 and ACC-34, and ACC-35's optional status.
5. Disclose the `make_packet.py` drift from the freeze.

VERDICT: fail