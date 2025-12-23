//! # liquid-glass-rs
//!
//! Native macOS glass effects implementation in Rust
//!
//! This crate provides a Rust interface to create and manage macOS glass effects
//! (NSGlassEffectView) for Electron windows or any other native macOS windows.
//!
//! ## Platform Support
//! - macOS 15+ (Sequoia): Full NSGlassEffectView support
//! - macOS < 15: Falls back to NSVisualEffectView
//! - Other platforms: No-op implementations
//!
//! ## Example
//! ```no_run
//! use electron_liquid_glass_rs::{GlassViewManager, GlassOptions};
//!
//! let manager = GlassViewManager::new();
//! let options = GlassOptions {
//!     corner_radius: 16.0,
//!     tint_color: Some("#FF0000AA".to_string()),
//!     opaque: false,
//! };
//!
//! // window_ptr is a pointer to NSView from Electron
//! let view_id = manager.add_glass_view(window_ptr as *mut std::ffi::c_void, options)?;
//! ```

#![warn(missing_docs)]

mod error;
mod platform;

#[cfg(target_os = "macos")]
mod macos;

pub use error::{GlassError, Result};
pub use platform::{GlassOptions, GlassViewManager, GlassMaterialVariant};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = GlassViewManager::new();
        assert!(manager.is_supported() || !cfg!(target_os = "macos"));
    }

    #[test]
    fn test_glass_options_default() {
        let opts = GlassOptions::default();
        assert_eq!(opts.corner_radius, 0.0);
        assert_eq!(opts.opaque, false);
    }
}
