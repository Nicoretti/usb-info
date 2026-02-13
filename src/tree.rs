//! Tree data structures for USB device hierarchy

use std::collections::HashMap;

use nusb::MaybeFuture;

use crate::device::UsbDevice;
use crate::error::UsbTreeError;
use crate::path::DevicePath;

/// A tree node for organizing port hierarchy
#[derive(Debug, Clone)]
pub struct PortTree<T> {
    /// Value stored at this node (if any)
    pub value: Option<T>,
    /// Children indexed by port number
    pub children: HashMap<u8, PortTree<T>>,
}

impl<T> Default for PortTree<T> {
    fn default() -> Self {
        Self {
            value: None,
            children: HashMap::new(),
        }
    }
}

impl<T> PortTree<T> {
    /// Create a new empty PortTree
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a value at the given port path
    pub fn insert(&mut self, ports: &[u8], value: T) {
        if ports.is_empty() {
            self.value = Some(value);
        } else {
            self.children
                .entry(ports[0])
                .or_insert_with(PortTree::new)
                .insert(&ports[1..], value);
        }
    }

    /// Get the subtree at the given port path
    pub fn get(&self, ports: &[u8]) -> Option<&PortTree<T>> {
        if ports.is_empty() {
            Some(self)
        } else {
            self.children.get(&ports[0])?.get(&ports[1..])
        }
    }

    /// Get all descendant values (including self)
    pub fn descendants(&self) -> Vec<&T> {
        let mut result = Vec::new();
        if let Some(ref v) = self.value {
            result.push(v);
        }
        for child in self.children.values() {
            result.extend(child.descendants());
        }
        result
    }

    /// Get direct children values
    pub fn direct_children(&self) -> Vec<(u8, &T)> {
        self.children
            .iter()
            .filter_map(|(&port, child)| child.value.as_ref().map(|v| (port, v)))
            .collect()
    }

    /// Get sorted child port numbers
    pub fn child_ports(&self) -> Vec<u8> {
        let mut ports: Vec<u8> = self.children.keys().copied().collect();
        ports.sort();
        ports
    }
}

/// Parse a port path string like "1.2.3" into a Vec<u8>
fn parse_port_path(path: &str) -> Vec<u8> {
    if path.is_empty() {
        return vec![];
    }
    path.split('.')
        .filter_map(|s| s.parse::<u8>().ok())
        .collect()
}

/// USB device tree with flat lookup and hierarchical structure
#[derive(Debug)]
pub struct UsbTree<T> {
    /// Flat map of path -> device
    pub devices: HashMap<String, T>,
    /// Hierarchical tree per bus: bus -> tree of keys
    tree: HashMap<String, PortTree<String>>,
}

impl<T> Default for UsbTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> UsbTree<T> {
    /// Create a new empty UsbTree
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            tree: HashMap::new(),
        }
    }

    /// Insert a device with a DevicePath
    pub fn insert_path(&mut self, path: &DevicePath, value: T) {
        let key = path.to_string();
        self.devices.insert(key.clone(), value);
        self.tree
            .entry(path.bus_str())
            .or_default()
            .insert(path.ports(), key);
    }

    /// Insert with bus_id and port_chain (convenience method)
    pub fn insert(&mut self, bus: &str, ports: &[u8], value: T) {
        let bus_num = bus.parse::<u8>().unwrap_or(0);
        let path = DevicePath::new(bus_num, ports.to_vec());
        self.insert_path(&path, value);
    }

    /// Get device by DevicePath
    pub fn get_by_path(&self, path: &DevicePath) -> Option<&T> {
        self.devices.get(&path.to_string())
    }

    /// Get device by path string "bus:port.path" e.g., "1:1.2.3"
    pub fn get(&self, path: &str) -> Option<&T> {
        // Try direct lookup first
        if let Some(device) = self.devices.get(path) {
            return Some(device);
        }
        // Try parsing and looking up
        if let Ok(parsed) = path.parse::<DevicePath>() {
            return self.devices.get(&parsed.to_string());
        }
        None
    }

    /// Get device by path, returning an error if not found
    pub fn try_get(&self, path: &str) -> Result<&T, UsbTreeError> {
        self.get(path)
            .ok_or_else(|| UsbTreeError::DeviceNotFound(path.to_string()))
    }

    /// Get device by DevicePath, returning an error if not found
    pub fn try_get_by_path(&self, path: &DevicePath) -> Result<&T, UsbTreeError> {
        self.get_by_path(path)
            .ok_or_else(|| UsbTreeError::DeviceNotFound(path.to_string()))
    }

    /// Get mutable device by path string
    pub fn get_mut(&mut self, path: &str) -> Option<&mut T> {
        self.devices.get_mut(path)
    }

    /// Get mutable device by DevicePath
    pub fn get_mut_by_path(&mut self, path: &DevicePath) -> Option<&mut T> {
        self.devices.get_mut(&path.to_string())
    }

    /// Get all devices under a subtree by DevicePath
    pub fn get_subtree_by_path(&self, path: &DevicePath) -> Vec<&T> {
        self.tree
            .get(&path.bus_str())
            .and_then(|t| t.get(path.ports()))
            .map(|node| {
                node.descendants()
                    .into_iter()
                    .filter_map(|key| self.devices.get(key))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all devices under a subtree, e.g., "1:1.2"
    pub fn get_subtree(&self, path: &str) -> Vec<&T> {
        if let Ok(parsed) = path.parse::<DevicePath>() {
            self.get_subtree_by_path(&parsed)
        } else {
            // Fallback to old parsing for backwards compatibility
            let (bus, port_path) = path.split_once(':').unwrap_or((path, ""));
            let ports = parse_port_path(port_path);

            self.tree
                .get(bus)
                .and_then(|t| t.get(&ports))
                .map(|node| {
                    node.descendants()
                        .into_iter()
                        .filter_map(|key| self.devices.get(key))
                        .collect()
                })
                .unwrap_or_default()
        }
    }

    /// Get all bus IDs
    pub fn buses(&self) -> Vec<&str> {
        let mut buses: Vec<&str> = self.tree.keys().map(|s| s.as_str()).collect();
        buses.sort();
        buses
    }

    /// Get the PortTree for a specific bus
    pub fn bus_tree(&self, bus: &str) -> Option<&PortTree<String>> {
        self.tree.get(bus)
    }

    /// Get all devices (flat iterator)
    pub fn all_devices(&self) -> impl Iterator<Item = (&String, &T)> {
        self.devices.iter()
    }

    /// Number of devices
    pub fn len(&self) -> usize {
        self.devices.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }
}

impl<T> std::ops::Index<&str> for UsbTree<T> {
    type Output = T;

    fn index(&self, path: &str) -> &Self::Output {
        self.get(path).expect("device not found")
    }
}

impl<T> std::ops::Index<&DevicePath> for UsbTree<T> {
    type Output = T;

    fn index(&self, path: &DevicePath) -> &Self::Output {
        self.get_by_path(path).expect("device not found")
    }
}

/// Build a UsbTree from actual system devices using nusb
pub fn usb_tree() -> Result<UsbTree<UsbDevice>, UsbTreeError> {
    let devices: Vec<nusb::DeviceInfo> = nusb::list_devices()
        .wait()
        .map_err(|e| UsbTreeError::ListDevices(e.to_string()))?
        .collect();

    let mut tree = UsbTree::new();

    for info in &devices {
        let device = UsbDevice::from_device_info(info);
        let path = device.path();
        tree.insert_path(&path, device);
    }

    Ok(tree)
}
