//! `bcc --format-check` / `bcc --format` -- BASCAL's own source formatter.
//!
//! Deliberately narrow: it never reflows an expression, and the only
//! rule that changes line count at all is splitting a `:`-chained
//! multi-statement line -- `a = 1 : b = 2` becomes two ordinary lines,
//! and a single-line `if` (`if cond then s1:s2`) whose then-/else-body
//! has more than one such statement is exploded into full block form
//! instead (see `explode_multi_statement_lines`). Every other fix here
//! is pure whitespace or a same-meaning keyword substitution, both
//! recomputed from the real token stream (`Lexer`, not a regex) so a
//! string or comment's own content is never touched:
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
//!   the `end <keyword>` form every other block already uses.
//! - **Keyword casing**: every token whose text case-insensitively
//!   matches a reserved word (see `KEYWORDS`) is lowercased, e.g. a
//!   stray `IF`/`PRINT`/`END FUNCTION` left over from a BASIC source a
//!   program was ported from becomes `if`/`print`/`end function`.
//!   `KEYWORDS` lists only words the parser itself treats as reserved
//!   syntax (`classify_keyword`, `check_keyword`'s own call sites,
//!   the `and`/`or`/`not`/`xor`/`true`/`false` operators/literals, and
//!   the scalar type names) -- never a builtin *function* name like
//!   `len`/`mid`/`sizeof` recognized by a different, name-based
//!   mechanism, since conflating the two would broaden this well past
//!   "keyword casing". Parsing itself is already fully case-insensitive
//!   (`keyword_eq`/`classify_keyword` both lowercase before comparing),
//!   so this can never change what a line parses as -- only a plain
//!   identifier that happens to be written exactly like a reserved word
//!   (vanishingly rare, and would show up as a generated-output diff in
//!   this formatter's own corpus verification) could visibly change.
//! - **Builtin casing**: a real BASIC intrinsic (`crate::codegen_basic::
//!   BASIC_BUILTINS` -- `RND`, `LEN`, `STR$`, `MID$`, ...) is lowercased
//!   the same way a reserved word is, stripping and reattaching its own
//!   `%`/`&`/`!`/`#`/`$` suffix first (the builtin list itself is bare
//!   names). A user function is never allowed to shadow one of these
//!   (see `reject_functions_shadowing_builtins` in `resolver.rs`), so
//!   there's no ambiguity to worry about.
//! - **Declaration-matching casing**: every reference to a function,
//!   procedure, or variable is rewritten to match how that name's own
//!   declaration was written (first occurrence wins, file-wide), the
//!   same way `rustfmt` doesn't touch identifier casing itself but a
//!   project's own consistent casing stops drifting between a `dim` and
//!   a later, differently-cased use. Built from one full parse of the
//!   file (`crate::parser::Parser`, matching `resolver.rs::
//!   check_strict_vars`'s own pre-`records::lower` AST) -- if that parse
//!   fails for any reason, this step is silently skipped (see this
//!   module's own doc comment on never refusing to run) and every other
//!   rule above still applies. Deliberately narrow: a name that's also a
//!   record type name is dropped entirely, both the type and any
//!   same-cased variable left untouched (`record Header` alongside `file
//!   header as Header = open(...)` is legal, real corpus code -- see
//!   `collect_program_names`'s own doc comment for why the two can't
//!   safely share this map's one slot per name); record *field* and
//!   *method* names (both reached only through `.` syntax, which needs
//!   its own receiver-type-aware AST walk this doesn't
//!   attempt) are excluded for the same reason a builtin *method* isn't
//!   touched above. All three are candidates for a future, more
//!   context-aware pass.
//! - **Multi-statement line splitting**: every physical line holding
//!   more than one top-level `:`-chained statement is split, one
//!   statement per line, at the same nesting level -- `a = 1 : b = 2`
//!   becomes two lines, no new block introduced. A single-line `if`
//!   (`if cond then s1:s2`, real BASIC's own no-`end if` form -- see
//!   `parser.rs::parse_single_line_if`) whose then- or else-body has
//!   more than one such statement is different: it can't just split in
//!   place, since a single-line `if` has no closing keyword for a
//!   multi-line body to end at. That case explodes into full block
//!   form instead -- `if cond then` / one statement per line / (`else` /
//!   one statement per line, if present) / `end if` -- indented and
//!   respaced/recased by every rule above exactly like a hand-written
//!   block `if`; a nested single-line `if` cascades too, since `if a
//!   then if b then s1:s2` can't leave the outer `if` alone once the
//!   inner one is forced onto multiple lines either (see
//!   `if_needs_explosion`). A trailing single-line comment on an
//!   exploded `if` moves to the new `if ... then` header line, since it
//!   almost always describes the condition, not whichever branch
//!   happened to be physically last; on a plain split line it just
//!   stays with whichever statement was already physically last.
//!   Narrower in one way: a line with a trailing multi-line `/* ... */`
//!   comment is left exactly as written rather than risk splitting a
//!   comment deliberately written to span lines. See
//!   `explode_multi_statement_lines`'s own doc comment for the exact
//!   rules -- it runs as its own pass, before every rule above, so its
//!   own output (indentation aside) is exactly what those rules already
//!   know how to handle.
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

use crate::ast::{BasicIdent, Program, Statement, Stmt, TypeSuffix};
use crate::lexer::{Lexer, Token, TokenKind};
use crate::parser::Parser;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

const INDENT_UNIT: &str = "    ";

/// Every word the parser itself treats as reserved syntax -- statement
/// dispatch (`classify_keyword`), mid-statement structural checks
/// (`check_keyword`/`expect_keyword`/`check_next_keyword`), the logical
/// operators and boolean literals (`keyword_eq`), and the scalar type
/// names (`scalar_type_name`) -- all in `parser.rs`. Deliberately excludes
/// builtin *function* names (`len`, `mid`, `sizeof`, ...) recognized by a
/// separate, name-based mechanism rather than the grammar itself; casing
/// those is a different, broader change than this list makes.
const KEYWORDS: &[&str] = &[
    "and", "append", "as", "base", "beep", "binary", "byref", "byval", "case", "catch", "clear",
    "close", "cls", "color", "combines", "common", "const", "continue", "data", "declare", "def",
    "dim", "do", "double", "downto", "else", "elseif", "end", "erase", "error", "exit", "false",
    "field", "file", "finally", "fluent", "fn", "for", "function", "get", "global", "gosub",
    "goto", "if", "import", "input", "integer", "is", "kill", "let", "library", "line", "locate",
    "long", "loop", "lprint", "lset", "method", "mod", "name", "next", "not", "on", "open",
    "option", "or", "out", "output", "poke", "print", "procedure", "program", "put", "random",
    "randomize", "read", "record", "require", "restore", "resume", "return", "returns", "rset",
    "seek", "select", "shared", "single", "step", "stop", "string", "swap", "system", "then",
    "throw", "to", "true", "try", "until", "using", "wend", "while", "width", "write", "xor",
];

fn is_keyword_word(lower: &str) -> bool {
    KEYWORDS.contains(&lower)
}

fn is_builtin_word(lower_base: &str) -> bool {
    crate::codegen_basic::BASIC_BUILTINS.contains(&lower_base)
}

/// A declared name's own key: its base name lowercased plus its type
/// suffix (`None` for an unsuffixed, record-typed variable) -- two names
/// that only differ by suffix are different declarations, matching how
/// BASCAL itself tells `total%` and `total$` apart.
type NameKey = (String, Option<TypeSuffix>);

fn record_declaration(map: &mut HashMap<NameKey, String>, ident: &BasicIdent) {
    map.entry((ident.name.to_ascii_lowercase(), ident.suffix))
        .or_insert_with(|| ident.as_basic());
}

/// Every name a `dim`/`declare`/`const`/`global`, a `for` loop's own
/// counter, a raw `FIELD` buffer variable, a `file <var> = open(...)`
/// declaration, or a `catch` clause's own `err`/`erl`/source variable
/// declares, recursing into every nested statement body -- deliberately
/// not scope-aware (unlike
/// `resolver.rs::collect_declarations`, which this otherwise mirrors):
/// this only needs one file-wide "what casing does this name use" answer, not
/// a validity check, and two same-named declarations in different scopes
/// overwhelmingly agree on casing in practice; the rare disagreement is
/// caught by this formatter's own corpus verification, not by a design
/// meant to handle it.
fn collect_declared_names(statements: &[Stmt], map: &mut HashMap<NameKey, String>) {
    for stmt in statements {
        match &stmt.kind {
            Statement::Dim { name, .. } | Statement::Const { name, .. } => {
                record_declaration(map, name);
            }
            Statement::GlobalDecl(ident) => {
                record_declaration(map, ident);
            }
            Statement::FileDecl { var, .. } => {
                record_declaration(map, var);
            }
            Statement::For { var, body, .. } => {
                record_declaration(map, var);
                collect_declared_names(body, map);
            }
            Statement::Field { fields, .. } => {
                for (_, name) in fields {
                    record_declaration(map, name);
                }
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_declared_names(then_body, map);
                collect_declared_names(else_body, map);
            }
            Statement::While { body, .. } | Statement::Do { body, .. } => {
                collect_declared_names(body, map);
            }
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_declared_names(&case.body, map);
                }
                collect_declared_names(else_body, map);
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                collect_declared_names(try_body, map);
                if let Some(catch) = catch {
                    record_declaration(map, &catch.err_var);
                    record_declaration(map, &catch.erl_var);
                    if let Some(source_var) = &catch.source_var {
                        record_declaration(map, source_var);
                    }
                    collect_declared_names(&catch.body, map);
                }
                collect_declared_names(finally_body, map);
            }
            _ => {}
        }
    }
}

/// Every declared name in `program` this formatter will canonicalize
/// call/use sites against: ordinary (non-method) functions and
/// procedures and their own parameters, and every variable
/// `collect_declared_names` finds (top-level and inside each function/
/// procedure body) -- *except* a name that's also a record type name.
/// See this module's own doc comment for why record *field* and
/// *method* names are excluded outright; record *type* names are their
/// own separate namespace instead: a type name and a same-cased variable
/// name are legal, real corpus code (`record Header` alongside `file
/// header as Header = open(...)` -- see `examples/card_catalog/
/// card_catalog.bcl`), but this map has only one slot per
/// case-insensitive name, so a variable and its own same-named record
/// type can't both live in it safely -- not even the record's own
/// declaration line is safe, since its type name token is otherwise
/// indistinguishable from an ordinary identifier at render time. Rather
/// than guess which of the two a given occurrence means, any name a
/// record type also claims is dropped from this map entirely, leaving
/// every occurrence of it (type *and* variable alike) untouched.
fn collect_program_names(program: &Program) -> HashMap<NameKey, String> {
    let mut map = HashMap::new();
    for f in &program.functions {
        if f.receiver.is_none() && f.record_receiver.is_none() {
            record_declaration(&mut map, &f.name);
        }
        for param in &f.params {
            record_declaration(&mut map, &param.name);
        }
        collect_declared_names(&f.body, &mut map);
    }
    collect_declared_names(&program.statements, &mut map);
    for r in &program.records {
        map.remove(&(r.name.to_ascii_lowercase(), None));
    }
    map
}

/// Collects declared names for declaration-matching casing, from `source`
/// alone plus -- when `filename` names a real, readable file -- every
/// `require`/`import`ed library and `shared` file it transitively pulls
/// in too, the same resolution `bcc` itself does before compiling (see
/// `load_merged_program`). Best-effort throughout: an unparseable file,
/// a `filename` with no file behind it (as in this module's own unit
/// tests, which pass a nonexistent placeholder path), or a library that
/// can't be resolved all just fall back to *not* including that source
/// -- this formatter never refuses to run over a syntax error or a
/// missing dependency.
fn declared_names(filename: &str, source: &str, library_dirs: &[PathBuf]) -> HashMap<NameKey, String> {
    if let Some(program) = load_merged_program(filename, library_dirs) {
        return collect_program_names(&program);
    }
    let tokens = Lexer::new(filename, source).lex();
    match Parser::new(filename.to_string(), tokens).parse_program() {
        Ok(program) => collect_program_names(&program),
        Err(_) => HashMap::new(),
    }
}

/// Parses `filename` from disk and merges in every file it transitively
/// `require`s/`import`s, reusing `bcc`'s own multi-file resolution
/// (`driver::load_program_recursive`) so a call/variable site that
/// refers to a function/procedure/variable declared in a required
/// library gets matched against *that* declaration's own casing, not
/// just this file's own. `library_dirs` mirrors the CLI's own `-L`
/// flags; the file's own parent directory and the bundled `com/`
/// standard library are always searched too (see
/// `driver::search_roots`). Returns `None` -- silently, no diagnostic --
/// when `filename` isn't a real file (this module's own unit tests use
/// placeholder filenames) or when parsing/resolution fails for any
/// reason; the caller falls back to a plain, single-file, in-memory
/// parse in that case.
fn load_merged_program(filename: &str, library_dirs: &[PathBuf]) -> Option<Program> {
    let path = Path::new(filename);
    if !path.is_file() {
        return None;
    }
    let mut options = crate::driver::CompileOptions::new();
    options.library_dirs = library_dirs.to_vec();
    if let Some(parent) = path.parent() {
        let parent = parent.to_path_buf();
        if !options.library_dirs.contains(&parent) {
            options.library_dirs.insert(0, parent);
        }
    }
    let mut visited = std::collections::HashSet::new();
    crate::driver::load_program_recursive(path, true, &options, &mut visited).ok()
}

/// Rewrites every physical line holding more than one top-level
/// `:`-chained statement, one statement per output line -- either at
/// the *same* nesting level (an ordinary `a = 1 : b = 2` colon-chain),
/// or, for a single-line `if` whose then-/else-body itself needs
/// splitting, exploded into full block form (`if ... then` / one
/// statement per line / `end if`) since single-line-if syntax has no
/// closing keyword for a multi-line body to end at. Either way, the
/// rest of this formatter's existing indentation logic takes over from
/// there unchanged: indentation of the newly emitted lines is not this
/// function's job -- whatever it emits gets re-lexed and reindented by
/// `reindent` right after, the same as any hand-written multi-line
/// source -- only correct newline placement and preserving every
/// token's own text verbatim matters here. A nested single-line `if`
/// found inside a body that's itself being exploded (`if a then if b
/// then s1:s2 else s3`) is exploded too, recursively, in the same pass.
///
/// Deliberately narrow, matching this module's own "never refuses to
/// run" philosophy: best-effort throughout, silently leaving a line
/// exactly as written whenever splitting it isn't a clean, unambiguous
/// rewrite -- an unparseable file is returned unchanged, and so is any
/// individual line with a trailing multi-line `/* ... */` block
/// comment, rather than risk splitting a comment that was never meant
/// to be read on one line.
///
/// A trailing single-line comment, when present, can only ever belong
/// to the line's own last top-level statement (a comment consumes to
/// end of line) -- if that statement is a single-line `if` being
/// exploded, the comment moves to the end of the newly generated `if
/// ... then` header line instead of to `end if` or to whichever branch
/// happened to be physically last, since it almost always describes the
/// condition; otherwise it just stays with that last statement, exactly
/// as written.
fn explode_multi_statement_lines(filename: &str, source: &str) -> String {
    let tokens = Lexer::new(filename, source).lex();
    let program = match Parser::new(filename.to_string(), tokens).parse_program() {
        Ok(p) => p,
        Err(_) => return source.to_string(),
    };

    let mut targets: BTreeMap<usize, Vec<&Stmt>> = BTreeMap::new();
    collect_line_targets(&program.statements, &mut targets);
    for f in &program.functions {
        collect_line_targets(&f.body, &mut targets);
    }
    if targets.is_empty() {
        return source.to_string();
    }

    // Re-lex: `parse_program` consumed the first token stream, and we
    // need real tokens (with real positions) to slice text from.
    let tokens = Lexer::new(filename, source).lex();
    let lines: Vec<&str> = source.lines().collect();

    let mut replacement: HashMap<usize, String> = HashMap::new();
    for (&lineno, group) in &targets {
        let line_tokens: Vec<&Token> = tokens
            .iter()
            .filter(|t| {
                t.pos.line == lineno && !matches!(t.kind, TokenKind::Newline | TokenKind::Eof)
            })
            .collect();
        let Some(raw_line) = lines.get(lineno - 1) else {
            continue;
        };
        let chars: Vec<char> = raw_line.chars().collect();
        if let Some(rendered) = render_target_line(group, &line_tokens, &chars) {
            replacement.insert(lineno, rendered);
        }
    }
    if replacement.is_empty() {
        return source.to_string();
    }

    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        let lineno = i + 1;
        match replacement.get(&lineno) {
            Some(rendered) => out.push_str(rendered),
            None => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    out
}

/// True for an `if` actually written in single-line form -- its
/// then-/else-body's own first statement (if any) still starts on the
/// *same* physical line as the `if` keyword itself, which single-line
/// form always does and block form never can (`parse_if` requires a
/// newline right after `then` for block form). An ordinary block `if`
/// can have a body of any length, including more than one statement --
/// `if_needs_explosion` must never mistake that for something needing
/// exploding just because of its body's length.
fn is_single_line_if(stmt: &Stmt) -> bool {
    let Statement::If {
        then_body,
        else_body,
        ..
    } = &stmt.kind
    else {
        return false;
    };
    let starts_here = |body: &[Stmt]| body.first().is_none_or(|s| s.pos.line == stmt.pos.line);
    starts_here(then_body) && starts_here(else_body)
}

/// True for a `Statement::If` that itself needs exploding (more than
/// one statement on either side), *or* whose then-/else-body is a lone
/// statement that is itself such an `if` -- `if a then if b then s1:s2`
/// parses as the outer's then-body holding exactly one statement (the
/// inner `if`), so the outer can't stay single-line either once the
/// inner one is forced onto multiple lines; single-line-if syntax has
/// no way to hold a multi-line body. Checked recursively so a deeper
/// chain (`if a then if b then if c then s1:s2`) still finds its way
/// back to the outermost `if` that has to become the actual rewrite
/// target -- see `collect_line_targets`.
fn if_needs_explosion(stmt: &Stmt) -> bool {
    if !is_single_line_if(stmt) {
        return false;
    }
    let Statement::If {
        then_body,
        else_body,
        ..
    } = &stmt.kind
    else {
        return false;
    };
    if then_body.len() > 1 || else_body.len() > 1 {
        return true;
    }
    if let [only] = then_body.as_slice() {
        if if_needs_explosion(only) {
            return true;
        }
    }
    if let [only] = else_body.as_slice() {
        if if_needs_explosion(only) {
            return true;
        }
    }
    false
}

/// Groups `statements` (recursing into every nested statement body) by
/// physical line, and records a line as a rewrite target whenever
/// either: it holds more than one top-level statement (an ordinary
/// `a = 1 : b = 2` colon-chain, no `if` involved at all), or its lone
/// statement `if_needs_explosion`. A target line's own group members
/// are *not* separately recursed into here for their own nested bodies
/// when the group is a lone exploding `if`: `render_exploded_if` walks
/// that one itself, so any nested single-line `if` inside it -- however
/// deep -- is discovered and exploded as part of rendering its
/// outermost ancestor, not as an independent target sharing the same
/// physical line as that ancestor's own, still-unwritten header. Every
/// other statement kind's own body always lives on later physical
/// lines, so it's always safe to recurse into regardless of whether
/// this line turned out to be a target.
fn collect_line_targets<'a>(statements: &'a [Stmt], targets: &mut BTreeMap<usize, Vec<&'a Stmt>>) {
    let mut by_line: BTreeMap<usize, Vec<&'a Stmt>> = BTreeMap::new();
    for stmt in statements {
        by_line.entry(stmt.pos.line).or_default().push(stmt);
    }
    for (line, group) in by_line {
        let (_, rest) = split_leading_label(&group);
        // A label appearing anywhere in `rest` (i.e. anywhere but as a
        // *leading* one, already set aside) is an exotic enough shape
        // to just leave alone entirely, rather than risk mangling it --
        // `render_target_line`'s own trailing-colon stripping assumes a
        // colon it finds there is a plain statement separator, which
        // isn't true for a label's own defining colon.
        let has_further_label = rest.iter().any(|s| matches!(s.kind, Statement::Label(_)));
        let is_target = !has_further_label
            && (rest.len() > 1 || rest.iter().any(|s| if_needs_explosion(s)));
        if is_target {
            targets.insert(line, group);
        }
    }

    for stmt in statements {
        match &stmt.kind {
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                if if_needs_explosion(stmt) {
                    continue;
                }
                collect_line_targets(then_body, targets);
                collect_line_targets(else_body, targets);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_line_targets(body, targets),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_line_targets(&case.body, targets);
                }
                collect_line_targets(else_body, targets);
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                collect_line_targets(try_body, targets);
                if let Some(catch) = catch {
                    collect_line_targets(&catch.body, targets);
                }
                collect_line_targets(finally_body, targets);
            }
            _ => {}
        }
    }
}

/// A 1-indexed source column, converted to a 0-indexed `chars` slot,
/// clamped to the line's own length.
fn col_to_index(column: usize, len: usize) -> usize {
    column.saturating_sub(1).min(len)
}

/// A trailing comment on a physical line, when present, is always the
/// very last token on it (a comment consumes to end of line) and so can
/// only ever belong to the *last* top-level statement on that line --
/// `render_target_line` uses this once per line, not once per
/// statement. Returns `Ok(None)` for no trailing comment, `Ok(Some((col,
/// text)))` for one, and `Err(())` for a trailing multi-line `/* ...
/// */` block comment, which bails the whole line out of rewriting:
/// splitting one deliberately written to span lines is more likely to
/// mangle a hand-aligned layout than respect it.
fn trailing_comment(line_tokens: &[&Token], chars: &[char]) -> Result<Option<(usize, String)>, ()> {
    match line_tokens.last() {
        Some(t) if matches!(t.kind, TokenKind::Comment(_)) => {
            let start = col_to_index(t.pos.column, chars.len());
            Ok(Some((t.pos.column, chars[start..].iter().collect())))
        }
        Some(t) => match &t.kind {
            TokenKind::BlockComment(text) if text.contains('\n') => Err(()),
            TokenKind::BlockComment(_) => {
                let start = col_to_index(t.pos.column, chars.len());
                Ok(Some((t.pos.column, chars[start..].iter().collect())))
            }
            _ => Ok(None),
        },
        None => Ok(None),
    }
}

/// Splits a leading `label:` off `group`, when its first member is one --
/// the common, deliberate `label: statement` idiom (a label and the
/// single statement it guards, written on one line) must never be torn
/// apart into a bare label line plus a separate statement line just
/// because splitting logic is running at all. Only a *leading* label is
/// recognized; one appearing later in `group` is handled by
/// `collect_line_targets` refusing to treat the line as a target at all
/// (see its own doc comment).
fn split_leading_label<'g, 'a>(group: &'g [&'a Stmt]) -> (Option<&'a Stmt>, &'g [&'a Stmt]) {
    match group.split_first() {
        Some((first, rest)) if matches!(first.kind, Statement::Label(_)) => (Some(*first), rest),
        _ => (None, group),
    }
}

/// True when every statement in `group` (after any leading label; see
/// `split_leading_label`) is separated from the next by a real `:`
/// token -- *not* true for `dim a%, b%`, which the parser desugars into
/// two sibling `Statement::Dim` nodes sharing one line and one another's
/// comma, not a colon (see `Parser::parse_dim`'s own doc comment): two
/// AST siblings on the same line doesn't by itself mean "independently
/// splittable statements". Splitting on anything but a genuine `:`
/// would silently turn a comma-joined multi-name declaration into two
/// bare, invalid statements.
fn adjacent_members_are_colon_separated(group: &[&Stmt], line_tokens: &[&Token]) -> bool {
    group.windows(2).all(|pair| {
        let next = pair[1];
        match line_tokens.iter().position(|t| t.pos == next.pos) {
            Some(0) | None => false,
            Some(idx) => matches!(line_tokens[idx - 1].kind, TokenKind::Colon),
        }
    })
}

/// Renders one target physical line (see `collect_line_targets`) as its
/// replacement text -- possibly several lines, always ending in `\n` --
/// or `None` when it turns out not to be a clean rewrite after all (a
/// trailing multi-line block comment, or adjacent statements that
/// aren't actually `:`-separated -- see
/// `adjacent_members_are_colon_separated`). `group` is every top-level
/// statement sharing this physical line, in source order; each is
/// either an ordinary statement (sliced verbatim onto its own line) or
/// an `if` needing explosion (rendered as a block via
/// `render_exploded_if`). A leading `label:` (see `split_leading_label`)
/// stays attached to whatever follows it, on the same output line.
/// Only the last member's own trailing region can carry the line's own
/// trailing comment -- see `trailing_comment`.
fn render_target_line(group: &[&Stmt], line_tokens: &[&Token], chars: &[char]) -> Option<String> {
    if !adjacent_members_are_colon_separated(group, line_tokens) {
        return None;
    }

    let comment = trailing_comment(line_tokens, chars).ok()?;
    let line_end_col = comment.as_ref().map(|(c, _)| *c).unwrap_or(chars.len() + 1);

    let (label, rest) = split_leading_label(group);
    let label_prefix = match label {
        Some(label_stmt) => {
            let start = col_to_index(label_stmt.pos.column, chars.len());
            let end = rest
                .first()
                .map(|s| col_to_index(s.pos.column, chars.len()))
                .unwrap_or(chars.len());
            let text: String = chars[start..end.max(start)].iter().collect();
            format!("{} ", text.trim())
        }
        None => String::new(),
    };

    let mut out = String::new();
    for (i, stmt) in rest.iter().enumerate() {
        let is_last = i + 1 == rest.len();
        let region_end_col = if is_last {
            line_end_col
        } else {
            rest[i + 1].pos.column
        };
        let prefix = if i == 0 { label_prefix.as_str() } else { "" };
        if if_needs_explosion(stmt) {
            let own_comment = if is_last { comment.clone() } else { None };
            let rendered =
                render_exploded_if(stmt, line_tokens, chars, region_end_col, own_comment)?;
            out.push_str(prefix);
            out.push_str(&rendered);
            continue;
        }
        let start = col_to_index(stmt.pos.column, chars.len());
        let end = col_to_index(region_end_col, chars.len());
        let text: String = chars[start..end.max(start)].iter().collect();
        let text = text.trim();
        let text = text.strip_suffix(':').map(str::trim_end).unwrap_or(text);
        out.push_str(prefix);
        out.push_str(text);
        if is_last {
            if let Some((_, c)) = &comment {
                out.push(' ');
                out.push_str(c.trim_end());
            }
        }
        out.push('\n');
    }
    Some(out)
}

/// Renders one target `if` as block-form text -- possibly several
/// lines, always ending in `\n` -- or `None` on the (already screened
/// out by the caller) multi-line-block-comment case. `line_tokens` is
/// every token on `stmt`'s own physical line, `chars` that line's own
/// raw text as chars -- both shared unchanged across a recursive call
/// for a nested single-line `if`, since it necessarily lives on the
/// very same physical line as its parent. `region_end_col` (1-indexed,
/// exclusive) is the caller-determined boundary for this `if`'s own
/// content -- the next sibling statement's start column, when this
/// `if` isn't the last thing on its line, or the line's own end
/// otherwise. `comment`, when `Some`, is placed at the end of the new
/// `if ... then` header line instead of wherever it originally
/// trailed -- since it almost always describes the condition, not
/// whichever branch happened to be physically last -- and is only ever
/// passed for the group's own last (and therefore outermost, in a
/// nested chain) `if`; a nested one recurses with `None`.
fn render_exploded_if(
    stmt: &Stmt,
    line_tokens: &[&Token],
    chars: &[char],
    region_end_col: usize,
    comment: Option<(usize, String)>,
) -> Option<String> {
    let Statement::If {
        then_body,
        else_body,
        ..
    } = &stmt.kind
    else {
        return None;
    };

    let if_idx = line_tokens.iter().position(|t| t.pos == stmt.pos)?;
    let then_idx = line_tokens[if_idx + 1..].iter().position(|t| {
        matches!(&t.kind, TokenKind::Ident(s) if s.eq_ignore_ascii_case("then"))
    })? + if_idx
        + 1;

    let else_idx = if else_body.is_empty() {
        None
    } else {
        line_tokens[then_idx + 1..]
            .iter()
            .position(|t| matches!(&t.kind, TokenKind::Ident(s) if s.eq_ignore_ascii_case("else")))
            .map(|i| i + then_idx + 1)
    };

    let condition_start = col_to_index(line_tokens[if_idx + 1].pos.column, chars.len());
    let condition_end = col_to_index(line_tokens[then_idx].pos.column, chars.len());
    let condition: String = chars[condition_start..condition_end.max(condition_start)]
        .iter()
        .collect::<String>()
        .trim()
        .to_string();

    let mut out = String::new();
    out.push_str("if ");
    out.push_str(&condition);
    out.push_str(" then");
    if let Some((_, text)) = &comment {
        out.push(' ');
        out.push_str(text.trim_end());
    }
    out.push('\n');

    let then_end_col = else_idx
        .map(|i| line_tokens[i].pos.column)
        .unwrap_or(region_end_col);
    render_body_lines(then_body, line_tokens, chars, then_end_col, &mut out)?;

    if else_idx.is_some() {
        out.push_str("else\n");
        render_body_lines(else_body, line_tokens, chars, region_end_col, &mut out)?;
    }

    out.push_str("end if\n");
    Some(out)
}

/// Renders one `if`'s then- or else-body, one statement per line
/// (recursing into `render_exploded_if` for a nested single-line `if`
/// that itself needs exploding -- it shares `line_tokens`/`chars` with
/// its parent unchanged, since it necessarily lives on the very same
/// physical line, and always recurses with no comment of its own: only
/// the outermost `if` in a nested chain ever carries one, already
/// placed on its own header by the caller), up to `region_end_col`
/// (1-indexed source column, exclusive -- the position of whatever
/// comes right after this body: `else`, a trailing comment, or simply
/// end of line).
fn render_body_lines(
    body: &[Stmt],
    line_tokens: &[&Token],
    chars: &[char],
    region_end_col: usize,
    out: &mut String,
) -> Option<()> {
    for (i, item) in body.iter().enumerate() {
        let start = col_to_index(item.pos.column, chars.len());
        let end = match body.get(i + 1) {
            Some(next) => col_to_index(next.pos.column, chars.len()),
            None => col_to_index(region_end_col, chars.len()),
        };
        if if_needs_explosion(item) {
            let item_region_end_col = match body.get(i + 1) {
                Some(next) => next.pos.column,
                None => region_end_col,
            };
            let nested = render_exploded_if(item, line_tokens, chars, item_region_end_col, None)?;
            out.push_str(&nested);
            continue;
        }
        let text: String = chars[start..end.max(start)].iter().collect();
        let text = text.trim();
        let text = text.strip_suffix(':').map(str::trim_end).unwrap_or(text);
        out.push_str(text);
        out.push('\n');
    }
    Some(())
}

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

/// Reindents `source` (BASCAL text, `filename` used both for diagnostic
/// positions the lexer attaches to tokens and, when it names a real file
/// on disk, to resolve its `require`/`import`ed libraries for
/// declaration-matching casing -- see `declared_names`; `library_dirs`
/// mirrors the CLI's own `-L` flags) and returns the result. Never
/// fails: an unrecognized token sequence just falls back to leaving that
/// line's own indent alone, since a formatter that refuses to run on
/// slightly-unusual-but-valid source is worse than one that quietly
/// leaves a corner case untouched.
pub fn reindent(filename: &str, source: &str, library_dirs: &[PathBuf]) -> String {
    let exploded = explode_multi_statement_lines(filename, source);
    let source = exploded.as_str();
    let tokens = Lexer::new(filename, source).lex();
    let declared = declared_names(filename, source, library_dirs);
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
        let content = respace(raw_line, &toks, words.len() == 1, first_word, &declared);
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
/// normalizing inter-token spacing and (for a reserved word) casing.
/// Every non-keyword token's text is still sliced verbatim from
/// `raw_line` by its real source span, never regenerated from its parsed
/// value, so a string's exact quoting or a number's exact digits can
/// never drift.
///
/// `is_solitary_keyword`/`first_word` identify a bare `wend` or `loop`
/// (the only content on the line besides an optional trailing comment)
/// so its own token text can be swapped for the `end while`/`end do`
/// form every other block already uses -- `loop while`/`loop until`
/// has no such equivalent and is left alone.
fn respace(
    raw_line: &str,
    toks: &[&Token],
    is_solitary_keyword: bool,
    first_word: &str,
    declared: &HashMap<NameKey, String>,
) -> String {
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
                let text = match first_word {
                    "wend" => "end while",
                    "loop" => "end do",
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

    // A token's rendered text: the legacy `wend`/`loop` replacement takes
    // priority; otherwise a reserved word is lowercased, and anything
    // else is sliced verbatim from the source (see this fn's own doc
    // comment for why: a string/comment/number's exact text must never
    // be regenerated).
    let render = |i: usize| -> String {
        if let TokenKind::Ident(name) = &real[i].kind {
            let lower = name.to_ascii_lowercase();
            if is_keyword_word(&lower) {
                return lower;
            }
            let ident = BasicIdent::parse(name);
            let base_lower = ident.name.to_ascii_lowercase();
            if is_builtin_word(&base_lower) {
                return BasicIdent {
                    name: base_lower,
                    suffix: ident.suffix,
                }
                .as_basic();
            }
            if let Some(canonical) = declared.get(&(base_lower, ident.suffix)) {
                return canonical.clone();
            }
        }
        let (s, e) = spans[i];
        chars[s..e].iter().collect()
    };

    let mut out = String::new();
    for i in 0..real.len() {
        if let Some((idx, text)) = legacy_replacement {
            if idx == i {
                out.push_str(text);
            } else {
                out.push_str(&render(i));
            }
        } else {
            out.push_str(&render(i));
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
/// See `reindent` for what `library_dirs` is for.
pub fn check(filename: &str, source: &str, library_dirs: &[PathBuf]) -> Vec<Diff> {
    let formatted = reindent(filename, source, library_dirs);
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
        let once = reindent("test.bcl", source, &[]);
        let twice = reindent("test.bcl", &once, &[]);
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
        assert_eq!(reindent("test.bcl", source, &[]), expected);
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
        assert_eq!(reindent("test.bcl", source, &[]), expected);
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
        assert_eq!(reindent("test.bcl", source, &[]), expected);
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
        assert_eq!(reindent("test.bcl", source, &[]), expected);
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
        assert_eq!(reindent("test.bcl", source, &[]), expected);
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
        assert_eq!(reindent("test.bcl", source, &[]), expected);
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
        assert_eq!(reindent("test.bcl", source, &[]), expected);
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
        assert_eq!(reindent("test.bcl", source, &[]), expected);
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
        assert_eq!(reindent("test.bcl", source, &[]), expected);
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
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn trailing_whitespace_is_stripped_everywhere() {
        // Including inside a multi-line block comment's own interior,
        // which stays otherwise untouched (see
        // multi_line_block_comment_interior_is_left_alone above).
        let source = "function f%()   \nprint 1  \n/* a comment   \n   more   \n*/\nend function\n";
        let expected = "function f%()\n    print 1\n    /* a comment\n   more\n*/\nend function\n";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn comma_gets_no_space_before_and_one_space_after() {
        let source = "function f%()\nprint a% ,b% , c%\nend function\n";
        let expected = "function f%()\n    print a%, b%, c%\nend function\n";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn always_binary_operators_get_padded_both_sides() {
        let source = "function f%()\nx%=1*2\nif x%<>3 and x%<=4 then print x%\nend function\n";
        let expected = "function f%()\n    x% = 1 * 2\n    if x% <> 3 and x% <= 4 then print x%\nend function\n";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
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
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn multiple_spaces_between_tokens_collapse_to_one() {
        let source = "function f%()\nprint    \"a\"   ;    \"b\"\nend function\n";
        let expected = "function f%()\n    print \"a\" ; \"b\"\nend function\n";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn bare_wend_becomes_end_while_lowercased_and_keeps_the_trailing_comment() {
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
    while x% > 0
        x% = x% - 1
    end while ' done
end function
";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
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
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn check_reports_only_the_differing_lines() {
        let source = "function f%()\n  print 1\nend function\n";
        let diffs = check("test.bcl", source, &[]);
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
        assert!(check("test.bcl", source, &[]).is_empty());
    }

    #[test]
    fn reserved_words_are_lowercased_regardless_of_original_casing() {
        let source = "\
PROGRAM p
FUNCTION f%()
IF x% = 1 THEN
PRINT \"one\"
ELSEIF x% = 2 THEN
PRINT \"two\"
ELSE
PRINT \"other\"
END IF
RETURN 0
END FUNCTION
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
    return 0
end function
";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn builtin_function_names_are_lowercased_too() {
        // `Len`/`Mid$` are builtin *functions*, resolved by name rather
        // than by the parser's own reserved-word grammar -- a separate
        // rule from keyword casing, but they get the same lowercase
        // treatment (real BASIC intrinsics can never be shadowed by a
        // user function, so there's no ambiguity in doing so).
        let source = "function f%()\nx% = Len(a$)\ny$ = Mid$(a$, 1, 2)\nend function\n";
        let expected =
            "function f%()\n    x% = len(a$)\n    y$ = mid$(a$, 1, 2)\nend function\n";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn plain_identifiers_are_never_mistaken_for_keywords() {
        let source = "function f%()\nBase% = 1\nprint Base%\nend function\n";
        let expected = "function f%()\n    Base% = 1\n    print Base%\nend function\n";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn variable_uses_are_recased_to_match_their_dim() {
        let source = "\
function f%()
dim myCount%
MYCOUNT% = MYCOUNT% + 1
print mycount%
end function
";
        let expected = "\
function f%()
    dim myCount%
    myCount% = myCount% + 1
    print myCount%
end function
";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn function_call_sites_are_recased_to_match_the_declaration() {
        let source = "\
function computeTotal%(n%)
return n% * 2
end function

program p
print COMPUTETOTAL%(3)
";
        let expected = "\
function computeTotal%(n%)
    return n% * 2
end function

program p
print computeTotal%(3)
";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn record_type_references_are_never_recased_even_when_wrong() {
        // `record Header` alongside `file header as Header = open(...)`
        // -- a type name and a case-insensitively-same variable name --
        // is legal, real corpus code (examples/card_catalog/
        // card_catalog.bcl). This formatter used to fold both into one
        // declared-name map, so whichever was recorded first silently
        // overwrote the other's own casing everywhere it was used.
        // Record type names are their own, deliberately untouched
        // namespace now (see this module's own doc comment) -- a
        // mismatched reference like `as HEADER` below stays exactly as
        // written, even though a variable in the same spot would get
        // corrected (`MYCOUNT%` below does).
        let source = "\
record Header
    id: integer
end record

function f%()
dim myCount%
MYCOUNT% = 1
dim s as HEADER
return 0
end function
";
        let expected = "\
record Header
    id: integer
end record

function f%()
    dim myCount%
    myCount% = 1
    dim s as HEADER
    return 0
end function
";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn declaration_matching_is_skipped_gracefully_for_a_file_that_fails_to_parse() {
        // A syntax error must never make the formatter itself fail --
        // every other rule (here, indentation and keyword casing) still
        // applies, just without any declaration-matching casing.
        let source = "function f%(\nPRINT myVar%\n";
        let out = reindent("test.bcl", source, &[]);
        assert!(out.contains("print myVar%"));
    }

    #[test]
    fn a_variable_sharing_a_record_types_own_name_leaves_both_untouched() {
        // The exact shape that broke this formatter for real:
        // `record Header` and a `file header as Header = open(...)`
        // variable sharing one case-insensitive name. Even the record's
        // *own* declaration line (`record Header`) used to get corrupted
        // here, because its type-name token is otherwise indistinguishable
        // from an ordinary identifier at render time and the variable's
        // map entry (whichever of the two happened to be recorded first)
        // silently won everywhere. Now the colliding name is dropped from
        // the declared-name map entirely, so every occurrence -- the
        // record's own declaration, the `file` declaration, and the type
        // reference inside it -- stays exactly as written.
        let source = "\
record Header
    id: integer
end record

function f%()
file header as Header = open(\"x\")
return 0
end function
";
        let expected = "\
record Header
    id: integer
end record

function f%()
    file header as Header = open(\"x\")
    return 0
end function
";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn declaration_matching_casing_reaches_into_a_required_librarys_own_file() {
        // Declaration-matching casing must not stop at this file's own
        // border: a call site naming a function declared in a `require`d
        // library should be recased to match *that* declaration, not
        // left alone just because the declaration itself lives
        // elsewhere. This only works when `filename` names a real file
        // on disk (see `load_merged_program`) -- unlike every other test
        // in this module, which passes a placeholder path and so never
        // exercises this at all.
        let dir = std::env::temp_dir().join(format!(
            "bcc_format_test_{}_{}",
            std::process::id(),
            line!()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("greeter.bcl"),
            "library greeter\n\nfunction greet$(name$)\n    return \"Hello, \" + name$\nend function\n",
        )
        .unwrap();
        let root_path = dir.join("root.bcl");
        let source = "program p\nrequire greeter\n\nprint GREET$(\"world\")\n";
        std::fs::write(&root_path, source).unwrap();

        let formatted = reindent(root_path.to_str().unwrap(), source, &[]);

        std::fs::remove_dir_all(&dir).ok();

        assert!(
            formatted.contains("greet$(\"world\")"),
            "expected the call site recased to match greeter.bcl's own \
             declaration, got:\n{formatted}"
        );
    }

    #[test]
    fn single_line_if_with_a_colon_chained_then_body_explodes_to_block_form() {
        let source = "\
function f%()
if x% > 0 then y% = 1 : z% = 2
return 0
end function
";
        let expected = "\
function f%()
    if x% > 0 then
        y% = 1
        z% = 2
    end if
    return 0
end function
";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn single_line_if_else_both_multi_statement_explodes_with_comment_on_the_header() {
        let source = "\
function f%()
if x% > 0 then y% = 1 : z% = 2 else a% = 3 : b% = 4 ' note
return 0
end function
";
        let expected = "\
function f%()
    if x% > 0 then ' note
        y% = 1
        z% = 2
    else
        a% = 3
        b% = 4
    end if
    return 0
end function
";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn nested_single_line_if_cascades_the_outer_if_into_block_form_too() {
        // The outer `if`'s own then-body has exactly one statement (the
        // inner `if`) -- not itself "multi-statement" by a flat body-length
        // check -- but it can't stay single-line once the inner one is
        // forced onto multiple lines, since single-line-if syntax has no
        // way to hold a multi-line body.
        let source = "\
function f%()
if a% = 1 then if b% = 2 then y% = 1 : z% = 2 else p% = 1
return 0
end function
";
        let expected = "\
function f%()
    if a% = 1 then
        if b% = 2 then
            y% = 1
            z% = 2
        else
            p% = 1
        end if
    end if
    return 0
end function
";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn a_multi_statement_single_line_if_colon_chained_with_a_sibling_splits_both() {
        // `stmt : if cond then a:b` -- the `if` shares its own physical
        // line with an unrelated sibling statement at the same nesting
        // level. The sibling splits onto its own line same as any other
        // multi-statement split, and the `if` explodes into block form
        // same as it would on its own line.
        let source = "\
function f%()
x% = 0 : if y% > 0 then p% = 1 : q% = 2
return 0
end function
";
        let expected = "\
function f%()
    x% = 0
    if y% > 0 then
        p% = 1
        q% = 2
    end if
    return 0
end function
";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn a_plain_top_level_colon_chain_splits_one_statement_per_line() {
        let source = "\
function f%()
x% = 1 : y% = 2 : z% = 3
return 0
end function
";
        let expected = "\
function f%()
    x% = 1
    y% = 2
    z% = 3
    return 0
end function
";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn a_trailing_comment_on_a_plain_colon_chain_stays_with_the_last_statement() {
        let source = "\
function f%()
x% = 1 : y% = 2 ' note
return 0
end function
";
        let expected = "\
function f%()
    x% = 1
    y% = 2 ' note
    return 0
end function
";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn a_leading_label_stays_attached_to_the_statement_it_guards_even_when_splitting() {
        // The common, deliberate `label: statement` idiom must never be
        // torn apart into a bare label line plus a separate statement
        // line just because the *rest* of the line needs splitting.
        let source = "\
function f%()
myLabel: x% = 1 : y% = 2
return 0
end function
";
        let expected = "\
function f%()
    myLabel: x% = 1
    y% = 2
    return 0
end function
";
        assert_eq!(reindent("test.bcl", source, &[]), expected);
        assert_idempotent(source);
    }

    #[test]
    fn an_already_block_form_if_with_a_multi_statement_body_is_never_mistaken_for_single_line() {
        // The exact bug this rule had to avoid: an ordinary block `if`
        // can have any number of statements in its body -- that's not
        // "single-line if with a colon-chained body" just because the
        // body's length happens to be more than one. Must be a no-op,
        // and specifically must not corrupt already-correct block form on
        // a second pass (this is what idempotency alone doesn't catch,
        // since the bug this pins was already stable after one pass).
        let source = "\
function f%()
    if x% > 0 then
        y% = 1
        z% = 2
    end if
    return 0
end function
";
        assert_eq!(reindent("test.bcl", source, &[]), source);
    }

    #[test]
    fn a_trailing_multiline_block_comment_on_an_exploding_if_is_left_untouched() {
        // A multi-line `/* ... */` comment trailing the `if` bails out
        // of explosion entirely (see `render_exploded_if`'s own doc
        // comment) -- splitting one deliberately written to span lines
        // is more likely to mangle a hand-aligned layout than respect
        // it. The comment's own interior indentation is a pre-existing,
        // unrelated formatter limitation when a block comment doesn't
        // start its own line (see `verbatim_until`'s `toks.first()`
        // check in `reindent`) -- not asserted on here.
        let source = "\
function f%()
if x% > 0 then y% = 1 : z% = 2 /* a
   multi-line comment */
return 0
end function
";
        let formatted = reindent("test.bcl", source, &[]);
        assert!(
            formatted.contains("if x% > 0 then y% = 1 : z% = 2 /* a"),
            "the if should not have been exploded, got:\n{formatted}"
        );
    }

    #[test]
    fn a_comma_joined_multi_name_dim_is_never_mistaken_for_a_colon_chain() {
        // `dim i%, total%` desugars into two sibling `Statement::Dim`
        // nodes sharing one physical line (see `Parser::parse_dim`) --
        // exactly the same AST shape a real `a = 1 : b = 2` colon-chain
        // has. Splitting this on the comma would produce `dim i%,` and
        // a bare `total%` on their own lines, neither of which is valid
        // BASCAL on its own -- a real bug this pins down.
        let source = "\
function f%()
dim i%, total%
return 0
end function
";
        assert_eq!(
            reindent("test.bcl", source, &[]),
            "function f%()\n    dim i%, total%\n    return 0\nend function\n"
        );
        assert_idempotent(source);
    }
}
