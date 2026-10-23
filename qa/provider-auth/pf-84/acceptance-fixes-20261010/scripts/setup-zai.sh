#!/bin/bash
# Disposable homes for #416/#417/#419.1. Real key only via the vault helper and stdin.
S=<scratch>
for h in hz hn hw; do rm -r $S/$h 2>/dev/null </dev/null; mkdir -p $S/$h; done
cfg() { printf 'model = "glm-5.3-flash"\nmodel_provider = "zai"\n\n[features]\nnamed_accounts = %s\n\n[projects."<scratch>/work"]\ntrust_level = "trusted"\n' "$2" > $S/$1/config.toml; }
cfg hz true; cfg hn true; cfg hw false
A="env -i HOME=$S/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1"
printf 'pf84fix-fake-zai-key-not-real' | $A CORBANU_HOME=$S/hz $S/bin/corbanu-after account add zai fake
ZAI_API_KEY="$(env -u CORBANU_TEST_NO_NATIVE_KEYRING -u CORBANU_HOME -u CODEX_HOME -u PFTERMINAL_HOME /Users/Neo/.local/bin/corbanu vault auth-helper provider/zai_api_key)" bash -c 'printf %s "$ZAI_API_KEY" | env -i HOME='$S'/fakehome PATH=/usr/bin:/bin CORBANU_TEST_NO_NATIVE_KEYRING=1 CORBANU_HOME='$S'/hn '$S'/bin/corbanu-after account add zai main'
for h in hz hn; do $A CORBANU_HOME=$S/$h $S/bin/corbanu-after account list; done
ls $S/hn/secrets 2>/dev/null
