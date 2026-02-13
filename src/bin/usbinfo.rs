//! USB device tree explorer CLI application

use anyhow::Result;
use usbinfo::{usb_tree, TreeFormatter};

fn main() -> Result<()> {
    let tree = usb_tree()?;
    let formatter = TreeFormatter::new(&tree);
    print!("{}", formatter);
    Ok(())
}
