use std::fmt::{self, Debug, Formatter};

use sqlx_core::io::BufStream;
use sqlx_core::{Close, Connect, Connection, DefaultRuntime, Runtime};

use crate::protocol::Capabilities;
use crate::{MySql, MySqlConnectOptions};

mod close;
mod connect;
mod ping;
mod stream;

#[allow(clippy::module_name_repetitions)]
pub struct MySqlConnection<Rt: Runtime = DefaultRuntime> {
    stream: BufStream<Rt::TcpStream>,
    connection_id: u32,

    // the capability flags are used by the client and server to indicate which
    // features they support and want to use.
    capabilities: Capabilities,

    // the sequence-id is incremented with each packet and may wrap around. It starts at 0 and is
    // reset to 0 when a new command begins in the Command Phase.
    sequence_id: u8,
}

impl<Rt: Runtime> MySqlConnection<Rt> {
    pub(crate) fn new(stream: Rt::TcpStream) -> Self {
        Self {
            stream: BufStream::with_capacity(stream, 4096, 1024),
            connection_id: 0,
            sequence_id: 0,
            capabilities: Capabilities::PROTOCOL_41 | Capabilities::LONG_PASSWORD
                | Capabilities::LONG_FLAG
                | Capabilities::IGNORE_SPACE
                | Capabilities::TRANSACTIONS
                | Capabilities::SECURE_CONNECTION
                // | Capabilities::MULTI_STATEMENTS
                // | Capabilities::MULTI_RESULTS
                // | Capabilities::PS_MULTI_RESULTS
                | Capabilities::PLUGIN_AUTH
                | Capabilities::PLUGIN_AUTH_LENENC_DATA
                // | Capabilities::CAN_HANDLE_EXPIRED_PASSWORDS
                // | Capabilities::SESSION_TRACK
                | Capabilities::DEPRECATE_EOF,
        }
    }
}

impl<Rt: Runtime> Debug for MySqlConnection<Rt> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("MySqlConnection").finish()
    }
}

impl<Rt: Runtime> Connection<Rt> for MySqlConnection<Rt> {
    type Database = MySql;

    #[cfg(feature = "async")]
    fn ping(&mut self) -> futures_util::future::BoxFuture<'_, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::AsyncRuntime,
        <Rt as Runtime>::TcpStream: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin,
    {
        Box::pin(self.ping_async())
    }
}

impl<Rt: Runtime> Connect<Rt> for MySqlConnection<Rt> {
    type Options = MySqlConnectOptions<Rt>;

    #[cfg(feature = "async")]
    fn connect(url: &str) -> futures_util::future::BoxFuture<'_, sqlx_core::Result<Self>>
    where
        Self: Sized,
        Rt: sqlx_core::AsyncRuntime,
        <Rt as Runtime>::TcpStream: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin,
    {
        use sqlx_core::ConnectOptions;

        let options = url.parse::<Self::Options>();
        Box::pin(async move { options?.connect().await })
    }
}

impl<Rt: Runtime> Close<Rt> for MySqlConnection<Rt> {
    #[cfg(feature = "async")]
    fn close(self) -> futures_util::future::BoxFuture<'static, sqlx_core::Result<()>>
    where
        Rt: sqlx_core::AsyncRuntime,
        <Rt as Runtime>::TcpStream: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin,
    {
        Box::pin(self.close_async())
    }
}

#[cfg(feature = "blocking")]
impl<Rt> sqlx_core::blocking::Connection<Rt> for MySqlConnection<Rt>
where
    Rt: sqlx_core::blocking::Runtime,
    <Rt as Runtime>::TcpStream: std::io::Read + std::io::Write,
{
    fn ping(&mut self) -> sqlx_core::Result<()> {
        <MySqlConnection<Rt>>::ping(self)
    }
}

#[cfg(feature = "blocking")]
impl<Rt> sqlx_core::blocking::Connect<Rt> for MySqlConnection<Rt>
where
    Rt: sqlx_core::blocking::Runtime,
{
    fn connect(url: &str) -> sqlx_core::Result<Self>
    where
        Self: Sized,
        <Rt as Runtime>::TcpStream: std::io::Read + std::io::Write,
    {
        Self::connect(&url.parse::<MySqlConnectOptions<Rt>>()?)
    }
}

#[cfg(feature = "blocking")]
impl<Rt> sqlx_core::blocking::Close<Rt> for MySqlConnection<Rt>
where
    Rt: sqlx_core::blocking::Runtime,
{
    fn close(self) -> sqlx_core::Result<()>
    where
        <Rt as Runtime>::TcpStream: std::io::Read + std::io::Write,
    {
        self.close()
    }
}
