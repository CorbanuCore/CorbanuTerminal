#!/usr/bin/env bash
# Slice A (AWS profile accounts) from the CLI. Disposable HOME with fake ~/.aws, local mock Bedrock, decoy
# AWS credentials of every other kind in the environment. Output: $EV/captures/aws-cli.txt
set -u
D=$(cd "$(dirname "$0")" && pwd); source "$D/env.sh"; source "$D/aws_setup.sh"
P=34611; LOG=$S/logs/mock-a.jsonl; : > "$LOG"
python3 "$D/mock.py" --port $P --log "$LOG" --bearer envbearer=fakebearer-env-S06CANARY --bearer managed=fakemanaged-S06CANARY &
MOCK=$!; trap 'kill $MOCK' EXIT; sleep 1
[ -d $S/homes/a1 ] && rm -r $S/homes/a1 $S/fh-a1 < /dev/null; aws_setup a1 $P yes
OUT=$EV/captures/aws-cli.txt
ex() { show run $CAND $H exec --skip-git-repo-check -C $S/ws "$@" "Reply with exactly: pong" < /dev/null; }
mark() { echo "-- mock lines so far: $(mocklines $LOG)"; }
sha() { shasum -a 256 "$FH/.aws/config" "$FH/.aws/credentials" | sed "s#$S#\$S#g"; }
{
echo "## candidate $(git -C $EV rev-parse --short=10 HEAD 2>/dev/null) bin=\$CAND; HOME=\$S/fh-a1; CORBANU_HOME=\$S/homes/a1"
sha
echo; echo "## A1 add / validation / list / rename (no decoys yet)"
XENV=()
show run $CAND $H account add amazon-bedrock work --kind aws-profile --value work
show run $CAND $H account add amazon-bedrock alt --kind aws-profile --value other
show run $CAND $H account add amazon-bedrock bad1 --kind aws-profile --value "not a profile"
show run $CAND $H account add amazon-bedrock bad2 --kind aws-profile --value "-leadingdash"
show run $CAND $H account add amazon-bedrock bad3 --kind aws-profile --value "$(printf 'a%.0s' {1..65})"
show run $CAND $H account add amazon-bedrock bad4 --kind aws-profile
echo "(stdin-only attempt: secret-looking value on stdin, no --value)"
printf 'AKIDSTDIN fakesecret-stdin-S06CANARY' | show run $CAND $H account add amazon-bedrock bad5 --kind aws-profile
echo "(api-key kind for Bedrock: should it be accepted?)"
printf 'fakebedrock-apikey-S06CANARY' | show run $CAND $H account add amazon-bedrock keyacct --kind api-key
for p in gone:nosuchprofile noreg:noregion envsrc:envsrc imds:imdssrc ecs:ecssrc proc:proc procfail:procfail chain:chain sso:sso ssonc:ssonocache spare:work; do
  show run $CAND $H account add amazon-bedrock ${p%%:*} --kind aws-profile --value ${p#*:}
done
show run $CAND $H account rename amazon-bedrock spare renamed
show run $CAND $H account list amazon-bedrock
show run $CAND $H account list
echo "(stored value of work's label, read back with the candidate's own vault helper: a profile name only)"
show run $CAND $H vault auth-helper provider/amazon-bedrock/accounts/work/aws_profile
echo; echo "## A2 signing: decoys set (env keys, AWS_PROFILE=other, bearer env, IMDS/ECS/STS/web-identity at the mock)"
aws_decoys $P; mark
ex --account work
ex --account alt
ex --account renamed
ex --account proc
mark
echo; echo "## A2 role chain and SSO (endpoints overridden only through the profile's services section)"
ex --account chain
ex --account sso
mark
echo; echo "## A7 default account (today's chain) with the same decoys: bearer env expected to win"
ex
echo "(default without the bearer env)"
XENV=("${XENV[@]/AWS_BEARER_TOKEN_BEDROCK=*/AWS_X=1}"); ex; aws_decoys $P
echo "(--account default explicit)"
ex --account default
mark
echo; echo "## A3 refusals: each must send no request to the mock (counted per run)"
for a in gone noreg envsrc imds ecs procfail ssonc ghost keyacct; do
  before=$(mocklines $LOG); ex --account $a; echo "-- requests during --account $a: $(( $(mocklines $LOG) - before ))"
done
echo; echo "## A5 region: config aws.region (us-west-2) wins over the profile's (alt: eu-central-1). See alt above."
echo "(override per run: -c model_providers.amazon-bedrock.aws.region=ap-northeast-1)"
ex --account alt -c 'model_providers.amazon-bedrock.aws.region="ap-northeast-1"'
echo; echo "## flag off"
before=$(mocklines $LOG)
ex --disable named_accounts --account work
show run $CAND $H account list --disable named_accounts
echo "-- requests during flag-off: $(( $(mocklines $LOG) - before ))"
echo; echo "## remove"
show run $CAND $H account remove amazon-bedrock renamed
show run $CAND $H account list amazon-bedrock
sha
echo; echo "## mock log (fallback sources: aws-imds/aws-ecs/aws-sts/aws-sso lines)"
sed "s#$S#\$S#g" "$LOG"
} > "$OUT" 2>&1
echo done
