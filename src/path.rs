//! Path type for URL parsing.

use std::fmt;
use std::ops::Deref;

/// A path component of a URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path(String);

impl Path {
    /// Creates a new Path from a string.
    pub fn new(s: impl Into<String>) -> Self {
        Path(s.into())
    }

    /// Returns the path as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for Path {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Path {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Path {
    fn from(s: String) -> Self {
        Path(s)
    }
}

impl From<&str> for Path {
    fn from(s: &str) -> Self {
        Path(s.to_string())
    }
}

impl Default for Path {
    fn default() -> Self {
        Path("/".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_creation() {
        let path = Path::new("/index.html");
        assert_eq!(path.as_str(), "/index.html");
    }

    #[test]
    fn test_path_deref() {
        let path = Path::new("/api/users");
        assert_eq!(&*path, "/api/users");
    }

    #[test]
    fn test_path_display() {
        let path = Path::new("/about");
        assert_eq!(format!("{}", path), "/about");
    }

    #[test]
    fn test_path_default() {
        let path = Path::default();
        assert_eq!(path.as_str(), "/");
    }

    #[test]
    fn test_path_as_ref() {
        let path = Path::new("/api/v1");
        let s: &str = path.as_ref();
        assert_eq!(s, "/api/v1");
    }

    #[test]
    fn test_path_from_string() {
        let path = Path::from(String::from("/users"));
        assert_eq!(path.as_str(), "/users");
    }

    #[test]
    fn test_path_from_str() {
        let path = Path::from("/products");
        assert_eq!(path.as_str(), "/products");
    }

    #[test]
    fn test_path_new_with_string() {
        let path = Path::new(String::from("/admin"));
        assert_eq!(path.as_str(), "/admin");
    }

    #[test]
    fn test_path_empty() {
        let path = Path::new("");
        assert_eq!(path.as_str(), "");
    }
}