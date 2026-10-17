use super::inline::Inline;

/// A nested section (subsection) within a parent section.
#[derive(Debug, Clone)]
pub struct NestedSection {
    pub level: u8,
    pub title: String,
    pub content: Vec<Block>,
}

/// Block-level elements in the intermediate representation.
#[derive(Debug, Clone)]
pub enum Block {
    Paragraph(Vec<Inline>),
    OrderedList(Vec<ListItem>),
    UnorderedList(Vec<ListItem>),
    CodeBlock { language: String, title: Option<String>, lines: Vec<String> },
    QuoteBlock { content: Vec<Block>, attribution: Option<String>, citation: Option<String> },
    Admonition { level: AdmonitionLevel, content: Vec<Block> },
    Table {
        headers: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
        /// Relative column widths from the `cols=` attribute (e.g., `cols="3,2,2,4"`).
        /// Empty if no column specs were provided.
        col_widths: Vec<usize>,
    },
    Image { path: String, alt: String },
    /// A custom [docops,type] macro that will be resolved to an SVG image.
    RemoteMacro { viz_type: String, body: String },
    /// A nested subsection within a parent section.
    Section(NestedSection),
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub content: Vec<Inline>,
    pub children: Vec<ListItem>,
    /// Blocks attached to this list item via the `+` continuation syntax
    /// (e.g., an admonition attached to a list item).
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub enum AdmonitionLevel {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}
