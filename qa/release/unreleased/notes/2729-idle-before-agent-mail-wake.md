## Fixes

- Agent messages: a finished turn now reports that the thread is idle before Corbanu checks queued agent mail, as upstream does. Queued user input and goal continuations can therefore run before a turn started only by agent mail; that mail is then delivered in their turn. Interrupt, the kill switch and plan mode still stop a pending agent-mail wake without losing its messages.
