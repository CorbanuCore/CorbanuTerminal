//! Cross-platform async Unix domain socket helpers.

use std::io::Result as IoResult;
use std::path::Path;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::ReadBuf;

/// Creates `socket_dir` if needed and restricts it to the current user where
/// the platform exposes Unix permissions.
pub async fn prepare_private_socket_directory(socket_dir: impl AsRef<Path>) -> IoResult<()> {
    platform::prepare_private_socket_directory(socket_dir.as_ref()).await
}

/// Returns whether `socket_path` points at a stale Unix socket rendezvous path.
///
/// On Unix this checks the file type. On Windows, `uds_windows` represents the
/// rendezvous as a regular path, so existence is the only useful stale-path
/// signal available.
pub async fn is_stale_socket_path(socket_path: impl AsRef<Path>) -> IoResult<bool> {
    platform::is_stale_socket_path(socket_path.as_ref()).await
}

/// Async Unix domain socket listener.
pub struct UnixListener {
    inner: platform::Listener,
}

impl UnixListener {
    /// Binds a new listener at `socket_path`.
    pub async fn bind(socket_path: impl AsRef<Path>) -> IoResult<Self> {
        platform::bind_listener(socket_path.as_ref())
            .await
            .map(|inner| Self { inner })
    }

    /// Accepts the next incoming stream.
    pub async fn accept(&mut self) -> IoResult<UnixStream> {
        self.inner.accept().await.map(|inner| UnixStream { inner })
    }
}

/// Async Unix domain socket stream.
pub struct UnixStream {
    inner: platform::Stream,
}

impl UnixStream {
    /// Connects to `socket_path`.
    pub async fn connect(socket_path: impl AsRef<Path>) -> IoResult<Self> {
        platform::connect_stream(socket_path.as_ref())
            .await
            .map(|inner| Self { inner })
    }

    /// Returns the authenticated peer user ID where the platform exposes it.
    pub fn peer_user_id(&self) -> IoResult<Option<u32>> {
        platform::peer_user_id(&self.inner)
    }
}

impl AsyncRead for UnixStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        Pin::new(&mut self.get_mut().inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for UnixStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<IoResult<usize>> {
        Pin::new(&mut self.get_mut().inner).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}

#[cfg(unix)]
mod platform {
    use std::io;
    use std::io::ErrorKind;
    use std::io::Result as IoResult;
    use std::os::unix::fs::FileTypeExt;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    use tokio::fs;
    use tokio::net::UnixListener;
    use tokio::net::UnixStream;

    /// Owner-only access keeps the control socket directory private while
    /// preserving owner traversal and socket path creation.
    const SOCKET_DIR_MODE: u32 = 0o700;
    const SOCKET_DIR_PERMISSION_BITS: u32 = 0o777;

    pub(super) type Stream = UnixStream;

    pub(super) struct Listener(UnixListener);

    pub(super) async fn prepare_private_socket_directory(socket_dir: &Path) -> IoResult<()> {
        let mut dir_builder = fs::DirBuilder::new();
        dir_builder.mode(SOCKET_DIR_MODE);
        match dir_builder.create(socket_dir).await {
            Ok(()) => return Ok(()),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {}
            Err(err) => return Err(err),
        }

        let metadata = fs::symlink_metadata(socket_dir).await?;
        if !metadata.is_dir() {
            return Err(io::Error::new(
                ErrorKind::AlreadyExists,
                format!(
                    "socket directory path exists and is not a directory: {}",
                    socket_dir.display()
                ),
            ));
        }

        let permissions = metadata.permissions();
        // The SSH-over-UDS control socket is reachable by path, so the
        // rendezvous directory must be owner-traversable while denying
        // group/other access; exact 0700 fixes insecure modes and unusable
        // owner-only modes like 0600.
        if permissions.mode() & SOCKET_DIR_PERMISSION_BITS != SOCKET_DIR_MODE {
            fs::set_permissions(socket_dir, std::fs::Permissions::from_mode(SOCKET_DIR_MODE))
                .await?;
        }

        Ok(())
    }

    pub(super) async fn bind_listener(socket_path: &Path) -> IoResult<Listener> {
        UnixListener::bind(socket_path).map(Listener)
    }

    impl Listener {
        pub(super) async fn accept(&mut self) -> IoResult<Stream> {
            self.0.accept().await.map(|(stream, _addr)| stream)
        }
    }

    pub(super) async fn connect_stream(socket_path: &Path) -> IoResult<Stream> {
        UnixStream::connect(socket_path).await
    }

    pub(super) fn peer_user_id(stream: &Stream) -> IoResult<Option<u32>> {
        stream
            .peer_cred()
            .map(|credentials| Some(credentials.uid()))
    }

    pub(super) async fn is_stale_socket_path(socket_path: &Path) -> IoResult<bool> {
        Ok(fs::symlink_metadata(socket_path)
            .await?
            .file_type()
            .is_socket())
    }
}

#[cfg(windows)]
mod platform {
    use std::ffi::OsString;
    use std::io;
    use std::io::Result as IoResult;
    use std::net::Shutdown;
    use std::ops::Deref;
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::ffi::OsStringExt;
    use std::os::windows::io::AsRawSocket;
    use std::os::windows::io::AsSocket;
    use std::os::windows::io::BorrowedSocket;
    use std::path::Path;
    use std::pin::Pin;
    use std::task::Context;
    use std::task::Poll;
    use std::task::ready;

    use async_io::Async;
    use tokio::io::AsyncRead;
    use tokio::io::AsyncWrite;
    use tokio::io::ReadBuf;
    use tokio::task;
    use tokio_util::compat::Compat;
    use tokio_util::compat::FuturesAsyncReadCompatExt;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::Foundation::HLOCAL;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Authorization::ConvertSidToStringSidW;
    use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
    use windows_sys::Win32::Security::DACL_SECURITY_INFORMATION;
    use windows_sys::Win32::Security::GetTokenInformation;
    use windows_sys::Win32::Security::PROTECTED_DACL_SECURITY_INFORMATION;
    use windows_sys::Win32::Security::PSECURITY_DESCRIPTOR;
    use windows_sys::Win32::Security::SetFileSecurityW;
    use windows_sys::Win32::Security::TOKEN_QUERY;
    use windows_sys::Win32::Security::TOKEN_USER;
    use windows_sys::Win32::Security::TokenUser;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    use windows_sys::Win32::System::Threading::OpenProcessToken;

    pub(super) struct Stream(Compat<Async<WindowsUnixStream>>);

    pub(super) async fn prepare_private_socket_directory(socket_dir: &Path) -> IoResult<()> {
        tokio::fs::create_dir_all(socket_dir).await?;
        apply_current_user_only_acl(socket_dir)
    }

    /// Windows AF_UNIX does not expose peer credentials. Protect the rendezvous
    /// directory with a non-inheriting DACL before the listener is bound so only
    /// this account (and LocalSystem) can reach the socket path.
    fn apply_current_user_only_acl(path: &Path) -> IoResult<()> {
        let current_user_sid = current_user_sid_string()?;
        let sddl = wide_null(format!("D:P(A;;FA;;;SY)(A;;FA;;;{current_user_sid})"));
        let mut security_descriptor: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
        let converted = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                1, // SDDL_REVISION_1
                &mut security_descriptor,
                std::ptr::null_mut(),
            )
        };
        if converted == 0 {
            return Err(io::Error::last_os_error());
        }

        let path = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let applied = unsafe {
            SetFileSecurityW(
                path.as_ptr(),
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                security_descriptor,
            )
        };
        unsafe {
            LocalFree(security_descriptor as HLOCAL);
        }
        if applied == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn current_user_sid_string() -> IoResult<String> {
        let mut token = 0;
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let token = OwnedHandle(token);

        let mut required = 0;
        unsafe {
            GetTokenInformation(token.0, TokenUser, std::ptr::null_mut(), 0, &mut required);
        }
        if required == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut buffer = vec![0_u8; required as usize];
        if unsafe {
            GetTokenInformation(
                token.0,
                TokenUser,
                buffer.as_mut_ptr().cast(),
                required,
                &mut required,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        if buffer.len() < std::mem::size_of::<TOKEN_USER>() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "current-user token did not contain TOKEN_USER",
            ));
        }
        let token_user = unsafe { std::ptr::read_unaligned(buffer.as_ptr().cast::<TOKEN_USER>()) };
        let mut string_sid = std::ptr::null_mut();
        if unsafe { ConvertSidToStringSidW(token_user.User.Sid, &mut string_sid) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let length = unsafe {
            let mut length = 0;
            while *string_sid.add(length) != 0 {
                length += 1;
            }
            length
        };
        let sid = unsafe { OsString::from_wide(std::slice::from_raw_parts(string_sid, length)) }
            .into_string()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "user SID was not UTF-16"));
        unsafe {
            LocalFree(string_sid as HLOCAL);
        }
        sid
    }

    fn wide_null(value: String) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    struct OwnedHandle(HANDLE);

    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            if self.0 != 0 {
                unsafe {
                    CloseHandle(self.0);
                }
            }
        }
    }

    pub(super) struct Listener(Async<WindowsUnixListener>);

    pub(super) async fn bind_listener(socket_path: &Path) -> IoResult<Listener> {
        let socket_path = socket_path.to_path_buf();
        let listener =
            spawn_blocking_io(move || uds_windows::UnixListener::bind(socket_path)).await?;
        Async::new(WindowsUnixListener::from(listener)).map(Listener)
    }

    impl Listener {
        pub(super) async fn accept(&mut self) -> IoResult<Stream> {
            let (stream, _addr) = self.0.read_with(|listener| listener.accept()).await?;
            Async::new(WindowsUnixStream::from(stream))
                .map(FuturesAsyncReadCompatExt::compat)
                .map(Stream)
        }
    }

    pub(super) async fn connect_stream(socket_path: &Path) -> IoResult<Stream> {
        let socket_path = socket_path.to_path_buf();
        let stream =
            spawn_blocking_io(move || uds_windows::UnixStream::connect(socket_path)).await?;
        Async::new(WindowsUnixStream::from(stream))
            .map(FuturesAsyncReadCompatExt::compat)
            .map(Stream)
    }

    pub(super) fn peer_user_id(_stream: &Stream) -> IoResult<Option<u32>> {
        Ok(None)
    }

    pub(super) async fn is_stale_socket_path(socket_path: &Path) -> IoResult<bool> {
        tokio::fs::try_exists(socket_path).await
    }

    async fn spawn_blocking_io<T>(
        operation: impl FnOnce() -> IoResult<T> + Send + 'static,
    ) -> IoResult<T>
    where
        T: Send + 'static,
    {
        task::spawn_blocking(operation)
            .await
            .map_err(|err| io::Error::other(format!("blocking socket task failed: {err}")))?
    }

    pub(super) struct WindowsUnixListener(uds_windows::UnixListener);

    impl From<uds_windows::UnixListener> for WindowsUnixListener {
        fn from(listener: uds_windows::UnixListener) -> Self {
            Self(listener)
        }
    }

    impl Deref for WindowsUnixListener {
        type Target = uds_windows::UnixListener;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl AsSocket for WindowsUnixListener {
        fn as_socket(&self) -> BorrowedSocket<'_> {
            unsafe { BorrowedSocket::borrow_raw(self.as_raw_socket()) }
        }
    }

    pub(super) struct WindowsUnixStream(uds_windows::UnixStream);

    impl From<uds_windows::UnixStream> for WindowsUnixStream {
        fn from(stream: uds_windows::UnixStream) -> Self {
            Self(stream)
        }
    }

    impl Deref for WindowsUnixStream {
        type Target = uds_windows::UnixStream;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl AsSocket for WindowsUnixStream {
        fn as_socket(&self) -> BorrowedSocket<'_> {
            unsafe { BorrowedSocket::borrow_raw(self.as_raw_socket()) }
        }
    }

    impl io::Read for WindowsUnixStream {
        fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
            io::Read::read(&mut self.0, buf)
        }
    }

    impl io::Write for WindowsUnixStream {
        fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
            io::Write::write(&mut self.0, buf)
        }

        fn flush(&mut self) -> IoResult<()> {
            io::Write::flush(&mut self.0)
        }
    }

    impl AsyncRead for Stream {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<IoResult<()>> {
            Pin::new(&mut self.get_mut().0).poll_read(cx, buf)
        }
    }

    impl AsyncWrite for Stream {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<IoResult<usize>> {
            Pin::new(&mut self.get_mut().0).poll_write(cx, buf)
        }

        fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
            Pin::new(&mut self.get_mut().0).poll_flush(cx)
        }

        fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
            let stream = &mut self.get_mut().0;
            ready!(Pin::new(&mut *stream).poll_flush(cx))?;
            // `Compat<Async<_>>` maps shutdown to `poll_close()`, which only
            // flushes for `async_io::Async`; call the socket shutdown directly.
            stream.get_ref().get_ref().shutdown(Shutdown::Write)?;
            Poll::Ready(Ok(()))
        }
    }

    unsafe impl async_io::IoSafe for WindowsUnixListener {}
    unsafe impl async_io::IoSafe for WindowsUnixStream {}
}

#[cfg(test)]
mod lib_tests;
