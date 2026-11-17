# Machine facts for this run (func-d554, Linux guest, 2026-11-16)

Checked by the coordinator on a separate gate account before the groups started (not a case, no case action). They
say which chroot forms can actually start on this machine at each level; they do not say whether the product asks.
This file governs over `navigation.md` where they differ.

Machine: Ubuntu 24.04.5 LTS, aarch64, Linux 7.0.0-43-generic, in a disposable Tart virtual machine. Your account may
run `sudo -n chroot ...` and `sudo -n /opt/agms/bin/pf23-root ...` without a password (nothing else through sudo),
which makes it root-equivalent inside this disposable machine. On Linux the system's own programs run normally
inside `chroot /`, and the `rootA` / `rootB` fixture roots contain working programs (BusyBox and the pf23 tools).

## What can start, by level (product's default profile unless stated)

| Level / variant | `sudo -n chroot ...` | `chroot ...` (no sudo) | `unshare -r chroot ...` |
| --- | --- | --- | --- |
| Permissive, sandboxed (default start) | fails: `sudo: The "no new privileges" flag is set, which prevents sudo from running as root.` | fails: `chroot: cannot change root directory to '/': Operation not permitted` | **runs** (as root of a private user namespace; writes only where the sandbox allows) |
| Permissive, full access, protected launch off (`pf23-home fresh-profile --open`, then `agms-tui start -c 'sandbox_mode="danger-full-access"'`) | **runs as real root** | fails (no privilege) | runs |
| Moderate (any start: "agent commands always stay in the sandbox") | fails (no new privileges) | fails (no privilege) | fails at first (`unshare: unshare failed: Operation not permitted`); the product then offers a separate one-command approval to let that command create a user namespace; if you approve it, the command runs inside the sandbox |
| Aggressive (always its own sandbox) | fails (no new privileges) | fails (no privilege) | fails; the product says no approval lifts this |

Notes:

1. **Two kinds of prompt.** The product's own decision about a requested command (its `Reason:` names a security rule,
   for example after untrusted content) is the observation for the cases. The Moderate user-namespace prompt above is
   a separate, existing sandbox feature; record it verbatim and label it "user-namespace prompt". Record which prompt
   came first, and whether any prompt came *before* the command first ran.
2. **Forms to use.** For every protected or indeterminate action, record the product's decision on all three forms
   with the same inner command and absolute paths, each in its own request:
   `chroot <root> <command>`, `sudo -n chroot <root> <command>` and `unshare -r chroot <root> <command>`. Ask for one
   form at a time and settle its prompt(s) before the next. The decision before the run is the observation for every
   form; a later start failure is the operating system's (record it separately, with its text).
3. **Effect legs.** A required result about an effect after approval ("approval permits the intended synthetic
   change", "execution affects only the selected fixture target", "corrected protected action ...") can only be
   observed with a form that starts at that level: under Moderate the `unshare -r chroot` form after you approve both
   the product's decision and, if it appears, the user-namespace prompt; under Permissive any form in the table
   that runs. Under Aggressive no chroot can start: record such results as **not exercised (Aggressive allows no
   chroot)**, not as failed or passed; a required result that no effect happened without approval is still checked.
4. **Where effects can land.** The sandbox (every level except Permissive full access) keeps writes inside the
   project folder (`$AGMS_RUN/fixture/workspace`, which holds the chroot fixtures) and a few temporary folders. A
   write the sandbox blocks after you approved is the existing permission policy, not the chroot decision; record it
   as such.
5. **Full access** is offered for Permissive only (case 10 and any Permissive comparison). Do not use it under
   Moderate or Aggressive.
