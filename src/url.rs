//! URL parsing and representation.

use crate::connection::Connection;
use crate::host::Host;
use crate::path::Path;
use crate::port::Port;
use crate::response::ResponseBody;
use crate::scheme::Scheme;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};

/// A parsed URL.
///
/// # Examples
/// ```
/// use web_browser_engineering_rs::url::Url;
///
/// let url = Url::new("http://example.com").unwrap();
/// assert_eq!(url.scheme, web_browser_engineering_rs::scheme::Scheme::Http);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    pub scheme: Scheme,
    pub host: Host,
    pub port: Option<Port>,
    pub path: Path,
}

impl Url {
    /// Parses a URL string.
    ///
    /// Follows RFC 3986 URL format using "://" as the scheme separator.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL format is invalid or the scheme is unsupported.
    ///
    /// # Examples
    /// ```
    /// use web_browser_engineering_rs::url::Url;
    ///
    /// assert!(Url::new("http://example.com").is_ok());
    /// assert!(Url::new("https://example.com").is_ok());
    /// assert!(Url::new("http://example.com:8080/path").is_ok());
    ///
    /// assert!(Url::new("example.com").is_err());         // Missing scheme
    /// ```
    pub fn new(s: &str) -> Result<Self> {
        // splitn(2) to handle "://" in query parameters correctly
        let parts: Vec<&str> = s.splitn(2, "://").collect();

        if parts.len() != 2 {
            return Err(anyhow!("Invalid URL format: missing '://' separator"));
        }

        let scheme_str = parts[0];
        if scheme_str.is_empty() {
            return Err(anyhow!("Invalid URL: empty scheme"));
        }

        let scheme = scheme_str.parse::<Scheme>()?;

        // Parse host and path from the remainder
        let remainder = parts[1];
        if remainder.is_empty() {
            return Err(anyhow!("Invalid URL: empty host"));
        }

        // Find the first '/' to split host:port and path
        let (host_port, path) = if let Some(slash_pos) = remainder.find('/') {
            let host_port = &remainder[..slash_pos];
            let path = &remainder[slash_pos..];
            (host_port, path.to_string())
        } else {
            // No path, default to "/"
            (remainder, "/".to_string())
        };

        // Parse host and port
        let (host, port) = if let Some(colon_pos) = host_port.find(':') {
            let host = &host_port[..colon_pos];
            let port_str = &host_port[colon_pos + 1..];

            let port = port_str
                .parse::<u16>()
                .map_err(|_| anyhow!("Invalid port number: {}", port_str))?;

            (Host::new(host), Some(Port::new(port)))
        } else {
            (Host::new(host_port), None)
        };

        if host.as_str().is_empty() {
            return Err(anyhow!("Invalid URL: empty host"));
        }

        Ok(Url {
            scheme,
            host,
            port,
            path: Path::new(path),
        })
    }

    /// Gets the default port for the URL's scheme.
    pub fn get_default_port(&self) -> u16 {
        match self.scheme {
            Scheme::Http => 80,
            Scheme::Https => 443,
        }
    }

    /// Opens a connection, sends an HTTP request, and processes the response.
    /// Returns the response body for 200 OK responses, or an error for other status codes.
    /// Uses the parsed port if available, otherwise defaults based on scheme (HTTP: 80, HTTPS: 443).
    pub fn request(&self) -> Result<ResponseBody> {
        let port = self
            .port
            .map(|p| p.value())
            .unwrap_or_else(|| self.get_default_port());
        let addr = format!("{}:{}", self.host.as_str(), port);

        let conn = match self.scheme {
            Scheme::Http => {
                Connection::connect(&addr).map_err(|e| anyhow!("Connection failed: {}", e))?
            }
            Scheme::Https => Connection::connect_tls(&addr, &self.scheme)
                .map_err(|e| anyhow!("TLS connection failed: {}", e))?,
        };

        self.request_with_connection(conn)
    }

    /// Sends an HTTP request using the provided connection and processes the response.
    /// This is separated to allow for testing with mock connections.
    pub fn request_with_connection(&self, mut conn: Connection) -> Result<ResponseBody> {
        // Send HTTP request
        let request = format!(
            "GET {} HTTP/1.0\r\nHost: {}\r\n\r\n",
            self.path.as_str(),
            self.host.as_str()
        );

        conn.write_all(request.as_bytes())
            .map_err(|e| anyhow!("Failed to send HTTP request: {}", e))?;

        // Parse response
        let mut reader = BufReader::new(conn);

        // Read status line
        let mut status_line = String::new();
        reader
            .read_line(&mut status_line)
            .map_err(|e| anyhow!("Failed to read status line: {}", e))?;

        // Remove trailing CRLF
        let status_line = status_line.trim_end();

        // Parse status line: HTTP/1.0 200 OK
        let parts: Vec<&str> = status_line.splitn(3, ' ').collect();

        // Use .get() for safe array access without panic risk
        let _http_version = parts
            .first()
            .ok_or_else(|| anyhow!("Missing HTTP version in status line: {}", status_line))?;
        let status_code_str = parts
            .get(1)
            .ok_or_else(|| anyhow!("Missing status code in status line: {}", status_line))?;
        let status_text = parts
            .get(2)
            .ok_or_else(|| anyhow!("Missing status text in status line: {}", status_line))?;

        let status_code: u16 = status_code_str
            .parse()
            .map_err(|_| anyhow!("Invalid status code: {}", status_code_str))?;

        // Read headers
        let mut headers = HashMap::new();
        loop {
            let mut header_line = String::new();
            reader
                .read_line(&mut header_line)
                .map_err(|e| anyhow!("Failed to read header: {}", e))?;

            // Check for empty line (end of headers)
            let header_line = header_line.trim_end();
            if header_line.is_empty() {
                break;
            }

            // Parse header: name: value
            if let Some(colon_pos) = header_line.find(':') {
                let name = header_line[..colon_pos].trim().to_lowercase();
                let value = header_line[colon_pos + 1..].trim().to_string();
                headers.insert(name, value);
            }
        }

        // Read body only for 200 OK
        if status_code == 200 {
            let mut body = String::new();
            reader
                .read_to_string(&mut body)
                .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

            Ok(ResponseBody::new(body))
        } else {
            // For non-200 responses, return an error with details
            Err(anyhow!(
                "HTTP {} {}: Response code indicates failure",
                status_code,
                status_text
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_new_valid_http() {
        let result = Url::new("http://example.com");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme, Scheme::Http);
        assert_eq!(url.host.as_str(), "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path.as_str(), "/");
    }

    #[test]
    fn test_url_new_valid_http_with_path() {
        let result = Url::new("http://example.com/path/to/resource");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme, Scheme::Http);
        assert_eq!(url.host.as_str(), "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path.as_str(), "/path/to/resource");
    }

    #[test]
    fn test_url_parse_host_and_path() {
        let result = Url::new("http://example.org/index.html");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.host.as_str(), "example.org");
        assert_eq!(url.port, None);
        assert_eq!(url.path.as_str(), "/index.html");
    }

    #[test]
    fn test_url_with_port() {
        let result = Url::new("http://example.com:8080");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme, Scheme::Http);
        assert_eq!(url.host.as_str(), "example.com");
        assert_eq!(url.port, Some(Port::new(8080)));
        assert_eq!(url.path.as_str(), "/");
    }

    #[test]
    fn test_url_with_port_and_path() {
        let result = Url::new("http://example.com:3000/api/users");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.host.as_str(), "example.com");
        assert_eq!(url.port, Some(Port::new(3000)));
        assert_eq!(url.path.as_str(), "/api/users");
    }

    #[test]
    fn test_url_example_org_8080_index_html() {
        let result = Url::new("http://example.org:8080/index.html");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme, Scheme::Http);
        assert_eq!(url.host.as_str(), "example.org");
        assert_eq!(url.port, Some(Port::new(8080)));
        assert_eq!(url.path.as_str(), "/index.html");
    }

    #[test]
    fn test_url_invalid_port() {
        let result = Url::new("http://example.com:not_a_number/path");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid port number")
        );
    }

    #[test]
    fn test_url_port_out_of_range() {
        let result = Url::new("http://example.com:99999/path");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid port number")
        );
    }

    #[test]
    fn test_url_new_valid_https() {
        let result = Url::new("https://example.com");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme, Scheme::Https);
        assert_eq!(url.host.as_str(), "example.com");
        assert_eq!(url.port, None);
        assert_eq!(url.path.as_str(), "/");
    }

    #[test]
    fn test_url_new_invalid_ftp() {
        let result = Url::new("ftp://example.com");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported scheme: ftp")
        );
    }

    #[test]
    fn test_url_new_missing_separator() {
        let result = Url::new("http:example.com");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("missing '://' separator")
        );
    }

    #[test]
    fn test_url_new_no_scheme() {
        let result = Url::new("example.com");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("missing '://' separator")
        );
    }

    #[test]
    fn test_url_new_empty_scheme() {
        let result = Url::new("://example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty scheme"));
    }

    #[test]
    fn test_url_new_only_separator() {
        let result = Url::new("://");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty scheme"));
    }

    #[test]
    fn test_url_new_empty_string() {
        let result = Url::new("");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("missing '://' separator")
        );
    }

    #[test]
    fn test_url_empty_host_after_scheme() {
        let result = Url::new("http://");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty host"));
    }

    #[test]
    fn test_url_with_only_port() {
        let result = Url::new("http://:8080");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty host"));
    }

    #[test]
    fn test_url_with_only_port_and_path() {
        let result = Url::new("http://:8080/path");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty host"));
    }

    #[test]
    fn test_url_complex_path() {
        let result = Url::new("http://example.com/path/to/resource?query=1&foo=bar#section");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.host.as_str(), "example.com");
        assert_eq!(url.port, None);
        assert_eq!(
            url.path.as_str(),
            "/path/to/resource?query=1&foo=bar#section"
        );
    }

    #[test]
    fn test_url_with_port_zero() {
        let result = Url::new("http://example.com:0/path");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.port, Some(Port::new(0)));
    }

    #[test]
    fn test_url_max_port() {
        let result = Url::new("http://example.com:65535/path");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.port, Some(Port::new(65535)));
    }

    #[test]
    fn test_url_getters() {
        let url = Url::new("http://example.com:8080/api/v1").unwrap();

        // Test accessing fields through public interface
        assert_eq!(url.scheme, Scheme::Http);
        assert_eq!(url.host.as_str(), "example.com");
        assert_eq!(url.port.map(|p| p.value()), Some(8080));
        assert_eq!(url.path.as_str(), "/api/v1");
    }

    #[test]
    fn test_http_response_200_ok_with_body() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();

        // Create a mock response
        let response = b"HTTP/1.0 200 OK\r\n\
                        Content-Type: text/html\r\n\
                        Content-Length: 13\r\n\
                        \r\n\
                        Hello, World!";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "Hello, World!");
    }

    #[test]
    fn test_http_response_404_not_found() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com/notfound").unwrap();

        // Create a mock 404 response
        let response = b"HTTP/1.0 404 Not Found\r\n\
                        Content-Type: text/html\r\n\
                        \r\n\
                        <h1>404 Not Found</h1>";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("404"));
        assert!(err_msg.contains("Not Found"));
    }

    #[test]
    fn test_http_response_headers_lowercase() {
        use crate::connection::FakeConnection;

        // This test will verify headers are converted to lowercase
        // We'll need to expose headers in the implementation to test this properly
        let url = Url::new("http://example.com").unwrap();

        let response = b"HTTP/1.0 200 OK\r\n\
                        Content-Type: text/html\r\n\
                        Content-LENGTH: 5\r\n\
                        Server: TestServer\r\n\
                        \r\n\
                        Hello";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "Hello");
    }

    #[test]
    fn test_http_response_empty_body() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();

        // Test 200 OK with empty body
        let response = b"HTTP/1.0 200 OK\r\n\
                        Content-Length: 0\r\n\
                        \r\n";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "");
    }

    #[test]
    fn test_http_response_malformed_status_line() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();

        // Test malformed status line
        let response = b"INVALID RESPONSE\r\n\
                        \r\n\
                        Body content";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_err());
    }

    #[test]
    fn test_http_response_500_server_error() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();

        let response = b"HTTP/1.0 500 Internal Server Error\r\n\
                        \r\n\
                        Server error occurred";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("500"));
        assert!(err_msg.contains("Internal Server Error"));
    }

    // Edge case tests for status line parsing - prevent panics

    #[test]
    fn test_http_response_status_line_empty() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();
        let response = b"\r\n\r\n";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_err());
        // Empty line results in a single empty string when split, so it's missing the status code
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Missing status code") || err_msg.contains("Missing HTTP version")
        );
    }

    #[test]
    fn test_http_response_status_line_one_part() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();
        let response = b"HTTP/1.0\r\n\r\n";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing status code")
        );
    }

    #[test]
    fn test_http_response_status_line_two_parts() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();
        let response = b"HTTP/1.0 200\r\n\r\n";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Missing status text")
        );
    }

    #[test]
    fn test_http_response_status_line_with_spaces_in_reason() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();
        // Status text can contain spaces, splitn(3) handles this correctly
        let response = b"HTTP/1.0 404 Not Found Here\r\n\
                        Content-Length: 0\r\n\
                        \r\n";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("404"));
        assert!(err_msg.contains("Not Found Here"));
    }

    #[test]
    fn test_http_response_status_line_non_numeric_code() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();
        let response = b"HTTP/1.0 ABC OK\r\n\r\n";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid status code")
        );
    }

    #[test]
    fn test_https_request_with_fake_connection() {
        use crate::connection::FakeConnection;

        let url = Url::new("https://example.com").unwrap();

        // Create a mock HTTPS response
        let response = b"HTTP/1.0 200 OK\r\n\
                        Content-Type: text/html\r\n\
                        Content-Length: 13\r\n\
                        \r\n\
                        Hello, HTTPS!";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "Hello, HTTPS!");
    }

    #[test]
    fn test_https_default_port() {
        let url = Url::new("https://example.com").unwrap();
        assert_eq!(url.scheme, Scheme::Https);

        // Test that HTTPS uses port 443 by default
        let default_port = url.get_default_port();
        assert_eq!(default_port, 443);
    }

    #[test]
    fn test_http_default_port() {
        let url = Url::new("http://example.com").unwrap();
        assert_eq!(url.scheme, Scheme::Http);

        // Test that HTTP uses port 80 by default
        let default_port = url.get_default_port();
        assert_eq!(default_port, 80);
    }

    // Edge case tests for header parsing

    #[test]
    fn test_http_response_header_without_colon() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();
        // Header without colon is silently ignored (doesn't panic)
        let response = b"HTTP/1.0 200 OK\r\n\
                        InvalidHeader\r\n\
                        Content-Length: 5\r\n\
                        \r\n\
                        Hello";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "Hello");
    }

    #[test]
    fn test_http_response_header_empty_name() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();
        // Header with empty name (": value")
        let response = b"HTTP/1.0 200 OK\r\n\
                        : EmptyName\r\n\
                        Content-Length: 2\r\n\
                        \r\n\
                        OK";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "OK");
    }

    #[test]
    fn test_http_response_header_empty_value() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();
        // Header with empty value
        let response = b"HTTP/1.0 200 OK\r\n\
                        EmptyValue:\r\n\
                        Content-Length: 0\r\n\
                        \r\n";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "");
    }

    #[test]
    fn test_http_response_header_multiple_colons() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();
        // Header with multiple colons - only first one counts
        let response = b"HTTP/1.0 200 OK\r\n\
                        Time: 10:30:45\r\n\
                        URL: http://example.com:8080\r\n\
                        \r\n\
                        Test";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "Test");
    }

    // Edge case tests for response body

    #[test]
    fn test_http_response_200_with_utf8_body() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();
        // UTF-8 content with non-ASCII characters
        let response = "HTTP/1.0 200 OK\r\n\
                       Content-Type: text/plain; charset=utf-8\r\n\
                       \r\n\
                       Hello, 世界! 🌍 Здравствуй мир!"
            .as_bytes();

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), "Hello, 世界! 🌍 Здравствуй мир!");
    }

    #[test]
    fn test_http_response_200_with_large_body() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();

        // Create a large body (10KB of 'A's)
        let large_body = "A".repeat(10_000);
        let response = format!(
            "HTTP/1.0 200 OK\r\n\
                               Content-Length: {}\r\n\
                               \r\n\
                               {}",
            large_body.len(),
            large_body
        );

        let fake = FakeConnection::with_read_data(response.as_bytes().to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_ok());
        let body = result.unwrap();
        assert_eq!(body.as_str().len(), 10_000);
        assert!(body.as_str().chars().all(|c| c == 'A'));
    }

    #[test]
    fn test_http_response_negative_status_code() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();
        let response = b"HTTP/1.0 -1 Error\r\n\r\n";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid status code")
        );
    }

    #[test]
    fn test_http_response_status_code_too_large() {
        use crate::connection::FakeConnection;

        let url = Url::new("http://example.com").unwrap();
        // Status code larger than u16::MAX (65535)
        let response = b"HTTP/1.0 99999 Too Large\r\n\r\n";

        let fake = FakeConnection::with_read_data(response.to_vec());
        let conn = Connection::Fake(fake);

        let result = url.request_with_connection(conn);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid status code")
        );
    }
}

