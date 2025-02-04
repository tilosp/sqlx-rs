#![allow(dead_code)]

use std::io;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

use sqlx_rt::{AsyncRead, AsyncWrite, TlsConnector, TlsStream};

use crate::error::Error;
use std::mem::replace;

pub enum MaybeTlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    Raw(S),
    Tls(TlsStream<S>),
    Upgrading,
}

impl<S> MaybeTlsStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    #[inline]
    pub fn is_tls(&self) -> bool {
        matches!(self, Self::Tls(_))
    }

    pub async fn upgrade(&mut self, host: &str, connector: TlsConnector) -> Result<(), Error> {
        let stream = match replace(self, MaybeTlsStream::Upgrading) {
            MaybeTlsStream::Raw(stream) => stream,

            MaybeTlsStream::Tls(_) => {
                // ignore upgrade, we are already a TLS connection
                return Ok(());
            }

            MaybeTlsStream::Upgrading => {
                // we previously failed to upgrade and now hold no connection
                // this should only happen from an internal misuse of this method
                return Err(Error::Io(io::ErrorKind::ConnectionAborted.into()));
            }
        };

        *self = MaybeTlsStream::Tls(
            connector
                .connect(host, stream)
                .await
                .map_err(|err| Error::Tls(err.into()))?,
        );

        Ok(())
    }
}

impl<S> AsyncRead for MaybeTlsStream<S>
where
    S: Unpin + AsyncWrite + AsyncRead,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self {
            MaybeTlsStream::Raw(s) => Pin::new(s).poll_read(cx, buf),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_read(cx, buf),

            MaybeTlsStream::Upgrading => Poll::Ready(Err(io::ErrorKind::ConnectionAborted.into())),
        }
    }

    #[cfg(any(
        feature = "runtime-actix-native-tls",
        feature = "runtime-tokio-native-tls"
    ))]
    fn poll_read_buf<B>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut B,
    ) -> Poll<io::Result<usize>>
    where
        Self: Sized,
        B: bytes::BufMut,
    {
        match &mut *self {
            MaybeTlsStream::Raw(s) => Pin::new(s).poll_read_buf(cx, buf),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_read_buf(cx, buf),

            MaybeTlsStream::Upgrading => Poll::Ready(Err(io::ErrorKind::ConnectionAborted.into())),
        }
    }
}

impl<S> AsyncWrite for MaybeTlsStream<S>
where
    S: Unpin + AsyncWrite + AsyncRead,
{
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self {
            MaybeTlsStream::Raw(s) => Pin::new(s).poll_write(cx, buf),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_write(cx, buf),

            MaybeTlsStream::Upgrading => Poll::Ready(Err(io::ErrorKind::ConnectionAborted.into())),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            MaybeTlsStream::Raw(s) => Pin::new(s).poll_flush(cx),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_flush(cx),

            MaybeTlsStream::Upgrading => Poll::Ready(Err(io::ErrorKind::ConnectionAborted.into())),
        }
    }

    #[cfg(any(
        feature = "runtime-actix-native-tls",
        feature = "runtime-tokio-native-tls"
    ))]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            MaybeTlsStream::Raw(s) => Pin::new(s).poll_shutdown(cx),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_shutdown(cx),

            MaybeTlsStream::Upgrading => Poll::Ready(Err(io::ErrorKind::ConnectionAborted.into())),
        }
    }

    #[cfg(feature = "runtime-async-std-native-tls")]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            MaybeTlsStream::Raw(s) => Pin::new(s).poll_close(cx),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_close(cx),

            MaybeTlsStream::Upgrading => Poll::Ready(Err(io::ErrorKind::ConnectionAborted.into())),
        }
    }

    #[cfg(any(
        feature = "runtime-actix-native-tls",
        feature = "runtime-tokio-native-tls"
    ))]
    fn poll_write_buf<B>(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut B,
    ) -> Poll<io::Result<usize>>
    where
        Self: Sized,
        B: bytes::Buf,
    {
        match &mut *self {
            MaybeTlsStream::Raw(s) => Pin::new(s).poll_write_buf(cx, buf),
            MaybeTlsStream::Tls(s) => Pin::new(s).poll_write_buf(cx, buf),

            MaybeTlsStream::Upgrading => Poll::Ready(Err(io::ErrorKind::ConnectionAborted.into())),
        }
    }
}

impl<S> Deref for MaybeTlsStream<S>
where
    S: Unpin + AsyncWrite + AsyncRead,
{
    type Target = S;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeTlsStream::Raw(s) => s,

            #[cfg(not(feature = "runtime-async-std-native-tls"))]
            MaybeTlsStream::Tls(s) => s.get_ref().get_ref().get_ref(),

            #[cfg(feature = "runtime-async-std-native-tls")]
            MaybeTlsStream::Tls(s) => s.get_ref(),

            MaybeTlsStream::Upgrading => panic!(io::Error::from(io::ErrorKind::ConnectionAborted)),
        }
    }
}

impl<S> DerefMut for MaybeTlsStream<S>
where
    S: Unpin + AsyncWrite + AsyncRead,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MaybeTlsStream::Raw(s) => s,

            #[cfg(not(feature = "runtime-async-std-native-tls"))]
            MaybeTlsStream::Tls(s) => s.get_mut().get_mut().get_mut(),

            #[cfg(feature = "runtime-async-std-native-tls")]
            MaybeTlsStream::Tls(s) => s.get_mut(),

            MaybeTlsStream::Upgrading => panic!(io::Error::from(io::ErrorKind::ConnectionAborted)),
        }
    }
}
