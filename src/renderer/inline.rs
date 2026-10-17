use std::collections::HashSet;

use crate::ir::inline::Inline;
use crate::renderer::util::decode_html_entities;

/// Renders a list of inline elements to Typst syntax.
pub fn render_inlines(inlines: &[Inline]) -> String {
    let mut rendered = HashSet::new();
    render_inlines_with_footnotes(inlines, &mut rendered)
}

/// Renders a list of inline elements to Typst syntax, with footnote deduplication.
pub fn render_inlines_with_footnotes(
    inlines: &[Inline],
    rendered_footnotes: &mut HashSet<String>,
) -> String {
    let mut output = String::new();
    for inline in inlines {
        output.push_str(&render_inline_with_footnotes(inline, rendered_footnotes));
    }
    output
}

/// Escape special Typst characters inside inline code (`=...=`).
/// In Typst code mode, `$` triggers math mode, `\` is an escape character,
/// and `=` ends the code span. These must be escaped with a backslash.
fn escape_typst_code(code: &str) -> String {
    let mut result = String::with_capacity(code.len());
    for ch in code.chars() {
        match ch {
            '\\' | '$' | '=' => {
                result.push('\\');
                result.push(ch);
            }
            _ => result.push(ch),
        }
    }
    result
}

#[cfg(test)]
fn render_inline(inline: &Inline) -> String {
    let mut rendered = HashSet::new();
    render_inline_with_footnotes(inline, &mut rendered)
}

fn render_inline_with_footnotes(
    inline: &Inline,
    rendered_footnotes: &mut HashSet<String>,
) -> String {
    match inline {
        Inline::Text(text) => {
            let decoded = decode_html_entities(text);
            // Escape $ in plain text to prevent Typst math mode ($...$ is math)
            let escaped = decoded.replace('$', "\\$");
            // Escape @ to prevent Typst from interpreting it as a label reference
            let escaped = escaped.replace('@', "\\@");
            escaped
        }
        Inline::Bold(children) => {
            let inner = render_inlines_with_footnotes(children, rendered_footnotes);
            // Use #strong[] instead of *...* because Typst delimiters require
            // word boundaries — *ul*timate fails but #strong[ul]timate works.
            format!("#strong[{}]", inner)
        }
        Inline::Italic(children) => {
            let inner = render_inlines_with_footnotes(children, rendered_footnotes);
            // Use #text(style: "italic")[] instead of _..._ because Typst delimiters
            // require word boundaries — _war_lock fails but #text(style: "italic")[war]lock works.
            format!("#text(style: \"italic\")[{}]", inner)
        }
        Inline::Code(code) => {
            let escaped = escape_typst_code(code);
            format!("={}=", escaped)
        }
        Inline::Link { text, url } => {
            // Use Typst's #link() function with blue color for links
            // URLs must be quoted strings so special chars like & don't break parsing
            // Decode HTML entities in URL (e.g., &amp; → &)
            let decoded_url = decode_html_entities(url);
            let escaped_text = text.replace('@', "\\@");
            if *text == *url {
                format!(
                    "#link(\"{}\")[#text(fill: blue)[({})]]",
                    decoded_url, decoded_url
                )
            } else {
                format!(
                    "#link(\"{}\")[#text(fill: blue)[{}]]",
                    decoded_url, escaped_text
                )
            }
        }
        Inline::Mark(children) => {
            let inner = render_inlines_with_footnotes(children, rendered_footnotes);
            format!("#highlight[{}]", inner)
        }
        Inline::Superscript(children) => {
            let inner = render_inlines_with_footnotes(children, rendered_footnotes);
            format!("#super[{}]", inner)
        }
        Inline::Subscript(children) => {
            let inner = render_inlines_with_footnotes(children, rendered_footnotes);
            format!("#sub[{}]", inner)
        }
        Inline::Footnote { id, text } => {
            // Deduplicate footnotes by text content.
            // First occurrence: render with a label.
            // Subsequent occurrences: render as @label cross-reference.
            // If text is just a number (footnote reference back), treat as cross-reference.
            let is_reference = text.chars().all(|c| c.is_ascii_digit());
            if is_reference || rendered_footnotes.contains(text.as_str()) {
                // Footnote reference or already rendered — use cross-reference
                if let Some(id_val) = id {
                    return format!("@fn-{}", id_val);
                }
                // No id — use counter-based label
                return format!("@fn-{}", rendered_footnotes.len());
            }
            // First occurrence — render with label
            rendered_footnotes.insert(text.clone());
            if let Some(id_val) = id {
                return format!("#footnote[{}] <fn-{}>", text, id_val);
            }
            format!("#footnote[{}]", text)
        }
        Inline::LineBreak => {
            String::from("#linebreak()\n")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_text_inline() {
        let inline = Inline::Text(String::from("Hello"));
        assert_eq!(render_inline(&inline), "Hello");
    }

    #[test]
    fn test_render_bold_inline() {
        let inline = Inline::Bold(vec![Inline::Text(String::from("bold"))]);
        assert_eq!(render_inline(&inline), "#strong[bold]");
    }

    #[test]
    fn test_render_italic_inline() {
        let inline = Inline::Italic(vec![Inline::Text(String::from("italic"))]);
        assert_eq!(render_inline(&inline), "#text(style: \"italic\")[italic]");
    }

    #[test]
    fn test_render_code_inline() {
        let inline = Inline::Code(String::from("fn main()"));
        assert_eq!(render_inline(&inline), "=fn main()=");
    }

    #[test]
    fn test_render_code_inline_with_dollar() {
        let inline = Inline::Code(String::from("$"));
        assert_eq!(render_inline(&inline), r"=\$=");
    }

    #[test]
    fn test_render_code_inline_with_backslash() {
        let inline = Inline::Code(String::from("path\\to\\file"));
        assert_eq!(render_inline(&inline), r"=path\\to\\file=");
    }

    #[test]
    fn test_render_code_inline_with_equals() {
        let inline = Inline::Code(String::from("a == b"));
        assert_eq!(render_inline(&inline), r"=a \=\= b=");
    }

    #[test]
    fn test_render_code_inline_with_multiple_special() {
        let inline = Inline::Code(String::from("$HOME == /usr"));
        assert_eq!(render_inline(&inline), r"=\$HOME \=\= /usr=");
    }

    #[test]
    fn test_render_link_inline() {
        let inline = Inline::Link {
            text: String::from("Example"),
            url: String::from("https://example.com"),
        };
        assert_eq!(
            render_inline(&inline),
            "#link(\"https://example.com\")[#text(fill: blue)[Example]]"
        );
    }

    #[test]
    fn test_render_link_inline_text_equals_url() {
        let url = String::from("https://example.com");
        let inline = Inline::Link {
            text: url.clone(),
            url: url.clone(),
        };
        assert_eq!(
            render_inline(&inline),
            "#link(\"https://example.com\")[#text(fill: blue)[(https://example.com)]]"
        );
    }

    #[test]
    fn test_render_link_decodes_html_entities_in_url() {
        let inline = Inline::Link {
            text: String::from("YouTube"),
            url: String::from("https://www.youtube.com/watch?v=abc&amp;list=xyz"),
        };
        assert_eq!(
            render_inline(&inline),
            "#link(\"https://www.youtube.com/watch?v=abc&list=xyz\")[#text(fill: blue)[YouTube]]"
        );
    }

    #[test]
    fn test_render_mark_inline() {
        let inline = Inline::Mark(vec![Inline::Text(String::from("highlighted"))]);
        assert_eq!(render_inline(&inline), "#highlight[highlighted]");
    }

    #[test]
    fn test_render_superscript_inline() {
        let inline = Inline::Superscript(vec![Inline::Text(String::from("2"))]);
        assert_eq!(render_inline(&inline), "#super[2]");
    }

    #[test]
    fn test_render_subscript_inline() {
        let inline = Inline::Subscript(vec![Inline::Text(String::from("2"))]);
        assert_eq!(render_inline(&inline), "#sub[2]");
    }

    #[test]
    fn test_render_footnote_inline() {
        let inline = Inline::Footnote {
            id: Some(String::from("fn1")),
            text: String::from("Note text"),
        };
        let mut rendered = HashSet::new();
        // First occurrence renders with label
        assert_eq!(
            render_inline_with_footnotes(&inline, &mut rendered),
            "#footnote[Note text] <fn-fn1>"
        );
    }

    #[test]
    fn test_render_footnote_dedup() {
        let inline1 = Inline::Footnote {
            id: Some(String::from("fn1")),
            text: String::from("Note text"),
        };
        let inline2 = Inline::Footnote {
            id: Some(String::from("fn1")),
            text: String::from("Note text"),
        };
        let mut rendered = HashSet::new();
        // First occurrence renders with label
        let result1 = render_inline_with_footnotes(&inline1, &mut rendered);
        assert_eq!(result1, "#footnote[Note text] <fn-fn1>");
        // Second occurrence renders as cross-reference
        let result2 = render_inline_with_footnotes(&inline2, &mut rendered);
        assert_eq!(result2, "@fn-fn1");
    }

    #[test]
    fn test_render_linebreak_inline() {
        let inline = Inline::LineBreak;
        assert_eq!(render_inline(&inline), "#linebreak()\n");
    }

    #[test]
    fn test_render_nested_bold_italic() {
        let inline = Inline::Bold(vec![Inline::Italic(vec![Inline::Text(String::from("nested"))])]);
        assert_eq!(render_inline(&inline), "#strong[#text(style: \"italic\")[nested]]");
    }

    #[test]
    fn test_render_multiple_inlines() {
        let inlines = vec![
            Inline::Text(String::from("Hello ")),
            Inline::Bold(vec![Inline::Text(String::from("world"))]),
        ];
        assert_eq!(render_inlines(&inlines), "Hello #strong[world]");
    }

    #[test]
    fn test_render_empty_inlines() {
        let inlines: Vec<Inline> = vec![];
        assert_eq!(render_inlines(&inlines), "");
    }
}
