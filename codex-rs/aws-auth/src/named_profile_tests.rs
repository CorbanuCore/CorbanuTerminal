use aws_config::profile::profile_file::ProfileFileKind;
use bytes::Bytes;
use http::HeaderMap;
use http::Method;
use pretty_assertions::assert_eq;

use super::*;
use crate::AwsRequestToSign;
use crate::signing::header_value;

const SERVICE: &str = "bedrock-mantle";

/// A `default` profile, a `work` profile with a region, a `noregion`
/// profile and two role profiles whose `credential_source` lies outside the
/// files, all with fake keys.
fn files() -> ProfileFiles {
    ProfileFiles::builder()
        .with_contents(
            ProfileFileKind::Config,
            "[default]\nregion = eu-west-1\n\
             [profile work]\nregion = us-west-2\n\
             [profile noregion]\n\
             [profile envsourced]\nregion = us-west-2\nrole_arn = arn:aws:iam::123456789012:role/demo\ncredential_source = Environment\n\
             [profile imdssourced]\nregion = us-west-2\nrole_arn = arn:aws:iam::123456789012:role/demo\ncredential_source = Ec2InstanceMetadata\n\
             [profile ecssourced]\nregion = us-west-2\nrole_arn = arn:aws:iam::123456789012:role/demo\ncredential_source = EcsContainer\n\
             [profile rolechain]\nrole_arn = arn:aws:iam::123456789012:role/demo\nsource_profile = noregion\n",
        )
        .with_contents(
            ProfileFileKind::Credentials,
            "[default]\naws_access_key_id = AKIDDEFAULTPROFILE\naws_secret_access_key = fake-default\n\
             [work]\naws_access_key_id = AKIDWORKPROFILE\naws_secret_access_key = fake-work\n\
             [noregion]\naws_access_key_id = AKIDNOREGION\naws_secret_access_key = fake-noregion\n",
        )
        .build()
}

/// The access key id the signature names.
async fn signing_key_id(context: &AwsAuthContext) -> String {
    let signed = context
        .sign(AwsRequestToSign {
            method: Method::POST,
            url: "https://bedrock-mantle.us-west-2.api.aws/openai/v1/responses".to_string(),
            headers: HeaderMap::new(),
            body: Bytes::from_static(b"{}"),
        })
        .await
        .expect("sign");
    let authorization =
        header_value(&signed.headers, http::header::AUTHORIZATION.as_str()).expect("signed");
    let credential = authorization
        .split("Credential=")
        .nth(1)
        .expect("credential scope");
    credential.split('/').next().unwrap_or_default().to_string()
}

#[tokio::test]
async fn named_profile_uses_only_that_profile() {
    let context = load_named_profile(
        "work",
        /*region_override*/ None,
        SERVICE,
        Some(files()),
    )
    .await
    .expect("work profile");
    assert_eq!(
        (context.region().to_string(), signing_key_id(&context).await),
        ("us-west-2".to_string(), "AKIDWORKPROFILE".to_string())
    );
    let overridden = load_named_profile(
        "work",
        Some(" eu-central-1 ".to_string()),
        SERVICE,
        Some(files()),
    )
    .await
    .expect("region override");
    assert_eq!(overridden.region(), "eu-central-1");
}

#[tokio::test]
async fn missing_profile_fails_without_falling_back_to_default() {
    let error = load_named_profile(
        "gone",
        /*region_override*/ None,
        SERVICE,
        Some(files()),
    )
    .await
    .expect_err("a missing profile never falls back");
    assert!(
        matches!(error, AwsAuthError::Credentials(_)),
        "{}",
        error.detailed_message()
    );
    assert!(
        error.detailed_message().contains("gone"),
        "{}",
        error.detailed_message()
    );
}

#[tokio::test]
async fn profile_without_region_is_refused() {
    let error = load_named_profile(
        "noregion",
        /*region_override*/ None,
        SERVICE,
        Some(files()),
    )
    .await
    .expect_err("no region");
    assert_eq!(
        error.to_string(),
        AwsAuthError::MissingProfileRegion("noregion".to_string()).to_string()
    );
    assert_eq!(
        named_profile_region("noregion", Some("us-east-1".to_string()), Some(files()))
            .await
            .expect("override"),
        "us-east-1"
    );
}

/// A profile's `credential_source` never reaches environment keys or
/// instance credentials for a named account.
#[tokio::test]
async fn credential_sources_outside_the_files_are_refused() {
    for (profile, source) in [
        ("envsourced", "Environment"),
        ("imdssourced", "Ec2InstanceMetadata"),
        ("ecssourced", "EcsContainer"),
    ] {
        let error = load_named_profile(
            profile,
            /*region_override*/ None,
            SERVICE,
            Some(files()),
        )
        .await
        .expect_err("refused credential source");
        assert_eq!(
            error.to_string(),
            AwsAuthError::RefusedCredentialSource(source.to_string()).to_string()
        );
    }
}

/// A credential helper's own failure text (which may echo anything) is not
/// put into user-facing messages.
#[test]
fn provider_failure_details_are_withheld() {
    let error = AwsAuthError::Credentials(CredentialsError::provider_error(
        "Error retrieving credentials from external process: exited with code 1. Stderr: \
         helper-secret-canary",
    ));
    let message = error.detailed_message();
    assert!(
        !message.contains("helper-secret-canary")
            && message.contains("exited with code 1")
            && message.contains("(helper output withheld)"),
        "{message}"
    );
}

/// A role chain without a region cannot reach STS; that is reported as the
/// missing region (no request is sent: endpoint resolution fails first).
#[tokio::test]
async fn role_chain_without_a_region_reports_the_region() {
    let error = load_named_profile(
        "rolechain",
        /*region_override*/ None,
        SERVICE,
        Some(files()),
    )
    .await
    .expect_err("no region for STS");
    assert_eq!(
        error.to_string(),
        AwsAuthError::MissingProfileRegion("rolechain".to_string()).to_string()
    );
}
