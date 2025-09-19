//! Host type for URL parsing.

use std::fmt;
use std::ops::Deref;

/// A hostname component of a URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Host(String);

impl Host {
    /// Creates a new Host from a string.
    pub fn new(s: impl Into<String>) -> Self {
        Host(s.into())
    }

    /// Returns the host as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for Host {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Host {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Host {
    fn from(s: String) -> Self {
        Host(s)
    }
}

impl From<&str> for Host {
    fn from(s: &str) -> Self {
        Host(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_creation() {
        let host = Host::new("example.com");
        assert_eq!(host.as_str(), "example.com");
    }

    #[test]
    fn test_host_deref() {
        let host = Host::new("example.com");
        assert_eq!(&*host, "example.com");
    }

    #[test]
    fn test_host_display() {
        let host = Host::new("example.com");
        assert_eq!(format!("{}", host), "example.com");
    }

    #[test]
    fn test_host_as_ref() {
        let host = Host::new("example.com");
        let s: &str = host.as_ref();
        assert_eq!(s, "example.com");
    }

    #[test]
    fn test_host_from_string() {
        let host = Host::from(String::from("example.com"));
        assert_eq!(host.as_str(), "example.com");
    }

    #[test]
    fn test_host_from_str() {
        let host = Host::from("example.com");
        assert_eq!(host.as_str(), "example.com");
    }

    #[test]
    fn test_host_new_with_string() {
        let host = Host::new(String::from("example.com"));
        assert_eq!(host.as_str(), "example.com");
    }
}