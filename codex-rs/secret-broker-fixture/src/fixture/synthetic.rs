//! Synthetic process fixture only: same-user inherited socket, mock keyring and
//! volatile integrity root. These are NOT measurements of protected eligibility.
use codex_keyring_store::tests::MockKeyringStore;
use codex_secret_broker::journal_adapter::*;
use codex_secret_broker::platform_contract::*;
use codex_secret_broker::*;
use codex_secret_broker_service::BrokerService;
use codex_secret_broker_service::TrustedSession;
use codex_security_audit::*;
use codex_security_policy::*;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_vault::*;
use std::collections::BTreeMap;
use std::io::Read;
use std::io::Write;
use std::os::fd::AsFd;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

const SYNTHETIC_SECRET: &str = "BROKER-SERVICE-SYNTHETIC-ONLY";

#[derive(Debug, Default)]
struct VolatileFixtureRoot(Mutex<Option<IntegrityCheckpoint>>);
impl IntegrityRootStore for VolatileFixtureRoot {
    fn load(&self) -> Result<Option<IntegrityCheckpoint>, IntegrityRootError> {
        Ok(self
            .0
            .lock()
            .map_err(|_| IntegrityRootError::Unavailable)?
            .clone())
    }
    fn compare_and_store(
        &self,
        expected: Option<&IntegrityCheckpoint>,
        next: &IntegrityCheckpoint,
    ) -> Result<(), IntegrityRootError> {
        let mut stored = self.0.lock().map_err(|_| IntegrityRootError::Unavailable)?;
        if stored.as_ref() != expected {
            return Err(IntegrityRootError::Conflict);
        }
        *stored = Some(next.clone());
        Ok(())
    }
}

struct FixtureClock;
impl BrokerJournalClock for FixtureClock {
    fn now_unix_seconds(&self) -> Result<i64, BrokerAuditError> {
        Ok(110)
    }
}
impl VaultBrokerClock for FixtureClock {
    fn now_unix_seconds(&self) -> Result<i64, BackendDispatchError> {
        Ok(110)
    }
}
struct SyntheticTransport;
impl VaultBrokerTransport for SyntheticTransport {
    fn execute_openai_responses(
        &self,
        secret: &str,
        _: &OpenAiResponsesOperation,
        fence: &CancellationFence,
    ) -> Result<TypedOperationReceipt, BackendDispatchError> {
        fence.ensure_active()?;
        if secret != SYNTHETIC_SECRET {
            return Err(BackendDispatchError::Failed);
        }
        // No external network or model output; only a secret-free fixed receipt.
        Ok(TypedOperationReceipt {
            response_status: 204,
            uploaded_bytes: 0,
            downloaded_bytes: 0,
        })
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() != Some("--synthetic-inherited-socket") {
        return Err("explicit synthetic fixture mode required".into());
    }
    // Test-only signal handler exercises EINTR without process-global changes
    // in the test runner. Production service signal policy remains uninstalled.
    let _signal = signal_hook::flag::register(
        signal_hook::consts::SIGUSR1,
        Arc::new(std::sync::atomic::AtomicBool::new(false)),
    )?;
    let inherited = std::io::stdin().as_fd().try_clone_to_owned()?;
    let mut socket = UnixStream::from(inherited);
    // Parent-owned inherited socket capability; not a path or worker JSON.
    // SO_PEERCRED identifies the socket-pair creator, NOT this child after exec.
    let peer = ObservedPeer::from_os(
        format!("uid:{}", nix::unistd::getuid()),
        u32::try_from(nix::unistd::getppid().as_raw())?,
    )?;
    if codex_secret_broker::linux_transport::observed_peer(&socket)? != peer {
        return Err("fixture parent identity mismatch".into());
    }
    socket.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
    let mut key = [0; 32];
    socket.read_exact(&mut key)?;
    let directory = tempfile::tempdir()?;
    let vault = Arc::new(Vault::new_with_keyring_store(
        directory.path().join("vault"),
        Arc::new(MockKeyringStore::default()),
    ));
    vault.add(AddCredential {
        label: "synthetic-key".into(),
        credential_type: CredentialType::BearerToken,
        provider: Some("openai".into()),
        notes: None,
        revocation_notes: None,
        secret: SYNTHETIC_SECRET.into(),
    })?;
    let text = |s: &str| BoundedText::new(s);
    let human = PolicyPrincipal::new(PrincipalKind::Human, "fixture-human")?;
    let actors = ActorChain::new(vec![
        human.clone(),
        PolicyPrincipal::new(PrincipalKind::Agent, "fixture-agent")?,
    ])?;
    let authorization = AuthorizationRequest::new(
        actors.clone(),
        ProtectedResource::new(ResourceKind::VaultCredential, "synthetic-key")?,
        PolicyAction::Use,
        AuthorizationContext {
            now_unix_seconds: 100,
            session_id: text("session")?,
            task_id: text("task")?,
            purpose: text("fixture")?,
            operation: text("openai.responses.create")?,
            destination: Some(text("https://api.openai.com:443")?),
            quantity: None,
            grant_id: None,
        },
    )?;
    let grant = BoundedGrant::issue(
        human,
        actors,
        GrantScope::new(
            authorization.resource.clone(),
            [PolicyAction::Use],
            GrantContext::new(
                authorization.context.session_id.clone(),
                authorization.context.task_id.clone(),
                authorization.context.purpose.clone(),
                authorization.context.operation.clone(),
            ),
            authorization.context.destination.clone(),
            BTreeMap::new(),
        )?,
        90,
        200,
        text("fixture-grant")?,
    )?;
    let revocations = RevocationState::new();
    let request = CredentialCapabilityRequest::new(
        authorization,
        grant,
        codex_security_policy::CredentialReference::new(
            "synthetic-key",
            "openai.responses.create",
        )?,
        CredentialHttpMethod::Post,
        CredentialDestination::https("api.openai.com", 443)?,
        "/v1/responses",
        100,
        180,
        &revocations,
        None,
    )?;
    let reference = codex_secret_broker::CredentialReference::from_sha256_hex("a".repeat(64))?;
    let credential = VaultCredentialRef::from_authorized(
        CapabilityId::from_sha256_hex(reference.as_str())?,
        request.clone(),
    )?;
    let operation = OpenAiResponsesOperation::new("/v1/responses")?;
    let binding = BrokerBinding {
        controller_instance: "controller".into(),
        worker_instance: "worker".into(),
        session_id: "session".into(),
        task_id: "task".into(),
        run_id: "run".into(),
        run_generation: 1,
    };
    let producer = PolicyPrincipal::new(PrincipalKind::Service, "synthetic-service")?;
    let root = Arc::new(VolatileFixtureRoot::default());
    let mut journal = ReferenceJournal::new(
        AbsolutePathBuf::from_absolute_path_checked(directory.path().join("journal"))?,
        JournalOwner::new(producer.clone(), 1, text("volatile-fixture-root")?)?,
        root.clone(),
        JournalConfig::default(),
    );
    if !matches!(
        journal.recover(1, 1, &revocations).state,
        RecoveryState::Empty | RecoveryState::Ready
    ) {
        return Err("synthetic journal recovery denied".into());
    }
    let mut audit_request = request.authorization.clone();
    audit_request.context.grant_id = Some(request.grant.grant_id.clone());
    let audit = JournalBrokerAudit::new(
        journal,
        BrokerJournalBinding {
            binding: binding.clone(),
            credential: reference.clone(),
            request: audit_request,
            authority: AuthorityIdentity::Grant {
                grant_id: request.grant.grant_id,
            },
            operation,
        },
        EventContext::new(producer, 1, 1)?,
        FixtureClock,
    )?;
    let backend = VaultBrokerBackend::with_clock(
        vault,
        vec![(reference.clone(), credential)],
        Arc::new(RwLock::new(revocations)),
        SyntheticTransport,
        FixtureClock,
    )?;
    let capabilities = REQUIRED_CAPABILITIES
        .iter()
        .copied()
        .map(|capability| CapabilityResult {
            capability,
            status: CapabilityStatus::Supported,
            observation: Observation::Denied,
            mechanism: "synthetic-fixture-only",
            detail_code: "denied",
        })
        .collect::<Vec<_>>();
    let target = "1".repeat(64);
    let probe = "2".repeat(64);
    let platform = validate_protected_mode_report(
        &PlatformReport {
            contract_version: CONTRACT_VERSION,
            fixture_protocol: FIXTURE_PROTOCOL_VERSION,
            probe_sha256: &probe,
            target_id: &target,
            measured_at_unix_seconds: 100,
            expires_at_unix_seconds: 200,
            capabilities: &capabilities,
            protected_mode_eligible: true,
        },
        &target,
        &probe,
        150,
    )
    .map_err(|_| "synthetic platform fixture invalid")?;
    let service = BrokerService::new(
        format!("{:064x}", std::process::id()),
        BrokerRuntimeConfig::bounded(/*max_sessions*/ 4, /*max_in_flight*/ 4)?,
        platform,
        backend,
        audit,
    )?;
    let peer = if std::env::args().nth(2).as_deref() == Some("--wrong-peer") {
        ObservedPeer::from_os("uid:4294967294", peer.process_id())?
    } else {
        peer
    };
    println!("synthetic-only ready");
    std::io::stdout().flush()?;
    let result = service.serve(TrustedSession {
        socket,
        expected_peer: peer,
        binding,
        channel_mac: BrokerChannelMac::from_secret(key),
        credential_grants: vec![BrokerCredentialGrant::expiring(reference, i64::MAX)?],
    });
    // Safe test receipt proves journal settlement without printing raw records.
    let count = root
        .load()?
        .map(|checkpoint| checkpoint.sequence)
        .unwrap_or(0);
    println!("synthetic-only journal-records={count}");
    if !matches!(result, Err(BrokerDispatchError::SessionUnavailable)) {
        result?;
    }
    Ok(())
}
