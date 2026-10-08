//! Typed-IR statement coverage of the conformance corpus.
//!
//! Every `SemanticStatementKind` variant must be exercised by at least one
//! fixture in `tests/fixtures/conformance`, so the cross-backend corpus can
//! stand in for the AST when checking that codegen backends consume only the
//! typed IR.  Kinds without a fixture yet are listed in `KNOWN_GAPS`; the list
//! may only shrink: covering a listed kind fails the test until it is removed.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

/// Statement kinds that have no conformance fixture yet.
const KNOWN_GAPS: &[&str] = &[
    "Beep",
    "Clear",
    "Cls",
    "Color",
    "Continue",
    "Data",
    "Do",
    "Erase",
    "Error",
    "Exit",
    "Expression",
    "FileDeclaration",
    "Global",
    "Goto",
    "Input",
    "Kill",
    "LineInput",
    "Locate",
    "Lprint",
    "OnErrorGoto",
    "OptionBase",
    "Out",
    "Poke",
    "Randomize",
    "Read",
    "Rename",
    "Restore",
    "Resume",
    "Seek",
    "SelectCase",
    "Stop",
    "Swap",
    "System",
    "While",
    "Width",
    "Write",
];

/// Kinds that are structural wrappers or parser recovery placeholders rather
/// than language statements.
const NOT_STATEMENTS: &[&str] = &["Line", "Unsupported"];

fn conformance_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/conformance")
}

/// Variant names of `SemanticStatementKind`, read from its declaration so the
/// inventory cannot drift from the typed IR.
fn declared_kinds() -> BTreeSet<String> {
    let source = include_str!("../src/semantic_ir.rs");
    let start = source
        .find("pub enum SemanticStatementKind {")
        .expect("SemanticStatementKind declaration");
    let body = &source[start..];
    let end = body.find("\n}\n").expect("end of SemanticStatementKind");
    body[..end]
        .lines()
        .skip(1)
        .filter(|line| line.starts_with("    ") && !line.starts_with("     "))
        .filter_map(|line| {
            let name: String = line.trim().chars().take_while(|c| c.is_alphanumeric()).collect();
            name.chars().next().filter(|c| c.is_uppercase()).map(|_| name)
        })
        .filter(|name| !NOT_STATEMENTS.contains(&name.as_str()))
        .collect()
}

/// Kinds used anywhere in a fixture's typed IR, including nested bodies and
/// callable bodies.  The derived `Debug` output prints each statement as
/// `kind: Name`, which avoids a hand-written walker that would itself need
/// updating whenever a statement gains a body.
fn used_kinds(source: &str, into: &mut BTreeSet<String>) {
    let module = bcc::semantic_ir::parse_and_adapt(source).expect("fixture parses");
    let mut dump = format!("{:#?}", module.statements);
    for callable in &module.callables {
        dump.push_str(&format!("{:#?}", callable.body));
    }
    for line in dump.lines() {
        if let Some(rest) = line.trim_start().strip_prefix("kind: ") {
            let name: String = rest.chars().take_while(|c| c.is_alphanumeric()).collect();
            into.insert(name);
        }
    }
}

#[test]
fn conformance_corpus_covers_every_statement_kind() {
    let mut used = BTreeSet::new();
    let mut fixtures = 0;
    for entry in fs::read_dir(conformance_dir()).expect("conformance dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().is_some_and(|ext| ext == "bcl") {
            let source = fs::read_to_string(&path).expect("read fixture");
            used_kinds(&source, &mut used);
            fixtures += 1;
        }
    }
    assert!(fixtures > 0, "no conformance fixtures found");

    let declared = declared_kinds();
    assert!(declared.len() > 40, "kind inventory looks truncated: {declared:?}");

    let gaps: BTreeSet<String> = KNOWN_GAPS.iter().map(|s| s.to_string()).collect();
    let uncovered: BTreeSet<_> = declared.difference(&used).cloned().collect();
    let new_gaps: Vec<_> = uncovered.difference(&gaps).collect();
    let stale: Vec<_> = gaps.intersection(&used).collect();
    let unknown: Vec<_> = gaps.difference(&declared).collect();
    assert!(
        new_gaps.is_empty(),
        "statement kinds with no conformance fixture (add one under tests/fixtures/conformance): {new_gaps:?}"
    );
    assert!(
        stale.is_empty(),
        "KNOWN_GAPS lists kinds that are now covered; remove them: {stale:?}"
    );
    assert!(
        unknown.is_empty(),
        "KNOWN_GAPS lists kinds not in SemanticStatementKind: {unknown:?}"
    );
}
