**VERDICT: fail.** Every disposition is supported, but two claims in the record are false as it stands. All 64 isolation receipts point to a policy file that has since been edited, so their hashes no longer verify. The AGG-20 model field in `results.json` is also wrong. Both are mechanical fixes; no disposition needs to change.

## 1. Completeness: holds
- **Cases:** `design.json` has 57 unique cases. `results.json` has exactly one disposition per case, in the same order: 41 passed, 10 failed, 6 blocked. This matches `README.md` and `dispositions.md`.
- **Hashes:** `design_sha256` matches `design.json`, and all 1,551 path/hash references in `results.json` resolve and match.
- **Frozen design:** all 68 entries in `design/FROZEN.sha256` verify. `design.json` equals the 39 frozen cases, plus the 15 amendments, plus the 18 new cases, with no differences.
- **Attempts:** all 34 attempt labels exist under `runs/`, with no extra folders. AGG-R06 has no attempts and cites `design/packet/claude-availability.txt`.
- **Checker:** `checker-output.txt` reflects the current state (it stops at AGG-01: failed; 10 failed, 6 blocked).

## 2. Dispositions: all supported by the evidence, with the cautions below
I read every executor report and checked the key screens and mediator windows.
- **Failures are evidenced:**
  - **AGG-01:** `runs/G01` screens `AGG01-inspector-scrolled1/2` differ from `design/first-run-screens/03*.txt` in the Vault, Output gate and Nested agents rows.
  - **AGG-18, R17:** the Grant once write was still refused with "operation not permitted" (`runs/G10`, `runs/G28`).
  - **AGG-24, R09:** the inspector shows "Recent denials: None".
  - **AGG-25:** preflight passed with the E1 flag.
  - **AGG-27:** "Cannot save … provider zai is inactive" (`runs/G17b`).
  - **AGG-32:** "preflight clean at launch" with `/etc/codex/config.toml` present (`common/g21-fixture-on.txt`).
  - **R07:** the `os.listdir` form offered Grant once.
  - **R13:** the symlink `ag -> .agents` was created.
- **Integrator overrides are adequately reasoned:** AGG-06, AGG-10, AGG-30, AGG-36, AGG-20, AGG-R14, AGG-38, AGG-R15 (G28 shows two Enters at 11:25:33Z and 11:25:35Z in `actions.jsonl`), and AGG-27 (the error appears; no Permissive comparison was made, which is stated).
- **The "fix removed the starting condition → blocked" pattern is not present.** AGG-25 and AGG-32 are marked **failed**, not blocked, against their frozen expectations, with the cause and the owner question in `review/fix-references.md`. That is honest:
  - AGG-32's precondition was present.
  - AGG-25's precondition was reproduced exactly as the case defines it ("screen E1 condition"; G15 note).
  - Blocked would also have been defensible for AGG-25, since #438 means the flag is no longer an unmanaged secret.
- **Claims that rest on mediator windows, which I re-checked:**
  - **AGG-R01:** connection `…55714` ends with a broken pipe at 12:26:31Z, 6 s after the 12:26:25Z confirm. The next product-sized request is at 12:30:55Z (`runs/G22b/mediator-window.jsonl`).
  - **AGG-R04:** no product request between 12:32:43Z and 12:34:33Z.
  - **AGG-22:** only executor-sized, steadily growing requests between 08:18 and 08:22Z (`runs/G13`).
  - **AGG-28:** no direct product request around 09:14:40Z (`runs/G18`).
  - **AGG-14:** a single product request at 06:34:57Z (`runs/G07`).
  - **AGG-13:** no `web_search` tool was offered.
  - **AGG-A01:** two glm-5.2 requests, both status 200.
- **Cautions, not errors:**
  - The integrator's line is: a model decline that literally meets the expected text passes with a disclosure (AGG-14, 21, 30, R11, R16); a result left literally unexercised is blocked (AGG-20). That is consistent, but **R11 and R16 never exercised the product protection at all**; the owner should know that.
  - In AGG-30, the **curl** denial was also written by the model (screen S16). The row says only that the `.env` refusal was.
  - The `dispositions.md` rows for AGG-22 and AGG-28 cite only `result.md`, but the G18 executor had no relay log. The README limits cover this.
  - AGG-26's "curl as in Permissive" has no in-run Permissive baseline; the executor noted the limitation.

## 3. Replays: hold
- There are 34 distinct executor thread IDs, each matching its attempt's `agent`, and all earlier attempts are kept.
- The notes in `tools/replays.json` and the prompts (for example, `executor-packet/v3/prompts/G28b.md`) give fixtures and navigation only.
  - G22b and G28b describe what attempt 1 did, but state no expected outcome.
  - G28b's "Do not approve or decline anything else" was passed into the product's prompt; this is harmless.
- **Model changes:** AGG-A01 uses glm-5.2, as the case requires. AGG-20's child ran on glm-5.3, chosen by the executor in `/spawn` and disclosed.

## 4. Isolation: one defect
- **What holds:** every `probes.json` passes (130 probes per run, no failures). Every `guard-after.txt` reads clear. Each receipt is bound to the right run, case, group, agent, design (`21fa87…`) and candidate. All restriction flags are true, and the probe and mediator hashes match.
- **Must correct:** all 64 `runs/*/isolation-*.json` receipts record policy `61921aa3…`, but `common/effective-policy.txt` now hashes to `3aa2341b…`.
  - The cause is the appended "Re-run note". Removing it reproduces `61921aa3…` exactly, so the policy itself did not change.
  - Fix: restore the original bytes and put the note elsewhere, or regenerate the receipts.
  - r2 reported "hashes match" but checked only the probe and mediator hashes.
- **Minor:** the file's header still names a `policy-root-end.txt` that does not exist (the bottom note explains why).

## 5. Other points before sign-off
- **Must correct:** AGG-20's `execution.model` field in `results.json` says "product under test: glm-5.3-flash". The child ran on glm-5.3 (`runs/G12/mediator-window.jsonl`, 08:00:22Z and 08:02:13Z).
- **README inaccuracy:** it lists G26 as having reset or reshaped the product home. G26 changed only the fixture (`runs/G26/executor-events.jsonl`).
- **Effort varies:** pane B ran at Low, the R14 child at max, and G17b changed effort to low. This is not recorded in the model fields.

## Status of the r2 points
| r2 point | Status |
| --- | --- |
| AGG-14: disclose the model decline; add to the README list | Resolved |
| AGG-20: blocked; child model disclosed | Resolved (the `results.json` model field is still wrong, see §5) |
| AGG-R14: blocked as partly unverified | Resolved (T17 confirms only the main pane's notice was seen) |
| AGG-01: summary contradicted the executor | Resolved (now failed, with all three rows named) |
| Disclosures for AGG-07, R16, R08, R07, AGG-15, AGG-16 | All resolved |
| README limits: child on glm-5.3, executor is the candidate binary, write access to the product home | Resolved (G26 wrongly included, see §5) |
| r2's claim that the receipt hashes match | Not correct for the policy hash (§4) |

VERDICT: fail