## Moderate: waiting on and starting agents after untrusted content

- **A turn stopped by automatic approval review no longer takes the next
  turn's input** (#2046). Guardian stops a turn in the background and waits
  briefly for it to finish. If you started a new turn in that moment, the
  stopping turn could take the new turn's steering messages and queued agent
  messages. Only the turn that is running now takes them.
