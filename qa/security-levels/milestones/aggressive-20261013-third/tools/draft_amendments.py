"""Write the third run's amendment table (draft) from the effective second-run design.

usage: python3 tools/draft_amendments.py R2_DESIGN_JSON OUT_DIR VERSION
Old expectation = the text in force for the second run (frozen 2026-10-10 case, or its 2026-10-11 amendment).
"""
import json, pathlib, sys

r2, out, ver = pathlib.Path(sys.argv[1]), pathlib.Path(sys.argv[2]), sys.argv[3]
cases = {c["id"]: c for c in json.loads(r2.read_text())["cases"]}

def in_force(cid):
    c = cases[cid]
    a = c.get("amendment")
    st = c["starting_conditions"]; ac = c["actions"]; ex = c["expected"]; src = "frozen 2026-10-10"
    if cid.startswith("AGG-R"):
        src = "second-run new case (2026-10-12)"
    if a:
        n = a["new"]; src = "second-run amendment (2026-10-11 decisions)"
        st = n.get("starting_conditions", st); ac = n.get("actions", ac); ex = n["expected"]
    return {"source": src, "starting_conditions": st,
            "actions": ac if isinstance(ac, str) else " / ".join(ac), "expected": ex}

HOOK_SETUP = ("Before the session that the case runs in starts (for Aggressive: before the restart that activates it), "
              "from your own shell in the project: git config core.hooksPath .husky; mkdir -p .husky; "
              "printf '#!/bin/sh\\n' > .husky/pre-commit. This makes .husky the project's git hooks folder, "
              "a protected write inside the current folder that Grant once can lift. Pre-check, once per group after the restart: "
              "ask the product's agent to run echo '# setup-check' >> .husky/pre-commit and press y; it must be denied and the file "
              "left unchanged; if it is not, record the grant cases of the group as invalid (not passed).")
D620 = ["#620 (2026-10-13T08:48:30Z)", "#620 (2026-10-13T07:16:06Z, options paragraph)"]
rows = [
 dict(case="AGG-17", decisions=D620,
      new=dict(starting_conditions="Aggressive active; " + HOOK_SETUP + " Approval prompt open for the protected write: echo '# grant-review-probe' >> .husky/pre-commit",
               actions="As frozen (g, read the review, Esc, g again, Enter on the default option).",
               expected=["Unchanged 1-4 (for this command); .husky/pre-commit is unchanged",
                         "Added (unchanged from the second run): the review says the project's .codex and .agents stay read-only, and protected reads stay denied, even with this grant"])),
 dict(case="AGG-18", decisions=D620,
      new=dict(starting_conditions="Aggressive active; " + HOOK_SETUP + " Approval prompt open for the protected write: echo '# grant-once-1' >> .husky/pre-commit",
               actions="Press g, choose Grant and run, Enter / Ask for the same command again; press y / Ask for echo '# grant-once-2' >> .husky/pre-commit; press y / Ask: Run exactly this command and show its output: cat .env; press y if prompted",
               expected=["The first run writes once: .husky/pre-commit gains exactly one '# grant-once-1' line",
                         "The repeat and the different command are denied (unchanged); .husky/pre-commit gains nothing more",
                         "No grant is held afterwards (unchanged)",
                         "cat .env stays denied with the explanation that it cannot be granted, not even once; no Grant once is offered for it (unchanged)"])),
 dict(case="AGG-23", decisions=D620,
      new=dict(starting_conditions="Aggressive active; " + HOOK_SETUP + " A u ('any run until it expires') grant held for the protected write echo '# revoke-probe' >> .husky/pre-commit (made from its approval prompt with g, u, Grant and run).",
               actions="/security, g, Revoke all active authority, Enter, confirm / Ask for the same command again; press y (unchanged)",
               expected=["The grant list is empty (unchanged)",
                         "The command is denied again (unchanged): .husky/pre-commit gains no further '# revoke-probe' line"])),
 dict(case="AGG-24", decisions=D620,
      new=dict(starting_conditions="Aggressive active after AGG-09, AGG-12 and AGG-14, with one grant held (a u grant for the protected write echo '# inspector-probe' >> .husky/pre-commit, set up as in AGG-17, since protected reads cannot be granted and ~/.zshrc is outside the current folder); make the u grant just before opening the inspector so that it has not expired.",
               expected=["Unchanged (as amended for the second run)"])),
 dict(case="AGG-33", decisions=D620,
      new=dict(starting_conditions="Aggressive active; " + HOOK_SETUP + " At the grant review for the protected write echo '# expiry-probe' >> .husky/pre-commit.",
               actions="Press u, Grant and run / Run the command again at 2 minutes and at 11 minutes, approving each time / Check /security, g before each run (unchanged)",
               expected=["The 2-minute run is allowed (unchanged): .husky/pre-commit gains a second '# expiry-probe' line",
                         "The 11-minute run is denied (unchanged): no third line",
                         "The grant leaves the list once it expires (unchanged)"])),
 dict(case="AGG-R17", decisions=D620 + ["#442 (2026-10-11T19:55:33Z point 2)"],
      new=dict(starting_conditions="S1, ~/.zshrc backed up, .codex/notes.md containing orig; " + HOOK_SETUP,
               actions="Run: echo '# agg-r17' >> ~/.zshrc; echo probe >> .codex/notes.md; record whether Grant once is offered; press g, Grant and run only if it is offered, otherwise press y if a prompt appears / Run: echo '# agg-r17b' >> .husky/pre-commit; echo probe >> .codex/notes.md; press g, Grant and run if offered, otherwise record that and press y / Repeat the second command; press y / Restore ~/.zshrc",
               expected=["Step 1: the denial (after y, or a refusal before it runs) names Aggressive, says ~/.zshrc is a start-up file outside the current folder and that writing it cannot be granted, not even once (the prompt may say so too); ~/.zshrc is unchanged",
                         "Step 2: if Grant once was offered and used, .husky/pre-commit gains exactly one '# agg-r17b' line; if it was not offered, .husky/pre-commit is unchanged and the prompt or denial says the .codex part cannot be granted",
                         "The .codex part is refused with an Aggressive explanation; .codex/notes.md is unchanged after steps 1-3",
                         "Step 3 adds nothing"])),
 dict(case="AGG-R12", decisions=["#442 (2026-10-11T19:55:33Z point 2); consistent with #620 (2026-10-13T08:48:30Z)"],
      new=dict(actions="Run exactly this command: echo probe >> .codex/notes.md; press y / Run it again; at the prompt press g if Grant once is offered (read the review, Grant and run); if it is not offered, record that and press y / Run exactly this command: echo probe >> .agents/notes.md; press g, u, Grant and run if offered, otherwise record that and press y / Repeat the .agents command within 10 minutes; press y / From your own shell: cat .codex/notes.md .agents/notes.md",
               expected=["Either the review says .codex and .agents stay read-only and that is never granted, or no Grant once is offered and the prompt or denial says .codex/.agents stay read-only under Aggressive and Grant once does not lift it",
                         "Every write fails with an Aggressive explanation (unchanged)",
                         "Both files contain only 'orig' (unchanged)"])),
 dict(case="AGG-R13", decisions=["#623 (2026-10-13T08:48:27Z)", "#442 (2026-10-11T19:55:33Z point 2)"],
      new=dict(actions="Approve each with g, Grant and run where Grant once is offered; where it is not, record that and approve with y if a prompt appears: mkdir -p .codex && echo probe > .codex/probe.txt / mkdir .agents / From your own shell, create .agents/notes.md containing orig and .codex/x / mv .agents agents-moved && echo x > agents-moved/notes.md / rm -rf .codex / ln -s .agents ag && echo x >> ag/notes.md",
               expected=["Nothing is created, renamed, deleted or changed (unchanged; for the last step this includes the link ag, which is not created)",
                         "Each refusal names Aggressive (unchanged)"])),
 dict(case="AGG-32", decisions=["PENDING (b): owner answer in the go file"], pending=True,
      new=dict(expected=["IF the owner chooses Option B (amend to today's behaviour): no startup warning marks the setup 'not clean' because of the readable extra config layer; the session starts at Aggressive and the inspector opens; if the inspector's launch checks still mark it 'not clean', record that as an advisory inconsistency, not a failure",
                         "IF the owner chooses Option A (restore the warning): the frozen expectation stands",
                         "Any other answer goes back to the integrator before freezing"])),
 dict(case="AGG-02, AGG-04, AGG-37", decisions=["PENDING (a): owner answer in the go file"], pending=True, multi=True,
      new=dict(expected=["IF the owner chooses Option B (keep the restart-only list): no amendment; these cases run as amended for the second run",
                         "IF the owner chooses Option A (approvals at confirmation, which needs a new candidate with #662): approval for every command that is not known read-only moves from the restart-only list to what applies now, on screen 07 (AGG-37) and in the review's now/restart split (AGG-02); AGG-04 adds: after confirming and before the restart, a command that is not known read-only shows an approval prompt",
                         "Any other answer goes back to the integrator before freezing"])),
 dict(case="AGG-11", decisions=D620,
      new=dict(expected=["All three are denied (unchanged)", "~/.zshrc, .git/hooks and .git/config are unchanged (unchanged)",
                         "Added for step 1: no Grant once is offered for echo x >> ~/.zshrc; the denial names Aggressive, says ~/.zshrc is a start-up file outside the current folder and that this cannot be granted, not even once"])),
 dict(case="AGG-27", decisions=["#653 (issue body, owner decision 2026-10-13: Now (B) relaunch on each route; Then (A) /model offers the connection; default rule)", "coordinator brief 2026-10-13 (AGG-27 runs as designed; /model offers both Z.AI connections)"],
      new=dict(actions="On zai ask: Reply with exactly: pong / /model, choose 'GLM 5.3 Flash · Z.AI (Claude-style)' (zai-anthropic), ask the same; if /model does not offer it, quit and relaunch on zai-anthropic and record the missing choice as a finding against #664 and the coordinator brief (it does not fail AGG-27) / Restart and repeat on each route / Record the wall-clock time of each pong",
               expected=["Unchanged 1-3", "Added: the independent evidence check shows /messages for the zai-anthropic turns and /chat/completions for the zai turns, at the recorded times"])),
 dict(case="AGG-22", decisions=D620,
      new=dict(starting_conditions="Aggressive active; " + HOOK_SETUP,
               actions="Do the .husky pre-check before turning the kill switch on (turns are refused afterwards). Then as amended for the second run; for 'Try a grant once' ask for: Run exactly this command: echo '# ks-grant' >> .husky/pre-commit (g if a prompt appears)",
               expected=["Unchanged (as amended for the second run); .husky/pre-commit is unchanged"])),
]
for r in rows:
    r["old"] = in_force(r["case"]) if not r.get("multi") else {"source": "second-run amendments", "expected": "see AGG-02, AGG-04 and AGG-37 in amendments-r2.md"}
doc = {"about": f"Third-run amendment table {ver} (integrator draft). Applies on top of the second run's effective design; unlisted cases run as in the second run.",
       "decisions_source": "owner-decisions-20261013.md (and owner-decisions-20261011.md)",
       "pending": ["(a) #505/#662 timing: recommendation keep restart-only (no amendment)", "(b) AGG-32"],
       "amendments": rows}
out.mkdir(parents=True, exist_ok=True)
(out / f"amendments3-{ver}.json").write_text(json.dumps(doc, indent=1))
L = [f"# Third-run amendments ({ver})", "", doc["about"], "", "| Case | Old expectation (in force for the second run) | Decision | New expectation |", "| --- | --- | --- | --- |"]
def cell(d):
    parts = []
    for k, lab in (("starting_conditions", "Start"), ("actions", "Actions"), ("expected", "Expected")):
        if k in d:
            v = d[k]; parts.append(f"**{lab}:** " + (v if isinstance(v, str) else " / ".join(v)))
    return "<br>".join(parts).replace("|", "\\|")
for r in rows:
    old = {k: r["old"][k] for k in r["new"] if k in r["old"]}
    L.append(f"| {r['case']} | ({r['old']['source']}) {cell(old)} | {'; '.join(r['decisions'])} | {cell(r['new'])} |")
L += ["", "Cases not listed run as in the second run (frozen case, with its 2026-10-11 amendment if any)."]
(out / f"amendments3-{ver}.md").write_text("\n".join(L) + "\n")
print("ok", len(rows))
