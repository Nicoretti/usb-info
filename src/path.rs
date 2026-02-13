//! Device path parsing and manipulation

use std::fmt;
use std::str::FromStr;

use crate::error::DevicePathError;

/// A parsed USB device path representing "bus:port.port.port" format
///
/// # Examples
///
/// ```
/// use usbinfo::DevicePath;
///
/// // Parse from string
/// let path: DevicePath = "1:2.3.4".parse().unwrap();
/// assert_eq!(path.bus(), 1);
/// assert_eq!(path.ports(), &[2, 3, 4]);
///
/// // Bus-only path
/// let bus_path: DevicePath = "2:".parse().unwrap();
/// assert_eq!(bus_path.bus(), 2);
/// assert!(bus_path.ports().is_empty());
///
/// // Create programmatically
/// let path = DevicePath::new(1, vec![2, 3]);
/// assert_eq!(path.to_string(), "1:2.3");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DevicePath {
    /// Bus number
    bus: u8,
    /// Port chain (may be empty for bus root)
    ports: Vec<u8>,
}

impl DevicePath {
    /// Create a new DevicePath from bus number and port chain
    pub fn new(bus: u8, ports: Vec<u8>) -> Self {
        Self { bus, ports }
    }

    /// Create a bus-only path (no ports)
    pub fn bus_only(bus: u8) -> Self {
        Self { bus, ports: vec![] }
    }

    /// Get the bus number
    pub fn bus(&self) -> u8 {
        self.bus
    }

    /// Get the port chain
    pub fn ports(&self) -> &[u8] {
        &self.ports
    }

    /// Check if this is a bus-only path (no ports)
    pub fn is_bus_only(&self) -> bool {
        self.ports.is_empty()
    }

    /// Get the parent path (one level up)
    /// Returns None if this is already a bus-only path
    pub fn parent(&self) -> Option<DevicePath> {
        if self.ports.is_empty() {
            None
        } else {
            Some(DevicePath {
                bus: self.bus,
                ports: self.ports[..self.ports.len() - 1].to_vec(),
            })
        }
    }

    /// Get the depth (number of ports in the chain)
    pub fn depth(&self) -> usize {
        self.ports.len()
    }

    /// Check if this path is an ancestor of another path
    pub fn is_ancestor_of(&self, other: &DevicePath) -> bool {
        self.bus == other.bus
            && self.ports.len() < other.ports.len()
            && other.ports.starts_with(&self.ports)
    }

    /// Check if this path is a descendant of another path
    pub fn is_descendant_of(&self, other: &DevicePath) -> bool {
        other.is_ancestor_of(self)
    }

    /// Create a child path by appending a port
    pub fn child(&self, port: u8) -> DevicePath {
        let mut ports = self.ports.clone();
        ports.push(port);
        DevicePath { bus: self.bus, ports }
    }

    /// Get the bus as a string (for HashMap keys)
    pub fn bus_str(&self) -> String {
        self.bus.to_string()
    }

    /// Convert to the canonical string key format
    pub fn to_key(&self) -> String {
        self.to_string()
    }
}

impl FromStr for DevicePath {
    type Err = DevicePathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (bus_str, port_str) = s.split_once(':').ok_or(DevicePathError::InvalidFormat)?;

        if bus_str.is_empty() {
            return Err(DevicePathError::MissingBus);
        }

        let bus = bus_str
            .parse::<u8>()
            .map_err(|_| DevicePathError::InvalidBus(bus_str.to_string()))?;

        let ports = if port_str.is_empty() {
            vec![]
        } else {
            port_str
                .split('.')
                .map(|p| {
                    p.parse::<u8>()
                        .map_err(|_| DevicePathError::InvalidPort(p.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        Ok(DevicePath { bus, ports })
    }
}

impl fmt::Display for DevicePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ports.is_empty() {
            write!(f, "{}:", self.bus)
        } else {
            write!(
                f,
                "{}:{}",
                self.bus,
                self.ports
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(".")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_path() {
        let path: DevicePath = "1:2.3.4".parse().unwrap();
        assert_eq!(path.bus(), 1);
        assert_eq!(path.ports(), &[2, 3, 4]);
    }

    #[test]
    fn test_parse_bus_only() {
        let path: DevicePath = "2:".parse().unwrap();
        assert_eq!(path.bus(), 2);
        assert!(path.ports().is_empty());
        assert!(path.is_bus_only());
    }

    #[test]
    fn test_display() {
        let path = DevicePath::new(1, vec![2, 3]);
        assert_eq!(path.to_string(), "1:2.3");
    }

    #[test]
    fn test_parent() {
        let path = DevicePath::new(1, vec![2, 3, 4]);
        let parent = path.parent().unwrap();
        assert_eq!(parent, DevicePath::new(1, vec![2, 3]));
    }

    #[test]
    fn test_child() {
        let path = DevicePath::new(1, vec![2]);
        let child = path.child(3);
        assert_eq!(child, DevicePath::new(1, vec![2, 3]));
    }

    #[test]
    fn test_ancestor() {
        let parent = DevicePath::new(1, vec![2]);
        let child = DevicePath::new(1, vec![2, 3, 4]);
        assert!(parent.is_ancestor_of(&child));
        assert!(child.is_descendant_of(&parent));
    }
}
