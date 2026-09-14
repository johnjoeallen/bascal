//! `bcc --format-check` / `bcc --format` -- BASCAL's own source formatter.
//!
//! Deliberately narrow: it never reflows an expression or changes line
//! count (the one exception -- splitting a `:`-joined multi-statement
//! line -- is intentionally *not* handled yet; see the module's own
//! tracking notes on why that needs block-conversion logic single-line
//! `if`/`elseif` can trigger). Every fix here is either pure whitespace
//! or a same-meaning keyword substitution, both recomputed from the real
//! token stream (`Lexer`, not a regex) so a string or comment's own
//! content is never touched:
//!
//! - **Indentation**, recomputed from a block-nesting stack (see `Frame`
//!   and `reindent`'s own per-keyword handling below).
//! - **Trailing whitespace**, stripped from every line unconditionally.
//! - **Comma spacing**: no space before, exactly one space after.
//! - **Operator spacing**: `=`, `<>`, `<=`, `>=`, `<`, `>`, `*`, `/`,
//!   `\`, `^`, `+=`, `-=`, `*=`, `/=`, `&&`, `||` are always binary in
//!   BASIC, so always get exactly one space on each side. `+`/`-` are
//!   only forced when they're unambiguously binary -- the token right
//!   before is a number, string, closing bracket, or a `%`/`&`/`!`/`#`/
//!   `$`-suffixed identifier (never a bare keyword, which is lexed
//!   identically to a variable name and would make `return -1` wrongly
//!   read as binary). Anything not covered by one of these rules keeps
//!   its original spacing, except a run of 2+ spaces between any two
//!   tokens, which always collapses to one.
//! - **Legacy closer keywords**: a bare `wend` becomes `end while`, and
//!   a bare `loop` (no trailing `while`/`until` -- that form has no
//!   `end do` equivalent, so it's left alone) becomes `end do`, matching
//!   the `end <keyword>` spelling every other block already uses.
//!
//! Keyword *casing* is deliberately not touched here -- the corpus is
//! consistent enough on indentation and closer spelling to infer a
//! confident default, but casing needs the same evidence-gathering
//! pass before picking one, and is left for a follow-up.
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
            out.push(raw_line.trim_end().to_string());
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
                "wend" | "loop" => pop_count += 1,
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
        let content = respace(raw_line, &toks, words.len() == 1, first_word);
        out.push(format!("{}{}", INDENT_UNIT.repeat(level), content));

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

/// True for a token kind that's always binary in BASIC -- safe to always
/// pad with exactly one space on each side, unlike `+`/`-`, which can
/// also be unary.
fn is_forced_binary_operator(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Eq
            | TokenKind::Ne
            | TokenKind::Lt
            | TokenKind::Le
            | TokenKind::Gt
            | TokenKind::Ge
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Backslash
            | TokenKind::Caret
            | TokenKind::PlusEq
            | TokenKind::MinusEq
            | TokenKind::StarEq
            | TokenKind::SlashEq
            | TokenKind::AndAnd
            | TokenKind::OrOr
    )
}

/// True for a token that can only be the *end* of a value -- a number,
/// string, closing bracket, or a `%`/`&`/`!`/`#`/`$`-suffixed identifier
/// (never a bare keyword, which lexes identically to a plain variable
/// name and would make `return -1`/`to -5` wrongly read as binary).
/// Used to tell a binary `+`/`-` (`total - 1`) from a unary one
/// (`return -1`) by looking at the token immediately before it.
fn is_value_ending(kind: &TokenKind) -> bool {
    match kind {
        TokenKind::Number(_)
        | TokenKind::Float(_)
        | TokenKind::HexLit(_)
        | TokenKind::String(_)
        | TokenKind::RParen
        | TokenKind::RBracket => true,
        TokenKind::Ident(s) => s.ends_with(['%', '&', '!', '#', '$']),
        _ => false,
    }
}

/// Rebuilds one line's content (everything after its own leading
/// whitespace, which `reindent` replaces separately) from `toks`,
/// normalizing inter-token spacing without ever touching a token's own
/// text -- every token's text is sliced verbatim from `raw_line` by its
/// real source span, never regenerated from its parsed value, so a
/// string's exact quoting or a number's exact digits can never drift.
///
/// `is_solitary_keyword`/`first_word` identify a bare `wend` or `loop`
/// (the only content on the line besides an optional trailing comment)
/// so its own token text can be swapped for the `end while`/`end do`
/// spelling every other block already uses -- `loop while`/`loop until`
/// has no such equivalent and is left alone.
fn respace(raw_line: &str, toks: &[&Token], is_solitary_keyword: bool, first_word: &str) -> String {
    let chars: Vec<char> = raw_line.chars().collect();
    let real: Vec<&&Token> = toks
        .iter()
        .filter(|t| !matches!(t.kind, TokenKind::Newline | TokenKind::Eof))
        .collect();
    if real.is_empty() {
        return raw_line.trim().to_string();
    }

    // Each real token's own [start, end) char span in `raw_line`, found
    // by trimming trailing whitespace off the gap up to the next
    // token's start (or end of line, for the last one).
    let mut spans: Vec<(usize, usize)> = Vec::with_capacity(real.len());
    for (i, t) in real.iter().enumerate() {
        let start = t.pos.column.saturating_sub(1).min(chars.len());
        let bound = if i + 1 < real.len() {
            real[i + 1].pos.column.saturating_sub(1)
        } else {
            chars.len()
        }
        .clamp(start, chars.len());
        let mut end = bound;
        while end > start && chars[end - 1].is_whitespace() {
            end -= 1;
        }
        spans.push((start, end));
    }

    let legacy_replacement =
        if is_solitary_keyword && (first_word == "wend" || first_word == "loop") {
            real.iter().enumerate().find_map(|(i, t)| {
                let TokenKind::Ident(original) = &t.kind else {
                    return None;
                };
                if !original.eq_ignore_ascii_case(first_word) {
                    return None;
                }
                let upper = original.chars().any(char::is_alphabetic)
                    && original
                        .chars()
                        .filter(|c| c.is_alphabetic())
                        .all(char::is_uppercase);
                let text = match (first_word, upper) {
                    ("wend", true) => "END WHILE",
                    ("wend", false) => "end while",
                    ("loop", true) => "END DO",
                    ("loop", false) => "end do",
                    _ => return None,
                };
                Some((i, text))
            })
        } else {
            None
        };

    // `+`/`-` are binary exactly when the token right before them ends a
    // value (see `is_value_ending`'s own doc comment for why a bare
    // keyword doesn't count).
    let mut is_binary_pm = vec![false; real.len()];
    for i in 1..real.len() {
        if matches!(real[i].kind, TokenKind::Plus | TokenKind::Minus)
            && is_value_ending(&real[i - 1].kind)
        {
            is_binary_pm[i] = true;
        }
    }

    let mut out = String::new();
    for i in 0..real.len() {
        if let Some((idx, text)) = legacy_replacement {
            if idx == i {
                out.push_str(text);
            } else {
                let (s, e) = spans[i];
                out.push_str(&chars[s..e].iter().collect::<String>());
            }
        } else {
            let (s, e) = spans[i];
            out.push_str(&chars[s..e].iter().collect::<String>());
        }

        if matches!(
            real[i].kind,
            TokenKind::Comment(_) | TokenKind::BlockComment(_)
        ) {
            break; // nothing meaningful can follow a comment on its own line
        }
        if i + 1 >= real.len() {
            break;
        }

        let this_kind = &real[i].kind;
        let next_kind = &real[i + 1].kind;
        let force_space = matches!(this_kind, TokenKind::Comma)
            || is_forced_binary_operator(this_kind)
            || is_forced_binary_operator(next_kind)
            || (matches!(this_kind, TokenKind::Plus | TokenKind::Minus) && is_binary_pm[i])
            || (matches!(next_kind, TokenKind::Plus | TokenKind::Minus) && is_binary_pm[i + 1]);

        let gap: &str = if matches!(next_kind, TokenKind::Comma) {
            ""
        } else if force_space {
            " "
        } else {
            // Not a rule above: preserve whether tokens were adjacent
            // (no space at all, e.g. a call's own `(`) or separated, but
            // collapse any run of 2+ spaces down to exactly one.
            let (_, this_end) = spans[i];
            let next_start = real[i + 1].pos.column.saturating_sub(1);
            if next_start.saturating_sub(this_end) == 0 {
                ""
            } else {
                " "
            }
        };
        out.push_str(gap);
    }
    out
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
end for
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
    end for
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
data 1, 2, 3
data 4, 5, 6
print \"after\"
end function
";
        let expected = "\
function f%()
    myLabel:
        ' a comment about the table
        data 1, 2, 3
        data 4, 5, 6
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
data 1, 2, 3
secondTable:
data 4, 5, 6
end function
";
        let expected = "\
function f%()
    firstTable:
        data 1, 2, 3
    secondTable:
        data 4, 5, 6
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
data 1, 2, 3

/*
 * Unrelated comment, not about myTable.
 */
print \"after\"
end function
";
        let expected = "\
function f%()
    myTable:
        data 1, 2, 3

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
    fn trailing_whitespace_is_stripped_everywhere() {
        // Including inside a multi-line block comment's own interior,
        // which stays otherwise untouched (see
        // multi_line_block_comment_interior_is_left_alone above).
        let source = "function f%()   \nprint 1  \n/* a comment   \n   more   \n*/\nend function\n";
        let expected = "function f%()\n    print 1\n    /* a comment\n   more\n*/\nend function\n";
        assert_eq!(reindent("test.bcl", source), expected);
        assert_idempotent(source);
    }

    #[test]
    fn comma_gets_no_space_before_and_one_space_after() {
        let source = "function f%()\nprint a% ,b% , c%\nend function\n";
        let expected = "function f%()\n    print a%, b%, c%\nend function\n";
        assert_eq!(reindent("test.bcl", source), expected);
        assert_idempotent(source);
    }

    #[test]
    fn always_binary_operators_get_padded_both_sides() {
        let source = "function f%()\nx%=1*2\nif x%<>3 and x%<=4 then print x%\nend function\n";
        let expected = "function f%()\n    x% = 1 * 2\n    if x% <> 3 and x% <= 4 then print x%\nend function\n";
        assert_eq!(reindent("test.bcl", source), expected);
        assert_idempotent(source);
    }

    #[test]
    fn binary_minus_is_padded_but_unary_minus_is_left_alone() {
        // `total%-1` (preceded by a suffixed identifier -- unambiguously
        // a value) is binary; `return -1`/`to -5` (preceded by a bare
        // keyword, lexed the same as a variable name) are left exactly
        // as written rather than risk guessing wrong.
        let source = "\
function f%()
y% = total%-1
if y% < 0 then return -1
for i% = 10 to -5 step -1
end for
end function
";
        let expected = "\
function f%()
    y% = total% - 1
    if y% < 0 then return -1
    for i% = 10 to -5 step -1
    end for
end function
";
        assert_eq!(reindent("test.bcl", source), expected);
        assert_idempotent(source);
    }

    #[test]
    fn multiple_spaces_between_tokens_collapse_to_one() {
        let source = "function f%()\nprint    \"a\"   ;    \"b\"\nend function\n";
        let expected = "function f%()\n    print \"a\" ; \"b\"\nend function\n";
        assert_eq!(reindent("test.bcl", source), expected);
        assert_idempotent(source);
    }

    #[test]
    fn bare_wend_becomes_end_while_preserving_case_and_trailing_comment() {
        let source = "\
function f%()
while x% < 10
x% = x% + 1
wend
WHILE x% > 0
x% = x% - 1
WEND ' done
end function
";
        let expected = "\
function f%()
    while x% < 10
        x% = x% + 1
    end while
    WHILE x% > 0
        x% = x% - 1
    END WHILE ' done
end function
";
        assert_eq!(reindent("test.bcl", source), expected);
        assert_idempotent(source);
    }

    #[test]
    fn bare_loop_becomes_end_do_but_conditioned_loop_is_untouched() {
        let source = "\
function f%()
do
x% = x% + 1
loop
do
x% = x% - 1
loop until x% = 0
end function
";
        let expected = "\
function f%()
    do
        x% = x% + 1
    end do
    do
        x% = x% - 1
    loop until x% = 0
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
