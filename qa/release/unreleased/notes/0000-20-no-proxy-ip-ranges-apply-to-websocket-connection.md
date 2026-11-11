## `NO_PROXY` IP ranges apply to WebSocket connections

- **WebSocket connections now honour IP-range entries in `NO_PROXY`** (#722,
  D247), such as `172.16.0.0/12`. A connection to an IP address inside a
  listed range now goes direct, as HTTP requests already did, instead of
  through `HTTP_PROXY`/`HTTPS_PROXY`. Host names are matched by name only (they
  are not resolved to check ranges), as before.
