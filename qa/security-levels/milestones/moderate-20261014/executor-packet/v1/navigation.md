# Neutral navigation

Read your assigned case headings in `/opt/agms/packet/design-cases.md` (frozen cases) and in
`/opt/agms/packet/amendments.md` (rules for every case, named limits, amendments to some cases, one new case MOD-34,
and the coordinator's answers and test-machine facts). For an amended case, the amendment governs where it differs
from the frozen text; everything it does not mention stays as frozen. Cases that mention a screen by number (for
example "screen 13a" or "E2b") mean the design-time screen whose text copy is in `/opt/agms/packet/screens/`
(`README.md` there says what each shows). Use their starting conditions, actions and expected results; preserve any
ambiguity. Run only the cases assigned to you, in the order given.

Each group starts from a fresh product profile and a fresh copy of the test project: you set up every starting
condition yourself in this run ("Moderate (restarted)" and any state a case describes). To reach a command's
approval prompt, ask the product's agent to run it (for example: Run exactly this command: ...). Do not use the
product's `!` shell escape unless a case asks for it.

Send prompt text and Enter as separate actions. Observe the live screen before choosing keys; use the menus,
choices and keys actually displayed. On an approval prompt a letter in brackets only highlights its option; Enter
confirms it. Never type text into an open approval prompt. Use only the synthetic fixture in your run directory.
Record offered cancellation, recovery and continuation paths without inventing missing controls.

Stop a case and record the exact observation on a credential or system password prompt, unexpected profile
access, missing required control, lost observation or exhausted budget. When the product's own model declines a
request, record its words verbatim; you may resubmit the same case wording unchanged once, and label it.
