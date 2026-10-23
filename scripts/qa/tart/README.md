# Tart QA VMs (evaluation tooling, #864)

Scripts for disposable macOS guests on Tart (Apple Virtualization.framework). Every run gets a fresh
APFS clone of one base image, is provisioned over SSH (no shared folder), and is deleted afterwards.
Findings and the recommendation: [docs/qa/tartEvaluation.md](../../../docs/qa/tartEvaluation.md).

## Rules these scripts enforce

- One Tart VM at a time, and never a third macOS guest: `guard_slot` (in `lib.sh`) refuses to start
  if a Tart VM is already running or two Virtualization.framework guests (UTM's included, counted by
  their VM processes) are up. It never touches UTM.
- No host share is ever configured (`tart run` is called without `--dir`); the guest gets no
  clipboard or audio either.
- The model key is read from the Corbanu vault at use time and piped over SSH. `cycle.sh` passes it
  into one command's environment only (never written to disk); `case-run.sh` writes it to the guest
  coordinator's 0600 key file for the mediator and deletes that file before the clone is deleted.
- Keychain checks use a throwaway keychain file with an explicit path; no login keychain is opened.

## Setup

```sh
# Signed, notarized release (the Homebrew tap's formula is stale and fails on Homebrew 7):
curl -LO https://github.com/cirruslabs/tart/releases/download/2.41.1/tart.tar.gz   # check the published SHA-256
tar -xzf tart.tar.gz && export TART="$PWD/tart.app/Contents/MacOS/tart"
export TART_QA_WORK=/Volumes/CorbanuDrive/Corbanu/.codex-work/tart-eval    # keys, logs, stage
export TART_HOME=/Volumes/CorbanuDrive/Corbanu/.codex-work/tart-home       # images (about 55 GB)
scripts/qa/tart/prepare-base.sh        # once: pull the image, build the local base "corbanu-qa-base"
```

## Scripts

| Script | What it does |
| --- | --- |
| `prepare-base.sh [IMAGE]` | On failure deletes its prep VM. Pulls `macos-tahoe-vanilla:26.6.2` (default), pins its SSH host key, installs an SSH key, turns password SSH off, replaces the public default password with a random unkept one, turns auto-login off (login window, as on UTM "macOS 3"), turns Gatekeeper back on, installs the Command Line Tools, sets 8 CPUs / 16 GB, saves the base. |
| `cycle.sh [RUNS]` | Clone, boot, SSH, copy the signed `corbanu` in, one GLM 5.3 Flash turn on Z.AI ("Reply with exactly: SMOKE-OK"), stop, delete. Writes timings, VM memory/CPU and APFS clone cost to `$TART_QA_WORK/logs/cycle-results.jsonl`. |
| `fidelity.sh` | In a fresh clone: SIP, Gatekeeper, Seatbelt (`sandbox-exec` and `corbanu sandbox`), keychain services (throwaway keychain), TCC, host shares, network. |
| `case-run.sh GROUP...` | Frozen code-blind case groups, one fresh clone each: `seal-guest.sh`, then the UTM milestone harness's own guest files (`provision.sh`, `setup-agent.sh`, `agms-*`), isolation gate, executor, collect, key file deletion, clone deletion. Inputs: `HARNESS`, `PACKAGE`, `PACKET` (see the script header). |
| `seal-guest.sh` | Root, inside a fresh clone: the sealed-guest baseline the harness expects (coordinator account `agent`, stand-in homes, pinned `api.z.ai`, pf allowing only host SSH, DHCP and Z.AI HTTPS, a loopback DNS responder that refuses everything). |
| `apfs_private_size.py` | Bytes a clone holds on its own (`du` counts shared clone blocks). |

## Notes

- `tart stop` on these guests powers off at once (no guest agent in the vanilla image); fine for a
  clone that is deleted next. `prepare-base.sh` shuts the prep guest down from inside instead.
- Clones share the base's SSH host key; it is pinned once under the alias `tart-qa-base`.
- SSH sessions in the guest have Full Disk Access (macOS's default with Remote Login on, as on UTM
  "macOS 3"). Build the base with `TART_QA_SSH_FDA=off` to make TCC apply to SSH-started processes.
- macOS's DHCP server leases addresses for a day; hundreds of clones a day would need the shorter
  lease Tart's docs describe (a host `sudo` change, not made here).
