// Conformance groups: core, records
//! Probes whether `record` currently works as a general-purpose,
//! file-independent BASCAL value type. This test makes NO changes to
//! `src/`; it only documents current behavior so future work can be
//! checked against it.
//!
//! Investigation finding (2026-08-26): today `record` is entirely
//! random-access-file DSL sugar. `records::lower` (src/records.rs) is a
//! pre-resolver pass that eliminates every `RecordDef`, `Statement::FileDecl`,
//! and DSL `Expr` variant (`FileIndex`, `FieldAccess`, `MethodCall`,
//! `RecordLit`) before `resolver::validate`/codegen ever run, and the only
//! way to populate a field-accessible `record_vars` entry is through a
//! `file <var> as Type = open(...)` declaration (or a `let p = file[i]` GET
//! binding). `dim` has no `as <Type>` clause, and neither `procedure`/
//! `function` parameters nor function return types support `as <Type>`
//! either (BASCAL uses BASCOM-style suffix typing: `%`, `$`, `#`, ... --
//! see tutorial/procedures.bcl, tutorial/functions.bcl). So there is
//! no grammar path to declare an ordinary in-memory record variable, pass
//! one as a parameter, return one, nest one inside another, build arrays
//! of them, or use a record literal outside file-write sugar.
//!
//! Each case below currently fails (parse or resolve error), asserted with
//! `.is_err()`. If/when general-purpose records are implemented, these
//! assertions should be flipped to `.expect(...)` success checks (and the
//! record-specific behaviors -- copy semantics, field/type checks on
//! literals, array-of-record element assignment, etc. -- should get their
//! own positive conformance fixtures under tests/fixtures/conformance/).

use bcc::{check_file, compile_file, diagnostics::Diagnostic, CompileOptions, Target};
use std::fs;
use std::path::PathBuf;

fn try_compile(name: &str, source: &str, target: Target) -> Result<String, Vec<Diagnostic>> {
    let dir = std::env::temp_dir().join("bascal_record_general_purpose_probe");
    fs::create_dir_all(&dir).expect("create scratch dir");
    let path: PathBuf = dir.join(format!("{name}.bcl"));
    fs::write(&path, source).expect("write probe source");
    let options = CompileOptions {
        target,
        ..CompileOptions::new()
    };
    compile_file(&path, &options)
}

/// The adventure port compiles through the front end without code generation.
/// This exercises nested records, typed arrays and parameters, record methods,
/// record literals, and dotted requires together.
#[test]
fn adventure_port_compiles_without_codegen() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/adventure/main.bcl");
    let options = CompileOptions {
        library_dirs: vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples")],
        ..CompileOptions::new()
    };
    check_file(&path, &options).expect("adventure port should parse with --check");
}

/// An ordinary record value can be initialized from a complete literal,
/// copied by assignment, read through members, and updated through members
/// without involving a random-access file.
#[test]
fn standalone_record_supports_init_assign_get_and_member_set() {
    let source = r#"program roomTest

record Room
    name: string(20)
    description: string(40)
    north: int16
    south: int16
    east: int16
    west: int16
end record

let room = { name: "Hall", description: "A large entrance hall.", north: 2, south: 0, east: 0, west: 0 }
let copy = room
copy.name = "Foyer"
copy.north = room.north + 1
print copy.name
print copy.north

end
"#;
    for target in [Target::Basic, Target::C] {
        let output = try_compile("standalone_record_literal", source, target).unwrap_or_else(
            |diagnostics| {
                panic!("record operations should compile for {target:?}: {diagnostics:?}")
            },
        );
        assert!(
            output.contains("copyname"),
            "expected generated member storage: {output}"
        );
    }
}

/// Case: `dim` has no `as <Type>` clause at all, so an ordinary declared
/// (not just literal-initialized) record variable can't be spelled.
#[test]
fn plain_dim_of_record_type_currently_fails() {
    let source = r#"program roomTest

record Room
    name: string(20)
    north: int16
end record

dim room as Room
room.name = "Hall"
print room.name

end
"#;
    let result = try_compile("plain_dim_record", source, Target::Basic);
    assert!(
        result.is_err(),
        "expected `dim room as Room` (no file DSL involved) to currently fail to compile; \
         it now succeeds -- general-purpose record variables may already be supported. Output: {:?}",
        result
    );
}

/// Case: arrays of records, e.g. `dim rooms(20) as Room`.
#[test]
fn array_of_records_currently_fails() {
    let source = r#"program roomTest

record Room
    name: string(20)
    north: int16
end record

dim rooms(20) as Room
rooms(1).name = "Hall"
rooms(1).north = 2

end
"#;
    let result = try_compile("array_of_records", source, Target::Basic);
    assert!(
        result.is_err(),
        "expected `dim rooms(20) as Room` (array of records) to currently fail to compile. Output: {:?}",
        result
    );
}

/// Case: a record-valued procedure parameter. BASCAL procedure params are
/// BASCOM-suffix-typed (`label$`, `score%`, ...), not `as <Type>`, so this
/// should fail regardless of the record/file-DSL question specifically.
#[test]
fn record_valued_parameter_currently_fails() {
    let source = r#"program roomTest

record Room
    name: string(20)
    description: string(40)
end record

procedure describeRoom(room as Room)
    print room.name
    print room.description
end procedure

end
"#;
    let result = try_compile("record_parameter", source, Target::Basic);
    assert!(
        result.is_err(),
        "expected a record-typed procedure parameter to currently fail to compile. Output: {:?}",
        result
    );
}

/// Case: a function returning a record value. BASCAL function return
/// types are suffix-encoded in the function name (`trimmed$`, `max%`),
/// not `as <Type>`, so this should fail regardless of the record/file-DSL
/// question specifically.
#[test]
fn record_valued_return_currently_fails() {
    let source = r#"program roomTest

record Room
    name: string(20)
end record

function makeRoom(n$) as Room
    let r as Room = { name: n$ }
    return r
end function

end
"#;
    let result = try_compile("record_return", source, Target::Basic);
    assert!(
        result.is_err(),
        "expected a record-valued function return type to currently fail to compile. Output: {:?}",
        result
    );
}

/// Case: a record field whose type is another record. Per the
/// investigation, `parse_record_field_type` (src/parser.rs) only accepts
/// int16/int32/float32/float64/string(N) as field types -- a record name
/// is not a legal field type at all, so this fails at record-declaration
/// parse time, before any file/DSL concern applies.
#[test]
fn nested_record_field_currently_fails() {
    let source = r#"program actorTest

record Position
    x: int16
    y: int16
end record

record Actor
    name: string(20)
    position: Position
end record

end
"#;
    let result = try_compile("nested_record_field", source, Target::Basic);
    assert!(
        result.is_err(),
        "expected a record field typed as another record to currently fail to compile. Output: {:?}",
        result
    );
}

/// Bare `string` members are variable-length and are valid for records used
/// in memory. They have no fixed packed width, so they are rejected when the
/// record is declared as a random-access file type (tested below).
#[test]
fn bare_dynamic_string_record_members_are_supported() {
    let source = r#"program thingTest

record Thing
    name: string
    description: string
end record

end
"#;
    let result = try_compile("bare_dynamic_string_field", source, Target::Basic);
    assert!(
        result.is_ok(),
        "expected bare string record members to compile for in-memory records. Output: {:?}",
        result
    );
}

#[test]
fn bare_dynamic_string_record_is_rejected_as_file_type() {
    let source = r#"program thingFile

record Thing
    name: string
end record

file db as Thing = open("thing.dat")
end
"#;
    let result = try_compile("bare_dynamic_string_file", source, Target::Basic);
    let diagnostics = result.expect_err("variable-width record file should be rejected");
    let text = format!("{diagnostics:?}");
    assert!(
        text.contains("variable-length string"),
        "unexpected diagnostics: {text}"
    );
    assert!(
        text.contains("random-access file type"),
        "unexpected diagnostics: {text}"
    );
}

/// Sanity check / control case: confirms the EXISTING random-access-file
/// record usage (the one thing records ARE for today) still compiles, so
/// the failures above are attributable to the general-purpose-record gap
/// specifically and not to a broken test harness or an unrelated
/// regression. Mirrors tutorial/random_and_record_files.bcl.
///
/// Basic and C only: the JVM backend is independently "minimal" and
/// already rejects the `Open`/random-file statement outright
/// (src/codegen_jvm.rs:660, "... is not supported by the minimal JVM
/// backend yet") -- a pre-existing gap unrelated to whether records are
/// general-purpose.
#[test]
fn existing_random_access_file_record_usage_still_compiles_on_all_targets() {
    let source = r#"program studentFileTest

record Student
    id:    int16
    name:  string(20)
    score: float64
end record

file db as Student = open("students.dat")

db[1] = { id: 1, name: "Alice", score: 95.0 }
let s = db[1]
print s.name

db.close()

end
"#;
    for target in [Target::Basic, Target::C] {
        let result = try_compile("existing_file_record_usage", source, target);
        assert!(
            result.is_ok(),
            "expected existing random-access-file record usage to keep compiling under {target:?}, got: {:?}",
            result.err()
        );
    }
}

/// Documents (does not assert a regression on) the JVM backend's existing,
/// unrelated limitation: it rejects random-access file records outright,
/// independent of the general-purpose-record question.
#[test]
fn jvm_backend_does_not_yet_support_random_access_file_records() {
    let source = r#"program studentFileTest

record Student
    id:    int16
    name:  string(20)
    score: float64
end record

file db as Student = open("students.dat")
db.close()

end
"#;
    let result = try_compile("jvm_file_record_usage", source, Target::Jvm);
    assert!(
        result.is_err(),
        "JVM backend now compiles random-access file records -- this pre-existing \
         limitation may have been lifted; update this test's expectation. Output: {:?}",
        result
    );
}
