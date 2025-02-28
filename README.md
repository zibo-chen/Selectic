# Selectic

**Selectic** is a Rust library that provides a cross-platform way to retrieve user-selected content from the operating system. Currently, it focuses on obtaining selected text, but it is designed to be extensible to handle other types of selected content like images and files in the future.

This crate aims to abstract away the platform-specific APIs for accessing selected content, offering a simple and consistent interface for Rust developers to integrate selection retrieval into their applications.

## Features

- **Cross-platform text selection:** Effortlessly retrieve the text currently selected by the user, regardless of the application they are using.
- **Platform Support:** Currently supports:
  - macOS
  - Windows
  - Linux
- **Extensible Content Types:** Designed with the future in mind, Selectic can be expanded to support:
  - Image data
  - File paths
  - Other content types as needed.

**Currently Implemented:**

- ✅ **Text Selection:** Get the user's selected text as a `String`.

**Planned Features (Contributions Welcome\!):**

- 🚧 **Image Selection:** Retrieve selected image data with format information.
- 🚧 **File Selection:** Obtain paths of selected files.

## Usage

To use Selectic in your Rust project, add the following to your `Cargo.toml` file under `[dependencies]`:

```toml
selectic = "0.1.0" # Replace with the actual version
```

**Example: Getting Selected Text**

Here's a simple example demonstrating how to get the currently selected text:

```rust
use selectic;

fn main() {
    match selectic::get_text() {
        Ok(text) => {
            println!("Selected text: {:?}", text);
        }
        Err(err) => {
            eprintln!("Error getting selected text: {:?}", err);
        }
    }
}
```

To run this example:

1.  Make sure you have some text selected in another application (e.g., a text editor, web browser).
2.  Run your Rust program.
3.  The program will attempt to retrieve the selected text and print it to the console.

## Platform Support Details

Selectic leverages platform-specific APIs to access selected content:

- **macOS:** Uses Accessibility APIs for robust and reliable selection retrieval.
- **Windows:** Employs clipboard functionality to get selected content.
- **Linux:** Utilizes clipboard mechanisms, potentially requiring clipboard managers for optimal functionality (implementation details in progress).

If your platform is not explicitly listed, Selectic will return an `UnsupportedPlatform` error.

## Contributions

Contributions to Selectic are highly welcome\! If you are interested in helping to expand Selectic's capabilities, particularly with image and file selection, or improving platform support, please feel free to:

- **Fork the repository** and submit pull requests with your enhancements.
- **Report issues** and suggest new features.
- **Help with testing** on different platforms and environments.

Areas for potential contributions include:

- Implementing image data retrieval across platforms.
- Implementing file path retrieval.
- Adding support for more Linux clipboard managers or selection mechanisms.
- Improving error handling and robustness.
- Expanding platform support to other operating systems.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

## Repository

[https://github.com/zibo-chen/Selectic](https://github.com/zibo-chen/Selectic)

---

