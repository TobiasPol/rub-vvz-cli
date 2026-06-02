use std::collections::BTreeMap;

pub type Row = BTreeMap<String, String>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub event_guid: Option<String>,
    pub term_guid: Option<String>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub url: String,
    pub value: Option<String>,
    pub guid: Option<String>,
    pub term_guid: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventDetail {
    pub title: String,
    pub url: String,
    pub event_guid: Option<String>,
    pub term_guid: Option<String>,
    pub fields: BTreeMap<String, String>,
    pub sections: BTreeMap<String, Vec<Row>>,
    pub description: String,
}
