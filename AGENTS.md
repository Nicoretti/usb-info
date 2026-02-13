# Agents

## Code Style

Rust 2024 edition preferred.

### Error Handling

- Use `thiserror` for custom error types in the library
- Use `anyhow` for error propagation in binaries
- Prefer `?` operator over explicit `match` on Results
- Return `Result` when errors are expected

```rust
// Good
let tree = build_usb_tree()?;

// Avoid
let tree = match build_usb_tree() {
    Ok(t) => t,
    Err(e) => return Err(e),
};
```

### Control Flow

- Prefer `match` over `if-else` when it reduces noise and improves readability
- Use `match` for enums, Options, and multiple branches
- `if-else` is fine for simple boolean conditions

```rust
// Good - match for Options
match device.manufacturer {
    Some(m) => println!("Made by {}", m),
    None => println!("Unknown manufacturer"),
}

// Good - match for multiple conditions
let color = match depth % 3 {
    0 => "red",
    1 => "green",
    _ => "blue",
};

// Fine - simple boolean
if devices.is_empty() {
    return Ok(());
}
```

### String Handling

- Prefer `&str` parameters over `String` where possible
- Use `impl AsRef<str>` for maximum flexibility in public APIs
- Avoid unnecessary allocations

```rust
// Good
pub fn get(&self, path: &str) -> Option<&T>

// Better for flexible APIs
pub fn get(&self, path: impl AsRef<str>) -> Option<&T>

// Avoid
pub fn get(&self, path: String) -> Option<&T>
```

### Struct Patterns

- Use builder pattern for structs with many optional fields
- Implement `Default` for configuration structs
- Use `#[derive]` liberally: `Debug, Clone, PartialEq, Eq, Hash` as appropriate

```rust
// Builder pattern
let style = TreeStyle::new()
    .with_color(false)
    .with_header(true);

// Default + override
let style = TreeStyle {
    colored: false,
    ..Default::default()
};
```

### Documentation

- Doc comments (`///`) on all public items
- Module-level documentation (`//!`) at the top of each file
- Include examples in doc comments for key functions

```rust
/// Parse a device path from a string.
///
/// # Examples
///
/// ```
/// use usbinfo::DevicePath;
///
/// let path: DevicePath = "1:2.3".parse().unwrap();
/// assert_eq!(path.bus(), 1);
/// ```
pub fn parse(s: &str) -> Result<DevicePath, DevicePathError>
```

### Imports

- Group imports: std, external crates, crate modules
- Prefer explicit imports over glob imports (`*`)
- Use `crate::` for internal imports

```rust
use std::collections::HashMap;
use std::fmt;

use colored::Colorize;
use thiserror::Error;

use crate::device::UsbDevice;
use crate::error::UsbTreeError;
```

### Iterators and Collections

- Prefer iterators over manual loops
- Use `collect()` with type inference where clear
- Chain iterator methods for transformations

```rust
// Good
let names: Vec<_> = devices
    .iter()
    .filter(|d| d.is_hub())
    .map(|d| &d.name)
    .collect();

// Avoid
let mut names = Vec::new();
for d in devices.iter() {
    if d.is_hub() {
        names.push(&d.name);
    }
}
```

### Naming Conventions

- Types: `PascalCase`
- Functions/methods: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`
- Prefer descriptive names over abbreviations (except well-known: `vid`, `pid`, `usb`)

### Project Organization

- Separate library code (`src/`) from binaries (`src/bin/`)
- One primary type per module when appropriate
- Re-export public API from `lib.rs`

## Agent Roles

For specialized tasks, checkout the `.roles/` folder:
- `pairing.md` - Collaborative coding and feedback
