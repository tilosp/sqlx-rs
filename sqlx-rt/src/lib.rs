#[cfg(not(any(
    feature = "runtime-actix-native-tls",
    feature = "runtime-async-std-native-tls",
    feature = "runtime-tokio-native-tls",
)))]
compile_error!(
    "one of 'runtime-actix-native-tls', 'runtime-async-std-native-tls' or 'runtime-tokio-native-tls' features must be enabled"
);

#[cfg(any(
    all(
        feature = "runtime-actix-native-tls",
        feature = "runtime-async-std-native-tls"
    ),
    all(
        feature = "runtime-actix-native-tls",
        feature = "runtime-tokio-native-tls"
    ),
    all(
        feature = "runtime-async-std-native-tls",
        feature = "runtime-tokio-native-tls"
    ),
))]
compile_error!(
    "only one of 'runtime-actix-native-tls', 'runtime-async-std-native-tls' or 'runtime-tokio-native-tls' features can be enabled"
);

pub use native_tls;

//
// Actix *OR* Tokio
//

#[cfg(all(
    not(feature = "shared-async-std"),
    any(feature = "shared-tokio", feature = "shared-actix"),
))]
pub use tokio::{
    self, fs, io::AsyncRead, io::AsyncReadExt, io::AsyncWrite, io::AsyncWriteExt, net::TcpStream,
    task::spawn, task::yield_now, time::delay_for as sleep, time::timeout,
};

#[cfg(all(
    unix,
    not(feature = "shared-async-std"),
    any(feature = "shared-tokio", feature = "shared-actix"),
))]
pub use tokio::net::UnixStream;

//
// tokio
//

#[cfg(all(
    feature = "shared-tokio",
    not(any(feature = "shared-actix", feature = "shared-async-std",))
))]
#[macro_export]
macro_rules! blocking {
    ($($expr:tt)*) => {
        $crate::tokio::task::block_in_place(move || { $($expr)* })
    };
}

#[cfg(all(feature = "tokio-native-tls", not(feature = "async-native-tls")))]
pub use tokio_native_tls::{TlsConnector, TlsStream};

#[cfg(all(feature = "tokio-native-tls", not(feature = "async-native-tls")))]
pub use native_tls::Error as TlsError;

//
// actix
//

#[cfg(feature = "shared-actix")]
pub use {actix_rt, actix_threadpool};

#[cfg(all(
    feature = "shared-actix",
    not(any(feature = "shared-tokio", feature = "shared-async-std",))
))]
#[macro_export]
macro_rules! blocking {
    ($($expr:tt)*) => {
        $crate::actix_threadpool::run(move || { $($expr)* }).await.map_err(|err| match err {
            $crate::actix_threadpool::BlockingError::Error(e) => e,
            $crate::actix_threadpool::BlockingError::Canceled => panic!("{}", err)
        })
    };
}

//
// async-std
//

#[cfg(all(
    feature = "shared-async-std",
    not(any(feature = "shared-actix", feature = "shared-tokio",))
))]
pub use async_std::{
    self, fs, future::timeout, io::prelude::ReadExt as AsyncReadExt,
    io::prelude::WriteExt as AsyncWriteExt, io::Read as AsyncRead, io::Write as AsyncWrite,
    net::TcpStream, task::sleep, task::spawn, task::yield_now,
};

#[cfg(all(
    feature = "shared-async-std",
    not(any(feature = "shared-actix", feature = "shared-tokio",))
))]
#[macro_export]
macro_rules! blocking {
    ($($expr:tt)*) => {
        $crate::async_std::task::spawn_blocking(move || { $($expr)* }).await
    };
}

#[cfg(all(
    unix,
    feature = "shared-async-std",
    not(any(feature = "shared-actix", feature = "shared-tokio",))
))]
pub use async_std::os::unix::net::UnixStream;

#[cfg(all(feature = "async-native-tls", not(feature = "tokio-native-tls")))]
pub use async_native_tls::{Error as TlsError, TlsConnector, TlsStream};

#[cfg(all(
    feature = "shared-async-std",
    not(any(feature = "shared-actix", feature = "shared-tokio"))
))]
pub use async_std::task::block_on;

#[cfg(all(
    feature = "shared-async-std",
    not(any(feature = "shared-actix", feature = "shared-tokio"))
))]
pub fn enter_runtime<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // no-op for async-std
    f()
}

#[cfg(all(
    any(feature = "shared-tokio", feature = "shared-actix"),
    not(feature = "shared-async-std")
))]
pub use tokio_runtime::{block_on, enter_runtime};

#[cfg(any(feature = "shared-tokio", feature = "shared-actix"))]
mod tokio_runtime {
    use once_cell::sync::Lazy;
    use tokio::runtime::{self, Runtime};

    // lazily initialize a global runtime once for multiple invocations of the macros
    static RUNTIME: Lazy<Runtime> = Lazy::new(|| {
        runtime::Builder::new()
            // `.basic_scheduler()` requires calling `Runtime::block_on()` which needs mutability
            .threaded_scheduler()
            .enable_io()
            .enable_time()
            .build()
            .expect("failed to initialize Tokio runtime")
    });

    #[cfg(any(feature = "shared-tokio", feature = "shared-actix"))]
    pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
        RUNTIME.enter(|| RUNTIME.handle().block_on(future))
    }

    pub fn enter_runtime<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        RUNTIME.enter(f)
    }
}
