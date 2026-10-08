#!/bin/bash
# usage: rtxrun.sh <remote args for execrun.sh, pre-quoted>
K="$(~/.local/bin/corbanu vault auth-helper provider/zai_api_key)" bash -c 'printf "%s\n" "$K" | ssh <rtx-user>@<rtx-host> "$0"' "~/corbanu-rtx/pf60-indep-20261008/execrun.sh $*"
