//! Connection abstraction for testable network operations.

use crate::scheme::Scheme;
use cfg_if::cfg_if;
use std::io::{Read, Result, Write};
use std::net::TcpStream;

#[cfg(test)]
use std::cell::RefCell;
#[cfg(test)]
use std::io::Cursor;

/// Connection enum that provides either real TCP, TLS, or fake connections for testing.
pub enum Connection {
    /// Real TCP connection for production use.
    Real(TcpStream),
    /// Real TLS connection for HTTPS production use.
    Tls(Box<TlsConnection>),
    /// Fake connection for testing.
    #[cfg(test)]
    Fake(FakeConnection),
}

/// Wrapper for TLS connection components.
pub struct TlsConnection {
    client: rustls::ClientConnection,
    socket: TcpStream,
}

impl Connection {
    /// Connects to the given address.
    /// Returns a fake connection in tests, real TCP connection in production.
    pub fn connect(addr: &str) -> Result<Self> {
        cfg_if! {
            if #[cfg(test)] {
                // In tests, ignore the address and return fake connection
                let _ = addr;
                Ok(Connection::Fake(FakeConnection::new()))
            } else {
                TcpStream::connect(addr).map(Connection::Real)
            }
        }
    }

    /// Creates a TLS connection to the given address.
    /// Returns a fake connection in tests, real TLS connection in production.
    pub fn connect_tls(addr: &str, _scheme: &Scheme) -> Result<Self> {
        cfg_if! {
            if #[cfg(test)] {
                // In tests, ignore the address and scheme, return fake connection
                let _ = addr;
                Ok(Connection::Fake(FakeConnection::new()))
            } else {
                // Extract hostname from address
                let host = addr.split(':').next().ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid address format")
                })?;

                // Create TLS configuration
                let root_store = rustls::RootCertStore {
                    roots: webpki_roots::TLS_SERVER_ROOTS.into(),
                };

                let config = rustls::ClientConfig::builder()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();

                // Connect TCP stream
                let tcp_stream = TcpStream::connect(addr)?;

                // Create TLS connection
                let server_name =
                    rustls::pki_types::ServerName::try_from(host.to_string()).map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Invalid hostname: {}", e),
                        )
                    })?;

                let client_conn =
                    rustls::ClientConnection::new(std::sync::Arc::new(config), server_name)
                        .map_err(|e| std::io::Error::other(format!("TLS connection error: {}", e)))?;

                let tls_conn = TlsConnection {
                    client: client_conn,
                    socket: tcp_stream,
                };

                Ok(Connection::Tls(Box::new(tls_conn)))
            }
        }
    }

    /// Gets the written data (only available for fake connections in tests).
    #[cfg(test)]
    pub fn get_written_data(&self) -> Option<Vec<u8>> {
        match self {
            Connection::Fake(fake) => Some(fake.written_data.borrow().clone()),
            _ => None,
        }
    }

    /// Sets the response data for fake connections in tests.
    #[cfg(test)]
    pub fn set_response_data(&mut self, data: Vec<u8>) {
        if let Connection::Fake(fake) = self {
            fake.set_response_data(data);
        }
    }
}

impl Read for Connection {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            Connection::Real(stream) => stream.read(buf),
            Connection::Tls(tls_conn) => {
                let mut stream = rustls::Stream::new(&mut tls_conn.client, &mut tls_conn.socket);
                stream.read(buf)
            }
            #[cfg(test)]
            Connection::Fake(fake) => fake.read_data.read(buf),
        }
    }
}

impl Write for Connection {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self {
            Connection::Real(stream) => stream.write(buf),
            Connection::Tls(tls_conn) => {
                let mut stream = rustls::Stream::new(&mut tls_conn.client, &mut tls_conn.socket);
                stream.write(buf)
            }
            #[cfg(test)]
            Connection::Fake(fake) => {
                fake.written_data.borrow_mut().extend_from_slice(buf);
                Ok(buf.len())
            }
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            Connection::Real(stream) => stream.flush(),
            Connection::Tls(tls_conn) => {
                let mut stream = rustls::Stream::new(&mut tls_conn.client, &mut tls_conn.socket);
                stream.flush()
            }
            #[cfg(test)]
            Connection::Fake(_) => Ok(()),
        }
    }
}

/// Fake connection for testing purposes.
#[cfg(test)]
pub struct FakeConnection {
    written_data: RefCell<Vec<u8>>,
    read_data: Cursor<Vec<u8>>,
}

#[cfg(test)]
impl FakeConnection {
    /// Creates a new fake connection.
    fn new() -> Self {
        FakeConnection {
            written_data: RefCell::new(Vec::new()),
            read_data: Cursor::new(vec![]),
        }
    }

    /// Creates a fake connection with specific data to be read.
    pub fn with_read_data(data: Vec<u8>) -> Self {
        FakeConnection {
            written_data: RefCell::new(Vec::new()),
            read_data: Cursor::new(data),
        }
    }

    /// Sets the response data that will be read from this connection.
    pub fn set_response_data(&mut self, data: Vec<u8>) {
        self.read_data = Cursor::new(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheme::Scheme;

    #[test]
    fn test_connection_connect_returns_fake_in_tests() {
        let conn = Connection::connect("example.com:80").unwrap();

        // In tests, this should be a fake connection
        match conn {
            Connection::Fake(_) => (),
            _ => panic!("Expected fake connection in tests"),
        }
    }

    #[test]
    fn test_fake_connection_captures_written_data() {
        let mut conn = Connection::connect("example.com:80").unwrap();
        let data = b"GET / HTTP/1.0\r\n";

        conn.write_all(data).unwrap();

        let written = conn.get_written_data().unwrap();
        assert_eq!(written, data);
    }

    #[test]
    fn test_fake_connection_accumulates_writes() {
        let mut conn = Connection::connect("example.com:80").unwrap();

        conn.write_all(b"GET ").unwrap();
        conn.write_all(b"/ ").unwrap();
        conn.write_all(b"HTTP/1.0\r\n").unwrap();

        let written = conn.get_written_data().unwrap();
        assert_eq!(written, b"GET / HTTP/1.0\r\n");
    }

    #[test]
    fn test_fake_connection_with_read_data() {
        let response = b"HTTP/1.0 200 OK\r\n\r\nHello";
        let fake = FakeConnection::with_read_data(response.to_vec());
        let mut conn = Connection::Fake(fake);

        let mut buf = [0u8; 100];
        let n = conn.read(&mut buf).unwrap();

        assert_eq!(&buf[..n], response);
    }

    #[test]
    fn test_connection_connect_tls_returns_fake_in_tests() {
        let conn = Connection::connect_tls("example.com:443", &Scheme::Https).unwrap();

        // In tests, this should be a fake connection
        match conn {
            Connection::Fake(_) => (),
            _ => panic!("Expected fake connection in tests"),
        }
    }
}
