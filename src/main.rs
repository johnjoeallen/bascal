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
    run: bool,
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

/// Case-insensitively parses a `--target`/config-file/env-var value into a
/// `Target`, or `None` if it names neither backend -- shared by the CLI
/// flag, `BASCAL_TARGET`, and both config files so all four accept exactly
/// the same spellings (`basic`/`BASIC`/`Basic`/..., `c`/`C`).
fn parse_target_str(value: &str) -> Option<Target> {
    match value.to_ascii_lowercase().as_str() {
        "basic" => Some(Target::Basic),
        "c" => Some(Target::C),
        _ => None,
    }
}

/// Finds `key`'s value in a simple `key=value` config file's contents --
/// one setting per line, blank lines and `#`-prefixed comments ignored,
/// key matched case-insensitively. Shared by the user (`~/.config/bascal/
/// config`) and system (`/etc/default/bascal`) config files -- same
/// format as a shell env file, deliberately not a new format/parser
/// dependency to learn.
fn parse_config_value(contents: &str, key: &str) -> Option<String> {
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim().eq_ignore_ascii_case(key) {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

/// The `--target` value to use when the CLI flag itself isn't given --
/// lets a user or system set `C` as their working default without typing
/// `--target c` on every invocation. Checked in order, first match wins:
/// the `BASCAL_TARGET` environment variable (works the same on every
/// platform, including Windows, where there's no real equivalent of
/// `/etc/default/`); `~/.config/bascal/config`, a per-user default;
/// `/etc/default/bascal`, a system-wide default (the standard Debian
/// `/etc/default/<pkgname>` convention -- a plain file, not a directory).
/// Falls back to `Target::Basic` (the original, complete backend) if none
/// of those are set, or set to something unrecognized. An explicit
/// `--target`/`-t` flag on the command line always overrides whatever
/// this returns -- see `parse_args`.
fn resolve_default_target() -> Target {
    if let Ok(value) = env::var("BASCAL_TARGET") {
        if let Some(target) = parse_target_str(&value) {
            return target;
        }
    }
    if let Ok(home) = env::var("HOME") {
        let user_config = PathBuf::from(home).join(".config/bascal/config");
        if let Ok(contents) = fs::read_to_string(&user_config) {
            if let Some(value) = parse_config_value(&contents, "target") {
                if let Some(target) = parse_target_str(&value) {
                    return target;
                }
            }
        }
    }
    if let Ok(contents) = fs::read_to_string("/etc/default/bascal") {
        if let Some(value) = parse_config_value(&contents, "target") {
            if let Some(target) = parse_target_str(&value) {
                return target;
            }
        }
    }
    Target::Basic
}

fn run() -> Result<(), String> {
    let cli = parse_args(env::args().skip(1).collect())?;
    // `--run` implies `--binary` -- there's no point building a binary
    // without one, and no way to run the program without building it
    // first.
    let want_binary = cli.binary || cli.run;

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
        if want_binary && !is_up_to_date(&cli.input, &binary_path) {
            let built = invoke_binary(cli.target, &output_path)?;
            return if cli.run { run_binary(&built) } else { Ok(()) };
        }
        println!("up to date: {}", output_path.display());
        return if cli.run { run_binary(&binary_path) } else { Ok(()) };
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

    if want_binary {
        let built = invoke_binary(cli.target, &output_path)?;
        if cli.run {
            return run_binary(&built);
        }
    }

    Ok(())
}

/// Runs an already-built binary with stdin/stdout/stderr all inherited
/// (so an interactive program -- `INPUT`, `INKEY$`, ... -- works exactly
/// as if it had been run directly), used by `--run`. Doesn't try to
/// propagate the child's exact exit code -- same "an error is an error"
/// simplicity every other failure path here already uses -- just whether
/// it succeeded or not.
fn run_binary(binary_path: &PathBuf) -> Result<(), String> {
    let status = Command::new(binary_path)
        .status()
        .map_err(|err| format!("error: failed to run {}: {err}", binary_path.display()))?;
    if !status.success() {
        return Err(format!(
            "error: {} exited with {status}",
            binary_path.display()
        ));
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
/// whatever third-party compiler actually understands that target's
/// output, and returns the binary's path on success (used by `--run` to
/// find what to execute next). `Target::Basic`'s `.bas` goes through
/// `fbc` (FreeBASIC) specifically, not real BASCOM: `fbc` produces a
/// binary the host can run directly, the same way `gcc` does for
/// `Target::C`'s `.c` -- real BASCOM (used only by the opt-in
/// `tests/dosbox_conformance.rs` conformance suite, see CONTRIBUTING.md)
/// instead produces a DOS `.EXE`, runnable only under a DOS
/// environment/emulator like dosbox-x, not directly by this process.
/// That's also why `--run` always means `fbc` for the `basic` target,
/// not a user-selectable choice between the two: `fbc`'s binary is the
/// only one of the pair `--run` could actually execute itself.
fn invoke_binary(target: Target, output_path: &PathBuf) -> Result<PathBuf, String> {
    match target {
        Target::Basic => invoke_fbc(output_path),
        Target::C => invoke_gcc(output_path),
    }
}

fn invoke_fbc(bas_path: &PathBuf) -> Result<PathBuf, String> {
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
    Ok(binary_path)
}

fn invoke_gcc(c_path: &PathBuf) -> Result<PathBuf, String> {
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
    Ok(binary_path)
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
    let mut run = false;
    let mut target = resolve_default_target();
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
            "--run" | "-r" => run = true,
            "--target" | "-t" => {
                i += 1;
                let value = args.get(i).ok_or_else(|| {
                    "error: --target requires a value (basic or C)".to_string()
                })?;
                target = parse_target_str(value).ok_or_else(|| {
                    format!(
                        "error: unknown --target `{value}` (expected `basic` or `C`, \
                         case-insensitive)"
                    )
                })?;
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
        run,
        target,
    })
}

fn usage() -> String {
    [
        "usage: bcc input.bcl [-o output.bas] [-L dir] [-l library]",
        "              [--line-numbers | --sparse-line-numbers] [--clean | -c]",
        "              [--binary | -b] [--run | -r] [--target | -t basic|C]",
        "",
        "Options:",
        "  -o output.bas          Output path (default: input with .bas/.c extension)",
        "  -L dir                 Add a library search directory for require resolution",
        "                         (repeatable)",
        "  -l name                Name a library (reserved for future use)",
        "  --line-numbers         Number every output line, not just branch targets",
        "                         (the default -- only needed to override an earlier",
        "                         --sparse-line-numbers on the same command line)",
        "  --sparse-line-numbers  Number only branch targets, not every line (invalid on",
        "                         real MBASIC/BASCOM; only safe with lenient dialects like",
        "                         FreeBASIC's -lang qb)",
        "  --clean, -c            Re-transpile even if the output is already up to date",
        "  --binary, -b           Compile the generated output to tmp/<stem>: fbc for",
        "                         --target basic's .bas, gcc for --target C's .c",
        "  --run, -r              Also run the compiled binary (implies --binary). For",
        "                         --target basic this always means fbc's binary, run",
        "                         directly -- not real BASCOM, whose own .EXE needs a DOS",
        "                         environment/emulator like dosbox-x to run at all",
        "  --target, -t <target>  Backend to generate code for: `basic` (the original,",
        "                         complete backend) or `C` (an experimental native-C",
        "                         backend). Case-insensitive; `c` also works.",
        "",
        "Default target (used when --target isn't given), first match wins:",
        "  1. BASCAL_TARGET environment variable",
        "  2. ~/.config/bascal/config (\"target=C\", one setting per line)",
        "  3. /etc/default/bascal (same format, system-wide)",
        "  4. basic, if none of the above are set",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_target_str_is_case_insensitive() {
        assert_eq!(parse_target_str("basic"), Some(Target::Basic));
        assert_eq!(parse_target_str("BASIC"), Some(Target::Basic));
        assert_eq!(parse_target_str("Basic"), Some(Target::Basic));
        assert_eq!(parse_target_str("c"), Some(Target::C));
        assert_eq!(parse_target_str("C"), Some(Target::C));
        assert_eq!(parse_target_str("bogus"), None);
    }

    #[test]
    fn parse_config_value_finds_a_key_case_insensitively_and_trims() {
        let contents = "# a comment\n\ntarget = C \nother=ignored\n";
        assert_eq!(parse_config_value(contents, "target"), Some("C".to_string()));
        assert_eq!(parse_config_value(contents, "TARGET"), Some("C".to_string()));
        assert_eq!(parse_config_value(contents, "missing"), None);
    }

    #[test]
    fn parse_config_value_skips_blank_lines_and_comments() {
        let contents = "\n# target=basic\n  \ntarget=C\n";
        assert_eq!(parse_config_value(contents, "target"), Some("C".to_string()));
    }

    #[test]
    fn parse_args_accepts_uppercase_target_flag() {
        let cli = parse_args(vec!["input.bcl".to_string(), "--target".to_string(), "C".to_string()])
            .expect("should parse");
        assert_eq!(cli.target, Target::C);
    }

    #[test]
    fn parse_args_rejects_unknown_target() {
        let err = parse_args(vec!["input.bcl".to_string(), "-t".to_string(), "bogus".to_string()])
            .expect_err("should reject an unknown target");
        assert!(err.contains("unknown --target"), "unexpected error: {err}");
    }

    #[test]
    fn parse_args_accepts_run_flag_long_and_short() {
        let cli = parse_args(vec!["input.bcl".to_string(), "--run".to_string()])
            .expect("should parse --run");
        assert!(cli.run);
        assert!(!cli.binary, "--run alone shouldn't also set binary -- run() adds that itself");

        let cli = parse_args(vec!["input.bcl".to_string(), "-r".to_string()])
            .expect("should parse -r");
        assert!(cli.run);
    }
}
