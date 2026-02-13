//! USB device tree explorer CLI application

use anyhow::Result;
use usbinfo::{build_usb_tree, TreeFormatter};

fn main() -> Result<()> {
    let tree = build_usb_tree()?;
    let formatter = TreeFormatter::new(&tree);
    print!("{}", formatter);
    Ok(())
}
