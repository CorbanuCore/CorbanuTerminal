use super::*;
use pretty_assertions::assert_eq;

fn epoch(revision: u64) -> AuthorityEpoch {
    AuthorityEpoch::new([1; 16], revision, 0).unwrap()
}
fn destination() -> AbsolutePathBuf {
    AbsolutePathBuf::from_absolute_path_checked(std::env::temp_dir().join("explicit-download.txt"))
        .unwrap()
}

#[test]
fn promotion_requires_exact_host_approval_and_consumes_artifact() {
    let artifact = QuarantinedArtifact::new(b"untrusted".to_vec(), epoch(1));
    let expected_digest = artifact.sha256().to_owned();
    let promoted = artifact
        .promote(destination(), epoch(1), |request| {
            assert_eq!(
                (&request.sha256, request.byte_length, request.epoch),
                (&expected_digest, 9, epoch(1))
            );
            assert_eq!(request.destination, destination());
            Ok(())
        })
        .unwrap();
    assert_eq!(
        (promoted.destination, promoted.bytes),
        (destination(), b"untrusted".to_vec())
    );
}

#[test]
fn denial_and_stale_epoch_never_release_download_bytes() {
    let denied =
        QuarantinedArtifact::new(vec![1], epoch(1)).promote(destination(), epoch(1), |_| {
            Err(BrowserError::PromotionDenied)
        });
    assert!(matches!(denied, Err(BrowserError::PromotionDenied)));
    let mut called = false;
    let stale =
        QuarantinedArtifact::new(vec![1], epoch(1)).promote(destination(), epoch(2), |_| {
            called = true;
            Ok(())
        });
    assert!(matches!(stale, Err(BrowserError::StaleAuthority)));
    assert!(!called);
}
