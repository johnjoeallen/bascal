use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use bcc::{compile_file, default_output_path, CompileOptions};

#[derive(Debug)]
struct Cli {
    input: PathBuf,
    output: Option<PathBuf>,
    library_dirs: Vec<PathBuf>,
    libraries: Vec<String>,
    line_numbers: bool,
    clean: bool,
    binary: bool,
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

    let output_path = cli
        .output
        .clone()
        .unwrap_or_else(|| default_output_path(cli.input.as_path()));

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "error: failed to create output directory {}: {err}",
                parent.display()
            )
        })?;
    }

    if !cli.clean && is_up_to_date(&cli.input, &output_path) {
        let binary_path = PathBuf::from("tmp").join(output_path.file_stem().ok_or_else(|| {
            format!("error: invalid BASIC output path {}", output_path.display())
        })?);
        if cli.binary && !is_up_to_date(&cli.input, &binary_path) {
            return invoke_fbc(&output_path);
        }
        println!("up to date: {}", output_path.display());
        return Ok(());
    }

    let options = CompileOptions {
        library_dirs: cli.library_dirs,
        libraries: cli.libraries,
        line_numbers: cli.line_numbers,
    };
    let basic = compile_file(&cli.input, &options).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    fs::write(&output_path, &basic)
        .map_err(|err| format!("error: failed to write {}: {err}", output_path.display()))?;

    if cli.binary {
        invoke_fbc(&output_path)?;
    }

    Ok(())
}

fn is_up_to_date(input: &PathBuf, output: &PathBuf) -> bool {
    let Ok(in_meta) = fs::metadata(input) else {
        return false;
    };
    let Ok(out_meta) = fs::metadata(output) else {
        return false;
    };
    let Ok(in_mtime) = in_meta.modified() else {
        return false;
    };
    let Ok(out_mtime) = out_meta.modified() else {
        return false;
    };
    out_mtime >= in_mtime
}

fn invoke_fbc(bas_path: &PathBuf) -> Result<(), String> {
    let binary_name = bas_path
        .file_stem()
        .ok_or_else(|| format!("error: invalid BASIC output path {}", bas_path.display()))?;
    let binary_dir = PathBuf::from("tmp");
    fs::create_dir_all(&binary_dir)
        .map_err(|err| format!("error: failed to create {}: {err}", binary_dir.display()))?;
    let binary_path = binary_dir.join(binary_name);
    let status = Command::new("fbc")
        .arg("-lang")
        .arg("qb")
        .arg(bas_path)
        .arg("-x")
        .arg(&binary_path)
        .status()
        .map_err(|err| format!("error: failed to invoke fbc: {err}"))?;
    if !status.success() {
        return Err(format!(
            "error: fbc failed compiling {}",
            bas_path.display()
        ));
    }
    println!("binary: {}", binary_path.display());
    Ok(())
}

fn parse_args(args: Vec<String>) -> Result<Cli, String> {
    let mut input = None;
    let mut output = None;
    let mut library_dirs = Vec::new();
    let mut libraries = Vec::new();
    let mut line_numbers = false;
    let mut clean = false;
    let mut binary = false;
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
            "--line-numbers" => line_numbers = true,
            "--clean" | "-c" => clean = true,
            "--binary" | "-b" => binary = true,
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
        library_dirs,
        libraries,
        line_numbers,
        clean,
        binary,
    })
}

fn usage() -> String {
    [
        "usage: bcc input.bcl [-o output.bas] [-L dir] [-l library]",
        "              [--line-numbers] [--clean | -c] [--binary | -b]",
        "",
        "Options:",
        "  -o output.bas        Output path (default: input with .bas extension)",
        "  -L dir               Add a library search directory for require resolution",
        "  --line-numbers       Number every output line, not just branch targets",
        "  --clean, -c          Recompile even if the output is already up to date",
        "  --binary, -b         Invoke fbc to compile the generated .bas to tmp/<stem>",
    ]
    .join("\n")
}
