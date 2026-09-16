use std::{env, fs, process};

fn main() {
    let mut args = env::args_os();
    let _program = args.next();
    let Some(path) = args.next() else {
        eprintln!("usage: rdgen <grammar.rdg> [--emit-rust]");
        process::exit(2);
    };
    let emit_rust = match args.next().as_deref() {
        None => false,
        Some(flag) if flag == std::ffi::OsStr::new("--emit-rust") => true,
        Some(_) => {
            eprintln!("usage: rdgen <grammar.rdg> [--emit-rust]");
            process::exit(2);
        }
    };
    if args.next().is_some() {
        eprintln!("usage: rdgen <grammar.rdg> [--emit-rust]");
        process::exit(2);
    }
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("rdgen: cannot read {}: {}", path.to_string_lossy(), error);
            process::exit(1);
        }
    };
    match rdgen_grammar::compile(&source) {
        Ok(grammar) if emit_rust => match rdgen_codegen_rust::emit(&grammar) {
            Ok(generated) => print!("{}", generated),
            Err(error) => {
                eprintln!("rdgen: Rust backend: {}", error);
                process::exit(1);
            }
        },
        Ok(grammar) => println!("grammar {}: {} rules", grammar.name, grammar.rules.len()),
        Err(error) => {
            eprintln!("rdgen: {}", error);
            process::exit(1);
        }
    }
}
