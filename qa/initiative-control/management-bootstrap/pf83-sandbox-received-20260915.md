# PF-83 per-run OS enforcement — implemented, reviewed, received

**Fable, 2026-09-15.** Candidate: harness increment-17 at
`/Volumes/CorbanuDrive/Corbanu/.codex-work/functional-pf83.20260915`. This work
lives outside the repository, so "received" here means the manager accepted the
evidence and recorded it, not a branch merge.

## Result

**32 of 175 controls valid, up from 2.** Positive controls unchanged at 2 of 13.
648 changed lines across `macos_bridge.sbpl`, `macos_boundary.py`,
`checks_macos_boundary.py`, `macos_preflight.py`, `macos_adapter.py` and
`NATIVE-MACOS.md`. All 14 offline checks pass. Exit 1, which is the honest
blocked-preflight exit, and `native_dispatch` stays false: **no case has run.**

## Independent review

`claude-opus-5-plan` high, adversarial, **no P1 or P2 findings**. What it
actually checked, rather than took on trust:

- Traced eight newly valid controls individually to genuine kernel refusals:
  reads, `listdir`, symlink escape, `kill`, pty open, FIFO, and unix and TCP
  `connect`. Dead-endpoint and plain-permission alternatives are excluded by
  errno class, so a denial cannot be confused with a target that would have
  failed anyway.
- Confirmed `getaddrinfo` failures stay in the **EAI** domain with `errno=None`,
  so a resolver failure can never be laundered into `EPERM`. That was the
  specific forgery this design was most exposed to.
- Rehashed all **269 sealed files across increments 08 to 17**: zero mismatches,
  zero unsealed extras.
- Confirmed `native_guest.py`, `cases.json` and `boundary-policy.txt` are
  byte-identical to the pre-change state, so the frozen validator is untouched.
- Confirmed target and target_child inherit nothing: `serve()` iterates only
  `("bridge","bridge_child")` and all 108 target and actor rows carry the
  no-claim string.
- Confirmed the credential at `/Users/agent/.pf83-auth/auth.json` is referenced
  only in prose. No code reads, copies, logs or embeds it.

## The witness pushback, and what honesty cost

The manager rejected three rows the design had marked unprovable because *our
own firewall* blocked the paired witness. The implementation built all three:
an owned loopback UDP responder for `dns` and a loopback 204 responder for
`internet_http`. Both were then marked `supporting_only` by the worker and are
**not credited**, because a witness bound inside the same perimeter does not
independently prove egress denial. That is the right call and it cost two
controls.

`lan_tcp` **is** credited, twice, on a witness that really connected to
`192.168.64.3:49303`. The review's first P3 is that this row carries no
`supporting_only` or scope marker even though it targets the guest's own
interface, so a reader of "32 of 175" can over-read it as proof of LAN egress
denial. Recorded here so the headline number is not read for more than it says.

## Disclosed and open, none blocking

1. `lan_tcp` lacks a scope marker on a self-interface witness.
2. Confined processes report their own errno; `sandbox-log` returned exit 77, so
   there is no kernel-side witness. Mitigated by external before and after
   probes, not eliminated.
3. `paired_row` does not require witness success, so `package_write` got a
   populated bound row on a failed witness.
4. The candidate execs under the profile for `package_version`; the scope string
   should be tightened.
5. Three positives lost specific limitation text for a generic placeholder.
6. A test creates a temporary directory under the completed increment-08.
7. Increments 08 to 17 are mode 0700 where 06 and 07 are 0500; sealing is
   hash-based and verified, so integrity holds.

## What this does not establish

Not a case result, not a functional gate, not human or release readiness. 143
controls remain UNPROVEN, most for the reasons the design named: Mach and
OSStatus error domains that are not POSIX `-1`/errno, missing fixtures, and
controls needing dispatcher enforcement. The target-containment decision is with
Travis and nothing here pre-empts it.
