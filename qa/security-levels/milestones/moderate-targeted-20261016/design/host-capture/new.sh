#!/bin/bash
# new.sh NAME : fresh capture dir with fixture + product config
set -eu
W3=/Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261016
C=$W3/cap/$1; mkdir -p $C/{home,tmp,product-home,product-logs,fixture}
tar -xf /Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261016/h/fixture.tar -C $C/fixture
cp /Volumes/CorbanuDrive/Corbanu/.codex-work/moderate-targeted-20261016/h/product-config.toml $C/product-home/config.toml
echo $C
