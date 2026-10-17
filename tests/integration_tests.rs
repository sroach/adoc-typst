/// Integration tests for the full Asciidoc → Typst conversion pipeline.
use adoc_typst::parser;
use adoc_typst::converter::mapper;
use adoc_typst::renderer::document;

fn convert(adoc: &str) -> String {
    let ast = parser::parse(adoc);
    let ir = mapper::convert(&ast);
    document::render_document(&ir, None, "en")
}

#[test]
fn test_empty_document() {
    let result = convert("");
    // Premium preamble is always emitted, even for empty documents
    assert!(result.contains("// Premium design system"));
    assert!(result.contains("#set page("));
}

#[test]
fn test_document_title() {
    let result = convert("= My Title");
    assert!(result.contains("#set document("));
    assert!(result.contains("title: [My Title],"));
    assert!(result.contains("// Premium design system"));
    assert!(result.contains("#v(1fr)"));
    assert!(result.contains("#align(center)"));
    assert!(result.contains("#text(size: 36pt, weight: 700, fill: rgb(\"#111827\"))[My Title]"));
    assert!(result.contains("#pagebreak()"));
    // Table of Contents
    assert!(result.contains("#outline("));
    assert!(result.contains("title: none"));
    assert!(result.contains("#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[]])"));
}

#[test]
fn test_paragraph() {
    let result = convert("This is a paragraph.");
    assert!(result.contains("This is a paragraph."));
}

#[test]
fn test_bold_text() {
    let result = convert("This is *bold* text.");
    assert!(result.contains("#strong[bold]"));
}

#[test]
fn test_italic_text() {
    let result = convert("This is _italic_ text.");
    assert!(result.contains("#text(style: \"italic\")[italic]"));
}

#[test]
fn test_inline_code() {
    let result = convert("Use `code` here.");
    assert!(result.contains("=code="));
}

#[test]
fn test_section_heading() {
    let result = convert("== Section Title");
    assert!(result.contains("== Section Title"));
    // First level-1 section after TOC does not get an extra pagebreak
    // (TOC already ends with pagebreak, avoiding blank pages)
}

#[test]
fn test_subsection_heading() {
    let result = convert("=== Subsection");
    assert!(result.contains("=== Subsection"));
    // No pagebreaks before subsections
    assert!(!result.contains("#pagebreak()"));
}

#[test]
fn test_unordered_list() {
    let result = convert("- Item one\n- Item two");
    assert!(result.contains("- Item one"));
    assert!(result.contains("- Item two"));
}

#[test]
fn test_ordered_list() {
    let result = convert(". Item one\n. Item two");
    assert!(result.contains("+ Item one"));
    assert!(result.contains("+ Item two"));
}

#[test]
fn test_source_block() {
    let result = convert("[source,rust]\n----\nfn main() {}\n----");
    assert!(result.contains("```rust"));
    assert!(result.contains("fn main() {}"));
}

#[test]
fn test_note_admonition() {
    let result = convert("NOTE: This is a note.");
    assert!(result.contains("Note"));
    assert!(result.contains("This is a note."));
}

#[test]
fn test_tip_admonition() {
    let result = convert("TIP: This is a tip.");
    assert!(result.contains("Tip"));
}

#[test]
fn test_warning_admonition() {
    let result = convert("WARNING: Be careful.");
    assert!(result.contains("Warning"));
}

#[test]
fn test_important_admonition() {
    let result = convert("IMPORTANT: Pay attention.");
    assert!(result.contains("Important"));
}

#[test]
fn test_caution_admonition() {
    let result = convert("CAUTION: Be cautious.");
    assert!(result.contains("Caution"));
}

#[test]
fn test_link() {
    let result = convert("https://example.com[Example]");
    assert!(result.contains("#link(\"https://example.com\")[#text(fill: blue)[Example]]"));
}

#[test]
fn test_image() {
    let result = convert("image::diagram.png[Diagram]");
    assert!(result.contains("#image(\"diagram.png\", width: 90%, alt: \"Diagram\")"));
    assert!(result.contains("#align(center)"));
}

#[test]
fn test_table() {
    let result = convert("|===\n|Header\n|===\n");
    assert!(result.contains("#table("));
}

#[test]
fn test_quote_block() {
    let result = convert("[quote, Author]\n____\nQuoted text.\n____");
    assert!(result.contains(">"));
    assert!(result.contains("Author"));
}

#[test]
fn test_highlight() {
    let result = convert("#highlighted#");
    assert!(result.contains("#highlight[highlighted]"));
}

#[test]
fn test_superscript() {
    let result = convert("x^2^");
    assert!(result.contains("#super[2]"));
}

#[test]
fn test_subscript() {
    let result = convert("H~2~O");
    assert!(result.contains("#sub[2]"));
}

#[test]
fn test_footnote() {
    let result = convert("Text footnote:[Note here.]");
    assert!(result.contains("#footnote[Note here.]"));
}

#[test]
fn test_author_and_date() {
    let result = convert("= Title\nAuthor Name\n2024-01-01");
    assert!(result.contains("#set document("));
    assert!(result.contains("title: [Title],"));
    assert!(result.contains("Author Name"));
    assert!(result.contains("January 1, 2024"));
    assert!(result.contains("#v(1fr)"));
    assert!(result.contains("#align(center)"));
    assert!(result.contains("#pagebreak()"));
    // Table of Contents
    assert!(result.contains("#outline("));
    assert!(result.contains("title: none"));
    assert!(result.contains("#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[]])"));
}

#[test]
fn test_complex_document() {
    let adoc = "\
= Test Document
Author <email@example.com>
2024-01-01

== Introduction

This is a *bold* and _italic_ paragraph.

== List

- First item
- Second item

== Code

[source,rust]
----
fn main() {}
----

NOTE: A note here.
";
    let result = convert(adoc);
    assert!(result.contains("#set document("));
    assert!(result.contains("title: [Test Document],"));
    assert!(result.contains("Author"));
    assert!(result.contains("#v(1fr)"));
    assert!(result.contains("#align(center)"));
    assert!(result.contains("#pagebreak()"));
    // Table of Contents
    assert!(result.contains("#outline("));
    assert!(result.contains("title: none"));
    assert!(result.contains("#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[]])"));
    // Headers now include colored bar
    assert!(result.contains("#set page(header: [#text(size: 12pt, weight: 400, fill: rgb(\"#3B82F6\"))[Introduction] #line(stroke: 1pt + rgb(\"#3B82F6\"))])"));
    assert!(result.contains("== Introduction"));
    assert!(result.contains("#strong[bold]"));
    assert!(result.contains("#text(style: \"italic\")[italic]"));
    assert!(result.contains("== List"));
    assert!(result.contains("- First item"));
    assert!(result.contains("== Code"));
    assert!(result.contains("```rust"));
    assert!(result.contains("fn main() {}"));
    assert!(result.contains("Note"));
}
