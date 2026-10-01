//! Rewrites `codegen_c.rs`'s own C99-shaped "mixed declarations and code"
//! (a local variable declared after an earlier statement in the same
//! block) into C89-legal form, for a `CDialectProfile` whose target
//! compiler doesn't accept it -- `cc65` is exactly such a compiler
//! (confirmed by hand: `printf(...); int x = 1;` inside one block fails
//! with "Error: Expression expected" / "';' expected" / "Undefined
//! symbol: 'x'" -- see `CDialectProfile::supports_mixed_declarations`'s
//! own doc comment). `codegen_c.rs` declares every local (string buffers
//! especially, always "declare the buffer, then `snprintf` into it")
//! exactly where BASIC's own "a variable springs into existence where
//! it's first used" semantics puts it, which is routinely after an
//! earlier statement in the same block -- valid C99, not valid C89.
//!
//! The fix: for every `{ ... }` compound-statement block (a function
//! body, an `if`/`for`/`while` body, or a bare block -- never a `{...}`
//! *initializer list*, distinguished here by checking whether the
//! preceding significant character is `=`), split each of that block's
//! own (not a nested block's) `TYPE name = expr;` declarations into a
//! bare `TYPE name;` hoisted to immediately after the opening `{`, plus a
//! plain `name = expr;` assignment left at the original position. This is
//! the standard, semantics-preserving C99-to-C89 transform: declaration
//! *order* never affects behavior in C (the storage exists from block
//! entry either way), only assignment *timing* does, and that's exactly
//! what's preserved by leaving the assignment where the declaration used
//! to be. A declaration with no initializer (`char buf[256];`, every
//! local array this backend emits) is simply hoisted whole, nothing left
//! behind.
//!
//! Whenever a line doesn't cleanly match one of this backend's own known
//! declaration shapes, it's left completely untouched rather than guessed
//! at -- the safe failure mode is `cl65` reporting its original "mixed
//! declarations" error for that one case, never a silent miscompile.
//!
//! Runs once, as a whole-file post-pass over `codegen_c::generate`'s
//! output, only when the active `CDialectProfile` needs it -- never for
//! `Target::C`'s own `host_gcc` profile, which supports C99 mixed
//! declarations exactly like every output `codegen_c.rs` produced before
//! this module existed.

/// Type keywords this backend ever declares a *local* (block-scoped, not
/// `static` file-scope -- statics are always emitted outside any `{ }`,
/// so this module never sees them) variable with. `float`/`double` are
/// included for completeness even though Phase 3's capability validation
/// means no `supports_mixed_declarations: false` profile (`cc65`'s) can
/// ever actually reach a `float`/`double` local in *program* code -- but
/// this backend's own fixed runtime-helper bodies (`size_t`/`int16_t`/
/// `int32_t` locals inside e.g. `bcc_read_file_field`/`bcc_mki`) are
/// exactly as subject to the mixed-declarations problem as anything this
/// backend generates per-program, and this list has to cover those too
/// (confirmed by hand: omitting `size_t` here left a real `bcc_read_file_
/// field` declaration unhoisted, reproducing the original bug for that
/// one helper). Checked longest-first so `"unsigned char"` never
/// partially matches as bare `"char"` first.
const LOCAL_DECL_KEYWORDS: &[&str] = &[
    "unsigned char",
    "double",
    "float",
    "size_t",
    "int16_t",
    "int32_t",
    "uint16_t",
    "uint32_t",
    "long",
    "char",
    "int",
];

pub(crate) fn hoist_declarations_for_c89(source: &str) -> String {
    transform(source)
}

enum Segment {
    /// `leading` (whitespace and any `//`/`/* */` comments right before
    /// the declaration -- e.g. a comment explaining *why* the variable
    /// exists) always stays at the original position, never hoisted:
    /// only `hoisted` (the bare declaration) moves to the block's top.
    Decl {
        leading: String,
        hoisted: String,
        inline: String,
    },
    Other(String),
}

/// Transforms one scope's worth of C source -- the whole file on the
/// initial call, and (recursively, for every nested `{ ... }` block found
/// within it) each block's own interior.
fn transform(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut segments: Vec<Segment> = Vec::new();
    let mut seg_start = 0usize;
    let mut paren_depth: i32 = 0;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => i = skip_string(bytes, i),
            b'\'' => i = skip_char(bytes, i),
            b'/' if bytes.get(i + 1) == Some(&b'/') => i = skip_line_comment(bytes, i),
            b'/' if bytes.get(i + 1) == Some(&b'*') => i = skip_block_comment(bytes, i),
            b'(' => {
                paren_depth += 1;
                i += 1;
            }
            b')' => {
                paren_depth -= 1;
                i += 1;
            }
            b'{' if paren_depth == 0 && !preceded_by_equals(text, seg_start, i) => {
                // A compound-statement block, not an initializer list --
                // recurse into its interior, then treat the whole
                // "control-construct prefix + { transformed interior }"
                // as one opaque unit at this scope's own level (it never
                // gets split for declaration hoisting itself; only a
                // `;`-terminated statement can be a declaration).
                let close = find_matching_brace(bytes, i).unwrap_or(bytes.len().saturating_sub(1));
                let inner = transform(&text[i + 1..close]);
                let mut block = String::with_capacity(text[seg_start..i].len() + inner.len() + 2);
                block.push_str(&text[seg_start..i]);
                block.push('{');
                block.push_str(&inner);
                block.push('}');
                segments.push(Segment::Other(block));
                i = close + 1;
                seg_start = i;
            }
            b'{' => {
                // An initializer-list brace (preceded by '=') -- opaque,
                // skip past its matching close without recursing or
                // treating it as a statement boundary at all.
                let close = find_matching_brace(bytes, i).unwrap_or(bytes.len().saturating_sub(1));
                i = close + 1;
            }
            b';' if paren_depth == 0 => {
                let stmt = &text[seg_start..=i];
                segments.push(classify_statement(stmt));
                i += 1;
                seg_start = i;
            }
            _ => i += 1,
        }
    }
    if seg_start < text.len() {
        segments.push(Segment::Other(text[seg_start..].to_string()));
    }

    // Reassemble: every hoisted bare declaration first, in original
    // order, then every segment's own in-place content (an `Other`
    // segment verbatim, a `Decl` segment's inline assignment if it had an
    // initializer, or nothing if it didn't).
    let mut out = String::with_capacity(text.len() + 64);
    for seg in &segments {
        if let Segment::Decl { hoisted, .. } = seg {
            out.push_str(hoisted);
        }
    }
    for seg in &segments {
        match seg {
            Segment::Decl { leading, inline, .. } => {
                out.push_str(leading);
                out.push_str(inline);
            }
            Segment::Other(text) => out.push_str(text),
        }
    }
    out
}

fn preceded_by_equals(text: &str, seg_start: usize, brace_pos: usize) -> bool {
    text[seg_start..brace_pos].trim_end().ends_with('=')
}

/// How far into `bytes` (from its start) leading whitespace *and*
/// `//`/`/* */` comments extend -- unlike a plain `str::trim_start`,
/// which only skips whitespace. Without this, a declaration preceded by
/// a comment (with no `;` inside the comment itself to end the previous
/// segment early) has its comment text merged into the *same* captured
/// span as the declaration; checking that combined span's prefix against
/// `LOCAL_DECL_KEYWORDS` then fails (it starts with `//`, not `char`/
/// `int`/...), so the declaration silently falls through to `Other` and
/// never gets hoisted at all -- confirmed by hand as the exact cause of
/// a real cc65 compile failure in `tutorial/files.bcl` (a `char
/// bt_s_0[256];` declaration preceded by a `//` comment stayed in place,
/// after an earlier statement in the same block).
fn skip_leading_trivia(bytes: &[u8]) -> usize {
    let mut i = 0;
    loop {
        while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\n' | b'\r') {
            i += 1;
        }
        if bytes.get(i) == Some(&b'/') && bytes.get(i + 1) == Some(&b'/') {
            i = skip_line_comment(bytes, i);
            continue;
        }
        if bytes.get(i) == Some(&b'/') && bytes.get(i + 1) == Some(&b'*') {
            i = skip_block_comment(bytes, i);
            continue;
        }
        break;
    }
    i
}

/// Classifies one `;`-terminated statement span (including its own
/// leading whitespace/comments, since `transform` captures spans that
/// way) as a hoistable local declaration or as ordinary code. `stmt`
/// always ends with `;` by construction.
fn classify_statement(stmt: &str) -> Segment {
    let content_start = skip_leading_trivia(stmt.as_bytes());
    let leading = &stmt[..content_start];
    let trimmed = &stmt[content_start..];
    // `indent`: whitespace-only tail of `leading`, reused as the hoisted
    // declaration's own indentation -- consistent even when `leading`
    // also carries a preceding comment.
    let indent_len = leading.len()
        - leading
            .trim_end_matches(|c: char| c == ' ' || c == '\t')
            .len();
    let indent = &leading[leading.len() - indent_len..];

    for &kw in LOCAL_DECL_KEYWORDS {
        let Some(after_kw) = trimmed.strip_prefix(kw) else {
            continue;
        };
        if !after_kw.starts_with(|c: char| c.is_whitespace()) {
            continue;
        }
        let after_kw = after_kw.trim_start();
        let Some(body) = after_kw.strip_suffix(';') else {
            continue;
        };

        let ident_end = body
            .find(|c: char| c == '[' || c == '=' || c.is_whitespace())
            .unwrap_or(body.len());
        if ident_end == 0 || !is_c_ident(&body[..ident_end]) {
            continue;
        }
        let name = &body[..ident_end];

        let mut rest = &body[ident_end..];
        let mut has_dims = false;
        while let Some(after_bracket) = rest.strip_prefix('[') {
            let Some(close) = after_bracket.find(']') else {
                break;
            };
            has_dims = true;
            rest = &after_bracket[close + 1..];
        }
        let rest = rest.trim_start();

        let Some(init) = rest.strip_prefix('=') else {
            // No initializer (every local array this backend emits, plus
            // any bare scalar declaration) -- hoist the bare declaration
            // (never `leading`: a comment explaining it stays at the
            // original position), nothing left behind at that position.
            return Segment::Decl {
                leading: leading.to_string(),
                hoisted: format!("{trimmed}\n"),
                inline: String::new(),
            };
        };
        if has_dims {
            // A local array WITH an initializer never occurs in this
            // backend's own output today (every local array is declared
            // bare, then populated via `snprintf`/a copy loop) -- hoisting
            // the whole thing verbatim is still correct as long as the
            // initializer is a compile-time constant with no dependency
            // on an earlier statement in this block, which is the only
            // shape C itself allows here anyway.
            return Segment::Decl {
                leading: leading.to_string(),
                hoisted: format!("{trimmed}\n"),
                inline: String::new(),
            };
        }

        // `has_dims` is always `false` here (the `if has_dims` branch
        // above already returned for the array case), so the declared
        // name alone -- no `[...]` suffix -- is the whole bare
        // declaration. `inline` doesn't repeat `indent`: `leading`
        // already ends with that same whitespace run, immediately before
        // where `inline` is concatenated onto it (see `transform`'s
        // reassembly).
        let init_expr = init.trim();
        return Segment::Decl {
            leading: leading.to_string(),
            hoisted: format!("{indent}{kw} {name};\n"),
            inline: format!("{name} = {init_expr};"),
        };
    }
    Segment::Other(stmt.to_string())
}

fn is_c_ident(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn find_matching_brace(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut i = open;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i = skip_string(bytes, i);
                continue;
            }
            b'\'' => {
                i = skip_char(bytes, i);
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                i = skip_line_comment(bytes, i);
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = skip_block_comment(bytes, i);
                continue;
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn skip_string(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return i + 1,
            _ => i += 1,
        }
    }
    i.min(bytes.len())
}

fn skip_char(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'\'' => return i + 1,
            _ => i += 1,
        }
    }
    i.min(bytes.len())
}

fn skip_line_comment(bytes: &[u8], start: usize) -> usize {
    let mut i = start;
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    i
}

fn skip_block_comment(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 2;
    while i + 1 < bytes.len() {
        if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            return i + 2;
        }
        i += 1;
    }
    bytes.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real bug, found by hand-compiling `tutorial/files.bcl` under
    /// `cl65`: a declaration preceded by a `//` comment (with no `;`
    /// inside the comment to end the previous segment early) had its
    /// comment text merged into the *same* captured span as the
    /// declaration -- `classify_statement` then failed to recognize it
    /// (the span starts with `//`, not a type keyword) and left it
    /// un-hoisted, reproducing the exact "mixed declarations" cc65 error
    /// this whole module exists to fix. See `skip_leading_trivia`'s own
    /// doc comment for the root cause.
    #[test]
    fn hoists_a_declaration_preceded_by_a_comment() {
        let input = "int main(void) {\n    // (produces data that input # can read back)\n\n    char bt_s_0[256];\n    snprintf(bt_s_0, sizeof(bt_s_0), \"%s\", \"x\");\n    char bt_s_1[256];\n    snprintf(bt_s_1, sizeof(bt_s_1), \"%s\", bt_s_0);\n}\n";
        let output = hoist_declarations_for_c89(input);
        let brace = output.find('{').unwrap();
        let decl = output.find("char bt_s_0[256];").unwrap();
        let comment_pos = output.find("// (produces").unwrap();
        assert!(
            decl > brace && decl < comment_pos,
            "bt_s_0 should hoist before the comment that used to swallow it:\n{output}"
        );
        // The comment itself must still appear, at its original position
        // (never hoisted, never dropped).
        assert!(output.contains("// (produces data that input # can read back)"));
    }

    #[test]
    fn hoists_a_scalar_declaration_after_a_statement() {
        let input = "int main(void) {\n    printf(\"before\\n\");\n    int x = 5;\n    printf(\"%d\\n\", x);\n    return 0;\n}\n";
        let output = hoist_declarations_for_c89(input);
        let decl_pos = output.find("int x;").expect("bare declaration hoisted");
        let brace_pos = output.find('{').unwrap();
        let printf_pos = output.find("printf(\"before").unwrap();
        assert!(
            decl_pos > brace_pos && decl_pos < printf_pos,
            "declaration should be hoisted before the first statement:\n{output}"
        );
        assert!(output.contains("x = 5;"), "{output}");
        assert!(
            !output.contains("int x = 5;"),
            "the original inline initializer form should be gone:\n{output}"
        );
    }

    #[test]
    fn hoists_a_string_buffer_declared_after_a_statement() {
        let input = "int main(void) {\n    printf(\"x\\n\");\n    char buf[256];\n    snprintf(buf, sizeof(buf), \"%s\", \"hi\");\n}\n";
        let output = hoist_declarations_for_c89(input);
        let decl_pos = output.find("char buf[256];").unwrap();
        let printf_pos = output.find("printf(\"x").unwrap();
        assert!(decl_pos < printf_pos, "{output}");
        assert!(output.contains("snprintf(buf"), "{output}");
    }

    #[test]
    fn recurses_into_nested_blocks_independently() {
        let input = "int main(void) {\n    if (1) {\n        printf(\"a\\n\");\n        int y = 2;\n        printf(\"%d\\n\", y);\n    }\n    int x = 1;\n    printf(\"%d\\n\", x);\n}\n";
        let output = hoist_declarations_for_c89(input);
        // Outer block's `x` hoists to right after the outer `{`, ahead of
        // the whole `if` block -- the nested block's own `y` hoists
        // independently, inside the `if`'s own braces.
        let outer_brace = output.find('{').unwrap();
        let x_decl = output.find("int x;").unwrap();
        let if_pos = output.find("if (1) {").unwrap();
        assert!(outer_brace < x_decl && x_decl < if_pos, "{output}");

        let if_brace = output[if_pos..].find('{').unwrap() + if_pos;
        let y_decl = output.find("int y;").unwrap();
        let a_pos = output.find("printf(\"a").unwrap();
        assert!(if_brace < y_decl && y_decl < a_pos, "{output}");
    }

    #[test]
    fn still_produces_valid_c89_when_the_declaration_was_already_first_in_its_block() {
        // The transform doesn't special-case "already legal" -- every
        // scalar declaration with an initializer is split uniformly. That's
        // still correct C89 (just not the minimal edit), which is all that
        // matters for a "DO NOT EDIT, always regenerated" intermediate file.
        let input = "int main(void) {\n    int x = 1;\n    printf(\"%d\\n\", x);\n}\n";
        let output = hoist_declarations_for_c89(input);
        assert!(output.contains("int x;"), "{output}");
        assert!(output.contains("x = 1;"), "{output}");
        assert!(!output.contains("int x = 1;"), "{output}");
    }

    #[test]
    fn does_not_treat_an_initializer_list_brace_as_a_block() {
        let input = "static char bv_s_x[256] = {0};\n\nint main(void) {\n    printf(\"hi\\n\");\n    return 0;\n}\n";
        let output = hoist_declarations_for_c89(input);
        assert!(output.contains("static char bv_s_x[256] = {0};"), "{output}");
    }

    #[test]
    fn does_not_misparse_braces_inside_string_literals() {
        let input = "int main(void) {\n    printf(\"{not a block}\\n\");\n    int x = 1;\n    printf(\"%d\\n\", x);\n}\n";
        let output = hoist_declarations_for_c89(input);
        assert!(output.contains("printf(\"{not a block}\\n\");"), "{output}");
        assert!(output.contains("int x;"), "{output}");
    }

    #[test]
    fn leaves_a_function_definition_header_alone() {
        let input = "static int helper(int a) {\n    printf(\"x\\n\");\n    int y = a;\n    return y;\n}\n";
        let output = hoist_declarations_for_c89(input);
        assert!(
            output.starts_with("static int helper(int a) {    int y;\n"),
            "{output}"
        );
        assert!(output.contains("y = a;"), "{output}");
        assert!(!output.contains("int y = a;"), "{output}");
    }

    #[test]
    fn preserves_the_select_case_pattern_from_the_real_bug_report() {
        let input = "    {\n        char bt_sel_2[256];\n        snprintf(bt_sel_2, sizeof(bt_sel_2), \"%s\", bv_s_day);\n        int bt_sel_match_3 = 0;\n        if (!bt_sel_match_3) {\n            if (1) {\n                bt_sel_match_3 = 1;\n                char bt_s_4[256];\n                snprintf(bt_s_4, sizeof(bt_s_4), \"%s\", \"x\");\n            }\n        }\n    }\n";
        let output = hoist_declarations_for_c89(input);
        assert!(output.contains("int bt_sel_match_3;"), "{output}");
        assert!(output.contains("bt_sel_match_3 = 0;"), "{output}");
        assert!(!output.contains("int bt_sel_match_3 = 0;"), "{output}");
        assert!(output.contains("char bt_s_4[256];"), "{output}");
    }
}
