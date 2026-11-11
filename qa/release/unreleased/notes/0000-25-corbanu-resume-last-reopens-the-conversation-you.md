## `corbanu resume --last` reopens the conversation you quit from

- **`resume --last` now opens the conversation that was on screen when you
  last quit** (#1412, D243). Opening an older conversation with `/resume`
  doesn't change its "updated" time until you send a turn, so `resume --last`
  used to reopen the conversation with the newest turn instead. Corbanu now
  records the conversation on screen when you quit and reopens it, as long as
  it is in the same scope (folder, provider, not archived) and no conversation
  there got a newer turn after you quit. Otherwise it still picks the newest
  turn, as before. The `/resume` list order and `fork --last` are unchanged.
