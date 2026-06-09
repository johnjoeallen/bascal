use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePos {
    pub filename: String,
    pub line: usize,
    pub column: usize,
}

impl SourcePos {
    pub fn new(filename: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            filename: filename.into(),
            line,
            column,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub pos: SourcePos,
}

impl Diagnostic {
    pub fn error(pos: SourcePos, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            message: message.into(),
            pos,
        }
    }

    pub fn warning(pos: SourcePos, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            message: message.into(),
            pos,
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(
            f,
            "{severity}: {}\n  --> {}:{}:{}",
            self.message, self.pos.filename, self.pos.line, self.pos.column
        )
    }
}
