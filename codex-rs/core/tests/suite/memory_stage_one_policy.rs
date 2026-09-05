use codex_core::memory_stage_one::StageOneMemoryDenial;
use codex_core::memory_stage_one::StageOneMemoryError;
use codex_protocol::ThreadId;
use codex_security_policy::SecurityLevel;
use core_test_support::responses::start_mock_server;
use core_test_support::test_codex::test_codex;

#[tokio::test]
async fn pf_30_s04_host_factory_rejects_protected_wrong_owner_and_terminated_thread() -> anyhow::Result<()> {
    for level in [SecurityLevel::Permissive, SecurityLevel::Moderate, SecurityLevel::Aggressive] {
        let server = start_mock_server().await;
        let test = test_codex().with_config(move |config| config.security_level = level)
            .build_with_auto_env(&server).await?;
        let result = test.codex.stage_one_memory_client(test.session_configured.thread_id, &test.config.model_provider).await;
        if level == SecurityLevel::Permissive {
            let client = result?;
            assert!(matches!(test.codex.stage_one_memory_client(ThreadId::new(), &test.config.model_provider).await,
                Err(StageOneMemoryError::Denied(StageOneMemoryDenial::OwnerMismatch))));
            test.codex.shutdown_and_wait().await?;
            assert!(matches!(client.check_completion().await,
                Err(StageOneMemoryError::Denied(StageOneMemoryDenial::OwnerTerminated))));
        } else {
            assert!(matches!(result,
                Err(StageOneMemoryError::Denied(StageOneMemoryDenial::ProtectedInputUnavailable))));
            test.codex.shutdown_and_wait().await?;
        }
        assert!(server.received_requests().await.unwrap().is_empty());
    }
    Ok(())
}
