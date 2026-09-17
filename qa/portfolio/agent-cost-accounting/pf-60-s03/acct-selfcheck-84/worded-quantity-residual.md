# Worded-quantity residual

Scope: the current `acct-acceptance-75/acceptance.md`, not linked documents.
This is a manual register by the acct-selfcheck-84 Astra High worker on September
17, 2026. It does not claim that arbitrary future prose is machine checked.

The literal regexes recognize “despite four priced attempts totaling”, “timed out
twice”, and “disclose the three round-75 edits”. Those particular statements are
bound to evidence. They are not a parser for cardinal/ordinal words: replacing
“four” with “five” makes the expected claim disappear, but adding another
sentence with “five” is not detected by the digit coverage guard.

Current unchecked semantic quantities and locations (identified by sentence
openings rather than changing line numbers):

| Sentence opening / region | Unchecked quantity or quantifier |
| --- | --- |
| “Use the inspector only for estimates…” | only, known root, other roots, same store/day, and the quoted “No recorded attempts”/coverage-unknown wording; numeric amounts are separately checked |
| “The title and secondary text…” | entire page and the scope of its conclusions |
| “No scope-zero fix or accepted waiver exists.” through “S03 is not accepted…” | absence of a fix/waiver, activation/shipping and acceptance status; the verifier does not infer authorizations |
| “Proven: saved evidence reconciles…” | “not … independent attempts” is not an independent uniqueness proof; category counts alone cannot prove it |
| “Integrity controls bind…” through “These are saved-corpus checks…” | unchanged rows, most assertions, only diagnostic mode, absence of independent acceptance/provider billing |
| “The round-76 gates…” and “Historical gate failure remains…” | separate/alone/full lane characterization and unexplained intermittency; raw counts/durations are checked, causation is not |
| “Attribution supports…” through “Subsequent passes…” | no contended resource, no excluded defect, no causal fix/relaxation/exemption |
| “Other limits remain…” through “Preparation is not execution…” | absent, whole-store, small, partial, complete, incomplete, exact, local/read-only; limitations and readiness assertions are not derived |
| “The command prints…” through “This expected baseline describes…” | general operational/no-credential/no-network/no-profile and deliberate-exclusion claims require code/provenance review; count/exit words and unavailable names here are now enforced by the dedicated baseline comparison |
| “The numerical coverage guard…” through “The inventory correction…” | universal descriptions of checker coverage and supersession are explanatory, not independently proven by that same checker |
| “Owner decisions only” list, “Routine preparation…” and closing paragraphs | none, full, new, remaining, no, independent and other policy/authorization/completeness assertions |

In particular, the baseline paragraph's “only expected unavailable items” is now
enforced as an exact identity set, not merely a count. Exit words zero/one/two/three
in its explanatory sentences are still prose: behavior is covered by process-exit
controls, not by parsing those words. Quoted product policy is not validated by
the digit guard. Fenced code, link destinations and known identifier exclusions
also remain outside general numeric coverage except for the dedicated baseline
fence parser.

Closing the gap requires a finite claim registry: give every measurable sentence
a stable claim ID, typed value/unit/scope, evidence binding and expected
availability; render those sentences from that data, and reject new unregistered
quantitative prose in the acceptance section. Treat status, causation, permission
and words such as “most”, “all”, “none” and “complete” as explicit reviewed claims,
not automatic numeric facts. Freeze reviewed prose or require review when it
changes. Add insertion and paraphrase controls, not just literal replacements.

Judgement: worthwhile if this becomes a maintained acceptance-report generator;
not worthwhile to build an unrestricted English quantity parser for this single
bounded QA correction. A word-number dictionary would catch simple additions
but miss paraphrases and semantic quantifiers while falsely flagging references.
The precise residual remains disclosed for owner evaluation; this worker does
not waive it or claim full natural-language reconciliation.
