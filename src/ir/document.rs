use super::blocks::Block;

/// Intermediate representation of the document metadata and structure.
#[derive(Debug, Clone)]
pub struct Document {
    pub title: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub keywords: Vec<String>,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub level: u8,
    pub title: String,
    pub content: Vec<Block>,
}
