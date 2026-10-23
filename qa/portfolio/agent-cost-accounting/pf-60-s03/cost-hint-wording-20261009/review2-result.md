**Verdict: APPROVE WITH NITS.** M1 is fixed, and Minors 3, 4, 6 and 7 are fixed or mostly fixed. I accept the reasons for leaving Minors 1, 2 and 5 as they are. I found no new Major issue. I only read the code (the sandbox is read-only), so I didn't run tests or clippy.

## Round-1 items

**M1 — fixed.**
- `states_charges` (`tokens.rs:1537-1546`) lists exactly the routes that record a charge in core: `accounting_chat.rs:357-361` (openrouter, pfterminal-plan), `accounting.rs:1132-1133` (pfterminal-plan-anthropic) and `accounting_responses.rs:375` (vercel).
- `unstated_charge` (`:1581-1622`) now splits providers into two groups: "doesn't report its charges" for providers that never report, and "stated no charge for this work" for providers that report but didn't here.
- When the only billable attempts are from reporting providers and reported no usage, the line reads "none stated … a refused request is normally not charged" (`:1644-1657`).
- The test that pinned the contradiction now checks the new wording ("OpenRouter stated no charge…", `tokens_tests.rs:2629-2640`).

**Minor 3 — fixed.** On the own and descendant pages, the billing line now comes after the estimate (`tokens.rs:1078-1080`). The unknown-parent page also has the estimate first (`:1136-1143`).

**Minor 4 — fixed.** The hint is hidden when `!self.blocks_direct_input` fails (`chat_composer.rs:4828`), and a test covers it through `set_parent_owned_thread` (`:5401-5405`).

**Minor 6 — mostly fixed.**
- **Now covered:** exactly one billing line per page (`tokens_tests.rs:2646`); never-reports, stated-none, mixed, three-name commas, the Corbanu API merge, refused, no-id fallback, plan-only and empty (`:2672-2719`); range overview complete vs partial (`:2724`); hint with several spaces, a tab, the 40-column box and a parent-owned thread.
- **Still untested:** the overflow line, a range that is complete but still in progress, and bash, masked or disabled input. These are nits.

**Minor 7 — the captures are committed but stale (see New Minor 1).**

**Nits from round 1:**
- **Fixed:** repeated names and plural "their bills"; "not reported — … doesn't state"; the hint cut off mid-word (it now shows only when the whole hint fits, `:4838`); the tab-width mismatch (`slash_input.rs:129`); `hint.dim()`; the obscure `rest_offset` check.
- **Still open:** the cursor still sits on the hint's first character.

**Declined items — I accept all three:**
- **Minor 1:** "at least $X (1 of 3 attempts stated a charge)" is true, and the owner asked to keep it.
- **Minor 2:** "any cost here" no longer claims there is an estimate. The no-price page still says "check … local-mock" twice (snapshot `cost_missing_price_request_page.snap:9,12`), which is acceptable.
- **Minor 5:** you reported that local and Linux runs passed with the feature. I can't verify that from here.

## New findings

**Minor 1 — the QA captures are older than the final tree.**
- `qa/.../cost-hint-wording-20261009/README.md:14` says the build came from `c97a5fa4bf`.
- Captures `10`, `11`, `12`, `13`, `20` and `21` still show the old "actual charge / so this is an estimate" wording.
- `914024b6e1` changed every billing line and the hint behaviour, so this tree has no true-TUI proof yet.
- Re-capture from the final commit before handoff.

**Minor 2 — `Basis::Local` is excluded in one place but not the others.** This is latent, since core doesn't record `Basis::Local` yet.
- `has_bill` excludes Local (`tokens.rs:1549-1551`). Three other places only check `is_plan()`:
  - `billed_charge` (`:1508`) counts Local attempts. A page with one OpenRouter attempt that stated a charge plus one local-model attempt would say "at least $X (1 of 2 attempts stated a charge)". The same count feeds the filter at `:1696`.
  - The overview (`:1856`) treats a day of only Local work as having a bill. Its fallback says "your provider's bill is the final amount" even though no bill exists.
- No test covers Local.
- **Fix:** use `has_bill` in all three places, or record this as a follow-up for when core starts writing Local.

**Nit 1 — "normally not charged" may be too reassuring.**
- An empty-usage attempt can be an interrupted stream, which `BilledCharge` itself says "may still have been billed" (`:1466-1468`).
- The `:1655` line drops the pointer to the bill for that case. Consider adding "…; check OpenRouter's bill if it failed mid-response."

**Nit 2 — two pages can describe the same attempt differently.**
- On a day where every attempt was a refused OpenRouter request, the overview says "OpenRouter stated no charge for this work — check OpenRouter's bill" (`:1864`).
- The request page for the same attempt says "none stated … normally not charged".

**Nit 3 — long clauses on mixed pages.** With several names on both sides, the line reads "a and b don't report their charges and OpenRouter stated no charge for this work, so…". It reads acceptably.

## Clippy risk

**Low.** Nothing I can see would trip `-D warnings` in either feature set:
- The new functions are all used.
- The closures `|quote| has_bill(quote)` need an auto-deref, so `redundant_closure` shouldn't fire.
- `text.contains(['\n', '\t'])` and `_rest_offset` are fine.
- The `setup: impl FnOnce(&mut ChatComposer)` parameter accepts `ChatComposer::set_parent_owned_thread`, which is not feature-gated (`chat_composer.rs:1563`).
- The hint test follows `cfg!(feature = "developer-accounting")`, so it holds in both builds.
- The `#[allow(clippy::trivially_copy_pass_by_ref)]` in `state` comes from the merge, not this change.