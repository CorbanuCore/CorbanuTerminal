use crate::synthetic_exchange::{Driver, forward};
use crate::synthetic_io::{CappedIo, Metrics};
use anyhow::{Result, anyhow, ensure};
use hyper::service::service_fn;
use hyper_util::rt::{TokioIo, TokioTimer};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::sync::{mpsc, watch};
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep_until};

pub(crate) const HEAD: usize = 16 * 1024;
pub(crate) const INPUT: usize = 1024 * 1024;
pub(crate) const OUTPUT: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy)]
pub(crate) struct Limits {
    pub head: Duration,
    pub upload: Duration,
    pub exchange: Duration,
    pub run: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            head: Duration::from_secs(2),
            upload: Duration::from_secs(5),
            exchange: Duration::from_secs(10),
            run: Duration::from_secs(60),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stage {
    Head,
    Upload,
    Connect,
    ResponseHead,
    Stream,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Phase {
    pub stage: Stage,
    pub deadline: Instant,
}

#[derive(Clone)]
pub(crate) struct Scope {
    pub accepted: Instant,
    pub end: Instant,
    pub limits: Limits,
    pub upstream: SocketAddr,
    pub phase: watch::Sender<Phase>,
    pub ingress: Arc<AtomicUsize>,
    pub drivers: mpsc::Sender<Driver>,
    pub metrics: Arc<Metrics>,
    pub used: Arc<AtomicBool>,
}

impl Scope {
    pub fn check(&self) -> Result<()> {
        ensure!(
            Instant::now() < self.phase.borrow().deadline.min(self.end),
            "phase expired"
        );
        Ok(())
    }
    pub fn advance(&self, stage: Stage, deadline: Instant) -> Result<()> {
        self.check()?;
        self.phase.send_replace(Phase {
            stage,
            deadline: deadline.min(self.end),
        });
        self.check()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    Complete,
    Failed(String),
    Expired(Stage),
    Cancelled,
    Panicked,
}

#[derive(Debug)]
pub(crate) struct RunSummary {
    pub accepted: usize,
    pub peak: usize,
    pub outcomes: Vec<Outcome>,
    pub remaining_sockets: usize,
}

pub(crate) fn run(args: &crate::Args, upstream: SocketAddr) -> Result<()> {
    ensure!(
        upstream.ip() == Ipv4Addr::LOCALHOST && upstream.port() != 0,
        "literal loopback required"
    );
    let port = args
        .port
        .filter(|port| *port != 0 && *port != upstream.port())
        .ok_or_else(|| anyhow!("distinct explicit nonzero --port required"))?;
    ensure!(
        args.server_info.is_none()
            && args.dump_dir.is_none()
            && !args.http_shutdown
            && args.upstream_url == "https://api.openai.com/v1/responses",
        "incompatible synthetic options"
    );
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let summary = runtime.block_on(async {
        let socket = TcpSocket::new_v4()?;
        socket.bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port)))?;
        serve(
            socket.listen(2)?,
            upstream,
            Limits::default(),
            Arc::default(),
        )
        .await
    })?;
    ensure!(
        summary.remaining_sockets == 0 && summary.accepted == summary.outcomes.len(),
        "unjoined work"
    );
    ensure!(
        !summary
            .outcomes
            .iter()
            .any(|o| matches!(o, Outcome::Panicked)),
        "connection panic"
    );
    Ok(())
}

pub(crate) async fn serve(
    listener: TcpListener,
    upstream: SocketAddr,
    limits: Limits,
    metrics: Arc<Metrics>,
) -> Result<RunSummary> {
    let end = Instant::now() + limits.run;
    let mut tasks = JoinSet::new();
    let mut listener = Some(listener);
    let mut summary = RunSummary {
        accepted: 0,
        peak: 0,
        outcomes: Vec::new(),
        remaining_sockets: 0,
    };
    let mut failure = None;
    loop {
        if listener.is_none() && tasks.is_empty() {
            break;
        }
        tokio::select! {
            biased;
            _ = sleep_until(end) => { break; }
            result = tasks.join_next(), if !tasks.is_empty() => {
                if let Some(result) = result { summary.outcomes.push(joined(result)); }
            }
            result = async { match listener.as_ref() {
                Some(listener) => listener.accept().await,
                None => std::future::pending().await,
            } },
                if listener.is_some() && tasks.len() < 2 => {
                match result {
                    Ok((stream, _)) => {
                        let accepted = Instant::now();
                        summary.accepted += 1;
                        metrics.accepted.fetch_add(1, Ordering::SeqCst);
                        tasks.spawn(serve_one(stream, accepted, end, upstream, limits, metrics.clone()));
                        summary.peak = summary.peak.max(tasks.len());
                        if summary.accepted == 8 { listener.take(); }
                    }
                    Err(error) => { failure = Some(error); break; }
                }
            }
        }
    }
    drop(listener);
    tasks.abort_all();
    while let Some(result) = tasks.join_next().await {
        summary.outcomes.push(joined(result));
    }
    summary.remaining_sockets = metrics.sockets.load(Ordering::SeqCst);
    if let Some(error) = failure {
        return Err(error.into());
    }
    Ok(summary)
}

fn joined(result: std::result::Result<Outcome, tokio::task::JoinError>) -> Outcome {
    match result {
        Ok(outcome) => outcome,
        Err(error) if error.is_cancelled() => Outcome::Cancelled,
        Err(_) => Outcome::Panicked,
    }
}

async fn serve_one(
    stream: TcpStream,
    accepted: Instant,
    run_end: Instant,
    upstream: SocketAddr,
    limits: Limits,
    metrics: Arc<Metrics>,
) -> Outcome {
    let end = (accepted + limits.exchange).min(run_end);
    let (phase, mut changes) = watch::channel(Phase {
        stage: Stage::Head,
        deadline: (accepted + limits.head).min(end),
    });
    let (drivers, mut pending) = mpsc::channel(1);
    let ingress = Arc::new(AtomicUsize::new(HEAD));
    let io = TokioIo::new(CappedIo::new(stream, ingress.clone(), metrics.clone()));
    let scope = Scope {
        accepted,
        end,
        limits,
        upstream,
        phase,
        ingress,
        drivers,
        metrics,
        used: Arc::default(),
    };
    let service_scope = scope.clone();
    let service = service_fn(move |request| forward(request, service_scope.clone()));
    let mut builder = hyper::server::conn::http1::Builder::new();
    builder
        .keep_alive(false)
        .max_headers(64)
        .max_buf_size(HEAD)
        .timer(TokioTimer::new())
        .header_read_timeout(limits.head);
    let server = builder.serve_connection(io, service);
    let driver = async {
        pending
            .recv()
            .await
            .ok_or_else(|| anyhow!("missing driver"))?
            .await
            .map_err(anyhow::Error::from)
    };
    tokio::pin!(server, driver);
    let mut driving = true;
    loop {
        let phase = *changes.borrow_and_update();
        tokio::select! {
            biased;
            _ = sleep_until(phase.deadline) => { return Outcome::Expired(phase.stage); }
            _ = changes.changed() => {}
            result = &mut driver, if driving => {
                driving = false;
                if let Err(error) = result { return Outcome::Failed(error.to_string()); }
                // A clean driver EOF can leave valid buffered Incoming frames.
            }
            result = &mut server => {
                return match result {
                    Ok(()) if scope.check().is_ok() => Outcome::Complete,
                    Ok(()) => Outcome::Expired(scope.phase.borrow().stage),
                    Err(error) => Outcome::Failed(error.to_string()),
                };
            }
        }
    }
}

#[cfg(test)]
#[path = "synthetic_tests.rs"]
mod tests;
