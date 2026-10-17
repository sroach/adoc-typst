use std::collections::HashSet;

use chrono::Datelike;

use crate::ir::blocks::Block;
use crate::ir::document::Document;
use crate::ir::document::Section;
use crate::renderer::util::decode_html_entities;

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
    let mut rendered_footnotes = HashSet::new();

    let mut output = String::new();

    // Render PDF metadata using #set document(...)
    let has_metadata = doc.title.is_some() || !doc.authors.is_empty() || doc.date.is_some() || !doc.keywords.is_empty();
    if has_metadata {
        output.push_str("#set document(\n");
        if let Some(title) = &doc.title {
            let clean = strip_html_tags(&decode_html_entities(title));
            output.push_str(&format!("  title: [{}],\n", clean));
        }
        if !doc.authors.is_empty() {
            let authors_str = doc.authors.iter()
                .map(|a| format!("\"{}\"", a))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!("  author: ({}),\n", authors_str));
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
    output.push_str("#set text(size: 10pt)\n");
    output.push_str("#set heading(numbering: \"1.1\", depth: 3)\n");
    output.push_str("#show heading: set text(fill: rgb(\"#111827\"), weight: 700)\n");
    output.push_str("#show heading: set block(outset: (top: 0pt, bottom: 4pt))\n");
    output.push_str("#show heading: it => {\n");
    output.push_str("  if it.level == 1 {\n");
    output.push_str("    set text(size: 16pt, weight: 700)\n");
    output.push_str("  } else if it.level == 2 {\n");
    output.push_str("    set text(size: 13pt, weight: 600)\n");
    output.push_str("  }\n");
    output.push_str("  it\n");
    output.push_str("}\n\n");

    // Render cover page with title, subtitle, authors, revision, and date
    let has_cover_metadata = doc.title.is_some() || !doc.authors.is_empty() || doc.date.is_some() || doc.subtitle.is_some() || doc.revision.is_some();
    if has_cover_metadata {
        output.push_str("#v(1fr)\n");
        output.push_str("#align(center)[\n");
        // Title (large)
        if let Some(title) = &doc.title {
            let clean = strip_html_tags(&decode_html_entities(title));
            output.push_str(&format!("  #text(size: 36pt, weight: 700, fill: rgb(\"#111827\"))[{}]\n", clean));
        }
        // Subtitle (smaller, separate line)
        if let Some(subtitle) = &doc.subtitle {
            let clean = strip_html_tags(&decode_html_entities(subtitle));
            output.push_str("  #v(12pt)\n");
            output.push_str(&format!("  #text(size: 20pt, fill: rgb(\"#6B7280\"))[{}]\n", clean));
        }
        // Authors (all of them)
        if !doc.authors.is_empty() {
            output.push_str("  #v(24pt)\n");
            let authors_str = doc.authors.join(", ");
            output.push_str(&format!("  #text(fill: rgb(\"#6B7280\"), size: 18pt)[{}]\n", authors_str));
        }
        // Revision info and date
        if doc.revision.is_some() || doc.date.is_some() {
            output.push_str("  #v(12pt)\n");
            let mut rev_parts = Vec::new();
            if let Some(rev) = &doc.revision {
                rev_parts.push(format!("Version {}", rev));
            }
            if let Some(date) = &doc.date {
                if let Some(readable) = format_date_readable(date) {
                    rev_parts.push(readable);
                } else {
                    rev_parts.push(date.clone());
                }
            }
            if !rev_parts.is_empty() {
                output.push_str(&format!("  #text(fill: rgb(\"#6B7280\"))[{}]\n", rev_parts.join(", ")));
            }
        }
        output.push_str("]\n");
        output.push_str("#v(1fr)\n");
        output.push_str("#pagebreak()\n");
        // Set page numbering before TOC so it.page() resolves correctly
        if let Some(org) = &doc.organization {
            output.push_str(&format!(
                "#set page(footer: [{}], numbering: \"1 / 1\", number-align: right)\n",
                org
            ));
        } else {
            output.push_str("#set page(numbering: \"1 / 1\", number-align: right)\n");
        }
        // Table of Contents page
        output.push_str("#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[Table of Contents]])\n");
        output.push_str("#v(1fr)\n");
        output.push_str("#align(center)[\n");
        output.push_str("  #text(size: 30pt, weight: 700, fill: rgb(\"#111827\"))[Table of Contents]\n");
        output.push_str("]\n");
        output.push_str("#v(24pt)\n");
        // Workaround for Typst heading counter starting at 0 - reset before content
        output.push_str("#counter(heading).update(1)\n");
        // Custom TOC with dotted leaders matching Asciidoctor PDF style
        output.push_str("#show outline.entry: it => link(\n");
        output.push_str("  it.element.location(),\n");
        output.push_str("  it.indented(\n");
        output.push_str("    it.prefix(),\n");
        output.push_str("    [\n");
        output.push_str("      #it.body()\n");
        output.push_str("      #box(width: 1fr, it.fill)\n");
        output.push_str("      #it.page()\n");
        output.push_str("    ]\n");
        output.push_str("  )\n");
        output.push_str(")\n");
        output.push_str("#outline(\n");
        output.push_str("  title: none,\n");
        output.push_str(")\n");
        output.push_str("#show outline.entry: set outline.entry(fill: [])\n");
        output.push_str("#v(1fr)\n");
        output.push_str("#pagebreak()\n");
        // Set footer and page numbering for content pages
        if let Some(org) = &doc.organization {
            output.push_str(&format!(
                "#set page(footer: [{}], numbering: \"1 / 1\", number-align: right)\n",
                org
            ));
        } else {
            output.push_str("#set page(numbering: \"1 / 1\", number-align: right)\n");
        }
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

    // Determine if the first section is a preamble/abstract (empty title with doc title present)
    let has_preamble = doc.title.is_some()
        && doc.sections.first().map(|s| s.title.is_empty()).unwrap_or(false);
    
    // Track whether we've rendered the first level-1 section (to avoid double pagebreak after TOC)
    let mut first_level_1_after_toc = true;
    
    for (idx, section) in doc.sections.iter().enumerate() {
        let section_title_clean = strip_html_tags(&decode_html_entities(&section.title));
        
        // Handle preamble/abstract sections (first section with empty title when doc has a title)
        if idx == 0 && has_preamble && section_title_clean.is_empty() {
            // Render the abstract content as "Preface" without affecting heading numbering
            output.push_str(&format!(
                "#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[Preface] #line(stroke: 1pt + rgb(\"#3B82F6\"))])\n"
            ));
            // Use text instead of heading to avoid consuming a heading number
            output.push_str("#text(size: 16pt, weight: 700, fill: rgb(\"#111827\"))[Preface]\n\n");
            // Render the abstract content blocks
            let pdf_ua = pdf_standard.is_some();
            for block in &section.content {
                output.push_str(&crate::renderer::blocks::render_block_with_footnotes(
                    block,
                    pdf_ua,
                    &mut rendered_footnotes,
                ));
            }
            // Don't add pagebreak after preface - let content flow naturally
            continue;
        }
        
        // Add page break before level-1 sections (matching Asciidoctor PDF behavior)
        // Skip for the first level-1 section after TOC since TOC already ends with pagebreak
        if section.level == 1 && !first_level_1_after_toc {
            output.push_str("#pagebreak()\n");
        } else if section.level == 1 {
            first_level_1_after_toc = false;
        }
        // Update running header to current section title with colored bar
        if !section_title_clean.is_empty() {
            output.push_str(&format!(
                "#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[{}] #line(stroke: 1pt + rgb(\"#3B82F6\"))])\n",
                section_title_clean
            ));
        }
        // Adjust skip_first_section_title for preamble-skipped sections
        let adjusted_idx = if has_preamble { idx.saturating_sub(1) } else { idx };
        output.push_str(&render_section(
            section,
            adjusted_idx == 0 && skip_first_section_title,
            pdf_standard,
            &mut rendered_footnotes,
        ));
    }

    output
}

fn render_section(
    section: &Section,
    skip_title: bool,
    pdf_standard: Option<&str>,
    rendered_footnotes: &mut HashSet<String>,
) -> String {
    let mut output = String::new();
    if !skip_title {
        let clean = strip_html_tags(&decode_html_entities(&section.title));
        // Skip rendering heading for sections with empty titles (e.g., abstract with [%notitle])
        if !clean.is_empty() {
            let prefix = "=".repeat(section.level as usize + 1);
            output.push_str(&format!("{} {}\n\n", prefix, clean));
        }
    }

    // Update header for nested subsections with colored bar
    for block in &section.content {
        if let Block::Section(nested) = block {
            let nested_title_clean = strip_html_tags(&decode_html_entities(&nested.title));
            output.push_str(&format!(
                "#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[{}] #line(stroke: 1pt + rgb(\"#3B82F6\"))])\n",
                nested_title_clean
            ));
        }
    }

    let pdf_ua = pdf_standard.is_some();
    for block in &section.content {
        output.push_str(&crate::renderer::blocks::render_block_with_footnotes(
            block,
            pdf_ua,
            rendered_footnotes,
        ));
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
            subtitle: None,
            authors: vec![],
            date: None,
            revision: None,
            keywords: vec![],
            organization: None,
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
        assert!(result.contains("#set text(size: 10pt)"));
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
    }

    #[test]
    fn test_render_document_with_author() {
        let doc = Document {
            title: Some(String::from("Doc")),
            subtitle: None,
            authors: vec![String::from("Author Name")],
            date: None,
            revision: None,
            keywords: vec![],
            organization: None,
            sections: vec![],
        };
        let result = render_document(&doc, None, "en");
        assert!(result.contains("Author Name"));
        assert!(result.contains("rgb(\"#6B7280\")"));
        assert!(result.contains("author: (\"Author Name\")"));
        assert!(result.contains("#text(fill: rgb(\"#6B7280\"), size: 18pt)[Author Name]"));
    }

    #[test]
    fn test_render_document_with_date() {
        let doc = Document {
            title: Some(String::from("Doc")),
            subtitle: None,
            authors: vec![],
            date: Some(String::from("2024-01-01")),
            revision: None,
            keywords: vec![],
            organization: None,
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
            subtitle: Some(String::from("A Subtitle")),
            authors: vec![String::from("Author")],
            date: Some(String::from("2024-01-01")),
            revision: Some(String::from("1.0")),
            keywords: vec![String::from("rust"), String::from("asciidoc")],
            organization: Some(String::from("Company Name")),
            sections: vec![],
        };
        let result = render_document(&doc, None, "en");
        assert!(result.contains("#set document("));
        assert!(result.contains("title: [Doc],"));
        assert!(result.contains("author: (\"Author\")"));
        assert!(result.contains("datetime(year: 2024, month: 1, day: 1)"));
        assert!(result.contains("keywords: (\"rust\", \"asciidoc\")"));
        assert!(result.contains("#v(1fr)"));
        assert!(result.contains("#align(center)"));
        assert!(result.contains("#text(size: 36pt, weight: 700, fill: rgb(\"#111827\"))[Doc]"));
        // Subtitle
        assert!(result.contains("#text(size: 20pt, fill: rgb(\"#6B7280\"))[A Subtitle]"));
        assert!(result.contains("#text(fill: rgb(\"#6B7280\"), size: 18pt)[Author]"));
        // Revision info
        assert!(result.contains("Version 1.0"));
        assert!(result.contains("January 1, 2024"));
        assert!(result.contains("#pagebreak()"));
        // Table of Contents
        assert!(result.contains("#outline("));
        assert!(result.contains("title: none"));
        // Footer with organization
        assert!(result.contains("Company Name"));
    }

    #[test]
    fn test_render_document_without_metadata() {
        let doc = Document {
            title: None,
            subtitle: None,
            authors: vec![],
            date: None,
            revision: None,
            keywords: vec![],
            organization: None,
            sections: vec![],
        };
        let result = render_document(&doc, None, "en");
        // Premium preamble is always emitted, even for empty documents
        assert!(result.contains("// Premium design system"));
        assert!(result.contains("#set page("));
        assert!(result.contains("#set text(size: 10pt)"));
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
        let result = render_section(&section, false, None, &mut HashSet::new());
        assert!(result.contains("== Section"));
    }

    #[test]
    fn test_render_section_level_2() {
        let section = Section {
            level: 2,
            title: String::from("Subsection"),
            content: vec![],
        };
        let result = render_section(&section, false, None, &mut HashSet::new());
        assert!(result.contains("=== Subsection"));
    }

    #[test]
    fn test_render_section_with_content() {
        let section = Section {
            level: 1,
            title: String::from("Section"),
            content: vec![Block::Paragraph(vec![Inline::Text(String::from("Content"))])],
        };
        let result = render_section(&section, false, None, &mut HashSet::new());
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_render_section_skip_title() {
        let section = Section {
            level: 1,
            title: String::from("Section"),
            content: vec![Block::Paragraph(vec![Inline::Text(String::from("Content"))])],
        };
        let result = render_section(&section, true, None, &mut HashSet::new());
        assert!(!result.contains("== Section"));
        assert!(result.contains("Content"));
    }

    #[test]
    fn test_render_document_with_multiple_sections() {
        let doc = Document {
            title: Some(String::from("Doc")),
            subtitle: None,
            authors: vec![],
            date: None,
            revision: None,
            keywords: vec![],
            organization: None,
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
        // Headers now include colored bar
        assert!(result.contains("#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[First] #line(stroke: 1pt + rgb(\"#3B82F6\"))])"));
        assert!(result.contains("#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[Second] #line(stroke: 1pt + rgb(\"#3B82F6\"))])"));
        // Cover page + TOC page + 1 level-1 section pagebreak (2nd section only, 1st follows TOC) = 3 pagebreaks
        let count = result.matches("#pagebreak()").count();
        assert_eq!(count, 3, "Expected 3 pagebreaks (cover + TOC + 1 section), got {}", count);
    }

    #[test]
    fn test_render_document_level2_section_no_pagebreak() {
        let doc = Document {
            title: Some(String::from("Doc")),
            subtitle: None,
            authors: vec![],
            date: None,
            revision: None,
            keywords: vec![],
            organization: None,
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
        // Cover + TOC = 2 pagebreaks (first level-1 section follows TOC, no extra pagebreak)
        // Level-2 section should NOT get a pagebreak
        let count = result.matches("#pagebreak()").count();
        assert_eq!(count, 2, "Expected 2 pagebreaks (cover + TOC), got {}", count);
        assert!(result.contains("=== Subsection"));
    }

    #[test]
    fn test_render_document_with_list_in_section() {
        let doc = Document {
            title: None,
            subtitle: None,
            authors: vec![],
            date: None,
            revision: None,
            keywords: vec![],
            organization: None,
            sections: vec![Section {
                level: 1,
                title: String::from("List Section"),
                content: vec![Block::UnorderedList(vec![ListItem {
                    content: vec![Inline::Text(String::from("Item 1"))],
                    children: vec![],
                    blocks: vec![],
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
            subtitle: None,
            authors: vec![String::from("Author")],
            date: None,
            revision: None,
            keywords: vec![],
            organization: None,
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
            subtitle: None,
            authors: vec![],
            date: None,
            revision: None,
            keywords: vec![],
            organization: None,
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
