#!/bin/bash
# Pane command: the real Z.AI key is fetched at use time and passed only through the environment.
S=/Volumes/CorbanuDrive/Corbanu/tmp/pf84fix; B=$1; H=$2
cd $S/work
ZAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME ~/.local/bin/corbanu vault auth-helper provider/zai_api_key)" \
  env -i HOME=$S/fakehome PATH=/usr/bin:/bin TERM=xterm-256color CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME=$H ZAI_API_KEY="$ZAI_API_KEY" \
  $B -c log_dir="\"$S/logs/live\"" -c tui.animations=false --no-alt-screen -m glm-5.3-flash -c model_provider="zai"
echo TUI_EXITED rc=$?; sleep 9999
