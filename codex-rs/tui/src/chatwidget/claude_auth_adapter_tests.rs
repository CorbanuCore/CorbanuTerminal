use codex_model_provider_info::built_in_model_providers;
use codex_provider_auth::ConfiguredAvailability;
use codex_provider_auth::CredentialControl;
use codex_provider_auth::ProviderAuthController;
use codex_provider_auth::ProviderAuthEffect;
use codex_provider_auth::ProviderAuthFlowSnapshot;
use codex_provider_auth::ProviderAvailabilityState;
use codex_provider_auth::ProviderCatalog;
use codex_provider_auth::ProviderConfigurationState;
use codex_provider_auth::ProviderCredentialSource;
use codex_provider_auth::ProviderCurrentState;
use codex_provider_auth::ProviderEligibilityState;
use codex_provider_auth::ProviderMethodState;
use codex_provider_auth::ProviderMethodStatus;
use codex_provider_auth::ProviderSetupCapability;
use codex_provider_auth::ProviderStatusSnapshot;
use codex_provider_auth::claude_account_flow::*;
use pretty_assertions::assert_eq;

use super::*;

#[test]
fn backend_results_collapse_without_retaining_raw_error_text() {
    let canary = "RAW-BACKEND-ERROR-CANARY";
    assert_eq!(
        collapse_backend_result(Some(Err(ClaudeCodeLoginBackendError::Other(
            canary.to_string(),
        )))),
        ClaudeCodeLoginOutcome::Rejected
    );
    assert_eq!(
        collapse_backend_result(Some(Err(ClaudeCodeLoginBackendError::IdentityConflict))),
        ClaudeCodeLoginOutcome::IdentityConflict
    );
    assert_eq!(
        collapse_backend_result(Some(Err(ClaudeCodeLoginBackendError::TimedOut))),
        ClaudeCodeLoginOutcome::TimedOut
    );
    assert_eq!(
        collapse_backend_result(None),
        ClaudeCodeLoginOutcome::Cancelled
    );
    assert!(!format!("{:?}", ClaudeCodeLoginOutcome::Rejected).contains(canary));
}

#[tokio::test]
async fn authorization_code_effect_is_redacted_and_consumed_by_exact_process() {
    let (attempt_id, process_id, target) = code_process_ids();
    let adapter = ClaudeAuthAdapter::new(tempfile::tempdir().unwrap().path().to_path_buf());
    let (input_tx, mut input_rx) = mpsc::unbounded_channel();
    begin_process(&adapter.processes, (attempt_id, process_id));
    register_start_returned(&adapter.processes, (attempt_id, process_id), input_tx);
    let (action_tx, _action_rx) = mpsc::unbounded_channel();
    let canary = "AUTHORIZATION-CODE-CANARY";
    let effect = ClaudeAccountEffect::SendAuthorizationCode {
        attempt_id,
        process_id,
        secret: ClaudeAuthorizationCodeSecret::new(canary),
    };
    assert!(!format!("{effect:?}").contains(canary));
    assert_eq!(adapter.execute(effect, action_tx), Ok(()));
    match input_rx.recv().await {
        Some(ClaudeCodeLoginInput::AuthorizationCode(code)) => {
            assert_eq!(code.as_str(), canary)
        }
        _ => panic!("authorization code was not sent to the exact process"),
    }

    adapter
        .processes
        .lock()
        .unwrap()
        .remove(&(attempt_id, process_id));
    let (action_tx, _action_rx) = mpsc::unbounded_channel();
    assert_eq!(
        adapter.execute(
            ClaudeAccountEffect::CancelClaudeCodeLogin {
                attempt_id,
                process_id,
            },
            action_tx,
        ),
        Err(ClaudeAuthAdapterError::MissingProcess)
    );
    drop(target);
}

#[tokio::test]
async fn ready_before_start_return_registers_exact_sender_without_overwrite() {
    let (attempt_id, process_id, target) = code_process_ids();
    let adapter = ClaudeAuthAdapter::new(tempfile::tempdir().unwrap().path().to_path_buf());
    let key = (attempt_id, process_id);
    begin_process(&adapter.processes, key);
    let (ready_tx, mut ready_rx) = mpsc::unbounded_channel();
    let (returned_tx, mut returned_rx) = mpsc::unbounded_channel();

    assert!(register_ready(&adapter.processes, key, ready_tx));
    register_start_returned(&adapter.processes, key, returned_tx);
    assert_eq!(
        adapter.send_process_input(attempt_id, process_id, ClaudeCodeLoginInput::Cancel),
        Ok(())
    );
    assert!(matches!(
        ready_rx.recv().await,
        Some(ClaudeCodeLoginInput::Cancel)
    ));
    assert!(returned_rx.try_recv().is_err());
    drop(target);
}

#[test]
fn finished_before_start_return_cannot_be_followed_by_stale_insertion() {
    let (attempt_id, process_id, target) = code_process_ids();
    let adapter = ClaudeAuthAdapter::new(tempfile::tempdir().unwrap().path().to_path_buf());
    let key = (attempt_id, process_id);
    begin_process(&adapter.processes, key);
    finish_process(&adapter.processes, key);
    let (returned_tx, _returned_rx) = mpsc::unbounded_channel();
    register_start_returned(&adapter.processes, key, returned_tx);

    assert!(!adapter.processes.lock().unwrap().contains_key(&key));
    assert_eq!(
        adapter.send_process_input(attempt_id, process_id, ClaudeCodeLoginInput::Cancel),
        Err(ClaudeAuthAdapterError::MissingProcess)
    );
    drop(target);
}

#[test]
fn renderer_owned_effects_are_not_executed_by_backend_adapter() {
    let (attempt_id, process_id, target) = code_process_ids();
    let adapter = ClaudeAuthAdapter::new(tempfile::tempdir().unwrap().path().to_path_buf());
    let (action_tx, _action_rx) = mpsc::unbounded_channel();
    assert_eq!(
        adapter.execute(
            ClaudeAccountEffect::PresentChallenge {
                attempt_id,
                process_id,
                challenge: ClaudeCodeChallenge::new("https://challenge.example/canary"),
            },
            action_tx,
        ),
        Err(ClaudeAuthAdapterError::UnsupportedEffect)
    );
    drop(target);
}

fn code_process_ids() -> (
    codex_provider_auth::ProviderAuthAttemptId,
    ClaudeCodeProcessId,
    ClaudeAccountTarget,
) {
    let target = target();
    let mut controller = ProviderAuthController::default();
    controller.dispatch(
        ClaudeAccountAction::Start(ClaudeAccountFlowStart {
            target: target.clone(),
            intent: ClaudeAccountIntent::Replace,
            status: status(ProviderCredentialSource::ClaudeManaged),
        })
        .into(),
    );
    controller
        .dispatch(ClaudeAccountAction::ChooseMethod(ClaudeAccountMethod::ClaudeCodeLogin).into());
    let transition = controller.dispatch(ClaudeAccountAction::Submit.into());
    let ProviderAuthFlowSnapshot::ClaudeAccount(ClaudeAccountSnapshot::StartingClaudeCodeLogin {
        attempt_id,
        process_id,
        ..
    }) = transition.snapshot
    else {
        panic!("unexpected controller state: {:?}", transition.snapshot);
    };
    assert!(matches!(
        transition.effects.as_slice(),
        [ProviderAuthEffect::ClaudeAccount(
            ClaudeAccountEffect::StartClaudeCodeLogin { .. }
        )]
    ));
    (attempt_id, process_id, target)
}

fn status(source: ProviderCredentialSource) -> ProviderStatusSnapshot {
    ProviderStatusSnapshot {
        id: target().provider_id,
        methods: vec![ProviderMethodStatus {
            capability: ProviderSetupCapability::ClaudeAccount,
            state: ProviderMethodState::Configured {
                source,
                control: CredentialControl::ManagedByCorbanu,
                availability: ConfiguredAvailability::Ready,
            },
        }],
        configuration: ProviderConfigurationState::Configured,
        eligibility: ProviderEligibilityState::Active,
        current: ProviderCurrentState::NotCurrent,
        availability: ProviderAvailabilityState::Ready,
    }
}

fn target() -> ClaudeAccountTarget {
    let catalog = ProviderCatalog::from_runtime_providers(&built_in_model_providers(None));
    ClaudeAccountTarget::from_catalog_entry(
        catalog.get("claude-plan").expect("Claude catalog entry"),
    )
    .expect("Claude account target")
}
