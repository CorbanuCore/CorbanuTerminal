# PF-28-S01 security review 6 (Opus 5.5 High, re-check of 7efea30fb7)

## Verdict: APPROVE WITH NITS

V1 and V2 are fixed. N3 is recorded and N4 is tested. This commit introduces no regressions, and behaviour with the
flag off is unchanged. Nothing here blocks shipping behind the default-off `secret_output_gate` flag.

Checks:
- `git show 7efea30fb7`;
- `disclosure_gate.rs` `present`, `strip_value`, `gate_value_with`, `scrub_json` and `survives_walk`;
- `output_gate.rs` `representations` and `admit`;
- the vault and login registration callers;
- README "Known limits".

Test runs:
- `cargo test -p codex-secret-broker`: 60/60 pass, including every `pf_28_s01` test.
- `python3 docs/sprints/check.py`: clean.
- I did not build core, per the brief. I traced the three new core tests by hand.
- I re-ran the review-5 scratch binary (`review/scratch5`, synthetic value). The serialized quick scan now finds the
  doubly escaped form (before: miss), and the per-field scan still finds it.

## Verified

- **V1** (`disclosure_gate.rs:290-296`, `:396-412`):
  - Take a clean composite over 16 MiB. The quick scan hits on size alone. The walk then scans every string under
    the limit and changes nothing.
  - `survives_walk` skips the whole-document rescan when the document is over the limit and checks only number
    leaves, which miss here. Because `!changed`, the result is `Unchanged`.
  - `strip_value` (`:255`) uses the same helper, so the `present` fallback no longer turns a large clean
    `SessionConfigured` into an error notice.
  - Over the limit, a number hit is still `Withheld`. `strip_value` cannot clear a number, so the result is the error
    notice. This fails closed.
  - Test `:551` covers it.
- **`!changed` short-circuit:** this branch is reached only when the walk changed nothing and the rescan missed.
  - Under the limit, the rescanned text matches what the quick scan saw, so the rescan hits again and the value is
    `Withheld` first.
  - The one exception is key reordering when serde_json `preserve_order` is off. The release binary enables it
    through `tui`.
  - In that case the old code returned `Changed` with identical content, so returning `Unchanged` delivers the same
    bytes. This is not a regression.
- **V2** (`output_gate.rs:913-922`):
  - The doubly escaped form is added only when it differs from the existing forms (the push deduplicates). That
    means only values containing `"`, `\` or control characters get an extra form.
  - The extra form is pushed before `base64_start`, so the base64 index range is unaffected.
  - It is subject to `MAX_REPRESENTATION_BYTES` and the same capacity accounting.
  - Test `:574` covers it.
- **N3:** recorded in Known limits.
- **N4:** test `:562` covers the key branch of `strip_value`.
- **Flag off:** no change.
  - Every core entry point still returns early on `active()`.
  - The vault (`vault/src/lib.rs:743`) and login (`storage_gate.rs:52`) register values only when the gate is
    armed. So the extra form cannot exhaust capacity or change any flag-off path.

## Nits (non-blocking)

- **M1.** In a document over the limit, `survives_walk` does not check object keys either. This has no practical
  effect: `scrub_json` and `strip` already gated every key, so a surviving key hit would need a match that spans
  fields, and that is already a Known limit. A one-word addition to the Known-limits sentence ("spanning fields or
  keys") would make it exact.
- **M2.** Detection is one escaping level deep. Text that already holds the doubly escaped form is escaped a third
  time inside a serialized item. The quick scan misses that form, so the value passes raw. This needs JSON nested in
  JSON inside tool output, so it is not realistic. If you want the README to be exhaustive, add it to Known limits
  next to the V2 note.
