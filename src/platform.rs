//! Platform-specific glass effect management
//!
//! This module provides a cross-platform interface that delegates to
//! platform-specific implementations or provides no-op fallbacks.

use crate::error::{GlassError, Result};
use std::sync::{Arc, Mutex};

/// Glass material variants (based on macOS private API)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum GlassMaterialVariant {
    /// Regular glass effect
    Regular = 0,
    /// Clear glass
    Clear = 1,
    /// Dock-style glass
    Dock = 2,
    /// App icons glass
    AppIcons = 3,
    /// Widgets glass
    Widgets = 4,
    /// Text glass
    Text = 5,
    /// AVPlayer glass
    AVPlayer = 6,
    /// FaceTime glass
    FaceTime = 7,
    /// Control Center glass
    ControlCenter = 8,
    /// Notification Center glass
    NotificationCenter = 9,
    /// Monogram glass
    Monogram = 10,
    /// Bubbles glass
    Bubbles = 11,
    /// Identity glass
    Identity = 12,
    /// Focus border glass
    FocusBorder = 13,
    /// Focus platter glass
    FocusPlatter = 14,
    /// Keyboard glass
    Keyboard = 15,
    /// Sidebar glass
    Sidebar = 16,
    /// Abutted sidebar glass
    AbuttedSidebar = 17,
    /// Inspector glass
    Inspector = 18,
    /// Control glass
    Control = 19,
    /// Loupe glass
    Loupe = 20,
    /// Slider glass
    Slider = 21,
    /// Camera glass
    Camera = 22,
    /// Cartouche popover glass
    CartouchePopover = 23,
}

/// Configuration options for glass views
#[derive(Debug, Clone)]
pub struct GlassOptions {
    /// Corner radius in points (default: 0.0)
    pub corner_radius: f64,
    /// Tint color in hex format (#RRGGBB or #RRGGBBAA)
    pub tint_color: Option<String>,
    /// Whether to add an opaque background layer
    pub opaque: bool,
}

impl Default for GlassOptions {
    fn default() -> Self {
        Self {
            corner_radius: 0.0,
            tint_color: None,
            opaque: false,
        }
    }
}

/// Manager for creating and manipulating glass effect views
pub struct GlassViewManager {
    #[cfg(target_os = "macos")]
    inner: Arc<Mutex<crate::macos::MacOSGlassManager>>,
    
    #[cfg(not(target_os = "macos"))]
    _phantom: std::marker::PhantomData<()>,
}

impl GlassViewManager {
    /// Create a new glass view manager
    pub fn new() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self {
                inner: Arc::new(Mutex::new(crate::macos::MacOSGlassManager::new())),
            }
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            Self {
                _phantom: std::marker::PhantomData,
            }
        }
    }

    /// Check if glass effects are supported on this platform
    pub fn is_supported(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.inner.lock().unwrap().is_supported()
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    /// Add a glass effect view to a window
    ///
    /// # Arguments
    /// * `window_handle` - Pointer to the native window (NSView* on macOS)
    /// * `options` - Configuration options for the glass effect
    ///
    /// # Returns
    /// A unique view ID that can be used to manipulate the view later, or an error
    pub fn add_glass_view(
        &self,
        window_handle: *mut std::ffi::c_void,
        options: GlassOptions,
    ) -> Result<i32> {
        if window_handle.is_null() {
            return Err(GlassError::InvalidHandle);
        }

        #[cfg(target_os = "macos")]
        {
            self.inner
                .lock()
                .unwrap()
                .add_glass_view(window_handle, options)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(GlassError::UnsupportedPlatform)
        }
    }

    /// Set the glass material variant for a view
    ///
    /// This is an experimental API that uses private macOS APIs
    pub fn set_variant(&self, view_id: i32, variant: GlassMaterialVariant) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            self.inner
                .lock()
                .unwrap()
                .set_variant(view_id, variant)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(GlassError::UnsupportedPlatform)
        }
    }

    /// Set the scrim state for a view (0 = none, 1 = light, 2 = dark)
    pub fn set_scrim_state(&self, view_id: i32, state: i64) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            self.inner
                .lock()
                .unwrap()
                .set_int_property(view_id, "scrimState", state)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(GlassError::UnsupportedPlatform)
        }
    }

    /// Set the subdued state for a view
    pub fn set_subdued_state(&self, view_id: i32, state: i64) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            self.inner
                .lock()
                .unwrap()
                .set_int_property(view_id, "subduedState", state)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(GlassError::UnsupportedPlatform)
        }
    }

    /// Remove a glass view by ID
    pub fn remove_view(&self, view_id: i32) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            self.inner.lock().unwrap().remove_view(view_id)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(GlassError::UnsupportedPlatform)
        }
    }
}

impl Default for GlassViewManager {
    fn default() -> Self {
        Self::new()
    }
}
