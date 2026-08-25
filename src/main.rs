use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use bcc::{compile_file, default_output_path, CompileOptions, Target};
use clap::Parser;

/// Translates structured `.bcl` source into plain 1980s Microsoft BASIC
/// (the `basic` target, complete) or an experimental native-C backend (the
/// `C` target).
#[derive(Parser, Debug)]
#[command(name = "bcc", version, about, after_help = DEFAULT_TARGET_HELP)]
struct Cli {
    /// BASCAL source file to compile
    #[arg(value_name = "input.bcl")]
    input: PathBuf,

    /// Output directory -- existing, or written with a trailing `/` even if it doesn't exist yet. The output file is auto-named inside it (input's stem plus .bas/.c); an exact output file path is not accepted here. Default: input with .bas/.c extension, same directory as input
    #[arg(short = 'o', value_name = "DIR")]
    output: Option<PathBuf>,

    /// Add a library search directory for `require` resolution (repeatable)
    #[arg(short = 'L', value_name = "DIR")]
    library_dirs: Vec<PathBuf>,

    /// Name a library (reserved for future use)
    #[arg(short = 'l', value_name = "NAME")]
    libraries: Vec<String>,

    /// Number every output line, not just branch targets (the default -- only needed to override an earlier --sparse-line-numbers on the same command line)
    #[arg(long)]
    line_numbers: bool,

    /// Number only branch targets, not every line. Real MBASIC/BASCOM-family compilers need a switch for this: Microsoft's BASCOM uses /C, IBM's BASIC Compiler uses /N; FreeBASIC's -lang qb accepts it with no switch at all
    #[arg(long)]
    sparse_line_numbers: bool,

    /// Re-transpile even if the output is already up to date
    #[arg(short = 'c', long)]
    clean: bool,

    /// Compile the generated output to a binary in tmp/: fbc for --target basic's .bas, gcc for --target C's .c
    #[arg(short = 'b', long)]
    binary: bool,

    /// Also run the compiled binary (implies --binary), with stdin/stdout/stderr inherited. For --target basic this always means fbc's binary, run directly -- not real BASCOM, whose own .EXE needs a DOS environment/emulator like dosbox-x to run at all
    #[arg(short = 'r', long)]
    run: bool,

    /// Backend to generate code for: `basic` (the original, complete backend) or `C` (an experimental native-C backend). Case-insensitive; `c` also works. Default, if this flag isn't given: see DEFAULT TARGET below
    #[arg(short = 't', long, value_name = "TARGET", value_parser = parse_target_value)]
    target: Option<Target>,

    /// Require every variable to be dim'd/declare'd before use (Pascal-style) -- opt-in, and not part of BASCAL's BASIC superset when on. Rejects the compile if any variable is used without one. Checked only against this program's own source, never a required library's
    #[arg(long)]
    strict_vars: bool,

    /// Same check as --strict-vars, but prints findings to stderr as warnings instead of failing the compile. Ignored if --strict-vars is also given
    #[arg(long)]
    strict_vars_warn: bool,
}

const DEFAULT_TARGET_HELP: &str = "\
Default target (used when --target isn't given), first match wins:
  1. BASCAL_TARGET environment variable
  2. ~/.config/bascal/config (\"target=C\", one setting per line)
  3. /etc/default/bascal (same format, system-wide)
  4. basic, if none of the above are set";

fn main() -> ExitCode {
    // Parsing happens before entering the fallible part of the program on
    // purpose: a bad flag or --help/--version are clap's own concern (it
    // prints its own formatted message and exits itself), entirely
    // separate from the String-based "error: ..." reporting every actual
    // compile/build/run failure below uses.
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

/// Case-insensitively parses a `--target`/config-file/env-var value into a
/// `Target`, or `None` if it names neither backend -- shared by the CLI
/// flag's `clap` value parser, `BASCAL_TARGET`, and both config files so
/// all four accept exactly the same spellings (`basic`/`BASIC`/`Basic`/...,
/// `c`/`C`).
fn parse_target_str(value: &str) -> Option<Target> {
    match value.to_ascii_lowercase().as_str() {
        "basic" => Some(Target::Basic),
        "c" => Some(Target::C),
        "jvm" => Some(Target::Jvm),
        _ => None,
    }
}

/// `clap`'s own `value_parser` for `--target`/`-t` -- thin wrapper around
/// `parse_target_str` matching the `Fn(&str) -> Result<Target, String>`
/// shape `clap` expects.
fn parse_target_value(value: &str) -> Result<Target, String> {
    parse_target_str(value).ok_or_else(|| {
        format!("expected `basic`, `C`, or `jvm` (case-insensitive), got `{value}`")
    })
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
/// this returns -- see `run`.
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

/// The `-o` value's *effective* output path. `-o` only ever names a
/// directory -- already existing as one, or written with a trailing path
/// separator even if it doesn't exist yet (`-o out/` for output that
/// hasn't been generated before) -- never an exact file path to spell out
/// by hand. The actual output file goes inside that directory, auto-named
/// the same way an omitted `-o` would name it (input's stem plus the
/// target's extension). Anything that doesn't look like a directory (no
/// trailing separator, and not an existing directory) is rejected outright
/// rather than silently reinterpreted as a literal file path.
fn resolve_output_path(cli: &Cli, target: Target) -> Result<PathBuf, String> {
    let Some(output) = &cli.output else {
        return Ok(default_output_path(cli.input.as_path(), target));
    };
    let looks_like_dir = output.is_dir()
        || output.as_os_str().to_string_lossy().ends_with('/')
        || output
            .as_os_str()
            .to_string_lossy()
            .ends_with(std::path::MAIN_SEPARATOR);
    if !looks_like_dir {
        return Err(format!(
            "error: -o must name a directory -- got {} -- point it at an existing directory, \
             or write a trailing `/` for one that doesn't exist yet (e.g. `-o build/`); the \
             output file's own name is always inferred from the input file",
            output.display()
        ));
    }
    let default_name = default_output_path(cli.input.as_path(), target);
    let file_name = default_name.file_name().ok_or_else(|| {
        format!(
            "error: can't derive an output file name from {}",
            cli.input.display()
        )
    })?;
    Ok(output.join(file_name))
}

fn run(cli: Cli) -> Result<(), String> {
    let target = cli.target.unwrap_or_else(resolve_default_target);
    // `--run` implies `--binary` -- there's no point building a binary
    // without one, and no way to run the program without building it
    // first.
    let want_binary = cli.binary || cli.run;

    let output_path = resolve_output_path(&cli, target)?;

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
            let built = invoke_binary(target, &output_path)?;
            return if cli.run { run_binary(&built) } else { Ok(()) };
        }
        println!("up to date: {}", output_path.display());
        return if cli.run {
            run_binary(&binary_path)
        } else {
            Ok(())
        };
    }

    let options = CompileOptions {
        library_dirs: cli.library_dirs,
        libraries: cli.libraries,
        line_numbers: cli.line_numbers || !cli.sparse_line_numbers,
        target,
        strict_vars: cli.strict_vars,
        strict_vars_warn: cli.strict_vars_warn && !cli.strict_vars,
    };
    let generated = compile_file(&cli.input, &options).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;

    fs::write(&output_path, &generated)
        .map_err(|err| format!("error: failed to write {}: {err}", output_path.display()))?;

    if want_binary {
        let built = invoke_binary(target, &output_path)?;
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
        Target::Jvm => Err(
            "--binary/--run don't support --target jvm yet -- it's a bootstrap-stage backend, \
             see codegen_jvm.rs's module doc comment"
                .to_string(),
        ),
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
        return Err(format!("error: gcc failed compiling {}", c_path.display()));
    }
    println!("binary: {}", binary_path.display());
    Ok(binary_path)
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
        assert_eq!(
            parse_config_value(contents, "target"),
            Some("C".to_string())
        );
        assert_eq!(
            parse_config_value(contents, "TARGET"),
            Some("C".to_string())
        );
        assert_eq!(parse_config_value(contents, "missing"), None);
    }

    #[test]
    fn parse_config_value_skips_blank_lines_and_comments() {
        let contents = "\n# target=basic\n  \ntarget=C\n";
        assert_eq!(
            parse_config_value(contents, "target"),
            Some("C".to_string())
        );
    }

    #[test]
    fn cli_accepts_uppercase_target_flag() {
        let cli = Cli::try_parse_from(["bcc", "input.bcl", "--target", "C"]).expect("should parse");
        assert_eq!(cli.target, Some(Target::C));
    }

    #[test]
    fn cli_rejects_unknown_target() {
        let err = Cli::try_parse_from(["bcc", "input.bcl", "-t", "bogus"])
            .expect_err("should reject an unknown target");
        assert!(err.to_string().contains("basic"), "unexpected error: {err}");
    }

    #[test]
    fn cli_accepts_run_flag_long_and_short() {
        let cli = Cli::try_parse_from(["bcc", "input.bcl", "--run"]).expect("should parse --run");
        assert!(cli.run);
        assert!(
            !cli.binary,
            "--run alone shouldn't also set binary -- run() adds that itself"
        );

        let cli = Cli::try_parse_from(["bcc", "input.bcl", "-r"]).expect("should parse -r");
        assert!(cli.run);
    }

    #[test]
    fn cli_target_defaults_to_none_letting_run_apply_resolve_default_target() {
        let cli = Cli::try_parse_from(["bcc", "input.bcl"]).expect("should parse");
        assert_eq!(cli.target, None);
    }

    #[test]
    fn cli_accepts_strict_vars_and_strict_vars_warn() {
        let cli =
            Cli::try_parse_from(["bcc", "input.bcl", "--strict-vars"]).expect("should parse");
        assert!(cli.strict_vars);
        assert!(!cli.strict_vars_warn);

        let cli = Cli::try_parse_from(["bcc", "input.bcl", "--strict-vars-warn"])
            .expect("should parse");
        assert!(!cli.strict_vars);
        assert!(cli.strict_vars_warn);

        let cli = Cli::try_parse_from(["bcc", "input.bcl"]).expect("should parse");
        assert!(!cli.strict_vars);
        assert!(!cli.strict_vars_warn);
    }

    #[test]
    fn strict_vars_wins_over_strict_vars_warn_when_both_are_given() {
        // `run()`'s own precedence rule (`cli.strict_vars_warn && !cli.strict_vars`)
        // -- both flags parse fine together, but only the hard-error mode
        // should end up active in the CompileOptions `run()` builds.
        let cli = Cli::try_parse_from([
            "bcc",
            "input.bcl",
            "--strict-vars",
            "--strict-vars-warn",
        ])
        .expect("should parse both flags together");
        assert!(cli.strict_vars);
        assert!(cli.strict_vars_warn);
        assert!(
            !(cli.strict_vars_warn && !cli.strict_vars),
            "the effective warn flag run() computes must be false once --strict-vars is also set"
        );
    }

    fn cli_with_output(input: PathBuf, output: Option<PathBuf>) -> Cli {
        Cli {
            input,
            output,
            library_dirs: Vec::new(),
            libraries: Vec::new(),
            line_numbers: true,
            sparse_line_numbers: false,
            clean: false,
            binary: false,
            run: false,
            target: None,
            strict_vars: false,
            strict_vars_warn: false,
        }
    }

    #[test]
    fn resolve_output_path_treats_an_existing_directory_as_a_target_directory() {
        let dir = tempfile::tempdir().unwrap();
        let cli = cli_with_output(
            PathBuf::from("some/input.bcl"),
            Some(dir.path().to_path_buf()),
        );
        let resolved = resolve_output_path(&cli, Target::C).unwrap();
        assert_eq!(resolved, dir.path().join("input.c"));
    }

    #[test]
    fn resolve_output_path_treats_a_trailing_slash_as_a_directory_even_if_it_does_not_exist_yet() {
        let dir = tempfile::tempdir().unwrap();
        let not_yet_created = dir.path().join("nested");
        let mut with_slash = not_yet_created.to_string_lossy().into_owned();
        with_slash.push('/');
        let cli = cli_with_output(PathBuf::from("input.bcl"), Some(PathBuf::from(with_slash)));
        let resolved = resolve_output_path(&cli, Target::Basic).unwrap();
        assert_eq!(resolved, not_yet_created.join("input.bas"));
    }

    #[test]
    fn resolve_output_path_rejects_a_plain_path_that_does_not_look_like_a_directory() {
        let cli = cli_with_output(
            PathBuf::from("input.bcl"),
            Some(PathBuf::from("exact/output.bas")),
        );
        let err = resolve_output_path(&cli, Target::Basic)
            .expect_err("-o must name a directory, not an exact file path");
        assert!(err.contains("-o must name a directory"), "unexpected: {err}");
    }

    #[test]
    fn resolve_output_path_defaults_next_to_the_input_when_o_is_omitted() {
        let cli = cli_with_output(PathBuf::from("some/dir/input.bcl"), None);
        let resolved = resolve_output_path(&cli, Target::C).unwrap();
        assert_eq!(resolved, PathBuf::from("some/dir/input.c"));
    }
}
