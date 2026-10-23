#!/usr/bin/env bash
# Slice A, second pass: no aws.region in config, AWS_REGION decoy, AWS_CONFIG_FILE/AWS_SHARED_CREDENTIALS_FILE,
# [provider_accounts] selection (incl. a missing one), a dash-leading profile name. Output: captures/aws-cli2.txt
set -u
D=$(cd "$(dirname "$0")" && pwd); source "$D/env.sh"; source "$D/aws_setup.sh"
P=34612; LOG=$S/logs/mock-a2.jsonl; : > "$LOG"
python3 "$D/mock.py" --port $P --log "$LOG" --bearer envbearer=fakebearer-env-S06CANARY &
MOCK=$!; trap 'kill $MOCK' EXIT; sleep 1
[ -d $S/homes/a2 ] && rm -r $S/homes/a2 $S/fh-a2 < /dev/null; aws_setup a2 $P no
OUT=$EV/captures/aws-cli2.txt
ex() { show run $CAND $H exec --skip-git-repo-check -C $S/ws "$@" "Reply with exactly: pong" < /dev/null; }
cnt() { local b=$(mocklines $LOG); ex "$@"; echo "-- requests during this run: $(( $(mocklines $LOG) - b ))"; }
{
echo "## home a2: config has NO model_providers.amazon-bedrock.aws.region"
XENV=()
for p in work:work alt:other noreg:noregion; do show run $CAND $H account add amazon-bedrock ${p%%:*} --kind aws-profile --value ${p#*:}; done
show run $CAND $H account add amazon-bedrock dash --kind aws-profile --value=-leadingdash
aws_decoys $P; XENV+=(AWS_REGION=ap-south-1 AWS_DEFAULT_REGION=ap-south-1)
echo; echo "## region from the profile (decoy AWS_REGION=ap-south-1 set)"
cnt --account alt
cnt --account work
echo; echo "## profile without a region and no config region: refused before any request"
cnt --account noreg
echo; echo "## AWS_CONFIG_FILE / AWS_SHARED_CREDENTIALS_FILE name other files"
mkdir -p $FH/alt-aws
printf '[profile work]\nregion = sa-east-1\n' > $FH/alt-aws/config
printf '[work]\naws_access_key_id = AKIDALTFILEWORK\naws_secret_access_key = fakesecret-altfile-S06CANARY\n' > $FH/alt-aws/credentials
XENV+=(AWS_CONFIG_FILE=$FH/alt-aws/config AWS_SHARED_CREDENTIALS_FILE=$FH/alt-aws/credentials)
cnt --account work
aws_decoys $P
echo; echo "## [provider_accounts] selection: amazon-bedrock = \"alt\" (no --account)"
cnt -c 'provider_accounts.amazon-bedrock="alt"'
echo; echo "## [provider_accounts] amazon-bedrock = \"ghost\": fail closed"
cnt -c 'provider_accounts.amazon-bedrock="ghost"'
echo; echo "## --account amazon-bedrock:work form"
cnt --account amazon-bedrock:work
echo; echo "## mock log"
sed "s#$S#\$S#g" "$LOG"
} > "$OUT" 2>&1
echo done
