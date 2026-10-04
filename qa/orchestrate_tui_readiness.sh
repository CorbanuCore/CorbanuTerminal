#!/usr/bin/env bash

# Read only the live viewport, not scrollback: an earlier footer cannot prove
# that the current pane has reached its composer with the intended QA provider.
# The one-line footer must end the viewport after the latest prompt; trailing
# dialog text or a different provider footer must not reuse an earlier match.
orchestrate_composer_ready() {
  awk '
    /[^[:space:]]/ { last_nonblank = NR }
    /^[[:space:]]*›[[:space:]]/ { prompt = NR; footer = 0 }
    /^[[:space:]]*qa-model via qa default[[:space:]]+·/ {
      if (prompt) footer = NR
    }
    END { exit !(footer > prompt && prompt > 0 && footer == last_nonblank) }
  '
}
