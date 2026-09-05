# Review scope: bounded complete-input segmentation

Review the continuation delta from `72f5de657` through the frozen candidate.
The tested source is `2a4fb5857`; subsequent changes, if any, are QA evidence only.
Do not review the entire integrated broker/UI branch as new provenance work.

Product mandate: content cannot assign its own authority or impersonate human
approval. The relevant specification heading is **Non-negotiable controls**:
“Classify instruction intent and provenance before external content can influence
tools or financial actions.” This remains a staged PF-30-S01 implementation, not
production screening qualification or completed native source coverage.

The production delta is confined to `core/src/security/ingress/mod.rs`: private
candidate segmentation, count exposure, empty-input refusal and matching the
complete ScreenedContent segment count at admission. Whole-input and context
projection bounds are unchanged; no history rewrite or final provider shaping
change is intended. Transferred client/session files were not edited.

Check exact full-input source/digest/count binding, atomic rejection, malformed
or substituted chunks, unknown source kinds, original-index reassembly, Unicode
escape boundaries and cache/input byte stability. Transport pieces are not
separately admitted. Screened Allow only releases untrusted data and never mints
authority. The tests use the existing synthetic screening fixture, not a real
production classifier or PF-35 qualification.

The existing projection can exceed 1,000 tokens (8,192 bytes plus fixed wrapper)
and therefore requires explicit manual attention under Core policy. Its bound
was not increased. Empty/oversized complete inputs reject; do not infer support
for arbitrary-length input or suggest a clipped-prefix fallback.

RTX final proof: scoped fix/full formatting, Core provenance 27/27, full
content-security 22/22, locked CLI build, full formatter check, actual-key TMUX
status/exit smoke 1/1, and plan/sprint governance passed. Earlier full-Core five
baseline failures are documented and are not an overall green suite.

The previously accepted unbound memory worker is owned by PF-30-S04 and is not
silently fixed here. Report any new concrete interaction with this delta, but
do not expand into that separate public-policy API or invent production Allow.
Read-only structured review, no code mutation, credentials or nested reviewers.
