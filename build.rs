//! Generates the rdgen-based BASCAL parser at build time from
//! `rdgen/grammars/bascal.bcl.rdg`, so `src/rdgen_frontend.rs` can
//! `include!` it as an ordinary Rust source file. This is the only place
//! rdgen's own crates are used at all: the shipped `bcc` binary embeds the
//! generated parser, not rdgen itself.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let grammar_path = manifest_dir.join("rdgen/grammars/bascal.bcl.rdg");
    println!("cargo:rerun-if-changed={}", grammar_path.display());

    let source = fs::read_to_string(&grammar_path).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", grammar_path.display());
    });
    let grammar = rdgen_grammar::compile(&source).unwrap_or_else(|err| {
        panic!("failed to compile {}: {err}", grammar_path.display());
    });
    let generated = rdgen_codegen_rust::emit(&grammar).unwrap_or_else(|err| {
        panic!(
            "failed to generate a Rust parser from {}: {err}",
            grammar_path.display()
        );
    });

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_path = out_dir.join("rdgen_bascal_parser.rs");
    fs::write(&out_path, generated)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", out_path.display()));
}
