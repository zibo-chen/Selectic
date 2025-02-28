use crate::{Selection, SelectionError, Selector};
use std::io::Read;
use std::time::Duration;
use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};
use wl_clipboard_rs::utils::is_primary_selection_supported;
use x11_clipboard::Clipboard;

pub struct LinuxSelector;

impl LinuxSelector {
    pub fn new() -> Self {
        LinuxSelector
    }
}

impl Default for LinuxSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl Selector for LinuxSelector {
    fn get_selection(&self) -> Result<Selection, SelectionError> {
        match std::env::var("XDG_SESSION_TYPE") {
            Ok(session_type) => match session_type.as_str() {
                "x11" => self.get_selection_on_x11(),
                "wayland" => self.get_selection_on_wayland(),
                _ => Err(SelectionError::UnsupportedPlatform),
            },
            Err(_) => Err(SelectionError::UnsupportedPlatform),
        }
    }
}

impl LinuxSelector {
    fn get_selection_on_x11(&self) -> Result<Selection, SelectionError> {
        let clipboard = Clipboard::new().map_err(|_| {
            SelectionError::ClipboardError("Failed to create clipboard".to_string())
        })?;
        let primary = clipboard
            .load(
                clipboard.getter.atoms.primary,
                clipboard.getter.atoms.utf8_string,
                clipboard.getter.atoms.property,
                Duration::from_millis(100),
            )
            .map_err(|_| {
                SelectionError::ClipboardError("Failed to load clipboard data".to_string())
            })?;

        let result = String::from_utf8_lossy(&primary)
            .trim_matches('\u{0}')
            .trim()
            .to_string();

        Ok(Selection::new_text(result))
    }

    fn get_selection_on_wayland(&self) -> Result<Selection, SelectionError> {
        if let Ok(support) = is_primary_selection_supported() {
            if !support {
                return self.get_selection_on_x11();
            }
        } else {
            return self.get_selection_on_x11();
        }

        let (mut pipe, _) = get_contents(ClipboardType::Primary, Seat::Unspecified, MimeType::Text)
            .map_err(|_| {
                SelectionError::ClipboardError("Failed to get contents from Wayland".to_string())
            })?;
        let mut contents = vec![];
        pipe.read_to_end(&mut contents)
            .map_err(|_| SelectionError::ClipboardError("Failed to read contents".to_string()))?;

        let contents = String::from_utf8_lossy(&contents)
            .trim_matches('\u{0}')
            .trim()
            .to_string();

        Ok(Selection::new_text(contents))
    }
}
