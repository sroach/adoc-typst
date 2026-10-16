use asciidoc_parser::blocks::{Block as AstBlock, Descend, FindBlocks, IsBlock, ListType};
use asciidoc_parser::inlines::InlineNode;
use asciidoc_parser::Document as AstDocument;

use crate::ir::blocks::{AdmonitionLevel, Block, ListItem, NestedSection};
use crate::ir::document::{Document, Section};
use crate::ir::inline::Inline;

/// Converts the asciidoc-parser Document to our intermediate representation.
pub fn convert(ast: &AstDocument<'_>) -> Document {
    let title = ast.doctitle().map(|s| s.to_string());

    let mut sections: Vec<Section> = Vec::new();

    for block in ast.traverse_blocks(|block| match block {
        AstBlock::Section(_) => Descend::Prune,
        AstBlock::Preamble(_) => Descend::Prune,
        _ => Descend::Accept,
    }) {
        match block {
            AstBlock::Section(section_block) => {
                let level = section_block.level() as u8;
                let title = section_block.section_title().to_string();
                let content: Vec<Block> = section_block
                    .child_blocks()
                    .filter_map(|b| convert_block_with_section(&b))
                    .collect();
                sections.push(Section {
                    level,
                    title,
                    content,
                });
            }
            AstBlock::Preamble(preamble) => {
                let content: Vec<Block> = preamble
                    .child_blocks()
                    .filter_map(|b| convert_block(&b))
                    .collect();
                if !content.is_empty() {
                    if let Some(first) = sections.first_mut() {
                        first.content.extend(content);
                    } else {
                        sections.push(Section {
                            level: 1,
                            title: String::new(),
                            content,
                        });
                    }
                }
            }
            _ => {
                if let Some(converted) = convert_block(&block) {
                    if let Some(first) = sections.first_mut() {
                        first.content.push(converted);
                    } else {
                        sections.push(Section {
                            level: 1,
                            title: String::new(),
                            content: vec![converted],
                        });
                    }
                }
            }
        }
    }

    let author = ast.authors().first().map(|a| a.name().to_string());

    let date = match ast.attribute_value("revdate") {
        asciidoc_parser::document::InterpretedValue::Value(v) => Some(v),
        _ => match ast.attribute_value("date") {
            asciidoc_parser::document::InterpretedValue::Value(v) => Some(v),
            _ => None,
        },
    };

    // Extract keywords from the "keywords" attribute (comma-separated)
    let keywords: Vec<String> = match ast.attribute_value("keywords") {
        asciidoc_parser::document::InterpretedValue::Value(v) => v.split(',')
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect(),
        _ => Vec::new(),
    };

    Document {
        title,
        author,
        date,
        keywords,
        sections,
    }
}

fn convert_block(block: &AstBlock<'_>) -> Option<Block> {
    match block {
        AstBlock::Simple(simple) => {
            let inlines: Vec<Inline> = simple
                .inlines()
                .map(|nodes| nodes.iter().flat_map(|n| convert_inline(n)).collect())
                .unwrap_or_default();
            if inlines.is_empty() {
                None
            } else {
                Some(Block::Paragraph(inlines))
            }
        }
        AstBlock::List(list_block) => {
            let items: Vec<ListItem> = list_block
                .child_blocks()
                .filter_map(|b| {
                    if let AstBlock::ListItem(li) = b {
                        convert_list_item(&li)
                    } else {
                        None
                    }
                })
                .collect();
            if items.is_empty() {
                return None;
            }
            match list_block.type_() {
                ListType::Ordered => Some(Block::OrderedList(items)),
                ListType::Unordered | ListType::Description => Some(Block::UnorderedList(items)),
                ListType::Callout => None,
            }
        }
        AstBlock::RawDelimited(raw) => {
            let content = raw.content();
            let body = content.original().data().to_string();

            // Check if this is a [docops,type] custom macro
            // The attrlist format is: pos 1 = format (e.g. "docops"), pos 2 = type
            if let Some(attrlist) = raw.attrlist() {
                let format_attr = attrlist.nth_attribute(1).map(|a| a.value().to_string());
                let viz_type = attrlist.nth_attribute(2).map(|a| a.value().to_string());

                if format_attr.as_deref() == Some("docops") {
                    let type_name = viz_type.unwrap_or_default();
                    return Some(Block::RemoteMacro {
                        viz_type: type_name,
                        body: body.trim().to_string(),
                    });
                }
            }

            let lines: Vec<String> = body.lines().map(|s| s.to_string()).collect();
            let language = raw
                .attrlist()
                .and_then(|al| al.nth_attribute(2))
                .map(|attr| attr.value().to_string())
                .unwrap_or_default();
            let title = block.title().map(|s| s.to_string());
            Some(Block::CodeBlock {
                language,
                title,
                lines,
            })
        }
        AstBlock::CompoundDelimited(cd) => {
            let style = cd.resolved_style().unwrap_or("");
            let mut content: Vec<Block> = cd
                .child_blocks()
                .filter_map(|b| convert_block(&b))
                .collect();
            if content.is_empty() {
                None
            } else if style == "quote" || style == "verse" {
                let attribution = cd
                    .attrlist()
                    .and_then(|al| al.named_or_positional_attribute("attribution", 2))
                    .map(|attr| attr.value().to_string());
                let citation = cd
                    .attrlist()
                    .and_then(|al| al.named_or_positional_attribute("citetitle", 3))
                    .map(|attr| attr.value().to_string());
                Some(Block::QuoteBlock { content, attribution, citation })
            } else {
                if content.len() == 1 {
                    Some(content.pop().unwrap())
                } else {
                    Some(Block::Paragraph(Vec::new()))
                }
            }
        }
        AstBlock::Admonition(admonition) => {
            let level = match admonition.name() {
                "note" => AdmonitionLevel::Note,
                "tip" => AdmonitionLevel::Tip,
                "important" => AdmonitionLevel::Important,
                "warning" => AdmonitionLevel::Warning,
                "caution" => AdmonitionLevel::Caution,
                _ => AdmonitionLevel::Note,
            };
            let mut content: Vec<Block> = admonition
                .child_blocks()
                .filter_map(|b| convert_block(&b))
                .collect();
            if content.is_empty() {
                if let Some(simple_content) = admonition.content() {
                    let nodes = simple_content.inlines();
                    let inlines: Vec<Inline> =
                        nodes.iter().flat_map(|n| convert_inline(n)).collect();
                    if !inlines.is_empty() {
                        content.push(Block::Paragraph(inlines));
                    }
                }
            }
            Some(Block::Admonition { level, content })
        }
        AstBlock::Quote(quote_block) => {
            let content: Vec<Block> = quote_block
                .child_blocks()
                .filter_map(|b| convert_block(&b))
                .collect();
            let attribution = quote_block.attribution().map(|s| s.to_string());
            let citation = quote_block.citetitle().map(|s| s.to_string());
            Some(Block::QuoteBlock { content, attribution, citation })
        }
        AstBlock::Table(table_block) => {
            let convert_cell = |cell: &asciidoc_parser::blocks::TableCell<'_>| -> Vec<Inline> {
                match cell.content() {
                    asciidoc_parser::blocks::TableCellContent::Simple(content) => {
                        let nodes = content.inlines();
                        nodes.iter().flat_map(|n| convert_inline(n)).collect()
                    }
                    asciidoc_parser::blocks::TableCellContent::AsciiDoc(_) => {
                        Vec::new()
                    }
                }
            };
            let headers: Vec<Vec<Inline>> = table_block
                .header_row()
                .map(|row| {
                    row.cells().iter().map(|cell| convert_cell(cell)).collect()
                })
                .unwrap_or_default();
            let rows: Vec<Vec<Vec<Inline>>> = table_block
                .body_rows()
                .iter()
                .map(|row| {
                    row.cells().iter().map(|cell| convert_cell(cell)).collect()
                })
                .collect();
            Some(Block::Table { headers, rows })
        }
        AstBlock::Media(media) => {
            let path = media.resolved_target().to_string();
            let alt = media
                .macro_attrlist()
                .nth_attribute(1)
                .map(|attr| attr.value().to_string())
                .unwrap_or_default();
            Some(Block::Image { path, alt })
        }
        AstBlock::Section(_) | AstBlock::Preamble(_) | AstBlock::ListItem(_) => None,
        AstBlock::Break(_) | AstBlock::Toc(_) | AstBlock::DocumentAttribute(_) => None,
        _ => None,
    }
}

/// Like `convert_block`, but also handles nested `Section` blocks by converting
/// them into `Block::Section(NestedSection)` so subsections are preserved.
fn convert_block_with_section(block: &AstBlock<'_>) -> Option<Block> {
    if let AstBlock::Section(section_block) = block {
        let level = section_block.level() as u8;
        let title = section_block.section_title().to_string();
        let content: Vec<Block> = section_block
            .child_blocks()
            .filter_map(|b| convert_block_with_section(&b))
            .collect();
        return Some(Block::Section(NestedSection { level, title, content }));
    }
    convert_block(block)
}

fn convert_list_item(item: &asciidoc_parser::blocks::ListItem<'_>) -> Option<ListItem> {
    let mut inlines: Vec<Inline> = Vec::new();
    let mut children: Vec<ListItem> = Vec::new();

    for block in item.child_blocks() {
        match block {
            AstBlock::Simple(simple) => {
                if inlines.is_empty() {
                    if let Some(nodes) = simple.inlines() {
                        for node in nodes {
                            inlines.extend(convert_inline(node));
                        }
                    }
                } else {
                    if let Some(nodes) = simple.inlines() {
                        let child_inlines: Vec<Inline> =
                            nodes.iter().flat_map(|n| convert_inline(n)).collect();
                        if !child_inlines.is_empty() {
                            children.push(ListItem {
                                content: child_inlines,
                                children: Vec::new(),
                            });
                        }
                    }
                }
            }
            AstBlock::List(list_block) => {
                let sub_items: Vec<ListItem> = list_block
                    .child_blocks()
                    .filter_map(|b| {
                        if let AstBlock::ListItem(li) = b {
                            convert_list_item(&li)
                        } else {
                            None
                        }
                    })
                    .collect();
                for sub in sub_items {
                    children.push(sub);
                }
            }
            _ => {}
        }
    }

    Some(ListItem { content: inlines, children })
}

fn convert_inline(node: &InlineNode<'_>) -> Vec<Inline> {
    match node {
        InlineNode::Text { value, .. } => {
            if value.is_empty() {
                Vec::new()
            } else {
                vec![Inline::Text(value.to_string())]
            }
        }
        InlineNode::CharRef { value, .. } => {
            let text = match value {
                asciidoc_parser::inlines::CharRef::Special(c) => c.to_string(),
                asciidoc_parser::inlines::CharRef::Replacement(s) => s.to_string(),
                asciidoc_parser::inlines::CharRef::Entity(e) => e.to_string(),
            };
            vec![Inline::Text(text)]
        }
        InlineNode::Raw { value, .. } => {
            if value.is_empty() {
                Vec::new()
            } else {
                vec![Inline::Text(value.to_string())]
            }
        }
        InlineNode::Styled(styled) => {
            let children: Vec<Inline> = styled
                .children
                .iter()
                .flat_map(|n| convert_inline(n))
                .collect();
            if children.is_empty() {
                return Vec::new();
            }
            match styled.variant {
                asciidoc_parser::inlines::StyleVariant::Strong => vec![Inline::Bold(children)],
                asciidoc_parser::inlines::StyleVariant::Emphasis => {
                    vec![Inline::Italic(children)]
                }
                asciidoc_parser::inlines::StyleVariant::Code => {
                    let text: String = children
                        .into_iter()
                        .filter_map(|i| match i {
                            Inline::Text(t) => Some(t),
                            _ => None,
                        })
                        .collect();
                    vec![Inline::Code(text)]
                }
                asciidoc_parser::inlines::StyleVariant::Mark => vec![Inline::Mark(children)],
                asciidoc_parser::inlines::StyleVariant::Superscript => {
                    vec![Inline::Superscript(children)]
                }
                asciidoc_parser::inlines::StyleVariant::Subscript => {
                    vec![Inline::Subscript(children)]
                }
                asciidoc_parser::inlines::StyleVariant::DoubleQuote |
                asciidoc_parser::inlines::StyleVariant::SingleQuote => children,
                asciidoc_parser::inlines::StyleVariant::Unquoted => children,
            }
        }
        InlineNode::Ref(ref_node) => {
            let text: String = ref_node
                .children
                .iter()
                .flat_map(|n| convert_inline(n))
                .filter_map(|i| match i {
                    Inline::Text(t) => Some(t),
                    _ => None,
                })
                .collect();
            let text = if text.is_empty() {
                ref_node.target.to_string()
            } else {
                text
            };
            let url = ref_node.target.to_string();
            vec![Inline::Link { text, url }]
        }
        InlineNode::Image(img) => {
            let target = img.target.to_string();
            vec![Inline::Link {
                text: target.clone(),
                url: target,
            }]
        }
        InlineNode::Footnote(footnote) => {
            let id = footnote.id.as_ref().map(|s| s.to_string());
            let text: String = footnote
                .children
                .iter()
                .flat_map(|n| convert_inline(n))
                .filter_map(|i| match i {
                    Inline::Text(t) => Some(t),
                    _ => None,
                })
                .collect();
            let text = if text.is_empty() {
                footnote.number.as_ref().map(|s| s.to_string()).unwrap_or_default()
            } else {
                text
            };
            vec![Inline::Footnote { id, text }]
        }
        InlineNode::LineBreak { .. } => {
            vec![Inline::LineBreak]
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    fn convert_adoc(adoc: &str) -> Document {
        let ast = parser::parse(adoc);
        convert(&ast)
    }

    #[test]
    fn test_convert_empty_document() {
        let doc = convert_adoc("");
        assert!(doc.title.is_none());
        assert!(doc.sections.is_empty());
    }

    #[test]
    fn test_convert_document_with_title() {
        let doc = convert_adoc("= My Title");
        assert_eq!(doc.title, Some(String::from("My Title")));
    }

    #[test]
    fn test_convert_document_with_author() {
        let doc = convert_adoc("= Title\nAuthor Name");
        assert_eq!(doc.author, Some(String::from("Author Name")));
    }

    #[test]
    fn test_convert_document_with_date() {
        let doc = convert_adoc("= Title\nAuthor\n2024-01-01");
        assert!(doc.date.is_some());
    }

    #[test]
    fn test_convert_paragraph() {
        let doc = convert_adoc("Hello world.");
        assert!(!doc.sections.is_empty());
        let section = &doc.sections[0];
        assert!(!section.content.is_empty());
        assert!(matches!(section.content[0], Block::Paragraph(_)));
    }

    #[test]
    fn test_convert_section() {
        let doc = convert_adoc("== Section Title\nContent.");
        assert_eq!(doc.sections.len(), 1);
        assert_eq!(doc.sections[0].title, "Section Title");
        assert_eq!(doc.sections[0].level, 1);
    }

    #[test]
    fn test_convert_subsection() {
        let doc = convert_adoc("=== Subsection");
        assert_eq!(doc.sections[0].level, 2);
    }

    #[test]
    fn test_convert_unordered_list() {
        let doc = convert_adoc("- Item one\n- Item two");
        let section = &doc.sections[0];
        assert!(matches!(section.content[0], Block::UnorderedList(_)));
    }

    #[test]
    fn test_convert_ordered_list() {
        let doc = convert_adoc(". Item one\n. Item two");
        let section = &doc.sections[0];
        assert!(matches!(section.content[0], Block::OrderedList(_)));
    }

    #[test]
    fn test_convert_source_block() {
        let doc = convert_adoc("[source,rust]\n----\nfn main() {}\n----");
        let section = &doc.sections[0];
        if let Block::CodeBlock { language, .. } = &section.content[0] {
            assert_eq!(language, "rust");
        } else {
            panic!("Expected CodeBlock");
        }
    }

    #[test]
    fn test_convert_note_admonition() {
        let doc = convert_adoc("NOTE: This is a note.");
        let section = &doc.sections[0];
        if let Block::Admonition { level, .. } = &section.content[0] {
            assert!(matches!(level, AdmonitionLevel::Note));
        } else {
            panic!("Expected Admonition");
        }
    }

    #[test]
    fn test_convert_bold_inline() {
        let doc = convert_adoc("*bold*");
        let section = &doc.sections[0];
        if let Block::Paragraph(inlines) = &section.content[0] {
            assert!(matches!(&inlines[0], Inline::Bold(_)));
        } else {
            panic!("Expected Paragraph");
        }
    }

    #[test]
    fn test_convert_link() {
        let doc = convert_adoc("https://example.com[Example]");
        let section = &doc.sections[0];
        if let Block::Paragraph(inlines) = &section.content[0] {
            assert!(matches!(&inlines[0], Inline::Link { .. }));
        } else {
            panic!("Expected Paragraph");
        }
    }

    #[test]
    fn test_convert_image() {
        let doc = convert_adoc("image::diagram.png[Diagram]");
        let section = &doc.sections[0];
        assert!(matches!(&section.content[0], Block::Image { .. }));
    }

    #[test]
    fn test_convert_docops_macro() {
        let doc = convert_adoc("[docops,recipe]\n----\ntest body\n----");
        let section = &doc.sections[0];
        assert!(matches!(&section.content[0], Block::RemoteMacro { .. }));
        if let Block::RemoteMacro { viz_type, body } = &section.content[0] {
            assert_eq!(viz_type, "recipe");
            assert_eq!(body, "test body");
        }
    }

    #[test]
    fn test_convert_keywords() {
        let doc = convert_adoc(":keywords: rust, asciidoc, typst\n= Title");
        assert_eq!(doc.keywords, vec!["rust", "asciidoc", "typst"]);
    }

    #[test]
    fn test_convert_no_keywords() {
        let doc = convert_adoc("= Title");
        assert!(doc.keywords.is_empty());
    }
}
