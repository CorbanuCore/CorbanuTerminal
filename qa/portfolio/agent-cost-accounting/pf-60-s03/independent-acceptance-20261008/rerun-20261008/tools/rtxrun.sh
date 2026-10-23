#!/bin/bash
# local side. usage: rtxrun.sh '<pre-quoted args for rtx-execrun.sh>'; key piped over ssh stdin
printf '%s\n' "$(~/.local/bin/corbanu vault auth-helper provider/zai_api_key)" | ssh <rtx-user>@<rtx-host> "<rtx-dir>/execrun.sh $*"
