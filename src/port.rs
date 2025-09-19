//! Port type for URL parsing.

use std::fmt;

/// A port number component of a URL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port(u16);

impl Port {
    /// Creates a new Port from a u16.
    pub fn new(port: u16) -> Self {
        Port(port)
    }

    /// Returns the port number as u16.
    pub fn value(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u16> for Port {
    fn from(port: u16) -> Self {
        Port(port)
    }
}

impl From<Port> for u16 {
    fn from(port: Port) -> u16 {
        port.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_creation() {
        let port = Port::new(8080);
        assert_eq!(port.value(), 8080);
    }

    #[test]
    fn test_port_display() {
        let port = Port::new(3000);
        assert_eq!(format!("{}", port), "3000");
    }

    #[test]
    fn test_port_from_u16() {
        let port: Port = 8080.into();
        assert_eq!(port.value(), 8080);
    }

    #[test]
    fn test_u16_from_port() {
        let port = Port::new(8080);
        let value: u16 = port.into();
        assert_eq!(value, 8080);
    }
}