use super::*;

#[test]
fn canonical_labels_are_narrow_and_stable() {
    assert_eq!(
        GpuCredentialKind::ProviderApiKey {
            provider: "vast".to_string()
        }
        .canonical_label(),
        Ok("gpu/provider/vast/api-key".to_string())
    );
    assert_eq!(
        GpuCredentialKind::RentalEndpointToken {
            rental_id: "rental-123".to_string()
        }
        .canonical_label(),
        Ok("gpu/rental/rental-123/endpoint-token".to_string())
    );
}

#[test]
fn secret_debug_is_redacted() {
    let secret = SecretValue::new("provider-secret-value".to_string()).expect("valid secret");
    let debug = format!("{secret:?}");
    assert!(!debug.contains("provider-secret-value"));
    assert!(debug.contains("REDACTED"));
}

#[test]
fn unsafe_rental_ids_cannot_select_arbitrary_vault_labels() {
    assert!(matches!(
        GpuCredentialKind::RentalEndpointToken {
            rental_id: "../../provider/vast".to_string()
        }
        .canonical_label(),
        Err(GpuCredentialError::InvalidRentalId)
    ));
}
