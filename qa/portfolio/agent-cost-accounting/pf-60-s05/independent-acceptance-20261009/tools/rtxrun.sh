#!/bin/bash
# local side. usage: rtxrun.sh <vault-label> '<pre-quoted args for rtx-execrun.sh>'. The vault substitution is attached to
# the consuming command (a bash that pipes it over ssh stdin); the remote side reads it into the process environment only.
lab=$1; shift
KEYVAL="$(~/.local/bin/corbanu vault auth-helper "$lab")" ARGS="$*" \
  bash -c 'printf "%s\n" "$KEYVAL" | ssh <rtx-host> "bash pf60-s05-indep/rtx-execrun.sh $ARGS"'
