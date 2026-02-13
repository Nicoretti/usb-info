//! USB device representation

use std::fmt;
use nusb::DeviceInfo;

use crate::path::DevicePath;

/// Represents a USB device
#[derive(Debug, Clone)]
pub struct UsbDevice {
    /// Vendor ID
    pub vid: u16,
    /// Product ID
    pub pid: u16,
    /// Bus number
    pub bus: u8,
    /// Device address on the bus
    pub address: u8,
    /// Device name/description
    pub name: String,
    /// Manufacturer string
    pub manufacturer: Option<String>,
    /// Product string
    pub product: Option<String>,
    /// Serial number
    pub serial: Option<String>,
    /// Device class
    pub class: u8,
    /// Device subclass
    pub subclass: u8,
    /// Device protocol
    pub protocol: u8,
    /// USB speed
    pub speed: Option<nusb::Speed>,
    /// Port path (for building hierarchy)
    pub port_path: Vec<u8>,
}

impl UsbDevice {
    /// Create a UsbDevice from nusb DeviceInfo
    pub fn from_device_info(info: &DeviceInfo) -> Self {
        Self {
            vid: info.vendor_id(),
            pid: info.product_id(),
            bus: info.busnum(),
            address: info.device_address(),
            name: info.product_string().unwrap_or_default().to_string(),
            manufacturer: info.manufacturer_string().map(|s| s.to_string()),
            product: info.product_string().map(|s| s.to_string()),
            serial: info.serial_number().map(|s| s.to_string()),
            class: info.class(),
            subclass: info.subclass(),
            protocol: info.protocol(),
            speed: info.speed(),
            port_path: info.port_chain().to_vec(),
        }
    }

    /// Returns the VID:PID string (e.g., "1234:5678")
    pub fn vid_pid(&self) -> String {
        format!("{:04x}:{:04x}", self.vid, self.pid)
    }

    /// Check if this device is a hub
    pub fn is_hub(&self) -> bool {
        self.class == 9
    }

    /// Get the DevicePath for this device
    pub fn path(&self) -> DevicePath {
        DevicePath::new(self.bus, self.port_path.clone())
    }

    /// Get the path as a string key
    pub fn path_key(&self) -> String {
        self.path().to_string()
    }
}

impl fmt::Display for UsbDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = if self.name.is_empty() {
            "Unknown Device"
        } else {
            &self.name
        };
        write!(
            f,
            "Device {:03}: ID {} {}",
            self.address,
            self.vid_pid(),
            name
        )
    }
}

impl From<&UsbDevice> for DevicePath {
    fn from(device: &UsbDevice) -> Self {
        DevicePath::new(device.bus, device.port_path.clone())
    }
}

impl From<UsbDevice> for DevicePath {
    fn from(device: UsbDevice) -> Self {
        DevicePath::new(device.bus, device.port_path)
    }
}

/// Filter predicate for VID:PID pairs
pub fn matches_vid_pid(device: &UsbDevice, filters: &[(u16, u16)]) -> bool {
    if filters.is_empty() {
        return true;
    }
    filters
        .iter()
        .any(|(vid, pid)| device.vid == *vid && device.pid == *pid)
}
