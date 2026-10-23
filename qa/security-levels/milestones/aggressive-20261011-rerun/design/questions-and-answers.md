# Re-run designer questions and coordinator answers

Answers come from the owner decisions, the public product docs (`docs/features/workspaces.md`) and the test
environment, not from implementation. Where there is no further requirement the designer's own assumption stands.

| Q | Topic | Answer |
| --- | --- | --- |
| 1 | Kill switch while an approval prompt is open | No further requirement. Assumption stands (a second pane if the product allows reaching it; otherwise AGG-R03 is blocked). |
| 2 | Opening a second Codex pane | Product docs: `/panes` opens the pane picker; "+ Corbanu Terminal Pane" creates a native pane (pick a model and name); selecting a row switches to that pane. |
| 3 | Claude panes under Aggressive | Claude Code is not installed in the test machine (`claude-availability.txt`): AGG-R06 is recorded as blocked. |
| 4 | Interrupt speed, partial text | No further requirement than "interrupts". Assumption stands. |
| 5 | Credential files created between confirm and restart | No further requirement. Assumption stands (advisory). |
| 6 | Where the Corbanu home is | Test environment: the product's home is `$AGMS_RUN/product-home`, which the product sees as `$CORBANU_HOME`. |
| 7 | Built-in file tools listing the home | Decision #506 says listing the home is denied. Assumption stands. |
| 8 | Nested `.codex`/`.agents`, links | Decision: "the project's `.codex` and `.agents`". Assumption stands (root folders; links advisory). |
| 9 | Child agents as "every pane" | Decision: "interrupts the current turn in every pane". Assumption stands (advisory). |

Test-environment facts for executors (neutral navigation, not expectations):
- The relay log is not visible to testers. The coordinator keeps each run's relay window (`mediator-window.jsonl`);
  expected results about relay traffic are checked from it by the evidence check.
- The product's `~` is the tester account's home, which is wiped after each run. Reading or backing up files there
  into `$AGMS_RUN` is allowed.
