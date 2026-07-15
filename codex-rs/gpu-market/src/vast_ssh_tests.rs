use super::*;

#[test]
fn public_key_identity_ignores_comments_but_rejects_malformed_keys() {
    assert_eq!(
        ssh_public_key_identity("ssh-ed25519 AAAATEST workstation"),
        Some(("ssh-ed25519", "AAAATEST"))
    );
    assert_eq!(ssh_public_key_identity("not-a-key"), None);
    assert_eq!(ssh_public_key_identity("x509-cert AAAATEST"), None);
}
