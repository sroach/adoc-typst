use chrono::Datelike;

use crate::ir::blocks::Block;
use crate::ir::document::Document;
use crate::ir::document::Section;

/// Decode common HTML entities that may appear in titles and metadata.
fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&#8217;", "\u{2019}")  // right single quotation mark
        .replace("&#8216;", "\u{2018}")  // left single quotation mark
        .replace("&#8220;", "\u{201C}")  // left double quotation mark
        .replace("&#8221;", "\u{201D}")  // right double quotation mark
        .replace("&#8211;", "\u{2013}")  // en dash
        .replace("&#8212;", "\u{2014}")  // em dash
        .replace("&#8230;", "\u{2026}")  // horizontal ellipsis
        .replace("&#8203;", "\u{200B}")  // zero-width space
        .replace("&#8482;", "™")          // trademark
        .replace("&#8242;", "\u{2032}")  // prime
        .replace("&#x20ac;", "€")         // euro sign
        .replace("&nbsp;", "\u{00A0}")   // non-breaking space
        .replace("&ndash;", "\u{2013}")  // en dash
        .replace("&mdash;", "\u{2014}")  // em dash
        .replace("&hellip;", "\u{2026}") // horizontal ellipsis
}

/// Strip HTML tags from text (e.g., `<em>text</em>` → `text`).
fn strip_html_tags(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_tag = false;
    for ch in text.chars() {
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

/// Format a date string (YYYY-MM-DD) into a readable format like "October 14, 2024".
fn format_date_readable(date: &str) -> Option<String> {
    if let Ok(d) = date.parse::<chrono::NaiveDate>() {
        let months = [
            "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December",
        ];
        let month = months[(d.month() as usize).saturating_sub(1)];
        Some(format!("{} {}, {}", month, d.day(), d.year()))
    } else {
        None
    }
}

/// Renders the IR document to Typst syntax.
pub fn render_document(doc: &Document, pdf_standard: Option<&str>, lang: &str) -> String {
    let mut output = String::new();

    // Render PDF metadata using #set document(...)
    let has_metadata = doc.title.is_some() || doc.author.is_some() || doc.date.is_some() || !doc.keywords.is_empty();
    if has_metadata {
        output.push_str("#set document(\n");
        if let Some(title) = &doc.title {
            let clean = strip_html_tags(&decode_html_entities(title));
            output.push_str(&format!("  title: [{}],\n", clean));
        }
        if let Some(author) = &doc.author {
            output.push_str(&format!("  author: (\"{}\",),\n", author));
        }
        if let Some(date) = &doc.date {
            // Try to parse YYYY-MM-DD format; fall back to auto if not parseable
            if let Ok(d) = date.parse::<chrono::NaiveDate>() {
                output.push_str(&format!(
                    "  date: datetime(year: {}, month: {}, day: {}),\n",
                    d.year(),
                    d.month(),
                    d.day()
                ));
            } else {
                output.push_str("  date: auto,\n");
            }
        }
        if !doc.keywords.is_empty() {
            let keywords_str = doc.keywords.iter()
                .map(|k| format!("\"{}\"", k))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("  keywords: ({}),\n", keywords_str));
        }
        output.push_str(")\n\n");
    }

    // PDF/UA compliance preamble
    if pdf_standard.is_some() {
        output.push_str(&format!("#set text(lang: \"{}\")\n", lang));
    }

    // Premium design system preamble
    output.push_str("// Premium design system\n");
    output.push_str("#set page(\n");
    output.push_str("  paper: \"a4\",\n");
    output.push_str("  margin: (top: 48pt, bottom: 48pt, left: 48pt, right: 48pt),\n");
    output.push_str(")\n");
    output.push_str("#set text(size: 16pt)\n");
    output.push_str("#set heading(numbering: \"1.1\", depth: 3)\n");
    output.push_str("#show heading: set text(fill: rgb(\"#111827\"), weight: 700)\n");
    output.push_str("#show heading: it => {\n");
    output.push_str("  if it.level == 1 {\n");
    output.push_str("    set text(size: 30pt, weight: 700)\n");
    output.push_str("  } else if it.level == 2 {\n");
    output.push_str("    set text(size: 24pt, weight: 600)\n");
    output.push_str("  }\n");
    output.push_str("  it\n");
    output.push_str("}\n\n");

    // Render cover page with title, author, and date (always when metadata exists)
    let has_cover_metadata = doc.title.is_some() || doc.author.is_some() || doc.date.is_some();
    if has_cover_metadata {
        output.push_str("#v(1fr)\n");
        output.push_str("#align(center)[\n");
        if let Some(title) = &doc.title {
            let clean = strip_html_tags(&decode_html_entities(title));
            output.push_str(&format!("  #text(size: 36pt, weight: 700, fill: rgb(\"#111827\"))[{}]\n", clean));
        }
        if doc.author.is_some() || doc.date.is_some() {
            output.push_str("  #v(24pt)\n");
        }
        if let Some(author) = &doc.author {
            output.push_str(&format!("  #text(fill: rgb(\"#6B7280\"), size: 18pt)[{}]\n", author));
        }
        if let Some(date) = &doc.date {
            if let Some(readable) = format_date_readable(date) {
                output.push_str(&format!("  #text(fill: rgb(\"#6B7280\"))[{}]\n", readable));
            } else {
                output.push_str(&format!("  #text(fill: rgb(\"#6B7280\"))[{}]\n", date));
            }
        }
        output.push_str("]\n");
        output.push_str("#v(1fr)\n");
        output.push_str("#pagebreak()\n");
        // Table of Contents page
        output.push_str("#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[Table of Contents]])\n");
        output.push_str("#v(1fr)\n");
        output.push_str("#align(center)[\n");
        output.push_str("  #text(size: 30pt, weight: 700, fill: rgb(\"#111827\"))[Table of Contents]\n");
        output.push_str("]\n");
        output.push_str("#v(24pt)\n");
        output.push_str("#outline(\n");
        output.push_str("  title: none,\n");
        output.push_str(")\n");
        output.push_str("#v(1fr)\n");
        output.push_str("#pagebreak()\n");
        // Start page numbering after the cover + TOC pages, right-aligned, "X / N" format
        output.push_str("#set page(numbering: \"1 / 1\", number-align: right)\n");
        // Set up running header with refined style (placeholder, replaced by section headers)
        output.push_str("#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[]])\n\n");
    }

    // Determine if the first section title matches the document title (to avoid duplication)
    let doc_title_clean = doc.title.as_ref().map(|t| strip_html_tags(&decode_html_entities(t)));
    let skip_first_section_title = doc_title_clean
        .as_ref()
        .and_then(|dt| {
            doc.sections.first().map(|s| {
                let s_clean = strip_html_tags(&decode_html_entities(&s.title));
                s_clean == *dt
            })
        })
        .unwrap_or(false);

    for (idx, section) in doc.sections.iter().enumerate() {
        // Update running header to current section title with refined style
        let section_title_clean = strip_html_tags(&decode_html_entities(&section.title));
        output.push_str(&format!(
            "#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[{}]])\n",
            section_title_clean
        ));
        output.push_str(&render_section(section, idx == 0 && skip_first_section_title, pdf_standard));
    }

    output
}

fn render_section(section: &Section, skip_title: bool, pdf_standard: Option<&str>) -> String {
    let mut output = String::new();
    if !skip_title {
        let prefix = "=".repeat(section.level as usize + 1);
        let clean = strip_html_tags(&decode_html_entities(&section.title));
        output.push_str(&format!("{} {}\n\n", prefix, clean));
    }

    // Update header for nested subsections with refined style
    for block in &section.content {
        if let Block::Section(nested) = block {
            let nested_title_clean = strip_html_tags(&decode_html_entities(&nested.title));
            output.push_str(&format!(
                "#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[{}]])\n",
                nested_title_clean
            ));
        }
    }

    let pdf_ua = pdf_standard.is_some();
    for block in &section.content {
        output.push_str(&crate::renderer::blocks::render_block(block, pdf_ua));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::blocks::{Block, ListItem};
    use crate::ir::inline::Inline;

    #[test]
    fn test_render_document_with_title() {
        let doc = Document {
            title: Some(String::from("My Document")),
            author: None,
            date: None,
            keywords: vec![],
            sections: vec![],
        };
        let result = render_document(&doc, None, "en");
        assert!(result.contains("#set document("));
        assert!(result.contains("title: [My Document],"));
        // Premium design system preamble
        assert!(result.contains("// Premium design system"));
        assert!(result.contains("#set page("));
        assert!(result.contains("paper: \"a4\""));
        assert!(result.contains("margin: (top: 48pt, bottom: 48pt, left: 48pt, right: 48pt)"));
        assert!(result.contains("#set text(size: 16pt)"));
        assert!(result.contains("#set heading(numbering: \"1.1\""));
        assert!(result.contains("#show heading: it =>"));
        // Cover page
        assert!(result.contains("#v(1fr)"));
        assert!(result.contains("#align(center)"));
        assert!(result.contains("#text(size: 36pt, weight: 700, fill: rgb(\"#111827\"))[My Document]"));
        assert!(result.contains("#pagebreak()"));
        // Table of Contents
        assert!(result.contains("#outline("));
        assert!(result.contains("title: none"));
        assert!(result.contains("#set page(numbering: \"1 / 1\", number-align: right)"));
        assert!(result.contains("#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[]])"));
    }

    #[test]
    fn test_render_document_with_author() {
        let doc = Document {
            title: Some(String::from("Doc")),
            author: Some(String::from("Author Name")),
            date: None,
            keywords: vec![],
            sections: vec![],
        };
        let result = render_document(&doc, None, "en");
        assert!(result.contains("Author Name"));
        assert!(result.contains("rgb(\"#6B7280\")"));
        assert!(result.contains("author: (\"Author Name\",)"));
        assert!(result.contains("#text(fill: rgb(\"#6B7280\"), size: 18pt)[Author Name]"));
    }

    #[test]
    fn test_render_document_with_date() {
        let doc = Document {
            title: Some(String::from("Doc")),
            author: None,
            date: Some(String::from("2024-01-01")),
            keywords: vec![],
            sections: vec![],
        };
        let result = render_document(&doc, None, "en");
        assert!(result.contains("datetime(year: 2024, month: 1, day: 1)"));
        assert!(result.contains("January 1, 2024"));
        assert!(result.contains("rgb(\"#6B7280\")"));
    }

    #[test]
    fn test_render_document_with_all_metadata() {
        let doc = Document {
            title: Some(String::from("Doc")),
            author: Some(String::from("Author")),
            date: Some(String::from("2024-01-01")),
            keywords: vec![String::from("rust"), String::from("asciidoc")],
            sections: vec![],
        };
        let result = render_document(&doc, None, "en");
        assert!(result.contains("#set document("));
        assert!(result.contains("title: [Doc],"));
        assert!(result.contains("author: (\"Author\",)"));
        assert!(result.contains("datetime(year: 2024, month: 1, day: 1)"));
        assert!(result.contains("keywords: (\"rust\", \"asciidoc\")"));
        assert!(result.contains("#v(1fr)"));
        assert!(result.contains("#align(center)"));
        assert!(result.contains("#text(size: 36pt, weight: 700, fill: rgb(\"#111827\"))[Doc]"));
        assert!(result.contains("#text(fill: rgb(\"#6B7280\"), size: 18pt)[Author]"));
        assert!(result.contains("January 1, 2024"));
        assert!(result.contains("#pagebreak()"));
        // Table of Contents
        assert!(result.contains("#outline("));
        assert!(result.contains("title: none"));
        assert!(result.contains("#set page(numbering: \"1 / 1\", number-align: right)"));
        assert!(result.contains("#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[]])"));
    }

    #[test]
    fn test_render_document_without_metadata() {
        let doc = Document {
            title: None,
            author: None,
            date: None,
            keywords: vec![],
            sections: vec![],
        };
        let result = render_document(&doc, None, "en");
        // Premium preamble is always emitted, even for empty documents
        assert!(result.contains("// Premium design system"));
        assert!(result.contains("#set page("));
        assert!(result.contains("#set text(size: 16pt)"));
        assert!(!result.contains("#set document("));
        assert!(!result.contains("#pagebreak()"));
    }

    #[test]
    fn test_render_section_level_1() {
        let section = Section {
            level: 1,
            title: String::from("Section"),
            content: vec![],
        };
        let result = render_section(&section, false, None);
        assert!(result.contains("== Section"));
    }

    #[test]
    fn test_render_section_level_2() {
        let section = Section {
            level: 2,
            title: String::from("Subsection"),
            content: vec![],
        };
        let result = render_section(&section, false, None);
        assert!(result.contains("=== Subsection"));
    }

    #[test]
    fn test_render_section_with_content() {
        let section = Section {
            level: 1,
            title: String::from("Section"),
            content: vec![Block::Paragraph(vec![Inline::Text(String::from("Content"))])],
        };
        let result = render_section(&section, false, None);
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_render_section_skip_title() {
        let section = Section {
            level: 1,
            title: String::from("Section"),
            content: vec![Block::Paragraph(vec![Inline::Text(String::from("Content"))])],
        };
        let result = render_section(&section, true, None);
        assert!(!result.contains("== Section"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_render_document_with_multiple_sections() {
        let doc = Document {
            title: Some(String::from("Doc")),
            author: None,
            date: None,
            keywords: vec![],
            sections: vec![
                Section {
                    level: 1,
                    title: String::from("First"),
                    content: vec![],
                },
                Section {
                    level: 1,
                    title: String::from("Second"),
                    content: vec![],
                },
            ],
        };
        let result = render_document(&doc, None, "en");
        assert!(result.contains("== First"));
        assert!(result.contains("== Second"));
        assert!(result.contains("#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[First]])"));
        assert!(result.contains("#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[Second]])"));
        // Cover page and TOC page each have a pagebreak, no pagebreaks before sections
        let count = result.matches("#pagebreak()").count();
        assert_eq!(count, 2, "Expected 2 pagebreaks (cover + TOC), got {}", count);
    }

    #[test]
    fn test_render_document_level2_section_no_pagebreak() {
        let doc = Document {
            title: Some(String::from("Doc")),
            author: None,
            date: None,
            keywords: vec![],
            sections: vec![
                Section {
                    level: 1,
                    title: String::from("Section"),
                    content: vec![],
                },
                Section {
                    level: 2,
                    title: String::from("Subsection"),
                    content: vec![],
                },
            ],
        };
        let result = render_document(&doc, None, "en");
        // Cover page and TOC page each have a pagebreak, no pagebreaks before sections
        let count = result.matches("#pagebreak()").count();
        assert_eq!(count, 2, "Expected 2 pagebreaks (cover + TOC), got {}", count);
        assert!(result.contains("=== Subsection"));
    }

    #[test]
    fn test_render_document_with_list_in_section() {
        let doc = Document {
            title: None,
            author: None,
            date: None,
            keywords: vec![],
            sections: vec![Section {
                level: 1,
                title: String::from("List Section"),
                content: vec![Block::UnorderedList(vec![ListItem {
                    content: vec![Inline::Text(String::from("Item 1"))],
                    children: vec![],
                }])],
            }],
        };
        let result = render_document(&doc, None, "en");
        assert!(result.contains("- Item 1"));
    }

    #[test]
    fn test_render_document_skips_duplicate_first_section_title() {
        let doc = Document {
            title: Some(String::from("My Document")),
            author: Some(String::from("Author")),
            date: None,
            keywords: vec![],
            sections: vec![
                Section {
                    level: 1,
                    title: String::from("My Document"),
                    content: vec![Block::Paragraph(vec![Inline::Text(String::from("Content"))])],
                },
                Section {
                    level: 1,
                    title: String::from("Second"),
                    content: vec![],
                },
            ],
        };
        let result = render_document(&doc, None, "en");
        // First section title should be skipped (duplicate of doc title)
        assert!(!result.contains("== My Document"));
        // Second section title should still appear
        assert!(result.contains("== Second"));
        // Content of first section should still render
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_render_document_keeps_first_section_title_if_different() {
        let doc = Document {
            title: Some(String::from("My Document")),
            author: None,
            date: None,
            keywords: vec![],
            sections: vec![
                Section {
                    level: 1,
                    title: String::from("Introduction"),
                    content: vec![],
                },
            ],
        };
        let result = render_document(&doc, None, "en");
        // First section title is different from doc title, should appear
        assert!(result.contains("== Introduction"));
    }
}
