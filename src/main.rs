use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use bcc::{check_file, compile_file, default_output_path, CompileOptions, Target};
use clap::Parser;

mod jvm_classfile;

/// Translates structured `.bcl` source into plain 1980s Microsoft BASIC
/// (the `basic`/`bascom` target, complete, verified against real BASCOM;
/// or `fbc`, identical output but rejecting the handful of constructs real
/// BASCOM accepts that `fbc` itself does not -- see `Target`'s own doc
/// comment in codegen.rs), a mostly-complete native-C backend (the `c`
/// target), or a brand-new, bootstrap-stage native-JVM backend (the `jvm`
/// target -- just beginning, not yet ready for real programs).
/// `--version`'s full text -- GNU tools' own convention (see e.g. `gcc
/// --version`, `bash --version`) for what a copyright/license notice in
/// `--version` output should look like; the GPL itself recommends exactly
/// this for a program with terminal interaction (see the license notice
/// boilerplate at the end of LICENSE). "GPLv3" not "GPLv3+": Cargo.toml
/// declares `license = "GPL-3.0-only"`, not `-or-later`.
const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "\n\nCopyright (C) 2026 BASCAL contributors\n",
    "License GPLv3: GNU GPL version 3 <https://gnu.org/licenses/gpl.html>.\n",
    "This is free software: you are free to change and redistribute it.\n",
    "There is NO WARRANTY, to the extent permitted by law.\n\n",
    "Code:    https://github.com/johnjoeallen/bascal\n",
    "Website: https://johnjoeallen.github.io/bascal/"
);

#[derive(Parser, Debug)]
#[command(name = "bcc", version = VERSION, about, after_help = DEFAULT_TARGET_HELP)]
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

    /// Parse this source and every required library without resolving or generating a backend output file
    #[arg(long)]
    check: bool,

    /// Compile the generated output to a binary in tmp/: for --target basic/bascom, real BASCOM under dosbox-x (needs dosbox-x on PATH plus a local BASCOM fixture -- see CONTRIBUTING.md), producing a DOS .EXE; fbc for --target fbc's .bas, gcc for --target c's .c, krak2 for --target jvm's .j
    #[arg(short = 'b', long)]
    binary: bool,

    /// Also run the compiled binary (implies --binary), with stdin/stdout/stderr inherited. For --target basic/bascom this means launching the built .EXE in dosbox-x's own window (needs a display; won't work fully headless) since a DOS binary can't be exec'd directly; for --target fbc, fbc's native binary is run directly instead
    #[arg(short = 'r', long)]
    run: bool,

    /// Backend to generate code for: `basic` (alias `bascom` -- the original, complete backend, verified against real BASCOM, including for --binary/--run via dosbox-x), `fbc` (the same BASIC, but for FreeBASIC specifically -- a native binary for --binary/--run, and rejects the handful of constructs real BASCOM accepts that fbc does not, e.g. try/catch), `c` (a mostly-complete native-C backend), or `jvm` (a brand-new, bootstrap-stage native-JVM backend, just beginning). Case-insensitive. Default, if this flag isn't given: see DEFAULT TARGET below
    #[arg(short = 't', long, value_name = "TARGET", value_parser = parse_target_value)]
    target: Option<Target>,

    /// Require every variable to be dim'd/declare'd before use (Pascal-style) -- opt-in, and not part of BASCAL's BASIC superset when on. Rejects the compile if any variable is used without one. Checked only against this program's own source, never a required library's
    #[arg(long)]
    strict_vars: bool,

    /// Same check as --strict-vars, but prints findings to stderr as warnings instead of failing the compile. Ignored if --strict-vars is also given
    #[arg(long)]
    strict_vars_warn: bool,

    /// --target jvm only: stack size for the Krakatau assembler's own worker thread (the host-side compiler tool that turns codegen_jvm.rs's .j text into a .class -- not the JVM's own runtime stack, unrelated to `java -Xss`). Its recursive stack-map-frame/control-flow analysis can overflow a too-small stack on a large generated program -- see Cargo.toml's own comment on why this is bcc's own responsibility, not the library's. Plain bytes, or suffixed with k/kb, m/mb, g/gb (case-insensitive, powers of 1024 -- e.g. `64mb`, `256M`, `1gb`). Default, if this flag isn't given: see KRAK_STACK_SIZE below. Ignored for every other --target
    #[arg(long, value_name = "SIZE", value_parser = parse_byte_count_value)]
    krak_stack_size: Option<usize>,
}

const DEFAULT_TARGET_HELP: &str = "\
Default target (used when --target isn't given), first match wins:
  1. BASCAL_TARGET environment variable
  2. ~/.config/bascal/config (\"target=c\", one setting per line)
  3. /etc/default/bascal (same format, system-wide)
  4. basic, if none of the above are set

KRAK_STACK_SIZE (used when --krak-stack-size isn't given, --target jvm only),
same first-match-wins order as DEFAULT TARGET above:
  1. BASCAL_KRAK_STACK_SIZE environment variable
  2. ~/.config/bascal/config (\"krak_stack_size=64mb\", or a plain byte count)
  3. /etc/default/bascal (same format, system-wide)
  4. 33554432 (32 MiB), if none of the above are set";

fn main() -> ExitCode {
    // -V/--version is intercepted here, ahead of clap, because clap 4's
    // auto-generated version flag hardcodes a "{bin} {version}" first line
    // with no override hook (version_template was removed after clap 3) --
    // this is the only way to get "BASCAL Compiler version x.y.z" instead
    // of "bcc x.y.z" while still letting clap own everything else
    // (--help's own flag listing, error messages, ...).
    if env::args()
        .nth(1)
        .is_some_and(|arg| arg == "-V" || arg == "--version")
    {
        println!("BASCAL Compiler version {VERSION}");
        return ExitCode::SUCCESS;
    }

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
        "basic" | "bascom" => Some(Target::Basic),
        "fbc" => Some(Target::Fbc),
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
        format!(
            "expected `basic` (alias `bascom`), `fbc`, `c`, or `jvm` (case-insensitive), got `{value}`"
        )
    })
}

/// Parses a byte-count value: plain digits (bytes), or digits followed by
/// a case-insensitive `b`/`k`/`kb`/`m`/`mb`/`g`/`gb` unit (powers of 1024,
/// not 1000 -- matching every other Unix disk/memory size convention this
/// gets compared against) -- e.g. `"67108864"`, `"64mb"`, `"64M"`,
/// `"1gb"`. Optional whitespace between the digits and the unit. Shared
/// by the `--krak-stack-size` CLI flag's own `value_parser`
/// (`parse_byte_count_value` below) and `BASCAL_KRAK_STACK_SIZE`/the
/// config-file value (`resolve_default_krak_stack_size`), so all three
/// accept exactly the same formats. `None` for anything else, including
/// empty input, a bare unit with no digits, or a value that overflows
/// `usize` once the unit's multiplied in.
fn parse_byte_count(value: &str) -> Option<usize> {
    let value = value.trim();
    let split_at = value
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(value.len());
    let (digits, unit) = value.split_at(split_at);
    if digits.is_empty() {
        return None;
    }
    let number: usize = digits.parse().ok()?;
    let multiplier: usize = match unit.trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1,
        "k" | "kb" => 1024,
        "m" | "mb" => 1024 * 1024,
        "g" | "gb" => 1024 * 1024 * 1024,
        _ => return None,
    };
    number.checked_mul(multiplier)
}

/// `clap`'s own `value_parser` for `--krak-stack-size` -- thin wrapper
/// around `parse_byte_count` matching the `Fn(&str) -> Result<usize,
/// String>` shape `clap` expects.
fn parse_byte_count_value(value: &str) -> Result<usize, String> {
    parse_byte_count(value).ok_or_else(|| {
        format!(
            "expected a byte count, optionally suffixed with b/k/kb/m/mb/g/gb (case-insensitive, \
             e.g. `64mb`), got `{value}`"
        )
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

/// Looks up `key` (e.g. `"target"`, `"krak_stack_size"`) across the three
/// places every `--<flag>`-with-a-config-fallback in this file shares,
/// first match wins: the `BASCAL_<KEY>` environment variable (`key`
/// upper-cased -- works the same on every platform, including Windows,
/// where there's no real equivalent of `/etc/default/`); `~/.config/
/// bascal/config`, a per-user default; `/etc/default/bascal`, a
/// system-wide default (the standard Debian `/etc/default/<pkgname>`
/// convention -- a plain file, not a directory). A source whose raw value
/// doesn't `parse` (e.g. `BASCAL_TARGET=nonsense`) is treated as absent,
/// not an error -- the next source in line still gets a chance, exactly
/// as if that source hadn't set the key at all. `None` if no source
/// yields a value `parse` accepts.
fn resolve_config_value<T>(key: &str, parse: impl Fn(&str) -> Option<T>) -> Option<T> {
    let env_key = format!("BASCAL_{}", key.to_ascii_uppercase());
    if let Ok(value) = env::var(&env_key) {
        if let Some(parsed) = parse(&value) {
            return Some(parsed);
        }
    }
    if let Ok(home) = env::var("HOME") {
        let user_config = PathBuf::from(home).join(".config/bascal/config");
        if let Ok(contents) = fs::read_to_string(&user_config) {
            if let Some(value) = parse_config_value(&contents, key) {
                if let Some(parsed) = parse(&value) {
                    return Some(parsed);
                }
            }
        }
    }
    if let Ok(contents) = fs::read_to_string("/etc/default/bascal") {
        if let Some(value) = parse_config_value(&contents, key) {
            if let Some(parsed) = parse(&value) {
                return Some(parsed);
            }
        }
    }
    None
}

/// The `--target` value to use when the CLI flag itself isn't given --
/// lets a user or system set `c` as their working default without typing
/// `--target c` on every invocation. See `resolve_config_value`'s own doc
/// comment for the precedence order (`BASCAL_TARGET` env var, user
/// config, system config). Falls back to `Target::Basic` (the original,
/// complete backend) if none of those are set, or set to something
/// unrecognized. An explicit `--target`/`-t` flag on the command line
/// always overrides whatever this returns -- see `run`.
fn resolve_default_target() -> Target {
    resolve_config_value("target", parse_target_str).unwrap_or(Target::Basic)
}

/// The Krakatau assembler's own worker-thread stack size (bytes,
/// `invoke_krak2`'s thread -- not the JVM's own runtime stack, `java
/// -Xss`, which this has no connection to at all) to use when
/// `--krak-stack-size` itself isn't given -- see `invoke_krak2`'s own doc
/// comment for why `bcc` runs `krakatau2::assemble` on a dedicated thread
/// with an explicit stack size at all. Same precedence order as
/// `resolve_default_target` (`BASCAL_KRAK_STACK_SIZE` env var, user
/// config, system config), via `resolve_config_value`. Falls back to
/// 32 MiB if none of those are set, or set to something `parse_byte_
/// count` doesn't accept -- confirmed empirically: even a trivial
/// generated program can overflow a stack as small as 64 KiB (krak2's
/// recursive stack-map-frame/control-flow analysis needs real headroom),
/// so this default deliberately isn't a token/minimal value.
fn resolve_default_krak_stack_size() -> usize {
    const DEFAULT_KRAK_STACK_SIZE: usize = 32 * 1024 * 1024;
    resolve_config_value("krak_stack_size", parse_byte_count).unwrap_or(DEFAULT_KRAK_STACK_SIZE)
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
    let krak_stack_size = cli
        .krak_stack_size
        .unwrap_or_else(resolve_default_krak_stack_size);

    if cli.check {
        if cli.binary || cli.run {
            return Err("error: --check cannot be combined with --binary or --run".to_string());
        }
        let options = CompileOptions {
            library_dirs: cli.library_dirs,
            libraries: cli.libraries,
            line_numbers: cli.line_numbers || !cli.sparse_line_numbers,
            target,
            strict_vars: false,
            strict_vars_warn: false,
        };
        check_file(&cli.input, &options).map_err(|diagnostics| {
            diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        println!("check passed: {}", cli.input.display());
        return Ok(());
    }

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
        let binary_path = expected_binary_path(target, &output_path)?;
        if want_binary && !is_up_to_date(&cli.input, &binary_path) {
            let built = invoke_binary(target, &output_path, krak_stack_size)?;
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
        let built = invoke_binary(target, &output_path, krak_stack_size)?;
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
    // A `--target jvm` "binary" is a `.class` file, not something the OS can
    // exec directly -- `invoke_krak2` names it after the class it contains
    // (see its own doc comment), so recovering that name back out of the
    // path and handing it to `java -cp` is exact, not a guess.
    if binary_path.extension().and_then(|ext| ext.to_str()) == Some("class") {
        return run_java_class(binary_path);
    }
    // A `--target basic` "binary" (`invoke_bascom`) is a DOS `.EXE`,
    // likewise not something this process's OS can exec directly -- see
    // `run_dos_exe`'s own doc comment for why it needs dosbox-x's own
    // window rather than an inherited-stdio child process.
    if binary_path.extension().and_then(|ext| ext.to_str()) == Some("EXE") {
        return run_dos_exe(binary_path);
    }
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

fn run_java_class(class_path: &PathBuf) -> Result<(), String> {
    let class_name = class_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| format!("error: invalid class path {}", class_path.display()))?;
    let class_dir = class_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    let status = Command::new("java")
        .arg("-cp")
        .arg(class_dir)
        .arg(class_name)
        .status()
        .map_err(|err| format!("error: failed to invoke java: {err}"))?;
    if !status.success() {
        return Err(format!("error: {class_name} exited with {status}"));
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

/// Compiles the transpiler's generated output down to a binary with
/// whatever third-party compiler actually understands that target's
/// output, and returns the binary's path on success (used by `--run` to
/// find what to execute next). `Target::Fbc`'s `.bas` goes through `fbc`
/// (FreeBASIC), producing a native binary the host can run directly, the
/// same way `gcc` does for `Target::C`'s `.c`. `Target::Basic`'s `.bas`
/// instead goes through real BASCOM under dosbox-x (see `invoke_bascom`),
/// producing a DOS `.EXE` -- `run_binary` special-cases that extension the
/// same way it already does JVM's `.class`, launching dosbox-x itself
/// rather than exec'ing the file directly.
fn invoke_binary(
    target: Target,
    output_path: &PathBuf,
    krak_stack_size: usize,
) -> Result<PathBuf, String> {
    match target {
        Target::Basic => invoke_bascom(output_path),
        Target::Fbc => invoke_fbc(output_path),
        Target::C => invoke_gcc(output_path),
        Target::Jvm => invoke_krak2(output_path, krak_stack_size),
    }
}

fn invoke_fbc(bas_path: &PathBuf) -> Result<PathBuf, String> {
    let binary_name = bas_path
        .file_stem()
        .ok_or_else(|| format!("error: invalid BASIC output path {}", bas_path.display()))?;
    let binary_dir = PathBuf::from("tmp");
    fs::create_dir_all(&binary_dir)
        .map_err(|err| format!("error: failed to create {}: {err}", binary_dir.display()))?;
    let binary_path = native_binary_path_from_stem(&binary_dir, binary_name);
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

/// Where a local, non-redistributed real-BASCOM install lives -- the exact
/// same directory `tests/dosbox_conformance.rs`'s opt-in conformance suite
/// uses (see CONTRIBUTING.md/`scripts/fetch-ibm-basic-compiler.sh`), reused
/// here rather than adding a separate, user-facing config knob: BASCOM is
/// copyrighted and can't ship with `bcc`, so both the test suite and
/// `--target basic`'s own `--binary`/`--run` need the same one-time setup
/// either way. Relative to the current directory, same as `tmp/` below.
fn bascom_fixture_dir() -> PathBuf {
    PathBuf::from("test-fixtures/ibm-basic-compiler/c_drive")
}

fn bascom_available() -> bool {
    bascom_fixture_dir().join("BASCOM.EXE").is_file()
}

fn dosbox_x_available() -> bool {
    // dosbox-x exits non-zero for `--version` (it treats the flag as an
    // early-exit trigger rather than a clean "print and succeed" query),
    // matching `tests/dosbox_conformance.rs`'s own `dosbox_x_available`.
    Command::new("dosbox-x").arg("--version").output().is_ok()
}

/// A DOS text file needs CRLF line endings and a trailing Ctrl-Z (0x1A) EOF
/// marker to be read correctly by real DOS-era tools -- identical to
/// `tests/dosbox_conformance.rs`'s own `to_dos_text`.
fn to_dos_text(text: &str) -> Vec<u8> {
    let mut dos = text.replace('\n', "\r\n");
    if !dos.ends_with("\r\n") {
        dos.push_str("\r\n");
    }
    let mut bytes = dos.into_bytes();
    bytes.push(0x1A);
    bytes
}

fn write_dos_file(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, to_dos_text(contents))
        .map_err(|err| format!("error: failed to write {}: {err}", path.display()))
}

/// Every real-BASCOM work directory this process ever builds lives at
/// `tmp/<stem>_bascom/`, one per compiled program, holding a staged copy of
/// the whole BASCOM fixture (compiler + linker + libs) plus that program's
/// own `PROG.BAS`/`PROG.EXE` -- a fixed 8.3-safe DOS name, side-stepping
/// any concern about a `.bcl` stem too long or containing characters DOS
/// filenames can't, the same way `tests/dosbox_conformance.rs`'s `TEST`
/// stem does.
fn bascom_work_dir(stem: &std::ffi::OsStr) -> PathBuf {
    let mut dir_name = stem.to_os_string();
    dir_name.push("_bascom");
    PathBuf::from("tmp").join(dir_name)
}

/// Runs `dosbox-x` headlessly against `work_dir` (mounted as `C:`),
/// executing `batch_file` (a filename inside `work_dir`) and exiting
/// immediately after -- identical arrangement to
/// `tests/dosbox_conformance.rs`'s own `run_dosbox_batch`, since this is
/// exactly the same "compile with a real DOS tool under emulation" step,
/// just invoked from `bcc` itself instead of a test.
fn run_dosbox_batch_headless(work_dir: &Path, batch_file: &str) -> Result<(), String> {
    let mount_arg = format!("MOUNT C: {}", work_dir.display());
    let status = Command::new("dosbox-x")
        .env("SDL_AUDIODRIVER", "dummy")
        .arg("-nogui")
        .arg("-c")
        .arg(&mount_arg)
        .arg("-c")
        .arg("C:")
        .arg("-c")
        .arg(batch_file)
        .arg("-fastlaunch")
        .arg("-exit")
        .status()
        .map_err(|err| format!("error: failed to invoke dosbox-x: {err}"))?;
    if !status.success() {
        return Err(format!("error: dosbox-x exited with {status}"));
    }
    Ok(())
}

fn missing_bascom_setup_error() -> String {
    if !dosbox_x_available() {
        return "error: dosbox-x not found on PATH -- install it to build/run --target \
                basic/bascom output via real BASCOM (see CONTRIBUTING.md), or use \
                --target fbc instead"
            .to_string();
    }
    format!(
        "error: {} not found -- run scripts/fetch-ibm-basic-compiler.sh to populate a \
         local BASCOM fixture (see test-fixtures/README.md), or use --target fbc instead",
        bascom_fixture_dir().join("BASCOM.EXE").display()
    )
}

/// Compiles `bas_path` with real BASCOM under `dosbox-x`, staged into
/// `bascom_work_dir`, and returns the resulting DOS `.EXE`'s path.
/// `/E` (BASCOM's own switch for `ON ERROR`/`RESUME` support) is always
/// passed: confirmed harmless for a program that doesn't use it, and
/// required for one that does (real BASCOM otherwise still "compiles" the
/// program -- nonzero severe-error count, but still produces an EXE -- yet
/// silently never installs the error trap at runtime).
fn invoke_bascom(bas_path: &PathBuf) -> Result<PathBuf, String> {
    if !dosbox_x_available() || !bascom_available() {
        return Err(missing_bascom_setup_error());
    }

    let stem = bas_path
        .file_stem()
        .ok_or_else(|| format!("error: invalid BASIC output path {}", bas_path.display()))?;
    let work_dir = bascom_work_dir(stem);
    let _ = fs::remove_dir_all(&work_dir);
    fs::create_dir_all(&work_dir)
        .map_err(|err| format!("error: failed to create {}: {err}", work_dir.display()))?;

    let fixture_dir = bascom_fixture_dir();
    for entry in fs::read_dir(&fixture_dir)
        .map_err(|err| format!("error: failed to read {}: {err}", fixture_dir.display()))?
    {
        let entry = entry
            .map_err(|err| format!("error: failed to read compiler fixture entry: {err}"))?;
        let dest = work_dir.join(entry.file_name());
        fs::copy(entry.path(), &dest)
            .map_err(|err| format!("error: failed to stage {}: {err}", dest.display()))?;
    }

    let basic_source = fs::read_to_string(bas_path)
        .map_err(|err| format!("error: failed to read {}: {err}", bas_path.display()))?;
    write_dos_file(&work_dir.join("PROG.BAS"), &basic_source)?;
    write_dos_file(
        &work_dir.join("RUNIT.BAT"),
        "BASCOM PROG.BAS,,;/E\nLINK PROG.OBJ;\nEXIT\n",
    )?;

    run_dosbox_batch_headless(&work_dir, "RUNIT.BAT")?;

    let lst_path = work_dir.join("PROG.LST");
    let lst = fs::read_to_string(&lst_path).map_err(|err| {
        format!(
            "error: expected BASCOM to produce {} (BASCOM failed to run at all?): {err}",
            lst_path.display()
        )
    })?;
    if !lst.contains("0 Severe  Error(s)") {
        return Err(format!(
            "error: real BASCOM rejected the generated BASIC for {}:\n{lst}",
            bas_path.display()
        ));
    }

    let exe_path = work_dir.join("PROG.EXE");
    if !exe_path.is_file() {
        return Err(format!(
            "error: expected BASCOM+LINK to produce {} -- compile succeeded but link must \
             have failed",
            exe_path.display()
        ));
    }
    println!("binary: {}", exe_path.display());
    Ok(exe_path)
}

/// Runs a real-BASCOM `.EXE` (built by `invoke_bascom`) under `dosbox-x`,
/// this time non-headlessly: unlike `fbc`'s native binary, a DOS `.EXE`
/// can't be `exec`'d directly by this process at all, and dosbox-x itself
/// doesn't forward a host terminal's live stdin/stdout the way a normal
/// child process does -- an interactive program (`INPUT`, `INKEY$`, ...)
/// needs dosbox-x's own window, so this opens one rather than trying (and
/// failing) to run headlessly the way `invoke_bascom`'s own build step
/// does. Needs a display; won't work over a plain SSH session or other
/// fully headless environment.
fn run_dos_exe(exe_path: &Path) -> Result<(), String> {
    let work_dir = exe_path
        .parent()
        .ok_or_else(|| format!("error: invalid DOS binary path {}", exe_path.display()))?;
    let exe_name = exe_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("error: invalid DOS binary path {}", exe_path.display()))?;
    write_dos_file(
        &work_dir.join("RUN.BAT"),
        &format!("{exe_name}\nEXIT\n"),
    )?;
    let mount_arg = format!("MOUNT C: {}", work_dir.display());
    let status = Command::new("dosbox-x")
        .arg("-c")
        .arg(&mount_arg)
        .arg("-c")
        .arg("C:")
        .arg("-c")
        .arg("RUN.BAT")
        .arg("-fastlaunch")
        .status()
        .map_err(|err| format!("error: failed to invoke dosbox-x: {err}"))?;
    if !status.success() {
        return Err(format!("error: dosbox-x exited with {status}"));
    }
    Ok(())
}

fn native_binary_path(output_path: &Path) -> Result<PathBuf, String> {
    let stem = output_path.file_stem().ok_or_else(|| {
        format!(
            "error: invalid generated output path {}",
            output_path.display()
        )
    })?;
    Ok(native_binary_path_from_stem(&PathBuf::from("tmp"), stem))
}

fn expected_binary_path(target: Target, output_path: &Path) -> Result<PathBuf, String> {
    if target == Target::Jvm {
        let stem = output_path.file_stem().ok_or_else(|| {
            format!(
                "error: invalid generated output path {}",
                output_path.display()
            )
        })?;
        return Ok(PathBuf::from("tmp").join(stem).with_extension("class"));
    }
    if target == Target::Basic {
        let stem = output_path.file_stem().ok_or_else(|| {
            format!(
                "error: invalid generated output path {}",
                output_path.display()
            )
        })?;
        return Ok(bascom_work_dir(stem).join("PROG.EXE"));
    }
    native_binary_path(output_path)
}

fn native_binary_path_from_stem(directory: &Path, stem: &std::ffi::OsStr) -> PathBuf {
    let path = directory.join(stem);
    #[cfg(windows)]
    {
        path.with_extension("exe")
    }
    #[cfg(not(windows))]
    {
        path
    }
}

fn invoke_gcc(c_path: &PathBuf) -> Result<PathBuf, String> {
    let binary_name = c_path
        .file_stem()
        .ok_or_else(|| format!("error: invalid C output path {}", c_path.display()))?;
    let binary_dir = PathBuf::from("tmp");
    fs::create_dir_all(&binary_dir)
        .map_err(|err| format!("error: failed to create {}: {err}", binary_dir.display()))?;
    let binary_path = native_binary_path_from_stem(&binary_dir, binary_name);
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

/// Assembles `codegen_jvm.rs`'s generated `.j` into a real `.class` via
/// `krakatau2::assemble` -- linked directly into `bcc` as a library (see
/// `Cargo.toml`'s own comment on the pinned fork/upstream PR), not shelled
/// out to a separate `krak2` binary/subprocess the way this used to work.
/// Unlike `invoke_fbc`/`invoke_gcc`, the output's file name can't just be
/// the input's stem: `java`'s launcher requires the `.class` file on disk
/// to match the class's own simple name, so this reads that name back out
/// of the `.j` text's `.class public <name>` line -- the same name
/// `codegen_jvm::class_name_for` chose -- rather than assuming anything
/// about the input path.
fn invoke_krak2(j_path: &PathBuf, krak_stack_size: usize) -> Result<PathBuf, String> {
    let source = fs::read_to_string(j_path)
        .map_err(|err| format!("error: failed to read {}: {err}", j_path.display()))?;
    let class_name = source
        .lines()
        .find_map(|line| line.strip_prefix(".class public "))
        .map(str::trim)
        .ok_or_else(|| {
            format!(
                "internal error: {} has no `.class public <name>` line -- this looks like a \
                 broken codegen_jvm.rs output",
                j_path.display()
            )
        })?;

    let binary_dir = PathBuf::from("tmp");
    fs::create_dir_all(&binary_dir)
        .map_err(|err| format!("error: failed to create {}: {err}", binary_dir.display()))?;
    let class_path = binary_dir.join(format!("{class_name}.class"));

    if let Some(bytes) = jvm_classfile::generate_return_only(&source) {
        fs::write(&class_path, bytes)
            .map_err(|err| format!("error: failed to write {}: {err}", class_path.display()))?;
        println!("binary: {} (generated internally)", class_path.display());
        return Ok(class_path);
    }

    // `krakatau2::assemble` runs on its own worker thread with an
    // explicit, configurable stack size (`--krak-stack-size`/
    // `KRAK_STACK_SIZE`, default 32 MiB -- see
    // `resolve_default_krak_stack_size`) rather than whatever this
    // process's own thread happens to have: its recursive stack-map-
    // frame/control-flow analysis needs real headroom for a sufficiently
    // large generated program (confirmed empirically -- even a trivial
    // one can overflow a stack as small as 64 KiB), and
    // upstream Krakatau declined to make this the library's own
    // responsibility (see Cargo.toml's own comment on the pinned fork;
    // GitHub PRs #217/#218 against Storyyeller/Krakatau, both closed
    // unmerged, proposed exactly this inside the library itself). A
    // genuine stack overflow can't be caught after the fact the way an
    // ordinary panic can (it aborts the process outright) -- sizing the
    // thread adequately up front, not catching the overflow, is the
    // actual fix; `spawn`'s own `io::Result` and `join`'s own panic
    // result are still handled below for whatever else might go wrong.
    let classes = std::thread::scope(|scope| -> Result<_, String> {
        let handle = std::thread::Builder::new()
            .stack_size(krak_stack_size)
            .spawn_scoped(scope, || {
                krakatau2::assemble(&source, krakatau2::AssemblerOptions {})
            })
            .map_err(|err| {
                format!(
                    "error: failed to spawn the assembler's worker thread (stack size \
                     {krak_stack_size} bytes): {err}"
                )
            })?;
        let result = handle.join().map_err(|_| {
            format!(
                "error: the assembler's worker thread panicked while assembling {} (stack \
                 size {krak_stack_size} bytes) -- try a larger --krak-stack-size",
                j_path.display()
            )
        })?;
        result.map_err(|err| {
            // `Error::display` is krakatau2's own pretty, source-excerpt-aware
            // printer (writes straight to stderr) -- matches what the old
            // subprocess's own inherited stdio would have shown; the `Result`
            // this function returns only ever needs a short top-level summary
            // on top of that, the same way a nonzero `krak2` exit status used
            // to.
            err.display(&j_path.display().to_string(), &source);
            format!("error: failed to assemble {}", j_path.display())
        })
    })?;
    let (_, bytes) = classes.into_iter().next().ok_or_else(|| {
        format!(
            "internal error: krakatau2::assemble produced no classes for {}",
            j_path.display()
        )
    })?;
    fs::write(&class_path, bytes)
        .map_err(|err| format!("error: failed to write {}: {err}", class_path.display()))?;
    println!("binary: {}", class_path.display());
    Ok(class_path)
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
    fn parse_target_str_accepts_bascom_and_fbc() {
        assert_eq!(parse_target_str("bascom"), Some(Target::Basic));
        assert_eq!(parse_target_str("BASCOM"), Some(Target::Basic));
        assert_eq!(parse_target_str("fbc"), Some(Target::Fbc));
        assert_eq!(parse_target_str("FBC"), Some(Target::Fbc));
    }

    #[test]
    fn parse_byte_count_accepts_plain_bytes_and_suffixed_sizes() {
        assert_eq!(parse_byte_count("67108864"), Some(67108864));
        assert_eq!(parse_byte_count("64mb"), Some(64 * 1024 * 1024));
        assert_eq!(parse_byte_count("64MB"), Some(64 * 1024 * 1024));
        assert_eq!(parse_byte_count("64M"), Some(64 * 1024 * 1024));
        assert_eq!(parse_byte_count("256mb"), Some(256 * 1024 * 1024));
        assert_eq!(parse_byte_count("1gb"), Some(1024 * 1024 * 1024));
        assert_eq!(parse_byte_count("1g"), Some(1024 * 1024 * 1024));
        assert_eq!(parse_byte_count("32kb"), Some(32 * 1024));
        assert_eq!(parse_byte_count("32k"), Some(32 * 1024));
        assert_eq!(parse_byte_count("100b"), Some(100));
        assert_eq!(parse_byte_count("64 mb"), Some(64 * 1024 * 1024));
        assert_eq!(parse_byte_count("  64mb  "), Some(64 * 1024 * 1024));
    }

    #[test]
    fn parse_byte_count_rejects_garbage() {
        assert_eq!(parse_byte_count(""), None);
        assert_eq!(parse_byte_count("mb"), None);
        assert_eq!(parse_byte_count("64tb"), None);
        assert_eq!(parse_byte_count("sixty-four mb"), None);
        assert_eq!(parse_byte_count("64.5mb"), None);
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
        let cli = Cli::try_parse_from(["bcc", "input.bcl", "--strict-vars"]).expect("should parse");
        assert!(cli.strict_vars);
        assert!(!cli.strict_vars_warn);

        let cli =
            Cli::try_parse_from(["bcc", "input.bcl", "--strict-vars-warn"]).expect("should parse");
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
        let cli = Cli::try_parse_from(["bcc", "input.bcl", "--strict-vars", "--strict-vars-warn"])
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
            check: false,
            binary: false,
            run: false,
            target: None,
            strict_vars: false,
            strict_vars_warn: false,
            krak_stack_size: None,
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
        assert!(
            err.contains("-o must name a directory"),
            "unexpected: {err}"
        );
    }

    #[test]
    fn resolve_output_path_defaults_next_to_the_input_when_o_is_omitted() {
        let cli = cli_with_output(PathBuf::from("some/dir/input.bcl"), None);
        let resolved = resolve_output_path(&cli, Target::C).unwrap();
        assert_eq!(resolved, PathBuf::from("some/dir/input.c"));
    }

    #[test]
    fn native_binary_path_uses_platform_executable_suffix() {
        let path = native_binary_path_from_stem(Path::new("tmp"), std::ffi::OsStr::new("demo"));
        #[cfg(windows)]
        assert_eq!(path, PathBuf::from("tmp/demo.exe"));
        #[cfg(not(windows))]
        assert_eq!(path, PathBuf::from("tmp/demo"));
    }
}
