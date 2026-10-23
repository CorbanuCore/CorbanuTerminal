#288's reported defect is fixed: a conversation whose requests come back with no usage now tells you to check that conversation's own provider's bill. The two priced-provider cases couldn't be tested. All the #289 leftovers still happen. The evidence is merged as docs-only **PR #315** (merge `ef512a9d7c`).

I built `origin/main` `886f19c873` on macOS, with developer accounting and as a default build. `git merge-base --is-ancestor f0d71e62b6 HEAD` returned 0, so #305 is included. RTX wasn't needed. Every displayed number matched my own recalculation from the token counts Z.AI reported.

**Verdicts**
- **8a. The no-usage conversation's next step names its own provider: PASS.** Its `/cost`, provider, request and technical pages all say "check the bill from **zainousage**". Other conversations' pages now name it too (`mac-c8-01`…`05`, `-10`, `-13`).
- **8b. Missing usage told apart from a missing price: NOT VERIFIABLE.** The "usage incomplete" label only applies to a priced provider, and the built-in priced `zai` can't be pointed at the proxy ("Built-in providers cannot be overridden"). On the custom provider the page says "tokens not reported" and "no price available", which is true there because custom providers have no price.
- **8c. Request priced only in part: NOT VERIFIABLE**, for the same reason. The closest case I could build was a custom route whose proxy drops only the output count: its next step names its own provider, and output shows as "not reported" rather than 0.
- **#289 default build, `/usage requests 2026-10-07`: FAIL (unchanged).** It still says "if this build records costs, send a turn…", even after a real turn in that home (`mac-289-01`).
- **#289 default build, `/usage requests 2026-10-09`: FAIL (unchanged).** It still says "Run /usage requests for today", which leads to the same "ledger not installed" page (`mac-289-02`, `-08`).
- **#289 week/month bucket reaching today: FAIL (unchanged).** It is still titled "Bucket unavailable", with a "send a turn … Refresh" step that can't complete it (`mac-289-05`, `-07`).
- **#289 retention wording: FAIL (unchanged).** The range header says "oldest daily total kept none"; the day page in the same home says "daily totals kept since 2025-10-09".
- **Criterion 13a. `/cost` absent from a default build: PASS on macOS** ("Unrecognized command '/cost'").
- **Criterion 13b. Default build opening the "Cost — this conversation" page for `/usage requests`: NOT VERIFIABLE.** It's a product decision, not a test result; behaviour is unchanged.

**Review:** one code-blind reviewer (Opus 5.5 High, read-only) concluded "Supported with corrections". Its first pass ran on a copy missing the README because I made a copy error; the second pass covered the full record. Both reports are in `REVIEW.md` and its corrections are applied.

**Issues:** #288 was already closed, so I commented there instead of closing it; the comment replaces the earlier "still FAIL". I left #289 open with the new results and one new low-severity point: a request rejected with a 401 (no response) is counted as a "billed" attempt with "check the bill".

**Housekeeping:**
- Spend was about $0.13.
- The key scan found 0 files across the evidence, every log, the scratch homes and the reviewer's copy.
- No real keychain was used.
- The scratch homes, target dir, worktree and branch are removed.

**For Travis**
- **Decision on criterion 13:** should a default build open the "Cost — this conversation" page for `/usage requests` at all? If yes, its next step shouldn't say "if this build records costs", because a default build never records.
- **Priced-provider cases (8b, 8c):** testing these end to end needs either a test hook or a way to point a priced provider at a proxy.
- **Old #288 item 3:** custom providers are still labelled "Pay per use". The sprint record already tracks this as an open review finding.