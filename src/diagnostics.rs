use std::fmt;
use std::path::Path;

/// A `SourcePos::filename` is always an absolute, canonicalized path (see
/// `lib.rs`'s own `normalize_path`) -- fine for a compiler diagnostic the
/// developer reads on the machine that ran `bcc`, but never appropriate to
/// bake into a *compiled program's own runtime output*: the machine
/// running that program may not be the machine that built it, and even
/// when it is, exposing the build tree's absolute layout serves no
/// purpose the program's own user needs. Used wherever a raise site's
/// filename becomes part of generated BASIC/C output itself (`catch`'s
/// optional `source$` binding), never for an ordinary compiler
/// diagnostic. Falls back to the absolute path unchanged if it isn't
/// actually under the current working directory, or the working
/// directory can't be read at all.
pub fn display_source_filename(filename: &str) -> String {
    std::env::current_dir()
        .ok()
        .and_then(|cwd| {
            Path::new(filename)
                .strip_prefix(&cwd)
                .ok()
                .map(|rel| rel.display().to_string())
        })
        .unwrap_or_else(|| filename.to_string())
}

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
