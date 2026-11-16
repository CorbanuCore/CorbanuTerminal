# PF-23-S04 on Linux: independent functional run (func-d554, 2026-11-16)

Owner decision D554 (Travis, 2026-11-16): run PF-23-S04 case 11 on a Linux guest and observe the effects of cases
1, 4, 7 and 8 that macOS could not show. Code-blind execution of the frozen
[design](../PF-23-S04/codeblind-design.md) in the Tart Ubuntu 24.04 worker VM. Result:
[PF-23-S04/functional-20261116-linux.md](../PF-23-S04/functional-20261116-linux.md). The macOS run is
[PF-23-functional-20261114](../PF-23-functional-20261114/README.md).

## Build under test

- Candidate: main `51bd768363065a0543e75e298be54b42dcf20972` (contains #1837 and #2396), built **inside the guest**
  (Ubuntu 24.04.5 LTS, aarch64, Linux 7.0.0-43-generic, 8 vCPUs, 12 GiB) with Rust 1.99.1:
  `cargo build --locked -p codex-cli --bin corbanu` (debug, `-j5` to `-j7` under a systemd memory cap). SHA-256
  `1e335a4d0ec5da013d886a0b2a29d81b6bc83fffd83572cb0cb98de2103375cd`. A glitch build was not usable: glitch is
  x86_64 and the guest is aarch64.
- Reference build (case 10): main `152f62bf6e` (the disposition's compatibility reference), built the same way in
  the same guest, SHA-256 `e530cfb1462054919e83d60d2dd79210e663e2e937a1cfb37b4be4b203dd2ef8`.

## Method

- Frozen case file copied byte for byte (SHA-256 `c43781a1…63d5`, [design-cases.md](packet/design-cases.md)).
- Executors: GLM 5.3 Flash on Z.AI (`corbanu exec`, one per group), code-blind: they had the packet, the read-only
  candidate and the fixtures, no source, history or earlier findings. Each drove the real TUI through `agms-tui`
  (tmux; every key and screen logged). The product's model was GLM 5.3 Flash; where it declined a step, the executor
  replayed that step with the local model (mlx-serve, MODEL-DECLINE rule): once, in L4 (case 7, the `sudo` form).
- Isolation (Linux port of the macOS harness): five unprivileged lane accounts in one guest, one group per lane, run
  concurrently; the coordinator's home (source and build) is unreadable to them; iptables lets lanes reach only the
  loopback mediator, which forwards allowlisted Z.AI inference paths over an SSH reverse tunnel to a host relay that
  adds the `zai_key_3` key (no real key entered the guest). 18 isolation probes and a smoke turn passed before every
  batch, and every group re-ran the probes itself. Synthetic fixtures only. Lanes may run `sudo -n chroot` and the
  fixture helper as root (as on macOS), which makes them root-equivalent inside the disposable guest.
- Fixtures: as on macOS, but `rootA` / `rootB` are working roots (static BusyBox plus static pf23 tools), so effects
  inside a root other than `/` can be seen on Linux.
- Machine facts the coordinator checked first on a separate gate account ([machine-facts.md](packet/machine-facts.md)):
  the product's Linux sandbox sets "no new privileges", so `sudo` fails inside it at every level, and plain `chroot`
  lacks the privilege. `unshare -r chroot` (a private user namespace) runs in Permissive's default sandbox; under
  Moderate it runs only after the product's separate one-command user-namespace approval; under Aggressive nothing
  can start a chroot. So Linux effect legs are observable under Moderate (`unshare -r chroot` form) and Permissive,
  not under Aggressive. One row of that table was not checked and was wrong: in Permissive **full access**
  `unshare -r` fails (`write failed /proc/self/uid_map: Operation not permitted`, Ubuntu's restriction on
  unprivileged user namespaces outside the sandbox); there `sudo -n chroot` is the form that runs (L5).
- After review of the first batch, [amendment-1.md](packet/amendment-1.md) (before the second batch) fixed the
  approvals-off misreading and said how to write case 5's rules (for the direct commands only, then the chroot forms).

## Groups

| Group | Cases | Level | Executor (UTC) | Status |
| --- | --- | --- | --- | --- |
| L1 | 1-5 | Moderate | 21:27-21:45 | Authoritative ([report](reports/L1.md); before amendment 1, case 5 with chroot-form rules) |
| L2 | 1-5 | Aggressive | 22:25-22:43 | Authoritative ([report](reports/L2.md)) |
| L3 | 6-9 | Moderate | 22:25-22:58 | Authoritative ([report](reports/L3.md)) |
| L4 | 6-9 | Aggressive | 22:26-23:03 | Authoritative for cases 6, 7 and case 8's missing-root leg ([report](reports/L4.md)); stopped by the VM memory floor |
| L5 | 10, 11 | Permissive | 22:26-23:03 | Authoritative for case 10 ([report](reports/L5.md)); stopped before case 11 |
| L6 | 5 | Moderate | 22:26-22:40 | Authoritative ([report](reports/L6.md)): case 5 with direct-command rules (amendment 1 item 5) |
| L7 | 8, 9 | Aggressive | 23:13-23:39 | Authoritative ([report](reports/L7.md)); replaces L4's unfinished cases |
| L8 | 11 | all three | 23:13-23:28 | Authoritative ([report](reports/L8.md)); replaces L5's unfinished case 11 |

Failed attempts, kept rather than hidden:

1. 21:23Z: void before any case (the executors waited on stdin).
2. 21:27Z batch: L2-L5 started the product with approvals off (`approval_policy="never"`) because the navigation
   listed "Approvals off" as an available setting and four executors read it as required; with it the product refuses
   where it would ask, so approve / refuse / cancel legs could not be exercised. Their reports are in
   [reports/attempt1](reports/attempt1), prompts in [packet/prompts/attempt1](packet/prompts/attempt1). At 21:45Z
   the Z.AI account ran out of balance (HTTP 429, code 1113) and every group stopped; L1 had finished. The run
   resumed at 22:25Z after the account was recharged.
3. ~23:03Z: the VM wrapper stopped the guest because host free memory fell to 19% (worker floor 20%); L4 and L5 had
   written their reports up to that point, which were collected after a restart, and L7/L8 ran the rest.

## Incidents in the worker VM (not product behaviour)

- The prepared guest's Rust toolchain files were zero-length (data lost at an earlier hard stop); the toolchain was
  reinstalled. The VM wrapper stopped the guest at the 20% host-memory floor three times in all (twice during the
  build, once during the run); the first left 727 zero-length build artifacts, so that target directory was set aside
  and the build restarted under a systemd memory cap with a `sync` loop.

## Limits

- Linux aarch64 (Ubuntu 24.04) only. Windows: no guest; blocked.
- Executors are a model: they mis-graded some results (L2 graded its cases "failed" because the TUI shows a shell
  `cat` as "Read …"; L3 graded case 7 "failed" for a prompt the direct command also gets). The sprint record gives
  the integrator's evidence review beside every executor verdict.
- Raw evidence (actions, screens, PTY logs, session records, mediator and relay logs) stays on the Mac under
  `.codex-work/func-d554-20261116/runs/`.
