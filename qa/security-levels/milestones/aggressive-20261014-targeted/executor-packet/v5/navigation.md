# Neutral navigation

Read your assigned case headings in `/opt/agms/packet/design-cases.md` (frozen cases) and in
`/opt/agms/packet/amendments-and-new-cases.md` (amendments to some frozen cases, in three rounds, and new cases
`AGG-R..` and `AGG-T..`). For an amended case, the latest amendment governs where it differs from earlier text: the
2026-10-14 amendment (part 5) over the 2026-10-13 one (part 3), and that over the 2026-10-11 one (part 1); everything
they do not mention stays as before, or as frozen. Cases that mention a screen by number (for example "screen 03" or
"E1") mean the design-time screenshot whose text copy is in `/opt/agms/packet/screens/`; `new-screens/`,
`new-screens-3/` and `new-screens-4/` hold screens of later candidates. Use their starting conditions, actions and
expected results; preserve any ambiguity. Run only the cases assigned to you, in the order given.

Each group starts from a fresh product profile and a fresh copy of the test project: you set up every starting
condition yourself in this run (S0, S1 and any state a case describes). To reach a command's approval prompt, ask the
product's agent to run it (for example: Run exactly this command: ...). Do not use the product's `!` shell escape
unless a case asks for it.

Send prompt text and Enter as separate actions. Observe the live screen before choosing keys; use the menus,
choices and keys actually displayed. On an approval prompt a letter in brackets only highlights its option; Enter
confirms it. Never type text into an open approval prompt. Use only the synthetic fixture in your run directory.
Record offered cancellation, recovery and continuation paths without inventing missing controls.

Stop a case and record the exact observation on a credential or system password prompt, unexpected profile
access, missing required control, lost observation or exhausted budget. When the product's own model declines a
request, record its words verbatim; you may resubmit the same case wording unchanged once, and label it.
