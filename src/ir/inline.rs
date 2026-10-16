/// Inline elements in the intermediate representation.
#[derive(Debug, Clone)]
pub enum Inline {
    Text(String),
    Bold(Vec<Inline>),
    Italic(Vec<Inline>),
    Code(String),
    Link { text: String, url: String },
    /// Highlighted/marked text (`#marked#`)
    Mark(Vec<Inline>),
    /// Superscript text (`^sup^`)
    Superscript(Vec<Inline>),
    /// Subscript text (`~sub~`)
    Subscript(Vec<Inline>),
    /// Footnote reference
    Footnote { id: Option<String>, text: String },
    /// Line break (trailing `+`)
    LineBreak,
}
