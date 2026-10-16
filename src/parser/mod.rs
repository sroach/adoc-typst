/// Parse Asciidoc source text into an `asciidoc_parser::Document`.
pub use asciidoc_parser;

/// Parse Asciidoc source text into a Document using the asciidoc-parser crate.
pub fn parse(source: &str) -> asciidoc_parser::Document<'_> {
    let mut parser = asciidoc_parser::Parser::default();
    parser.parse(source)
}
