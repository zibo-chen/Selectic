//! macOS implementation for the selection library

use accessibility_ng::{AXAttribute, AXUIElement};
use accessibility_sys_ng::{kAXFocusedUIElementAttribute, kAXSelectedTextAttribute};
use core_foundation::string::CFString;
use log::{error, info};
use std::process::Command;

use crate::{Selection, SelectionError, Selector};

/// macOS implementation of the Selector trait
pub struct MacOSSelector;

impl MacOSSelector {
    /// Create a new macOS selector
    pub fn new() -> Self {
        MacOSSelector
    }
}

impl Default for MacOSSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl Selector for MacOSSelector {
    /// Get user selection using the best available method for macOS
    fn get_selection(&self) -> Result<Selection, SelectionError> {
        // Try accessibility API first
        match get_selection_by_accessibility() {
            Ok(selection) if !selection.is_empty() => {
                info!("Retrieved selection via macOS accessibility API");
                Ok(selection)
            }
            Ok(_) => {
                info!("Selection via macOS accessibility API is empty");
                // Fall back to clipboard method
                get_selection_by_clipboard()
            }
            Err(err) => {
                error!(
                    "Error getting selection via macOS accessibility API: {}",
                    err
                );
                // Fall back to clipboard method
                get_selection_by_clipboard()
            }
        }
    }
}

/// Get selected text from macOS using the best available method
///
/// This is a convenience function for macOS-specific code
pub fn get_macos_text() -> Result<String, SelectionError> {
    let selector = MacOSSelector::new();
    match selector.get_selection() {
        Ok(selection) => selection
            .as_text()
            .ok_or(SelectionError::InvalidContentType {
                expected: "text".to_string(),
                received: selection.content_type.to_string(),
            }),
        Err(err) => Err(err),
    }
}

/// Get user selection using macOS Accessibility API
fn get_selection_by_accessibility() -> Result<Selection, SelectionError> {
    let system_element = AXUIElement::system_wide();

    // Get focused UI element - fixing the type conversion issues
    let focused_element_result = system_element.attribute(&AXAttribute::new(
        &CFString::from_static_string(kAXFocusedUIElementAttribute),
    ));

    if focused_element_result.is_err() {
        return Err(SelectionError::NoFocusedElement);
    }

    let element_value = focused_element_result.unwrap();
    let focused_element = match element_value.downcast_into::<AXUIElement>() {
        Some(element) => element,
        None => return Err(SelectionError::NoFocusedElement),
    };

    // Get selected text from focused element
    let selected_text_result = focused_element.attribute(&AXAttribute::new(
        &CFString::from_static_string(kAXSelectedTextAttribute),
    ));

    if selected_text_result.is_err() {
        return Err(SelectionError::NoSelectedContent);
    }

    let text_value = selected_text_result.unwrap();
    let selected_text = match text_value.downcast_into::<CFString>() {
        Some(text) => text,
        None => return Err(SelectionError::NoSelectedContent),
    };

    Ok(Selection::new_text(selected_text.to_string()))
}

/// Get user selection using macOS clipboard
fn get_selection_by_clipboard() -> Result<Selection, SelectionError> {
    const APPLE_SCRIPT: &str = r#"
use AppleScript version "2.4"
use scripting additions
use framework "Foundation"
use framework "AppKit"

set initialClipboard to the clipboard

tell application "System Events"
    keystroke "c" using {command down}
end tell
delay 0.1

set copiedText to the clipboard

if copiedText is not initialClipboard and copiedText is not "" then
    set the clipboard to initialClipboard
end if

copiedText
"#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(APPLE_SCRIPT)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SelectionError::AppleScriptError(stderr.to_string()));
    }

    let content = String::from_utf8(output.stdout)?;

    // Check if we got a file path
    if content.starts_with("[FILE]") {
        let file_path = content.trim_start_matches("[FILE]").to_string();
        Ok(Selection::new_file(file_path))
    }
    // Otherwise, assume it's text
    else {
        Ok(Selection::new_text(content))
    }
}
