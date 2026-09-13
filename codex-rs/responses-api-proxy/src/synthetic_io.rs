use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

#[derive(Default)]
pub(crate) struct Metrics {
    pub sockets: AtomicUsize,
    pub blocked_writes: AtomicUsize,
    pub accepted: AtomicUsize,
}

pub(crate) struct CappedIo<T> {
    inner: T,
    read: usize,
    allowance: Arc<AtomicUsize>,
    metrics: Arc<Metrics>,
}

impl<T> CappedIo<T> {
    pub fn new(inner: T, allowance: Arc<AtomicUsize>, metrics: Arc<Metrics>) -> Self {
        metrics.sockets.fetch_add(1, Ordering::SeqCst);
        Self {
            inner,
            read: 0,
            allowance,
            metrics,
        }
    }
}

impl<T> Drop for CappedIo<T> {
    fn drop(&mut self) {
        self.metrics.sockets.fetch_sub(1, Ordering::SeqCst);
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for CappedIo<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        if buf.remaining() == 0 {
            return Poll::Ready(Ok(()));
        }
        let left = this
            .allowance
            .load(Ordering::SeqCst)
            .saturating_sub(this.read);
        if left == 0 {
            return Poll::Ready(Err(io::Error::other("raw byte budget exhausted")));
        }
        let size = left.min(buf.remaining());
        let mut limited = ReadBuf::new(&mut buf.initialize_unfilled()[..size]);
        let result = Pin::new(&mut this.inner).poll_read(cx, &mut limited);
        let count = limited.filled().len();
        this.read += count;
        buf.advance(count);
        result
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for CappedIo<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let result = Pin::new(&mut this.inner).poll_write(cx, buf);
        if result.is_pending() && !buf.is_empty() {
            this.metrics.blocked_writes.fetch_add(1, Ordering::SeqCst);
        }
        result
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}
