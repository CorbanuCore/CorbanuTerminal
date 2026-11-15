# PF-23-S04 and PF-23-S05: independent functional run (func-pf23, 2026-11-14)

Code-blind execution of the two frozen designs in disposable Tart guests. Results per sprint:
[PF-23-S04](../PF-23-S04/functional-20261114.md) · [PF-23-S05](../PF-23-S05/functional-20261114.md).
This folder holds what the executors were given and what they reported.

## Build under test

- Candidate: main `1dea8b3b0304f844ef8785295e0c051c0d17586e` (postmerge-ci run 43198497609 green), dev build with
  `developer-accounting`, Rust 1.99.1, signed `--options runtime --timestamp --identifier com.corbanu.corbanu`
  (Developer ID K5N2T25C23). SHA-256 `546a2440b9f9d593e57168bef99e9d9989ac1ab6f581f1c725a2a0777bd9344b`. The same
  binary was installed as the coordinator build (INSTALL.json, 2026-11-14 10:56Z).
- Reference build (S04 case 10, S05 case 18 and the S05 replay comparison): main `152f62bf6e` (the compatibility
  reference named by the S04 disposition; it predates #1837 and #2133), built and signed the same way, SHA-256
  `def14ee64ae863c361e12f7b1dc3c8a920f5b6a7b29e967a5bc31b4966d52c7f`. Never installed.

## Method

- Frozen cases, unchanged: `qa/security-levels/sprints/PF-23-S04/codeblind-design.md` (SHA-256 `c43781a1…63d5`) and
  `qa/security-levels/sprints/PF-23-S05/codeblind-design.md` (SHA-256 `a606c7fc…68b4`), copied byte for byte into
  the packet ([design-cases.md](packet/design-cases.md)).
- Executors: GLM 5.3 Flash on Z.AI (`corbanu exec`, one per group), code-blind: they had the packet, the read-only
  candidate package and the fixtures; no source, history or earlier findings. Each drove the real TUI through
  `agms-tui` (tmux; every key and screen logged). The product's own model was GLM 5.3 Flash; where it declined a
  step, the local model (Travis's mlx-serve, Qwen3.8-Flash-Next-Uncensored, through the host gate) replayed it
  (MODEL-DECLINE rule).
- Isolation: one fresh APFS clone of `corbanu-qa-base` (macOS 26.6.2, SIP on) per clone run, one VM at a time under
  `vm-tart.lock`, sealed with `scripts/qa/tart/seal-guest.sh` (pf: host SSH, DHCP and the loopback mediator only),
  the milestone harness's isolation probes (99 checks) and a smoke turn passed before every clone's groups. Groups of
  one clone ran concurrently on separate unprivileged lane accounts. No real key entered a guest: the host relay
  added the `zai_key_3` key; every clone was deleted afterwards. Synthetic fixtures only.
- S04 clones additionally let the lane accounts run `sudo -n /usr/sbin/chroot` and the fixture helper (chroot needs
  root on macOS); that makes a lane root-equivalent inside that disposable guest. S05 clones had no sudo.
- Harness: the qa-local3 Tart harness (`.codex-work/qa-local3-20261022/h`) with these changes: Z.AI relay label
  `zai_key_3`; `agms-tui start --baseline` and `--cwd`; fixture helpers `pf23-home` and `pf23-root`; an open profile
  variant (`secretless_agent_launch` and `secret_output_gate` off) for full-access legs; budget 165 minutes.

## Packet and amendments

[navigation.md](packet/navigation.md) plus seven amendments, each written before the groups it governs, none changing
a case or an expected result:

| Amendment | Why | Applies to |
| --- | --- | --- |
| [1](packet/amendment-1.md) | Attempt 1 (11:22-11:55Z) skipped the post-taint step the S04 disposition requires, asked the agent to request approval (a sandbox-exception prompt, not the product's decision), and its chroot fixture folders were unreadable, which stopped Aggressive at startup (harness defect, fixed). | S04 (item 2: all) |
| [2](packet/amendment-2.md) | Attempt 2 (12:06-12:34Z) compared different inner programs and its full-access legs were refused by protected launch. | S04 |
| [3](packet/amendment-3.md) | Coordinator check on the gate account: Moderate keeps commands in its sandbox even with full access, so `sudo` (and any chroot) can run only under Permissive full access. | S04 |
| [4](packet/amendment-4.md) | Local-model replay of S04 steps the hosted model declined. | G4R |
| [5](packet/amendment-5.md) | Local-model replay of S05 steps the hosted model declined, plus the same forms on two more fixture targets. | G5R, G5S |
| [6](packet/amendment-6.md) | Direct comparisons no group had recorded: a credential file directly and through `chroot /`; resume on the same and on another route (the latter turned out to open a new conversation). | G6X |
| [7](packet/amendment-7.md) | The `.env` comparison on the reference build. | G6Y |

Attempts 1 and 2 are kept as failed attempts: their reports are in [reports/attempt1](reports/attempt1) and
[reports/attempt2](reports/attempt2); attempt 3 and the replays are authoritative.

## Groups (authoritative)

| Group | Sprint | Cases | Level | Clone | Executor |
| --- | --- | --- | --- | --- | --- |
| G4A | S04 | 1-5 | Moderate | K1e | 12:50-13:23Z |
| G4B | S04 | 1-5 | Aggressive | K1e | 12:50-13:30Z |
| G4E | S04 | 10, 11 | Permissive (10), all (11) | K1e | 12:50-13:22Z |
| G4C | S04 | 6-9 | Moderate | K2e | 13:35-13:58Z |
| G4D | S04 | 6-9 | Aggressive | K2e | 13:35-14:03Z |
| G5A | S05 | 1-5 | Moderate | K3 | 14:08-14:36Z |
| G5B | S05 | 6-10 | Moderate | K3 | 14:08-14:29Z |
| G5E | S05 | 11-14 | Moderate | K3 | 14:09-14:48Z |
| G5C | S05 | 1-5 | Aggressive | K4 | 14:52-15:13Z |
| G5D | S05 | 6-10 | Aggressive | K4 | 14:52-15:12Z |
| G5F | S05 | 15-19 | Moderate; Permissive (18) | K4 | 14:52-15:18Z |
| G4R | S04 replay | 1, 10 | Moderate, Permissive | K5 | 15:26-15:45Z |
| G5R, G5S | S05 replay | 1, 4, 12, 15, 16 | Moderate, Aggressive | K6 | 15:49-16:32Z |
| G6X | follow-up | S04 1/5, S05 9 | Moderate | K7 | 16:41-16:56Z |
| G6Y | follow-up | `.env` on the reference build | Moderate | K8 | 17:01-17:09Z |

Executor reports: [reports/](reports). Raw evidence (actions, screens, PTY logs, session records, mediator logs) stays
on the Mac under `.codex-work/func-pf23-20261114/runs/`.

## Issues filed

#2165 (rules and `sudo chroot /`), #2166 (`resume --last` after a provider switch silently starts a new
conversation; first filed as a lost untrusted-content state and corrected after review), #2167
(project `.env` read without asking after untrusted content; predates both sprints).

## Limits

- macOS only. With SIP on, macOS kills Apple's own programs inside any chroot (AMFI: "hardened runtime not allowed in
  chroot") and nothing can start inside a root other than `/` (no usable system loader there), so S04 effects inside
  other roots cannot be observed on macOS. No Linux or Windows guest was available; using Docker on glitch would
  touch the shared Ambient service, so it was not used.
- Under Moderate and Aggressive the product's sandbox stops `sudo` itself, so in protected levels the S04 effect legs
  are "not exercised (sandbox stops sudo)"; the decisions before the run are the observation.
- Executors are a model: they finished in 20-40 of 150 minutes, mis-keyed once (G4D) and interpreted some cases
  differently between levels (S04 case 5). Their verdicts are reported as given; the sprint records add the
  integrator's evidence review beside them.
