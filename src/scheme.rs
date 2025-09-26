//! URL scheme definitions for the web browser.

use anyhow::anyhow;
use std::str::FromStr;

/// Supported URL schemes.
///
/// # Examples
/// ```
/// use web_browser_engineering_rs::scheme::Scheme;
/// use std::str::FromStr;
///
/// let scheme = "http".parse::<Scheme>().unwrap();
/// assert_eq!(scheme, Scheme::Http);
///
/// let https_scheme = "https".parse::<Scheme>().unwrap();
/// assert_eq!(https_scheme, Scheme::Https);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    Http,
    Https,
}

impl FromStr for Scheme {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "http" => Ok(Scheme::Http),
            "https" => Ok(Scheme::Https),
            _ => Err(anyhow!("Unsupported scheme: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheme_from_str_http() {
        let result = "http".parse::<Scheme>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Scheme::Http);
    }

    #[test]
    fn test_scheme_from_str_https() {
        let result = "https".parse::<Scheme>();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Scheme::Https);
    }

    #[test]
    fn test_scheme_from_str_ftp_unsupported() {
        let result = "ftp".parse::<Scheme>();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Unsupported scheme: ftp");
    }

    #[test]
    fn test_scheme_from_str_empty() {
        let result = "".parse::<Scheme>();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Unsupported scheme: ");
    }

    #[test]
    fn test_scheme_from_str_random() {
        let result = "random".parse::<Scheme>();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unsupported scheme: random"
        );
    }
}

