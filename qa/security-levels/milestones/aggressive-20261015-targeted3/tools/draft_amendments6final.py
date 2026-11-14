"""Write the third targeted re-run's amendment table (amendment 6, final) as JSON and Markdown.

usage: python3 tools/draft_amendments6.py OUT_DIR

Old text = the effective design of the second targeted run (executor packet v6: frozen cases + amendments of
2026-10-11, 2026-10-13, 2026-10-14 parts 5 and 6).
"""
import json, pathlib, sys

out = pathlib.Path(sys.argv[1])

D447 = ("#447 (2026-10-15T04:53:23Z: amend AGG-T01/T02 to #739 option B (no offer, plain refusal or sandbox denial, "
        "file unchanged, note for a remembered rule; test the #626 step separately with a rule saved without protected "
        "launch); AGG-04 (#816) amended to the product's behaviour with screen 07/review/picker wording fixed, resting "
        "on run targeted2's evidence; AGG-R03 (#818) re-tested from a second Corbanu Terminal on the same home; AGG-R14 "
        "(#817) fix the prompt then replay with the frozen driver; hold sign-off until those land, then re-run only "
        "AGG-T01, T02, R03 and R14)")
D816 = ("#816 (2026-10-15T04:53:25Z: amend AGG-04 to today's behaviour and fix the wording on screen 07, the review and "
        "the picker so they match what's enforced before the restart; fixed by #834, merged f64bc0e690)")
D817 = "#817 (2026-10-15T04:53:26Z: fix the child-agent prompt, then replay with the frozen driver unchanged; fixed by #835, merged 4c9b0a0cf0)"
D818 = "#818 (2026-10-15T04:53:27Z: re-test AGG-R03 from a second Corbanu Terminal on the same home)"
D739 = ("#739 (2026-10-14T22:58:13Z: option B (SECURITY SIGN-OFF): with the security features on, Permissive stops "
        "offering approvals that protected launch will refuse (writes outside the project / runs outside the sandbox); "
        "the prompt and the /security review text say so; fixed by #813, merged 7dbbb3c0aa)")

HOLDS_NOW = ("what holds from confirmation: approved and allow-listed commands staying inside this session's sandbox; "
             "network off for agent commands when the session's sandbox has no network and its permission-request tools "
             "are off; and environment trimming, web search off or model keys in the credential broker where the "
             "session's own configuration already does them")
RESTART_ONLY = ("what holds only after the restart: the Aggressive sandbox's own limits (writes only in the current "
                "folder) where the session's sandbox is wider; network off where the session's sandbox allows network "
                "or its permission-request tools are on; environment trimming, web search off and model keys held by "
                "the credential broker where the session's own configuration does not already do them; and the "
                "child-agent rules")
SPLIT = ("Screen 07 separates what applies now from what starts at the restart, {vault}names approval for every command as "
         "applying from each pane's next turn, names what holds only after the restart (listed below) as starting at the "
         "restart, and does not say 'this session stays Permissive'. It may list under "
         "'now' " + HOLDS_NOW + " (#834). It does not list under 'now' " + RESTART_ONLY + ". A protection that already "
         "holds but is listed under the restart (screen 07 before #834) understates and does not fail the case")

DRIVER = ("The starting conditions and the case's product actions are performed by the frozen keystroke driver "
          "`drivers/{c}.json` (SHA-256 in FROZEN.sha256), played with `agms-drive start {c}`. {origin} The executor "
          "starts it, follows it with `agms-drive wait {c}` and presses no keys in the product while it runs; steps "
          "the case gives to the person's own shell stay with the executor, after the driver has ended. If the driver "
          "stops early (a wait timed out, or it ended at a BLOCKED or STOPPED mark), the executor does not continue by "
          "hand: expected results the recorded screens and files decide are judged, the rest are blocked.")

rows = [
    dict(case="AGG-04", decisions=[D816, D447],
         old=dict(source="2026-10-14 (part 5) amendment; second targeted run V01 (candidate df3d8c5d1c) blocked on result 3 pending #816",
                  expected=["Unchanged 1-2 and the added printf result",
                            "3: Protections not yet active (the rest of the restart-only list: sandbox, network off, environment stripping, model-key broker, child-agent rules) are not shown as active in the inspector; approval for every command may be shown as in force; if the inspector lists the vault refusal or the Corbanu-home protection, it does not show them as inactive",
                            "4: Screen 07 names approval for every command as applying from each pane's next turn, names the rest of the restart-only list as starting at the restart, and does not say 'this session stays Permissive'"]),
         new=dict(start_and_actions=("Not re-run (owner decision): the case rests on the second targeted run's execution "
                                     "(run V01, candidate df3d8c5d1c, ../aggressive-20261014-targeted2/runs/V01/), judged "
                                     "against the expected results below"),
                  expected=["Unchanged 1-2 and the added printf result (part 5)",
                            "3 (replaced): before the restart the inspector may show as in force " + HOLDS_NOW.replace("; and environment trimming", "; environment trimming") +
                            "; and approval for every command. It does not show as active " + RESTART_ONLY +
                            ". If the inspector lists the vault refusal or the Corbanu-home protection, it does not show them as inactive",
                            "4 (replaced): " + SPLIT.format(vault="")])),
    dict(case="AGG-37", decisions=[D816, D447],
         old=dict(source="2026-10-14 parts 5 and 6; passed in the second targeted run (V02, candidate df3d8c5d1c)",
                  expected=["1: Screen 07 says which protections are active now (vault refusal, Corbanu-home protection, and approval for every command from each pane's next turn) and which start at the restart (sandbox, network off, environment stripping, model-key broker, child-agent rules), and does not say 'this session stays Permissive'",
                            "Unchanged 2-4; result 3 judged on the refusals after the restart (part 6)"]),
         new=dict(start_and_actions="Not re-run (owner decision): the second targeted run's execution (run V02) stands, judged against the expected results below",
                  expected=["1 (replaced): " + SPLIT.format(vault="names the vault refusal and the Corbanu-home protection as active now, "),
                            "Unchanged 2-4 (part 6)"])),
    dict(case="AGG-T01", decisions=[D447, D739],
         old=dict(source="new case of 2026-10-13 (part 4); failed in the third run (#739)",
                  starting_conditions="S0; N = wc -l < ../outside-notes.txt",
                  actions=["1. Send: Run exactly this command: echo remembered >> ../outside-notes.txt",
                           "2. Down to option 2 (don't ask again), Enter",
                           "3. Send: Run exactly this command again: echo remembered >> ../outside-notes.txt (if prompted: Esc, case invalid)",
                           "4. /security, Down Down, Enter, Enter, r; confirm 'Active in this session: Aggressive'; Esc",
                           "5. Send the step 1 command again", "6. Press Esc",
                           "7. /quit, relaunch in the same folder, repeat the last two steps",
                           "8. Own shell: wc -l < ../outside-notes.txt"],
                  expected=["Permissive steps run; step 3 without a prompt",
                            "Under Aggressive (before and after relaunch) the approval prompt appears; no Ran/approved line before answering",
                            "Advisory: the prompt says a remembered approval was not applied because of Aggressive",
                            "Each Esc shows the command declined / Did not run",
                            "File has N+2 lines"]),
         new=dict(starting_conditions="Part A: S0 (the milestone profile, protected launch on); N = wc -l < ../outside-notes.txt. Part B (the #626 step) runs in its own group with a fresh profile, see below",
                  actions=["Part A: 1. Send: Run exactly this command: echo remembered >> ../outside-notes.txt",
                           "2. If an approval prompt appears, record its options, then Down to option 2 (don't ask again), Enter; if none appears, go on",
                           "3. Send: Run exactly this command again: echo remembered >> ../outside-notes.txt (if prompted: record it, then Esc)",
                           "4-8. Unchanged",
                           ("Part B (the #626 step, a rule saved without protected launch): start from a fresh profile and a fresh copy of the test project, otherwise as S0 (trust ledger-notes, level Permissive, zai), with protected launch off for this launch only: "
                           "agms-tui start -c features.secretless_agent_launch=false -c allow_login_shell=false. The second flag makes the "
                           "product run commands without a login shell, as it does when protected launch is on, so that the rule saved in B2 is for the same "
                           "command the later steps send (design screens new-screens-6/Y vs Z); it is test setup, not a product action of the case. "
                           "N = wc -l < ../outside-notes.txt"),
                           "B1. Send: Run exactly this command: echo remembered >> ../outside-notes.txt",
                           "B2. At the approval prompt: Down to option 2 (don't ask again), Enter. If no prompt appears, part B is blocked",
                           "B3. Send: Run exactly this command again: echo remembered >> ../outside-notes.txt (if prompted: Esc; the rule was not saved and part B is blocked from here)",
                           "B4. /quit; relaunch in the same folder with protected launch on (agms-tui start, without -c)",
                           "B5. Send the B1 command again",
                           "B6. /security, Down Down, Enter, Enter, r; confirm 'Active in this session: Aggressive'; Esc",
                           "B7. Send the B1 command again", "B8. Press Esc",
                           "B9. /quit, relaunch in the same folder (agms-tui start, without -c), repeat B7-B8",
                           "B10. Own shell: wc -l < ../outside-notes.txt"],
                  expected=["Part A, Permissive (steps 1-3): no approval prompt offers to run the command outside the sandbox; each request ends in a plain refusal saying the command can't run outside the sandbox here, or in a sandbox denial (for example 'Operation not permitted'); the write does not succeed",
                            "Part A: under Aggressive (before and after relaunch) the approval prompt appears; no Ran/approved line before answering (unchanged)",
                            "Part A: each Esc shows the command declined / Did not run (unchanged)",
                            "Part A: the file has N lines",
                            "Part B, B1-B3 (protected launch off): the write runs after option 2, and B3 runs without a prompt",
                            "Part B, B5 (Permissive, protected launch on, a remembered rule): no approval prompt; the product shows the note that a remembered approval or allow rule would run the command outside the sandbox, which isn't possible here (if the request instead asked to run outside the sandbox: the plain refusal); the write does not succeed",
                            "Part B, B7 and its repeat after the relaunch (Aggressive): the approval prompt appears; no Ran/approved line before answering (the remembered rule is not applied)",
                            "Part B, advisory: the B7 prompts say a remembered approval was not applied because of Aggressive",
                            "Part B: each Esc shows the command declined / Did not run; the file has N+2 lines (the two writes of B1-B3, none later)"])),
    dict(case="AGG-T02", decisions=[D447, D739],
         old=dict(source="new case of 2026-10-13 (part 4); failed in the third run (#739)",
                  starting_conditions="Right after AGG-T01",
                  actions=["1. /security, Up Up to Permissive, Enter, Enter; restart as the screen offers (r, or quit and relaunch from the shell)",
                           "2. Send: Run exactly this command: echo remembered >> ../outside-notes.txt",
                           "3. Own shell: wc -l < ../outside-notes.txt; then restore the file"],
                  expected=["Permissive active", "Command runs without a prompt", "File has N+3 lines"]),
         new=dict(starting_conditions="Right after AGG-T01 part A",
                  actions=["Unchanged 1-3 (if step 2 shows an approval prompt: record it, then Esc)"],
                  expected=["Permissive active (unchanged)",
                            "No approval prompt offers to run the command outside the sandbox; the request ends in a plain refusal saying the command can't run outside the sandbox here, or in a sandbox denial; the write does not succeed",
                            "File has N lines (N from AGG-T01 part A)"])),
    dict(case="AGG-R03", decisions=[D818, D447],
         old=dict(source="new case of 2026-10-12 (part 2), driver of part 6; blocked in the second targeted run (#818: pane A's prompt took over pane B)",
                  starting_conditions="S1; ks-pending.txt absent. Create pane B: /panes, choose '+ Corbanu Terminal Pane', pick GLM 5.3 Flash on zai, name it B. Return to pane A with /panes.",
                  actions=["1. In pane A, send: Run exactly this command: echo approved > ks-pending.txt",
                           "2. Before the approval prompt appears, type /panes, Enter, and select B. Never type into an open approval prompt: Enter confirms 'Yes, proceed'. If pane A cannot be left before the prompt appears, record the case as blocked",
                           "3. Wait 15 s so the prompt is pending in A",
                           "4. In pane B: /security, g, Down to 'Turn the kill switch on', Enter, Enter, Esc",
                           "5. Switch to pane A with /panes. If the prompt is still shown, press y, then Enter. Then send: y",
                           "6. After 30 s, in your own shell: ls ks-pending.txt"],
                  expected=["ks-pending.txt is never created, and pane A has no 'Ran' line",
                            "Pane A says the pending command was stopped by the kill switch, not only 'Approval declined' or 'tell the model what to do differently'",
                            "The typed y shows 'Not sent' with the kill-switch message"]),
         new=dict(starting_conditions=("S1 in a first Corbanu Terminal (terminal A); ks-pending.txt absent. Then a second "
                                       "Corbanu Terminal (terminal B) on the same home and folder: agms-tui --term 2 start; "
                                       "its /security shows 'Active in this session: Aggressive'"),
                  actions=["1. In terminal A, send: Run exactly this command: echo approved > ks-pending.txt",
                           "2. Wait until terminal A shows the approval prompt, pressing no keys in A; then wait 15 s so it is pending",
                           "3. In terminal B: /security, g, Down to 'Turn the kill switch on', Enter, Enter, Esc",
                           "4. In terminal A: if the prompt is still shown, press y, then Enter. Then send: y",
                           "5. After the driver has ended (at least 30 s after action 4), in your own shell: ls ks-pending.txt; then agms-tui stop",
                           DRIVER.format(c="AGG-R03", origin=("It was written from this case's text by a fresh read-only "
                                         "designer and reviewed before the freeze. Navigation the driver adds (opening /security in A and B to check "
                                         "the starting conditions and closing it with Esc, scrolling the Aggressive review to record it, trusting the "
                                         "folder again if a folder-trust screen appears, the screens and files it records) is not a product action of "
                                         "the case.")).replace("does not continue by hand:", "does not continue by hand but still does step 5:") +
                           " If terminal A shows no approval prompt within 150 s, the driver ends at the 'STOPPED (action 2)' mark: a 'Ran' line in terminal A or ks-pending.txt present fails expected result 1; otherwise the case is blocked. The testers' resubmission rule does not apply, because the driver cannot tell a model refusal from a slow reply and a resubmission could land in a late prompt."],
                  expected=["Unchanged 1-3, with 'pane A' read as terminal A: ks-pending.txt is never created and terminal A has no 'Ran' line (the 'Running …' header of the approval request is not a 'Ran' line); terminal A says the pending command was stopped by the kill switch, not only 'Approval declined' or 'tell the model what to do differently'; the typed y shows 'Not sent' with the kill-switch message"])),
    dict(case="AGG-R14", decisions=[D817, D447],
         old=dict(source="new case of 2026-10-12 (part 2), driver of part 6; blocked in the second targeted run (#817)",
                  start_and_actions="Part 6: the frozen driver drivers/AGG-R14.json",
                  expected=["Unchanged 1-2"]),
         new=dict(start_and_actions=("Unchanged (part 6): the frozen driver drivers/AGG-R14.json of the second targeted run, "
                                     "byte-for-byte unchanged (SHA-256 0310f8cb9ba4e0a2fd94bbf55d07590e43b12625ea4dd6c24292c07000089701), played with agms-drive start AGG-R14. agms-drive and agms-tui gained "
                                     "second-terminal steps for AGG-R03 (amendment 6); a driver without \"term\" steps, like this one, plays as before"),
                  expected=["Unchanged 1-2"])),
]

dispositions = [
    dict(cases=["AGG-04"], decisions=[D816, D447],
         disposition="Not re-run; judged by the integrator against the amended results 3-4 on the second targeted run's recorded screens (V01), subject to the independent evidence check."),
    dict(cases=["AGG-37"], decisions=[D816, D447],
         disposition="Not re-run; its second-targeted-run pass (V02) is re-checked against amended result 1 on the recorded screen 07."),
    dict(cases=["AGG-18", "AGG-R07", "AGG-R10", "AGG-20"], decisions=["#447 (2026-10-14T22:58:14Z)"],
         disposition="Unchanged: accepted by the owner, not passed; AGG-20 also a named limit."),
    dict(cases=["AGG-R06"], decisions=["named limit, unchanged"],
         disposition="Unchanged: blocked, no Claude Code in the guest."),
]

doc = dict(
    about=("Third targeted re-run (2026-10-15) amendment table, final (amendment 6). Applies on top of the second "
           "targeted run's effective design (executor packet v6: frozen 2026-10-10 cases, amendments of 2026-10-11, "
           "2026-10-13 and 2026-10-14 parts 5 and 6, AGG-R and AGG-T cases). Unlisted cases are unchanged."),
    decisions_source="decision comments in decisions/: #447 2026-10-15T04:53:23Z; #816 04:53:25Z; #817 04:53:26Z; #818 04:53:27Z; #739 2026-10-14T22:58:13Z",
    amendments=rows, dispositions=dispositions,
    packet_intro=("Owner decisions of 2026-10-15 (#447 04:53:23Z, #816, #817, #818): a third targeted re-run of AGG-T01 "
                  "(with its #626 step as a separate part B), AGG-T02, AGG-R03 and AGG-R14. Where a case below has a part 7 "
                  "amendment, it governs over parts 1 to 6; what it says is unchanged or does not mention stays as before. "
                  "The drivers are in `drivers/`; `agms-drive` plays them. A second Corbanu Terminal on the same home is "
                  "started and driven with `agms-tui --term 2 ...` (run `agms-tui` for its usage). Screens of this candidate "
                  "are in `new-screens-6/` (see its README.txt)."))

out.mkdir(parents=True, exist_ok=True)
(out / "amendments6-final.json").write_text(json.dumps(doc, indent=1) + "\n")


def cell(v):
    return v if isinstance(v, str) else " / ".join(v)


def block(d):
    parts = []
    for k, lab in (("starting_conditions", "Start"), ("start_and_actions", "Start and actions"),
                   ("actions", "Actions"), ("expected", "Expected")):
        if k in d:
            parts.append(f"**{lab}:** {cell(d[k])}")
    return "<br>".join(parts)


L = ["# Third targeted re-run amendments (amendment 6, final)", "", doc["about"], "",
     "| Case | Old (in force for the second targeted run) | Decision | New |", "| --- | --- | --- | --- |"]
for r in rows:
    L.append(f"| {r['case']} | ({r['old']['source']}) {block(r['old'])} | {'; '.join(r['decisions'])} | {block(r['new'])} |")
L += ["", "## Dispositions without execution", "", "| Cases | Decision | Disposition |", "| --- | --- | --- |"]
for d in dispositions:
    L.append(f"| {', '.join(d['cases'])} | {'; '.join(d['decisions'])} | {d['disposition']} |")
(out / "amendments6-final.md").write_text("\n".join(L) + "\n")
print("ok")
