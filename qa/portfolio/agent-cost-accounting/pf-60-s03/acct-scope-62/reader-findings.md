# Reader finding — scope-zero (P2)

This is supporting code-aware evidence from `scope-run-01`, not an independent
functional acceptance. Product code is unchanged.

## Starting state and action

At fixed UTC time **2026-08-10T12:00:00Z**, send four actual local-fixture prompts
through the developer-accounting CLI: two in root A, one in root B, one in root C.
Each successful OpenAI/Sol response reports input 100, cache read 20, cache write
0, output 10 and reasoning 4. Native admission binds a price snapshot for each.
Open a fresh session in that same synthetic profile, send **`/usage requests
2026-08-10`**, then Enter separately. Traverse the page with actual keys.

## Exact rendered text beside the store evidence

The fresh-session page renders:

> No recorded attempts in this day; collection coverage unknown.
> Known subtotal exact USD: 0
> 90-day wall-clock detail cutoff: Some(1778587200000); aggregate day floor at checkpoint: 20311; oldest recorded day: None

The full page also contains these mitigating scope/estimate qualifications:

> Collection coverage: unknown; recorded root and resolved descendants only. Unknown parent population excluded.
> Billed cost: unavailable — no settlement evidence
> Root total = own attempts + resolved descendant attempts. Provider/model groups partition the same root total. Compare exact USD, not rounded displays.

Its title is **“Recorded requests — root and descendants”** and its requested-day
header is **“Requested UTC day: 2026-08-10”**. Thus “nothing names the scope” would
be too literal: the problem is that the primary empty-state sentence, zero
subtotal and oldest-day label fail to attach that scope to the conclusions.

At that exact inspection, the same store holds these four priced attempts.
All were admitted at **1786363200000 ms UTC**, on the requested day:

| Attempt ID | Root/thread ID | Provider/model | Exact estimated USD |
| --- | --- | --- | ---: |
| cd95c387-e1c2-46ec-99fe-65ad38c17b2c | 019feb8b-4200-76f1-9b4c-60c4ffdd807b | openai / gpt-5.6-sol | 0.00071 |
| de749048-449c-49c8-86fc-dc4f478663e7 | 019feb8b-4200-76f1-9b4c-60c4ffdd807b | openai / gpt-5.6-sol | 0.00071 |
| 05a7c6e3-9589-475f-8481-dcc40695f8f2 | 019feb8b-4200-7b50-a9f0-88b2f8955f4e | openai / gpt-5.6-sol | 0.00071 |
| 89ee3e05-2ad8-4b89-b38c-470e72432863 | 019feb8b-4200-7790-9681-077c69d95463 | openai / gpt-5.6-sol | 0.00071 |

Independent arithmetic per attempt: **(100−20)×5/1,000,000 + 20×0.5/1,000,000
+ 10×30/1,000,000 = 0.00071 USD**. The known recorded day sum across those roots
is **0.00284 USD**; it is an estimate, not a measured bill. The store contains
both initial and final estimate revisions: the audit selects each contribution's
exact evidence key, never sums the eight quote rows as eight attempts.

Readback immediately before and during the fresh reader is identical. Reopening
A, B and C through real keys shows **0.00142**, **0.00071**, **0.00071 USD**.
All four attempt details also reconcile; seven priced pages are checked.

**Verdict: P2 scope/coverage misread.** A user asking what they spent on this day
from a new session can read the primary zero as a day-wide result even though
collected, priced evidence exists in the same store. The zero is arithmetically
correct only for the selected root and descendants. Generic scope qualifications
mitigate the risk but do not make “No recorded attempts in this day” a reliable
statement about the requested day across sessions. No product fix is made here.

**What the page should say:** “No recorded attempts for this session and its
resolved descendants on 2026-08-10. Other sessions are excluded. This is not your
total spend for the day.” Label the numeric result **“Known subtotal for this
session: USD 0”** and history **“Oldest recorded day in this session: none.”**
Keep the unknown collection and billed-cost qualifiers. Do not claim the UI can
already compute a cross-session total or expose an all-session navigation action.

Evidence: [exact selected text](scope-run-01/fresh-scope-zero-selected.json),
[all viewports](scope-run-01/fresh-scope-zero.txt.gz),
[raw PTY](scope-run-01/fresh-scope-reader.raw.gz),
[keys](scope-run-01/keys.json), [emissions](scope-run-01/emissions.json),
[before store](scope-run-01/store-before-fresh-reader.json),
[during store](scope-run-01/store-during-fresh-reader.json),
[root controls and four identities](scope-run-01/scope-zero-result.json),
[independent audit](scope-audit.json), [exact binary](scope-run-01/manifest.json).

## Correction to the frozen brief's original counts

The P2 remains valid, but `acct-readers-61/reader-run-05/emissions.json` records
**four emitted responses** on August 10, not four collected, priced attempts.
Its store bindings hold four attempts **across two days**, as follows:

| Original attempt ID | UTC day | Model | Price / usage / complete estimate |
| --- | --- | --- | --- |
| 6b671f36-55d4-4a42-ae64-c539e64f3d34 | 2026-08-08 | gpt-5.6-sol | Price and usage; USD 0.00071 |
| f00a2eba-eeb2-40b4-95aa-f42752260380 | 2026-08-10 | gpt-5.6-sol | Price and usage; USD 0.00071 |
| a206d847-73f3-4643-9140-a6ed82b3fe8f | 2026-08-10 | gpt-6-astra | Usage, no price; unknown |
| 06ff0e1c-b6ad-49ff-b87c-32ce8cf8497a | 2026-08-10 | gpt-5.6-sol | Price, no usage; unknown |

The fourth August-10 emission was `reader_local`, which was not collected.
Thus that day has three collected attempts across three roots, only one complete
priced estimate. The new reproduction above deliberately supplies the stronger
four-collected-and-priced same-day condition without rewriting the prior run.
Original rows and full thread IDs are retained in [scope-audit.json](scope-audit.json)
and the original [native bindings](../acct-readers-61/reader-run-05/native-bindings.json).
