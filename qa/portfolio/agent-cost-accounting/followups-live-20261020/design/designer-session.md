# Designer session (access record)

| Field | Value |
| --- | --- |
| Session (thread) | `01a156fc-532e-7ee0-a6c0-0096f4ca3b19` |
| Runtime | installed `corbanu exec` 0.1.48 (`~/.local/bin/corbanu` launcher; signed developer build 78dd811d6e, SHA-256 `e7200306…25bdaa3`) |
| Model | `claude-opus-5-5-plan`, provider `claude-plan`, reasoning effort high |
| Flags | `--json --skip-git-repo-check -s read-only -C <packet dir> -o designer-output.md`, prompt on stdin |
| Context | new process, no inherited conversation; prompt = [prompt.md](prompt.md) (the repository's `qa/code-blind-functional/designer-prompt.md` verbatim, plus where the packet is, the `FUP-` prefix, feasibility and grouping, and the output shape) |
| Working directory | a directory holding only the packet (`.codex-work/acc-tart1-20261020/design/packet`), outside any source checkout (not a git repository) |
| Time | 2026-10-20 01:45:39Z to 01:49:57Z, exit 0 (a first launch at 01:42:48Z died with its tool shell before its first event and is not counted) |
| Tokens | 305,180 input (265,752 cached), 27,923 output |

## Packet

- [packet/intent.md](packet/intent.md): written by the integrator from the accounting sprint records (S06–S09 mandates,
  acceptance criteria, owner decisions), the owner's decisions for this run (D25, D3, D16, D34) and the test
  environment the harness provides. No code, tests, implementation notes or results.
- [packet/screens/](packet/screens/): 13 text screens: eight from the S06–S08 sprints' captures (as in PF-60-S10's
  packet), three from the #898 fix's own tmux run (paths shortened), and one realtime `/cost` render from a test
  against a stand-in (labelled as such).

## Isolation: instruction-only

The read-only sandbox stopped writes but not reads; no OS-level read restriction was applied (as in the earlier
milestone and PF-60-S10 designs). Recorded as `"isolation": "instruction-only"`.

## What it actually accessed

[designer-events.jsonl.gz](designer-events.jsonl.gz) holds the full event stream. Two commands, both inside the packet
directory: `cat intent.md; ls -la screens; cat screens/README.md`, then `cat` of every screen. No other path,
network or tool.

## Output

[designer-output.md](designer-output.md), unedited: 18 questions and 24 cases (FUP-01..FUP-24) in four groups (Z, O, M,
R) plus one case for all groups.
