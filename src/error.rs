//! Error types for the glass effect library

/// Result type alias for glass operations
pub type Result<T> = std::result::Result<T, GlassError>;

/// Errors that can occur when working with glass effects
#[derive(Debug, Clone, thiserror::Error)]
pub enum GlassError {
    /// Platform not supported
    #[error("Glass effects are not supported on this platform")]
    UnsupportedPlatform,

    /// Invalid window handle
    #[error("Invalid window handle provided")]
    InvalidHandle,

    /// Invalid view ID
    #[error("View ID {0} not found")]
    InvalidViewId(i32),

    /// Color parsing error
    #[error("Invalid color format: {0}")]
    InvalidColor(String),

    /// Runtime error from Objective-C
    #[error("Objective-C runtime error: {0}")]
    RuntimeError(String),

    /// View creation failed
    #[error("Failed to create glass view")]
    CreationFailed,
}
