use crate::ir::blocks::{AdmonitionLevel, Block, ListItem};
use crate::renderer::util::decode_html_entities;

/// Renders a block to Typst syntax.
pub fn render_block(block: &Block, pdf_ua: bool) -> String {
    match block {
        Block::Paragraph(inlines) => {
            let text = super::inline::render_inlines(inlines);
            format!("{}\n\n", text)
        }
        Block::OrderedList(items) => {
            let mut output = String::new();
            for item in items {
                output.push_str(&render_list_item(item, true, 0));
            }
            output.push_str("\n");
            output
        }
        Block::UnorderedList(items) => {
            let mut output = String::new();
            for item in items {
                output.push_str(&render_list_item(item, false, 0));
            }
            output.push_str("\n");
            output
        }
        Block::CodeBlock { language, title, lines } => {
            let mut output = String::new();
            if let Some(t) = title {
                output.push_str(&format!("#title(\"{}\")\n", t));
            }
            let lang = if language.is_empty() { "" } else { language.as_str() };
            output.push_str(&format!("```{}\n", lang));
            for line in lines {
                output.push_str(&format!("{}\n", line));
            }
            output.push_str("```\n\n");
            output
        }
        Block::QuoteBlock { content, attribution, citation } => {
            let mut output = String::new();
            for block in content {
                let rendered = render_block(block, pdf_ua);
                for line in rendered.lines() {
                    output.push_str(&format!("> {}\n", line));
                }
            }
            let mut attr_parts = Vec::new();
            if let Some(attr) = attribution {
                attr_parts.push(attr.clone());
            }
            if let Some(cite) = citation {
                let cleaned = strip_html_tags(cite);
                attr_parts.push(format!("#strong[{}]", cleaned));
            }
            if !attr_parts.is_empty() {
                output.push_str(&format!("\n-- {}\n\n", attr_parts.join(", ")));
            } else {
                output.push_str("\n");
            }
            output
        }
        Block::Admonition { level, content } => {
            let (label, icon) = match level {
                AdmonitionLevel::Note => ("Note", "📝"),
                AdmonitionLevel::Tip => ("Tip", "💡"),
                AdmonitionLevel::Important => ("Important", "❗"),
                AdmonitionLevel::Warning => ("Warning", "⚠️"),
                AdmonitionLevel::Caution => ("Caution", "⚠️"),
            };
            let mut output = String::new();
            output.push_str(&format!("[#text(size: 1.1em)[#strong[{} {}]]]\n", icon, label));
            for block in content {
                output.push_str(&render_block(block, pdf_ua));
            }
            output.push_str("\n");
            output
        }
        Block::Table { headers, rows } => {
            let mut output = String::new();
            let num_cols = if !headers.is_empty() {
                headers.len()
            } else if !rows.is_empty() {
                rows[0].len()
            } else {
                0
            };
            if num_cols == 0 {
                return String::new();
            }
            output.push_str(&format!("#table(columns: {},\n", num_cols));
            if !headers.is_empty() {
                if pdf_ua {
                    // PDF/UA requires table.header() for proper PDF tagging
                    // Note: no # prefix — we're already inside #table() code context
                    let header_cells: Vec<String> = headers
                        .iter()
                        .map(|cell| format!("[{}]", super::inline::render_inlines(cell)))
                        .collect();
                    output.push_str(&format!("  table.header({}),\n", header_cells.join(", ")));
                } else {
                    let header_cells: Vec<String> = headers
                        .iter()
                        .map(|cell| format!("[{}]", super::inline::render_inlines(cell)))
                        .collect();
                    output.push_str(&format!("  {},\n", header_cells.join(", ")));
                }
            }
            for row in rows {
                let cells: Vec<String> = row
                    .iter()
                    .map(|cell| format!("[{}]", super::inline::render_inlines(cell)))
                    .collect();
                output.push_str(&format!("  {},\n", cells.join(", ")));
            }
            output.push_str(")\n\n");
            output
        }
        Block::Image { path, alt } => {
            let mut output = String::new();
            output.push_str("#v(16pt)\n");
            if !alt.is_empty() && pdf_ua {
                // PDF/UA requires figure element for proper PDF tagging
                // Note: no # prefix on image() — we're already inside #figure() code context
                output.push_str("#figure(\n");
                output.push_str(&format!("  image(\"{}\", width: 90%),\n", path));
                output.push_str(&format!("  caption: [{}]\n", alt));
                output.push_str(")\n");
            } else {
                output.push_str("#align(center)[\n");
                if !alt.is_empty() {
                    output.push_str(&format!("  #image(\"{}\", width: 90%, alt: \"{}\")\n", path, alt));
                } else {
                    output.push_str(&format!("  #image(\"{}\", width: 90%)\n", path));
                }
                output.push_str("]\n");
            }
            output.push_str("#v(16pt)\n\n");
            output
        }
        Block::RemoteMacro { viz_type, body } => {
            // If the body is empty, the remote macro was not resolved (no --remote-macro-url provided).
            // Fall back to rendering as a code block so the document still compiles.
            if body.is_empty() {
                let mut output = String::new();
                output.push_str(&format!("```{}\n", viz_type));
                output.push_str("Remote macro not resolved. Pass --remote-macro-url to render.\n");
                output.push_str("```\n\n");
                output
            } else {
                // Body contains the resolved SVG path — render as a centered image.
                let mut output = String::new();
                output.push_str("#v(16pt)\n");
                output.push_str("#align(center)[\n");
                output.push_str(&format!("  #image(\"{}\", width: 90%)\n", body));
                output.push_str("]\n");
                output.push_str("#v(16pt)\n\n");
                output
            }
        }
        Block::Section(nested) => {
            let mut output = String::new();
            let prefix = "=".repeat(nested.level as usize + 1);
            let clean_title = strip_html_tags(&decode_html_entities(&nested.title));
            output.push_str(&format!("{} {}\n\n", prefix, clean_title));
            for block in &nested.content {
                output.push_str(&render_block(block, pdf_ua));
            }
            output.push_str("\n");
            output
        }
    }
}

/// Strip HTML tags from text rendered by the asciidoc-parser's inline renderer.
/// The parser renders inline markup (like *bold*) as HTML (<strong>bold</strong>),
/// so we strip the tags for Typst output.
fn strip_html_tags(input: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in input.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }
    result
}

fn render_list_item(item: &ListItem, ordered: bool, indent: usize) -> String {
    let prefix = if ordered { "+ " } else { "- " };
    let text = super::inline::render_inlines(&item.content);
    let indent_str = "  ".repeat(indent);
    let mut output = format!("{}{}{}\n", indent_str, prefix, text);
    for child in &item.children {
        output.push_str(&render_list_item(child, ordered, indent + 1));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::inline::Inline;

    #[test]
    fn test_render_paragraph() {
        let block = Block::Paragraph(vec![Inline::Text(String::from("Hello"))]);
        assert_eq!(render_block(&block, false), "Hello\n\n");
    }

    #[test]
    fn test_render_unordered_list() {
        let items = vec![
            ListItem {
                content: vec![Inline::Text(String::from("First"))],
                children: vec![],
            },
            ListItem {
                content: vec![Inline::Text(String::from("Second"))],
                children: vec![],
            },
        ];
        let block = Block::UnorderedList(items);
        let result = render_block(&block, false);
        assert!(result.contains("- First"));
        assert!(result.contains("- Second"));
    }

    #[test]
    fn test_render_ordered_list() {
        let items = vec![ListItem {
            content: vec![Inline::Text(String::from("First"))],
            children: vec![],
        }];
        let block = Block::OrderedList(items);
        let result = render_block(&block, false);
        assert!(result.contains("+ First"));
    }

    #[test]
    fn test_render_code_block_with_language() {
        let block = Block::CodeBlock {
            language: String::from("rust"),
            title: None,
            lines: vec![String::from("fn main() {}")],
        };
        let result = render_block(&block, false);
        assert!(result.contains("```rust"));
        assert!(result.contains("fn main() {}"));
    }

    #[test]
    fn test_render_code_block_without_language() {
        let block = Block::CodeBlock {
            language: String::new(),
            title: None,
            lines: vec![String::from("plain text")],
        };
        let result = render_block(&block, false);
        assert!(result.contains("```\nplain text"));
    }

    #[test]
    fn test_render_code_block_with_title() {
        let block = Block::CodeBlock {
            language: String::from("python"),
            title: Some(String::from("Example")),
            lines: vec![String::from("print('hi')")],
        };
        let result = render_block(&block, false);
        assert!(result.contains("#title(\"Example\")"));
    }

    #[test]
    fn test_render_quote_with_attribution() {
        let block = Block::QuoteBlock {
            content: vec![Block::Paragraph(vec![Inline::Text(String::from("Words"))])],
            attribution: Some(String::from("Author")),
            citation: Some(String::from("Book")),
        };
        let result = render_block(&block, false);
        assert!(result.contains(">"));
        assert!(result.contains("-- Author"));
        assert!(result.contains("#strong[Book]"));
    }

    #[test]
    fn test_render_quote_without_attribution() {
        let block = Block::QuoteBlock {
            content: vec![Block::Paragraph(vec![Inline::Text(String::from("Words"))])],
            attribution: None,
            citation: None,
        };
        let result = render_block(&block, false);
        assert!(!result.contains("--"));
    }

    #[test]
    fn test_render_note_admonition() {
        let block = Block::Admonition {
            level: AdmonitionLevel::Note,
            content: vec![Block::Paragraph(vec![Inline::Text(String::from("Info"))])],
        };
        let result = render_block(&block, false);
        assert!(result.contains("Note"));
        assert!(result.contains("📝"));
    }

    #[test]
    fn test_render_tip_admonition() {
        let block = Block::Admonition {
            level: AdmonitionLevel::Tip,
            content: vec![Block::Paragraph(vec![Inline::Text(String::from("Hint"))])],
        };
        let result = render_block(&block, false);
        assert!(result.contains("Tip"));
        assert!(result.contains("💡"));
    }

    #[test]
    fn test_render_warning_admonition() {
        let block = Block::Admonition {
            level: AdmonitionLevel::Warning,
            content: vec![Block::Paragraph(vec![Inline::Text(String::from("Careful"))])],
        };
        let result = render_block(&block, false);
        assert!(result.contains("Warning"));
    }

    #[test]
    fn test_render_table_with_headers() {
        let block = Block::Table {
            headers: vec![
                vec![Inline::Text(String::from("Name"))],
                vec![Inline::Text(String::from("Age"))],
            ],
            rows: vec![vec![
                vec![Inline::Text(String::from("Alice"))],
                vec![Inline::Text(String::from("30"))],
            ]],
        };
        let result = render_block(&block, false);
        assert!(result.contains("#table(columns: 2,"));
        assert!(result.contains("[Name]"));
        assert!(result.contains("[Age]"));
        assert!(result.contains("[Alice]"));
        assert!(result.contains("[30]"));
    }

    #[test]
    fn test_render_image_with_alt() {
        let block = Block::Image {
            path: String::from("diagram.png"),
            alt: String::from("Architecture"),
        };
        let result = render_block(&block, false);
        assert!(result.contains("#image(\"diagram.png\", width: 90%, alt: \"Architecture\")"));
        assert!(result.contains("#align(center)"));
        assert!(result.contains("#v(16pt)"));
    }

    #[test]
    fn test_render_image_without_alt() {
        let block = Block::Image {
            path: String::from("photo.jpg"),
            alt: String::new(),
        };
        let result = render_block(&block, false);
        assert!(result.contains("#image(\"photo.jpg\", width: 90%)"));
        assert!(result.contains("#align(center)"));
        assert!(!result.contains("alt:"));
    }

    #[test]
    fn test_render_nested_list_item() {
        let item = ListItem {
            content: vec![Inline::Text(String::from("Parent"))],
            children: vec![ListItem {
                content: vec![Inline::Text(String::from("Child"))],
                children: vec![],
            }],
        };
        let result = render_list_item(&item, false, 0);
        assert!(result.contains("- Parent"));
        assert!(result.contains("  - Child"));
    }
}
