#!/bin/bash
# Host: stage the candidate package, harness and packet on the guest; place the campaign leaf and the two campaign
# transport keys (vault values piped straight to agent-only files, never printed or written on the host); then
# erase-etc-codex, setup-agent (as agent) and provision (as root). Logs -> common/.
set -euo pipefail
W2=/Volumes/CorbanuDrive/Corbanu/.codex-work/pf60-s10-20261016
S=$W2/h/vmssh.sh; G=agent@192.168.64.3
cd $W2
$S $G 'mkdir -p -m 700 ~/agms ~/agms/stage ~/agms/mediator; rm -R ~/agms/stage/package ~/agms/stage/packet ~/agms/stage/h 2>/dev/null; echo ok'
COPYFILE_DISABLE=1 tar --no-xattrs -C $W2/stage -cf - package package-files.txt packet | $S $G 'cd ~/agms/stage && tar -xf - && echo staged-package-packet'
items="h/agms-launch h/agms-probe h/agms-product h/agms-tui h/tmux.conf h/mediation-ca.pem h/product-config.toml h/fixture.tar h/expected.json h/acc_mediator.py h/acc_standin.py h/chatgpt_standin.py"
COPYFILE_DISABLE=1 tar --no-xattrs -czf - $items | $S $G 'cd ~/agms/stage && tar -xzf - && echo staged-harness'
$S $G 'umask 077; cat > ~/agms/mediator/leaf.pem' < $W2/ca/leaf.pem
$S $G 'umask 077; cat > ~/agms/mediator/leaf.key' < $W2/ca/leaf.key
OK="$(corbanu vault auth-helper openai-api-key)" sh -c 'printf "%s" "$OK"' | $S $G 'umask 077; mkdir -p ~/.pf83-auth; cat > ~/.pf83-auth/openai_api_key; test -s ~/.pf83-auth/openai_api_key && echo openai-key-placed'
KK="$(corbanu vault auth-helper kimi_api3)" sh -c 'printf "%s" "$KK"' | $S $G 'umask 077; cat > ~/.pf83-auth/kimi_api_key; test -s ~/.pf83-auth/kimi_api_key && echo kimi-key-placed'
NEOPW="$(corbanu vault auth-helper neo-vm)" $W2/h/admin.sh $W2/h/erase-etc-codex.sh > $W2/common/erase-etc-codex.out 2>&1
$S $G 'bash -s' < $W2/h/setup-agent.sh > $W2/common/setup-agent.out 2>&1
NEOPW="$(corbanu vault auth-helper neo-vm)" $W2/h/admin.sh $W2/h/provision.sh > $W2/common/provision-1.out 2>&1
NEOPW="$(corbanu vault auth-helper neo-vm)" $W2/h/admin.sh $W2/h/capture-policy.sh > $W2/common/policy-root-start.txt 2>&1
tail -3 $W2/common/setup-agent.out; tail -2 $W2/common/provision-1.out
