#[cfg(target_os = "macos")]
use accessibility_ng::Error as AccessibilityErrorNg;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SelectionError {
    #[error("No focused UI element found")]
    NoFocusedElement,

    #[error("No selected content in focused element")]
    NoSelectedContent,

    #[error("Unsupported platform")]
    UnsupportedPlatform,

    #[error("Invalid content type: expected {expected}, received {received}")]
    InvalidContentType { expected: String, received: String },

    #[error("AppleScript execution failed: {0}")]
    AppleScriptError(String),

    #[error("Accessibility API error: {0}")]
    AccessibilityError(String),

    #[error("Clipboard error: {0}")]
    ClipboardError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Selection error: {0}")]
    Other(String),
}

impl From<String> for SelectionError {
    fn from(error: String) -> Self {
        SelectionError::Other(error)
    }
}

impl From<&str> for SelectionError {
    fn from(error: &str) -> Self {
        SelectionError::Other(error.to_string())
    }
}

#[cfg(target_os = "macos")]
impl From<AccessibilityErrorNg> for SelectionError {
    fn from(error: AccessibilityErrorNg) -> Self {
        SelectionError::AccessibilityError(error.to_string())
    }
}
