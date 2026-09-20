//! The generated semantic BASCAL frontend.
//!
//! This module is intentionally parallel to the legacy `parser` module for
//! now.  Its generated AST is the source-span-aware semantic frontend being
//! prepared for the next compiler pipeline boundary; converting it to the
//! legacy `ast::Program` would discard that information and would therefore
//! be the wrong integration point.

// The generated recursive-descent control flow intentionally uses compact
// closures and parser-position rewinds that Clippy's source-oriented lints
// cannot distinguish from handwritten mistakes.
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/rdgen_bascal_parser.rs"));

/// Parse BASCAL source with the generated semantic frontend.
pub fn parse(source: &str) -> Result<Program, ParseError> {
    Parser::with_lexical_config(source, scanner, trivia, literal).parse()
}

/// Scan the terminal classes declared in `bascal.bcl.rdg`.
fn scanner(source: &str, position: usize, name: &str) -> Option<(Token, usize)> {
    let character = source.get(position..)?.chars().next()?;
    let accepted = match name {
        "letter" => character.is_ascii_alphabetic(),
        "digit" => character.is_ascii_digit(),
        "hex_digit" => character.is_ascii_hexdigit(),
        "any_char" => true,
        "any_char_except_quote" => character != '"',
        "any_char_except_newline" => character != '\n',
        _ => false,
    };
    accepted.then(|| (Token(character.to_string()), position + character.len_utf8()))
}

/// Skip lexical trivia while leaving line-oriented grammar elements to the
/// generated parser's newline-aware primitives.
fn trivia(source: &str, mut position: usize) -> usize {
    while source
        .get(position..)
        .and_then(|rest| rest.chars().next())
        .is_some_and(char::is_whitespace)
    {
        position += source[position..].chars().next().unwrap().len_utf8();
    }
    position
}

/// BASCAL keywords are case-insensitive, matching the legacy lexer/parser.
fn literal(source: &str, position: usize, expected: &str) -> Option<usize> {
    source
        .get(position..)?
        .get(..expected.len())
        .filter(|candidate| candidate.eq_ignore_ascii_case(expected))
        .map(|_| position + expected.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_source_with_the_build_generated_frontend() {
        let program = parse("total% = -2 ^ 2\nprint total%\n").unwrap();
        let Program::File { items, .. } = program;
        assert_eq!(items.len(), 2);
    }
}
