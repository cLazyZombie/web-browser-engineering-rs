//! URL parsing and representation.

use anyhow::{anyhow, Result};
use crate::scheme::Scheme;
use crate::host::Host;
use crate::port::Port;
use crate::path::Path;
use std::net::TcpStream;

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
    /// assert!(Url::new("http://example.com:8080/path").is_ok());
    ///
    /// assert!(Url::new("https://example.com").is_err()); // Not supported yet
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

            let port = port_str.parse::<u16>()
                .map_err(|_| anyhow!("Invalid port number: {}", port_str))?;

            (Host::new(host), Some(Port::new(port)))
        } else {
            (Host::new(host_port), None)
        };

        if host.as_str().is_empty() {
            return Err(anyhow!("Invalid URL: empty host"));
        }

        Ok(Url { scheme, host, port, path: Path::new(path) })
    }

    /// Opens a TCP socket connection to the host.
    /// Uses the parsed port if available, otherwise defaults to port 80.
    pub fn request(&self) -> Result<TcpStream> {
        let port = self.port.map(|p| p.value()).unwrap_or(80);
        let addr = format!("{}:{}", self.host.as_str(), port);
        TcpStream::connect(addr)
            .map_err(|e| anyhow!("Connection failed: {}", e))
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
    fn test_url_invalid_port() {
        let result = Url::new("http://example.com:not_a_number/path");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid port number"));
    }

    #[test]
    fn test_url_port_out_of_range() {
        let result = Url::new("http://example.com:99999/path");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid port number"));
    }


    #[test]
    fn test_url_new_invalid_https() {
        let result = Url::new("https://example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported scheme: https"));
    }

    #[test]
    fn test_url_new_invalid_ftp() {
        let result = Url::new("ftp://example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported scheme: ftp"));
    }

    #[test]
    fn test_url_new_missing_separator() {
        let result = Url::new("http:example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing '://' separator"));
    }

    #[test]
    fn test_url_new_no_scheme() {
        let result = Url::new("example.com");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing '://' separator"));
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
        assert!(result.unwrap_err().to_string().contains("missing '://' separator"));
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
        assert_eq!(url.path.as_str(), "/path/to/resource?query=1&foo=bar#section");
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
}