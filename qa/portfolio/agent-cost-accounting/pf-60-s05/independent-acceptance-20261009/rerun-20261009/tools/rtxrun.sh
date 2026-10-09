#!/bin/bash
# local side. usage: rtxrun.sh <vault-label> <remote args for execrun.sh...>. The key is resolved by the installed wrapper
# (without test env vars) on the consuming command and piped over ssh stdin; the remote side holds it in the environment only.
V(){ env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper "$1" < /dev/null; }
lab=$1; shift
KEYVAL="$(V "$lab")" ARGS="$*" bash -c 'printf "%s\n" "$KEYVAL" | ssh -o BatchMode=yes <rtx-user>@<rtx-host> "bash pf60-s05-rerun/execrun.sh $ARGS"'
