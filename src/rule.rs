use crate::document::Document;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub rule_id: String,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
}

pub trait Rule: Send + Sync {
    fn id(&self) -> &str;
    fn check(&self, doc: &Document) -> Vec<Issue>;
}
