//! USB device tree library
//!
//! This library provides functionality for exploring and displaying
//! USB device trees with Rich-style formatting and colors.
//!
//! # Examples
//!
//! ```no_run
//! use usbinfo::{build_usb_tree, TreeFormatter, TreeStyle};
//!
//! // Build the USB device tree
//! let tree = build_usb_tree().unwrap();
//!
//! // Display with default colored output
//! let formatter = TreeFormatter::new(&tree);
//! println!("{}", formatter);
//!
//! // Access devices by path
//! if let Some(device) = tree.get("1:2.3") {
//!     println!("Found device: {}", device);
//! }
//!
//! // Get all devices under a hub
//! let children = tree.get_subtree("1:2");
//! for dev in children {
//!     println!("  - {}", dev);
//! }
//! ```

mod device;
mod error;
mod formatter;
mod path;
mod tree;

// Re-export public API
pub use device::{matches_vid_pid, UsbDevice};
pub use error::{DevicePathError, UsbTreeError};
pub use formatter::{TreeFormatter, TreeStyle};
pub use path::DevicePath;
pub use tree::{build_usb_tree, PortTree, UsbTree};
