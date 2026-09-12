# PF27 private descriptor-pair root dispatch

Work in progress; final exact-source qualification/reviews pending.
Allocation23d8e54a1 imported at launch00b07dad10b215e756696f8c5477055f1e1722d5;
timestamp correction imported1cfdce29d. Source/size base12e0bc2d8 unchanged.
Fifteen exact allocated paths, target740/hard800 including receipt/tests/runner.

Private dispatch binds the admitted role/generation and same pair-control identity
to its fixed root. The matched OwnedChild mints retained identity; no raw PID/fd
authority or mock identity enters PF20. At most two handlers; the existing worker
alone owns termination/reaping. Receipt drop fences both channels on every return
or panic; start/open failure cancels. Completion requires handler joins and owner
cleanup. Typed PF20 errors remain distinct, including Ambiguous.

The Linux/GNU synthetic-only bridge creates fresh private temporary roots with
fixed synthetic bindings. It accepts no existing path/root/key/UID and calls the
actual storage, transport and client; stale sequence injection reuses actual HMAC
framing. Existing fixed-system factories, public root.rs/Child and default builds
are unchanged. Caller retains fixture lifetime; returned roots are fixture data,
not native authority. No installation, protected activation, credentials or UI.

## Evidence and original attempts

Remote originals: `/home/travis/security-round5/evidence/pf27-root-dispatch-20260912`.
Initial compile101: test referred to a nonexistent IntegrityRootError::Ambiguous;
PF41 maps that outcome to Timeout. Lost-reply test now exercises policy's actual
RootError::Ambiguous, without changing runtime mappings. Repaired compilation0.
First actual six-case run:5pass/1fail; namespace assertion expected client Invalid,
but existing protocol closes without an error frame. Corrected client expectation
to Unavailable and asserts exact server Invalid; added reverse namespace case.
Distinct repaired run:all6pass. Original logs/exits remain; no hidden retries.
Quoted remote scp brace failed before transfer; explicit source arguments worked.

Cases: reversed role arrival, both namespace load/CAS/reload, duplicate socket
receives no handshake, both wrong namespaces, malformed and authenticated replay,
committed withheld policy reply remains Ambiguous and consumes client, actual
death/cancel/deadline/drop closes both, wrong parent peer and old receipt against
same-number replacement generation reject without a key, open/start/panic failure
cleanup. Real OS/protocol happy paths; only construction/panic faults use seams.

## Final proof and review

Pending final frozen-tree RTX TMUX, strict affected lint, Cargo/Bazel parity and
reviews35 Astra High/36 Fable5.1 High; preserve1–34. The committed runner retains
all25 predecessor commands and adds relay build/profile plus six dispatch cases.
No negative fixture or ignored case may be relabeled passing without execution.
Manager-accepted policy1.7 internal-only N/A applies only to this increment;
later protected-user/PF26 isolated functional/native/live-repository qualification,
human acceptance, benchmark and whole-sprint/release gates remain open.
