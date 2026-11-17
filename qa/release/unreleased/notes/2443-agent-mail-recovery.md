## Fixes

- Save pending input and agent mail after terminal turn errors without retrying forever. Unanswered mail waits for the next turn, idle children can release capacity without losing queued mail, and Interrupt cancels message wakes during sign-in discovery.
