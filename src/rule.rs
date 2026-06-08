use crate::document::Document;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

pub type ByteRange = std::ops::Range<usize>;

#[derive(Debug, Clone)]
pub struct Fix {
    pub range: ByteRange,
    pub replacement: String,
}

#[derive(Debug, Clone)]
pub struct Issue {
    pub rule_id: String,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub fix: Option<Fix>,
}

impl Issue {
    pub fn new(
        rule_id: impl Into<String>,
        message: impl Into<String>,
        line: usize,
        column: usize,
        severity: Severity,
    ) -> Self {
        Issue {
            rule_id: rule_id.into(),
            message: message.into(),
            line,
            column,
            severity,
            fix: None,
        }
    }

    pub fn with_fix(mut self, fix: Fix) -> Self {
        self.fix = Some(fix);
        self
    }
}

pub trait Rule: Send + Sync {
    fn id(&self) -> &str;
    fn check(&self, doc: &Document) -> Vec<Issue>;
}
