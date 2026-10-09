#!/bin/bash
# local side. usage: rtxrun.sh '<pre-quoted args for rtx-execrun.sh>'. The key substitution is attached to the consuming
# command (a bash that pipes it over ssh stdin); the remote side reads it into the process environment only.
ZAI_API_KEY="$(~/.local/bin/corbanu vault auth-helper provider/zai_api_key)" ARGS="$*" \
  bash -c 'printf "%s\n" "$ZAI_API_KEY" | ssh <rtx-host> "<rtx-dir>/execrun.sh $ARGS"'
