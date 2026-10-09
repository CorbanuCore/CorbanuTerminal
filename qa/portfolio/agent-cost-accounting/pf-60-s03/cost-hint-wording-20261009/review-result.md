**Verdict: REQUEST CHANGES.** There is one major issue: some pages can say "OpenRouter doesn't state its actual charge" when OpenRouter does report its charge. Everything else is minor or a nit.

I only read the code. The sandbox is read-only, so I didn't run any tests or clippy.

## Major

**M1. A charge missing from one response is reported as the provider never reporting charges.**
- `unstated_charge` (`tokens.rs:1534-1566`) decides what to say from each response alone. If an OpenRouter, Corbanu API or Vercel response has no charge on it, the provider is listed as one that "doesn't state its actual charge".
- Those providers do report charges: see `core/src/accounting_chat.rs:357-364`, `core/src/accounting.rs:1132`, `core/src/accounting_responses.rs:375`. A response can still lack one, for example a failed or refused attempt with no usage, an interrupted stream, or a figure the decimal type can't hold.
- Where it shows up:
  - **Attempt page for a failed retry.** `plain_header(&[quote])` at `:987` leads to `billed_line` at `:1587-1591`, which says "OpenRouter doesn't state its actual charge… check the bill from OpenRouter". The existing code already says such an attempt is "normally not charged" (`:1516`).
  - **Request page for a single failed attempt.** Same wording.
  - **The day overview,** when another conversation used the same provider (`:1792`).
- The test pins this contradiction. In `accounting_inspect_states_the_providers_billed_charge`, the overview shows "Billed by alpha: $X" and the alpha · two request page then asserts "alpha doesn't state its actual charge" (`tokens_tests.rs:2639`).
- This breaks the brief's rule: never claim "not reported" where a charge was stated.
- **Fix:** for providers known to report charges, describe the missing charge for this request only, e.g. "OpenRouter stated no charge for this request". If the attempt reported no usage at all, reuse the existing "a refused request is normally not charged" wording. Keep "doesn't state its actual charge" for providers that never report one. One shared provider check in core would keep the list in a single place.

## Minor

1. **Partly reported charges with several providers.** The figure reads "Billed cost: at least $X (1 of 3 attempts stated a charge)", and the overview says "billed figures are what the provider stated…" (`:1783`). Nothing names the provider whose share is only an estimate, such as Z.ai on a day mixing it with OpenRouter. This falls short of "naming the provider(s) where known". Consider adding which provider's bill to check for the rest.
2. **"so this is an estimate" where there is no estimate.** The no-price request snapshot shows "Estimated cost: no price available", then "Billed cost: not reported — … so this is an estimate; check the bill from local-mock." That page also says "check the bill from local-mock" twice, because the no-price next step is still there.
3. **Line order on the own/descendant attempt pages.** The billing line comes first (`:1056`), before the estimate it refers to. Previously it came last, and on provider pages it sits after "—— Details ——".
4. **Hint on threads owned by a parent.** The hint appears there even though submitting `/cost` with arguments is blocked (`parent_owned_command_is_allowed`, `chat_composer.rs:356`, which doesn't allow `/cost`). Suggest hiding it when `blocks_direct_input` is set.
5. **Tests run by CI never check that the hint appears.** `slash_command_argument_hint_shows_until_an_argument_is_typed` only asserts it appears when built with the developer-accounting feature. The CI workflow neither tests nor runs clippy on `codex-tui` with that feature. The step at `corbanu-terminal-ci.yml:167` only builds with the `codex-core` feature. Please record a feature-enabled test and clippy run.
6. **Missing test cases:**
   - **Billing lines:**
     - Every page that has attempts shows exactly one line (the current test only checks at most one).
     - The all-subscription page shows "Billed cost: none — …".
     - The overflow message.
     - The fallback line for an unknown provider.
     - Three or more names joined with commas.
     - `pfterminal-plan` and `-anthropic` merged into one "Corbanu API".
     - Unknown-parent page and its attempts.
     - Range overview: the line appears when complete, including in progress, and is absent when partial.
   - **Composer hint:** bash mode, masked input, input disabled, typed text as wide as the box, `/cost   ` with several spaces.
7. **The QA evidence isn't committed.** `qa/portfolio/.../cost-hint-wording-20261009/` is untracked in this checkout.

## Nits

- **Wording:**
  - The overview names providers twice: "local-mock and OpenAI don't state their actual charge, so check the bill from local-mock and OpenAI". It also says "the bill" for several providers. Suggest "…; local-mock and OpenAI don't report what they charged — check their bills."
  - "not reported — X doesn't state" says the same thing twice.
  - The fallback says "the provider" on pages with several providers.
  - Nearby text uses "did not"; the new lines use "doesn't".
- **Repeated lines:**
  - Subscription pages repeat the billing type: "Billing: Covered…" then "Billed cost: none — covered…".
  - Pages with a reported charge still show both "Billed by provider: $X" and "Billed cost: $X…". This was already the case before the change.
- **Composer hint:**
  - At narrow widths it is cut mid-word ("…START END hou" in capture `03`). Consider only showing it when the whole hint fits.
  - The cursor sits on the hint's first character.
  - The typed width uses `UnicodeWidthStr` directly, but the text box draws tabs as one column (`textarea.rs:83`). Using the same tab handling would be safer, though tabs only arrive by pasting.
- **Style:** `Span::from(hint).dim()` could be `hint.dim()`, as the TUI style guide suggests. `rest_offset == name.len() + 1` is obscure; a check for whitespace right after the name would read better.
- **Footer note:** it takes one or two rows from the scrolling area on short terminals. "START END" doesn't say that dates or UTC timestamps ending in Z are accepted. The full rules are in the usage message, so this is acceptable.

## Answers to the six questions

1. **Each page type:**
   - **Not ready / diagnostic pages:** no billing line, so they have no billing line.
   - **Overview:** one estimate line, plus a "Billed cost:" line only when a charge was reported. The two branches agree because both use `billed_figure(all)`.
   - **Provider, request and attempt pages:** one line each.
   - **Own and descendant attempt pages:** one line, none when the group is empty.
   - **Unknown provider/model attribution page:** none when empty. When there is unknown attribution, its provider page uses the fallback wording.
   - **Unknown-parent page and its attempts:** one line each.
   - **Range overview:** zero or one.
   - **Range bucket pages:** this change removed the old line from the shared header (`context`), so they no longer carry a duplicate.
   - **What it never does:** invent a figure, or point subscription work at a bill. Subscription attempts are skipped at `:1540`.
   - **The failure:** M1, plus the partial gap in Minor 1.
2. **Range overview only when complete:** sound. A billed sum over partial buckets would be exactly the partial total the screen refuses to give. When the range is complete but a day is in progress, the "In progress — totals so far" line comes right before it.
3. **Composer hint:**
   - **Excluded correctly:** multi-line text, bash mode, masked input, disabled input and no-room cases (`slash_input.rs:128-132`, `chat_composer.rs:4826-4837`). It sits on the first row of the text box, which is correct for a single line.
   - **Popups:** no overlap.
   - **Wide text:** command names are ASCII, so only odd whitespace after the name matters, and that is handled except for tabs.
   - **Scope:** only `/cost`, and only where `/cost` is visible, because the lookup goes through `is_visible`. Side conversations allow `/cost`. Parent-owned threads are Minor 4.
4. **Default build:** no change users can see. `/cost` isn't visible, so no hint appears, and the view never opens, so the footer never shows. Every new item is used in both builds and I see no clippy risk from the lints the workspace denies. This was not run; see Minor 5.
5. **Tests:** the updated tests would fail on the old text: the "settlement" check, `NOT_STATED`, and the snapshots. But one test enshrines M1, and the gaps are in Minor 5 and 6.
6. **Wording:** short and plain overall, but repetitive in places (see Nits). Fix M1 first.