use anyhow::{Context, Result};
use base64::Engine;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

use crate::remote::svg::resolve_css_variables;

/// Represents a custom macro block that needs remote rendering.
#[derive(Debug, Clone)]
pub struct RemoteMacro {
    /// The macro type (e.g., "recipe", "chart", "diagram").
    pub viz_type: String,
    /// The raw body content of the block.
    pub body: String,
}

/// Compresses a string payload using gzip and returns a base64-encoded string.
pub fn compress_and_encode(payload: &str) -> Result<String> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(payload.as_bytes())
        .context("Failed to gzip compress payload")?;
    let compressed = encoder
        .finish()
        .context("Failed to finish gzip compression")?;
    Ok(base64::engine::general_purpose::STANDARD.encode(compressed))
}

/// Fetches SVG from the remote docops extension server.
/// Sends a GET request with gzip-compressed data in query parameters.
///
/// # Arguments
/// * `server_url` - Base URL of the docops extension server (e.g. "http://localhost:3000")
/// * `viz_type` - The visualization type (e.g. "recipe", "chart")
/// * `data` - The raw data payload
///
/// # Returns
/// The SVG content returned by the server
pub fn fetch_svg(server_url: &str, viz_type: &str, data: &str) -> Result<String> {
    let encoded = compress_and_encode(data)?;
    let request_url = format!(
        "{}/svg?type={}&data={}&format=pdf",
        server_url.trim_end_matches('/'),
        urlencoding(viz_type),
        urlencoding(&encoded)
    );

    println!("[adoc-typst] Fetching SVG from: {}", request_url);

    let response = reqwest::blocking::get(&request_url)
        .context("Failed to send HTTP request")?
        .error_for_status()
        .context(format!(
            "Server returned error status for type '{}'",
            viz_type
        ))?;

    let mut svg = response.text().context("Failed to read response body")?;
    
    // Fallback: if the server didn't resolve CSS variables (no format=pdf support),
    // resolve them client-side so the SVG renders correctly in Typst/PDF
    if svg.contains("var(") {
        println!("[adoc-typst] Resolving CSS variables in SVG for PDF compatibility");
        svg = resolve_css_variables(&svg);
    }
    
    Ok(svg)
}

/// Simple URL encoding for query parameters.
fn urlencoding(input: &str) -> String {
    let mut result = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'_' | b'~' => {
                result.push(byte as char)
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// Saves the fetched SVG content to a file and returns the relative path.
///
/// # Arguments
/// * `assets_dir` - Directory to save the SVG file
/// * `viz_type` - The visualization type (used for the filename)
/// * `svg` - The SVG content
/// * `index` - Index for unique filename generation when multiple macros exist
///
/// # Returns
/// The relative path to the saved SVG file
pub fn save_svg(assets_dir: &str, viz_type: &str, svg: &str, index: usize) -> Result<String> {
    use std::fs;

    fs::create_dir_all(assets_dir).context("Failed to create assets directory")?;

    let filename = format!("docops-{}-{}.svg", viz_type, index);
    let path = format!("{}/{}", assets_dir, filename);

    fs::write(&path, svg).context(format!("Failed to write SVG file: {}", path))?;

    Ok(filename)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_and_encode() {
        let result = compress_and_encode("Hello, world!").unwrap();
        // Just verify it produces a non-empty base64 string
        assert!(!result.is_empty());
        // Base64 should only contain valid characters
        for ch in result.chars() {
            assert!(
                ch.is_ascii_alphanumeric() || ch == '=' || ch == '+' || ch == '/',
                "Invalid base64 character: {}",
                ch
            );
        }
    }

    #[test]
    fn test_compress_and_encode_empty() {
        let result = compress_and_encode("").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_urlencoding() {
        assert_eq!(urlencoding("hello"), "hello");
        assert_eq!(urlencoding("hello world"), "hello%20world");
        assert_eq!(urlencoding("a=b&c=d"), "a%3Db%26c%3Dd");
        assert_eq!(urlencoding("recipe"), "recipe");
    }

    #[test]
    fn test_save_svg() {
        let dir = std::env::temp_dir().join("adoc-typst-test");
        let _ = std::fs::remove_dir_all(&dir);
        let svg = "<svg></svg>";
        let path = save_svg(dir.to_str().unwrap(), "recipe", svg, 0).unwrap();
        assert_eq!(path, "docops-recipe-0.svg");
        let full_path = dir.join(&path);
        assert!(full_path.exists());
        let content = std::fs::read_to_string(full_path).unwrap();
        assert_eq!(content, svg);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
