#!/bin/bash
# Local packet-screen capture (integrator only; never a test case). Usage: product.sh CAPDIR ROUTE [ARGS...]
# Disposable HOME/CORBANU_HOME, no native keyring; real Z.AI key resolved only at use time.
set -u
C="$1"; route="$2"; shift 2
PKG=/Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261016/candidate/package/bin/corbanu
cd "$C/fixture/workspace" || exit 66
exec env -i PATH=/usr/bin:/bin:/usr/sbin:/sbin HOME="$C/home" USER="$USER" LOGNAME="$USER" SHELL=/bin/bash \
  TERM=screen-256color LANG=en_US.UTF-8 TMPDIR="$C/tmp/" CORBANU_HOME="$C/product-home" CODEX_HOME="$C/product-home" \
  PFTERMINAL_HOME="$C/product-home" CORBANU_TEST_NO_NATIVE_KEYRING=1 \
  /bin/bash -c 'ZAI_API_KEY="$(/usr/bin/env -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME -u CORBANU_TEST_NO_NATIVE_KEYRING HOME=/Users/Neo /Users/Neo/.local/bin/corbanu vault auth-helper provider/zai_api_key)" exec "$0" --no-alt-screen -m glm-5.3-flash -c "model_provider=\"$1\"" -c "log_dir=\"$2/product-logs\"" -C "$2/fixture/workspace" "${@:3}"' \
  "$PKG" "$route" "$C" "$@"
