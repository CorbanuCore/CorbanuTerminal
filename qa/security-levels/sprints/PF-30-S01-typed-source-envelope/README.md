# PF-30-S01 checkpoint — not sprint completion

Product initiative, **Non-negotiable controls**: “Classify instruction intent
and provenance before external content can influence tools or financial actions.”
Plan: `docs/plans/active/p0-security-levels.md`, feature PF-30, sprint PF-30-S01.

## Current implementation boundary

- Validated descriptive protocol envelopes cannot represent human/system authority.
- Core-only admission binds an existing screening capability to the exact source
  envelope and normalized content digest; an Allow verdict releases untrusted data.
- Bounded context fragments use the user/data role, escaped delimiters and explicit
  Unicode controls. The JSON projection is capped at 8,192 bytes plus fixed wrapper text. This can exceed
  1,000 tokens and requires explicit reviewer attention under Core context policy;
  it remains below the hard 10,000-token per-item bound.
- A host-held exact-item sidecar observes native history append, tool dispatch and
  MCP origin refinement. Message-shaped sources are conservatively untrusted
  transcripts; child messages have child provenance. An output cannot register
  its own tool origin. Unsupported native variants remain absent and deny.
- The bounded normalized payload and exact source binding remain in a private
  pending carrier. A producer can read that immutable candidate and return real
  `ScreenedContent` through the native client handoff. Reading is not admission;
  a mismatch consumes the pending candidate and never restores raw data.
- Native Responses, Chat Completions and Anthropic request constructors project
  only exactly bound admitted items and reject raw/legacy/forged inputs. Initial
  session creation and model-provider replacement enforce the maximum of
  configured intent and live inherited policy. Unavailable policy fails closed.
  Memory and both realtime transport startups also fail closed. Running realtime
  tasks recheck after each dequeued input/event and before a sideband connection,
  and suppress transcript-tail re-entry after policy denial. This is per-operation
  admission, not retroactive revocation of frames already sent or immediate
  closure of an idle socket.
- Host authorization notices require the separately held human controller and
  an exact validated confirmation. They explicitly do not apply policy changes.
- Permissive request behavior is unchanged; no protected mode is activated.

## Explicit remaining work

Native producer observation and wire projection are connected; production
screening capability delivery and complete-input segmentation are not. Tool-based
file/web/plugin output retains conservative tool provenance, and message-based
hook/social/email content retains conservative transcript provenance rather than
claiming unobserved finer origin. Native hosted web/search/opaque variants without
a registered source carrier reject. Provider-safe projection fixtures alone do
not prove native successful protected inference. Persistent
resume/memory lineage remains PF-30-S02, post-taint enforcement PF-30-S03, and
qualified detector delivery PF-35. Unknown or unavailable routes remain closed.

Verification so far: all 22 focused Core provenance tests and all 285 protocol
tests pass on RTX. Full Core: 3,455 passes, five pre-existing request-permissions
failures independently reproduced at allocation `4f263ca73`, eight skips.
The existing actual-key TMUX `/status` and `/exit` smoke passed on `7b884e477`,
alongside locked CLI build, formatter check and plan/sprint governance.
The second remediation passed scoped fix/full formatting and 88 combined Core
provenance/realtime tests (22 provenance plus 66 other realtime tests), before
commit. Both independent reviews are recorded in [the disposition ledger](review-disposition.md);
the realtime fix awaits final TMUX and authorized review 3. No clean overall review,
human acceptance, release or benchmark pass is claimed.
Builds/tests run only on the allocated RTX host.
The pinned OpenClaw source checkout was lost; OC-4/OC-10 repository review records
were consulted, not fresh source execution. Adversarial fixtures here are original
Corbanu tests based on the recorded requirements, not copied implementation code.
