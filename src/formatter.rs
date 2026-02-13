//! Formatting and display for USB device trees

use std::fmt;
use colored::{ColoredString, Colorize};

use crate::device::UsbDevice;
use crate::tree::{PortTree, UsbTree};

/// Configuration for tree output formatting
#[derive(Debug, Clone)]
pub struct TreeStyle {
    /// Whether to use colored output
    pub colored: bool,
    /// Whether to show the header
    pub show_header: bool,
    /// Indent string for each level
    pub indent: String,
    /// Connector for non-last items
    pub branch: &'static str,
    /// Connector for last items
    pub corner: &'static str,
    /// Vertical line for continuing branches
    pub vertical: &'static str,
}

impl Default for TreeStyle {
    fn default() -> Self {
        Self {
            colored: true,
            show_header: true,
            indent: "    ".to_string(),
            branch: "├── ",
            corner: "└── ",
            vertical: "│   ",
        }
    }
}

impl TreeStyle {
    /// Create a new TreeStyle with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a plain (non-colored) style
    pub fn plain() -> Self {
        Self {
            colored: false,
            ..Self::default()
        }
    }

    /// Create an ASCII-only style (no Unicode box drawing)
    pub fn ascii() -> Self {
        Self {
            branch: "|-- ",
            corner: "`-- ",
            vertical: "|   ",
            ..Self::default()
        }
    }

    /// Set whether to use colors
    pub fn with_color(mut self, colored: bool) -> Self {
        self.colored = colored;
        self
    }

    /// Set whether to show the header
    pub fn with_header(mut self, show_header: bool) -> Self {
        self.show_header = show_header;
        self
    }
}

/// Formatter for rendering USB device trees
///
/// # Examples
///
/// ```no_run
/// use usbinfo::{build_usb_tree, TreeFormatter, TreeStyle};
///
/// let tree = build_usb_tree().unwrap();
///
/// // Default colored output
/// let formatter = TreeFormatter::new(&tree);
/// println!("{}", formatter);
///
/// // Plain (no colors) output
/// let formatter = TreeFormatter::with_style(&tree, TreeStyle::plain());
/// println!("{}", formatter);
///
/// // ASCII style
/// let formatter = TreeFormatter::with_style(&tree, TreeStyle::ascii());
/// println!("{}", formatter);
/// ```
pub struct TreeFormatter<'a> {
    tree: &'a UsbTree<UsbDevice>,
    style: TreeStyle,
}

impl<'a> TreeFormatter<'a> {
    /// Create a new formatter with default style (colored)
    pub fn new(tree: &'a UsbTree<UsbDevice>) -> Self {
        Self {
            tree,
            style: TreeStyle::default(),
        }
    }

    /// Create a formatter with a custom style
    pub fn with_style(tree: &'a UsbTree<UsbDevice>, style: TreeStyle) -> Self {
        Self { tree, style }
    }

    /// Create a plain (non-colored) formatter
    pub fn plain(tree: &'a UsbTree<UsbDevice>) -> Self {
        Self {
            tree,
            style: TreeStyle::plain(),
        }
    }

    /// Colorize text based on depth level (if colors enabled)
    fn colorize(&self, text: &str, depth: usize) -> String {
        if !self.style.colored {
            return text.to_string();
        }

        let colored: ColoredString = match depth % 10 {
            0 => text.red(),
            1 => text.yellow(),
            2 => text.green(),
            3 => text.cyan(),
            4 => text.blue(),
            5 => text.magenta(),
            6 => text.bright_red(),
            7 => text.bright_yellow(),
            8 => text.bright_green(),
            9 => text.bright_cyan(),
            _ => text.white(),
        };
        colored.to_string()
    }

    /// Format a port tree node recursively
    fn fmt_port_tree(
        &self,
        port_tree: &PortTree<String>,
        prefix: &str,
        is_last: bool,
        depth: usize,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        // Print current node if it has a value
        if let Some(ref key) = port_tree.value {
            if let Some(device) = self.tree.devices.get(key) {
                let connector = if depth == 0 {
                    ""
                } else if is_last {
                    self.style.corner
                } else {
                    self.style.branch
                };

                let device_str = self.colorize(&device.to_string(), depth);
                writeln!(f, "{}{}{}", prefix, connector, device_str)?;
            }
        }

        // Print children
        let child_ports = port_tree.child_ports();
        let count = child_ports.len();

        for (i, port) in child_ports.into_iter().enumerate() {
            if let Some(child) = port_tree.children.get(&port) {
                let new_prefix = if depth == 0 {
                    String::new()
                } else if is_last {
                    format!("{}{}", prefix, self.style.indent)
                } else {
                    format!("{}{}", prefix, self.style.vertical)
                };

                self.fmt_port_tree(child, &new_prefix, i == count - 1, depth + 1, f)?;
            }
        }

        Ok(())
    }
}

impl<'a> fmt::Display for TreeFormatter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        for bus_str in self.tree.buses() {
            let bus: u8 = bus_str.parse().unwrap_or(0);

            // Bus level is depth 0
            let bus_label = format!("Bus {:03}", bus);
            writeln!(f, "{}", self.colorize(&bus_label, 0))?;

            if let Some(port_tree) = self.tree.bus_tree(bus_str) {
                let child_ports = port_tree.child_ports();
                let count = child_ports.len();

                for (i, port) in child_ports.into_iter().enumerate() {
                    if let Some(child) = port_tree.children.get(&port) {
                        self.fmt_port_tree(child, "", i == count - 1, 1, f)?;
                    }
                }
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
