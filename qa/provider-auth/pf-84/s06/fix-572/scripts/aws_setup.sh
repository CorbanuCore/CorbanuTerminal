# Source after env.sh. Builds a disposable HOME with fake AWS files (fake key ids only) and a
# Corbanu home on Bedrock pointed at the local mock. Usage: aws_setup <name> <port> [region-in-config: yes|no]
aws_setup() {
  local n=$1 port=$2 cfgregion=${3:-yes}
  export FH=$S/fh-$n H=$S/homes/$n
  mkdir -p "$FH/.aws/sso/cache" "$H"
  cat > "$FH/.aws/config" <<EOF
[default]
region = eu-west-1
[profile work]
region = us-west-2
[profile other]
region = eu-central-1
[profile noregion]
[profile envsrc]
role_arn = arn:aws:iam::111111111111:role/envsrc
credential_source = Environment
region = us-west-2
[profile imdssrc]
role_arn = arn:aws:iam::111111111111:role/imds
credential_source = Ec2InstanceMetadata
region = us-west-2
[profile ecssrc]
role_arn = arn:aws:iam::111111111111:role/ecs
credential_source = EcsContainer
region = us-west-2
[profile proc]
credential_process = $FH/proc.sh
region = us-west-2
[profile procfail]
credential_process = $FH/procfail.sh
region = us-west-2
[profile chain]
role_arn = arn:aws:iam::222222222222:role/chain
source_profile = work
region = us-west-2
services = mock
[profile sso]
sso_session = corp
sso_account_id = 333333333333
sso_role_name = Dev
region = us-west-2
services = mock
[profile ssonocache]
sso_session = other
sso_account_id = 333333333333
sso_role_name = Dev
region = us-west-2
[sso-session corp]
sso_start_url = https://example.awsapps.com/start
sso_region = us-east-1
[sso-session other]
sso_start_url = https://other.awsapps.com/start
sso_region = us-east-1
[services mock]
sts =
  endpoint_url = http://127.0.0.1:$port
sso =
  endpoint_url = http://127.0.0.1:$port
EOF
  cat > "$FH/.aws/credentials" <<EOF
[default]
aws_access_key_id = AKIDDEFAULTPROFILE
aws_secret_access_key = fakesecret-default-S06CANARY
[work]
aws_access_key_id = AKIDWORKPROFILE
aws_secret_access_key = fakesecret-work-S06CANARY
[other]
aws_access_key_id = AKIDOTHERPROFILE
aws_secret_access_key = fakesecret-other-S06CANARY
[noregion]
aws_access_key_id = AKIDNOREGION
aws_secret_access_key = fakesecret-noregion-S06CANARY
EOF
  printf '#!/bin/sh\necho %s\n' "'{\"Version\":1,\"AccessKeyId\":\"AKIDPROCESSHELPER\",\"SecretAccessKey\":\"fakesecret-proc-S06CANARY\"}'" > "$FH/proc.sh"
  printf '#!/bin/sh\necho "PROCSTDERR-S06CANARY helper failed" >&2\nexit 3\n' > "$FH/procfail.sh"
  chmod +x "$FH/proc.sh" "$FH/procfail.sh"
  # Valid cached SSO token for session `corp` (file name = sha1 of the session name); none for `other`.
  printf '{"accessToken":"fake-sso-access-S06CANARY","expiresAt":"2099-01-01T00:00:00Z","region":"us-east-1","startUrl":"https://example.awsapps.com/start"}' \
    > "$FH/.aws/sso/cache/$(printf corp | sha1sum | cut -d' ' -f1).json"
  {
    echo 'model = "openai.gpt-5.4"'
    echo 'model_provider = "amazon-bedrock"'
    echo '[features]'
    echo 'named_accounts = true'
    echo '[model_providers.amazon-bedrock]'
    echo "base_url = \"http://127.0.0.1:$port/openai/v1\""
    if [ "$cfgregion" = yes ]; then echo '[model_providers.amazon-bedrock.aws]'; echo 'region = "us-west-2"'; fi
    echo "[projects.\"$S/ws\"]"
    echo 'trust_level = "trusted"'
  } > "$H/config.toml"
}
# Decoys every named Bedrock account must ignore (fake values). Each fallback source points at the mock,
# which would answer with a distinctive key id (AKIDENVDECOY, AKIDIMDSFALLBACK, AKIDECSFALLBACK, ASIAROLECHAIN).
aws_decoys() {
  local port=$1
  XENV=(AWS_ACCESS_KEY_ID=AKIDENVDECOY AWS_SECRET_ACCESS_KEY=fakesecret-env-S06CANARY AWS_SESSION_TOKEN=fake-env-session
    AWS_PROFILE=other AWS_DEFAULT_PROFILE=other AWS_BEARER_TOKEN_BEDROCK=fakebearer-env-S06CANARY
    AWS_EC2_METADATA_SERVICE_ENDPOINT=http://127.0.0.1:$port AWS_CONTAINER_CREDENTIALS_FULL_URI=http://127.0.0.1:$port/ecs-creds
    AWS_ROLE_ARN=arn:aws:iam::444444444444:role/webid AWS_WEB_IDENTITY_TOKEN_FILE=$FH/webid.token
    AWS_ENDPOINT_URL_STS=http://127.0.0.1:$port)
  echo fake-webid-token > "$FH/webid.token"
}
