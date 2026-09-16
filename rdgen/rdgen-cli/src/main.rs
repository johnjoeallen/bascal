use std::{env, fs, process};

fn main() {
    let mut args = env::args_os();
    let _program = args.next();
    let Some(path) = args.next() else {
        eprintln!("usage: rdgen <grammar.rdg>");
        process::exit(2);
    };
    if args.next().is_some() {
        eprintln!("usage: rdgen <grammar.rdg>");
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
        Ok(grammar) => println!("grammar {}: {} rules", grammar.name, grammar.rules.len()),
        Err(error) => {
            eprintln!("rdgen: {}", error);
            process::exit(1);
        }
    }
}
