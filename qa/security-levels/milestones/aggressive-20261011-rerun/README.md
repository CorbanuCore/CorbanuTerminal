# Milestone "Aggressive ships": re-run (2026-10-12)

Code-blind functional re-run of the P1 security milestone after the defects of the first run
([../aggressive-20261010](../aggressive-20261010), #447: 17 passed, 15 failed, 7 blocked) and the owner decisions of
2026-10-11 were implemented. **Not passed: the milestone needs Travis's named sign-off.**

## Result

57 cases: **41 passed, 10 failed, 6 blocked** ([dispositions.md](dispositions.md), [results.json](results.json)).

- First run's 39 cases: 30 passed, 6 failed (AGG-01, AGG-18, AGG-24, AGG-25, AGG-27, AGG-32), 3 blocked (AGG-20,
  AGG-33, AGG-38). The per-case comparison with the first run is in [dispositions.md](dispositions.md).
- 18 new cases for the decisions: 11 passed, 4 failed (AGG-R07, R09, R13, R17), 3 blocked (AGG-R03; AGG-R14 child
  stop not observed; AGG-R06 not executed: the guest has no Claude Code).
- Defects filed: #620 (Grant once for a `~/.zshrc` write still fails: AGG-18, R17), #621 (home listing via
  `os.listdir` offers Grant once: R07), #622 (inspector shows no recent denials: R09, AGG-24), #623 (symlink to
  `.agents` can be created: R13), #624 (`/panes` picker keys reach a hidden approval prompt: blocks R03), #625
  (`/model` "provider zai is inactive", no route switch: AGG-27), #626 (owner question: a Permissive "don't ask
  again" rule still applies under Aggressive, seen in the AGG-33 replay).
- AGG-01, AGG-25 and AGG-32 fail against their frozen expectations where later fixes or decisions changed the screens
  or behaviour they compare with (#440/#458 for one AGG-01 row; #438/#444 via #448/#457, see
  [review/fix-references.md](review/fix-references.md)), with no amendment in the record; two AGG-01 rows are not
  explained by any decision in the record. The owner should decide whether the cases or the product are wrong.
- Several passes rest on the product model declining a request rather than on the protection itself (AGG-10, 14,
  21, 30, R08 in part, R11, R16; AGG-R10 used `rg`); each is disclosed in its summary. AGG-20 is blocked for the same
  reason (the child agent declined everything).
- `qa/code-blind-functional/check.py` stops at the first unresolved case, as expected with failures and blocks
  ([checker-output.txt](checker-output.txt)).

## Candidate

`origin/main` `6ebfa1f47c849c264c8b8c64beb64f8b6bf2c7b7`, packaged macOS arm64 0.1.48, `bin/corbanu` SHA-256
`9fe39035…e5d1` ([candidate/candidate-manifest.md](candidate/candidate-manifest.md)).

## Design

The first run's frozen design is reused verbatim ([design/frozen-design-20261010.json](design/frozen-design-20261010.json)).
Owner decisions that change expected results are applied as an amendment table, and new cases cover the decisions'
new behaviours; both were written, reviewed and designed by one fresh read-only agent (`corbanu exec`, Opus 5.5
High) from a decisions-only packet and frozen with SHA-256 at commit `4cf4083227` before execution
([design/designer-session.md](design/designer-session.md), [design/FROZEN.sha256](design/FROZEN.sha256),
[design/amendments.md](design/amendments.md), [design/new-cases.json](design/new-cases.json)). `design.json`
combines them (built by [tools/make_design_r.py](tools/make_design_r.py)).

## Execution

Isolated execution in the UTM guest "macOS 3" per `qa/code-blind-functional/isolated-execution.md`, harness in
[harness/](harness/). Isolation gate re-measured first ([common/gate/](common/gate/): 85 probes ok, source hosts
denied, inference reachable); every group re-ran the probes before its executor and the native-prompt guard stayed
clear ([runs/*/guard-after.txt](runs/)). One fresh executor (`corbanu exec` of the candidate, GLM 5.3 Flash) per
group, run one at a time on rotating accounts wiped before and after: 28 groups (G01–G28, 2026-10-12 05:06Z–11:30Z)
and 6 replays (G08b, G11b, G14b, G17b, G22b, G28b; reasons and neutral notes in
[tools/replays.json](tools/replays.json); earlier attempts kept). Product under test on GLM 5.3 Flash (`zai`),
except AGG-A01 which requires GLM 5.2 on `zai-anthropic`, and the AGG-20 child agent, which the executor started on
GLM 5.3 in `/spawn`. The ZAI key reached the guest only through the
owner-approved mediator path of the first run.

## Evidence check

Three independent fresh-agent checks (none an executor), all within the budget of 5 reviews: r1 and r2 led to the
corrections in the ledger; r3 found every disposition supported but failed on three mechanical points, corrected
afterwards and not independently verified ([review/r3-corrections.md](review/r3-corrections.md)). Last report:
[review/evidence-check.md](review/evidence-check.md); ledger in
[review-ledger.md](review-ledger.md).

## Known limits

- macOS only. No code-blind milestone execution exists for Windows or Linux; that is a limit, not a pass.
- #505: the restart-only protections named on screen 07 still need a restart; the cases follow that screen.
- AGG-R06 (kill switch interrupting a Claude pane) not executed: no Claude Code ≥ 2.1.292 in the guest.
- Isolation of the designer and checker is instruction-only (read-only sandbox, no OS read restriction).
- `utmctl` failed at the end (scripting bridge exception); the guest was shut down from inside instead, so no
  end-of-run root policy capture exists (only `common/policy-root-start.txt`).
- Mediator rows are not labelled by lane, and a run's mediator window overlaps the neighbouring run; claims that
  "nothing was sent" (AGG-R01, R04, AGG-22, AGG-28) rest on the evidence checker's reading of request order and
  sizes. AGG-16's "redacted in model context" rests only on the model's statement.
- The executor model (GLM 5.3 Flash) is set by the harness default (`harness/run_group.sh`), not recorded in the event
  logs. The executor is the candidate binary itself (`corbanu exec`).
- Executors had shell write access to the product home; G02, G15 and G17 reset or reshaped it, each openly in its
  report.
- Reasoning effort of the product varied where the executor had to choose one (pane B at Low in AGG-R03, the AGG-R14
  child at max, Low in G17b); it is not recorded in the model fields.
- `common/effective-policy.txt` names a `policy-root-end.txt`; it does not exist for this re-run (see the `utmctl`
  item above).

Note: each run's end-state copy of the test project is under `runs/<label>/collected/fixture/workspace/`. Its git
directory is stored as `dot-git/` (renamed only so the outer repository keeps the files); its synthetic `.env` is
ignored by the project's own `.gitignore` and so is not in the repository.
The SQLite WAL files the G28 executor copied from the used product homes (`runs/G28/collected/evidence/used-home-r1*/`) are
stored gzipped to stay under the repository blob-size limit; one (`used-home-r15/product-home/pfterminal_logs_2.sqlite-wal.gz`,
644 KB, SHA-256 `e15037fec18e1992781f5a2527c2d40e2c6fe7eff172893b211aa9029ffe1790`) is kept outside the repository. No result depends on them.
