# Next local-model run: case list

Cases waiting for the next code-blind run with the owner's local model as the product's model. That is the standard
way to test protections a hosted model refuses to attempt (D75). Earlier runs:
[first](moderate-localmodel-20261020/README.md) (#1078) and [second](localmodel-rerun2-20261021/README.md) (#1191).
Each run freezes its own packet. This list says only what goes into it.

| Case | Added by | Case text | Notes |
| --- | --- | --- | --- |
| AGG-20P-A, AGG-20P-B (blocker) | D111 (owner, 2026-10-21) | [amendment 7](aggressive-amendment7-20261021/design/amendment7-final.md), "AGG-20P" | A custom role a person defines, with more access than Aggressive, is refused at start. A control role that sets nothing is accepted. Two single-case groups. The run's local-model rules apply, as L-1..L-5 of amendment L2. Before the freeze, capture the screens the amendment lists. If AGG-20P passes, the owner decides whether AGG-20's result 2 (named limit, #447) counts as exercised. |

For every packet of that run:
- add the waiting rule of [tester-packet-rules.md](tester-packet-rules.md) (D114: check about every 30 s during a long
  turn);
- give AGG-R07, if it runs again, amendment 7 on top of L2 (D110).
