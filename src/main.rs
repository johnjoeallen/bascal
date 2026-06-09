use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use bcc::{compile_file, default_output_path, CompileOptions};

#[derive(Debug)]
struct Cli {
    input: PathBuf,
    output: Option<PathBuf>,
    include_dirs: Vec<PathBuf>,
    library_dirs: Vec<PathBuf>,
    libraries: Vec<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let cli = parse_args(env::args().skip(1).collect())?;
    let options = CompileOptions {
        include_dirs: cli.include_dirs,
        library_dirs: cli.library_dirs,
        libraries: cli.libraries,
    };
    let output = compile_file(&cli.input, &options).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    let output_path = cli
        .output
        .unwrap_or_else(|| default_output_path(cli.input.as_path()));
    fs::write(&output_path, output)
        .map_err(|err| format!("error: failed to write {}: {err}", output_path.display()))?;

    Ok(())
}

fn parse_args(args: Vec<String>) -> Result<Cli, String> {
    let mut input = None;
    let mut output = None;
    let mut include_dirs = Vec::new();
    let mut library_dirs = Vec::new();
    let mut libraries = Vec::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-o" => {
                i += 1;
                output =
                    Some(PathBuf::from(args.get(i).ok_or_else(|| {
                        "error: -o requires an output path".to_string()
                    })?));
            }
            "-I" => {
                i += 1;
                include_dirs
                    .push(PathBuf::from(args.get(i).ok_or_else(|| {
                        "error: -I requires a directory".to_string()
                    })?));
            }
            "-L" => {
                i += 1;
                library_dirs
                    .push(PathBuf::from(args.get(i).ok_or_else(|| {
                        "error: -L requires a directory".to_string()
                    })?));
            }
            "-l" => {
                i += 1;
                libraries.push(
                    args.get(i)
                        .ok_or_else(|| "error: -l requires a library name".to_string())?
                        .clone(),
                );
            }
            "-h" | "--help" => return Err(usage()),
            flag if flag.starts_with('-') => return Err(format!("error: unknown flag `{flag}`")),
            path => {
                if input.replace(PathBuf::from(path)).is_some() {
                    return Err("error: only one input file is supported".to_string());
                }
            }
        }
        i += 1;
    }

    Ok(Cli {
        input: input.ok_or_else(usage)?,
        output,
        include_dirs,
        library_dirs,
        libraries,
    })
}

fn usage() -> String {
    "usage: bcc input.bcl [-o output.bas] [-I dir] [-L dir] [-l library]".to_string()
}
