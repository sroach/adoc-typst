use super::blocks::Block;

/// Intermediate representation of the document metadata and structure.
#[derive(Debug, Clone)]
pub struct Document {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub authors: Vec<String>,
    pub date: Option<String>,
    pub revision: Option<String>,
    pub keywords: Vec<String>,
    pub organization: Option<String>,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub level: u8,
    pub title: String,
    pub content: Vec<Block>,
}
