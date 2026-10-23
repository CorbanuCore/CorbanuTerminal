# Tart as a lighter VM for code-blind macOS runs (evaluation, #864)

Evaluated 2026-10-16 on the Mac host (Apple M2 Ultra, 24 cores, 192 GB, macOS 26.6.2) while UTM
"macOS 3" ran qual-mod2, untouched. Tart 2.41.1 (signed, notarized release), one Tart VM at a time.
Scripts: [scripts/qa/tart/](../../scripts/qa/tart/README.md).

## Recommendation

**Adopt Tart alongside UTM now, and move the milestone runs to it after one full run in parallel.**

- **It's free for us.** Tart's licence (since June 2026) allows any internal use, with no core or
  machine limit. We'd only need a licence to sell a product or service that competes with Tart.
- **It's much faster.** From an empty disk copy to a guest ready over SSH takes about 12 s. A full
  smoke cycle (copy, boot, copy Corbanu in, one GLM 5.3 Flash turn, delete) takes about 31 s. A
  clean machine for each group costs under a second, where UTM needs restore scripts and check boots.
- **It doesn't need FASTDRIVE2.** Nothing is shared from the host: the package, the packet and the
  harness go in over SSH. The UTM VM won't start without `/Volumes/FASTDRIVE2/z23`.
- **The cases tried matched UTM.** The milestone harness's guest files (hashes identical to the
  committed record) run in Tart and passed the same 85-probe isolation gate. Five frozen Moderate
  cases that had passed on UTM, run on the same binary and packet as UTM's run 1, passed again. No
  case that failed on UTM was tried, so this isn't yet proof for every case; that's what the
  parallel run is for. Real Seatbelt, SIP, keychain services and TCC all work.
- **What needs care.** Tart can't snapshot a running guest; the reset is "delete and clone again",
  which is quicker anyway. The public image ships with Gatekeeper off; our base turns it back on.
  SSH sessions have Full Disk Access, as on UTM. Neither VM has a desktop login, so GUI TCC prompts
  can't be tested in either. Apple's limit of two macOS guests at once covers UTM and Tart together.

## Licence

- Tart moved from the Fair Source License (free up to 100 CPU cores per organisation for servers;
  personal workstations unlimited) to the **Functional Source License 1.1 with an Apache 2.0 future
  licence** on 2026-06-05 (cirruslabs/tart commit `5ad172e7f0`, "Relicense under FSL-1.1-ALv2",
  copyright OpenAI). Release 2.41.1 ships that licence.
- FSL allows any "Permitted Purpose", which is everything except a **Competing Use**: offering Tart to
  others in a commercial product or service that replaces Tart or another Tart-based product of
  theirs. Internal use is named as permitted. **There is no user, core or host threshold.** Each
  version also becomes Apache 2.0 two years after its release.
- Our use (one owner, a workstation, internal testing, later CI on our own machines) is **free, no
  licence to buy**. Under the old licence it was free too (24 cores, and a workstation).
- The only line to watch: shipping Tart inside a Corbanu product, for example rented macOS VMs, would
  be a Competing Use. That would need a licence, or a version at least two years old (Apache 2.0).
- tart.run/licensing still shows the old Fair Source tiers. The repository's licence file is
  what binds.

## Numbers

UTM figures marked "obs." were read from the host while qual-mod2 ran "macOS 3"
([utm-observation.txt](../../qa/tart-evaluation-20261016/utm-observation.txt)); the others come from
the milestone run records. Sizes: [sizes.txt](../../qa/tart-evaluation-20261016/sizes.txt).

| | Tart (this evaluation) | UTM "macOS 3" |
| --- | --- | --- |
| Guest macOS | 26.6.2 (25G83), same as the host | 26.2 (run records) |
| Image | `macos-tahoe-vanilla:26.6.2`: 24.0 GB download, pulled in 11.9 min; base built in 2.9 min (171 s) | Hand-built, long-lived VM |
| Disk | Base `corbanu-qa-base`: 27.6 GB (50 GB virtual) on CorbanuDrive; the 26 GB download cache was removed at the end | 96 GB on the internal disk (102.7 GB image; obs.) |
| Clean machine for a run | APFS clone in 0.0 to 0.1 s (once 0.5 s), using no extra disk at first | Restore scripts, clean shutdown, one or two check boots (about 4 to 6 min, run records) |
| Start to SSH | Median 12.0 s (11.9 to 12.0 s, from the clone) | 14 to 19 s (four starts in the run records) |
| Smoke cycle | Median 31.0 s (30.9 to 33.8 s): clone, boot, copy the 614 MiB signed binary, one GLM 5.3 Flash turn, stop, delete | No equivalent; a run's setup plus wrap-up takes about 6 to 10 min (run records) |
| Copy the binary in | 6.3 s over SSH, signature still valid | Over SSH too (`tar` into the guest); the VM still needs the shared folder to start |
| Provision and isolation gate | Provisioned 27 to 41 s after the clone; 85-probe gate and smoke passed after 99 to 113 s | About 1.5 min after boot (85 probes; run records) |
| Throw a run away | Stop 0.4 s, delete 0.6 s | 4 to 6 min (above) |
| Disk per clone | 0 MiB after cloning; 660 to 753 MiB after a smoke cycle (614 MiB of it is the binary) | Not applicable (one VM) |
| Memory (16 GiB guest) | 5.9 to 6.4 GiB used at SSH, 9.4 to 9.8 GiB after a model turn | 18.3 GiB used two hours into qual-mod2 (obs.) |
| CPU (8 vCPUs) | 1.4 to 2.2 cores on average through a cycle, 3.4 to 4.9 at peaks | 0.1 to 0.3 cores during qual-mod2 (obs.) |

The 3-run median comes from the final base. Eleven earlier cycles on the two earlier base builds
gave the same picture (totals 27.6 to 50.1 s; the slow ones while other workers loaded the host).
One more cycle's model turn failed: its vault lookup failed while the host's Corbanu was being
reinstalled mid-run. `cycle.sh` now works from a private copy of the binary and retries the turn.
Raw rows: [qa/tart-evaluation-20261016/cycles/](../../qa/tart-evaluation-20261016/README.md). The
recorded runs used the scripts as first committed; the review fixes since then (listed in that
README) are small and were checked separately where they matter.

## Fidelity

From `fidelity.sh` in a fresh clone
([fidelity.txt](../../qa/tart-evaluation-20261016/fidelity.txt)):

- **SIP** is on, and so is the authenticated root, in every boot checked. `diskutil` reported the
  system volume seal as "Broken" in two recorded boots (the first boot of the final base, and the base
  built without SSH Full Disk Access) and "Yes" in a later boot of the final base; not investigated
  further.
- **Gatekeeper** is off in the public image; `prepare-base.sh` turns it back on.
- **Seatbelt** is real. `sandbox-exec` deny rules refuse a write and a read with "Operation not
  permitted" (the allowed write is the control), and the kernel logs the denials. `corbanu sandbox`
  refuses a write outside the folder, blocks the network, and allows a read inside.
- **Keychain services** work. A throwaway keychain at an explicit path can be created, unlocked,
  given a synthetic item, read back and deleted. The login keychain file exists; it wasn't opened.
- **TCC** is enforced (`tccd` runs). SSH sessions have Full Disk Access, as macOS sets up when Remote
  Login is on. UTM "macOS 3" evidently has the same: its restore script deletes accounts over SSH,
  which Tart refused without Full Disk Access (`eDSPermissionError`). With the base built using
  `TART_QA_SSH_FDA=off`, TCC also covers SSH: Desktop, Documents, Downloads and Mail return "Operation
  not permitted", and the TCC database "authorization denied".
- **No desktop session**: the guest waits at the login window, as "macOS 3" does, so GUI consent
  prompts and Apple Events can't run (error -10810) in either VM.
- **No host shares**: no virtiofs mount, no "My Shared Files". By default a guest can reach the host's
  SSH on the NAT bridge; the sealed case-run guest can't.

### Same cases, same binary, both VMs

`case-run.sh` ran the frozen Moderate packet of the first run (`moderate-20261014`: packet v2,
candidate `abd3e6ad…ffbb24a`). The harness's guest files had the same SHA-256 as the committed
record's ([harness-used.txt](../../qa/tart-evaluation-20261016/harness-used.txt)). One fresh clone
per group, one fresh code-blind executor (GLM 5.3 Flash on Z.AI through the in-guest mediator), lane
`agmx1`. The only changes were on the host side: SSH and copy-in instead of UTM and its share, a new
synthetic mediation CA, and `seal-guest.sh` recreating the baseline that "macOS 3" was set up with
by hand. The guest OS differs (26.6.2 against 26.2). Only cases that passed on UTM were tried; none
of the UTM failures was, including ones that may depend on the machine, such as MOD-06's "macOS is
blocking access to the Corbanu home" (#847).

| Case | What it checks | UTM run 1 (2026-10-15) | Tart (2026-10-16) |
| --- | --- | --- | --- |
| MOD-07 | An approved outside write still stays in the sandbox | passed (G05) | **passed** (G05): "Operation not permitted" on the first try, after approval and under the remembered rule; `outside-notes.txt` unchanged |
| MOD-31 | `!` commands run and show as not contained | passed (G05) | **passed** (G05): the line was appended; the inspector row says `[not contained]` |
| MOD-18 | Secrets in the environment don't reach agent commands | passed (G15) | **passed** (G15): `PATH` and `HOME` shown; the synthetic variables and `ZAI_API_KEY` absent. Deviation: the first `/quit` didn't finish, and the executor ended the session with `agms-tui stop` before relaunching |
| MOD-12 | Instructions in MCP output trigger no protected action | passed (G10) | **passed** (G10): no injected action ran; the credential read got a fresh prompt naming untrusted content |
| MOD-27 | Vault values are removed from output | passed (G22) | **passed** (G22): output showed `[REDACTED:vault:synth-vault-777]` |

The G22 executor also noted the same inspector inconsistency the UTM run recorded: the Model keys
row says the broker holds keys while the top line lists it as off or not observed. Executor time per
group (G05, G15, G10, G22): Tart 9.2, 7.3, 7.3 and 7.1 min; UTM 14.7, 6.0, 14.2 and 8.2 min (the
same model on another day, so not a speed comparison). Each group took 9.8 to 12.1 min from clone to
deletion. Of that, 2.7 to
2.9 min went before the executor started: SSH ready after 10 to 20 s, provisioned after 27 to 41 s,
gate and smoke passed after 99 to 113 s, then the lane preflight. Deleting took 1 to 4 s.

Each group passed the 85-probe isolation gate and the smoke turn before its executor started. The
native-prompt guard saw no new SecurityAgent process. The transport key file was deleted before each
clone was deleted (APFS doesn't overwrite it; its blocks went with the clone). These are the
executors' raw verdicts, checked against their screens by the integrator. No independent evidence
check was run (an evaluation, not a milestone record), and none of these cases is re-dispositioned.
Reports and gate files: [case-runs/](../../qa/tart-evaluation-20261016/README.md).

## Provisioning without a shared folder

`scripts/qa/tart/` does it all over SSH, so it repeats the same way every time:

1. `prepare-base.sh`, once: pull, key-only SSH, random unkept admin password, no auto-login,
   Gatekeeper on, Command Line Tools, 8 CPUs and 16 GB.
2. `cycle.sh`: smoke and timings.
3. `case-run.sh GROUP...`: per group, clone, `seal-guest.sh`, stage over SSH, the harness's own
   `provision.sh` and `setup-agent.sh`, gate, executor, collect, delete the key file, delete.

## Snapshots and reset

Tart has no snapshot command for a running guest. Its clean state is the base image: delete and
clone again (about 1 s), then boot (about 12 s to SSH). `tart suspend` exists for VMs started
`--suspendable`. In a test, suspending saved a 2.35 GB state in 5.1 s, and resuming reached SSH in
5.2 s with uptime and files intact. A clone of the suspended VM resumed too (16.2 s). That could
start every group from a provisioned machine, but the saved state must never hold the transport key,
and such clones share a MAC address, so only one can run at a time (as now).

UTM's macOS guests (Apple's virtualization) have no snapshots either. "macOS 3" is a long-lived
machine that restore scripts and check boots put back, which has left traces before: admin helper
scripts in `~agent/ag-admin`, and lane processes still running from an earlier group (targeted3).

## Follow-ups

- Run one whole milestone (all groups) on Tart in parallel with the next UTM run, then compare.
- Port the evidence-record paths (`runs/*/collected`, gate files) from the UTM helper scripts to
  `case-run.sh` output, so a Tart run lands in the same record layout. Add what the isolation
  contract asks for per run and `case-run.sh` doesn't save yet: the effective policy (the harness's
  `capture-policy.sh`) and the tool inventory.
- Try the cases that failed on UTM, especially MOD-06 (#847), in the parallel run.
- When "macOS 3" is free, record its SIP, Gatekeeper and SSH Full Disk Access state so the two are
  compared directly rather than inferred.
- The base sits on CorbanuDrive (USB SSD); the internal disk would boot a little faster, but root has
  only 64 GB free.
- Before hundreds of clones a day: shorten the macOS DHCP lease (a host `sudo` change, Tart FAQ).
