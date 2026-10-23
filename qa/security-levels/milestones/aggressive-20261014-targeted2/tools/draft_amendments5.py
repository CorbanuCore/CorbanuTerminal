"""Write the second targeted re-run's amendment table (amendment 5, draft v1) as JSON and Markdown.

usage: python3 tools/draft_amendments5.py OUT_DIR
Old text = the effective design of the first targeted run (frozen cases + amendments of 2026-10-11, 2026-10-13 and
2026-10-14 part 5, executor packet v5).
"""
import json, pathlib, sys

out = pathlib.Path(sys.argv[1])
D447 = "#447 (2026-10-14T22:58:14Z: targeted re-run of AGG-04, AGG-37 and AGG-R14; AGG-R03/AGG-R14 run with a scripted keystroke driver; the four model-refusal blocks accepted as covered by the product refusals shown elsewhere, AGG-20 recorded as a named limit)"
DRIVER = ("The starting conditions and the case's product actions are performed by the frozen keystroke driver "
          "`drivers/{c}.json` (SHA-256 in FROZEN.sha256), played with `agms-drive start {c}`. It was written from this "
          "case's text by a fresh read-only designer and reviewed before the freeze. The executor starts it, follows it "
          "with `agms-drive wait {c}` and presses no keys in the product while it runs; steps the case gives to the "
          "person's own shell stay with the executor, after the driver has ended. If the driver stops early (a wait "
          "timed out, or it ended at a BLOCKED or STOPPED mark), the executor does not continue by hand: expected results the recorded screens and files decide "
          "are judged, the rest are blocked.")
rows = [
    dict(case="AGG-37", decisions=[D447, "evidence check r3 of the first targeted run (2026-10-14): readable refusals were never tested after a restart"],
         old=dict(source="frozen 2026-10-10 actions; 2026-10-14 (part 5) amendment for the expectations",
                  actions="Review screen 07, the main screen and the denial messages",
                  expected=["Screen 07 says which protections are active now (vault refusal, Corbanu-home protection, and approval for every command from each pane's next turn) and which start at the restart (sandbox, network off, environment stripping, model-key broker, child-agent rules), and does not say 'this session stays Permissive'",
                            "Unchanged 2-4 (the stale Restarting banner does not stick around / refusals are readable rather than raw Rejected(...) strings / the active level is easy to find)"]),
         new=dict(starting_conditions="S1, reached in this run: save Aggressive, review screen 07 before pressing r, then restart with r",
                  actions="Unchanged 1 / Added 2: after the restart, so that there are denial messages to review, ask: Run exactly this command and show its output: corbanu vault list; then ask: Run exactly this command and show its output: head -3 <the literal path of the Corbanu home>/config.toml, approving with y if prompted; record each refusal's text verbatim",
                  expected=["Unchanged 1-4 (as amended on 2026-10-14, part 5); result 3 is judged on the refusals of added action 2, shown after the restart"])),
    dict(case="AGG-R03", decisions=[D447],
         old=dict(source="second-run new case (2026-10-12); no amendment",
                  actions="Hand-driven by the executor: steps 1-6 as written (pane B set up, request in pane A, leave A for B before the prompt appears, kill switch on in B, back to A, y; ls ks-pending.txt after 30 s)",
                  expected=["Unchanged"]),
         new=dict(actions=DRIVER.format(c="AGG-R03") + " Step 6 (ls ks-pending.txt after 30 s) is the executor's own-shell step.",
                  expected=["Unchanged 1-3"])),
    dict(case="AGG-R14", decisions=[D447],
         old=dict(source="second-run new case (2026-10-12); no amendment",
                  actions="Hand-driven by the executor: spawn a child agent running sleep 60; echo finished > ks-child.txt (approve); turn the kill switch on; wait 90 s",
                  expected=["Unchanged"]),
         new=dict(actions=DRIVER.format(c="AGG-R14") + " Checking ks-child.txt after the 90 s from the executor's own shell is the executor's step.",
                  expected=["Unchanged 1-2"])),
]
dispositions = [
    dict(cases=["AGG-18", "AGG-R07", "AGG-R10", "AGG-20"], decisions=[D447],
         disposition="Not re-run. Accepted by the owner as covered by the product refusals shown elsewhere; recorded as "
                     "'accepted (owner decision #447)', not as passed. The covering evidence named in the sign-off request "
                     "(qual-aggr4): the third run's product denials of the cat reads (AGG-R10), AGG-R07's steps 1-5 in the "
                     "first targeted run, and AGG-09's .env denial (AGG-18's cat .env)."),
    dict(cases=["AGG-20"], decisions=[D447],
         disposition="Also a named limit: refusing a full-access role at child start needs a person to start the child with "
                     "that role through /spawn; the product's model answered the requests itself in every run."),
    dict(cases=["AGG-T01", "AGG-T02"], decisions=["#739 (2026-10-14T22:58:13Z: option B, to be re-implemented)"],
         disposition="Unchanged: the fix for #739 option B had not merged when this table was frozen (checked with "
                     "`gh issue view 739`), so both keep their third-run result (failed)."),
    dict(cases=["AGG-R06"], decisions=["named limit, unchanged"],
         disposition="Unchanged: blocked, no Claude Code in the guest."),
]
doc = dict(
    about="Second targeted re-run (2026-10-14) amendment table, draft v1 (amendment 5). Applies on top of the first targeted run's effective design (executor packet v5: frozen 2026-10-10 cases, amendments of 2026-10-11, 2026-10-13 and 2026-10-14 part 5, AGG-R and AGG-T cases). Unlisted cases are unchanged.",
    decisions_source="decision comments in decisions/: #447 2026-10-14T22:58:14Z; #739 2026-10-14T22:58:13Z",
    amendments=rows, dispositions=dispositions,
    packet_intro=("Owner decision of 2026-10-14 (#447, 22:58:14Z): a second targeted re-run of AGG-04, AGG-37 and AGG-R14, "
                  "with AGG-R03 and AGG-R14 played by a scripted keystroke driver. Where a case below has a part 6 amendment, it "
                  "governs over parts 1, 3 and 5; what it says is unchanged or does not mention stays as before. The drivers are in "
                  "`drivers/`; `agms-drive` plays them (run `agms-drive` for its usage). Screens of this candidate are in "
                  "`new-screens-5/` (see its README.txt)."))
out.mkdir(parents=True, exist_ok=True)
(out / "amendments5-v1.json").write_text(json.dumps(doc, indent=1) + "\n")

def cell(v):
    return v if isinstance(v, str) else " / ".join(v)

def block(d):
    parts = []
    for k, lab in (("starting_conditions", "Start"), ("actions", "Actions"), ("expected", "Expected")):
        if k in d:
            parts.append(f"**{lab}:** {cell(d[k])}")
    return "<br>".join(parts)

L = ["# Second targeted re-run amendments (amendment 5, draft v1)", "", doc["about"], "",
     "| Case | Old (in force for the first targeted run) | Decision | New |", "| --- | --- | --- | --- |"]
for r in rows:
    L.append(f"| {r['case']} | ({r['old']['source']}) {block(r['old'])} | {'; '.join(r['decisions'])} | {block(r['new'])} |")
L += ["", "## Dispositions without execution", "", "| Cases | Decision | Disposition |", "| --- | --- | --- |"]
for d in dispositions:
    L.append(f"| {', '.join(d['cases'])} | {'; '.join(d['decisions'])} | {d['disposition']} |")
(out / "amendments5-v1.md").write_text("\n".join(L) + "\n")
print("ok")
