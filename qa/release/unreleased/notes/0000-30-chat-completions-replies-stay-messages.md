## Chat Completions: replies stay messages

- **A JSON answer that mentions `"function_call"` is a message** (#1582,
  #1581, thanks to @bradsbots). Such a reply used to be read as a broken tool
  call and fail with `malformed function-call JSON as assistant text`, then be
  retried, even after it had been shown. Only JSON whose top-level `type` is
  `function_call` or `custom_tool_call` counts as a call.
- **A reply that opens with such JSON and goes on in prose is a message**
  (#1659, #1654). It used to fail with `trailing characters` after five
  retries.
- **A reply that opens with JSON-like text that does not parse is a message**
  (#1728, #1660, thanks to @bradsbots), for example `{"function_call": auto} …`.
  It used to fail with `malformed function-call JSON` after five retries. A
  broken reply that names a tool-call `type` still counts as a broken call and
  is retried.
- **Text already shown stays a message** (#1656, #1608, thanks to
  @bradsbots), even if it turns out to parse as a tool call.
- **A `null` usage counter no longer drops its chunk** (#1589, #1588, thanks
  to @bradsbots). When that chunk was the last one, the reply lost its last
  text, its finish reason and its usage, so a reply cut off at the output limit
  ended as if it had finished normally. A `null` counter now reads as 0 for display.
