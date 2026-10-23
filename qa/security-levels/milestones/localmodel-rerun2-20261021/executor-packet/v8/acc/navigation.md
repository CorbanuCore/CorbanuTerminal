# Neutral navigation

Your cases are in `/opt/agms/packet/design-cases.md` (the frozen design: questions, then cases ACC-01..ACC-34 by
group) and `/opt/agms/packet/amendments.md` (the independent review's amendments and new cases, applied before the
freeze). Where an amendment changes a case, the amendment governs; everything it doesn't mention stays as frozen.
`/opt/agms/packet/answers.md` holds the facts about this test environment the design relies on (how to leave `/side`,
the automatic-reviewer settings, the stand-in controls, the Moonshot limit). Screens a case mentions are text copies in
`/opt/agms/packet/screens/` (from earlier builds: layout only, not expected results). Use the cases' starting
conditions, actions and expected results; preserve any ambiguity. Run only the cases assigned to you, in the order
given.

Your group starts from a fresh product profile and a fresh copy of the test project; set up every starting condition
yourself. Cases in a group share the profile and continue from each other as the design says.

Send prompt text and Enter as separate actions (`agms-tui text '...'`, then `agms-tui key Enter`). After starting the
product, wait until its composer is shown (for example `agms-tui wait 'model:' 30`, then a few seconds) before typing,
or the first characters can be lost; check the screen shows exactly what you typed before pressing Enter. Observe the
live screen before choosing keys; use the menus, choices and keys actually displayed. Model replies can take 10-90
seconds; use `wait` with a generous timeout. `/cost` may need a moment after a reply: if a request you just made is
missing, use its `Refresh` entry or reopen `/cost` once before judging.

Record what you read on the `/cost` pages exactly (copy the lines you judge into your report). Recompute money
yourself from the tokens and rates shown and the figures in the packet; show the arithmetic.

Stop a case and record the exact observation on a credential or system password prompt, unexpected profile access,
missing required control, lost observation or exhausted budget. When the product's own model declines a request,
record its words verbatim; you may resubmit the same case wording unchanged once, and label it.

Real routes (`zai`, `openai`, `moonshot`) spend real money: send only the turns your cases ask for. Moonshot allows
one first turn per model in the whole run; do not retry a Moonshot turn.

Packet v2 navigation note (added 2026-10-16 after the first runs; navigation only, no expected results changed):
`/cost` pages are taller than the terminal. The screen shows a window of the page; Up/Down move the selection (the
line marked `›`) through every line and scroll the window. The entries you can open (`→ ...` links, `Refresh`,
`Close`) are near the bottom, under `—— Open ——`, below `—— Details ——`; reaching them can take 20-40 presses of
Down (`agms-tui key Down Down Down ...`). Enter opens the selected `→` line; Esc goes back one level, then closes.

