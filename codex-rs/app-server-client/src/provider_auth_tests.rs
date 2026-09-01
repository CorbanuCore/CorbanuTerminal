use std::io::Error as IoError;
use std::io::ErrorKind;

use codex_app_server_protocol::JSONRPCErrorError;
use codex_model_provider_info::OPENAI_PROVIDER_ID;
use codex_model_provider_info::built_in_model_providers;
use codex_provider_auth::ConfiguredAvailability;
use codex_provider_auth::CredentialControl;
use codex_provider_auth::ProviderAuthController;
use codex_provider_auth::ProviderAuthEffect;
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
use pretty_assertions::assert_eq;

use super::*;

#[test]
fn request_mapping_freezes_browser_ordinary_and_device_contexts() {
    let cases = [
        (
            OpenAiAccountMethod::Browser,
            OpenAiAccountLoginContext::PrimaryAuth,
            LoginAccountParams::Chatgpt {
                codex_streamlined_login: false,
                use_hosted_login_success_page: false,
                app_brand: None,
            },
        ),
        (
            OpenAiAccountMethod::DeviceCode,
            OpenAiAccountLoginContext::PrimaryAuth,
            LoginAccountParams::ChatgptDeviceCode,
        ),
        (
            OpenAiAccountMethod::DeviceCode,
            OpenAiAccountLoginContext::ProviderEnrollment,
            LoginAccountParams::OpenaiProviderDeviceCode,
        ),
    ];
    for (index, (method, context, expected_params)) in cases.into_iter().enumerate() {
        let mut adapter = OpenAiAccountAppServerAdapter::default();
        let effect = start_effect(method, context);
        assert_eq!(
            adapter.request_for_effect(RequestId::Integer(index as i64), &effect),
            Ok(ClientRequest::LoginAccount {
                request_id: RequestId::Integer(index as i64),
                params: expected_params,
            })
        );
    }
}

#[test]
fn browser_provider_enrollment_is_rejected_even_for_manually_constructed_effect() {
    let mut adapter = OpenAiAccountAppServerAdapter::default();
    let effect = start_effect(
        OpenAiAccountMethod::Browser,
        OpenAiAccountLoginContext::PrimaryAuth,
    );
    let OpenAiAccountEffect::StartLogin {
        attempt_id, target, ..
    } = effect
    else {
        unreachable!()
    };
    assert_eq!(
        adapter.request_for_effect(
            RequestId::Integer(1),
            &OpenAiAccountEffect::StartLogin {
                attempt_id,
                target,
                method: OpenAiAccountMethod::Browser,
                context: OpenAiAccountLoginContext::ProviderEnrollment,
            },
        ),
        Err(OpenAiAccountAdapterError::BrowserUnavailableForProviderEnrollment)
    );
}

#[test]
fn start_response_maps_ephemeral_challenge_and_redacts_enclosing_action() {
    let mut adapter = OpenAiAccountAppServerAdapter::default();
    let effect = start_effect(
        OpenAiAccountMethod::Browser,
        OpenAiAccountLoginContext::PrimaryAuth,
    );
    let attempt_id = effect_attempt(&effect);
    adapter
        .request_for_effect(RequestId::Integer(1), &effect)
        .expect("start effect should map");
    let action = adapter.start_finished(
        attempt_id,
        LoginAccountResponse::Chatgpt {
            login_id: "login-1".into(),
            auth_url: "https://canary.example/secret".into(),
        },
    );
    assert!(!format!("{action:?}").contains("canary"));
    match action {
        OpenAiAccountAction::StartFinished {
            result:
                OpenAiAccountStartResult::Started {
                    login_id,
                    challenge,
                },
            ..
        } => {
            assert_eq!(login_id, OpenAiAccountLoginId::new("login-1"));
            assert_eq!(
                challenge.browser_auth_url(),
                Some("https://canary.example/secret")
            );
        }
        other => panic!("unexpected start result: {other:?}"),
    }
}

#[test]
fn device_response_requires_expected_variant_and_unexpected_data_is_typed() {
    let mut adapter = OpenAiAccountAppServerAdapter::default();
    let effect = start_effect(
        OpenAiAccountMethod::DeviceCode,
        OpenAiAccountLoginContext::ProviderEnrollment,
    );
    let attempt_id = effect_attempt(&effect);
    adapter
        .request_for_effect(RequestId::Integer(1), &effect)
        .expect("device start should map");
    assert!(matches!(
        adapter.start_finished(attempt_id, LoginAccountResponse::ApiKey {}),
        OpenAiAccountAction::StartFinished {
            result: OpenAiAccountStartResult::ProtocolMismatch,
            ..
        }
    ));
}

#[test]
fn wrong_variant_retains_login_id_for_correlated_cancel_cleanup() {
    let (mut controller, effect) = controller_and_start_effect(
        OpenAiAccountMethod::DeviceCode,
        OpenAiAccountLoginContext::PrimaryAuth,
    );
    let attempt_id = effect_attempt(&effect);
    let mut adapter = OpenAiAccountAppServerAdapter::default();
    adapter
        .request_for_effect(RequestId::Integer(1), &effect)
        .expect("device start should map");
    let action = adapter.start_finished(
        attempt_id,
        LoginAccountResponse::Chatgpt {
            login_id: "wrong-mode-login".into(),
            auth_url: "https://unexpected.example".into(),
        },
    );
    let transition = controller.dispatch(action.into());
    let [ProviderAuthEffect::OpenAiAccount(cancel_effect)] = transition.effects.as_slice() else {
        panic!("wrong variant must issue correlated cancel: {transition:?}");
    };
    assert!(matches!(
        cancel_effect,
        OpenAiAccountEffect::CancelLogin { login_id, .. }
            if login_id == &OpenAiAccountLoginId::new("wrong-mode-login")
    ));
    assert!(matches!(
        adapter
            .request_for_effect(RequestId::Integer(2), cancel_effect)
            .expect("cancel should map"),
        ClientRequest::CancelLoginAccount {
            params: CancelLoginAccountParams { login_id },
            ..
        } if login_id == "wrong-mode-login"
    ));
    let cancelled = adapter.cancel_finished(
        attempt_id,
        CancelLoginAccountResponse {
            status: CancelLoginAccountStatus::Canceled,
        },
    );
    assert!(matches!(
        controller.dispatch(cancelled.into()).snapshot,
        codex_provider_auth::ProviderAuthFlowSnapshot::OpenAiAccount(
            codex_provider_auth::OpenAiAccountSnapshot::Failed {
                reason: codex_provider_auth::OpenAiAccountFailureReason::ProtocolMismatch,
                ..
            }
        )
    ));
    assert_eq!(
        adapter.login_completed(AccountLoginCompletedNotification {
            login_id: Some("wrong-mode-login".into()),
            success: true,
            error: Some("must be discarded".into()),
        }),
        None
    );
}

#[test]
fn cancel_request_and_not_found_preserve_late_completion_correlation() {
    let (mut adapter, attempt_id, login_id) = started_adapter();
    assert_eq!(
        adapter.request_for_effect(
            RequestId::Integer(2),
            &OpenAiAccountEffect::CancelLogin {
                attempt_id,
                login_id: login_id.clone(),
            },
        ),
        Ok(ClientRequest::CancelLoginAccount {
            request_id: RequestId::Integer(2),
            params: CancelLoginAccountParams {
                login_id: "login-1".into(),
            },
        })
    );
    assert!(matches!(
        adapter.cancel_finished(
            attempt_id,
            CancelLoginAccountResponse {
                status: CancelLoginAccountStatus::NotFound,
            },
        ),
        OpenAiAccountAction::CancelFinished {
            result: OpenAiCancelResult::NotFound,
            ..
        }
    ));
    let action = adapter
        .login_completed(AccountLoginCompletedNotification {
            login_id: Some("login-1".into()),
            success: true,
            error: Some("canary server text".into()),
        })
        .expect("late completion should retain correlation");
    assert!(!format!("{action:?}").contains("canary"));
    assert_eq!(
        action,
        OpenAiAccountAction::LoginCompleted {
            attempt_id,
            login_id,
            outcome: OpenAiAccountLoginOutcome::Succeeded,
        }
    );
}

#[test]
fn stale_notifications_and_definite_cancel_are_dropped() {
    let (mut adapter, attempt_id, _) = started_adapter();
    assert_eq!(
        adapter.login_completed(AccountLoginCompletedNotification {
            login_id: Some("stale".into()),
            success: true,
            error: None,
        }),
        None
    );
    adapter.cancel_finished(
        attempt_id,
        CancelLoginAccountResponse {
            status: CancelLoginAccountStatus::Canceled,
        },
    );
    assert_eq!(
        adapter.login_completed(AccountLoginCompletedNotification {
            login_id: Some("login-1".into()),
            success: false,
            error: Some("arbitrary".into()),
        }),
        None
    );
    assert_eq!(
        adapter.login_completed(AccountLoginCompletedNotification {
            login_id: None,
            success: true,
            error: None,
        }),
        None
    );
}

#[test]
fn typed_request_errors_collapse_without_server_strings() {
    let mut adapter = OpenAiAccountAppServerAdapter::default();
    let effect = start_effect(
        OpenAiAccountMethod::Browser,
        OpenAiAccountLoginContext::PrimaryAuth,
    );
    let attempt_id = effect_attempt(&effect);
    adapter
        .request_for_effect(RequestId::Integer(1), &effect)
        .expect("start should map");
    let server = TypedRequestError::Server {
        method: "account/login/start".into(),
        source: JSONRPCErrorError {
            code: -32603,
            message: "canary arbitrary server string".into(),
            data: None,
        },
    };
    let action = adapter.start_failed(attempt_id, &server);
    assert!(matches!(
        action,
        OpenAiAccountAction::StartFinished {
            result: OpenAiAccountStartResult::Rejected,
            ..
        }
    ));
    assert!(!format!("{action:?}").contains("canary"));

    let transport = TypedRequestError::Transport {
        method: "account/login/cancel".into(),
        source: IoError::new(ErrorKind::BrokenPipe, "canary transport"),
    };
    assert!(matches!(
        adapter.cancel_failed(attempt_id, &transport),
        OpenAiAccountAction::CancelFinished {
            result: OpenAiCancelResult::TransportLost,
            ..
        }
    ));
}

fn started_adapter() -> (
    OpenAiAccountAppServerAdapter,
    ProviderAuthAttemptId,
    OpenAiAccountLoginId,
) {
    let mut adapter = OpenAiAccountAppServerAdapter::default();
    let effect = start_effect(
        OpenAiAccountMethod::Browser,
        OpenAiAccountLoginContext::PrimaryAuth,
    );
    let attempt_id = effect_attempt(&effect);
    adapter
        .request_for_effect(RequestId::Integer(1), &effect)
        .expect("start should map");
    let action = adapter.start_finished(
        attempt_id,
        LoginAccountResponse::Chatgpt {
            login_id: "login-1".into(),
            auth_url: "https://example.test".into(),
        },
    );
    let OpenAiAccountAction::StartFinished {
        result: OpenAiAccountStartResult::Started { login_id, .. },
        ..
    } = action
    else {
        unreachable!()
    };
    (adapter, attempt_id, login_id)
}

fn start_effect(
    method: OpenAiAccountMethod,
    context: OpenAiAccountLoginContext,
) -> OpenAiAccountEffect {
    controller_and_start_effect(method, context).1
}

fn controller_and_start_effect(
    method: OpenAiAccountMethod,
    context: OpenAiAccountLoginContext,
) -> (ProviderAuthController, OpenAiAccountEffect) {
    let providers = built_in_model_providers(None);
    let catalog = ProviderCatalog::from_runtime_providers(&providers);
    let entry = catalog
        .get(OPENAI_PROVIDER_ID)
        .expect("OpenAI catalog entry should exist");
    let target = codex_provider_auth::OpenAiAccountTarget::from_catalog_entry(entry)
        .expect("OpenAI account target should derive");
    let status = ProviderStatusSnapshot {
        id: entry.id.clone(),
        methods: vec![
            ProviderMethodStatus {
                capability: ProviderSetupCapability::OpenAiAccount,
                state: ProviderMethodState::NotConfigured,
            },
            ProviderMethodStatus {
                capability: entry
                    .setup_capabilities
                    .alternatives
                    .first()
                    .expect("OpenAI API-key alternative")
                    .clone(),
                state: ProviderMethodState::Configured {
                    source: ProviderCredentialSource::OpenAiApiKey,
                    control: CredentialControl::ManagedByCorbanu,
                    availability: ConfiguredAvailability::Ready,
                },
            },
        ],
        configuration: ProviderConfigurationState::Configured,
        eligibility: ProviderEligibilityState::Active,
        current: ProviderCurrentState::NotCurrent,
        availability: ProviderAvailabilityState::Ready,
    };
    let mut controller = ProviderAuthController::default();
    let transition = controller.dispatch(
        codex_provider_auth::OpenAiAccountAction::Start(
            codex_provider_auth::OpenAiAccountFlowStart {
                target,
                method,
                context,
                status,
            },
        )
        .into(),
    );
    match transition.effects.into_iter().next() {
        Some(ProviderAuthEffect::OpenAiAccount(
            effect @ OpenAiAccountEffect::StartLogin { .. },
        )) => (controller, effect),
        other => panic!("unexpected effect: {other:?}"),
    }
}

fn effect_attempt(effect: &OpenAiAccountEffect) -> ProviderAuthAttemptId {
    match effect {
        OpenAiAccountEffect::StartLogin { attempt_id, .. }
        | OpenAiAccountEffect::PresentChallenge { attempt_id, .. }
        | OpenAiAccountEffect::CancelLogin { attempt_id, .. }
        | OpenAiAccountEffect::RefreshStatus { attempt_id, .. } => *attempt_id,
    }
}
