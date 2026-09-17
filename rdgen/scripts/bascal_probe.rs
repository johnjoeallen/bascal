include!(env!("RDGEN_GENERATED"));

fn scan_terminal(source: &str, position: usize, name: &str) -> Option<(Token, usize)> {
    let character = source.get(position..)?.chars().next()?;
    let accepted = match name {
        "letter" => character.is_ascii_alphabetic(),
        "digit" | "hex_digit" => character.is_ascii_hexdigit(),
        "any_char" => true,
        "any_char_except_quote" => character != '"',
        "any_char_except_newline" => character != '\n',
        "suffix" => matches!(character, '%' | '$' | '!' | '#' | '&'),
        "escaped_quote" => character == '\\' || character == '"',
        _ => character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '"'),
    };
    accepted.then(|| (Token(character.to_string()), position + character.len_utf8()))
}

fn probe_match_literal(source: &str, position: usize, expected: &str) -> Option<usize> {
    let candidate = source.get(position..)?.get(..expected.len())?;
    candidate.eq_ignore_ascii_case(expected).then_some(position + expected.len())
}

fn skip_trivia(source: &str, mut position: usize) -> usize {
    loop {
        if source
            .get(position..)
            .is_some_and(|rest| rest.starts_with("//") || rest.starts_with('\''))
        {
            position += source[position..]
                .find('\n')
                .unwrap_or(source.len() - position);
            continue;
        }
        if source.get(position..).is_some_and(|rest| rest.starts_with("/*")) {
            position += source[position..]
                .find("*/")
                .map_or(source.len() - position, |end| end + 2);
            continue;
        }
        let Some(character) = source.get(position..).and_then(|rest| rest.chars().next()) else {
            return position;
        };
        if character.is_whitespace() {
            position += character.len_utf8();
        } else {
            return position;
        }
    }
}

fn main() {
    let mut failures = 0;
    for path in std::env::args().skip(1) {
        let source = std::fs::read_to_string(&path).expect("read BASCAL source");
        let mut parser = Parser::with_lexical_config(
            &source,
            scan_terminal,
            skip_trivia,
            probe_match_literal,
        );
        match parser.parse() {
            Ok(_) => println!("OK {path}"),
            Err(error) => {
                failures += 1;
                println!("FAIL {path}: {} at {}", error.message, error.position);
            }
        }
    }
    if failures != 0 {
        std::process::exit(1);
    }
}
