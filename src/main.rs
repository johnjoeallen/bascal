use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use bcc::{compile_file, default_output_path, CompileOptions, Target};

#[derive(Debug)]
struct Cli {
    input: PathBuf,
    output: Option<PathBuf>,
    library_dirs: Vec<PathBuf>,
    libraries: Vec<String>,
    line_numbers: bool,
    sparse_line_numbers: bool,
    clean: bool,
    binary: bool,
    target: Target,
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
        .unwrap_or_else(|| default_output_path(cli.input.as_path(), cli.target));

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
            return invoke_binary(cli.target, &output_path);
        }
        println!("up to date: {}", output_path.display());
        return Ok(());
    }

    let options = CompileOptions {
        library_dirs: cli.library_dirs,
        libraries: cli.libraries,
        line_numbers: cli.line_numbers && !cli.sparse_line_numbers,
        target: cli.target,
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
        invoke_binary(cli.target, &output_path)?;
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

/// Compiles the transpiler's generated output down to a native binary with
/// whatever third-party compiler actually understands that target's output
/// -- `fbc` (FreeBASIC) for `Target::Basic`'s `.bas`, `gcc` for `Target::C`'s
/// (not yet generated) `.c`. `Target::C` can't reach this in practice today:
/// `compile_file` already fails with a "not implemented" diagnostic before
/// `--binary` ever gets a `.c` file to hand to `gcc`, but the dispatch is
/// wired up now so it needs no changes once that backend exists.
fn invoke_binary(target: Target, output_path: &PathBuf) -> Result<(), String> {
    match target {
        Target::Basic => invoke_fbc(output_path),
        Target::C => invoke_gcc(output_path),
    }
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

fn invoke_gcc(c_path: &PathBuf) -> Result<(), String> {
    let binary_name = c_path
        .file_stem()
        .ok_or_else(|| format!("error: invalid C output path {}", c_path.display()))?;
    let binary_dir = PathBuf::from("tmp");
    fs::create_dir_all(&binary_dir)
        .map_err(|err| format!("error: failed to create {}: {err}", binary_dir.display()))?;
    let binary_path = binary_dir.join(binary_name);
    let status = Command::new("gcc")
        .arg(c_path)
        .arg("-o")
        .arg(&binary_path)
        // Always linked, even for programs that don't need it (e.g. `\`'s
        // round()) -- harmless when unused, and simpler than detecting
        // per-file whether <math.h> was pulled in.
        .arg("-lm")
        .status()
        .map_err(|err| format!("error: failed to invoke gcc: {err}"))?;
    if !status.success() {
        return Err(format!(
            "error: gcc failed compiling {}",
            c_path.display()
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
    // Full line numbering (every emitted line, not just branch targets) is
    // the default: real MBASIC/BASCOM has no notion of an unnumbered
    // statement line, so sparse numbering only works on more lenient
    // dialects (e.g. FreeBASIC's `-lang qb`).
    let mut line_numbers = true;
    let mut sparse_line_numbers = false;
    let mut clean = false;
    let mut binary = false;
    let mut target = Target::Basic;
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
            "--sparse-line-numbers" => sparse_line_numbers = true,
            "--clean" | "-c" => clean = true,
            "--binary" | "-b" => binary = true,
            "--target" | "-t" => {
                i += 1;
                let value = args.get(i).ok_or_else(|| {
                    "error: --target requires a value (basic or c)".to_string()
                })?;
                target = match value.as_str() {
                    "basic" => Target::Basic,
                    "c" => Target::C,
                    other => {
                        return Err(format!(
                            "error: unknown --target `{other}` (expected `basic` or `c`)"
                        ))
                    }
                };
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
        library_dirs,
        libraries,
        line_numbers,
        sparse_line_numbers,
        clean,
        binary,
        target,
    })
}

fn usage() -> String {
    [
        "usage: bcc input.bcl [-o output.bas] [-L dir] [-l library]",
        "              [--sparse-line-numbers] [--clean | -c] [--binary | -b]",
        "              [--target | -t basic|c]",
        "",
        "Options:",
        "  -o output.bas          Output path (default: input with .bas extension)",
        "  -L dir                 Add a library search directory for require resolution",
        "  --sparse-line-numbers  Number only branch targets, not every line (invalid on",
        "                         real MBASIC/BASCOM; only safe with lenient dialects like",
        "                         FreeBASIC's -lang qb)",
        "  --clean, -c            Re-transpile even if the output is already up to date",
        "  --binary, -b           Compile the generated output to tmp/<stem>: fbc for",
        "                         --target basic's .bas, gcc for --target c's .c",
        "  --target, -t <target>  Backend to generate code for: `basic` (default, the only",
        "                         one implemented) or `c` (reserved for a future native",
        "                         backend; currently always fails with a diagnostic)",
    ]
    .join("\n")
}
