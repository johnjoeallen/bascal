//! `bcc --format-check` / `bcc --format` -- BASCAL's own source formatter.
//!
//! v1 is deliberately narrow: it only fixes *indentation*, leaving every
//! other stylistic choice (keyword casing, spacing around operators,
//! blank-line placement) exactly as the author wrote it. This is a much
//! smaller, safer problem than full token-level pretty-printing (no risk
//! of subtly reflowing an expression wrong), and it's the dimension real
//! BASCAL source actually drifts on in practice -- copy-pasted blocks,
//! hand-edited `if`/`for` bodies, and so on.
//!
//! Indentation is recomputed from the real token stream (`Lexer`, not a
//! regex), so keywords inside a string or comment never confuse the
//! block-nesting tracker. Everything else about each line -- keyword
//! casing, inter-token spacing, trailing comments -- is copied through
//! byte-for-byte after its own leading whitespace is replaced.
//!
//! Multi-line `/* ... */` comments are a deliberate exception: only their
//! opening line is reindented. Interior lines are often hand-aligned
//! (e.g. a `* ` prefix per line, or ASCII art), and this formatter has no
//! opinion on that -- reindenting them blindly would be more likely to
//! break a deliberate layout than fix an accidental one.
//!
//! A bare `label:` (nothing else on its own line) indents the `data`
//! lines that immediately follow it one level deeper, matching its usual
//! role as a `restore` target for a data table -- grouping what belongs
//! to it, for the same reason a `for`/`if` body is indented. That block
//! has no closing keyword of its own: it ends at the first line that
//! isn't a `data` statement or a comment about the table, or at a blank
//! line (which is how a table's own trailing comment ends and something
//! unrelated -- often the next function's own doc comment -- begins,
//! even when that next thing is itself just a comment). A label
//! immediately followed by a real statement on the same line (e.g.
//! `L6483: FOR Z0=0 TO Z9`) is not a bare label and never triggers this
//! -- its statement is classified normally instead.

use crate::lexer::{Lexer, Token, TokenKind};
use std::collections::BTreeMap;

const INDENT_UNIT: &str = "    ";

/// One nested block currently open, and what closes it. `SelectCaseHeader`
/// is the odd one out: `select case` opens it, but the first `case` line
/// replaces it with a `CaseBody` rather than popping it -- see
/// `reindent`'s own `case`/`select` handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Frame {
    If,
    For,
    While,
    Do,
    SelectCaseHeader,
    Case,
    Try,
    Fn,
    Record,
    /// A bare `label:` immediately followed by one or more `data`
    /// statements (its usual role: a `restore` target for a data
    /// table). Pushed by the label itself; popped automatically by the
    /// first line afterward that isn't a `data` statement or a comment,
    /// so the block needs no closing keyword of its own.
    LabelData,
    Method,
}

/// Reindents `source` (BASCAL text, `filename` used only for diagnostic
/// positions the lexer attaches to tokens) and returns the result. Never
/// fails: an unrecognized token sequence just falls back to leaving that
/// line's own indent alone, since a formatter that refuses to run on
/// slightly-unusual-but-valid source is worse than one that quietly
/// leaves a corner case untouched.
pub fn reindent(filename: &str, source: &str) -> String {
    let tokens = Lexer::new(filename, source).lex();
    let lines: Vec<&str> = source.lines().collect();

    let mut by_line: BTreeMap<usize, Vec<&Token>> = BTreeMap::new();
    for t in &tokens {
        by_line.entry(t.pos.line).or_default().push(t);
    }

    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut stack: Vec<Frame> = Vec::new();
    let mut verbatim_until = 0usize; // lines <= this are inside a multi-line block comment's interior

    for (i, raw_line) in lines.iter().enumerate() {
        let lineno = i + 1;
        if lineno <= verbatim_until {
            out.push(raw_line.to_string());
            continue;
        }
        if raw_line.trim().is_empty() {
            // A blank line is the natural way a data table's own trailing
            // comment ends and something unrelated (often the next
            // function's own doc comment) begins -- unlike a comment
            // line right in the middle of the table, it does close a
            // still-open `LabelData` block.
            if matches!(stack.last(), Some(Frame::LabelData)) {
                stack.pop();
            }
            out.push(String::new());
            continue;
        }

        let toks: Vec<&Token> = by_line.get(&lineno).cloned().unwrap_or_default();

        // A block comment starting here may span further lines; leave its
        // interior (and closing `*/` line) untouched either way.
        if let Some(first) = toks.first() {
            if let TokenKind::BlockComment(text) = &first.kind {
                let span = text.matches('\n').count();
                if span > 0 {
                    verbatim_until = lineno + span;
                }
            }
        }

        // A leading `label:` (bare, or immediately followed by a real
        // statement on the same line, like `L6483: FOR Z0=0 TO Z9`)
        // never itself opens/closes a block or gets an indent bump --
        // but a statement riding along after the colon still needs
        // classifying normally, so strip just the label prefix off
        // before looking at what follows.
        let has_leading_label = toks.len() >= 2
            && matches!(toks[0].kind, TokenKind::Ident(_))
            && matches!(toks[1].kind, TokenKind::Colon);
        let effective: &[&Token] = if has_leading_label {
            &toks[2..]
        } else {
            &toks[..]
        };

        let words: Vec<String> = effective
            .iter()
            .filter_map(|t| match &t.kind {
                TokenKind::Ident(s) => Some(s.to_ascii_lowercase()),
                _ => None,
            })
            .collect();

        let first_word = words.first().map(String::as_str).unwrap_or("");
        // `effective` still holds the line's own trailing `Newline`
        // token even when nothing else follows the label, so "nothing
        // real after the colon" has to be judged by `words` (Ident
        // tokens only), not by whether the slice itself is empty.
        let is_bare_label = has_leading_label && words.is_empty();
        let is_comment_only = words.is_empty()
            && toks
                .iter()
                .any(|t| matches!(t.kind, TokenKind::Comment(_) | TokenKind::BlockComment(_)));

        // `case`/`select case` need to know whether the LAST significant
        // token on the line is `then` (a block `if`/`elseif`) so a
        // single-line `if cond then stmt` doesn't open a level nobody
        // closes.
        let ends_with_then = effective
            .iter()
            .rev()
            .find(|t| {
                !matches!(
                    t.kind,
                    TokenKind::Comment(_) | TokenKind::BlockComment(_) | TokenKind::Newline
                )
            })
            .map(|t| matches!(&t.kind, TokenKind::Ident(s) if s.eq_ignore_ascii_case("then")))
            .unwrap_or(false);

        let mut pop_count = 0usize;
        let mut pending_push: Option<Frame> = None;

        // A `LabelData` block (a bare `label:` followed by `data` lines)
        // has no closing keyword of its own -- it just ends the moment
        // something other than a `data` statement or a comment shows up,
        // including a fresh label starting a new block right after it.
        if matches!(stack.last(), Some(Frame::LabelData))
            && first_word != "data"
            && !is_comment_only
        {
            pop_count += 1;
        }

        if !effective.is_empty() {
            match first_word {
                "end" => {
                    // `end` alone (program terminator) doesn't close a
                    // block; `end if`/`end for`/... does. `end select` is
                    // the one closer that must pop twice: the last
                    // `case`'s own body frame (if any case ran at all),
                    // then the `select case` header frame underneath it.
                    if words.get(1).map(String::as_str) == Some("select") {
                        if matches!(stack.last(), Some(Frame::Case)) {
                            pop_count += 1;
                        }
                        if matches!(
                            stack.get(stack.len().wrapping_sub(1 + pop_count)),
                            Some(Frame::SelectCaseHeader)
                        ) {
                            pop_count += 1;
                        }
                    } else if words.len() > 1 {
                        pop_count += 1;
                    }
                }
                "next" | "wend" | "loop" => pop_count += 1,
                "elseif" | "else" => {
                    if stack.last() == Some(&Frame::If) {
                        pop_count += 1;
                        pending_push = Some(Frame::If);
                    }
                }
                "catch" | "finally" => {
                    if stack.last() == Some(&Frame::Try) {
                        pop_count += 1;
                        pending_push = Some(Frame::Try);
                    }
                }
                "case" => {
                    // The first `case` after `select case` sits right on
                    // top of the still-open `SelectCaseHeader` frame --
                    // that frame itself accounts for this line's own
                    // depth, so it's left in place, not popped. Every
                    // later `case` instead sits on top of the *previous*
                    // case's own `Case` body frame, which does need
                    // popping first so this line re-aligns with its
                    // siblings rather than nesting inside the last one.
                    if matches!(stack.last(), Some(Frame::Case)) {
                        pop_count += 1;
                    }
                    pending_push = Some(Frame::Case);
                }
                "if" => {
                    if ends_with_then {
                        pending_push = Some(Frame::If);
                    }
                }
                "for" => pending_push = Some(Frame::For),
                "while" => pending_push = Some(Frame::While),
                "do" => pending_push = Some(Frame::Do),
                "select" => {
                    if words.get(1).map(String::as_str) == Some("case") {
                        pending_push = Some(Frame::SelectCaseHeader);
                    }
                }
                "try" => pending_push = Some(Frame::Try),
                "function" | "procedure" => pending_push = Some(Frame::Fn),
                "record" => pending_push = Some(Frame::Record),
                "method" => pending_push = Some(Frame::Method),
                _ => {}
            }
        }

        for _ in 0..pop_count {
            if stack.is_empty() {
                break;
            }
            stack.pop();
        }

        let level = stack.len();
        out.push(format!(
            "{}{}",
            INDENT_UNIT.repeat(level),
            raw_line.trim_start()
        ));

        if let Some(frame) = pending_push {
            stack.push(frame);
        } else if is_bare_label {
            stack.push(Frame::LabelData);
        }
    }

    let mut result = out.join("\n");
    result.push('\n');
    result
}

/// A single line that differs between the original and the reindented
/// form -- what `--format-check` reports.
pub struct Diff {
    pub line: usize,
    pub before: String,
    pub after: String,
}

/// Compares `source` against its own reindented form and returns every
/// differing line, 1-indexed. Empty means the file is already compliant.
pub fn check(filename: &str, source: &str) -> Vec<Diff> {
    let formatted = reindent(filename, source);
    let before_lines: Vec<&str> = source.lines().collect();
    let after_lines: Vec<&str> = formatted.lines().collect();
    let mut diffs = Vec::new();
    for i in 0..before_lines.len().max(after_lines.len()) {
        let before = before_lines.get(i).copied().unwrap_or("");
        let after = after_lines.get(i).copied().unwrap_or("");
        if before != after {
            diffs.push(Diff {
                line: i + 1,
                before: before.to_string(),
                after: after.to_string(),
            });
        }
    }
    diffs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_idempotent(source: &str) {
        let once = reindent("test.bcl", source);
        let twice = reindent("test.bcl", &once);
        assert_eq!(once, twice, "reindent should be idempotent");
    }

    #[test]
    fn reindents_a_simple_if_for_function() {
        let source = "\
program p
function f%()
if x% = 1 then
print \"one\"
elseif x% = 2 then
print \"two\"
else
print \"other\"
end if
for i% = 1 to 3
print i%
next
return 0
end function
";
        let expected = "\
program p
function f%()
    if x% = 1 then
        print \"one\"
    elseif x% = 2 then
        print \"two\"
    else
        print \"other\"
    end if
    for i% = 1 to 3
        print i%
    next
    return 0
end function
";
        assert_eq!(reindent("test.bcl", source), expected);
        assert_idempotent(source);
    }

    #[test]
    fn single_line_if_does_not_open_a_level() {
        let source = "\
function f%()
if x% = 1 then return 0
print \"after\"
end function
";
        let expected = "\
function f%()
    if x% = 1 then return 0
    print \"after\"
end function
";
        assert_eq!(reindent("test.bcl", source), expected);
    }

    #[test]
    fn select_case_double_indents_case_bodies() {
        let source = "\
function f%()
select case x%
case 1
return 1
case 2
return 2
case else
return 0
end select
end function
";
        let expected = "\
function f%()
    select case x%
        case 1
            return 1
        case 2
            return 2
        case else
            return 0
    end select
end function
";
        assert_eq!(reindent("test.bcl", source), expected);
        assert_idempotent(source);
    }

    #[test]
    fn multi_line_block_comment_interior_is_left_alone() {
        let source = "\
function f%()
/* a comment
   * with its own alignment
   */
return 0
end function
";
        let expected = "\
function f%()
    /* a comment
   * with its own alignment
   */
    return 0
end function
";
        assert_eq!(reindent("test.bcl", source), expected);
    }

    #[test]
    fn bare_label_indents_its_own_data_lines_but_not_what_follows() {
        // A bare `label:` (its usual role: a `restore` target for a data
        // table) indents consecutive `data` lines -- and a comment among
        // them -- one level deeper, for the same reason a `for`/`if`
        // body does: grouping what belongs to it. The first real,
        // non-`data` statement after it (here `print`) closes that
        // implicit block with no keyword of its own.
        let source = "\
function f%()
myLabel:
' a comment about the table
data 1,2,3
data 4,5,6
print \"after\"
end function
";
        let expected = "\
function f%()
    myLabel:
        ' a comment about the table
        data 1,2,3
        data 4,5,6
    print \"after\"
end function
";
        assert_eq!(reindent("test.bcl", source), expected);
        assert_idempotent(source);
    }

    #[test]
    fn back_to_back_data_labels_each_get_their_own_indent() {
        let source = "\
function f%()
firstTable:
data 1,2,3
secondTable:
data 4,5,6
end function
";
        let expected = "\
function f%()
    firstTable:
        data 1,2,3
    secondTable:
        data 4,5,6
end function
";
        assert_eq!(reindent("test.bcl", source), expected);
        assert_idempotent(source);
    }

    #[test]
    fn blank_line_closes_a_data_label_even_before_an_unrelated_comment() {
        // A comment right in the middle of a table (no blank line
        // separating it) stays part of the table (see the test above).
        // But a *blank* line is how a table's own trailing comment ends
        // and something unrelated -- often the next function's own doc
        // comment -- begins, so it must close the block even though the
        // very next line is itself just a comment.
        let source = "\
function f%()
myTable:
data 1,2,3

/*
 * Unrelated comment, not about myTable.
 */
print \"after\"
end function
";
        let expected = "\
function f%()
    myTable:
        data 1,2,3

    /*
 * Unrelated comment, not about myTable.
 */
    print \"after\"
end function
";
        assert_eq!(reindent("test.bcl", source), expected);
        assert_idempotent(source);
    }

    #[test]
    fn label_with_a_statement_on_the_same_line_does_not_indent_what_follows() {
        // Only a *bare* label (nothing after the colon on its own line)
        // opens a data-block indent. `label: statement` already had its
        // statement classified normally (see the test above this one in
        // the file); it must not also behave like a bare label.
        let source = "\
function f%()
loopStart: for i% = 1 to 3
print i%
end for
end function
";
        let expected = "\
function f%()
    loopStart: for i% = 1 to 3
        print i%
    end for
end function
";
        assert_eq!(reindent("test.bcl", source), expected);
    }

    #[test]
    fn label_and_statement_on_one_line_still_classifies_the_statement() {
        // `L6483: FOR Z0=0 TO Z9` -- a label immediately followed by a
        // real statement on the *same* line. The label itself adds no
        // indent, but the `for` after it must still open a level, or
        // its body and the matching `end for` end up misindented.
        let source = "\
function f%()
loopStart: for i% = 1 to 3
print i%
end for
end function
";
        let expected = "\
function f%()
    loopStart: for i% = 1 to 3
        print i%
    end for
end function
";
        assert_eq!(reindent("test.bcl", source), expected);
        assert_idempotent(source);
    }

    #[test]
    fn try_catch_finally_share_the_try_level() {
        let source = "\
function f%()
try
open \"x\" for input as #1
catch err%, erl%
print \"failed\"
finally
close #1
end try
end function
";
        let expected = "\
function f%()
    try
        open \"x\" for input as #1
    catch err%, erl%
        print \"failed\"
    finally
        close #1
    end try
end function
";
        assert_eq!(reindent("test.bcl", source), expected);
        assert_idempotent(source);
    }

    #[test]
    fn check_reports_only_the_differing_lines() {
        let source = "function f%()\n  print 1\nend function\n";
        let diffs = check("test.bcl", source);
        assert_eq!(
            diffs.len(),
            1,
            "diffs: {:?}",
            diffs
                .iter()
                .map(|d| (d.line, &d.before, &d.after))
                .collect::<Vec<_>>()
        );
        assert_eq!(diffs[0].line, 2);
    }

    #[test]
    fn already_compliant_source_reports_no_diffs() {
        let source = "function f%()\n    print 1\nend function\n";
        assert!(check("test.bcl", source).is_empty());
    }
}
