// Conformance groups: tutorials, basic, c

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

static REMLINE_RUNTIME_TEST_LOCK: Mutex<()> = Mutex::new(());

fn is_library_path(path: &Path) -> bool {
    path.components()
        .any(|c| matches!(c.as_os_str().to_str(), Some("com" | "lib")))
}

#[test]
fn compiles_every_example_bcl_file() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tutorial_dir = repo_root.join("tutorial");
    let output_dir = repo_root.join("output");

    let mut examples: Vec<PathBuf> = collect_example_sources(&tutorial_dir)
        .into_iter()
        .filter(|path| !is_library_path(path))
        .collect();
    examples.sort();

    assert!(
        !examples.is_empty(),
        "expected at least one .bcl file in {}",
        tutorial_dir.display()
    );

    for example in examples {
        compile_example(&example, &tutorial_dir, &output_dir);
    }
}

/// The normal CI build must exercise tutorials end to end, not only check
/// that their source parses.  These tutorials are deterministic and do not
/// require interactive input or external files, so they can be compiled and
/// run through the C backend on every build machine with gcc.
#[test]
fn c_target_builds_and_runs_noninteractive_tutorials() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping tutorial runtime checks: gcc is unavailable");
        return;
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tutorials = [
        "hello",
        "variables",
        "arithmetic",
        "conditions",
        "loops",
        "select_case",
        "functions",
        "arrays",
        "data",
        "require",
        "procedures",
        "short_circuit",
        "stdlib",
        "methods",
        "portable_error_handling",
        "restore_data",
    ];

    for stem in tutorials {
        let temp = tempfile::tempdir().expect("failed to create tutorial output directory");
        let mut output_dir = temp.path().join("out").into_os_string();
        output_dir.push("/");
        let mut command = Command::new(env!("CARGO_BIN_EXE_bcc"));
        command
            .arg(root.join("tutorial").join(format!("{stem}.bcl")))
            .arg("--target")
            .arg("c")
            .arg("--clean")
            .arg("--run")
            .arg("-L")
            .arg(root.join("tutorial").join("lib"))
            .arg("-o")
            .arg(output_dir);
        let output = command
            .current_dir(root)
            .output()
            .unwrap_or_else(|err| panic!("failed to invoke bcc for tutorial {stem}: {err}"));
        assert!(
            output.status.success(),
            "tutorial {stem} failed to build or run:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn c_semantic_multi_index_arrays_compile_and_run() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping multidimensional C runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create multidimensional array directory");
    let source_path = temp.path().join("rank_two.bcl");
    fs::write(
        &source_path,
        "program RankTwo\ndim grid%(2, 3)\ndim labels(2, 3) as string\ngrid%(2, 3) = 41\nlabels(2, 3) = \"ready\"\nprint grid%(2, 3)\nprint labels(2, 3)\nend\n",
    )
    .expect("failed to write multidimensional array source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for multidimensional C program");

    assert!(
        output.status.success(),
        "multidimensional C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("41\nready\n"),
        "unexpected multidimensional C output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_driver_transpiles_callable_array_indices_from_typed_ir() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping callable array-index C runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create callable array-index directory");
    let source_path = temp.path().join("callable_array_index.bcl");
    fs::write(
        &source_path,
        "program CallableArrayIndex\ndim values%(3)\nfunction index%()\nreturn 2\nend function\nvalues%(index%()) = 17\nprint values%(index%())\nend\n",
    )
    .expect("failed to write callable array-index source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for callable array-index program");

    assert!(
        output.status.success(),
        "callable array-index program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("17\n"),
        "unexpected callable array-index output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_driver_runs_nested_semantic_input_through_typed_ir() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping nested C INPUT runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create nested INPUT directory");
    let source_path = temp.path().join("nested_input.bcl");
    fs::write(
        &source_path,
        "program NestedInput\ndim values%(2, 3)\nfunction axis%(value%)\nreturn value%\nend function\nif true then\ninput \"value\"; values%(axis%(1), axis%(2))\nend if\nprint values%(1, 2)\nend\n",
    )
    .expect("failed to write nested INPUT source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let mut child = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to invoke bcc for nested INPUT program");
    child
        .stdin
        .take()
        .expect("compiler stdin should be piped to the generated program")
        .write_all(b"41\n")
        .expect("failed to provide nested INPUT value");
    let output = child
        .wait_with_output()
        .expect("failed to collect nested INPUT program output");
    assert!(
        output.status.success(),
        "nested INPUT program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("value? 41\n"),
        "unexpected nested INPUT output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_driver_runs_nested_typed_data_read() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping nested READ runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create nested READ directory");
    let source_path = temp.path().join("nested_read.bcl");
    fs::write(
        &source_path,
        "program NestedRead\ndata 42\nif true then\nread value%\nend if\nprint value%\nend\n",
    )
    .expect("failed to write nested READ source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for nested READ program");

    assert!(
        output.status.success(),
        "nested READ program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("42\n"),
        "unexpected nested READ output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_driver_runs_nested_typed_file_write_after_output_open() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping nested file WRITE runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create nested file WRITE directory");
    let source_path = temp.path().join("nested_file_write.bcl");
    let data_path = temp.path().join("typed-output.txt");
    fs::write(
        &source_path,
        "program NestedFileWrite\nopen \"typed-output.txt\" for output as #4\nif true then\nwrite #4, 42\nend if\nclose #4\nend\n",
    )
    .expect("failed to write nested file WRITE source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for nested file WRITE program");

    assert!(
        output.status.success(),
        "nested file WRITE program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&data_path).expect("generated program should write its output file"),
        "42\n"
    );
}

#[test]
fn c_driver_runs_callable_nested_typed_output_open() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping callable nested OUTPUT runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create callable OUTPUT directory");
    let source_path = temp.path().join("callable_output.bcl");
    let data_path = temp.path().join("typed-output.txt");
    fs::write(
        &source_path,
        "program CallableOutput\nprocedure writer()\nif true then\nopen \"typed-output.txt\" for output as #4\nend if\nwrite #4, 73\nclose #4\nend procedure\nwriter()\nend\n",
    )
    .expect("failed to write callable OUTPUT source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for callable OUTPUT program");

    assert!(
        output.status.success(),
        "callable OUTPUT program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&data_path).expect("generated procedure should write its output file"),
        "73\n"
    );
}

#[test]
fn c_semantic_dynamic_multi_index_arrays_compile_and_run() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping dynamic multidimensional C runtime check: gcc is unavailable");
        return;
    }

    let temp =
        tempfile::tempdir().expect("failed to create dynamic multidimensional array directory");
    let source_path = temp.path().join("dynamic_rank_two.bcl");
    fs::write(
        &source_path,
        "program DynamicRankTwo\nrows% = 2\ncols% = 3\ndim grid%(rows%, cols%)\ndim labels$(rows%, cols%)\ngrid%(1, 2) = 41\nlabels$(1, 2) = \"ready\"\nprint grid%(1, 2)\nprint labels$(1, 2)\nend\n",
    )
    .expect("failed to write dynamic multidimensional array source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for dynamic multidimensional C program");

    assert!(
        output.status.success(),
        "dynamic multidimensional C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("41\nready\n"),
        "unexpected dynamic multidimensional C output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_semantic_dynamic_array_allocates_inside_typed_branch() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping conditional dynamic C runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create conditional dynamic array directory");
    let source_path = temp.path().join("conditional_dynamic_array.bcl");
    fs::write(
        &source_path,
        "program ConditionalDynamicArray\nbound%=3\nflag%=1\nif flag% then\ndim values%(bound%)\nvalues%(3)=42\nend if\nprint values%(3)\nend\n",
    )
    .expect("failed to write conditional dynamic array source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for conditional dynamic C program");

    assert!(
        output.status.success(),
        "conditional dynamic C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("42\n"),
        "unexpected conditional dynamic C output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_semantic_callable_dynamic_array_uses_typed_parameter() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping callable dynamic C runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create callable dynamic array directory");
    let source_path = temp.path().join("callable_dynamic_array.bcl");
    fs::write(
        &source_path,
        "program CallableDynamicArray\nfunction local%(limit%)\ndim values%(limit%)\nvalues%(limit%)=42\nreturn values%(limit%)\nend function\nprint local%(3)\nend\n",
    )
    .expect("failed to write callable dynamic array source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for callable dynamic C program");

    assert!(
        output.status.success(),
        "callable dynamic C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("42\n"),
        "unexpected callable dynamic C output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_semantic_dynamic_array_evaluates_typed_bound_function_once() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping dynamic bound function C runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create bound function C directory");
    let source_path = temp.path().join("dynamic_bound_function.bcl");
    fs::write(
        &source_path,
        "program DynamicBoundFunction\ncount%=0\nfunction nextBound%()\nglobal count%\ncount%=count%+1\nreturn 3\nend function\ndim values%(nextBound%())\nvalues%(3)=42\nprint values%(3);\",\";count%\nend\n",
    )
    .expect("failed to write dynamic bound function source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for dynamic bound C program");

    assert!(
        output.status.success(),
        "dynamic bound C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("42,1\n"),
        "typed dynamic bound function should run once: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_semantic_callable_dynamic_array_evaluates_typed_bound_function_once() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping callable dynamic bound C runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create callable bound function C directory");
    let source_path = temp.path().join("callable_dynamic_bound_function.bcl");
    fs::write(
        &source_path,
        "program CallableDynamicBoundFunction\ncount%=0\nfunction nextBound%()\nglobal count%\ncount%=count%+1\nreturn 3\nend function\nfunction local%()\ndim values%(nextBound%())\nvalues%(3)=42\nreturn values%(3)\nend function\nresult%=local%()\nprint result%;\",\";count%\nend\n",
    )
    .expect("failed to write callable dynamic bound function source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for callable dynamic bound C program");

    assert!(
        output.status.success(),
        "callable dynamic bound C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("42,1\n"),
        "callable typed bound function should run once: stdout={} stderr={} generated={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        fs::read_to_string(temp.path().join("out/callable_dynamic_bound_function.c"))
            .unwrap_or_else(|error| format!("<unavailable: {error}>"))
    );
}

#[test]
fn c_semantic_dynamic_long_array_uses_suffix_inferred_element_type() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping dynamic LONG array C runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create dynamic LONG array directory");
    let source_path = temp.path().join("dynamic_long_array.bcl");
    fs::write(
        &source_path,
        "program DynamicLongArray\nbound&=3\ndim values&(bound&)\nvalues&(bound&)=123456789\nprint values&(bound&)\nend\n",
    )
    .expect("failed to write dynamic LONG array source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for dynamic LONG C program");

    assert!(
        output.status.success(),
        "dynamic LONG C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("123456789\n"),
        "suffix-inferred LONG array value was not preserved: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_semantic_dynamic_array_preserves_fixed_radix_axis_at_runtime() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping mixed-axis dynamic C runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create mixed-axis dynamic C directory");
    let source_path = temp.path().join("mixed_axis_dynamic_array.bcl");
    fs::write(
        &source_path,
        "program MixedAxisDynamicArray\nrow%=3\ndim grid%(&H10,row%)\ngrid%(16,3)=42\nprint grid%(16,3)\nend\n",
    )
    .expect("failed to write mixed-axis dynamic C source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for mixed-axis dynamic C program");

    assert!(
        output.status.success(),
        "mixed-axis dynamic C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("42\n"),
        "C dynamic array should retain its fixed radix axis: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_semantic_dynamic_array_preserves_fixed_octal_axis_at_runtime() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping mixed-axis octal C runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create octal mixed-axis C directory");
    let source_path = temp.path().join("octal_mixed_axis_array.bcl");
    fs::write(
        &source_path,
        "program OctalMixedAxisArray\nrow%=3\ndim grid%(&O20,row%)\ngrid%(16,3)=42\nprint grid%(16,3)\nend\n",
    )
    .expect("failed to write octal mixed-axis C source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for octal mixed-axis C program");

    assert!(
        output.status.success(),
        "octal mixed-axis C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("42\n"),
        "C dynamic array should retain its fixed octal axis: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_semantic_multidimensional_bounds_each_evaluate_once() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping multidimensional bound C runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create multidimensional bound C directory");
    let source_path = temp.path().join("multidimensional_bound_calls.bcl");
    fs::write(
        &source_path,
        "program MultidimensionalBoundCalls\ncount%=0\nfunction nextBound%()\nglobal count%\ncount%=count%+1\nreturn 3\nend function\ndim grid%(nextBound%(),nextBound%())\ngrid%(3,3)=42\nresult%=grid%(3,3)\nprint result%;\",\";count%\nend\n",
    )
    .expect("failed to write multidimensional bound source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for multidimensional bound C program");

    assert!(
        output.status.success(),
        "multidimensional bound C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("42,2\n"),
        "each typed C array axis bound should run exactly once: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_semantic_dynamic_dim_evaluates_typed_arithmetic_bound() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping arithmetic dynamic C runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create arithmetic dynamic C directory");
    let source_path = temp.path().join("arithmetic_dynamic_dim.bcl");
    fs::write(
        &source_path,
        "program ArithmeticDynamicDim\nbase%=2\ndim values%(base%+1)\nvalues%(3)=42\nprint values%(3)\nend\n",
    )
    .expect("failed to write arithmetic dynamic C source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for arithmetic dynamic C program");

    assert!(
        output.status.success(),
        "arithmetic dynamic C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("42\n"),
        "C dynamic DIM should evaluate its typed arithmetic bound: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_semantic_dynamic_array_preserves_zero_upper_bound() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping zero-bound C runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create zero-bound C directory");
    let source_path = temp.path().join("zero_bound_dynamic_array.bcl");
    fs::write(
        &source_path,
        "program ZeroBoundDynamicArray\nbound%=0\ndim values%(bound%)\nvalues%(0)=42\nprint values%(0)\nend\n",
    )
    .expect("failed to write zero-bound C source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for zero-bound C program");

    assert!(
        output.status.success(),
        "zero-bound C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("42\n"),
        "zero upper bound must allocate one element: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_semantic_skipped_dynamic_dim_does_not_evaluate_typed_bound() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping skipped-bound C runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create skipped-bound C directory");
    let source_path = temp.path().join("skipped_dynamic_dim.bcl");
    fs::write(
        &source_path,
        "program SkippedDynamicDim\ncount%=0\nfunction nextBound%()\nglobal count%\ncount%=count%+1\nreturn 3\nend function\nif 0 then\ndim values%(nextBound%())\nend if\nprint count%\nend\n",
    )
    .expect("failed to write skipped-bound C source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for skipped-bound C program");

    assert!(
        output.status.success(),
        "skipped-bound C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("0\n"),
        "a skipped typed C DIM must not evaluate its bound: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_string_intrinsics_compile_and_run_through_semantic_ir() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping string intrinsic runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create string intrinsic directory");
    let source_path = temp.path().join("string_intrinsics.bcl");
    fs::write(
        &source_path,
        "program StringIntrinsics\nprint chr$(65)\nprint str$(7)\nprint mid$(\"abcdef\", 2, 3)\nprint left$(\"abcdef\", 2)\nprint right$(\"abcdef\", 2)\nprint len(\"abc\")\nprint asc(\"A\")\nprint val(\"12.5\")\nprint instr(\"BASCAL\", \"CAL\")\nend\n",
    )
    .expect("failed to write string intrinsic source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for string intrinsic program");

    assert!(
        output.status.success(),
        "string intrinsic program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("A\n 7\nbcd\nab\nef\n3\n65\n12.5\n4\n"),
        "unexpected string intrinsic output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_numeric_user_function_calls_compile_and_run() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping numeric user function runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create user function directory");
    let source_path = temp.path().join("numeric_calls.bcl");
    fs::write(
        &source_path,
        "program NumericCalls\nfunction add%(left%, right% = 3)\nreturn left% + right%\nend function\nfunction double%(value%)\nreturn value% * 2\nend function\nfunction widen#(value#)\nreturn value#\nend function\nfunction bump%(byref value%)\nvalue% = value% + 1\nreturn value%\nend function\nresult% = double%(add%(3, 4))\nprint result%\nprint double%(add%(5, 6))\nprint add%(9)\nprint widen#(7)\ninitial% = 4\nprint bump%(initial%)\nprint initial%\nend\n",
    )
    .expect("failed to write user function source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for numeric user function program");

    assert!(
        output.status.success(),
        "numeric user function program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("14\n22\n12\n7\n5\n5\n"),
        "unexpected numeric user function output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_callable_end_exits_the_process() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping callable END runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create callable END directory");
    let source_path = temp.path().join("callable_end.bcl");
    fs::write(
        &source_path,
        "program CallableEndC\nfunction stop%()\nif 1 = 1 then\nend\nend if\nreturn 1\nend function\nprint \"before\"\nstop%()\nprint \"after\"\nend\n",
    )
    .expect("failed to write callable END source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for callable END program");

    assert!(
        output.status.success(),
        "callable END program failed to build or run:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n");
    assert!(stdout.ends_with("before\n"), "unexpected output: {stdout}");
    assert!(
        !stdout.contains("after"),
        "callable END must suppress statements after the call: {stdout}"
    );
}

#[test]
fn c_string_user_function_calls_compile_and_run() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping string user function runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create user function directory");
    let source_path = temp.path().join("string_calls.bcl");
    fs::write(
        &source_path,
        "program StringCalls\nfunction join$(left$, right$ = \"!\")\nreturn left$ + right$\nend function\nresult$ = join$(\"Hi\")\nprint result$\nprint join$(join$(\"A\"))\nend\n",
    )
    .expect("failed to write user function source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for string user function program");

    assert!(
        output.status.success(),
        "string user function program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("Hi!\nA!!\n"),
        "unexpected string user function output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_semantic_print_channel_writes_formatted_tokens() {
    if Command::new("gcc").arg("--version").output().is_err() {
        return;
    }
    let temp = tempfile::tempdir().expect("failed to create PRINT # directory");
    let source_path = temp.path().join("print_channel.bcl");
    fs::write(
        &source_path,
        "program PrintChannel\nopen \"print-channel.txt\" for output as #4\nprint #4, \"semantic\"; 99\nclose #4\nend\n",
    )
    .expect("failed to write PRINT # source");
    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for PRINT # program");
    assert!(
        output.status.success(),
        "PRINT # program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(temp.path().join("print-channel.txt")).unwrap(),
        "semantic99\n"
    );
}

#[test]
fn c_semantic_file_input_stores_indexed_array_values() {
    if Command::new("gcc").arg("--version").output().is_err() {
        return;
    }
    let temp = tempfile::tempdir().expect("failed to create INPUT # array directory");
    let source_path = temp.path().join("input_channel_array.bcl");
    fs::write(
        &source_path,
        "program InputChannelArray\ndim values%(2)\ndim labels$(2)\nopen \"input-channel.txt\" for input as #4\ninput #4, values%(2), labels$(1)\nclose #4\nprint values%(2)\nprint labels$(1)\nend\n",
    )
    .expect("failed to write INPUT # array source");
    fs::write(temp.path().join("input-channel.txt"), "41,\"ready\"\n")
        .expect("failed to write INPUT # data");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for INPUT # array program");
    assert!(
        output.status.success(),
        "INPUT # array program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("41\nready\n"),
        "unexpected INPUT # array output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn freebasic_runs_sort_driver_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("examples/sort_driver/sort_driver.bcl");
    let output_path = repo_root.join("output/sort_driver.bas");

    compile_with_cli(&source_path, &output_path, &["--clean", "--binary"]);

    let executable_path = repo_root.join("tmp/sort_driver");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled sort driver");
    assert!(
        run.status.success(),
        "compiled sort driver failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    for label in ["Bubble: OK", "Shaker: OK", "Shell: OK", "Quick: OK"] {
        assert_eq!(stdout.matches(label).count(), 1, "missing {label}");
    }
}

/// Classic resumable error handling is intentionally BASIC-only.  The
/// labels tutorial remains valid for the BASIC backend, but the C backend
/// must diagnose its `ON ERROR GOTO` rather than emit a partial translation.
#[test]
fn c_target_rejects_labels_and_error_handling_tutorial() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tutorial/labels_and_error_handling.bcl");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("C")
        .arg("--clean")
        .output()
        .expect("failed to invoke bcc");
    assert!(
        !output.status.success(),
        "bcc unexpectedly accepted {source_path:?} under --target C"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("`on error goto` is not supported with --target c"),
        "{stderr}"
    );
    assert!(stderr.contains("`try`/`catch`/`finally`"), "{stderr}");
}

/// End-to-end confirmation for GitHub issue #60's procedures-only C-target
/// propagation: a raise two call frames deep (inside `checkPart()`,
/// called by `editRecord()`, called from inside a `try`) reaches that
/// `try`'s own `catch`, correctly skipping the rest of both `checkPart()`
/// and `editRecord()` -- the scenario `--target basic` already handles for
/// free via real `ON ERROR GOTO`'s own global trap, and that `--target C`
/// needed `collect_try_reachable_procedures`'s `bcc_result_void`
/// propagation for (see codegen_c.rs). Skipped (not failed) when `gcc`
/// isn't available, matching this file's other C-target tests.
#[test]
fn gcc_runs_try_catch_through_nested_procedure_calls_under_c_target_when_available() {
    if Command::new("gcc").arg("--version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("nested_try.bcl");
    fs::write(
        &source_path,
        r#"program nestedTry

procedure checkPart()
    print "checking"
    open "missing.dat" for input as #1
    print "unreachable in checkPart"
end procedure

procedure editRecord()
    print "editing"
    checkPart()
    print "unreachable in editRecord"
end procedure

dim e%
dim l%

try
    editRecord()
catch e%, l%
    print "caught " + str$(e%) + " at " + str$(l%)
end try
print "after"
end
"#,
    )
    .unwrap();

    let output_dir = dir.path().join("out");
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));
    let mut dir_arg = output_dir.as_os_str().to_owned();
    dir_arg.push("/");

    let status = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("-o")
        .arg(&dir_arg)
        .arg("--target")
        .arg("C")
        .arg("--clean")
        .arg("--binary")
        .current_dir(repo_root)
        .status()
        .expect("failed to invoke bcc");
    assert!(
        status.success(),
        "bcc failed to compile/build {source_path:?} under --target C"
    );

    let executable_path = repo_root.join("tmp/nested_try");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled nested_try binary");
    assert!(
        run.status.success(),
        "compiled nested_try binary failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(stdout.contains("checking"), "{stdout}");
    assert!(stdout.contains("editing"), "{stdout}");
    assert!(!stdout.contains("unreachable"), "{stdout}");
    // `str$` matches real BASIC's own leading-space-for-non-negative
    // convention, so the failed input open's error 53 is printed as " 53".
    assert!(stdout.contains("caught  53 at"), "{stdout}");
    assert!(stdout.contains("after"), "{stdout}");
}

#[test]
fn c_semantic_throw_propagates_through_try_reachable_procedure() {
    if Command::new("gcc").arg("--version").output().is_err() {
        return;
    }
    let temp = tempfile::tempdir().expect("failed to create semantic THROW directory");
    let source_path = temp.path().join("semantic_throw.bcl");
    fs::write(
        &source_path,
        "program SemanticThrow\nprocedure fail()\nthrow 7\nend procedure\ntry\nfail()\ncatch e%, l%\nprint e%\nend try\nend\n",
    )
    .expect("failed to write semantic THROW source");
    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for semantic THROW program");
    assert!(
        output.status.success(),
        "semantic THROW program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("7\n"),
        "unexpected caught THROW output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_semantic_seek_positions_random_record_channel() {
    if Command::new("gcc").arg("--version").output().is_err() {
        return;
    }
    let temp = tempfile::tempdir().expect("failed to create semantic SEEK directory");
    let source_path = temp.path().join("semantic_seek.bcl");
    fs::write(
        &source_path,
        "program SemanticSeek\nopen \"records.dat\" for random as #1 len = 4\nfield #1, 4 as record$\nseek #1, 2\nprint #1, \"X\"\nclose #1\nend\n",
    )
    .expect("failed to write semantic SEEK source");
    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for semantic SEEK program");
    assert!(
        output.status.success(),
        "semantic SEEK program failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let bytes = fs::read(temp.path().join("records.dat")).expect("SEEK output file missing");
    assert_eq!(bytes, b"\0\0\0\0X\n");
}

#[test]
fn c_semantic_lset_packed_numeric_runtime_roundtrip() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping packed LSET runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create packed LSET directory");
    let source_path = temp.path().join("packed_lset.bcl");
    fs::write(
        &source_path,
        "program PackedLset\nopen \"records.dat\" for random as #1 len = 2\nfield #1, 2 as raw$\nlset raw$ = mki$(1234)\nput #1, 1\nget #1, 1\nprint cvi(raw$)\nclose #1\nend\n",
    )
    .expect("failed to write packed LSET source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for packed LSET program");

    assert!(
        output.status.success(),
        "packed LSET C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("1234\n"),
        "unexpected packed LSET round-trip output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_record_dsl_open_registers_synthesized_error_runtime() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping record DSL runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create record DSL directory");
    let source_path = temp.path().join("record_dsl_open.bcl");
    fs::write(
        &source_path,
        "program RecordDslOpen\nrecord R\nvalue: int16\nend record\nfile db as R = open(\"records.dat\")\ndb[1] = { value: 1234 }\nlet row = db[1]\nprint row.value\ndb.close()\nend\n",
    )
    .expect("failed to write record DSL source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for record DSL program");

    assert!(
        output.status.success(),
        "record DSL C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("1234\n"),
        "unexpected record DSL runtime output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_record_variable_write_roundtrips_all_typed_fields() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping record variable runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create record variable directory");
    let source_path = temp.path().join("record_variable_write.bcl");
    fs::write(
        &source_path,
        "program RecordVariableWrite\nfunction recordNumber%(index%)\nreturn index%\nend function\nfunction realRecord#(index#)\nreturn index#\nend function\nrecord Identity\nid: int16\nwide: int32\nend record\nrecord R combines Identity\nsmallFloat: float32\nlargeFloat: float64\nname: string(8)\nend record\nfile db as R = open(\"records.dat\")\ndim slots%(3)\nslots%(1) = 1\nslots%(2) = 2\nslots%(3) = 3\ndb[recordNumber%(1)] = { id: 4, wide: 30000, smallFloat: 1.5, largeFloat: 2.25, name: \"Ada\" }\nlet row = db[recordNumber%(1)]\nrow.id = 5\ndb[slots%(2)] = row\nprocedure save()\nglobal db\nlet localRow = db[recordNumber%(1)]\nlocalRow.wide = 32000\nlocalRow.name = \"Grace\"\ndb[realRecord#(3.0)] = (localRow)\nend procedure\nsave()\nlet first = db[recordNumber%(1)]\nlet second = db[recordNumber%(2)]\nlet third = db[recordNumber%(3)]\nprint first.id\nprint first.wide\nprint first.smallFloat\nprint first.largeFloat\nprint first.name\nprint second.id\nprint second.wide\nprint second.smallFloat\nprint second.largeFloat\nprint second.name\nprint third.id\nprint third.wide\nprint third.smallFloat\nprint third.largeFloat\nprint third.name\ndb.close()\nend\n",
    )
    .expect("failed to write record variable source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for record variable program");

    assert!(
        output.status.success(),
        "record variable C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with(
            "4\n30000\n1.5\n2.25\nAda\n5\n30000\n1.5\n2.25\nAda\n4\n32000\n1.5\n2.25\nGrace\n"
        ),
        "unexpected record variable round-trip output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_semantic_lset_packed_numeric_record_dsl_roundtrip() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping record DSL packed LSET runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create record DSL packed LSET directory");
    let source_path = temp.path().join("record_dsl_packed_lset.bcl");
    fs::write(
        &source_path,
        "program PackedRecordLset\nrecord R\ni16: int16\ni32: int32\nf32: float32\nf64: float64\nend record\nfile db as R = open(\"records.dat\")\nlset dbI16Buf$ = mki$(1234)\nlset dbI32Buf$ = mkl$(1234567)\nlset dbF32Buf$ = mks$(1.25)\nlset dbF64Buf$ = mkd$(9.5)\nput #1, 1\nget #1, 1\nprint cvi(dbI16Buf$)\nprint cvl(dbI32Buf$)\nprint cvs(dbF32Buf$)\nprint cvd(dbF64Buf$)\ndb.close()\nend\n",
    )
    .expect("failed to write record DSL packed LSET source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for record DSL packed LSET program");

    assert!(
        output.status.success(),
        "record DSL packed LSET C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).ends_with("1234\n1234567\n1.25\n9.5\n"),
        "unexpected record DSL packed LSET output: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn c_record_dsl_partial_update_preserves_omitted_field_runtime() {
    if Command::new("gcc").arg("--version").output().is_err() {
        eprintln!("skipping record DSL partial update runtime check: gcc is unavailable");
        return;
    }

    let temp = tempfile::tempdir().expect("failed to create partial record update directory");
    let source_path = temp.path().join("record_dsl_partial_update.bcl");
    fs::write(
        &source_path,
        "program PartialRecordUpdate\nrecord R\nid: int16\nwide: int32\nsmallFloat: float32\nlargeFloat: float64\nname: string(8)\nend record\nfile db as R = open(\"records.dat\")\nprocedure show()\nglobal db\nlet row = db[1]\nprint row.id\nprint row.wide\nprint row.smallFloat\nprint row.largeFloat\nprint row.name\nend procedure\ndb[1] = { id: 4, wide: 30000, smallFloat: 1.5, largeFloat: 2.25, name: \"Ada\" }\ndb[1] = ?{ id: 9 }\nlet topRow = db[1]\nprint topRow.id\nprint topRow.wide\nprint topRow.smallFloat\nprint topRow.largeFloat\nprint topRow.name\nshow()\ndb.close()\nend\n",
    )
    .expect("failed to write partial record update source");

    let mut output_dir = temp.path().join("out").into_os_string();
    output_dir.push("/");
    let output = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("--target")
        .arg("c")
        .arg("--clean")
        .arg("--run")
        .arg("-o")
        .arg(output_dir)
        .current_dir(temp.path())
        .output()
        .expect("failed to invoke bcc for partial record update program");

    assert!(
        output.status.success(),
        "partial record update C program failed to build or run:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .ends_with("9\n30000\n1.5\n2.25\nAda\n9\n30000\n1.5\n2.25\nAda\n"),
        "partial update did not preserve the omitted field: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

/// End-to-end confirmation that the real case-study program (issue #66's
/// try/catch migration) actually runs correctly under `--target C`, not
/// just compiles -- exercising the full chain landed to get it there:
/// try/catch propagation through checkPart()/editRecord()/etc. (#60),
/// `TAB`/`SPC`/`STOP`/`SYSTEM`, a real non-blocking `INKEY$` that doesn't
/// break `INPUT`'s own buffered reads on the same stdin (`bcc_inkey`'s
/// own doc comment), a top-level `const` correctly readable from inside
/// a procedure (`collect_top_level_const_c_names` -- `partCount%` was
/// reading as `0` inside `showMainMenu()`/`initializeInventoryFileIfNew()`
/// before that fix), and `initializeInventoryFileIfNew()` itself
/// (pre-populating a brand-new `inven.dat` so it doesn't have to be
/// supplied by hand). Skipped (not failed) when `gcc` isn't available,
/// matching this file's other C-target tests.
///
/// This test drives the compiled binary's
/// `INKEY$` prompts by piping keystrokes into its stdin, which works on
/// POSIX because `bcc_inkey`'s non-Windows arm does a real
/// `read(STDIN_FILENO, ...)` -- but on Windows `bcc_inkey` uses
/// `_kbhit()`/`_getch()` (see `INKEY_BODY` in `codegen_c.rs`), which read
/// the actual console input buffer directly and never see bytes written
/// to a redirected/piped stdin at all. Against a pipe, `_kbhit()` simply
/// never reports a key, so any loop waiting on `INKEY$` spins forever --
/// this is a real, structural platform gap (a compiled BASCAL program
/// with `INKEY$` genuinely cannot be scripted via piped input on Windows
/// today, not just this test), not a flaky-CI issue, so there's no retry
/// or longer timeout that would fix it. Tracked as a real backend gap
/// separately from this test skip -- see GitHub issue #94.
#[test]
fn gcc_runs_inventory_tutorial_under_c_target_when_available() {
    if Command::new("gcc").arg("--version").output().is_err() {
        return;
    }
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tutorial/inventory.bcl");
    let dir = tempfile::tempdir().unwrap();
    let output_dir = dir.path().join("out");
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));
    let mut dir_arg = output_dir.as_os_str().to_owned();
    dir_arg.push("/");

    let status = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("-o")
        .arg(&dir_arg)
        .arg("--target")
        .arg("C")
        .arg("--clean")
        .arg("--binary")
        .current_dir(repo_root)
        .status()
        .expect("failed to invoke bcc");
    assert!(
        status.success(),
        "bcc failed to compile/build {source_path:?} under --target C"
    );

    let executable_path = repo_root.join("tmp/inventory");
    // Run from a fresh temp dir (not the repo root) so the freshly
    // written inven.dat lands there instead of polluting the repo.
    // Keystrokes: "1" selects "check a part" (INKEY$, one raw byte);
    // "1\n" is the part-number INPUT line (1 is still an empty slot,
    // since initializeInventoryFileIfNew() just populated it); "x"
    // dismisses the "press any key" prompt; a second "x" exits (option 7,
    // eXit to system -- the old separate "Quit to BASIC" option 7/Q was
    // dropped, since it meant nothing for a compiled program with no
    // interpreter to return to).
    let mut child = Command::new(&executable_path)
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn compiled inventory binary");
    child
        .stdin
        .take()
        .expect("child stdin should be piped")
        .write_all(b"11\nxx")
        .expect("failed to write keystrokes to inventory binary");
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if child
            .try_wait()
            .expect("failed to poll inventory binary")
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("compiled inventory binary timed out after 30 seconds");
        }
        thread::sleep(Duration::from_millis(50));
    }
    let run = child
        .wait_with_output()
        .expect("failed to collect inventory binary output");
    assert!(
        run.status.success(),
        "compiled inventory binary failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(stdout.contains("Inventory Program"), "{stdout}");
    // `str$` matches real BASIC's own leading-space-for-non-negative
    // convention: "L)ist all" + " 100" + "parts".
    assert!(stdout.contains("L)ist all 100parts"), "{stdout}");
    assert!(stdout.contains("Input part number"), "{stdout}");
    assert!(
        stdout.contains("Part number 1is still a null entry at this time"),
        "{stdout}"
    );

    let inven_dat = dir.path().join("inven.dat");
    let metadata = fs::metadata(&inven_dat)
        .unwrap_or_else(|err| panic!("expected {} to exist: {err}", inven_dat.display()));
    // 100 records * 39 bytes each (1 flag + 30 desc + 2 qty + 2 reorder +
    // 4 price) -- confirms initializeInventoryFileIfNew() actually wrote
    // all 100 blank records, not zero (the very bug this test exists to
    // catch: partCount% reading as 0 inside the procedure made `for i% =
    // 1 to partCount%` a no-op, leaving inven.dat empty).
    assert_eq!(
        metadata.len(),
        3900,
        "inven.dat should hold 100 blank records"
    );
}

/// Regression test for a real, source-level bug in `tutorial/inventory.bcl`
/// itself (present identically under every target -- `--target basic`,
/// `--target C`, `--target jvm` -- since it's a print/`LOCATE` sequencing
/// bug, not a codegen one): `printListHeader()` used to `LOCATE 25, 1` and
/// print "Press the AnyKey to scroll listing..." immediately, before any
/// item had been listed -- nothing actually paused there, so the very next
/// statement (`listAll()`'s first item) kept printing from that same
/// cursor position, gluing item 1 onto the end of that line. Every 20
/// items, `waitAnyKey()` then did `LOCATE 25, 10` -- rewinding the cursor
/// back to that *same* row -- and overwrote from column 10 onward with
/// "Press the AnyKey to continue...". Since the first prompt started at
/// column 1, columns 1-9 ("Press the") survived underneath the second
/// message, producing `"Press thePress the AnyKey to continue..."` on any
/// terminal tall enough that the collision isn't scrolled away by
/// accident before it's ever visible (BASCAL has no `VIEW PRINT`
/// scroll-region support -- see the tutorial's own header note -- so
/// nothing bounds where a fixed-row prompt can collide with unbounded
/// scrolling content). Fixed by dropping the premature prompt from
/// `printListHeader()`/`printReorderHeader()` and redrawing the header on
/// each new page after `waitAnyKey()`, so `waitAnyKey()`'s own row-25
/// write always lands on a freshly-cleared row. This test needs no pty
/// (the bug is pure print/CLS ordering, not terminal-size-dependent
/// timing): pipes are sufficient to catch a regression back to the
/// glued/doubled text.
#[test]
fn gcc_runs_inventory_list_all_without_a_garbled_press_any_key_prompt_when_available() {
    if Command::new("gcc").arg("--version").output().is_err() {
        return;
    }
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = tempfile::tempdir().unwrap();
    // A distinct filename stem from `gcc_runs_inventory_tutorial_under_c_
    // target_when_available`'s own `tutorial/inventory.bcl` compile --
    // `bcc --binary`'s output always lands at `tmp/<stem>` regardless of
    // `-o` (see `native_binary_path_from_stem` in main.rs), a fixed,
    // repo-relative path shared across the whole test binary; two tests
    // compiling the same stem in parallel race on that one file.
    let source_path = dir.path().join("inventory_list_all.bcl");
    fs::copy(repo_root.join("tutorial/inventory.bcl"), &source_path)
        .expect("failed to copy tutorial/inventory.bcl");
    let output_dir = dir.path().join("out");
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));
    let mut dir_arg = output_dir.as_os_str().to_owned();
    dir_arg.push("/");

    let status = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("-o")
        .arg(&dir_arg)
        .arg("--target")
        .arg("C")
        .arg("--clean")
        .arg("--binary")
        .current_dir(repo_root)
        .status()
        .expect("failed to invoke bcc");
    assert!(
        status.success(),
        "bcc failed to compile/build {source_path:?} under --target C"
    );

    let executable_path = repo_root.join("tmp/inventory_list_all");
    // "3" selects "List all" (100 parts, 20 per page -- 5 pages, 5
    // `waitAnyKey()` calls); one "x" per page to dismiss its prompt, then
    // "7" to exit back at the main menu.
    let mut child = Command::new(&executable_path)
        .current_dir(dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn compiled inventory binary");
    child
        .stdin
        .take()
        .expect("child stdin should be piped")
        .write_all(b"3xxxxx7")
        .expect("failed to write keystrokes to inventory binary");
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if child
            .try_wait()
            .expect("failed to poll inventory binary")
            .is_some()
        {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("compiled inventory binary timed out after 30 seconds");
        }
        thread::sleep(Duration::from_millis(50));
    }
    let run = child
        .wait_with_output()
        .expect("failed to collect inventory binary output");
    assert!(
        run.status.success(),
        "compiled inventory binary failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        !stdout.contains("Press thePress the"),
        "the initial header prompt and waitAnyKey()'s own prompt collided \
         on the same row:\n{stdout}"
    );
    assert_eq!(
        stdout.matches("I N V E N T O R Y   L I S T I N G").count(),
        5,
        "expected one freshly-redrawn header per page (5 pages of 20 for \
         100 parts):\n{stdout}"
    );
    assert_eq!(
        stdout.matches("Press the AnyKey to continue").count(),
        5,
        "expected exactly one wait prompt per page, each on its own \
         freshly-cleared row:\n{stdout}"
    );
}

/// GitHub issue #29's own acceptance criterion: `LINE INPUT #` into a
/// `dim`'d string array element (`rawLine$(lineCount%)` in
/// `examples/remline/com/bascal/examples/remline/transform.bcl`) now
/// compiles and runs correctly under `--target c`, producing output
/// identical to `--target basic`'s own (see
/// `freebasic_runs_remline_when_available`'s matching assertion against
/// the same `examples/remline/sample/expected.bas` fixture).
#[test]
fn gcc_runs_remline_under_c_target_when_available() {
    if Command::new("gcc").arg("--version").output().is_err() {
        return;
    }
    let _guard = REMLINE_RUNTIME_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("examples/remline/remline.bcl");
    let sample_output_path = repo_root.join("examples/remline/sample/output.bas");
    let output_dir = repo_root.join("output/c_target_remline");
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));
    let mut dir_arg = output_dir.as_os_str().to_owned();
    dir_arg.push("/");

    let _ = fs::remove_file(&sample_output_path);

    let status = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("-o")
        .arg(&dir_arg)
        .arg("--target")
        .arg("C")
        .arg("-L")
        .arg("examples/remline")
        .arg("--clean")
        .arg("--binary")
        .status()
        .expect("failed to invoke bcc");
    assert!(
        status.success(),
        "bcc failed to compile/build {source_path:?} under --target C"
    );

    let executable_path = repo_root.join("tmp/remline");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled remline binary");
    assert!(
        run.status.success(),
        "compiled remline binary failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let expected = fs::read_to_string(repo_root.join("examples/remline/sample/expected.bas"))
        .expect("expected output should be readable");
    let actual = fs::read_to_string(&sample_output_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", sample_output_path.display()));
    assert_eq!(
        normalize_newlines(&actual),
        normalize_newlines(&expected),
        "remline output under --target c should match the sample expectation"
    );
}

#[test]
fn freebasic_runs_mid_assign_edge_cases_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tests/fixtures/mid_assign_edge_cases.bcl");
    let output_path = repo_root.join("output/mid_assign_edge_cases.bas");

    compile_with_cli(&source_path, &output_path, &["--clean", "--binary"]);

    let executable_path = repo_root.join("tmp/mid_assign_edge_cases");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled mid_assign_edge_cases");
    assert!(
        run.status.success(),
        "compiled mid_assign_edge_cases failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    let lines: Vec<&str> = stdout.lines().map(str::trim).collect();
    assert_eq!(
        lines,
        vec![
            "01XY456789", // repl$ shorter than len: only LEN(repl$) chars overwritten
            "ABC3456789", // repl$ longer than len: truncated to len
            "0123456789", // repl$ empty: no-op, length preserved
            "012345678Z", // 2-arg form, pos at the very end of the string
        ],
        "MID$ assignment edge cases produced unexpected output:\n{stdout}"
    );
}

#[test]
fn freebasic_runs_semantic_mid_assign_byref_operand_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tests/fixtures/mid_assign_byref.bcl");
    let output_path = repo_root.join("output/mid_assign_byref.bas");
    compile_with_cli(&source_path, &output_path, &["--clean", "--binary"]);

    let executable_path = repo_root.join("tmp/mid_assign_byref");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled mid_assign_byref");
    assert!(
        run.status.success(),
        "compiled mid_assign_byref failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    let lines: Vec<&str> = stdout.lines().map(str::trim).collect();
    assert_eq!(lines, ["aXYdef", "2"], "ByRef MID$ operand output mismatch");
}

#[test]
fn freebasic_runs_semantic_mid_assign_dynamic_target_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tests/fixtures/mid_assign_dynamic_target.bcl");
    let output_path = repo_root.join("output/mid_assign_dynamic_target.bas");
    compile_with_cli(&source_path, &output_path, &["--clean", "--binary"]);

    let executable_path = repo_root.join("tmp/mid_assign_dynamic_target");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled mid_assign_dynamic_target");
    assert!(
        run.status.success(),
        "compiled mid_assign_dynamic_target failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    let lines: Vec<&str> = stdout.lines().map(str::trim).collect();
    assert_eq!(
        lines,
        ["aXYdefgh", "1"],
        "dynamic MID$ target output mismatch"
    );
}

#[test]
fn freebasic_runs_semantic_mid_assign_array_argument_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tests/fixtures/mid_assign_array_argument.bcl");
    let output_path = repo_root.join("output/mid_assign_array_argument.bas");
    compile_with_cli(&source_path, &output_path, &["--clean", "--binary"]);

    let executable_path = repo_root.join("tmp/mid_assign_array_argument");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled mid_assign_array_argument");
    assert!(
        run.status.success(),
        "compiled mid_assign_array_argument failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    let lines: Vec<&str> = stdout.lines().map(str::trim).collect();
    assert_eq!(
        lines,
        ["aXYdef", "2"],
        "array-argument MID$ output mismatch"
    );
}

/// `continue` across every loop shape that needs its own continue target
/// (see codegen_basic.rs's `loop_continue_stack`/codegen_c.rs's own field
/// of the same name) -- specifically the do-loop-with-a-post-condition
/// case, which had a real bug on the C backend while `continue` was
/// being added: `Statement::Do` always compiles to a `while (1) { ...;
/// guard }` shape there, never a native `do { } while (...)`, so a bare
/// C `continue;` skipped the post-condition guard entirely and looped
/// forever. Real `fbc`, not just bcc's own unit tests, since the BASIC
/// backend's own `do` codegen has the same "guard runs after the body"
/// shape and deserves the same real-compiler check the C-target bug was
/// actually found under.
#[test]
fn freebasic_runs_continue_across_every_loop_kind_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tests/fixtures/continue_all_loop_kinds.bcl");
    let output_path = repo_root.join("output/continue_all_loop_kinds.bas");

    compile_with_cli(&source_path, &output_path, &["--clean", "--binary"]);

    let executable_path = repo_root.join("tmp/continue_all_loop_kinds");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled continue_all_loop_kinds");
    assert!(
        run.status.success(),
        "compiled continue_all_loop_kinds failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    let lines: Vec<&str> = stdout.lines().map(str::trim).collect();
    assert_eq!(
        lines,
        vec![
            "for:  25",
            "while:  25",
            "do-while(pre):  25",
            "do-loop-until(post):  25",
            "do-loop-while(post):  25",
        ],
        "continue should skip even numbers in every loop shape, leaving the sum of odd \
         numbers 1..10 (25) in each case:\n{stdout}"
    );
}

/// `MID$(...) = ...` statement-form assignment under `--target C`, checked
/// against the same real-BASCOM-verified expectation
/// `freebasic_runs_mid_assign_edge_cases_when_available` already pins for
/// FreeBASIC and `tests/dosbox_conformance.rs`'s
/// `mid_assign_matches_real_bascom` pins for real BASCOM 2.00 -- one
/// fixture, three backends, same expected output. Skipped (not failed) when
/// `gcc` isn't available, matching this file's other C-target tests.
#[test]
fn gcc_runs_mid_assign_conformance_fixture_under_c_target_when_available() {
    if Command::new("gcc").arg("--version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tests/fixtures/conformance/mid_assign.bcl");
    let dir = tempfile::tempdir().unwrap();
    let output_dir = dir.path().join("out");
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));
    let mut dir_arg = output_dir.as_os_str().to_owned();
    dir_arg.push("/");

    let status = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("-o")
        .arg(&dir_arg)
        .arg("--target")
        .arg("C")
        .arg("--clean")
        .arg("--binary")
        .current_dir(repo_root)
        .status()
        .expect("failed to invoke bcc");
    assert!(
        status.success(),
        "bcc failed to compile/build {source_path:?} under --target C"
    );

    let executable_path = repo_root.join("tmp/mid_assign");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled mid_assign binary");
    assert!(
        run.status.success(),
        "compiled mid_assign binary failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let expected =
        fs::read_to_string(repo_root.join("tests/fixtures/conformance/mid_assign.expected.txt"))
            .expect("expected output should be readable");
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert_eq!(
        normalize_newlines(&stdout),
        normalize_newlines(&expected),
        "MID$ assignment under --target c should match the real-BASCOM-verified expectation"
    );
}

/// `"MM-DD-YYYY"`, zero-padded, real MBASIC/BASCOM's own fixed `DATE$`
/// format -- shared by the C and JVM `DATE$` regression tests (`bcc_date`
/// in codegen_c.rs; `java.time.LocalDate`/`DateTimeFormatter` in
/// codegen_jvm.rs). Can't check an exact value (today's actual date), so
/// this just checks the shape: two digits, a dash, two digits, a dash, four
/// digits, all numeric.
fn assert_looks_like_date_dollar(value: &str) {
    let parts: Vec<&str> = value.trim().split('-').collect();
    assert_eq!(parts.len(), 3, "expected MM-DD-YYYY, got {value:?}");
    assert_eq!(parts[0].len(), 2, "expected 2-digit month, got {value:?}");
    assert_eq!(parts[1].len(), 2, "expected 2-digit day, got {value:?}");
    assert_eq!(parts[2].len(), 4, "expected 4-digit year, got {value:?}");
    assert!(
        parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())),
        "expected only digits and dashes, got {value:?}"
    );
}

/// `DATE$` under `--target C`. `codegen_c.rs`'s own `bcc_date` used to not
/// exist at all -- `date$` silently fell through to an ordinary, always-
/// empty auto-declared string variable (see `register_var`'s matching
/// skip for why that's excluded now). Skipped (not failed) when `gcc` isn't
/// available, matching this file's other C-target tests.
#[test]
fn gcc_runs_date_dollar_under_c_target_when_available() {
    if Command::new("gcc").arg("--version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = tempfile::tempdir().unwrap();
    let source_path = dir.path().join("date_dollar.bcl");
    fs::write(&source_path, "program dateDollar\nprint date$\nend\n").unwrap();
    let output_dir = dir.path().join("out");
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));
    let mut dir_arg = output_dir.as_os_str().to_owned();
    dir_arg.push("/");

    let status = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("-o")
        .arg(&dir_arg)
        .arg("--target")
        .arg("C")
        .arg("--clean")
        .arg("--binary")
        .current_dir(repo_root)
        .status()
        .expect("failed to invoke bcc");
    assert!(
        status.success(),
        "bcc failed to compile/build {source_path:?} under --target C"
    );

    let executable_path = repo_root.join("tmp/date_dollar");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled date_dollar binary");
    assert!(
        run.status.success(),
        "compiled date_dollar binary failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_looks_like_date_dollar(&String::from_utf8_lossy(&run.stdout));
}

#[test]
fn freebasic_runs_self_referential_string_concatenation_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tests/fixtures/string_self_concat.bcl");
    let output_path = repo_root.join("output/string_self_concat.bas");

    compile_with_cli(&source_path, &output_path, &["--clean", "--binary"]);

    let executable_path = repo_root.join("tmp/string_self_concat");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled string_self_concat");
    assert!(
        run.status.success(),
        "compiled string_self_concat failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let stdout = String::from_utf8_lossy(&run.stdout);
    let lines: Vec<&str> = stdout.lines().map(str::trim).collect();
    assert_eq!(
        lines,
        vec!["abcdefghij", "abcdefgghij", "[ij]abcdefg", "fg-a-fghij"],
        "self-referential LEFT$/MID$/RIGHT$ concatenation produced unexpected output:\n{stdout}"
    );
}

/// Third leg of GitHub issue #38's own explicit ask (FreeBASIC, real
/// BASCOM 2.00, and C) -- the other two live in `tests/dosbox_conformance.rs`
/// (`builtin_scalar_methods_match_real_bascom`/`_match_c_target`), all three
/// checked against the same golden file.
#[test]
fn freebasic_runs_builtin_scalar_methods_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tests/fixtures/builtin_scalar_methods.bcl");
    let output_path = repo_root.join("output/builtin_scalar_methods.bas");

    compile_with_cli(&source_path, &output_path, &["--clean", "--binary"]);

    let executable_path = repo_root.join("tmp/builtin_scalar_methods");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled builtin_scalar_methods");
    assert!(
        run.status.success(),
        "compiled builtin_scalar_methods failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let expected = fs::read_to_string(
        repo_root.join("tests/fixtures/conformance/builtin_scalar_methods.expected.txt"),
    )
    .expect("expected output should be readable");
    let actual = String::from_utf8_lossy(&run.stdout);
    assert_eq!(
        normalize_newlines(&actual),
        normalize_newlines(&expected),
        "FreeBASIC output for built-in scalar methods should match the golden expectation"
    );
}

/// GitHub issue #41: third leg (FreeBASIC) for com.bascal.stdlib's
/// ltrim$/rtrim$/ucase$/lcase$, now scalar methods -- the other two live
/// in tests/dosbox_conformance.rs
/// (`stdlib_functions_match_real_bascom`/`_match_c_target`), all three
/// checked against the same golden file.
#[test]
fn freebasic_runs_stdlib_functions_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("tests/fixtures/stdlib_functions.bcl");
    let output_path = repo_root.join("output/stdlib_functions.bas");

    compile_with_cli(&source_path, &output_path, &["--clean", "--binary"]);

    let executable_path = repo_root.join("tmp/stdlib_functions");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled stdlib_functions");
    assert!(
        run.status.success(),
        "compiled stdlib_functions failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let expected = fs::read_to_string(
        repo_root.join("tests/fixtures/conformance/stdlib_functions.expected.txt"),
    )
    .expect("expected output should be readable");
    let actual = String::from_utf8_lossy(&run.stdout);
    assert_eq!(
        normalize_newlines(&actual),
        normalize_newlines(&expected),
        "FreeBASIC output for com.bascal.stdlib should match the golden expectation"
    );
}

#[test]
fn freebasic_runs_remline_when_available() {
    if Command::new("fbc").arg("-version").output().is_err() {
        return;
    }
    let _guard = REMLINE_RUNTIME_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = repo_root.join("examples/remline/remline.bcl");
    let output_path = repo_root.join("output/remline/remline.bas");
    let sample_output_path = repo_root.join("examples/remline/sample/output.bas");

    let _ = fs::remove_file(&sample_output_path);

    compile_with_cli(
        &source_path,
        &output_path,
        &["-L", "examples/remline", "--clean", "--binary"],
    );

    let executable_path = repo_root.join("tmp/remline");
    let run = Command::new(&executable_path)
        .output()
        .expect("failed to run compiled remline example");
    assert!(
        run.status.success(),
        "compiled remline example failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    let expected = fs::read_to_string(repo_root.join("examples/remline/sample/expected.bas"))
        .expect("expected output should be readable");
    let actual = fs::read_to_string(&sample_output_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", sample_output_path.display()));
    assert_eq!(
        normalize_newlines(&actual),
        normalize_newlines(&expected),
        "remline output should match the sample expectation"
    );
}

fn collect_example_sources(dir: &Path) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    collect_example_sources_recursive(dir, &mut sources);
    sources
}

fn collect_example_sources_recursive(dir: &Path, sources: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|err| {
        panic!("failed to read {}: {err}", dir.display());
    });

    for entry in entries {
        let path = entry
            .unwrap_or_else(|err| panic!("failed to read entry in {}: {err}", dir.display()))
            .path();

        if path.is_dir() {
            collect_example_sources_recursive(&path, sources);
            continue;
        }

        if path.extension().is_some_and(|extension| extension == "bcl") {
            sources.push(path);
        }
    }
}

fn compile_example(path: &Path, tutorial_dir: &Path, output_dir: &Path) {
    let mut options = bcc::CompileOptions::new();
    // Make any sibling `lib/` directory available as a search root.
    if let Some(parent) = path.parent() {
        let lib_dir = parent.join("lib");
        if lib_dir.is_dir() {
            options.library_dirs.push(lib_dir);
        }
    }
    let output = match bcc::compile_file(path, &options) {
        Ok(o) => o,
        Err(ref diagnostics)
            if diagnostics.iter().all(|d| {
                d.message
                    .contains("`shared` declaration is only valid in shared-variable files")
            }) =>
        {
            return; // shared-variables file — not a standalone compilable program
        }
        Err(diagnostics) => {
            panic!("failed to compile {}:\n{diagnostics:#?}", path.display())
        }
    };

    let output_path = output_path_for_source(path, tutorial_dir, output_dir);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
    }
    fs::write(&output_path, &output)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", output_path.display()));

    assert!(
        output.contains("' BASCAL generated BASIC -- DO NOT EDIT"),
        "{} should produce generated BASIC",
        path.display()
    );
    assert!(
        output.lines().all(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with('\'')
                || (!trimmed.to_ascii_lowercase().starts_with("function ")
                    && !trimmed.to_ascii_lowercase().starts_with("end function"))
        }),
        "{} should not emit structured BASIC functions",
        path.display()
    );
    assert!(
        !output.contains("FN_") && !output.contains("IF_"),
        "{} should not expose symbolic labels",
        path.display()
    );
    assert_branch_targets_are_numeric(&output, path);
}

/// `output_path` names the exact file `bcc` is expected to produce --
/// `-o` itself only ever accepts a directory (auto-naming the file inside
/// it from the input's own stem), so this passes `output_path`'s parent
/// directory instead and relies on that auto-naming landing on exactly
/// `output_path`, which it always does here since every caller already
/// names it `<input stem>.bas`.
fn compile_with_cli(source_path: &Path, output_path: &Path, extra_args: &[&str]) {
    let parent = output_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
    let mut dir_arg = parent.as_os_str().to_owned();
    dir_arg.push("/");

    let mut command = Command::new(env!("CARGO_BIN_EXE_bcc"));
    // Every current caller of this helper runs the result through `fbc`,
    // so it needs `--target fbc` specifically (required for --binary/--run
    // on BASIC output, see GitHub issue #152 -- `--target basic`/`bascom`
    // is verified against real BASCOM instead, whose .EXE this process
    // can't run) -- explicit rather than relying on `bcc`'s own ambient
    // default-target resolution (BASCAL_TARGET / a dev's own
    // ~/.config/bascal/config), which a machine set to `C` by default
    // would otherwise silently break this against.
    command
        .arg(source_path)
        .arg("-o")
        .arg(&dir_arg)
        .arg("--target")
        .arg("fbc");
    for arg in extra_args {
        command.arg(arg);
    }

    let compile = command.output().expect("failed to run bcc");
    assert!(
        compile.status.success(),
        "bcc failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );
}

fn output_path_for_source(source: &Path, tutorial_dir: &Path, output_dir: &Path) -> PathBuf {
    let relative = source
        .strip_prefix(tutorial_dir)
        .unwrap_or_else(|_| source.file_name().map(Path::new).unwrap_or(source));
    output_dir.join(relative).with_extension("bas")
}

fn assert_branch_targets_are_numeric(output: &str, path: &Path) {
    for line in output.lines() {
        if line_payload_is_comment(line) {
            continue;
        }
        let trimmed = line.trim_start();
        if let Some(target) = branch_target_after_keyword(trimmed, "GOTO") {
            assert!(
                target
                    .chars()
                    .next()
                    .is_some_and(|first| first.is_ascii_digit()),
                "{} should use numeric GOTO targets, got `{line}`",
                path.display()
            );
        }
        if let Some(target) = branch_target_after_keyword(trimmed, "GOSUB") {
            assert!(
                target
                    .chars()
                    .next()
                    .is_some_and(|first| first.is_ascii_digit()),
                "{} should use numeric GOSUB targets, got `{line}`",
                path.display()
            );
        }
    }
}

fn branch_target_after_keyword<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    if line.starts_with(keyword) {
        return line.strip_prefix(keyword).map(str::trim_start);
    }

    if line.starts_with("IF ") {
        for marker in [format!(" THEN {keyword} "), format!(" THEN {keyword}\t")] {
            if let Some(index) = line.find(&marker) {
                return Some(line[index + marker.len()..].trim_start());
            }
        }
    }

    None
}

fn line_payload_is_comment(line: &str) -> bool {
    let payload = line
        .trim_start()
        .trim_start_matches(|ch: char| ch.is_ascii_digit())
        .trim_start();
    payload.starts_with('\'')
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}

/// Regression test for a real bug: glibc's stdout is line-buffered against
/// a real terminal (fully buffered otherwise), flushing only on a `\n`
/// byte -- so a `print "...";`/`print "...",` (no trailing newline --
/// `tutorial/inventory.bcl`'s `waitAnyKey()` own `"Press the AnyKey..."`)
/// or `INPUT`'s own `"...? "` prompt sat invisible in the buffer until
/// something else happened to flush it (neither `bcc_inkey`'s `read()` nor
/// `bcc_read_line`'s `fgets` flushes stdout first). Observed on a real
/// terminal as the prompt appearing only *after* a keystroke was read
/// blind, with whatever printed next arriving all at once right alongside
/// it. `Statement::Print`'s and `Statement::Input`'s C codegen now emit an
/// explicit `fflush(stdout);` right after any printf with no trailing
/// `\n`. Needs no `gcc`: this pins the exact generated C text, which is
/// sufficient (the bug was entirely about whether the `fflush` call gets
/// emitted at all).
#[test]
fn c_print_without_trailing_newline_flushes_stdout() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = tempfile::tempdir().expect("failed to create print-flush fixture directory");
    let source_path = dir.path().join("print_flush.bcl");
    fs::write(
        &source_path,
        "program printFlush\nprint \"prompt\";\ninput \"n\"; x%\nprint \"done\"\nend\n",
    )
    .expect("failed to write print-flush fixture");
    let output_dir = dir.path().join("out");
    fs::create_dir_all(&output_dir)
        .unwrap_or_else(|err| panic!("failed to create {}: {err}", output_dir.display()));
    let mut dir_arg = output_dir.as_os_str().to_owned();
    dir_arg.push("/");

    let status = Command::new(env!("CARGO_BIN_EXE_bcc"))
        .arg(&source_path)
        .arg("-o")
        .arg(&dir_arg)
        .arg("--target")
        .arg("C")
        .arg("--clean")
        .current_dir(repo_root)
        .status()
        .expect("failed to invoke bcc");
    assert!(
        status.success(),
        "bcc failed to compile {source_path:?} under --target C"
    );

    let generated = fs::read_to_string(output_dir.join("print_flush.c"))
        .expect("failed to read generated print_flush.c");
    let flush_count = generated.matches("fflush(stdout);").count();
    assert_eq!(
        flush_count, 2,
        "expected a flush after both the bare `print \"prompt\";` and the \
         INPUT prompt (but not after `print \"done\"`, which already ends \
         in a newline):\n{generated}"
    );
}
