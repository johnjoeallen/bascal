use std::{env, fs, process};

fn main() {
    let mut args = env::args_os();
    let _program = args.next();
    let Some(path) = args.next() else {
        eprintln!("usage: rdgen <grammar.rdg> [--emit-rust|--emit-c|--emit-c-parser]");
        process::exit(2);
    };
    let emission = match args.next().as_deref() {
        None => None,
        Some(flag) if flag == std::ffi::OsStr::new("--emit-rust") => Some("rust"),
        Some(flag) if flag == std::ffi::OsStr::new("--emit-c") => Some("c"),
        Some(flag) if flag == std::ffi::OsStr::new("--emit-c-parser") => Some("c-parser"),
        Some(_) => {
            eprintln!("usage: rdgen <grammar.rdg> [--emit-rust|--emit-c|--emit-c-parser]");
            process::exit(2);
        }
    };
    if args.next().is_some() {
        eprintln!("usage: rdgen <grammar.rdg> [--emit-rust|--emit-c|--emit-c-parser]");
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
        Ok(grammar) if emission == Some("rust") => match rdgen_codegen_rust::emit(&grammar) {
            Ok(generated) => print!("{}", generated),
            Err(error) => {
                eprintln!("rdgen: Rust backend: {}", error);
                process::exit(1);
            }
        },
        Ok(grammar) if emission == Some("c") => match rdgen_codegen_c::emit(&grammar) {
            Ok(generated) => print!("{}", generated),
            Err(error) => {
                eprintln!("rdgen: C backend: {}", error);
                process::exit(1);
            }
        },
        Ok(grammar) if emission == Some("c-parser") => match rdgen_codegen_c::emit_parser(&grammar) {
            Ok(generated) => print!("{}", generated),
            Err(error) => {
                eprintln!("rdgen: C backend: {}", error);
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
