//! Error types for the usbinfo library

use thiserror::Error;

/// Error type for DevicePath parsing
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DevicePathError {
    /// Missing bus number
    #[error("missing bus number")]
    MissingBus,
    /// Invalid bus number
    #[error("invalid bus number: '{0}'")]
    InvalidBus(String),
    /// Invalid port number in path
    #[error("invalid port number: '{0}'")]
    InvalidPort(String),
    /// Invalid format (missing colon separator)
    #[error("invalid format, expected 'bus:port.path'")]
    InvalidFormat,
}

/// Error type for USB tree operations
#[derive(Debug, Error)]
pub enum UsbTreeError {
    /// Failed to list USB devices
    #[error("failed to list USB devices: {0}")]
    ListDevices(String),
    /// Device not found at path
    #[error("device not found at path: '{0}'")]
    DeviceNotFound(String),
    /// Invalid device path
    #[error("invalid device path: {0}")]
    InvalidPath(#[from] DevicePathError),
}
