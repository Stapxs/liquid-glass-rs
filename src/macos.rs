//! macOS-specific glass effect implementation using Objective-C runtime

use crate::error::{GlassError, Result};
use crate::platform::{GlassMaterialVariant, GlassOptions};
use cocoa::appkit::{NSVisualEffectView, NSColor};
use cocoa::base::{id, nil, YES};
use cocoa::foundation::{NSRect, NSString};
use objc::runtime::{Class, Sel};
use objc::{msg_send, sel, sel_impl, class};
use std::collections::HashMap;
use std::ffi::c_void;

/// Manager for macOS glass effects
pub struct MacOSGlassManager {
    views: HashMap<i32, id>,
    next_id: i32,
}

impl MacOSGlassManager {
    /// Create a new macOS glass manager
    pub fn new() -> Self {
        Self {
            views: HashMap::new(),
            next_id: 0,
        }
    }

    /// Check if glass effects are supported on this macOS version
    pub fn is_supported(&self) -> bool {
        // NSGlassEffectView is available on macOS 15+
        Class::get("NSGlassEffectView").is_some()
    }

    /// Add a glass effect view to a window
    pub fn add_glass_view(
        &mut self,
        window_handle: *mut c_void,
        options: GlassOptions,
    ) -> Result<i32> {
        unsafe {
            // Check main thread
            let current_thread: id = msg_send![class!(NSThread), currentThread];
            let is_main: bool = msg_send![current_thread, isMainThread];
            if !is_main {
                return Err(GlassError::RuntimeError(
                    "Must be called from main thread".to_string(),
                ));
            }

            // Cast the window handle to NSView
            let root_view = window_handle as id;
            if root_view.is_null() {
                return Err(GlassError::InvalidHandle);
            }

            // Get bounds
            let bounds: NSRect = msg_send![root_view, bounds];

            // Create background view if opaque
            let background_view = if options.opaque {
                Some(self.create_background_view(bounds)?)
            } else {
                None
            };

            // Try to create NSGlassEffectView first, fall back to NSVisualEffectView
            let glass_view = if let Some(glass_view) = self.create_glass_view(bounds)? {
                glass_view
            } else {
                self.create_fallback_view(bounds)?
            };

            // Add views to container
            if let Some(bg) = background_view {
                self.add_subview(root_view, bg, nil)?;
            }

            let relative_to = background_view.unwrap_or(nil);
            self.add_subview(root_view, glass_view, relative_to)?;

            // Configure the glass view
            self.configure_glass_view(glass_view, &options)?;

            // Store view ID
            let view_id = self.next_id;
            self.next_id += 1;
            
            self.views.insert(view_id, glass_view);

            Ok(view_id)
        }
    }

    /// Create an NSGlassEffectView if available
    unsafe fn create_glass_view(&self, bounds: NSRect) -> Result<Option<id>> {
        if let Some(glass_class) = Class::get("NSGlassEffectView") {
            let instance: id = msg_send![glass_class, alloc];
            let instance: id = msg_send![instance, initWithFrame: bounds];
            
            if !instance.is_null() {
                // Enable autoresizing (NSViewWidthSizable | NSViewHeightSizable)
                let mask: usize = 2 | 16;
                let _: () = msg_send![instance, setAutoresizingMask: mask];
                return Ok(Some(instance));
            }
        }
        Ok(None)
    }

    /// Create fallback NSVisualEffectView
    unsafe fn create_fallback_view(&self, bounds: NSRect) -> Result<id> {
        let visual = NSVisualEffectView::alloc(nil);
        let visual: id = msg_send![visual, initWithFrame: bounds];
        
        if visual.is_null() {
            return Err(GlassError::CreationFailed);
        }

        // Configure visual effect view
        // blendingMode = 0 (behindWindow)
        let _: () = msg_send![visual, setBlendingMode: 0_isize];
        // material = 0 (underWindowBackground)  
        let _: () = msg_send![visual, setMaterial: 0_isize];
        // state = 1 (active)
        let _: () = msg_send![visual, setState: 1_isize];
        
        // Enable autoresizing
        let mask: usize = 2 | 16;
        let _: () = msg_send![visual, setAutoresizingMask: mask];

        Ok(visual)
    }

    /// Create opaque background view
    unsafe fn create_background_view(&self, bounds: NSRect) -> Result<id> {
        let box_class = Class::get("NSBox").ok_or(GlassError::CreationFailed)?;
        let bg: id = msg_send![box_class, alloc];
        let bg: id = msg_send![bg, initWithFrame: bounds];
        
        if bg.is_null() {
            return Err(GlassError::CreationFailed);
        }

        // Configure box
        let _: () = msg_send![bg, setBoxType: 4_isize]; // NSBoxCustom
        let _: () = msg_send![bg, setBorderType: 0_isize]; // NSNoBorder
        
        // Set background color
        let window_bg_class = Class::get("NSColor").ok_or(GlassError::CreationFailed)?;
        let window_bg_color: id = msg_send![window_bg_class, windowBackgroundColor];
        let _: () = msg_send![bg, setFillColor: window_bg_color];
        
        // Enable layer and autoresizing
        let _: () = msg_send![bg, setWantsLayer: YES];
        let mask: usize = 2 | 16;
        let _: () = msg_send![bg, setAutoresizingMask: mask];

        Ok(bg)
    }

    /// Add subview with positioning
    unsafe fn add_subview(
        &self,
        container: id,
        subview: id,
        relative_to: id,
    ) -> Result<()> {
        let positioned = -1_isize; // NSWindowBelow
        
        let _: () = msg_send![
            container,
            addSubview: subview
            positioned: positioned
            relativeTo: relative_to
        ];
        
        Ok(())
    }

    /// Configure glass view with options
    unsafe fn configure_glass_view(&self, view: id, options: &GlassOptions) -> Result<()> {
        // Set corner radius
        if options.corner_radius > 0.0 {
            let _: () = msg_send![view, setWantsLayer: YES];
            let layer: id = msg_send![view, layer];
            if !layer.is_null() {
                let _: () = msg_send![layer, setCornerRadius: options.corner_radius];
                let _: () = msg_send![layer, setMasksToBounds: YES];
            }
        }

        // Set tint color
        if let Some(ref tint) = options.tint_color {
            if let Ok(color) = self.parse_hex_color(tint) {
                // Try to set tintColor using runtime
                let sel = sel!(setTintColor:);
                let responds: bool = msg_send![view, respondsToSelector: sel];
                if responds {
                    let _: () = msg_send![view, setTintColor: color];
                } else {
                    let layer: id = msg_send![view, layer];
                    if !layer.is_null() {
                        // Fallback to layer backgroundColor
                        let cg_color: id = msg_send![color, CGColor];
                        let _: () = msg_send![layer, setBackgroundColor: cg_color];
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse hex color string to NSColor
    unsafe fn parse_hex_color(&self, hex: &str) -> Result<id> {
        let cleaned = hex.trim().trim_start_matches('#');
        
        if cleaned.len() != 6 && cleaned.len() != 8 {
            return Err(GlassError::InvalidColor(hex.to_string()));
        }

        let rgba = u32::from_str_radix(cleaned, 16)
            .map_err(|_| GlassError::InvalidColor(hex.to_string()))?;

        let (r, g, b, a) = if cleaned.len() == 6 {
            (
                ((rgba >> 16) & 0xFF) as f64 / 255.0,
                ((rgba >> 8) & 0xFF) as f64 / 255.0,
                (rgba & 0xFF) as f64 / 255.0,
                1.0,
            )
        } else {
            (
                ((rgba >> 24) & 0xFF) as f64 / 255.0,
                ((rgba >> 16) & 0xFF) as f64 / 255.0,
                ((rgba >> 8) & 0xFF) as f64 / 255.0,
                (rgba & 0xFF) as f64 / 255.0,
            )
        };

        let color = NSColor::colorWithSRGBRed_green_blue_alpha_(nil, r, g, b, a);
        Ok(color)
    }

    /// Set glass material variant
    pub fn set_variant(&self, view_id: i32, variant: GlassMaterialVariant) -> Result<()> {
        self.set_int_property(view_id, "variant", variant as i64)
    }

    /// Set integer property using runtime
    pub fn set_int_property(&self, view_id: i32, key: &str, value: i64) -> Result<()> {
        let view = self.views.get(&view_id)
            .ok_or(GlassError::InvalidViewId(view_id))?;

        unsafe {
            // Try private setter first (set_key:)
            let private_setter = format!("set_{}:", key);
            if let Some(sel) = self.try_get_selector(&private_setter) {
                let responds: bool = msg_send![*view, respondsToSelector: sel];
                if responds {
                    // Use NSInvocation or performSelector for setting int values
                    let number: id = msg_send![class!(NSNumber), numberWithLongLong: value];
                    let _: () = msg_send![*view, setValue:number forKey: NSString::alloc(nil).init_str(key)];
                    return Ok(());
                }
            }

            // Try public setter (setKey:)
            let public_setter = format!(
                "set{}{}:",
                key.chars().next().unwrap().to_uppercase(),
                &key[1..]
            );
            if let Some(sel) = self.try_get_selector(&public_setter) {
                let responds: bool = msg_send![*view, respondsToSelector: sel];
                if responds {
                    let number: id = msg_send![class!(NSNumber), numberWithLongLong: value];
                    let _: () = msg_send![*view, setValue:number forKey: NSString::alloc(nil).init_str(key)];
                    return Ok(());
                }
            }

            Err(GlassError::RuntimeError(format!(
                "Property '{}' not found or not accessible",
                key
            )))
        }
    }

    /// Try to get a selector
    fn try_get_selector(&self, name: &str) -> Option<Sel> {
        Some(Sel::register(name))
    }

    /// Remove a glass view
    pub fn remove_view(&mut self, view_id: i32) -> Result<()> {
        let view = self.views.remove(&view_id)
            .ok_or(GlassError::InvalidViewId(view_id))?;

        unsafe {
            let _: () = msg_send![view, removeFromSuperview];
        }

        Ok(())
    }
}
