# Review disposition (Opus 5.5 High, three passes)

1. **First pass (approve with fixes) on wrapper parsing.** The many parsing
   bypasses it found led to the rework. The zsh-fork path, the docs and the
   allow-widening test were added.
2. **Second pass (request changes) on argv suffixes.** Fixed:
   - the original command is scanned (heredocs);
   - splitting goes to any depth, on shell punctuation;
   - option skipping has no fixed cap;
   - launcher suffixes and both cases are matched;
   - duplicate matches are removed;
   - the docs say best effort;
   - tests go through the approval path, and there is a zsh-fork strict test.

   Finding 6 (nested `corbanu exec`) is left for a product decision.
   Finding 9 (over-matching) is documented.
3. **Third pass (approve with fixes).** Fixed:
   - **1:** a second stream with quotes removed (`va''ult`, `c\orbanu`,
     `cor^banu`).
   - **2 and 3:** matching is rule-aware. Only multi-word forbidden rules are
     used, the next word is looked up by position (the first 8 occurrences),
     and there is no word cap.
   - **4:** single-word rules keep exact matching, and the refusal reason says
     a strict text match fired.
   - **5:** keys are compared in lowercase.
   - **6:** the doc lists more out-of-reach forms.
   - **7:** "never less restrictive" is checked over every case.
   - **8:** known gaps are asserted.
   - **9:** formatting is committed.
   - **10:** a comment says the cluster rule is loose on purpose.
