## Windows sandbox: capability SIDs are recorded under a lock (#1871)

- **Two Corbanu processes on one Corbanu home no longer lose each other's
  sandbox capability SIDs** when they record them at the same time. Each
  update of the home's `cap_sid` file now holds a lock (`cap_sid.lock`) from
  reading the file to replacing it, and the sandbox audit no longer rewrites
  the file. Before, the later write could drop a SID the other process had
  just recorded and granted, and two first uses could each make a different
  set.
- An update waits at most 5 seconds for the lock, then goes ahead unlocked,
  as all updates did before. So a program that holds the lock (anything that
  can read the Corbanu home) can delay updates by 5 seconds and bring back
  the old race, but cannot stop them.
