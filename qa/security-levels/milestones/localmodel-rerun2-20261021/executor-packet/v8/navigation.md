# Neutral navigation

This packet holds three signed-off case packets, copied byte for byte, and one amendment on top of them:

- `/opt/agms/packet/agg/`: the Aggressive packet (`design-cases.md`, `amendments-and-new-cases.md`, `navigation.md`,
  screens folders).
- `/opt/agms/packet/acc/`: the accounting acceptance packet (`design-cases.md`, `amendments.md`, `answers.md`,
  `navigation.md`, `screens/`).
- `/opt/agms/packet/mod/`: the Moderate packet (`design-cases.md`, `amendments.md`, `amendments-v3.md`,
  `amendments-v4.md`, `amendments-v5.md`, `navigation.md`, screens folders).
- `/opt/agms/packet/amendments-L2.md`: this re-run's amendment (the product's model is the owner's local model). It
  governs over every document of the three packets for the cases it names; everything it does not mention stays as in
  force there.
- `/opt/agms/packet/screens-v6/`: screens of the candidate under test on the local model (see its `README.md`).
- `/opt/agms/packet/design-cases.md`: the index of the frozen files with their SHA-256.

Inside a sub-packet, its own `navigation.md` and case texts name files as `/opt/agms/packet/<file>` or by folder name
(for example `screens/`, `new-screens-4/`, `screens-v5/`); read those as the same file or folder inside that
sub-packet (for example `/opt/agms/packet/agg/design-cases.md`). Each sub-packet's own precedence rule stays; then
`amendments-L2.md` governs over all of it. Use the cases' starting conditions, actions and expected results; preserve
any ambiguity. Run only the cases assigned to you, in the order given.

Each group starts from a fresh product profile and a fresh copy of the test project: you set up every starting
condition yourself in this run. To reach a command's approval prompt, ask the product's agent to run it (for example:
Run exactly this command: ...). Do not use the product's `!` shell escape unless a case asks for it.

Send prompt text and Enter as separate actions. Observe the live screen before choosing keys; use the menus, choices
and keys actually displayed. On an approval prompt a letter in brackets only highlights its option; Enter confirms it.
Never type text into an open approval prompt. Use only the synthetic fixture in your run directory. Record offered
cancellation, recovery and continuation paths without inventing missing controls.

Stop a case and record the exact observation on a credential or system password prompt, unexpected profile access,
missing required control, lost observation or exhausted budget. When the product's own model declines a request,
record its words verbatim; you may resubmit the same case wording unchanged once, and label it.

The Aggressive navigation's keystroke drivers (`agms-drive`) and second terminal (`agms-tui --term 2`) are not part of
this re-run; none of these cases uses them.
