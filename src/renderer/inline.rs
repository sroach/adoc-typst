use crate::ir::inline::Inline;

/// Renders a list of inline elements to Typst syntax.
pub fn render_inlines(inlines: &[Inline]) -> String {
    let mut output = String::new();
    for inline in inlines {
        output.push_str(&render_inline(inline));
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

/// Decode common HTML entities that may appear in text content.
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

fn render_inline(inline: &Inline) -> String {
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
            let inner = render_inlines(children);
            // Use #strong[] instead of *...* because Typst delimiters require
            // word boundaries — *ul*timate fails but #strong[ul]timate works.
            format!("#strong[{}]", inner)
        }
        Inline::Italic(children) => {
            let inner = render_inlines(children);
            // Use #text(style: "italic")[] instead of _..._ because Typst delimiters
            // require word boundaries — _war_lock fails but #text(style: "italic")[war]lock works.
            format!("#text(style: \"italic\")[{}]", inner)
        }
        Inline::Code(code) => {
            let escaped = escape_typst_code(code);
            format!("={}=", escaped)
        }
        Inline::Link { text, url } => {
            // Escape @ in link text to prevent Typst label reference
            let escaped_text = text.replace('@', "\\@");
            format!("[{}]({})", escaped_text, url)
        }
        Inline::Mark(children) => {
            let inner = render_inlines(children);
            format!("#highlight[{}]", inner)
        }
        Inline::Superscript(children) => {
            let inner = render_inlines(children);
            format!("#super[{}]", inner)
        }
        Inline::Subscript(children) => {
            let inner = render_inlines(children);
            format!("#sub[{}]", inner)
        }
        Inline::Footnote { text, .. } => {
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
        assert_eq!(render_inline(&inline), "[Example](https://example.com)");
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
        assert_eq!(render_inline(&inline), "#footnote[Note text]");
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
