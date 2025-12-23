# liquid-glass-rs

[![Crates.io](https://img.shields.io/crates/v/liquid-glass-rs.svg)](https://crates.io/crates/liquid-glass-rs)
[![Documentation](https://docs.rs/liquid-glass-rs/badge.svg)](https://docs.rs/liquid-glass-rs)
[![License](https://img.shields.io/crates/l/liquid-glass-rs.svg)](https://github.com/Stapxs/liquid-glass-rs/blob/main/LICENSE)

<img width="1045" height="165" alt="image" src="https://github.com/user-attachments/assets/9dd3422a-a4bc-4f3a-adaf-620cd632d227" />


Native macOS glass effects implementation in Rust. This crate provides a safe Rust interface to create and manage macOS glass effects (NSGlassEffectView) for Tauri windows or any other native macOS windows.

> [!IMPORTANT]
> This project comes from the logic and inspiration of [electron-liquid-glass](https://github.com/Meridius-Labs/electron-liquid-glass). I ported it into the Rust language for use in the Tauri application.
> Thank you very much for the work done by Meridius-Labs to explore the Liquid Glass effect API of macOS.

## Platform Support
This project will not downgrade unsupported platforms and versions by itself. Please make your own platform judgement.

- **macOS 26+**: Full NSGlassEffectView support with all material variants

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
liquid-glass-rs = "0.1"
```

Or install via cargo:

```bash
cargo add liquid-glass-rs
```

## Usage

### Basic Example

```rust
use liquid_glass_rs::{GlassViewManager, GlassOptions};

// Create a glass view manager
let manager = GlassViewManager::new();

// Check if glass effects are supported
if !manager.is_supported() {
    eprintln!("Glass effects are not supported on this platform");
    return;
}

// Configure glass options
let options = GlassOptions {
    corner_radius: 25.0,
    tint_color: Some("#ffffff".to_string()),
    opaque: false,
};

// Add glass effect to a window
// window_ptr should be a pointer to NSView (from Tauri or native macOS)
let view_id = manager.add_glass_view(
    window_ptr as *mut std::ffi::c_void,
    options
)?;

// The glass effect is now applied!
```

### Advanced Configuration

```rust
use liquid_glass_rs::{GlassViewManager, GlassOptions, GlassMaterialVariant};

let manager = GlassViewManager::new();

// Create with default options
let options = GlassOptions::default();
let view_id = manager.add_glass_view(window_ptr, options)?;

// Set a specific material variant
manager.set_variant(view_id, GlassMaterialVariant::Dock)?;

// Adjust scrim state (0 = none, 1 = light, 2 = dark)
manager.set_scrim_state(view_id, 1)?;

// Adjust subdued state
manager.set_subdued_state(view_id, 0)?;

// Remove the glass view when done
manager.remove_view(view_id)?;
```

### Available Material Variants

The crate provides access to 24 different glass material styles:

```rust
pub enum GlassMaterialVariant {
    Regular,           // Default glass effect
    Clear,             // Clear glass
    Dock,              // Dock-style glass
    AppIcons,          // App icons glass
    Widgets,           // Widgets glass
    Text,              // Text glass
    AVPlayer,          // AVPlayer glass
    FaceTime,          // FaceTime glass
    ControlCenter,     // Control Center glass
    NotificationCenter,// Notification Center glass
    Monogram,          // Monogram glass
    Bubbles,           // Bubbles glass
    Identity,          // Identity glass
    FocusBorder,       // Focus border glass
    FocusPlatter,      // Focus platter glass
    Keyboard,          // Keyboard glass
    Sidebar,           // Sidebar glass
    AbuttedSidebar,    // Abutted sidebar glass
    Inspector,         // Inspector glass
    Control,           // Control glass
    Loupe,             // Loupe glass
    Slider,            // Slider glass
    Camera,            // Camera glass
    CartouchePopover,  // Cartouche popover glass
}
```

### GlassOptions

```rust
pub struct GlassOptions {
    /// Corner radius in points (default: 0.0)
    pub corner_radius: f64,
    
    /// Tint color in hex format (#RRGGBB or #RRGGBBAA)
    pub tint_color: Option<String>,
    
    /// Whether to add an opaque background layer
    pub opaque: bool,
}
```

## Integration with Tauri

This crate is particularly useful for Tauri applications running on macOS. Here's a typical integration pattern:

1. Get the window's native handle using Tauri's raw window handle API
2. Pass the handle to this crate to apply glass effects
3. Example Tauri command:

```rust
use tauri::Runtime;
use liquid_glass_rs::{GlassViewManager, GlassOptions};

#[tauri::command]
fn apply_glass_effect<R: Runtime>(window: tauri::Window<R>) -> Result<i32, String> {
    use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
    
    let handle = window.raw_window_handle();
    if let RawWindowHandle::AppKit(appkit_handle) = handle {
        let ns_view = appkit_handle.ns_view;
        
        let manager = GlassViewManager::new();
        let options = GlassOptions {
            corner_radius: 16.0,
            tint_color: Some("#ffffff80".to_string()),
            opaque: false,
        };
        
        manager.add_glass_view(ns_view, options)
            .map_err(|e| e.to_string())
    } else {
        Err("Not a macOS window".to_string())
    }
}
```

## Error Handling

The crate provides comprehensive error handling through the `GlassError` enum:

```rust
pub enum GlassError {
    UnsupportedPlatform,
    InvalidHandle,
    CreationFailed,
    ViewNotFound(i32),
    InvalidColor(String),
}
```

All methods return `Result<T, GlassError>` for proper error propagation.

## Safety

This crate uses `unsafe` code to interface with macOS Objective-C APIs through the `objc` and `cocoa` crates. All unsafe operations are carefully encapsulated within safe public APIs with proper validation and error handling.

## API Documentation

For detailed API documentation, visit [docs.rs/liquid-glass-rs](https://docs.rs/liquid-glass-rs).

## Examples

Check out the [repository](https://github.com/Stapxs/liquid-glass-rs) for complete working examples.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
