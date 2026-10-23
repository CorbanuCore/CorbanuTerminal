# PF-83: why the firewall cannot move the preflight, and what can

**Fable, 2026-09-15.** Written after increment-07, which is the first preflight
to complete against the sealed guest.

## The run

Increment-07 finished in **9.76 seconds**, no timeout, deadlines unchanged. The
resolver fix worked: the previous run died on an SSH boundary read because every
name lookup cost 30 seconds.

Result: **2 reported_evidence_valid, 173 UNPROVEN**, `functional_verdict:
not_reached`. Native dispatch stays disabled and no case ran.

What actually changed against increment-05, the last run with the network open,
for both `bridge` and `bridge_child`:

| Control | Before | After |
| --- | --- | --- |
| `dns` | successful resolution | fast resolution failure, errno 8 |
| `internet_http` | **HTTP 200** | resolution failure, errno 8 |
| `lan_tcp` | connected | ~1 second timeout |

The isolation is working and observable. The controls still read UNPROVEN.

## Why they will never turn green from firewall work

The validator's criterion for a denial control is exact:

```
row["return_value"] == -1
and row["errno"] in (errno.EPERM, errno.EACCES, errno.EROFS)
and row["before_rc"] == 0 and row["after_rc"] == 0
```

`EPERM`, `EACCES`, `EROFS`. Those are **operating-system permission denials**. A
packet filter produces `ETIMEDOUT`, `ENETUNREACH` or, once the resolver refuses,
`EAI_NONAME` — errno 8. It cannot produce `EPERM`, so no amount of `pf`
configuration will ever satisfy this validator. I spent a round tuning denial
latency before reading this criterion; the tuning was necessary to get a
completed run at all, but it was never going to move the score.

The harness says the same thing about itself, in `macos_preflight.py`:

```
enforcement="partial account/VM separation; no per-run filesystem/process/IPC/network sandbox"
```

and `boundary-policy.txt` lists the native prerequisite as *"disposable macOS VM
or dedicated synthetic account **plus OS enforcement**"*. We have the VM and the
dedicated account. **The OS-enforcement half does not exist.** The guest
processes run as an ordinary user with ordinary permissions, so every denial
control is UNPROVEN by construction, and would have been even with the network
wide open.

## What this means for the two goals

These are different goals and only one of them needed the firewall.

- **"The executor must not cheat by reading Corbanu Terminal source."**
  Achieved and verified. `github.com`, `raw.githubusercontent.com` and
  `codeload.github.com` are all denied instantly from the guest, while
  `chatgpt.com` is reachable. That property is real and does not depend on the
  preflight score.
- **"The 175 controls must be proven."** Not achievable by network isolation.
  It needs a per-run sandbox profile that denies filesystem, process, IPC and
  network operations at the OS layer so the probes return `EPERM`/`EACCES`.

## The next unit

Give each run a seatbelt profile and launch the bridge, its child and the
target under it, so that a denied operation returns a qualifying errno instead
of hanging or resolving nothing. The profile is the missing prerequisite, not a
harness convenience.

One caution for whoever takes it: the **target** is the product whose own
permission behaviour PF-83 exists to qualify. Wrapping the target in a second
sandbox changes what is being measured. The profile belongs on the executor and
bridge side; for the target, the qualification has to distinguish the harness's
enforcement from the product's own.
