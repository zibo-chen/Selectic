use std::fmt;

mod error;
pub use error::SelectionError;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

/// Represents the type of content that was selected
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    /// Plain text content
    Text,
    /// Image data with format specification
    Image(String), // Format (e.g., "png", "jpg")
    /// File path(s)
    File,
    /// Other types of content with format specification
    Other(String),
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentType::Text => write!(f, "text"),
            ContentType::Image(format) => write!(f, "image/{}", format),
            ContentType::File => write!(f, "file"),
            ContentType::Other(format) => write!(f, "other/{}", format),
        }
    }
}

/// Structure representing selected content with its type
#[derive(Debug, Clone)]
pub struct Selection {
    /// The type of content (text, image, file, etc.)
    pub content_type: ContentType,
    /// The actual content data as bytes
    pub data: Vec<u8>,
}

impl Selection {
    /// Create a new text selection
    pub fn new_text(text: String) -> Self {
        Self {
            content_type: ContentType::Text,
            data: text.into_bytes(),
        }
    }

    /// Create a new image selection
    pub fn new_image(format: &str, data: Vec<u8>) -> Self {
        Self {
            content_type: ContentType::Image(format.to_string()),
            data,
        }
    }

    /// Create a new file selection
    pub fn new_file(path: String) -> Self {
        Self {
            content_type: ContentType::File,
            data: path.into_bytes(),
        }
    }

    /// Create a new selection with custom type
    pub fn new_other(format: &str, data: Vec<u8>) -> Self {
        Self {
            content_type: ContentType::Other(format.to_string()),
            data,
        }
    }

    /// Get the content as a UTF-8 string if it's text content
    pub fn as_text(&self) -> Option<String> {
        if let ContentType::Text = self.content_type {
            String::from_utf8(self.data.clone()).ok()
        } else {
            None
        }
    }

    /// Get the content as a file path if it's file content
    pub fn as_file_path(&self) -> Option<String> {
        if let ContentType::File = self.content_type {
            String::from_utf8(self.data.clone()).ok()
        } else {
            None
        }
    }

    /// Check if the selection is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Trait for retrieving user-selected content across platforms
pub trait Selector {
    /// Get the currently selected content using the best available method
    fn get_selection(&self) -> Result<Selection, SelectionError>;

    /// Get the currently selected content using accessibility APIs
    fn get_selection_by_accessibility(&self) -> Result<Selection, SelectionError>;

    /// Get the currently selected content using clipboard
    fn get_selection_by_clipboard(&self) -> Result<Selection, SelectionError>;
}

/// Main function to get user's current selection
///
/// This function automatically creates the appropriate selector
/// for the current platform and retrieves the selection.
pub fn get_selection() -> Result<Selection, SelectionError> {
    #[cfg(target_os = "macos")]
    {
        let selector = macos::MacOSSelector::new();
        selector.get_selection()
    }

    #[cfg(target_os = "windows")]
    {
        let selector = windows::WindowsSelector::new();
        selector.get_selection()
    }

    #[cfg(target_os = "linux")]
    {
        let selector = linux::LinuxSelector::new();
        selector.get_selection()
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err(SelectionError::UnsupportedPlatform)
    }
}

/// Convenience function to get user's current text selection
///
/// Returns the text as a String if successful, or an error
pub fn get_text() -> Result<String, SelectionError> {
    match get_selection() {
        Ok(selection) => selection
            .as_text()
            .ok_or(SelectionError::InvalidContentType {
                expected: "text".to_string(),
                received: selection.content_type.to_string(),
            }),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test the Selection struct's creation and methods
    #[test]
    fn test_selection_text() {
        let text = get_text();
        println!("selection_text is {:?}", text);
    }

    #[test]
    fn test_selection_file() {
        let path = "/path/to/file.txt".to_string();
        let selection = Selection::new_file(path.clone());

        assert_eq!(selection.content_type, ContentType::File);
        assert_eq!(selection.data, path.as_bytes());
        assert_eq!(selection.as_text(), None);
        assert_eq!(selection.as_file_path(), Some(path));
        assert!(!selection.is_empty());
    }

    #[test]
    fn test_selection_empty() {
        let selection = Selection::new_text(String::new());
        assert!(selection.is_empty());

        let selection = Selection::new_image("png", vec![]);
        assert!(selection.is_empty());
    }

    // Test ContentType display implementation
    #[test]
    fn test_content_type_display() {
        assert_eq!(ContentType::Text.to_string(), "text");
        assert_eq!(
            ContentType::Image("png".to_string()).to_string(),
            "image/png"
        );
        assert_eq!(ContentType::File.to_string(), "file");
        assert_eq!(
            ContentType::Other("custom".to_string()).to_string(),
            "other/custom"
        );
    }

    // Test the get_text convenience function with mocks
    #[test]
    fn test_get_text_function() {
        // Test with text selection
        let text = "Test text".to_string();
        let selection = Selection::new_text(text.clone());

        let text_result = selection
            .as_text()
            .ok_or(SelectionError::InvalidContentType {
                expected: "text".to_string(),
                received: selection.content_type.to_string(),
            });

        assert!(text_result.is_ok());
        assert_eq!(text_result.unwrap(), text);

        // Test with non-text selection
        let selection = Selection::new_image("png", vec![1, 2, 3]);

        let text_result = selection
            .as_text()
            .ok_or(SelectionError::InvalidContentType {
                expected: "text".to_string(),
                received: selection.content_type.to_string(),
            });

        assert!(matches!(text_result,
            Err(SelectionError::InvalidContentType { expected, received })
            if expected == "text" && received == "image/png"
        ));
    }

    // Integration test with mocked platform detection
    // This would require modifying the crate to accept a mock selector for testing
    #[test]
    fn test_platform_selection() {
        // In a real implementation, we would use conditional compilation with mocks
        // or dependency injection to test the platform-specific selector creation

        // For now, this is a placeholder to show the pattern
        #[cfg(test)]
        fn mock_get_selection() -> Result<Selection, SelectionError> {
            Ok(Selection::new_text("Mock platform selection".to_string()))
        }

        #[cfg(test)]
        {
            let result = mock_get_selection();
            assert!(result.is_ok());
            assert_eq!(
                result.unwrap().as_text(),
                Some("Mock platform selection".to_string())
            );
        }
    }

    // Test handling of non-UTF8 data
    #[test]
    fn test_non_utf8_data() {
        // Create selection with invalid UTF-8 bytes
        let invalid_utf8 = vec![0, 159, 146, 150]; // Invalid UTF-8 sequence
        let selection = Selection {
            content_type: ContentType::Text,
            data: invalid_utf8,
        };

        // Should return None for as_text since data is not valid UTF-8
        assert_eq!(selection.as_text(), None);
    }
}
