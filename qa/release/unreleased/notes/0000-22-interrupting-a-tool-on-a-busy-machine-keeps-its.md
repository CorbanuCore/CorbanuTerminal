## Interrupting a tool on a busy machine keeps its elapsed time

- **The model now always sees how long an interrupted tool ran** (#1467,
  #1448). On a loaded machine, an interrupted turn could miss its 100 ms grace
  period. The interrupted tool's output (`Wall time: N seconds` /
  `aborted by user`) was then replaced by a bare `aborted`. That output is now
  recorded in this case too, timed to the moment of the interrupt.
- **A tool that had already finished keeps its real result** (#1536, D244).
  In the same rare case, a tool that completed just before the interrupt (for
  example a patch that was applied) was reported to the model as aborted. Its
  real output is now recorded instead.
