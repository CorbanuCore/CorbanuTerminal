use crate::ipc::*;
use crate::platform_contract::*;
use crate::resolver::*;
use crate::resolver_types::*;
use pretty_assertions::assert_eq;
use std::sync::Arc;
use std::sync::Barrier;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

const KEY: [u8; 32] = [9; 32];
const INSTANCE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const OTHER_INSTANCE: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const TARGET: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const PROBE: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const REFERENCE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER_REFERENCE: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

#[cfg(target_os = "linux")]
#[test]
fn pf_27_s01_native_registered_connection_replacement_and_revocation() {
    use crate::linux_transport::LinuxBrokerChannel;
    use crate::linux_transport::LinuxBrokerSession;
    use crate::linux_transport::observed_peer;
    use crate::linux_transport::serve_connection;
    use std::os::unix::net::UnixStream;

    let backend = Arc::new(FakeBackend::default());
    let runtime = Arc::new(runtime(INSTANCE, backend.clone(), Arc::new(FakeAudit::default())));
    let (client, server) = UnixStream::pair().expect("socket pair");
    let peer = observed_peer(&client).expect("OS peer");
    let channel = LinuxBrokerChannel::new(client, &peer).expect("client");
    let handle = register(&runtime, 1, peer.clone());
    let handler = LinuxBrokerSession::new(runtime.clone(), handle);
    let old_server = std::thread::spawn(move || serve_connection(server, &peer, &handler));
    assert!(channel.dispatch(&frame(binding(1), 1)).is_ok());

    // A fresh connection for the same run replaces the retained old session;
    // neither an old open channel nor its cached binding can inherit the grant.
    let (new_client, new_server) = UnixStream::pair().expect("new socket pair");
    let peer = observed_peer(&new_client).expect("new OS peer");
    let new_channel = LinuxBrokerChannel::new(new_client, &peer).expect("new client");
    let handle = register(&runtime, 2, peer.clone());
    let handler = LinuxBrokerSession::new(runtime.clone(), handle);
    let new_server = std::thread::spawn(move || serve_connection(new_server, &peer, &handler));
    assert!(channel.dispatch(&frame(binding(1), 2)).is_err());
    assert!(old_server.join().expect("old server").is_err());
    assert!(new_channel.dispatch(&frame(binding(2), 1)).is_ok());
    runtime.revoke_run("controller-1", "run-1").expect("revoke");
    assert!(new_channel.dispatch(&frame(binding(2), 2)).is_err());
    assert!(new_server.join().expect("new server").is_err());
    assert_eq!(backend.calls.load(Ordering::SeqCst), 2);
}

#[derive(Default)]
struct FakeBackend {
    calls: AtomicUsize,
    entered: Option<Arc<Barrier>>,
    release: Option<Arc<Barrier>>,
}

impl TypedCredentialBackend for Arc<FakeBackend> {
    fn execute_openai_responses(
        &self,
        _credential: &CredentialReference,
        operation: &OpenAiResponsesOperation,
        cancellation: &CancellationFence,
    ) -> Result<TypedOperationReceipt, BackendDispatchError> {
        cancellation.ensure_active()?;
        assert_eq!(operation.host(), "api.openai.com");
        assert_eq!(operation.method(), "POST");
        self.calls.fetch_add(1, Ordering::SeqCst);
        if let Some(entered) = &self.entered {
            entered.wait();
        }
        if let Some(release) = &self.release {
            release.wait();
        }
        cancellation.ensure_active()?;
        Ok(TypedOperationReceipt {
            response_status: 200,
            uploaded_bytes: 11,
            downloaded_bytes: 22,
        })
    }
}

#[derive(Default)]
struct FakeAudit {
    next: AtomicUsize,
    fail_reserve: AtomicBool,
    fail_resolve: AtomicBool,
    intents: Mutex<Vec<BrokerAuditIntent>>,
    resolutions: Mutex<Vec<BrokerAuditResolution>>,
}

impl DurableBrokerAudit for Arc<FakeAudit> {
    type Permit = usize;

    fn reserve(&self, intent: &BrokerAuditIntent) -> Result<Self::Permit, BrokerAuditError> {
        if self.fail_reserve.load(Ordering::SeqCst) {
            return Err(BrokerAuditError::Unavailable);
        }
        self.intents.lock().expect("intents").push(intent.clone());
        Ok(self.next.fetch_add(1, Ordering::SeqCst))
    }

    fn resolve(
        &self,
        _permit: Self::Permit,
        resolution: BrokerAuditResolution,
    ) -> Result<(), BrokerAuditError> {
        if self.fail_resolve.load(Ordering::SeqCst) {
            return Err(BrokerAuditError::CommitUnknown);
        }
        self.resolutions
            .lock()
            .expect("resolutions")
            .push(resolution);
        Ok(())
    }
}

fn authorization() -> ProtectedModeAuthorization {
    let capabilities = REQUIRED_CAPABILITIES
        .iter()
        .copied()
        .map(|capability| CapabilityResult {
            capability,
            status: CapabilityStatus::Supported,
            observation: Observation::Denied,
            mechanism: "synthetic-test-only",
            detail_code: "denied",
        })
        .collect::<Vec<_>>();
    validate_protected_mode_report(
        &PlatformReport {
            contract_version: CONTRACT_VERSION,
            fixture_protocol: FIXTURE_PROTOCOL_VERSION,
            probe_sha256: PROBE,
            target_id: TARGET,
            measured_at_unix_seconds: 100,
            expires_at_unix_seconds: 200,
            capabilities: &capabilities,
            protected_mode_eligible: true,
        },
        TARGET,
        PROBE,
        150,
    )
    .expect("synthetic authorization")
}

fn platform_report<'a>(capabilities: &'a [CapabilityResult<'a>]) -> PlatformReport<'a> {
    PlatformReport {
        contract_version: CONTRACT_VERSION,
        fixture_protocol: FIXTURE_PROTOCOL_VERSION,
        probe_sha256: PROBE,
        target_id: TARGET,
        measured_at_unix_seconds: 100,
        expires_at_unix_seconds: 200,
        capabilities,
        protected_mode_eligible: true,
    }
}

fn binding(generation: u64) -> BrokerBinding {
    BrokerBinding {
        controller_instance: "controller-1".to_string(),
        worker_instance: format!("worker-{generation}"),
        session_id: format!("session-{generation}"),
        task_id: "task-1".to_string(),
        run_id: "run-1".to_string(),
        run_generation: generation,
    }
}

fn peer(pid: u32) -> ObservedPeer {
    ObservedPeer::from_os("worker-uid-501", pid).expect("peer")
}

fn operation_for(reference: &str) -> BrokerOperation {
    BrokerOperation::OpenAiResponses {
        credential: CredentialReference::from_sha256_hex(reference).expect("reference"),
        request: OpenAiResponsesOperation::new("/v1/responses").expect("operation"),
    }
}

fn operation() -> BrokerOperation {
    operation_for(REFERENCE)
}

fn frame(binding: BrokerBinding, sequence: u64) -> SignedBrokerFrame {
    frame_for(binding, sequence, operation())
}

fn frame_for(
    binding: BrokerBinding,
    sequence: u64,
    operation: BrokerOperation,
) -> SignedBrokerFrame {
    BrokerChannelMac::from_secret(KEY)
        .sign(binding, sequence, operation)
        .expect("frame")
}

fn runtime(
    instance: &str,
    backend: Arc<FakeBackend>,
    audit: Arc<FakeAudit>,
) -> BrokerRuntime<Arc<FakeBackend>, Arc<FakeAudit>> {
    BrokerRuntime::new(
        instance,
        BrokerRuntimeConfig::bounded(4, 4).expect("config"),
        authorization(),
        backend,
        audit,
    )
    .expect("runtime")
}

fn register(
    runtime: &BrokerRuntime<Arc<FakeBackend>, Arc<FakeAudit>>,
    generation: u64,
    observed_peer: ObservedPeer,
) -> BrokerSessionHandle {
    runtime
        .register_session(
            binding(generation),
            observed_peer,
            BrokerChannelMac::from_secret(KEY),
            vec![
                BrokerCredentialGrant::expiring(
                    CredentialReference::from_sha256_hex(REFERENCE).expect("reference"),
                    i64::MAX,
                )
                .expect("grant"),
            ],
        )
        .expect("session")
}

#[test]
fn pf_27_s04_pf_27_s01_typed_dispatch_is_audited_before_backend_and_resolved() {
    let backend = Arc::new(FakeBackend::default());
    let audit = Arc::new(FakeAudit::default());
    let runtime = runtime(INSTANCE, backend.clone(), audit.clone());
    let observed_peer = peer(100);
    let handle = register(&runtime, 1, observed_peer.clone());

    let receipt = runtime
        .dispatch(&handle, &observed_peer, &frame(binding(1), 1))
        .expect("receipt");

    assert_eq!(
        receipt,
        TypedOperationReceipt {
            response_status: 200,
            uploaded_bytes: 11,
            downloaded_bytes: 22,
        }
    );
    assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
    let intents = audit.intents.lock().expect("intents");
    assert_eq!(intents.len(), 1);
    assert_eq!(intents[0].path, "/v1/responses");
    assert_eq!(
        *audit.resolutions.lock().expect("resolutions"),
        vec![BrokerAuditResolution::Completed]
    );
}

#[test]
fn pf_27_s04_pf_27_s01_replay_wrong_peer_and_binding_are_rejected_before_backend() {
    let backend = Arc::new(FakeBackend::default());
    let audit = Arc::new(FakeAudit::default());
    let runtime = runtime(INSTANCE, backend.clone(), audit);
    let observed_peer = peer(100);
    let handle = register(&runtime, 1, observed_peer.clone());
    let first = frame(binding(1), 1);
    runtime
        .dispatch(&handle, &observed_peer, &first)
        .expect("first");
    assert_eq!(
        runtime.dispatch(&handle, &observed_peer, &first),
        Err(BrokerDispatchError::ReplayOrSequenceGap)
    );
    assert_eq!(
        runtime.dispatch(&handle, &peer(101), &frame(binding(1), 2)),
        Err(BrokerDispatchError::WrongPeer)
    );
    let mut forged = binding(1);
    forged.task_id = "task-forged".to_string();
    assert_eq!(
        runtime.dispatch(&handle, &observed_peer, &frame(forged, 2)),
        Err(BrokerDispatchError::BindingMismatch)
    );
    assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn pf_27_s04_pf_27_s01_same_run_replacement_invalidates_old_and_stale_connections() {
    let backend = Arc::new(FakeBackend::default());
    let audit = Arc::new(FakeAudit::default());
    let runtime = runtime(INSTANCE, backend.clone(), audit);
    let old_peer = peer(100);
    let old = register(&runtime, 1, old_peer.clone());
    let new_peer = peer(101);
    let new = register(&runtime, 2, new_peer.clone());

    assert_eq!(
        runtime.dispatch(&old, &old_peer, &frame(binding(1), 1)),
        Err(BrokerDispatchError::SessionUnavailable)
    );
    assert!(matches!(
        runtime.register_session(
            binding(1),
            old_peer,
            BrokerChannelMac::from_secret(KEY),
            vec![
                BrokerCredentialGrant::expiring(
                    CredentialReference::from_sha256_hex(REFERENCE).expect("reference"),
                    i64::MAX,
                )
                .expect("grant")
            ],
        ),
        Err(BrokerDispatchError::StaleRunGeneration)
    ));
    runtime
        .dispatch(&new, &new_peer, &frame(binding(2), 1))
        .expect("new generation");
    assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
}

#[test]
fn pf_27_s04_pf_27_s01_restart_and_cancel_reject_old_handles_without_fallback() {
    let backend = Arc::new(FakeBackend::default());
    let audit = Arc::new(FakeAudit::default());
    let first = runtime(INSTANCE, backend.clone(), audit.clone());
    let observed_peer = peer(100);
    let old = register(&first, 1, observed_peer.clone());
    let restarted = runtime(OTHER_INSTANCE, backend.clone(), audit);
    assert_eq!(
        restarted.dispatch(&old, &observed_peer, &frame(binding(1), 1)),
        Err(BrokerDispatchError::BrokerRestarted)
    );
    first.cancel_session(&old).expect("cancel");
    assert_eq!(
        first.dispatch(&old, &observed_peer, &frame(binding(1), 1)),
        Err(BrokerDispatchError::SessionUnavailable)
    );
    assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn pf_27_s04_pf_27_s01_cross_run_reference_theft_expiry_and_revocation_fail_closed() {
    let backend = Arc::new(FakeBackend::default());
    let audit = Arc::new(FakeAudit::default());
    let runtime = runtime(INSTANCE, backend.clone(), audit);
    let observed_peer = peer(100);
    let handle = runtime
        .register_session(
            binding(1),
            observed_peer.clone(),
            BrokerChannelMac::from_secret(KEY),
            vec![
                BrokerCredentialGrant::expiring(
                    CredentialReference::from_sha256_hex(OTHER_REFERENCE).expect("reference"),
                    i64::MAX,
                )
                .expect("grant"),
            ],
        )
        .expect("session");
    assert_eq!(
        runtime.dispatch(&handle, &observed_peer, &frame(binding(1), 1)),
        Err(BrokerDispatchError::CredentialUnavailable)
    );
    let other_reference = CredentialReference::from_sha256_hex(OTHER_REFERENCE).expect("reference");
    runtime
        .revoke_credential(&handle, &other_reference)
        .expect("revoke credential");
    assert_eq!(
        runtime.dispatch(
            &handle,
            &observed_peer,
            &frame_for(binding(1), 1, operation_for(OTHER_REFERENCE)),
        ),
        Err(BrokerDispatchError::CredentialUnavailable)
    );

    let expiring = runtime
        .register_session(
            binding(2),
            observed_peer.clone(),
            BrokerChannelMac::from_secret(KEY),
            vec![
                BrokerCredentialGrant::expiring(
                    CredentialReference::from_sha256_hex(REFERENCE).expect("reference"),
                    1,
                )
                .expect("grant"),
            ],
        )
        .expect("session");
    assert_eq!(
        runtime.dispatch(&expiring, &observed_peer, &frame(binding(2), 1)),
        Err(BrokerDispatchError::CredentialExpired)
    );
    assert_eq!(backend.calls.load(Ordering::SeqCst), 0);
}

#[test]
fn pf_27_s04_pf_27_s01_audit_failure_blocks_dispatch_and_ambiguous_resolution_is_visible() {
    let backend = Arc::new(FakeBackend::default());
    let audit = Arc::new(FakeAudit::default());
    audit.fail_reserve.store(true, Ordering::SeqCst);
    let runtime = runtime(INSTANCE, backend.clone(), audit.clone());
    let observed_peer = peer(100);
    let handle = register(&runtime, 1, observed_peer.clone());
    assert_eq!(
        runtime.dispatch(&handle, &observed_peer, &frame(binding(1), 1)),
        Err(BrokerDispatchError::AuditUnavailable)
    );
    assert_eq!(backend.calls.load(Ordering::SeqCst), 0);

    audit.fail_reserve.store(false, Ordering::SeqCst);
    audit.fail_resolve.store(true, Ordering::SeqCst);
    assert_eq!(
        runtime.dispatch(&handle, &observed_peer, &frame(binding(1), 2)),
        Err(BrokerDispatchError::AuditCommitUnknown)
    );
    assert_eq!(backend.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        runtime.dispatch(&handle, &observed_peer, &frame(binding(1), 3)),
        Err(BrokerDispatchError::SessionUnavailable)
    );
}

#[test]
fn pf_27_s04_pf_27_s01_unsupported_platform_report_cannot_construct_runtime() {
    let mut capabilities = REQUIRED_CAPABILITIES
        .iter()
        .copied()
        .map(|capability| CapabilityResult {
            capability,
            status: CapabilityStatus::Supported,
            observation: Observation::Denied,
            mechanism: "synthetic-test-only",
            detail_code: "denied",
        })
        .collect::<Vec<_>>();
    capabilities[0].status = CapabilityStatus::Unsupported;
    capabilities[0].observation = Observation::Allowed;
    let backend = Arc::new(FakeBackend::default());
    let audit = Arc::new(FakeAudit::default());
    assert!(matches!(
        BrokerRuntime::from_platform_report(
            INSTANCE,
            BrokerRuntimeConfig::bounded(1, 1).expect("config"),
            &platform_report(&capabilities),
            TARGET,
            PROBE,
            150,
            backend,
            audit,
        ),
        Err(BrokerDispatchError::PlatformUnavailable)
    ));
}

#[test]
fn pf_27_s04_pf_27_s01_concurrent_revocation_cancels_open_upload_before_effect() {
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let backend = Arc::new(FakeBackend {
        calls: AtomicUsize::new(0),
        entered: Some(entered.clone()),
        release: Some(release.clone()),
    });
    let audit = Arc::new(FakeAudit::default());
    let runtime = Arc::new(runtime(INSTANCE, backend, audit.clone()));
    let observed_peer = peer(100);
    let handle = register(&runtime, 1, observed_peer.clone());
    let worker_runtime = runtime.clone();
    let worker = std::thread::spawn(move || {
        worker_runtime.dispatch(&handle, &observed_peer, &frame(binding(1), 1))
    });

    entered.wait();
    runtime.revoke_run("controller-1", "run-1").expect("revoke");
    release.wait();
    assert_eq!(
        worker.join().expect("worker"),
        Err(BrokerDispatchError::Cancelled)
    );
    assert_eq!(
        *audit.resolutions.lock().expect("resolutions"),
        vec![BrokerAuditResolution::Cancelled]
    );
}

#[test]
fn pf_27_s04_pf_27_s01_resource_bounds_fail_without_eviction_or_enumeration() {
    assert_eq!(
        BrokerRuntimeConfig::bounded(0, 1),
        Err(BrokerDispatchError::InvalidConfig)
    );
    let backend = Arc::new(FakeBackend::default());
    let audit = Arc::new(FakeAudit::default());
    let runtime = BrokerRuntime::new(
        INSTANCE,
        BrokerRuntimeConfig::bounded(1, 1).expect("config"),
        authorization(),
        backend,
        audit,
    )
    .expect("runtime");
    register(&runtime, 1, peer(100));
    assert!(matches!(
        runtime.register_session(
            BrokerBinding {
                run_id: "run-2".to_string(),
                ..binding(1)
            },
            peer(101),
            BrokerChannelMac::from_secret(KEY),
            vec![
                BrokerCredentialGrant::expiring(
                    CredentialReference::from_sha256_hex(REFERENCE).expect("reference"),
                    i64::MAX,
                )
                .expect("grant")
            ],
        ),
        Err(BrokerDispatchError::SessionCapacityReached)
    ));
}

#[test]
fn pf_27_s04_pf_27_s01_run_generation_history_is_bounded_without_stale_eviction() {
    let backend = Arc::new(FakeBackend::default());
    let audit = Arc::new(FakeAudit::default());
    let config = BrokerRuntimeConfig::bounded(1, 1).expect("config");
    let max_tracked_runs = config.max_tracked_runs;
    let runtime =
        BrokerRuntime::new(INSTANCE, config, authorization(), backend, audit).expect("runtime");

    for index in 0..max_tracked_runs {
        let run_binding = BrokerBinding {
            run_id: format!("run-{index}"),
            ..binding(1)
        };
        let handle = runtime
            .register_session(
                run_binding,
                peer(u32::try_from(index + 1).expect("pid")),
                BrokerChannelMac::from_secret(KEY),
                vec![
                    BrokerCredentialGrant::expiring(
                        CredentialReference::from_sha256_hex(REFERENCE).expect("reference"),
                        i64::MAX,
                    )
                    .expect("grant"),
                ],
            )
            .expect("bounded run registration");
        runtime.cancel_session(&handle).expect("cancel session");
    }

    assert!(matches!(
        runtime.register_session(
            BrokerBinding {
                run_id: "one-run-too-many".to_string(),
                ..binding(1)
            },
            peer(999),
            BrokerChannelMac::from_secret(KEY),
            vec![
                BrokerCredentialGrant::expiring(
                    CredentialReference::from_sha256_hex(REFERENCE).expect("reference"),
                    i64::MAX,
                )
                .expect("grant")
            ],
        ),
        Err(BrokerDispatchError::ResourceExhausted)
    ));
}
