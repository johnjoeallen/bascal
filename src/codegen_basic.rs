use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::ast::*;
use crate::diagnostics::{Diagnostic, SourcePos};

// "ucase", "lcase", "ltrim", and "rtrim" are deliberately absent here: real
// MBASIC/BASCOM 2.00 has none of them (verified against a real IBM BASIC
// Compiler 2.00 under dosbox-x -- see com/bascal/stdlib/), so treating them
// as safe passthrough builtins the target dialect provides would be wrong.
// BASCAL provides its own implementations instead, as an ordinary
// require-able library; see `lib::stdlib_search_roots`.
pub(crate) const BASIC_BUILTINS: &[&str] = &[
    // Type-suffixed single-arg — parser creates Expr::ArrayRef for these
    "str", "chr", "hex", "oct", "space", "environ", "command",
    // Multi-arg string (Expr::Call, but include for completeness)
    "left", "right", "mid", "instr", "format", "string", "input",
    // Single-arg numeric (no suffix → Expr::Call already, but included for safety)
    "len", "val", "asc", "sqr", "abs", "int", "fix", "sgn", "rnd", "eof", "sin", "cos", "tan",
    "atn", "log", "exp", "cint", "clng", "csng", "cdbl", "peek", "inp", "lof", "loc", "pos",
    "csrlin", "freefile", "fre", "lpos", "varptr", "date", "time", "timer", "inkey", "err", "erl",
    // Machine, screen and device functions of MBASIC/BASCOM 2.00
    "usr", "point", "screen", "pen", "stick", "strig", "play", "ioctl", "erdev",
    // Print-position helpers (used inside PRINT)
    "tab", "spc", // Multi-arg numeric
    "ubound", "lbound", "iif", // Random-access record packing/unpacking
    "mki", "mkl", "mks", "mkd", "cvi", "cvl", "cvs", "cvd",
];

#[cfg(test)]
mod tests {
    #[test]
    fn semantic_callable_dim_types_are_retained() {
        let module = crate::semantic_ir::parse_and_adapt(
            "function f%()\ndim text as string\nend function\n",
        )
        .expect("semantic callable declaration parses");
        let types = module.callables[0].dim_types();
        assert_eq!(types.get("text").map(String::as_str), Some("string"));
    }

    #[test]
    fn basic_generation_uses_semantic_dependency_declarations() {
        let ast_source = "require com.example.ast\nimport com.example.astImport\nprint 1\nend\n";
        let semantic_source =
            "require com.example.semantic\nimport com.example.semanticImport\nprint 9\nend\n";
        let parsed =
            crate::parse_source("semantic_dependency.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic =
            crate::semantic_ir::parse_and_adapt_named("semantic_dependency.bcl", semantic_source)
                .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("' require com.example.semantic"),
            "{output}"
        );
        assert!(
            output.contains("' import com.example.semanticImport (alias for require)"),
            "{output}"
        );
        assert!(!output.contains("' require com.example.ast"), "{output}");
        assert!(
            !output.contains("' import com.example.astImport"),
            "{output}"
        );
        assert!(output.contains("PRINT 9"), "{output}");
    }

    #[test]
    fn semantic_callable_return_controls_implicit_return_emission() {
        let ast_source = "function amount%()\nreturn 1\n' AST trailing comment\nend function\nprint amount%()\nend\n";
        let semantic_source = "function amount%()\nreturn 2\nend function\nprint amount%()\nend\n";
        let parsed =
            crate::parse_source("semantic_callable_return.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_callable_return.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("amountResult0% = 2"), "{output}");
        assert_eq!(output.matches("    RETURN").count(), 1, "{output}");
    }

    #[test]
    fn unmatched_semantic_callable_keeps_ast_return_compatibility() {
        let ast_source = "function amount%()\nreturn 1\nend function\nprint amount%()\nend\n";
        let semantic_source = "print 2\nend\n";
        let parsed =
            crate::parse_source("semantic_return_fallback.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_return_fallback.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("amountResult0% = 1"), "{output}");
        assert_eq!(output.matches("    RETURN").count(), 1, "{output}");
    }

    #[test]
    fn semantic_missing_return_rejects_ast_only_return() {
        let ast_source =
            "function amount%()\nvalue% = 1\nreturn 1\nend function\nprint amount%()\nend\n";
        let semantic_source =
            "function amount%()\nvalue% = 2\nend function\nprint amount%()\nend\n";
        let parsed =
            crate::parse_source("semantic_partial_return.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_partial_return.bcl",
            semantic_source,
        )
        .unwrap();
        let diagnostics = match crate::resolver::resolve_with_semantic(program, Some(semantic)) {
            Err(diagnostics) => diagnostics,
            Ok(_) => panic!("AST-only RETURN incorrectly satisfied typed callable validation"),
        };
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("implicit function return")),
            "typed callable return validation did not report the missing RETURN: {diagnostics:?}"
        );
    }

    #[test]
    fn basic_generation_uses_semantic_suffixless_string_dim() {
        let source = "dim text as string\nend\n";
        let parsed = crate::parse_source("semantic_dim.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt(source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM text AS STRING"), "{output}");
    }

    #[test]
    fn basic_generation_uses_semantic_long_dim_over_legacy_suffix() {
        let legacy_source = "dim value%\nvalue% = 1\nend\n";
        let semantic_source = "dim value as long\nvalue = 2\nend\n";
        let parsed =
            crate::parse_source("semantic_long_dim_authority.bcl".to_string(), legacy_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(
                crate::semantic_ir::parse_and_adapt_named(
                    "semantic_long_dim_authority.bcl",
                    semantic_source,
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(output.contains("DIM value AS LONG"), "{output}");
        assert!(output.contains("value = 2"), "{output}");
        assert!(!output.contains("DIM value%"), "{output}");
    }

    #[test]
    fn basic_generation_dispatches_typed_record_field_reads_and_writes() {
        let ast_source = "program roomTest\nrecord Room\nname: string(20)\nend record\nlet room = { name: \"Hall\" }\nroom.name = \"AST\"\nfunction read$()\nreturn \"AST return\"\nend function\nfunction localRead$()\nlet local = { name: \"Local\" }\nlocal.name = \"AST local\"\nreturn \"AST local\"\nend function\nprint \"AST\"\nprint read$()\nprint localRead$()\nend\n";
        let semantic_source = "program roomTest\nrecord Room\nname: string(20)\nend record\nlet room = { name: \"Hall\" }\nmid$(room.name, 2, 2) = \"IR\"\nfunction read$()\nreturn room.name\nend function\nfunction localRead$()\nlet local = { name: \"Local\" }\nmid$(local.name, 2, 2) = \"XY\"\nreturn local.name\nend function\nprint room.name\nprint read$()\nprint localRead$()\nend\n";
        let parsed =
            crate::parse_source("semantic_record_member.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_record_member.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("LEFT$(BCCT"),
            "typed MID$ field target did not emit its splice: {output}"
        );
        assert!(
            output.contains("roomname$ = LEFT$(BCCT"),
            "typed MID$ field target did not write back to typed field storage: {output}"
        );
        assert!(
            output.contains("PRINT roomname$"),
            "typed field read was not emitted: {output}"
        );
        assert!(
            !output.contains("PRINT \"AST\""),
            "AST print replaced semantic field read: {output}"
        );
        assert!(
            output.contains(" = roomname$"),
            "typed callable field read was not emitted: {output}"
        );
        assert!(
            !output.contains("AST return"),
            "AST callable return replaced semantic IR: {output}"
        );
        assert!(
            output.contains(" = localreadLocalName0$"),
            "typed local record-field read was not emitted: {output}"
        );
        assert!(
            output.contains("localreadLocalName0$ = LEFT$(BCCT"),
            "callable-local MID$ write did not use allocated record-field storage: {output}"
        );
        assert!(
            !output.contains("AST local"),
            "AST local return replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_generation_emits_typed_record_file_declaration_without_record_ops() {
        let ast_source = "record Student\nid: int16\nname: string(6)\nend record\nfile db as Student = open(\"ast.dat\")\nend\n";
        let semantic_source = "record Student\nid: int16\nname: string(6)\nend record\nfile db as Student = open(\"typed.dat\")\nend\n";
        let parsed = crate::parse_source("typed_record_file.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered {
            program,
            lowered_record_files,
            ..
        } = crate::lower::lower(parsed).unwrap();
        assert_eq!(lowered_record_files[0].record_length, 8);
        assert_eq!(
            lowered_record_files[0]
                .fields
                .iter()
                .map(|field| (field.width, field.offset))
                .collect::<Vec<_>>(),
            [(2, 0), (6, 2)]
        );
        let mut semantic = crate::semantic_ir::parse_and_adapt_named(
            "typed_record_file.bcl",
            semantic_source,
        )
        .unwrap();
        semantic.lowered_record_files = lowered_record_files;
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(output.contains("OPEN \"typed.dat\" FOR RANDOM AS #1 LEN = 8"), "{output}");
        assert!(
            output.contains("FIELD #1, 2 AS dbidbuf$, 6 AS dbnamebuf$"),
            "{output}"
        );
        assert!(!output.contains("ast.dat"), "AST file path leaked: {output}");
    }

    #[test]
    fn basic_semantic_record_buffers_replace_compatibility_ast_names() {
        let ast_source = "program p\nfield #1, 4 as value$\nfunction read$()\nlet value$ = \"AST\"\nreturn value$\nend function\nprint read$()\nend\n";
        let semantic_source = "program p\nbeep\nfunction read$()\nlet value$ = \"typed\"\nreturn value$\nend function\nprint read$()\nend\n";
        let parsed =
            crate::parse_source("semantic_record_buffer_scope.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_record_buffer_scope.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("BEEP"),
            "semantic root statement missing: {output}"
        );
        assert!(
            output.contains("\"typed\""),
            "semantic local initializer missing: {output}"
        );
        assert!(
            !output.contains("\"AST\""),
            "AST local initializer leaked: {output}"
        );
        assert!(
            !output.contains("PRINT value$") && !output.contains("= value$"),
            "AST FIELD binding leaked into semantic local scope: {output}"
        );
    }

    #[test]
    fn basic_semantic_field_binding_uses_typed_suffix_metadata() {
        let ast_source = "open \"record.dat\" for random as #1 len = 4\nfield #1, 4 as buffer%\nend\n";
        let semantic_source = "open \"record.dat\" for random as #1 len = 4\nfield #1, 4 as buffer$\nend\n";
        let parsed =
            crate::parse_source("typed_field_binding.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut semantic = crate::semantic_ir::parse_and_adapt_named(
            "typed_field_binding.bcl",
            semantic_source,
        )
        .unwrap();
        let crate::semantic_ir::SemanticStatementKind::Line(statements) =
            &mut semantic.statements[1].kind
        else {
            panic!()
        };
        let crate::semantic_ir::SemanticStatementKind::Field { bindings, .. } =
            &mut statements[0].kind
        else {
            panic!()
        };
        bindings[0].name = "buffer%".to_string();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(output.contains("FIELD #1, 4 AS buffer$"), "{output}");
        assert!(!output.contains("FIELD #1, 4 AS buffer%"), "{output}");
    }

    #[test]
    fn basic_generation_resolves_composed_semantic_record_fields() {
        let declarations = "record Core\nid: int16\nend record\nrecord Identity combines Core\nkey: int32\nend record\nrecord Entry combines Identity\nname: string(8)\nend record\ndim row as Entry\n";
        let ast_source = format!("{declarations}print 999\nend\n");
        let semantic_source = format!("{declarations}print row.id\nend\n");
        let parsed = crate::parse_source("semantic_composed_record.bcl".to_string(), &ast_source)
            .expect("compatibility AST parses");
        let crate::lower::Lowered { program, .. } =
            crate::lower::lower(parsed).expect("compatibility AST lowers");
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_composed_record.bcl",
            &semantic_source,
        )
        .expect("typed IR parses");
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic))
            .expect("program resolves");
        let module = resolved
            .semantic_module
            .as_ref()
            .expect("typed module remains attached");
        assert!(
            super::semantic_record_storage_names(module).contains("rowid%"),
            "inherited member storage was not reserved"
        );
        let output = super::CodeGenerator::new()
            .generate(&resolved)
            .expect("BASIC transpires");
        assert!(output.contains("PRINT rowid%"), "{output}");
        assert!(!output.contains("PRINT 999"), "AST print leaked: {output}");
    }

    #[test]
    fn basic_generation_emits_simple_terminal_statements_from_semantic_ir() {
        let source = "beep\n";
        let semantic_source = "stop\n";
        let parsed = crate::parse_source("semantic_terminal.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("\nSTOP\n"), "{output}");
        assert!(!output.contains("BEEP"), "{output}");
    }

    #[test]
    fn basic_generation_emits_terminal_intrinsic_family_from_semantic_ir() {
        for (semantic_statement, expected) in [
            ("stop", "STOP"),
            ("cls", "CLS"),
            ("beep", "BEEP"),
            ("system", "SYSTEM"),
            ("clear", "CLEAR"),
        ] {
            let source = "stop\nend\n";
            let semantic_source = format!("{semantic_statement}\nend\n");
            let parsed = crate::parse_source("semantic_intrinsic.bcl".to_string(), source).unwrap();
            let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
            let mut resolved = crate::resolver::resolve(program).unwrap();
            resolved.semantic_module =
                Some(crate::semantic_ir::parse_and_adapt(&semantic_source).unwrap());
            let output = super::CodeGenerator::new().generate(&resolved).unwrap();
            assert!(output.contains(&format!("\n{expected}\n")), "{output}");
            assert!(output.contains("\nEND\n"), "{output}");
        }
    }

    #[test]
    fn basic_generation_emits_semantic_label_transfers_from_ir() {
        let source = "beep\n";
        let semantic_source = "top:\ngosub sub\ngoto ender\nsub:\nrestore top\nresume next\non error goto 0\nender:\nstop\nend\n";
        let parsed = crate::parse_source("semantic_labels.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        for expected in ["GOSUB", "GOTO", "RESTORE", "RESUME NEXT", "ON ERROR GOTO 0"] {
            assert!(output.contains(expected), "missing {expected}: {output}");
        }
        assert!(output.contains("GOSUB 20"), "{output}");
        assert!(output.contains("GOTO 30"), "{output}");
        assert!(output.contains("RESTORE 10"), "{output}");
        assert!(output.contains("\nEND\n"), "{output}");
        assert!(!output.contains("BEEP"), "{output}");
    }

    #[test]
    fn basic_generation_dispatches_single_line_if_from_semantic_ir() {
        let parsed = crate::parse_source("semantic_single_line_if.bcl".to_string(), "beep\n")
            .expect("compatibility AST parses");
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_single_line_if.bcl",
            "if 1 then stop\n",
        )
        .expect("typed IR parses");
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("IF (1) = 0 THEN GOTO"), "{output}");
        assert!(output.contains("STOP"), "{output}");
        assert!(
            !output.contains("BEEP"),
            "legacy statement leaked: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_single_line_if_else_from_semantic_ir() {
        let parsed = crate::parse_source("semantic_single_line_if_else.bcl".to_string(), "cls\n")
            .expect("compatibility AST parses");
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_single_line_if_else.bcl",
            "if 1 then stop else beep\n",
        )
        .expect("typed IR parses");
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("IF (1) = 0 THEN GOTO"), "{output}");
        assert!(output.contains("STOP"), "{output}");
        assert!(output.contains("BEEP"), "{output}");
        assert!(!output.contains("CLS"), "legacy statement leaked: {output}");
    }

    #[test]
    fn basic_callable_dispatches_single_line_if_from_semantic_ir() {
        let ast_source = "function choose%()\nreturn 1\nend function\nend\n";
        let semantic_source =
            "function choose%()\nif 1 then return 2 else return 3\nend function\nend\n";
        let parsed = crate::parse_source("semantic_callable_single_if.bcl".to_string(), ast_source)
            .expect("compatibility AST parses");
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_callable_single_if.bcl",
            semantic_source,
        )
        .expect("typed IR parses");
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("IF (1) = 0 THEN GOTO"), "{output}");
        assert!(output.contains("chooseResult0% = 2"), "{output}");
        assert!(output.contains("chooseResult0% = 3"), "{output}");
        assert!(
            !output.contains("chooseResult0% = 1"),
            "AST return leaked: {output}"
        );
    }

    #[test]
    fn basic_generation_consumes_semantic_global_declarations_without_output() {
        let source = "beep\n";
        let semantic_source = "global value%\nstop\nend\n";
        let parsed = crate::parse_source("semantic_global_noop.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("\nSTOP\n"), "{output}");
        assert!(output.contains("\nEND\n"), "{output}");
        assert!(!output.contains("BEEP"), "{output}");
    }

    #[test]
    fn basic_generation_emits_semantic_data_and_read_statements() {
        let source = "beep\n";
        let semantic_source = "data 7, \"ok\"\nread value%\nend\n";
        let parsed = crate::parse_source("semantic_data_read.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DATA 7, \"ok\""), "{output}");
        assert!(output.contains("READ value%"), "{output}");
        assert!(output.contains("END"), "{output}");
        assert!(!output.contains("BEEP"), "{output}");
    }

    #[test]
    fn basic_generation_dispatches_semantic_top_level_expression_statements() {
        let ast_source =
            "function tick%()\nreturn 7\nend function\nprint \"ast expression\"\nend\n";
        let semantic_source = "function tick%()\nreturn 7\nend function\ntick%()\nend\n";
        let parsed =
            crate::parse_source("semantic_top_level_expression.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_top_level_expression.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("GOSUB 10"),
            "semantic expression call was not transpiled: {output}"
        );
        assert!(
            !output.contains("BCCT1% = tickResult0%"),
            "discarded expression result allocated an unnecessary BASIC temporary: {output}"
        );
        assert!(
            !output.contains("ast expression"),
            "AST top-level expression replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_callable_print_operands_from_semantic_ir() {
        let ast_source =
            "function tick%()\nreturn 7\nend function\nfunction makeFormat$()\nreturn \"###\"\nend function\nfunction display%()\nlprint using \"###\"; \"ast expression\"\nwrite #1, 0, 0\ninput #1, inputValue%\nline input #1, inputText$\nopen \"ast.dat\" for input as #1\nclose #1\nkill \"ast.tmp\"\nname \"a\" as \"b\"\nprint #1, using \"###\"; 0\nreturn 0\nend function\nlprint using \"###\"; \"ast expression\"\nprint \"ast expression\"\nprint 0, 0\nwrite #1, 0, 0\ninput #1, inputValue%\nline input #1, inputText$\nopen \"ast.dat\" for input as #1\nclose #1\nkill \"ast.tmp\"\nname \"a\" as \"b\"\nprint #1, using \"###\"; 0\nend\n";
        let semantic_source =
            "function tick%()\nreturn 7\nend function\nfunction makeFormat$()\nreturn \"###\"\nend function\nfunction display%()\nlprint using makeFormat$(); tick%(), tick%()\nwrite #tick%(), tick%(), tick%()\ninput #tick%(), inputValue%\nline input #tick%(), inputText$\nopen \"semantic.dat\" for input as #tick%()\nclose #tick%()\nkill makeFormat$()\nname makeFormat$() as makeFormat$()\nprint #tick%(), using makeFormat$(); tick%()\nreturn 0\nend function\nlprint using makeFormat$(); tick%(), tick%()\nprint using makeFormat$(); tick%()\nprint tick%(), tick%()\nwrite #tick%(), tick%(), tick%()\ninput #tick%(), inputValue%\nline input #tick%(), inputText$\nopen \"semantic.dat\" for input as #tick%()\nclose #tick%()\nkill makeFormat$()\nname makeFormat$() as makeFormat$()\nprint #tick%(), using makeFormat$(); tick%()\nend\n";
        let parsed = crate::parse_source(
            "semantic_print_callable.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_print_callable.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("GOSUB 10"), "{output}");
        assert!(output.contains("PRINT USING BCCT"), "{output}");
        assert!(output.contains("PRINT #BCCT"), "{output}");
        assert!(output.contains("PRINT BCCT"), "{output}");
        assert!(output.contains("LPRINT USING BCCT"), "{output}");
        assert!(output.contains("WRITE #BCCT"), "{output}");
        assert!(output.contains("INPUT #BCCT"), "{output}");
        assert!(output.contains("LINE INPUT #BCCT"), "{output}");
        assert!(output.contains("OPEN \"semantic.dat\" FOR INPUT AS #BCCT"), "{output}");
        assert!(output.contains("CLOSE #BCCT"), "{output}");
        assert!(output.contains("KILL BCCT"), "{output}");
        assert!(output.contains("NAME BCCT"), "{output}");
        assert!(!output.contains("ast expression"), "{output}");
    }

    #[test]
    fn basic_generation_dispatches_typed_locate_operands_at_module_scope() {
        let ast_source = "function tick%()\nreturn 1\nend function\nlocate 1, 2\nend\n";
        let semantic_source =
            "function tick%()\nreturn 9\nend function\nlocate tick%(), tick%()\nend\n";
        let parsed =
            crate::parse_source("semantic_module_locate_call.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_module_locate_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("LOCATE BCCT1%, BCCT2%"),
            "typed module LOCATE operands must use their evaluated call results: {output}"
        );
        assert!(
            !output.contains("LOCATE 1, 2"),
            "AST module LOCATE operands must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_typed_color_operands_at_module_scope() {
        let ast_source = "function tick%()\nreturn 1\nend function\ncolor 1, 2\nend\n";
        let semantic_source =
            "function tick%()\nreturn 9\nend function\ncolor tick%(), tick%()\nend\n";
        let parsed =
            crate::parse_source("semantic_module_color_call.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_module_color_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("COLOR BCCT1%, BCCT2%"),
            "typed module COLOR operands must use their evaluated call results: {output}"
        );
        assert!(
            !output.contains("COLOR 1, 2"),
            "AST module COLOR operands must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_typed_width_operands_at_module_scope() {
        let ast_source = "function tick%()\nreturn 1\nend function\nwidth #1, 2\nend\n";
        let semantic_source =
            "function tick%()\nreturn 9\nend function\nwidth #tick%(), tick%()\nend\n";
        let parsed =
            crate::parse_source("semantic_module_width_call.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_module_width_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("WIDTH #BCCT1%, BCCT2%"),
            "typed module WIDTH operands must use evaluated call results: {output}"
        );
        assert!(
            !output.contains("WIDTH #1, 2"),
            "AST module WIDTH operands must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_typed_poke_and_out_operands_at_module_scope() {
        let ast_source = "function tick%()\nreturn 1\nend function\npoke 1, 2\nout 3, 4\nend\n";
        let semantic_source = "function tick%()\nreturn 9\nend function\npoke tick%(), tick%()\nout tick%(), tick%()\nend\n";
        let parsed =
            crate::parse_source("semantic_module_poke_out_calls.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_module_poke_out_calls.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("POKE BCCT1%, BCCT2%") && output.contains("OUT BCCT3%, BCCT4%"),
            "typed module POKE/OUT operands must use evaluated call results: {output}"
        );
        assert!(
            !output.contains("POKE 1, 2") && !output.contains("OUT 3, 4"),
            "AST module POKE/OUT operands must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_typed_read_indices_with_callable_operands() {
        let ast_source = "dim values%(4)\nfunction index%()\nreturn 1\nend function\nfunction load%()\nread values%(1)\nreturn 0\nend function\ndata 0\nread values%(1)\nprint load%()\nend\n";
        let semantic_source = "dim values%(4)\nfunction index%()\nreturn 2\nend function\nfunction load%()\nread values%(index%())\nreturn 0\nend function\ndata 7\nread values%(index%())\nprint load%()\nend\n";
        let parsed =
            crate::parse_source("semantic_read_index.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_read_index.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("READ values%(BCCT2%)")
                && output.contains("READ loadValues0%(BCCT4%)"),
            "typed module and callable READ target indices must use callable results: {output}"
        );
        assert!(
            !output.contains("READ values%(1)"),
            "AST READ target must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_typed_swap_lvalues_with_callable_indices() {
        let ast_source = "dim values%(4)\nfunction index%()\nreturn 1\nend function\nfunction exchange%()\nswap values%(1), values%(2)\nreturn 0\nend function\nswap values%(1), values%(2)\nprint exchange%()\nend\n";
        let semantic_source = "dim values%(4)\nfunction index%()\nreturn 2\nend function\nfunction exchange%()\nswap values%(index%()), values%(index%())\nreturn 0\nend function\nswap values%(index%()), values%(index%())\nprint exchange%()\nend\n";
        let parsed =
            crate::parse_source("semantic_swap_indices.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_swap_indices.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("SWAP values%(BCCT2%), values%(BCCT4%)"),
            "typed module SWAP must use ordered callable-index snapshots: {output}"
        );
        assert!(
            output.contains("SWAP exchangeValues0%(BCCT")
                && output.contains("), exchangeValues0%(BCCT"),
            "typed callable SWAP must use callable-index snapshots: {output}"
        );
        assert!(
            !output.contains("SWAP values%(1), values%(2)"),
            "AST SWAP operands must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_typed_input_lvalues_with_callable_indices() {
        let ast_source = "dim values%(4)\ndim texts$(4)\nopen \"input.txt\" for input as #1\nfunction index%()\nreturn 1\nend function\nfunction load%()\ninput values%(1)\nline input #1, texts$(1)\nreturn 0\nend function\ninput values%(1)\nline input #1, texts$(1)\nprint load%()\nend\n";
        let semantic_source = "dim values%(4)\ndim texts$(4)\nopen \"input.txt\" for input as #1\nfunction index%()\nreturn 2\nend function\nfunction load%()\ninput values%(index%())\nline input #1, texts$(index%())\nreturn 0\nend function\ninput values%(index%())\nline input #1, texts$(index%())\nprint load%()\nend\n";
        let parsed =
            crate::parse_source("semantic_input_indices.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_input_indices.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("INPUT values%(BCCT2%)")
                && output.contains("INPUT loadValues0%(BCCT6%)"),
            "typed module and callable INPUT targets must use callable-index snapshots: {output}"
        );
        assert!(
            !output.contains("INPUT values%(1)"),
            "AST INPUT targets must not replace typed IR: {output}"
        );
        assert!(
            output.contains("LINE INPUT #1, texts$(BCCT4%)")
                && output.contains("LINE INPUT #1, loadTexts0$(BCCT8%)"),
            "typed module and callable LINE INPUT targets must use callable-index snapshots: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_typed_assignment_lvalues_with_callable_indices() {
        let ast_source = "dim values%(4)\nfunction index%()\nreturn 1\nend function\nfunction change%()\nvalues%(1) = 3\nreturn 0\nend function\nvalues%(1) = 3\nprint change%()\nend\n";
        let semantic_source = "dim values%(4)\nfunction index%()\nreturn 2\nend function\nfunction change%()\nvalues%(index%()) = index%()\nreturn 0\nend function\nvalues%(index%()) = index%()\nprint change%()\nend\n";
        let parsed = crate::parse_source(
            "semantic_assignment_indices.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_assignment_indices.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("values%(BCCT2%) = indexResult0%")
                && output.contains("changeValues0%(BCCT4%) = indexResult0%"),
            "typed assignment targets must snapshot callable indices before evaluating callable values: {output}"
        );
        assert!(
            !output.contains("values%(1) = 3"),
            "AST assignment target and value must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_typed_rank_two_assignment_indices_with_callables() {
        let ast_source = "function index%()\nreturn 1\nend function\nfunction update%()\ndim grid%(2, 2)\ngrid%(1, 1) = 2\nreturn 0\nend function\ndim grid%(2, 2)\ngrid%(1, 1) = 2\nprint update%()\nend\n";
        let semantic_source = "function index%()\nreturn 2\nend function\nfunction update%()\ndim grid%(2, 2)\ngrid%(index%(), index%()) = 7\nreturn 0\nend function\ndim grid%(2, 2)\ngrid%(index%(), index%()) = 7\nprint update%()\nend\n";
        let parsed = crate::parse_source(
            "semantic_rank_two_assignment_indices.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_rank_two_assignment_indices.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.matches(" = 7").count() == 2,
            "typed rank-two module and callable assignments must replace AST values: {output}"
        );
        assert!(
            output.contains("grid%(BCCT") && output.contains(", BCCT"),
            "typed rank-two indices must be snapshotted before assignment: {output}"
        );
        assert!(
            !output.contains("grid%(1, 1) = 2"),
            "AST rank-two lvalues must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_typed_dim_bounds_with_callable_expressions() {
        let ast_source = "function size%()\nreturn 1\nend function\nfunction allocate%()\ndim local%(1)\nreturn 0\nend function\ndim values%(1)\nprint allocate%()\nend\n";
        let semantic_source = "function size%()\nreturn 2\nend function\nfunction allocate%()\ndim local%(size%())\nreturn 0\nend function\ndim values%(size%())\nprint allocate%()\nend\n";
        let parsed =
            crate::parse_source("semantic_dim_callable_bounds.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_dim_callable_bounds.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("DIM values%(BCCT")
                && output.contains("DIM allocateLocal0%(BCCT"),
            "typed module and callable DIM bounds must use callable results: {output}"
        );
        assert!(
            !output.contains("DIM values%(1)") && !output.contains("DIM local%(1)"),
            "AST DIM bounds must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_typed_scalar_method_expression_statements() {
        let ast_source = "method adjust%[integer]()\nreturn self% + 1\nend method\nbase% = 10\nprint \"AST method\"\nend\n";
        let semantic_source = "method adjust%[integer]()\nreturn self% + 7\nend method\nbase% = 20\nbase%.adjust()\nend\n";
        let parsed = crate::parse_source(
            "semantic_method_expression_statement.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_method_expression_statement.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("base% = 20"),
            "semantic receiver assignment missing: {output}"
        );
        assert!(
            output.contains("GOSUB 10"),
            "typed scalar method call was not transpiled: {output}"
        );
        assert!(
            !output.contains("AST method"),
            "AST method statement replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_typed_scalar_method_arguments() {
        let ast_source = "method adjust%[integer](delta$)\nreturn self% + delta$\nend method\nbase% = 10\nprint \"AST method\"\nend\n";
        let semantic_source = "method adjust%[integer](delta%)\nreturn self% + delta%\nend method\nbase% = 20\nbase%.adjust(7)\nend\n";
        let parsed = crate::parse_source(
            "semantic_method_expression_arguments.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_method_expression_arguments.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("base% = 20"),
            "semantic receiver assignment missing: {output}"
        );
        assert!(
            output.contains("= 7") && output.contains("GOSUB 10"),
            "typed scalar method argument or call missing: {output}"
        );
        assert!(
            output.contains("BCCT2% = 7") && !output.contains("BCCT2$ = 7"),
            "method argument temporary did not use the typed parameter suffix: {output}"
        );
        assert!(
            !output.contains("AST method"),
            "AST method statement replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_scalar_method_with_typed_call_receiver() {
        let ast_source = "function identity%(value%)\nreturn value%\nend function\nmethod adjust%[integer](delta%)\nreturn self% + delta%\nend method\nprint \"AST method\"\nend\n";
        let semantic_source = "function identity%(value%)\nreturn value%\nend function\nmethod adjust%[integer](delta%)\nreturn self% + delta%\nend method\nidentity%(20).adjust(7)\nend\n";
        let parsed =
            crate::parse_source("semantic_method_call_receiver.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_method_call_receiver.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("identityResult0%") && output.contains("GOSUB 20"),
            "typed method receiver call prelude or method call missing: {output}"
        );
        assert!(
            !output.contains("AST method"),
            "AST method statement replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_generation_copies_back_scalar_method_byref_arguments() {
        let ast_source = "method update%[integer](byref target%)\ntarget% = target% + self%\nreturn target%\nend method\ntarget% = 3\nbase% = 10\nprint \"AST method\"\nend\n";
        let semantic_source = "method update%[integer](byref target%)\ntarget% = target% + self%\nreturn target%\nend method\ntarget% = 5\nbase% = 20\nbase%.update(target%)\nend\n";
        let parsed =
            crate::parse_source("semantic_method_byref.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic =
            crate::semantic_ir::parse_and_adapt_named("semantic_method_byref.bcl", semantic_source)
                .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.matches("target% = ").count() >= 2,
            "typed method ByRef value was not copied back to its caller: {output}"
        );
        assert!(
            !output.contains("AST method"),
            "AST method statement replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_generation_preserves_typed_method_receiver_and_argument_order() {
        let ast_source = "function firstValue%()\nreturn 1\nend function\nfunction secondValue%()\nreturn 2\nend function\nmethod combine%[integer](first%, second%)\nreturn self% + first% + second%\nend method\nprint \"AST method\"\nend\n";
        let semantic_source = "function firstValue%()\nreturn 1\nend function\nfunction secondValue%()\nreturn 2\nend function\nmethod combine%[integer](first%, second%)\nreturn self% + first% + second%\nend method\nfirstValue%().combine(secondValue%(), firstValue%())\nend\n";
        let parsed =
            crate::parse_source("semantic_method_call_order.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_method_call_order.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        let main = output.split("\nEND\n").next().unwrap_or(&output);
        let calls = main
            .lines()
            .filter(|line| line.contains("GOSUB "))
            .map(str::trim)
            .collect::<Vec<_>>();
        assert_eq!(calls, ["GOSUB 10", "GOSUB 20", "GOSUB 10", "GOSUB 30"]);
        assert!(
            !output.contains("AST method"),
            "AST method statement replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_generation_emits_semantic_assignments_and_print_tokens() {
        let source = "beep\n";
        let semantic_source = "value% = 4\nvalue% += 3\nprint value%;, \"ok\"\nend\n";
        let parsed =
            crate::parse_source("semantic_assignment_print.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("value% = 4"), "{output}");
        assert!(output.contains("value% = value% + 3"), "{output}");
        assert!(output.contains("PRINT value%;, \"ok\""), "{output}");
        assert!(output.contains("END"), "{output}");
        assert!(!output.contains("BEEP"), "{output}");
    }

    #[test]
    fn basic_generation_normalizes_boolean_names_from_semantic_ir() {
        let source = "beep\n";
        let semantic_source = "enabled% = true\ndisabled% = false\nend\n";
        let parsed = crate::parse_source("semantic_boolean_names.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("enabled% = -1"), "{output}");
        assert!(output.contains("disabled% = 0"), "{output}");
        assert!(!output.contains("BEEP"), "{output}");
    }

    #[test]
    fn basic_generation_emits_semantic_formatted_and_device_prints() {
        let source = "beep\n";
        let semantic_source = "print using \"###\"; 5\nprint #1, \"file\"\nlprint \"paper\"\nlprint using \"##\"; 7\nend\n";
        let parsed = crate::parse_source("semantic_print_modes.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        for expected in [
            "PRINT USING \"###\"; 5",
            "PRINT #1, \"file\"",
            "LPRINT \"paper\"",
            "LPRINT USING \"##\"; 7",
        ] {
            assert!(output.contains(expected), "missing {expected}: {output}");
        }
        assert!(!output.contains("BEEP"), "{output}");
    }

    #[test]
    fn basic_generation_emits_semantic_file_operations() {
        let source = "beep\n";
        let semantic_source = "open \"file.dat\" for random as #1 len = 128\nfield #1, 4 as record$\nwrite #1, \"x\", 4\ninput #1, value%\nline input #1, text$\nlset record$ = \"left\"\nrset record$ = \"right\"\nget #1, 1\nput #1, 1\nclose #1\nkill \"old.dat\"\nname \"a\" as \"b\"\nend\n";
        let parsed = crate::parse_source("semantic_file_ops.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        for expected in [
            "OPEN \"file.dat\" FOR RANDOM AS #1 LEN = 128",
            "FIELD #1, 4 AS record$",
            "WRITE #1, \"x\", 4",
            "INPUT #1, value%",
            "LINE INPUT #1, text$",
            "LSET record$ = \"left\"",
            "RSET record$ = \"right\"",
            "GET #1, 1",
            "PUT #1, 1",
            "CLOSE #1",
            "KILL \"old.dat\"",
            "NAME \"a\" AS \"b\"",
        ] {
            assert!(output.contains(expected), "missing {expected}: {output}");
        }
        assert!(!output.contains("BEEP"), "{output}");
    }

    #[test]
    fn basic_generation_emits_semantic_machine_and_console_statements() {
        let source = "beep\n";
        let semantic_source = "option base 1\nrandomize 5\nswap x%, y%\npoke 100, 3\nout 888, 1\nwidth #1, 80\nwidth 40\nlocate 2, 3\ncolor 7, 0\ncolor 2\nerror 5\nthrow 6\non choice% goto first, second\nerase values%\nfirst:\nsecond:\nend\n";
        let parsed = crate::parse_source("semantic_machine_io.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        for expected in [
            "OPTION BASE 1",
            "RANDOMIZE 5",
            "SWAP x%, y%",
            "POKE 100, 3",
            "OUT 888, 1",
            "WIDTH #1, 80",
            "WIDTH 40",
            "LOCATE 2, 3",
            "COLOR 7, 0",
            "COLOR 2",
            "ERROR 5",
            "ERROR 6",
            "ON choice% GOTO",
            "ERASE values%",
        ] {
            assert!(output.contains(expected), "missing {expected}: {output}");
        }
        assert!(!output.contains("BEEP"), "{output}");
    }

    #[test]
    fn basic_generation_emits_semantic_if_blocks_and_nested_assignments() {
        let source = "beep\n";
        let semantic_source = "if choice% > 0 then\nresult% = 1\nelse\nresult% = 2\nend if\nend\n";
        let parsed = crate::parse_source("semantic_if_block.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("IF (choice% > 0) = 0 THEN GOTO 10\n    result% = 1\nGOTO 20\n10 result% = 2\n20 REM END IF"), "{output}");
        assert!(output.contains("END"), "{output}");
        assert!(!output.contains("BEEP"), "{output}");
    }

    #[test]
    fn basic_generation_restores_label_counter_when_semantic_if_stream_falls_back() {
        let source = "if true then\nbeep\nend if\nend\n";
        let semantic_source =
            "if true then\nstop\ntry\nthrow 5\ncatch e%, l%\nprint \"semantic\"\nend try\nend if\n";
        let parsed = crate::parse_source("semantic_if_fallback.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("IF (-1) = 0 THEN GOTO 10"), "{output}");
        assert!(output.contains("BEEP"), "{output}");
        assert!(!output.contains("STOP"), "{output}");
    }

    #[test]
    fn basic_generation_emits_semantic_counted_loops() {
        let source = "beep\n";
        let semantic_source = "for i% = 1 to 3 step 2\nsum% += i%\nend for\nfor j% = 3 downto 1\nsum% += j%\nend for\nfor k% = 1 to 2\ncontinue\nend for\nend\n";
        let parsed = crate::parse_source("semantic_for.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("FOR i% = 1 TO 3 STEP 2\n    sum% = sum% + i%\n10 NEXT i%"),
            "{output}"
        );
        assert!(
            output.contains("FOR j% = 3 TO 1 STEP -1\n    sum% = sum% + j%\n20 NEXT j%"),
            "{output}"
        );
        assert!(
            output.contains("GOTO 30\n30 NEXT k%"),
            "CONTINUE needs a line-numbered label before NEXT: {output}"
        );
        assert!(!output.contains("BEEP"), "{output}");

        let semantic_module = resolved.semantic_module.as_mut().unwrap();
        fn find_for(
            statements: &mut [crate::semantic_ir::SemanticStatement],
        ) -> Option<&mut crate::semantic_ir::SemanticStatement> {
            for statement in statements {
                if matches!(statement.kind, crate::semantic_ir::SemanticStatementKind::For { .. }) {
                    return Some(statement);
                }
                if let crate::semantic_ir::SemanticStatementKind::Line(body) = &mut statement.kind {
                    if let Some(statement) = find_for(body) {
                        return Some(statement);
                    }
                }
            }
            None
        }
        let for_statement = find_for(&mut semantic_module.statements).unwrap();
        let crate::semantic_ir::SemanticStatementKind::For { variable_type, .. } =
            &mut for_statement.kind
        else {
            unreachable!()
        };
        *variable_type = crate::semantic_ir::SemanticValueType::Long;
        let typed_output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            typed_output.contains("FOR i& = 1 TO 3 STEP 2"),
            "top-level semantic FOR suffix should come from typed IR: {typed_output}"
        );
    }

    #[test]
    fn basic_generation_transpiles_typed_method_calls_in_for_bounds() {
        let ast_source = "method adjust%[integer](delta%)\nreturn self%+delta%\nend method\ndim index%\nvalue%=10\nif 0 then\nprint 2\nend if\nwhile 0\nprint 2\nend while\ndo while 0\nprint 2\nend do\ndo\nprint 2\nloop until 1\nfor index%=0 to 1 step 1\nprint 2\nend for\nfunction runner%()\ndim local%\nif 0 then\nprint 2\nend if\nwhile 0\nprint 2\nend while\ndo while 0\nprint 2\nend do\ndo\nprint 2\nloop until 1\nfor local%=0 to 1 step 1\nprint 2\nend for\nreturn 0\nend function\nrunner%()\nend\n";
        let semantic_source = "method adjust%[integer](delta%)\nreturn self%+delta%\nend method\ndim index%\nvalue%=20\nif value%.adjust(0) then\nprint index%\nend if\nwhile value%.adjust(-20)\nprint index%\nend while\ndo while value%.adjust(0)\nprint index%\nend do\ndo\nprint index%\nloop until value%.adjust(0)\nfor index%=value%.adjust(1) to value%.adjust(2) step value%.adjust(0)\nprint index%\nend for\nfunction runner%()\ndim local%\nif value%.adjust(0) then\nprint local%\nend if\nwhile value%.adjust(-20)\nprint local%\nend while\ndo while value%.adjust(0)\nprint local%\nend do\ndo\nprint local%\nloop until value%.adjust(0)\nfor local%=value%.adjust(3) to value%.adjust(4) step value%.adjust(0)\nprint local%\nend for\nreturn 0\nend function\nrunner%()\nend\n";
        let parsed =
            crate::parse_source("basic_semantic_method_for.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_method_for.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("FOR index% = BCCT")
                && output.contains(" TO BCCT")
                && output.contains(" STEP BCCT"),
            "typed FOR expressions weren't staged before the loop: {output}"
        );
        assert!(
            output.contains("FOR runnerLocal0% = BCCT"),
            "typed callable FOR expressions weren't staged before the loop: {output}"
        );
        assert!(
            output.matches("GOSUB ").count() >= 10,
            "typed method calls in module and callable conditions/bounds weren't emitted: {output}"
        );
        assert!(
            output.contains("PRINT index%"),
            "typed loop body didn't replace the AST PRINT: {output}"
        );
        assert!(
            !output.contains("PRINT 2"),
            "AST FOR bodies leaked: {output}"
        );
        assert!(
            !output.contains("FOR index% = 0 TO 1"),
            "AST FOR bounds leaked: {output}"
        );
    }

    #[test]
    fn basic_generation_snapshots_for_start_before_method_bounds() {
        let source = "method update%[integer](byref target%)\ntarget%=target%+1\nreturn self%\nend method\nvalue%=5\nbound%=8\ndim index%\nindex%=5\nfor index%=index% to index%.update(index%)\nprint index%\nend for\nfor stepIndex%=value% to bound% step bound%.update(bound%)\nprint stepIndex%\nend for\nfunction runner%()\nfor local%=value% to value%.update(value%)\nprint local%\nend for\nreturn 0\nend function\nrunner%()\nend\n";
        let ast_source = source
            .replace("index%.update(index%)", "1")
            .replace("value%.update(value%)", "1")
            .replace("bound%.update(bound%)", "1");
        let parsed =
            crate::parse_source("basic_for_start_snapshot.bcl".to_string(), &ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("basic_for_start_snapshot.bcl", source)
                .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        let main = output.split("\nEND\n").next().unwrap_or(&output);
        let start_snapshot = main
            .lines()
            .position(|line| line.contains("BCCT") && line.contains(" = index%"))
            .expect("FOR start snapshot");
        let method_call = main
            .lines()
            .position(|line| line.trim_start().starts_with("GOSUB "))
            .unwrap_or_else(|| panic!("method call in FOR bound missing: {main}"));
        assert!(
            start_snapshot < method_call,
            "FOR start was evaluated after its method bound: {main}"
        );
        assert!(
            main.lines().any(|line| line.contains("FOR index% = BCCT")),
            "FOR header did not use the captured start: {main}"
        );
        assert!(
            !main.contains("FOR index% = index%"),
            "FOR header rereads the mutated start variable: {main}"
        );
        assert!(
            output.contains("FOR runnerLocal0% = BCCT"),
            "callable FOR header didn't use its captured start: {output}"
        );
        let lines = output.lines().collect::<Vec<_>>();
        let limit_snapshot = lines
            .iter()
            .position(|line| line.contains("BCCT") && line.contains(" = bound%"))
            .expect("FOR limit snapshot before side-effecting STEP");
        let step_call = lines
            .iter()
            .enumerate()
            .skip(limit_snapshot + 1)
            .find(|(_, line)| line.trim_start().starts_with("GOSUB "))
            .map(|(index, _)| index)
            .expect("method call in FOR STEP");
        assert!(
            limit_snapshot < step_call,
            "FOR limit was read after its STEP method call: {output}"
        );
        assert!(
            output.contains("FOR stepindex% = BCCT"),
            "FOR header did not use the captured limit and start: {output}"
        );
    }

    #[test]
    fn basic_generation_emits_semantic_while_loops() {
        let source = "beep\n";
        let semantic_source = "while value% < 3\nvalue% += 1\nend while\nend\n";
        let parsed = crate::parse_source("semantic_while.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("IF (value% < 3) = 0 THEN GOTO"), "{output}");
        assert!(output.contains("value% = value% + 1"), "{output}");
        assert!(output.contains("GOTO 10"), "{output}");
        assert!(output.contains("REM END WHILE"), "{output}");
        assert!(!output.contains("BEEP"), "{output}");
    }

    #[test]
    fn basic_generation_emits_typed_semantic_select_case_dispatch() {
        let source = "beep\n";
        let semantic_source = "select case choice%\ncase 1 to 3, is >= 9\nresult% = 1\ncase else\nresult% = 0\nend select\nselect case mode\ncase 1\nresult% = 3\nend select\nend\n";
        let parsed = crate::parse_source("semantic_select_case.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("BCCT2% = choice%"), "{output}");
        assert!(output.contains("BCCT2% >= 1 AND BCCT2% <= 3"), "{output}");
        assert!(output.contains("BCCT2% >= 9"), "{output}");
        assert!(output.contains("BCCT4 = mode"), "{output}");
        assert!(
            !output.contains('\0'),
            "semantic temporary contains a NUL: {output:?}"
        );
        assert!(output.contains("REM END SELECT"), "{output}");
        assert!(!output.contains("BEEP"), "{output}");
    }

    #[test]
    fn basic_generation_resolves_semantic_transfers_to_callable_entries() {
        let source = "on error goto worker\nstop\nprocedure worker()\nresume next\nend procedure\n";
        let parsed =
            crate::parse_source("semantic_callable_transfer.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .unwrap();
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("ON ERROR GOTO 10"), "{output}");
    }

    #[test]
    fn basic_semantic_dispatch_emits_top_level_return_from_typed_ir() {
        let module = crate::semantic_ir::parse_and_adapt_named(
            "typed_gosub_return.bcl",
            "return\n",
        )
        .unwrap();
        let lines = super::basic_semantic_intrinsics(
            &mut super::CodeGenerator::new(),
            &module,
            &module.statements,
            true,
        )
        .expect("typed GOSUB RETURN should be supported");

        assert_eq!(lines, ["RETURN"]);
    }

    #[test]
    fn basic_generation_keeps_mixed_semantic_statements_on_compatibility_path() {
        let source = "beep\n";
        let semantic_source = "stop\ntry\nthrow 5\ncatch e%, l%\nprint \"semantic\"\nend try\n";
        let parsed =
            crate::parse_source("mixed_semantic_intrinsic.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("\nBEEP\n"), "{output}");
        assert!(!output.contains("STOP"), "{output}");
    }

    #[test]
    fn basic_generation_dispatches_aligned_semantic_nodes_around_ast_fallbacks() {
        let ast_source = "print 0: print 0\n\nprint 0\ntry\nthrow 5\ncatch e%, l%\nprint \"ast\"\nend try\nprint 0\nend\n";
        let semantic_source = "print 1: print 2\n\nprint 3\ntry\nthrow 6\ncatch e%, l%\nprint \"sem\"\nend try\nprint 4\nend\n";
        let parsed = crate::parse_source("aligned_dispatch.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(
                crate::semantic_ir::parse_and_adapt_named("aligned_dispatch.bcl", semantic_source)
                    .unwrap(),
            ),
        )
        .unwrap();
        assert!(
            super::basic_semantic_statements_by_source(
                &mut super::CodeGenerator::new(),
                resolved.semantic_module.as_ref().unwrap(),
                &resolved.program.statements,
            )
            .is_some(),
            "source-aligned dispatcher declined matching input positions"
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        for expected in ["PRINT 1", "PRINT 2", "PRINT 3", "PRINT 4"] {
            assert!(
                output.contains(expected),
                "missing semantic node {expected}: {output}"
            );
        }
        assert!(
            output.contains("ERROR 6"),
            "semantic TRY body was not emitted: {output}"
        );
        assert!(
            output.contains("PRINT \"sem\""),
            "semantic catch body was not emitted: {output}"
        );
        assert!(
            !output.contains("ERROR 5") && !output.contains("PRINT \"ast\""),
            "AST TRY replaced the typed semantic body: {output}"
        );
        assert!(
            !output.contains("PRINT 0"),
            "AST output replaced supported semantic nodes: {output}"
        );
    }

    #[test]
    fn basic_generation_dispatches_semantic_try_filters_and_finally() {
        let ast_source = "try\nerror 1\ncatch err%(2), erl%\nprint \"ast catch\"\nfinally\nprint \"ast finally\"\nend try\nend\n";
        let semantic_source = "try\nerror 5\ncatch err%(5), erl%\nprint \"semantic catch\"\nfinally\nprint \"semantic finally\"\nend try\nend\n";
        let parsed = crate::parse_source("basic_semantic_try.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("basic_semantic_try.bcl", semantic_source)
                .unwrap(),
        );

        let module = resolved.semantic_module.as_ref().unwrap();
        assert!(
            super::basic_semantic_try_stream_is_typed(module),
            "source-aligned TRY without catch-source mapping should use typed dispatch"
        );
        assert!(
            super::basic_semantic_intrinsics(
                &mut super::CodeGenerator::new(),
                module,
                &module.statements,
                true,
            )
            .is_some(),
            "typed whole-module dispatcher declined semantic TRY"
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("ERROR 5"), "{output}");
        assert!(output.contains("IF (ERR = 5)"), "{output}");
        assert!(output.contains("PRINT \"semantic catch\""), "{output}");
        assert!(output.contains("PRINT \"semantic finally\""), "{output}");
        assert!(!output.contains("ERROR 1"), "{output}");
        assert!(!output.contains("ast catch"), "{output}");
        assert!(!output.contains("ast finally"), "{output}");
    }

    #[test]
    fn basic_program_termination_uses_semantic_ir_when_available() {
        let with_end = crate::semantic_ir::parse_and_adapt("print 1\nend\n").unwrap();
        let without_end = crate::semantic_ir::parse_and_adapt("print 1\n").unwrap();
        assert!(with_end.ends_with_end());
        assert!(!without_end.ends_with_end());
    }

    #[test]
    fn basic_callable_generation_dispatches_aligned_semantic_assignment_and_print() {
        let ast_source = "function value%()\nlocal% = 1\nprint local%\nreturn local%\nend function\nprint value%()\nend\n";
        let semantic_source = "function value%()\nlocal% = 9\nprint 17\nreturn 23\nend function\nprint value%()\nend\n";
        let parsed =
            crate::parse_source("basic_callable_semantic.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains(" = 9"),
            "semantic callable assignment missing: {output}"
        );
        assert!(
            !output.contains(" = 1"),
            "AST callable assignment replaced semantic IR: {output}"
        );
        assert!(
            output.contains("PRINT 17"),
            "semantic callable PRINT missing: {output}"
        );
        assert!(
            !output.contains("PRINT local%"),
            "AST callable PRINT replaced semantic IR: {output}"
        );
        assert!(
            output.contains(" = 23"),
            "semantic callable RETURN missing: {output}"
        );
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_mid_assign() {
        let ast_source = "function edit$(text$)\ntext$ = \"AST\"\nreturn text$\nend function\nprint edit$(\"abcdef\")\nend\n";
        let semantic_source = "function edit$(text$)\nmid$(text$, 4, 2) = \"IR\"\nreturn text$\nend function\nprint edit$(\"abcdef\")\nend\n";
        let parsed = crate::parse_source(
            "basic_callable_semantic_mid_assign.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_mid_assign.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("= 4"),
            "semantic MID$ start missing: {output}"
        );
        assert!(
            output.contains("= 2"),
            "semantic MID$ length missing: {output}"
        );
        assert!(
            output.contains("= \"IR\""),
            "semantic MID$ value missing: {output}"
        );
        assert!(
            !output.contains("\"AST\""),
            "AST MID$ operands replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_callable_mid_assign_evaluates_typed_scalar_calls_once_in_order() {
        let ast_source = "function offset%(byref counter%)\ncounter% = counter% + 1\nreturn counter%\nend function\nfunction position%(first%, second%)\nreturn first% + second%\nend function\nfunction replacement$()\nreturn \"AST\"\nend function\nfunction edit$(text$, counter%)\ntext$ = \"AST\"\nreturn text$\nend function\ncounter% = 0\nprint edit$(\"abcdef\", counter%)\nend\n";
        let semantic_source = "function offset%(byref counter%)\ncounter% = counter% + 1\nreturn counter%\nend function\nfunction position%(first%, second%)\nreturn first% + second%\nend function\nfunction replacement$()\nreturn \"IR\"\nend function\nfunction edit$(text$, counter%)\nmid$(text$, abs(position%(offset%(counter%), offset%(counter%))) + rnd(), 2) = replacement$()\nreturn text$\nend function\ncounter% = 0\nprint edit$(\"abcdef\", counter%)\nend\n";
        let parsed = crate::parse_source(
            "basic_callable_semantic_mid_call.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_mid_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("\"IR\""),
            "typed replacement function was not emitted: {output}"
        );
        assert!(
            !output.contains("\"AST\""),
            "AST MID$ replacement leaked into semantic emission: {output}"
        );
        assert!(
            !output.contains("' function midassign$"),
            "semantic MID$ should emit inline without a helper call: {output}"
        );
        let offset_call = output
            .find("GOSUB 10")
            .unwrap_or_else(|| panic!("nested offset call missing: {output}"));
        let second_offset_call = output[offset_call + 1..]
            .find("GOSUB 10")
            .map(|index| index + offset_call + 1)
            .unwrap_or_else(|| panic!("second nested offset call missing: {output}"));
        let position_call = output
            .find("GOSUB 20")
            .unwrap_or_else(|| panic!("position call missing: {output}"));
        let replacement_call = output
            .find("GOSUB 30")
            .unwrap_or_else(|| panic!("replacement call missing: {output}"));
        let splice = output
            .find("IF LEN(")
            .unwrap_or_else(|| panic!("inline MID$ splice missing: {output}"));
        let abs_snapshot = output
            .find(" = ABS(")
            .expect("nested ABS call was not snapshotted");
        let rnd_snapshot = output
            .find(" = RND")
            .expect("RND operand was not snapshotted");
        assert!(
            offset_call < second_offset_call
                && second_offset_call < position_call
                && position_call < abs_snapshot
                && abs_snapshot < rnd_snapshot
                && rnd_snapshot < replacement_call
                && replacement_call < splice,
            "MID$ operands were not evaluated left-to-right: {output}"
        );
        assert!(
            output.contains("offsetCounter0% = editCounter0%")
                && output.contains("editCounter0% = offsetCounter0%"),
            "by-reference argument was not copied in and back: {output}"
        );
        assert!(
            output.contains("positionFirst0% = BCCT") && output.contains("positionSecond0% = BCCT"),
            "nested call argument was not captured before the outer call: {output}"
        );
        assert!(
            output.contains("LEFT$(BCCT") && output.contains("MID$(BCCT"),
            "semantic MID$ splice expression missing: {output}"
        );
    }

    #[test]
    fn basic_module_mid_assign_dispatches_byref_array_argument_from_semantic_ir() {
        let ast_source = "program semanticArrayMid\ndim values%(1)\nvalues%(0) = 1\nvalue$ = \"abcdef\"\nfunction advance%(byref values%(10))\nvalues%(0) = values%(0) + 1\nreturn values%(0)\nend function\nfunction edit$(value$, byref values%(10))\nvalue$ = \"AST\"\nreturn value$\nend function\nprint edit$(value$, values%)\nprint values%(0)\nend\n";
        let semantic_source = "program semanticArrayMid\ndim values%(1)\nvalues%(0) = 1\nvalue$ = \"abcdef\"\nfunction advance%(byref values%(10))\nvalues%(0) = values%(0) + 1\nreturn values%(0)\nend function\nfunction edit$(value$, byref values%(10))\nmid$(value$, advance%(values%), 2) = \"XY\"\nreturn value$\nend function\nprint edit$(value$, values%)\nprint values%(0)\nend\n";
        let parsed = crate::parse_source(
            "basic_semantic_byref_array_argument.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_byref_array_argument.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("IF LEN("),
            "semantic MID$ splice was not emitted: {output}"
        );
        assert!(
            output.contains("copy array argument into transpiled function storage"),
            "typed array copy-in was not emitted: {output}"
        );
        assert!(
            output.contains("copy mutated array argument back to caller storage"),
            "typed ByRef array copy-back was not emitted: {output}"
        );
        assert!(
            !output.contains("' function midassign$"),
            "semantic MID$ should emit inline without a helper call: {output}"
        );
        assert!(
            !output.contains("\"AST\""),
            "AST assignment replaced semantic MID$: {output}"
        );
    }

    #[test]
    fn basic_module_generation_dispatches_semantic_mid_assign() {
        let ast_source = "text$ = \"abcdef\"\ntext$ = \"AST\"\nprint text$\nend\n";
        let semantic_source = "text$ = \"abcdef\"\nmid$(text$, 4, 2) = \"IR\"\nprint text$\nend\n";
        let parsed = crate::parse_source(
            "basic_module_semantic_mid_assign.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_module_semantic_mid_assign.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("= 4"),
            "semantic MID$ start missing: {output}"
        );
        assert!(
            output.contains("= 2"),
            "semantic MID$ length missing: {output}"
        );
        assert!(
            output.contains("= \"IR\""),
            "semantic MID$ value missing: {output}"
        );
        assert!(
            !output.contains("\"AST\""),
            "AST MID$ operands replaced semantic IR: {output}"
        );
        assert!(
            !output.contains("' function midassign$"),
            "semantic MID$ should emit inline without a helper call: {output}"
        );
    }

    #[test]
    fn basic_ast_mid_assign_transpiles_inline_without_helper_injection() {
        let ast_source = "text$ = \"abcdef\"\nmid$(text$, 2, 2) = \"XY\"\nprint text$\nend\n";
        let parsed =
            crate::parse_source("basic_ast_mid_assign_fallback.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        assert!(
            program.functions.iter().all(|function| {
                !function
                    .name
                    .name
                    .eq_ignore_ascii_case(super::MID_ASSIGN_HELPER_NAME)
            }),
            "AST lowering injected a MID$ helper"
        );
        let resolved = crate::resolver::resolve(program).unwrap();
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("LEFT$(BCCT"),
            "AST fallback did not emit the splice inline: {output}"
        );
        assert!(
            output.contains("MID$(BCCT"),
            "AST fallback did not emit the suffix splice: {output}"
        );
        assert!(
            !output.contains("function midassign$"),
            "AST fallback emitted the legacy helper: {output}"
        );
    }

    #[test]
    fn basic_ast_mid_assign_evaluates_target_start_and_value_once_in_order() {
        let source = "dim values$(2)\nfunction nextIndex%()\nreturn 0\nend function\nfunction nextStart%()\nreturn 2\nend function\nfunction replacement$()\nreturn \"XY\"\nend function\nvalues$(0) = \"abcdef\"\nmid$(values$(nextIndex%()), nextStart%()) = replacement$()\nend\n";
        let parsed =
            crate::parse_source("basic_ast_mid_assign_order.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve(program).unwrap();
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        let target_call = output
            .find("GOSUB 30")
            .unwrap_or_else(|| panic!("target index call missing: {output}"));
        let start_call = output
            .find("GOSUB 40")
            .unwrap_or_else(|| panic!("start call missing: {output}"));
        let value_call = output
            .find("GOSUB 50")
            .unwrap_or_else(|| panic!("replacement call missing: {output}"));
        let splice = output
            .find("IF LEN(")
            .unwrap_or_else(|| panic!("inline MID$ splice missing: {output}"));
        assert!(
            target_call < start_call && start_call < value_call && value_call < splice,
            "AST MID$ operands were not emitted left-to-right: {output}"
        );
        assert!(
            output.contains("LEN(BCCT"),
            "omitted MID$ length did not use the snapshotted replacement: {output}"
        );
        assert_eq!(
            output.matches("GOSUB 30").count(),
            1,
            "target index call must execute once: {output}"
        );
        assert!(
            !output.contains("function midassign$"),
            "AST MID$ emitted the removed compatibility helper: {output}"
        );
    }

    #[test]
    fn basic_module_mid_assign_evaluates_typed_scalar_calls_once_in_order() {
        let ast_source = "function position%()\nreturn 1\nend function\nfunction replacement$()\nreturn \"AST\"\nend function\ntext$ = \"abcdef\"\ntext$ = \"AST\"\nprint text$\nend\n";
        let semantic_source = "function position%()\nreturn 4\nend function\nfunction replacement$()\nreturn \"IR\"\nend function\ntext$ = \"abcdef\"\nmid$(text$, position%(), 2) = replacement$()\nprint text$\nend\n";
        let parsed =
            crate::parse_source("basic_module_semantic_mid_call.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_module_semantic_mid_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        let position_call = output
            .find("GOSUB 30")
            .unwrap_or_else(|| panic!("position call missing: {output}"));
        let replacement_call = output
            .find("GOSUB 40")
            .unwrap_or_else(|| panic!("replacement call missing: {output}"));
        let splice = output
            .find("IF LEN(")
            .unwrap_or_else(|| panic!("inline MID$ splice missing: {output}"));
        assert!(
            position_call < replacement_call && replacement_call < splice,
            "MID$ operands were not evaluated left-to-right: {output}"
        );
        assert!(
            output.contains("\"IR\""),
            "typed replacement function was not emitted: {output}"
        );
        assert!(
            !output.contains("\"AST\""),
            "AST MID$ replacement leaked into semantic emission: {output}"
        );
    }

    #[test]
    fn basic_mid_assign_renders_array_bound_intrinsic_with_nested_callable() {
        let ast_source = "dim names$(3)\nfunction axis%()\nreturn 1\nend function\ntext$ = \"abcdef\"\ntext$ = \"AST\"\nend\n";
        for (intrinsic, expected) in [
            ("sizeof(names$, 0)", 4),
            ("lbound(names$, 0)", 0),
            ("ubound(names$, 0)", 3),
        ] {
            let semantic_source = format!(
                "dim names$(3)\nfunction axis%()\nreturn 1\nend function\ntext$ = \"abcdef\"\nmid$(text$, {intrinsic} + axis%(), 2) = \"IR\"\nend\n"
            );
            let parsed = crate::parse_source(
                "basic_mid_assign_array_bound_call.bcl".to_string(),
                ast_source,
            )
            .unwrap();
            let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
            let semantic = crate::semantic_ir::parse_and_adapt_named(
                "basic_mid_assign_array_bound_call.bcl",
                &semantic_source,
            )
            .unwrap();
            let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();
            let output = super::CodeGenerator::new().generate(&resolved).unwrap();
            assert!(
                output.contains("GOSUB 30"),
                "nested axis function call was not emitted for {intrinsic}: {output}"
            );
            assert!(
                output.contains(&format!("= {expected}")),
                "typed {intrinsic} result was not resolved inside MID$: {output}"
            );
            assert!(
                output.contains("LEFT$(BCCT") && output.contains("MID$(BCCT"),
                "typed MID$ expression did not reach inline splice emission: {output}"
            );
            assert!(
                !output.contains("\"AST\""),
                "AST replacement leaked into semantic output: {output}"
            );
        }
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_lprint_using() {
        let ast_source =
            "function value%()\nlprint \"ast\"; 1\nreturn 0\nend function\nprint value%()\nend\n";
        let semantic_source = "function value%()\nlprint using \"##\"; \"semantic\"; 7\nreturn 0\nend function\nprint value%()\nend\n";
        let parsed =
            crate::parse_source("basic_callable_semantic_lprint.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_lprint.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("LPRINT USING \"##\"; \"semantic\"; 7"),
            "semantic callable LPRINT USING missing: {output}"
        );
        assert!(
            !output.contains("LPRINT \"ast\"; 1"),
            "AST callable LPRINT replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_line_input_and_write() {
        let ast_source = "function readValue%(file%, value$)\nvalue$ = \"ast\"\nvalue$ = \"ast2\"\nvalue$ = \"ast3\"\nreturn 0\nend function\nprint readValue%(1, \"\")\nend\n";
        let semantic_source = "function readValue%(file%, value$)\nline input #file%, value$\nwrite #file%, value$\nclose #file%\nreturn 0\nend function\nprint readValue%(1, \"\")\nend\n";
        let parsed = crate::parse_source(
            "basic_callable_semantic_line_input.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_line_input.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("LINE INPUT #readvalueFile0%, readvalueValue0$"),
            "semantic callable LINE INPUT missing: {output}"
        );
        assert!(
            output.contains("WRITE #readvalueFile0%, readvalueValue0$"),
            "semantic callable WRITE missing: {output}"
        );
        assert!(
            output.contains("CLOSE #readvalueFile0%"),
            "semantic callable CLOSE missing: {output}"
        );
        assert!(
            !output.contains("value$ = \"ast\""),
            "AST assignment replaced semantic file operations: {output}"
        );
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_expression_statements() {
        let ast_source = "function tick%()\nreturn 7\nend function\nfunction run%()\nprint \"ast expression\"\nreturn 0\nend function\nprint run%()\nend\n";
        let semantic_source = "function tick%()\nreturn 7\nend function\nfunction run%()\ntick%()\nreturn 0\nend function\nprint run%()\nend\n";
        let parsed = crate::parse_source(
            "basic_callable_semantic_expression.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_expression.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("GOSUB 10") && !output.contains("BCCT1% = tickResult0%"),
            "semantic expression call was not transpiled: {output}"
        );
        assert!(
            !output.contains("ast expression"),
            "AST expression statement replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_scalar_method_body_dispatches_semantic_statements() {
        // The typed IR names a scalar method by its bare spelling (`shout`);
        // the AST function carries the synthesized `$` result suffix.
        let ast_source = "method shout[string]()\nprint \"ast marker\"\nreturn self$\nend method\nprint \"a\".shout()\nend\n";
        let semantic_source = "method shout[string]()\nn% = len(self$)\nreturn self$\nend method\nprint \"a\".shout()\nend\n";
        let parsed = crate::parse_source("basic_method_semantic.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("basic_method_semantic.bcl", semantic_source)
                .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("LEN(shoutSelf0$)") && !output.contains("ast marker"),
            "scalar method body was not emitted from semantic IR: {output}"
        );
    }

    fn generate_with_diverging_semantic_source(ast_source: &str, semantic_source: &str) -> String {
        let parsed = crate::parse_source("basic_diverging.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("basic_diverging.bcl", semantic_source)
                .unwrap(),
        );
        super::CodeGenerator::new().generate(&resolved).unwrap()
    }

    #[test]
    fn basic_top_level_aligns_a_label_followed_by_a_comment() {
        // The label and its trailing comment are two legacy statements on one
        // line but a single semantic line node; that must not decline the
        // whole stream.
        let output = generate_with_diverging_semantic_source(
            "skip: ' note\nprint \"ast marker\"\nend\n",
            "skip: ' note\nprint \"semantic marker\"\nend\n",
        );
        assert!(
            output.contains("semantic marker") && !output.contains("ast marker"),
            "label plus comment line declined semantic dispatch: {output}"
        );
    }

    #[test]
    fn basic_top_level_ignores_a_trailing_comment_the_ast_discards() {
        let output = generate_with_diverging_semantic_source(
            "x% = 1\nprint \"ast marker\"\nend\n",
            "x% = 1 ' trailing note\nprint \"semantic marker\"\nend\n",
        );
        assert!(
            output.contains("semantic marker") && !output.contains("ast marker"),
            "trailing comment declined semantic dispatch: {output}"
        );
    }

    #[test]
    fn basic_semantic_ordinary_call_on_a_scalar_method_becomes_a_method_call() {
        let method = "method shout[string]()\nreturn self$\nend method\n";
        let output = generate_with_diverging_semantic_source(
            &format!("{method}x$ = \"a\"\nwhile 1\nprint \"ast marker\"\nend while\nend\n"),
            &format!("{method}x$ = \"a\"\nwhile 1\nx$ = shout$(x$)\nend while\nend\n"),
        );
        assert!(
            output.contains("GOSUB") && !output.contains("ast marker"),
            "ordinary-syntax scalar method call declined semantic emission: {output}"
        );
    }

    #[test]
    fn basic_semantic_if_lowers_and_chain_to_one_guard_per_operand() {
        let output = generate_with_diverging_semantic_source(
            "x% = 1\nprint \"ast marker\"\n\n\nend\n",
            "x% = 1\nif x% > 0 && x% < 5 then\nprint 1\nend if\nend\n",
        );
        assert!(!output.contains("ast marker"), "{output}");
        assert_eq!(output.matches("THEN GOTO").count(), 2, "{output}");
        assert!(!output.contains("SC_"), "AND chain needs no skip label: {output}");
    }

    #[test]
    fn basic_semantic_if_lowers_or_chain_with_a_skip_label() {
        let output = generate_with_diverging_semantic_source(
            "x% = 1\nprint \"ast marker\"\n\n\nend\n",
            "x% = 1\nif x% < 0 || x% > 5 then\nprint 1\nend if\nend\n",
        );
        assert!(!output.contains("ast marker"), "{output}");
        assert_eq!(output.matches("<> 0 THEN GOTO").count(), 2, "{output}");
        assert!(
            output.lines().any(|line| line.trim_start().starts_with("GOTO ")),
            "OR chain must skip the body when no operand held: {output}"
        );
    }

    #[test]
    fn basic_semantic_loops_lower_short_circuit_chains() {
        let output = generate_with_diverging_semantic_source(
            "x% = 1\nprint \"ast marker\"\nprint \"ast marker\"\n\n\n\nend\n",
            "x% = 1\nwhile x% > 0 && x% < 5\nx% = x% + 1\nend while\ndo until x% > 9 || x% = 7\nx% = x% + 1\nloop\nend\n",
        );
        assert!(!output.contains("ast marker"), "{output}");
        assert!(output.contains("REM END WHILE"), "{output}");
        assert!(output.contains("REM END DO"), "{output}");
    }

    #[test]
    fn basic_semantic_print_of_ordinary_and_chained_scalar_method_calls() {
        let method = "method shout[string]()\nreturn self$ + \"!\"\nend method\n";
        let output = generate_with_diverging_semantic_source(
            &format!("{method}print \"ast marker\"\nprint \"ast marker\"\nend\n"),
            &format!("{method}print shout$(\"a\")\nprint \"b\".shout().shout()\nend\n"),
        );
        assert!(!output.contains("ast marker"), "{output}");
        let calls = output
            .lines()
            .filter(|line| line.trim_start().starts_with("GOSUB"))
            .count();
        assert_eq!(calls, 3, "{output}");
    }

    #[test]
    fn basic_record_program_is_emitted_from_typed_ir_without_the_ast_statements() {
        let source = "method rtrim$[string]()\n    return self$\nend method\nrecord Part\n    desc: string(20)\n    qty: int16\nend record\n\
             file inv as Part = open(\"inven.dat\")\n\
             inv[1] = { desc: \"x\", qty: 3 }\n\
             let p = inv[1]\nprint p.desc\ninv.close()\nend\n";
        let parsed = crate::parse_source("record_typed_only.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered {
            program,
            lowered_record_files,
            ..
        } = crate::lower::lower(parsed).unwrap();
        let mut semantic =
            crate::semantic_ir::parse_and_adapt_named("record_typed_only.bcl", source).unwrap();
        semantic.lowered_record_files = lowered_record_files;
        assert!(crate::record_transpile::transpile(&mut semantic));
        let mut resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();
        // Everything the AST would emit at top level is gone; the record
        // operations must still come from the typed module.
        resolved.program.statements.clear();

        let output = super::CodeGenerator::new()
            .generate(&resolved)
            .expect("typed record program generates");
        // Buffer names are case-folded here (the driver supplies the
        // synthesized-name set that preserves their case), so compare folded.
        let folded = output.to_ascii_lowercase();
        for expected in [
            "open \"inven.dat\" for random as #1 len = 22",
            "field #1, 20 as invdescbuf$, 2 as invqtybuf$",
            "lset invdescbuf$ = \"x\"",
            "put #1, 1",
            "get #1, 1",
            "rtrimself0$ = invdescbuf$",
            "pdesc$ = bcct",
            "close #1",
        ] {
            assert!(folded.contains(expected), "missing {expected}:\n{output}");
        }
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_terminal_and_error_statements() {
        let ast_source = "function value%()\nprint 1\nprint 2\nprint 3\nprint 4\nprint 5\nprint 6\nprint 7\nreturn 0\nend function\nprint value%()\nend\n";
        let semantic_source = "function value%()\ncls\nbeep\nclear\nstop\nsystem\nerror 9\nthrow\nreturn 0\nend function\nprint value%()\nend\n";
        let parsed = crate::parse_source(
            "basic_callable_semantic_terminal.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_terminal.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        for statement in ["CLS", "BEEP", "CLEAR", "STOP", "SYSTEM"] {
            assert!(
                output.contains(statement),
                "semantic callable {statement} missing: {output}"
            );
        }
        assert!(
            output.contains("ERROR 9"),
            "semantic callable ERROR missing: {output}"
        );
        assert!(
            output.contains("ERROR ERR"),
            "semantic callable bare THROW missing: {output}"
        );
        for ast_statement in [
            "PRINT 1", "PRINT 2", "PRINT 3", "PRINT 4", "PRINT 5", "PRINT 6", "PRINT 7",
        ] {
            assert!(
                !output.contains(ast_statement),
                "AST callable statement leaked in place of semantic IR: {output}"
            );
        }
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_end_statement() {
        let ast_source = "function stop%()\nif true then\nprint 0\nend if\nselect case 1\ncase 1\nprint 0\nend select\nreturn 0\nend function\nprint stop%()\nend\n";
        let semantic_source = "function stop%()\nif true then\nend\nend if\nselect case 1\ncase 1\nend\nend select\nreturn 0\nend function\nprint stop%()\nend\n";
        let parsed = crate::parse_source("semantic_callable_end.bcl".to_string(), ast_source)
            .expect("legacy source parses");
        let crate::lower::Lowered { program, .. } =
            crate::lower::lower(parsed).expect("legacy source lowers");
        let mut resolved = crate::resolver::resolve(program).expect("legacy program resolves");
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named("semantic_callable_end.bcl", semantic_source)
                .expect("typed callable END parses"),
        );

        let output = super::CodeGenerator::new()
            .generate(&resolved)
            .expect("typed callable END transpiles");
        assert!(
            output.matches("END").count() >= 3,
            "nested semantic END statements are missing: {output}"
        );
        assert!(
            !output.contains("PRINT 0"),
            "AST statement replaced typed callable END: {output}"
        );
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_file_and_console_operations() {
        let ast_source = "function value%()\nprint 1\nprint 2\nprint 3\nprint 4\nprint 5\nprint 6\nprint 7\nprint 8\nprint 9\nprint 10\nreturn 0\nend function\nprint value%()\nend\n";
        let semantic_source = "function value%()\nopen \"new.dat\" for output as #1\nseek #1, 3\nget #1, 2\nput #1, 3\nclose #1\nkill \"old.dat\"\nname \"a.dat\" as \"b.dat\"\nwidth #1, 80\nlocate 1, 2\ncolor 7, 0\nreturn 0\nend function\nprint value%()\nend\n";
        let parsed = crate::parse_source(
            "basic_callable_semantic_file_console.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_file_console.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        for expected in [
            "OPEN \"new.dat\" FOR OUTPUT AS #1",
            "SEEK #1, 3",
            "GET #1, 2",
            "PUT #1, 3",
            "CLOSE #1",
            "KILL \"old.dat\"",
            "NAME \"a.dat\" AS \"b.dat\"",
            "WIDTH #1, 80",
            "LOCATE 1, 2",
            "COLOR 7, 0",
        ] {
            assert!(
                output.contains(expected),
                "semantic callable statement {expected} missing: {output}"
            );
        }
        assert!(
            !output.contains("PRINT 1"),
            "AST callable statements replaced semantic operations: {output}"
        );
    }

    #[test]
    fn basic_callable_semantic_function_statement_copies_back_byref() {
        let ast_source = "function increment%(byref value%)\nvalue% = value% + 1\nreturn value%\nend function\nvalue% = 3\nprint \"AST call\"\nreturn\nend\n";
        let semantic_source = "function increment%(byref value%)\nvalue% = value% + 1\nreturn value%\nend function\nvalue% = 5\nincrement%(value%)\nreturn\nend\n";
        let parsed = crate::parse_source(
            "basic_callable_semantic_function_byref.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "basic_callable_semantic_function_byref.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("value% = 5") && output.contains("GOSUB 10"),
            "semantic setup or call missing: {output}"
        );
        assert!(
            output.contains("value% = incrementValue0%"),
            "typed ByRef mutation was not copied back after the discarded call: {output}"
        );
        assert!(
            !output.contains("AST call"),
            "AST call statement replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_callable_semantic_default_return_ignores_ast_expression() {
        let ast_source = "function value%()\nreturn 7\nend function\nprint value%()\nend\n";
        let semantic_source = "function value%()\nreturn\nend function\nprint value%()\nend\n";
        let parsed = crate::parse_source(
            "basic_callable_semantic_default_return.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "basic_callable_semantic_default_return.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("RETURN"),
            "typed default return is missing: {output}"
        );
        assert!(
            !output.contains("value% = 7"),
            "AST return expression replaced the typed default return: {output}"
        );
    }

    #[test]
    fn basic_callable_arity_and_bindings_come_from_semantic_signature() {
        let ast_source =
            "function total%(old%)\nreturn old%\nend function\nresult%=total%(1)\nend\n";
        let semantic_source = "function total%(left%, right%)\nreturn left%+right%\nend function\nresult%=total%(4,5)\nend\n";
        let parsed =
            crate::parse_source("basic_semantic_callable_arity.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_callable_arity.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("' function total%(left%, right%)"),
            "callable parameter listing must come from typed IR: {output}"
        );
        assert!(
            output.contains("totalResult0% = totalLeft0% + totalRight0%"),
            "callable body must use the typed parameter bindings: {output}"
        );
        assert!(
            !output.contains("oldValue0%"),
            "stale AST parameter must not leak into BASIC output: {output}"
        );
        assert!(
            output.contains("totalLeft0% = 4\ntotalRight0% = 5\nGOSUB 10"),
            "typed call arguments must be preserved: {output}"
        );
    }

    #[test]
    fn basic_generation_uses_typed_error_and_throw_call_operands() {
        let ast_source = "function tick%()\nreturn 1\nend function\nfunction fail%()\nerror 2\nthrow 3\nreturn 0\nend function\nerror 4\nthrow 5\nend\n";
        let semantic_source = "function tick%()\nreturn 9\nend function\nfunction fail%()\nerror tick%()\nthrow tick%()\nreturn 0\nend function\nerror tick%()\nthrow tick%()\nend\n";
        let parsed = crate::parse_source(
            "basic_semantic_error_throw_calls.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_error_throw_calls.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.matches("ERROR BCCT").count() == 4,
            "typed ERROR/THROW operands must execute and feed all four statements: {output}"
        );
        assert!(
            !output.contains("ERROR 2")
                && !output.contains("ERROR 3")
                && !output.contains("ERROR 4")
                && !output.contains("ERROR 5"),
            "AST ERROR/THROW operands must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_generation_uses_typed_randomize_call_operands() {
        let ast_source = "function tick%()\nreturn 1\nend function\nfunction seed%()\nrandomize 2\nreturn 0\nend function\nrandomize 3\nend\n";
        let semantic_source = "function tick%()\nreturn 9\nend function\nfunction seed%()\nrandomize tick%()\nreturn 0\nend function\nrandomize tick%()\nend\n";
        let parsed =
            crate::parse_source("basic_semantic_randomize_call.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_randomize_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert_eq!(
            output.matches("RANDOMIZE BCCT").count(),
            2,
            "typed function calls must feed both RANDOMIZE statements: {output}"
        );
        assert!(
            !output.contains("RANDOMIZE 2") && !output.contains("RANDOMIZE 3"),
            "AST RANDOMIZE operands must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_generation_uses_typed_on_branch_selector_calls() {
        let ast_source = "function tick%()\nreturn 1\nend function\nfunction dispatch%()\non 1 goto localA, localB\nlocalA:\nreturn 0\nlocalB:\nreturn 1\nend function\non 1 goto topA, topB\ntopA:\nend\ntopB:\nend\n";
        let semantic_source = "function tick%()\nreturn 9\nend function\nfunction dispatch%()\non tick%() goto localA, localB\nlocalA:\nreturn 0\nlocalB:\nreturn 1\nend function\non tick%() goto topA, topB\ntopA:\nend\ntopB:\nend\n";
        let parsed =
            crate::parse_source("basic_semantic_on_branch_call.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_on_branch_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert_eq!(
            output.matches("ON BCCT").count(),
            2,
            "typed top-level and callable branch selectors must be evaluated: {output}"
        );
        assert!(
            !output.contains("ON 1 GOTO"),
            "AST branch selectors must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_generation_uses_typed_lset_rset_string_call_operands() {
        let ast_source = "function text$()\nreturn \"ast\"\nend function\nfunction update%()\nlset local$ = \"old\"\nrset local$ = \"old\"\nreturn 0\nend function\nlset module$ = \"old\"\nrset module$ = \"old\"\nend\n";
        let semantic_source = "function text$()\nreturn \"typed\"\nend function\nfunction update%()\nlset local$ = text$()\nrset local$ = text$()\nreturn 0\nend function\nlset module$ = text$()\nrset module$ = text$()\nend\n";
        let parsed =
            crate::parse_source("basic_semantic_lset_rset_calls.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_lset_rset_calls.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert_eq!(
            output.matches("LSET ").count(),
            2,
            "both typed LSET statements should emit: {output}"
        );
        assert_eq!(
            output.matches("RSET ").count(),
            2,
            "both typed RSET statements should emit: {output}"
        );
        assert_eq!(
            output.matches("GOSUB 10").count(),
            4,
            "each typed string call must execute before assignment: {output}"
        );
        assert!(
            !output.contains("= \"old\"") && !output.contains("return ast"),
            "AST string operands must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_generation_uses_typed_lset_rset_scalar_method_operands() {
        let ast_source = "method suffix$[string](extra$)\nreturn self$\nend method\nlset local$ = \"old\"\nrset local$ = \"old\"\nlset module$ = \"old\"\nrset module$ = \"old\"\nend\n";
        let semantic_source = "method suffix$[string](extra$)\nreturn self$+extra$\nend method\nlset local$ = name$.suffix(\"typed\")\nrset local$ = name$.suffix(\"typed\")\nlset module$ = name$.suffix(\"typed\")\nrset module$ = name$.suffix(\"typed\")\nend\n";
        let parsed = crate::parse_source(
            "basic_semantic_lset_rset_methods.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_lset_rset_methods.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert_eq!(output.matches("LSET ").count(), 2, "{output}");
        assert_eq!(output.matches("RSET ").count(), 2, "{output}");
        assert_eq!(output.matches("GOSUB 10").count(), 4, "{output}");
        assert!(output.contains("suffixSelf0$ = name$"), "{output}");
        assert!(output.contains("suffixExtra0$ = BCCT"), "{output}");
        assert!(!output.contains("= \"old\""), "{output}");
    }

    #[test]
    fn basic_generation_uses_typed_seek_channel_and_position_calls() {
        let ast_source = "function tick%()\nreturn 1\nend function\nfunction reposition%()\nseek #1, 2\nreturn 0\nend function\nseek #3, 4\nend\n";
        let semantic_source = "function tick%()\nreturn 9\nend function\nfunction reposition%()\nseek #tick%(), tick%()\nreturn 0\nend function\nseek #tick%(), tick%()\nend\n";
        let parsed =
            crate::parse_source("basic_semantic_seek_calls.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_seek_calls.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert_eq!(
            output.matches("SEEK #BCCT").count(),
            2,
            "typed callable and module SEEK operands must be evaluated: {output}"
        );
        assert!(
            !output.contains("SEEK #1, 2") && !output.contains("SEEK #3, 4"),
            "AST SEEK operands must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_generation_uses_typed_get_put_position_calls() {
        let ast_source = "function tick%()\nreturn 1\nend function\nfunction transfer%()\nget #1, 2, 3\nput #1, 2, 3\nreturn 0\nend function\nget #4, 5, 6\nput #4, 5, 6\nend\n";
        let semantic_source = "function tick%()\nreturn 9\nend function\nfunction transfer%()\nget #tick%(), tick%(), tick%()\nput #tick%(), tick%(), tick%()\nreturn 0\nend function\nget #tick%(), tick%(), tick%()\nput #tick%(), tick%(), tick%()\nend\n";
        let parsed =
            crate::parse_source("basic_semantic_get_put_calls.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_get_put_calls.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert_eq!(
            output.matches("GET #BCCT").count(),
            2,
            "typed GET positions must be evaluated in both scopes: {output}"
        );
        assert_eq!(
            output.matches("PUT #BCCT").count(),
            2,
            "typed PUT positions must be evaluated in both scopes: {output}"
        );
        assert!(
            !output.contains("GET #1, 2, 3") && !output.contains("PUT #1, 2, 3"),
            "AST file positions must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_callable_return_function_call_uses_typed_expression_prelude() {
        let ast_source = "function value%(item%)\nreturn item%\nend function\nfunction outer%(arg%)\nreturn 1\nend function\nresult%=outer%(8)\nend\n";
        let semantic_source = "function value%(item%)\nreturn item%+1\nend function\nfunction outer%(arg%)\nreturn value%(arg%)\nend function\nresult%=outer%(8)\nend\n";
        let parsed = crate::parse_source(
            "basic_semantic_callable_return_call.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_callable_return_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("GOSUB 10\n    BCCT1% = valueResult0%\n    outerResult0% = BCCT1%"),
            "typed function call result must be returned from the callable: {output}"
        );
        assert!(
            output.contains("valueItem0% = outerArg0%"),
            "typed argument binding must be emitted in the return prelude: {output}"
        );
        assert!(
            !output.contains("outerResult0% = 1"),
            "AST return expression must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_callable_locate_uses_typed_expression_preludes() {
        let ast_source = "function row%()\nreturn 1\nend function\nfunction position%()\nlocate 1, 2\nreturn 0\nend function\nprint position%()\nend\n";
        let semantic_source = "function row%()\nreturn 9\nend function\nfunction position%()\nlocate row%(), row%()\nreturn 0\nend function\nprint position%()\nend\n";
        let parsed = crate::parse_source(
            "basic_semantic_callable_locate_call.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_callable_locate_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("GOSUB 10") && output.contains("LOCATE BCCT1%, BCCT2%"),
            "typed LOCATE operands must execute before LOCATE emission: {output}"
        );
        assert!(
            !output.contains("LOCATE 1, 2"),
            "AST LOCATE operands must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_callable_color_uses_typed_expression_preludes() {
        let ast_source = "function shade%()\nreturn 1\nend function\nfunction paint%()\ncolor 1, 2\nreturn 0\nend function\nprint paint%()\nend\n";
        let semantic_source = "function shade%()\nreturn 9\nend function\nfunction paint%()\ncolor shade%(), shade%()\nreturn 0\nend function\nprint paint%()\nend\n";
        let parsed = crate::parse_source(
            "basic_semantic_callable_color_call.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_callable_color_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("COLOR BCCT1%, BCCT2%"),
            "typed COLOR operands must execute before COLOR emission: {output}"
        );
        assert!(
            !output.contains("COLOR 1, 2"),
            "AST COLOR operands must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_callable_width_uses_typed_expression_preludes() {
        let ast_source = "function channel%()\nreturn 1\nend function\nfunction resize%()\nwidth #1, 2\nreturn 0\nend function\nprint resize%()\nend\n";
        let semantic_source = "function channel%()\nreturn 9\nend function\nfunction resize%()\nwidth #channel%(), channel%()\nreturn 0\nend function\nprint resize%()\nend\n";
        let parsed = crate::parse_source(
            "basic_semantic_callable_width_call.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_callable_width_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("WIDTH #BCCT1%, BCCT2%"),
            "typed WIDTH operands must execute before WIDTH emission: {output}"
        );
        assert!(
            !output.contains("WIDTH #1, 2"),
            "AST WIDTH operands must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_compatibility_width_evaluates_channel_before_columns() {
        let source = "function channel%()\nreturn 1\nend function\nfunction columns%()\nreturn 2\nend function\nwidth #channel%(), columns%()\nend\n";
        let parsed =
            crate::parse_source("basic_compatibility_width_order.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve(program).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        let channel_call = output.find("GOSUB 10");
        let columns_call = output.find("GOSUB 20");
        let width = output.find("WIDTH #");

        assert!(
            channel_call.is_some_and(|channel| {
                columns_call.is_some_and(|columns| {
                    width.is_some_and(|width| channel < columns && columns < width)
                })
            }),
            "WIDTH compatibility emission must evaluate channel before columns: {output}"
        );
    }

    #[test]
    fn basic_callable_poke_and_out_use_typed_expression_preludes() {
        let ast_source = "function hardware%()\nreturn 1\nend function\nfunction io%()\npoke 1, 2\nout 3, 4\nreturn 0\nend function\nprint io%()\nend\n";
        let semantic_source = "function hardware%()\nreturn 9\nend function\nfunction io%()\npoke hardware%(), hardware%()\nout hardware%(), hardware%()\nreturn 0\nend function\nprint io%()\nend\n";
        let parsed = crate::parse_source(
            "basic_semantic_callable_poke_out_calls.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_callable_poke_out_calls.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(
            output.contains("POKE BCCT1%, BCCT2%") && output.contains("OUT BCCT3%, BCCT4%"),
            "typed POKE and OUT operands must execute before their statements: {output}"
        );
        assert!(
            !output.contains("POKE 1, 2") && !output.contains("OUT 3, 4"),
            "AST POKE/OUT operands must not replace typed IR: {output}"
        );
    }

    #[test]
    fn basic_function_call_uses_typed_default_argument() {
        let ast_source = "function sum%(left%, right%=3)\nreturn left%+right%\nend function\nresult%=sum%(4)\nend\n";
        let semantic_source = "function sum%(left%, right%=8)\nreturn left%+right%\nend function\nresult%=sum%(4)\nend\n";
        let parsed = crate::parse_source(
            "basic_semantic_function_default.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_function_default.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(output.contains("sumRight0% = 8"), "{output}");
        assert!(!output.contains("sumRight0% = 3"), "{output}");
    }

    #[test]
    fn basic_callable_assignment_uses_typed_function_call() {
        let ast_source = "function value%(old%)\nreturn old%\nend function\nfunction outer%()\ntotal%=value%(1)\nreturn total%\nend function\nresult%=outer%()\nend\n";
        let semantic_source = "function value%(item%)\nreturn item%+2\nend function\nfunction outer%()\ntotal%=value%(7)\nreturn total%\nend function\nresult%=outer%()\nend\n";
        let parsed = crate::parse_source(
            "basic_semantic_callable_assignment_call.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_callable_assignment_call.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(output.contains("valueItem0% = 7"), "{output}");
        assert!(!output.contains("valueItem0% = 1"), "{output}");
        assert!(output.contains("outerTotal0% = valueResult0%"), "{output}");
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_machine_statements() {
        let ast_source = "function value%()\nprint 1\nprint 2\nprint 3\nprint 4\nprint 5\nprint 6\nprint 7\nprint 8\nprint 9\nprint 10\nreturn 0\nend function\nprint value%()\nend\n";
        let semantic_source = "function value%()\nglobal typedGlobal%\ndata 7, \"typed\"\nrandomize 5\nswap left%, right%\npoke 100, 3\nout 888, 1\nfield #1, 4 as typed$\nlset record$ = \"left\"\nrset record$ = \"right\"\nerase values%\nreturn 0\nend function\nprint value%()\nend\n";
        let parsed = crate::parse_source(
            "basic_callable_semantic_machine.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_machine.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        for expected in [
            "DATA 7, \"typed\"",
            "RANDOMIZE 5",
            "SWAP ",
            "POKE 100, 3",
            "OUT 888, 1",
            "FIELD #1, 4 AS typed$",
            "LSET ",
            "RSET ",
            "ERASE ",
        ] {
            assert!(
                output.contains(expected),
                "semantic callable operation {expected} missing: {output}"
            );
        }
        assert!(
            !output.contains("PRINT 1"),
            "AST callable statements replaced semantic machine operations: {output}"
        );
        assert!(
            !output.contains("PRINT 2"),
            "AST callable statement replaced semantic DATA: {output}"
        );
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_control_transfers() {
        let ast_source = "function flow%()\nprint 1\nprint 2\nprint 3\nprint 4\nprint 5\nprint 6\nprint 7\nprint 8\nprint 9\nprint 10\nreturn 0\nend function\nprint flow%()\nend\n";
        let semantic_source = "function flow%()\ngoto done\ngosub worker\non 2 goto done, worker\non 2 gosub worker, done\non error goto 0\nresume next\nrestore\nstart:\nworker:\ndone:\nreturn 0\nend function\nprint flow%()\nend\n";
        let parsed = crate::parse_source(
            "basic_callable_semantic_transfers.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_transfers.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        for expected in [
            "GOTO ",
            "GOSUB ",
            "ON 2 GOTO ",
            "ON 2 GOSUB ",
            "ON ERROR GOTO 0",
            "RESUME NEXT",
            "RESTORE",
        ] {
            assert!(
                output.contains(expected),
                "semantic callable transfer {expected} missing: {output}"
            );
        }
        for ast_statement in [
            "PRINT 1", "PRINT 2", "PRINT 3", "PRINT 4", "PRINT 5", "PRINT 6", "PRINT 7", "PRINT 8",
            "PRINT 9", "PRINT 10",
        ] {
            assert!(
                !output.contains(ast_statement),
                "AST callable statements replaced semantic control transfers: {output}"
            );
        }
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_formatted_and_channel_prints() {
        let ast_source = "function value%()\nprint \"ast one\"\nprint \"ast two\"\nreturn 0\nend function\nprint value%()\nend\n";
        let semantic_source = "function value%()\nprint using \"##\"; 7\nprint #1, using \"###\"; 8\nreturn 0\nend function\nprint value%()\nend\n";
        let parsed = crate::parse_source(
            "basic_callable_semantic_print_modes.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_print_modes.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("PRINT USING \"##\"; 7"),
            "semantic callable PRINT USING missing: {output}"
        );
        assert!(
            output.contains("PRINT #1, USING \"###\"; 8"),
            "semantic callable channel PRINT USING missing: {output}"
        );
        assert!(
            !output.contains("ast one") && !output.contains("ast two"),
            "AST callable print replaced semantic output: {output}"
        );
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_block_if() {
        let ast_source = "function result%()\nif flag% then\nprint \"ast then\"\nelse\nprint \"ast else\"\nend if\nreturn 0\nend function\nprint result%()\nend\n";
        let semantic_source = "function result%()\nif flag% then\nvalue% = 11\nelse\nvalue% = 22\nend if\nreturn value%\nend function\nprint result%()\nend\n";
        let parsed =
            crate::parse_source("basic_callable_semantic_if.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_if.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("IF (resultFlag0%) = 0 THEN GOTO "),
            "semantic callable IF missing: {output}"
        );
        assert!(
            output.contains(" = 11") && output.contains(" = 22"),
            "semantic branch assignments missing: {output}"
        );
        assert!(
            !output.contains("ast then") && !output.contains("ast else"),
            "AST callable branches replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_module_assignments_expressions_and_prints_use_typed_ir_without_ast_alignment() {
        let ast_source = "legacy% = 1\nprint legacy%\nend\n";
        let semantic_source = "canonical% = 42\nprint canonical%\nend\n";
        let parsed = crate::parse_source("typed_module_stream.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "typed_module_stream.bcl",
            semantic_source,
        )
        .unwrap();
        let mut resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();
        resolved
            .semantic_module
            .as_mut()
            .unwrap()
            .statement_sources
            .clear();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(output.contains("canonical% = 42"), "{output}");
        assert!(output.contains("PRINT canonical%"), "{output}");
        assert!(!output.contains("legacy%"), "{output}");
    }

    #[test]
    fn basic_semantic_catch_identifiers_use_typed_binding_types() {
        let source = "try\nerror 5\ncatch failure%, linenum%, filename$\nprint failure%\nend try\nend\n";
        let parsed = crate::parse_source("typed_basic_catch.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut semantic =
            crate::semantic_ir::parse_and_adapt_named("typed_basic_catch.bcl", source).unwrap();
        let crate::semantic_ir::SemanticStatementKind::Line(statements) =
            &mut semantic.statements[0].kind
        else {
            panic!()
        };
        let crate::semantic_ir::SemanticStatementKind::Try {
            catch: Some(catch), ..
        } = &mut statements[0].kind
        else {
            panic!()
        };
        catch.error = "failure$".to_string();
        catch.line = "linenum&".to_string();
        catch.source = Some("filename%".to_string());
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(output.contains("failure% = ERR"), "{output}");
        assert!(output.contains("linenum% = ERL"), "{output}");
        assert!(output.contains("filename$ = BCCSOURCEFILE$"), "{output}");
        assert!(!output.contains("failure$ = ERR"), "{output}");
        assert!(!output.contains("linenum& = ERL"), "{output}");
        assert!(!output.contains("filename% = BCCSOURCEFILE$"), "{output}");
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_try_catch_finally() {
        let ast_source = "procedure work()\ntry\nerror 1\ncatch err%(2), erl%, source$\nprint \"ast catch\"\nfinally\nprint \"ast finally\"\nend try\nend procedure\nwork()\nend\n";
        let semantic_source = "procedure work()\ntry\nerror 5\ncatch err%(5), erl%, source$\nprint \"semantic catch\"\nfinally\nprint \"semantic finally\"\nend try\nend procedure\nwork()\nend\n";
        let parsed =
            crate::parse_source("basic_callable_semantic_try.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut semantic = crate::semantic_ir::parse_and_adapt_named(
            "basic_callable_semantic_try.bcl",
            semantic_source,
        )
        .unwrap();
        fn stale_callable_catch_names(
            statements: &mut [crate::semantic_ir::SemanticStatement],
        ) -> bool {
            for statement in statements {
                match &mut statement.kind {
                    crate::semantic_ir::SemanticStatementKind::Try {
                        catch: Some(catch), ..
                    } => {
                        catch.error = "err$".to_string();
                        catch.line = "erl&".to_string();
                        catch.source = Some("source%".to_string());
                        return true;
                    }
                    crate::semantic_ir::SemanticStatementKind::Line(body) => {
                        if stale_callable_catch_names(body) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            false
        }
        assert!(stale_callable_catch_names(&mut semantic.callables[0].body));
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("ERROR 5"), "{output}");
        assert!(output.contains("IF (ERR = 5)"), "{output}");
        assert!(output.contains("PRINT \"semantic catch\""), "{output}");
        assert!(output.contains("PRINT \"semantic finally\""), "{output}");
        assert!(output.contains("workSource0$ = BCCSOURCEFILE$"), "{output}");
        assert!(!output.contains("ERROR 1"), "{output}");
        assert!(!output.contains("ast catch"), "{output}");
        assert!(!output.contains("ast finally"), "{output}");

        let parsed = crate::parse_source(
            "basic_callable_semantic_try.bcl".to_string(),
            semantic_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let compatibility = crate::resolver::resolve(program).unwrap();
        let compatibility_output = super::CodeGenerator::new()
            .generate(&compatibility)
            .unwrap();
        let matching_semantic_source = crate::resolver::resolve_with_semantic(
            compatibility.program.clone(),
            Some(
                crate::semantic_ir::parse_and_adapt_named(
                    "basic_callable_semantic_try.bcl",
                    semantic_source,
                )
                .unwrap(),
            ),
        )
        .unwrap();
        let matching_semantic_output = super::CodeGenerator::new()
            .generate(&matching_semantic_source)
            .unwrap();
        assert_eq!(matching_semantic_output, compatibility_output);
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_for_and_while_loops() {
        let ast_source = "function count%()\nfor i% = 1 to 3 step 1\nprint \"ast for\"\ncontinue\nend for\nwhile value% < 10\nprint \"ast while\"\nexit\nend while\nreturn 0\nend function\nprint count%()\nend\n";
        let semantic_source = "function count%()\nfor i% = 1 to 3 step 1\nvalue% += i%\ncontinue\nend for\nwhile value% < 10\nvalue% += 1\nexit\nend while\nreturn value%\nend function\nprint count%()\nend\n";
        let parsed =
            crate::parse_source("basic_callable_semantic_loops.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_loops.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        assert!(
            super::basic_semantic_callable_statements_by_source(
                resolved.semantic_module.as_ref().unwrap(),
                &resolved.program.functions[0],
            )
            .is_some(),
            "callable loop source alignment failed"
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("FOR countI0% = 1 TO 3 STEP 1"),
            "semantic callable FOR missing: {output}"
        );
        assert!(
            output.contains("NEXT countI0%"),
            "semantic callable NEXT missing: {output}"
        );
        assert!(
            output.contains("IF (countValue0% < 10) = 0 THEN GOTO 40"),
            "semantic callable WHILE missing: {output}"
        );
        assert!(
            output.contains("GOTO 20") && output.contains("GOTO 30"),
            "semantic callable EXIT/CONTINUE targets missing: {output}"
        );
        assert!(
            output.contains("+ countI0%") && output.contains(" + 1"),
            "semantic callable loop assignments missing: {output}"
        );
        assert!(
            !output.contains("ast for") && !output.contains("ast while"),
            "AST callable loops replaced semantic IR: {output}"
        );

        let semantic_module = resolved.semantic_module.as_mut().unwrap();
        fn find_for(
            statements: &mut [crate::semantic_ir::SemanticStatement],
        ) -> Option<&mut crate::semantic_ir::SemanticStatement> {
            for statement in statements {
                if matches!(statement.kind, crate::semantic_ir::SemanticStatementKind::For { .. }) {
                    return Some(statement);
                }
                match &mut statement.kind {
                    crate::semantic_ir::SemanticStatementKind::Line(body)
                    | crate::semantic_ir::SemanticStatementKind::While { body, .. }
                    | crate::semantic_ir::SemanticStatementKind::Do { body, .. } => {
                        if let Some(statement) = find_for(body) {
                            return Some(statement);
                        }
                    }
                    _ => {}
                }
            }
            None
        }
        let for_statement = find_for(&mut semantic_module.callables[0].body).unwrap();
        if let crate::semantic_ir::SemanticStatementKind::For { variable_type, .. } =
            &mut for_statement.kind
        {
            *variable_type = crate::semantic_ir::SemanticValueType::Long;
        } else {
            unreachable!()
        }
        let typed_output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            typed_output.contains("FOR countI0& = 1 TO 3 STEP 1"),
            "semantic FOR identifier suffix should come from typed IR: {typed_output}"
        );
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_select_case() {
        let ast_source = "function result%()\nselect case choice%\ncase 1\nprint \"ast one\"\ncase else\nprint \"ast else\"\nend select\nreturn 0\nend function\nprint result%()\nend\n";
        let semantic_source = "function result%()\nselect case choice%\ncase 1 to 3, is >= 9\nvalue% = 10\ncase else\nvalue% = 20\nend select\nreturn value%\nend function\nprint result%()\nend\n";
        let parsed =
            crate::parse_source("basic_callable_semantic_select.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_select.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("BCCT") && output.contains("<="),
            "semantic callable SELECT CASE dispatch missing: {output}"
        );
        assert!(
            output.contains(" = 10") && output.contains(" = 20"),
            "semantic callable CASE bodies missing: {output}"
        );
        assert!(
            !output.contains("ast one") && !output.contains("ast else"),
            "AST CASE bodies replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_do_loop() {
        let ast_source = "function count%()\ndo while value% < 2\nprint \"ast loop\"\nloop\nreturn 0\nend function\nprint count%()\nend\n";
        let semantic_source = "function count%()\ndo while value% < 2\nvalue% += 1\nloop\nreturn value%\nend function\nprint count%()\nend\n";
        let parsed =
            crate::parse_source("basic_callable_semantic_do.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_do.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        // The loop's top label resolves to a line number that the closing
        // jump targets; a symbolic label must not survive numbering.
        assert!(
            !output.contains("DO_0001_TOP")
                && output.contains("10 IF (countValue0% < 2) = 0 THEN GOTO 30")
                && output.contains("GOTO 10"),
            "semantic callable DO labels missing: {output}"
        );
        assert!(
            output.contains("countValue0% = countValue0% + 1"),
            "semantic callable DO body missing: {output}"
        );
        assert!(
            !output.contains("ast loop"),
            "AST callable DO body replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_input_nodes() {
        let ast_source =
            "function readValue%()\nvalue% = 12\nreturn 0\nend function\nprint readValue%()\nend\n";
        let semantic_source = "function readValue%()\ninput \"semantic\"; value%\nreturn 0\nend function\nprint readValue%()\nend\n";
        let parsed =
            crate::parse_source("basic_callable_semantic_input.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_input.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("INPUT \"semantic\";"),
            "semantic callable INPUT missing: {output}"
        );
        assert!(
            !output.contains(" = 12"),
            "AST callable assignment replaced by semantic INPUT: {output}"
        );
    }

    #[test]
    fn basic_callable_generation_dispatches_semantic_local_array_reads_and_writes() {
        let ast_source = "function value%()\ndim values%(10)\ndim grid%(2, 3)\nvalues%(1) = 2\ngrid%(1, 1) = 3\ngrid%(1, 1) += 4\nprint values%(1), grid%(1, 1)\nreturn values%(1)\nend function\nprint value%()\nend\n";
        let semantic_source = "function value%()\ndim values%(10)\ndim grid%(2, 3)\nvalues%(2) = 7\ngrid%(2, 3) = 9\ngrid%(2, 3) += 12\nprint values%(2), grid%(2, 3)\nreturn values%(2)\nend function\nprint value%()\nend\n";
        let parsed =
            crate::parse_source("basic_callable_semantic_array.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_array.bcl",
                semantic_source,
            )
            .unwrap(),
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("(2) = 7"),
            "semantic callable array write missing: {output}"
        );
        assert!(
            output.contains("valueGrid0%(2, 3) = 9"),
            "semantic callable rank-two array write missing: {output}"
        );
        assert!(
            output.contains("valueGrid0%(2, 3) = valueGrid0%(2, 3) + 12"),
            "semantic callable rank-two compound assignment missing: {output}"
        );
        assert!(
            output.contains("PRINT ") && output.contains("(2)"),
            "semantic callable array read missing: {output}"
        );
        assert!(
            !output.contains("(1) = 2"),
            "AST callable array write replaced semantic IR: {output}"
        );
        assert!(
            !output.contains("valueGrid0%(1, 1) = 3"),
            "AST callable rank-two array write replaced semantic IR: {output}"
        );
        assert!(
            !output.contains("valueGrid0%(1, 1) = valueGrid0%(1, 1) + 4"),
            "AST callable rank-two compound assignment replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_generation_uses_typed_try_stream_when_source_alignment_fails() {
        let ast_source = "beep\n";
        let semantic_source = "stop\ntry\nthrow 6\ncatch e%, l%\nprint \"sem\"\nend try\nend\n";
        let parsed = crate::parse_source("ast_origin.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(
                crate::semantic_ir::parse_and_adapt_named("different_origin.bcl", semantic_source)
                    .unwrap(),
            ),
        )
        .unwrap();
        assert!(
            super::basic_semantic_statements_by_source(
                &mut super::CodeGenerator::new(),
                resolved.semantic_module.as_ref().unwrap(),
                &resolved.program.statements,
            )
            .is_none(),
            "mismatched source identity should decline per-node dispatch"
        );
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("STOP") && output.contains("PRINT \"sem\""),
            "typed try stream was not emitted: {output}"
        );
        assert!(
            !output.contains("BEEP"),
            "AST statements leaked into the typed stream: {output}"
        );
    }

    #[test]
    fn basic_generation_uses_typed_stream_without_ast_source_alignment() {
        let parsed = crate::parse_source("ast_origin.bcl".to_string(), "beep\n").unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic =
            crate::semantic_ir::parse_and_adapt_named("different_origin.bcl", "stop\nend\n")
                .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("\nSTOP\n"),
            "typed stream was not emitted: {output}"
        );
        assert!(
            !output.contains("BEEP"),
            "AST statement was emitted despite a complete typed stream: {output}"
        );
    }

    #[test]
    fn basic_semantic_dispatch_declines_ambiguous_same_line_ast_mapping() {
        let source = "print 0\nend\n";
        let parsed = crate::parse_source("ambiguous_mapping.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { mut program, .. } = crate::lower::lower(parsed).unwrap();
        let duplicate = program.statements[0].clone();
        program.statements.insert(1, duplicate);
        let semantic =
            crate::semantic_ir::parse_and_adapt_named("ambiguous_mapping.bcl", "print 1\nend\n")
                .unwrap();
        let mut generator = super::CodeGenerator::new();
        assert!(
            super::basic_semantic_statements_by_source(
                &mut generator,
                &semantic,
                &program.statements,
            )
            .is_none(),
            "multiple legacy statements at one source line must decline alignment"
        );
        assert!(
            generator.output.is_empty(),
            "ambiguous alignment wrote output before declining"
        );
    }

    #[test]
    fn basic_generation_prefers_semantic_array_dimensions() {
        let source = "dim values%(10)\nend\n";
        let semantic_source = "dim values%(20)\nend\n";
        let parsed =
            crate::parse_source("semantic_array_precedence.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM values%(20)"), "{output}");
        assert!(!output.contains("DIM values%(10)"), "{output}");
    }

    #[test]
    fn basic_generation_dispatches_dim_name_and_bounds_from_semantic_ir() {
        let ast_source = "dim legacy%(2)\nend\n";
        let semantic_source = "dim canonical%(9)\nend\n";
        let parsed =
            crate::parse_source("basic_semantic_dim_dispatch.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_semantic_dim_dispatch.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM canonical%(9)"), "{output}");
        assert!(!output.contains("DIM legacy%"), "{output}");
    }

    #[test]
    fn basic_generation_retains_semantic_array_type_annotations() {
        for type_name in ["LONG", "SINGLE", "DOUBLE", "STRING"] {
            let source = format!("dim values(2) as {}\nend\n", type_name.to_ascii_lowercase());
            let parsed = crate::parse_source("typed_array.bcl".to_string(), &source).unwrap();
            let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
            let resolved = crate::resolver::resolve_with_semantic(
                program,
                Some(crate::semantic_ir::parse_and_adapt(&source).unwrap()),
            )
            .unwrap();
            let output = super::CodeGenerator::new().generate(&resolved).unwrap();
            assert!(
                output.contains(&format!("DIM values(2) AS {type_name}")),
                "{output}"
            );
        }
    }

    #[test]
    fn basic_dim_emission_uses_resolved_element_type() {
        let source = "dim values(2) as long\nend\n";
        let parsed = crate::parse_source("typed_array_ir.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .unwrap();
        let module = resolved.semantic_module.as_mut().unwrap();
        let crate::semantic_ir::SemanticStatementKind::Line(body) = &mut module.statements[0].kind
        else {
            panic!("expected statement line")
        };
        let crate::semantic_ir::SemanticStatementKind::Dim(items) = &mut body[0].kind else {
            panic!("expected DIM")
        };
        items[0].type_annotation = Some("STRING".to_string());

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM values(2) AS LONG"), "{output}");
        assert!(!output.contains("DIM values(2) AS STRING"), "{output}");
    }

    #[test]
    fn basic_semantic_const_suffix_rejects_unresolved_types() {
        assert_eq!(
            super::semantic_const_suffix(crate::semantic_ir::SemanticValueType::Unknown),
            None
        );
        assert_eq!(
            super::semantic_const_suffix(crate::semantic_ir::SemanticValueType::Long),
            Some(crate::ast::TypeSuffix::Long)
        );
    }

    #[test]
    fn basic_unsuffixed_function_uses_typed_default_result_type() {
        let source = "function total()\nreturn 1\nend function\nprint total()\nend\n";
        let parsed = crate::parse_source("unsuffixed_function_result.bcl".to_string(), source)
            .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();

        assert!(output.contains("totalResult0!"), "{output}");
    }

    #[test]
    fn basic_generation_retains_callable_local_array_type_annotations() {
        for type_name in ["long", "single", "double", "string"] {
            let source = format!(
                "function read%()\ndim values(2) as {type_name}\nreturn 0\nend function\nprint read%()\nend\n"
            );
            let parsed =
                crate::parse_source("callable_typed_array.bcl".to_string(), &source).unwrap();
            let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
            let resolved = crate::resolver::resolve_with_semantic(
                program,
                Some(crate::semantic_ir::parse_and_adapt(&source).unwrap()),
            )
            .unwrap();
            let output = super::CodeGenerator::new().generate(&resolved).unwrap();
            assert!(
                output.contains(&format!("AS {}", type_name.to_ascii_uppercase())),
                "{output}"
            );
        }
    }

    #[test]
    fn basic_generation_resolves_bounds_for_every_scalar_suffix() {
        for suffix in ['%', '&', '!', '#', '$'] {
            let source = format!(
                "dim values{suffix}(4)\nconst highest = ubound(values{suffix})\nprint highest\nend\n"
            );
            let parsed =
                crate::parse_source("suffixed_array_bound.bcl".to_string(), &source).unwrap();
            let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
            let resolved = crate::resolver::resolve_with_semantic(
                program,
                Some(crate::semantic_ir::parse_and_adapt(&source).unwrap()),
            )
            .unwrap();
            let output = super::CodeGenerator::new().generate(&resolved).unwrap();
            assert!(
                output.contains("CONSTHIGHEST% = 4"),
                "suffix {suffix}: {output}"
            );
        }
    }

    #[test]
    fn basic_generation_resolves_bounds_for_every_declared_array_type() {
        for type_name in ["integer", "long", "single", "double", "string"] {
            let source = format!(
                "dim values(3) as {type_name}\nconst highest = ubound(values)\nprint highest\nend\n"
            );
            let parsed =
                crate::parse_source("declared_array_bound.bcl".to_string(), &source).unwrap();
            let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
            let resolved = crate::resolver::resolve_with_semantic(
                program,
                Some(crate::semantic_ir::parse_and_adapt(&source).unwrap()),
            )
            .unwrap();
            let output = super::CodeGenerator::new().generate(&resolved).unwrap();
            assert!(
                output.contains("CONSTHIGHEST% = 3"),
                "type {type_name}: {output}"
            );
        }
    }

    #[test]
    fn basic_generation_resolves_semantic_array_bound_constants() {
        let source = "dim values%(4)\nconst count = sizeof(values%)\nprint count\nend\n";
        let semantic_source = "dim values%(9)\nconst count = sizeof(values%)\nprint count\nend\n";
        let parsed = crate::parse_source("semantic_array_bound.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM values%(9)"), "{output}");
        assert!(output.contains("CONSTCOUNT% = 10"), "{output}");
        assert!(!output.contains("CONSTCOUNT% = 5"), "{output}");
    }

    #[test]
    fn basic_generation_dispatches_top_level_const_from_semantic_ir() {
        let ast_source = "const amount = 4\nprint amount\nend\n";
        let semantic_source = "const amount = 9\nprint amount\nend\n";
        let parsed =
            crate::parse_source("semantic_top_level_const.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_top_level_const.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("CONSTAMOUNT% = 9"), "{output}");
        assert!(!output.contains("CONSTAMOUNT% = 4"), "{output}");
    }

    #[test]
    fn basic_generation_dispatches_line_and_block_comments_from_semantic_ir() {
        let ast_source = "function amount%()\n/*\n * AST callable comment\n */\nreturn 1\nend function\n// AST slash module comment\n/*\n * AST module comment\n */\nend\n";
        let semantic_source = "function amount%()\n/*\n * semantic callable comment\n */\nreturn 1\nend function\n// semantic slash module comment\n/*\n * semantic module comment\n */\nend\n";
        let parsed = crate::parse_source("semantic_comments.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic =
            crate::semantic_ir::parse_and_adapt_named("semantic_comments.bcl", semantic_source)
                .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("' semantic callable comment"), "{output}");
        assert!(output.contains("' semantic module comment"), "{output}");
        assert!(
            output.contains("' semantic slash module comment"),
            "{output}"
        );
        assert!(!output.contains("AST callable comment"), "{output}");
        assert!(!output.contains("AST module comment"), "{output}");
    }

    #[test]
    fn basic_callable_generation_dispatches_const_from_semantic_ir() {
        let ast_source = "function amount%()\nconst value = 4\nreturn value\nend function\nprint amount%()\nend\n";
        let semantic_source = "function amount%()\nconst value = 9\nreturn value\nend function\nprint amount%()\nend\n";
        let parsed =
            crate::parse_source("semantic_callable_const.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_callable_const.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("CONSTVALUE% = 9"), "{output}");
        assert!(!output.contains("CONSTVALUE% = 4"), "{output}");
    }

    #[test]
    fn basic_generation_resolves_parenthesized_semantic_array_axis() {
        let ast_source = "dim grid%(1, 1)\nconst highest = ubound(grid%, 0)\nprint highest\nend\n";
        let semantic_source =
            "dim grid%(2, 3)\nconst highest = ubound((grid%), (1))\nprint highest\nend\n";
        let parsed = crate::parse_source(
            "semantic_parenthesized_array_axis.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("CONSTHIGHEST% = 3"),
            "semantic parenthesized axis was not used: {output}"
        );
        assert!(
            !output.contains("CONSTHIGHEST% = 1"),
            "AST axis replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_callable_generation_resolves_parenthesized_semantic_array_axis() {
        let ast_source = "dim grid%(1, 1)\nfunction highest%()\nconst edge = ubound(grid%, 0)\nreturn edge\nend function\nprint highest%()\nend\n";
        let semantic_source = "dim grid%(2, 3)\nfunction highest%()\nconst edge = ubound(grid%, (1))\nreturn edge\nend function\nprint highest%()\nend\n";
        let parsed = crate::parse_source(
            "semantic_callable_parenthesized_array_axis.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("CONSTEDGE% = 3"),
            "callable semantic parenthesized axis was not used: {output}"
        );
        assert!(
            !output.contains("CONSTEDGE% = 1"),
            "AST callable axis replaced semantic IR: {output}"
        );
    }

    #[test]
    fn basic_generation_prefers_semantic_callable_array_dimensions() {
        let source =
            "function read%()\ndim values%(4)\nreturn 0\nend function\nprint read%()\nend\n";
        let semantic_source =
            "function read%()\ndim values%(9)\nreturn 0\nend function\nprint read%()\nend\n";
        let parsed =
            crate::parse_source("callable_array_precedence.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module =
            Some(crate::semantic_ir::parse_and_adapt(semantic_source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("(9)"), "{output}");
        assert!(!output.contains("(4)"), "{output}");
    }

    #[test]
    fn basic_generation_uses_semantic_parameter_capacity_facts() {
        let ast_source = "function consume%(values%(2))\nreturn 0\nend function\nend\n";
        let semantic_source = "function consume%(values%(3))\nreturn 0\nend function\nend\n";
        let parsed =
            crate::parse_source("semantic_parameter_capacity.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_parameter_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(3)"), "{output}");
        assert!(!output.contains("DIM consumeValues0%(2)"), "{output}");
    }

    #[test]
    fn basic_callable_parameter_annotation_drives_typed_identifier() {
        let ast_source =
            "function read%(stale!)\nreturn 0\nend function\nprint read%(1)\nend\n";
        let semantic_source =
            "function read%(value as long)\nreturn value\nend function\nprint read%(1)\nend\n";
        let parsed = crate::parse_source(
            "basic_callable_parameter_annotation.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_parameter_annotation.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("readValue0&"),
            "BASIC callable parameter should use typed-IR annotation: {output}"
        );
        assert!(
            !output.contains("readStale0!"),
            "AST callable parameter leaked into semantic BASIC output: {output}"
        );
    }

    #[test]
    fn basic_unresolved_semantic_parameter_type_returns_diagnostic() {
        let source = "function read%(value%)\nreturn value%\nend function\nend\n";
        let parsed = crate::parse_source("unknown_parameter_type.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        let mut semantic = crate::semantic_ir::parse_and_adapt_named(
            "unknown_parameter_type.bcl",
            source,
        )
        .unwrap();
        semantic.callables[0].parameters[0].value_type =
            crate::semantic_ir::SemanticValueType::Unknown;
        resolved.semantic_module = Some(semantic);

        let diagnostics = super::CodeGenerator::new()
            .generate(&resolved)
            .unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("no resolved BASIC scalar type")),
            "unresolved typed-IR parameter should produce a codegen diagnostic: {diagnostics:?}"
        );
    }

    #[test]
    fn basic_unresolved_semantic_function_result_returns_diagnostic() {
        let source = "function read%()\nreturn 1\nend function\nend\n";
        let parsed = crate::parse_source("unknown_function_result.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        let mut semantic = crate::semantic_ir::parse_and_adapt_named(
            "unknown_function_result.bcl",
            source,
        )
        .unwrap();
        semantic.callables[0].result_type = None;
        resolved.semantic_module = Some(semantic);

        let diagnostics = super::CodeGenerator::new()
            .generate(&resolved)
            .unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("no resolved BASIC result type")),
            "unresolved typed-IR result should produce a codegen diagnostic: {diagnostics:?}"
        );
    }

    #[test]
    fn semantic_parameter_rank_seeds_capacity_when_ast_parameter_is_scalar() {
        let ast_source = "dim actual%(5)\nfunction consume%(values%)\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let semantic_source = "dim actual%(5)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let parsed =
            crate::parse_source("semantic_array_parameter_rank.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_array_parameter_rank.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("consumeValuesDim00% = 5"), "{output}");
        assert!(!output.contains("consumeValuesDim00% = 0"), "{output}");
    }

    #[test]
    fn semantic_fixed_array_parameter_allocates_storage_without_ast_rank() {
        let ast_source = "function consume%(values%)\nreturn 0\nend function\nend\n";
        let semantic_source = "function consume%(values%(5))\nreturn 0\nend function\nend\n";
        let parsed = crate::parse_source(
            "semantic_fixed_array_parameter_rank.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_fixed_array_parameter_rank.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5)"), "{output}");
    }

    #[test]
    fn semantic_procedure_array_parameter_allocates_typed_storage() {
        let ast_source = "procedure consume(values%)\nend procedure\nend\n";
        let semantic_source = "procedure consume(values%(5))\nend procedure\nend\n";
        let parsed = crate::parse_source(
            "semantic_procedure_array_parameter.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_procedure_array_parameter.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5)"), "{output}");
    }

    #[test]
    fn semantic_scalar_parameter_drops_ast_array_capacity() {
        let ast_source = "function consume%(values%(2))\nreturn 0\nend function\nend\n";
        let semantic_source = "function consume%(values%)\nreturn 0\nend function\nend\n";
        let parsed =
            crate::parse_source("semantic_scalar_parameter_rank.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_scalar_parameter_rank.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(!output.contains("DIM consumeValues0%(2)"), "{output}");
        assert!(!output.contains("consumeValuesDim00%"), "{output}");
    }

    #[test]
    fn semantic_parameter_rank_adds_capacity_axes_missing_from_ast() {
        let ast_source = "dim actual%(2, 3)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let semantic_source = "dim actual%(5, 7)\nfunction consume%(values%(?, ?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let parsed =
            crate::parse_source("semantic_parameter_added_rank.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_parameter_added_rank.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("consumeValuesDim00% = 5"), "{output}");
        assert!(output.contains("consumeValuesDim10% = 7"), "{output}");
    }

    #[test]
    fn basic_generation_evaluates_semantic_parameter_capacity_expressions() {
        let ast_source = "function consume%(values%(2))\nreturn 0\nend function\nend\n";
        let semantic_source = "const capacity = 2 + 3\nfunction consume%(values%(capacity))\nreturn 0\nend function\nend\n";
        let parsed = crate::parse_source(
            "semantic_parameter_capacity_expression.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_parameter_capacity_expression.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5)"), "{output}");
        assert!(!output.contains("DIM consumeValues0%(2)"), "{output}");
    }

    #[test]
    fn basic_generation_parses_radix_semantic_parameter_capacity() {
        let ast_source = "function consume%(values%(2))\nreturn 0\nend function\nend\n";
        let semantic_source = "function consume%(values%(&H5))\nreturn 0\nend function\nend\n";
        let parsed = crate::parse_source(
            "semantic_radix_parameter_capacity.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_radix_parameter_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5)"), "{output}");
        assert!(!output.contains("DIM consumeValues0%(2)"), "{output}");
    }

    #[test]
    fn basic_generation_evaluates_division_in_semantic_parameter_capacity() {
        let ast_source = "function consume%(values%(2))\nreturn 0\nend function\nend\n";
        let semantic_source =
            "function consume%(values%((12 / 2) - 1))\nreturn 0\nend function\nend\n";
        let parsed = crate::parse_source(
            "semantic_parameter_capacity_division.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_parameter_capacity_division.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5)"), "{output}");
        assert!(!output.contains("DIM consumeValues0%(2)"), "{output}");
    }

    #[test]
    fn basic_inferred_capacity_parses_radix_semantic_actual_dimension() {
        let ast_source = "dim actual%(2)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let semantic_source = "dim actual%(&H5)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let parsed =
            crate::parse_source("semantic_radix_actual_capacity.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_radix_actual_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5)"), "{output}");
        assert!(!output.contains("DIM consumeValues0%(2)"), "{output}");
    }

    #[test]
    fn semantic_inferred_parameter_capacity_does_not_keep_ast_capacity() {
        let ast_source = "function consume%(values%(2))\nreturn 0\nend function\nend\n";
        let semantic_source = "function consume%(values%(?))\nreturn 0\nend function\nend\n";
        let parsed = crate::parse_source(
            "semantic_inferred_parameter_capacity.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_inferred_parameter_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let diagnostics = super::CodeGenerator::new().generate(&resolved).unwrap_err();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.message.contains("never called") && diagnostic.message.contains("values")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn basic_inferred_parameter_capacity_uses_semantic_actual_array_dimensions() {
        let ast_source = "dim actual%(2)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let semantic_source = "dim actual%(5)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let parsed =
            crate::parse_source("semantic_actual_array_capacity.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_actual_array_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5)"), "{output}");
        assert!(!output.contains("DIM consumeValues0%(2)"), "{output}");
    }

    #[test]
    fn basic_parameter_rank_diagnostic_points_at_the_typed_parameter() {
        let source = "function f%(a%(?))\nreturn a%(1) + a%(1, 2)\nend function\nx% = 0\nprint f%(x%)\nend\n";
        let parsed = crate::parse_source("rank_position.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic =
            crate::semantic_ir::parse_and_adapt_named("rank_position.bcl", source).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let diagnostics = super::CodeGenerator::new().generate(&resolved).unwrap_err();
        let rank = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.message.contains("different numbers of subscripts"))
            .expect("rank diagnostic");
        assert_eq!(rank.pos.filename, "rank_position.bcl", "{rank:?}");
        assert_eq!((rank.pos.line, rank.pos.column), (1, 13), "{rank:?}");
    }

    #[test]
    fn omitted_semantic_array_declaration_does_not_reuse_ast_capacity() {
        let ast_source = "dim actual%(5)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let semantic_source =
            "function consume%(values%(?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let parsed = crate::parse_source(
            "semantic_missing_actual_declaration.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_missing_actual_declaration.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let diagnostics = super::CodeGenerator::new().generate(&resolved).unwrap_err();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.message.contains("can't automatically size")
                    && diagnostic.message.contains("call site passes an array")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn basic_inferred_parameter_capacity_evaluates_module_constants() {
        let ast_source = "dim actual%(2)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let semantic_source = "const capacity = 2 + 3\ndim actual%(capacity)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let parsed = crate::parse_source(
            "semantic_module_actual_array_capacity.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_module_actual_array_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5)"), "{output}");
        assert!(!output.contains("DIM consumeValues0%(2)"), "{output}");
    }

    #[test]
    fn semantic_actual_array_capacity_drives_parameter_overflow_diagnostic() {
        let ast_source = "dim actual%(2)\nfunction consume%(values%(3))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let semantic_source = "dim actual%(5)\nfunction consume%(values%(3))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let parsed =
            crate::parse_source("semantic_actual_array_overflow.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_actual_array_overflow.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let diagnostics = super::CodeGenerator::new().generate(&resolved).unwrap_err();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.message.contains("passes 5 elements")
                    && diagnostic.message.contains("storage is only sized for 3")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn semantic_expression_parameter_capacity_drives_overflow_diagnostic() {
        let ast_source = "dim actual%(2)\nfunction consume%(values%(10))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let semantic_source = "dim actual%(5)\nfunction consume%(values%(1 + 2))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let parsed = crate::parse_source(
            "semantic_expression_capacity_overflow.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_expression_capacity_overflow.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let diagnostics = super::CodeGenerator::new().generate(&resolved).unwrap_err();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.message.contains("passes 5 elements")
                    && diagnostic.message.contains("storage is only sized for 3")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn semantic_multidimensional_overflow_reports_the_exceeding_axis() {
        let ast_source = "dim actual%(2, 3)\nfunction consume%(values%(3, 4))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let semantic_source = "dim actual%(2, 6)\nfunction consume%(values%(3, 4))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let parsed = crate::parse_source(
            "semantic_multidimensional_overflow.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_multidimensional_overflow.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let diagnostics = super::CodeGenerator::new().generate(&resolved).unwrap_err();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic
                    .message
                    .contains("passes 6 elements along axis 1")
                    && diagnostic.message.contains("storage is only sized for 4")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn basic_inferred_capacity_uses_semantic_forwarded_parameter_capacity() {
        let ast_source = "dim actual%(1)\nfunction consume%(values%(?))\nreturn 0\nend function\nfunction forward%(values%(2))\nconsume%(values%)\nreturn 0\nend function\nforward%(actual%)\nend\n";
        let semantic_source = "dim actual%(1)\nfunction consume%(values%(?))\nreturn 0\nend function\nfunction forward%(values%(6))\nconsume%(values%)\nreturn 0\nend function\nforward%(actual%)\nend\n";
        let parsed = crate::parse_source(
            "semantic_forwarded_array_capacity.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_forwarded_array_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(6)"), "{output}");
        assert!(!output.contains("DIM consumeValues0%(2)"), "{output}");
    }

    #[test]
    fn semantic_parameter_expression_capacity_propagates_through_forwarding() {
        let ast_source = "dim actual%(1)\nfunction consume%(values%(?))\nreturn 0\nend function\nfunction forward%(values%(2))\nconsume%(values%)\nreturn 0\nend function\nforward%(actual%)\nend\n";
        let semantic_source = "const capacity = 2 + 3\ndim actual%(1)\nfunction consume%(values%(?))\nreturn 0\nend function\nfunction forward%(values%(capacity))\nconsume%(values%)\nreturn 0\nend function\nforward%(actual%)\nend\n";
        let parsed = crate::parse_source(
            "semantic_expression_forwarded_capacity.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_expression_forwarded_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5)"), "{output}");
        assert!(!output.contains("DIM consumeValues0%(2)"), "{output}");
    }

    #[test]
    fn semantic_dynamic_actual_array_dimension_does_not_use_ast_bound() {
        let ast_source = "dim actual%(5)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let semantic_source = "dim limit%\ndim actual%(limit%)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let parsed = crate::parse_source(
            "semantic_dynamic_actual_array_capacity.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_dynamic_actual_array_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let diagnostics = super::CodeGenerator::new().generate(&resolved).unwrap_err();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.message.contains("can't automatically size")
                    && diagnostic.message.contains("call site passes an array")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn semantic_dynamic_callable_array_dimension_does_not_use_ast_bound() {
        let ast_source = "function consume%(values%(?))\nreturn 0\nend function\nfunction send%()\ndim actual%(5)\nconsume%(actual%)\nreturn 0\nend function\nprint send%()\nend\n";
        let semantic_source = "function consume%(values%(?))\nreturn 0\nend function\nfunction send%()\ndim limit%\ndim actual%(limit%)\nconsume%(actual%)\nreturn 0\nend function\nprint send%()\nend\n";
        let parsed = crate::parse_source(
            "semantic_dynamic_callable_array_capacity.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_dynamic_callable_array_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let diagnostics = super::CodeGenerator::new().generate(&resolved).unwrap_err();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.message.contains("can't automatically size")
                    && diagnostic.message.contains("call site passes an array")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn basic_inferred_parameter_capacity_uses_semantic_multidimensional_bounds() {
        let ast_source = "dim actual%(2, 3)\nfunction consume%(values%(?, ?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let semantic_source = "dim actual%(5, 7)\nfunction consume%(values%(?, ?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let parsed = crate::parse_source(
            "semantic_multidimensional_actual_capacity.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_multidimensional_actual_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5, 7)"), "{output}");
        assert!(!output.contains("DIM consumeValues0%(2, 3)"), "{output}");
    }

    #[test]
    fn dynamic_semantic_bound_blocks_only_its_inferred_axis() {
        let ast_source = "dim actual%(5, 8)\nfunction consume%(values%(?, ?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let semantic_source = "dim limit%\ndim actual%(5, limit%)\nfunction consume%(values%(?, ?))\nreturn 0\nend function\nconsume%(actual%)\nend\n";
        let parsed = crate::parse_source(
            "semantic_dynamic_multidimensional_capacity.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_dynamic_multidimensional_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let diagnostics = super::CodeGenerator::new().generate(&resolved).unwrap_err();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.message.contains("along axis 1")
                    && diagnostic.message.contains("can't automatically size")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn basic_inferred_capacity_uses_maximum_semantic_callsite_bound() {
        let ast_source = "dim first%(1)\ndim second%(2)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(first%)\nconsume%(second%)\nend\n";
        let semantic_source = "dim first%(5)\ndim second%(8)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(first%)\nconsume%(second%)\nend\n";
        let parsed =
            crate::parse_source("semantic_max_callsite_capacity.bcl".to_string(), ast_source)
                .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_max_callsite_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(8)"), "{output}");
        assert!(!output.contains("DIM consumeValues0%(2)"), "{output}");
    }

    #[test]
    fn basic_inferred_capacity_uses_semantic_callsites_without_legacy_call() {
        let filename = "semantic_callsite_capacity_without_ast_call.bcl";
        let parsed = crate::parse_source(
            filename.to_string(),
            "function consume%(values%(?))\nreturn 0\nend function\nend\n",
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            filename,
            "dim actual%(5)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(actual%)\nend\n",
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5)"), "{output}");
    }

    #[test]
    fn dynamic_semantic_callsite_blocks_inference_with_other_known_bounds() {
        let ast_source = "dim known%(5)\ndim dynamic%(8)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(known%)\nconsume%(dynamic%)\nend\n";
        let semantic_source = "dim known%(5)\ndim limit%\ndim dynamic%(limit%)\nfunction consume%(values%(?))\nreturn 0\nend function\nconsume%(known%)\nconsume%(dynamic%)\nend\n";
        let parsed = crate::parse_source(
            "semantic_mixed_callsite_capacity.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_mixed_callsite_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let diagnostics = super::CodeGenerator::new().generate(&resolved).unwrap_err();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.message.contains("can't automatically size")
                    && diagnostic.message.contains("call site passes an array")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn basic_inferred_parameter_capacity_uses_semantic_callable_array_dimensions() {
        let ast_source = "function consume%(values%(?))\nreturn 0\nend function\nfunction send%()\ndim actual%(2)\nconsume%(actual%)\nreturn 0\nend function\nprint send%()\nend\n";
        let semantic_source = "function consume%(values%(?))\nreturn 0\nend function\nfunction send%()\ndim actual%(5)\nconsume%(actual%)\nreturn 0\nend function\nprint send%()\nend\n";
        let parsed = crate::parse_source(
            "semantic_callable_actual_array_capacity.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_callable_actual_array_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5)"), "{output}");
        assert!(!output.contains("DIM consumeValues0%(2)"), "{output}");
    }

    #[test]
    fn unmatched_semantic_callable_scope_keeps_ast_array_bound_fallback() {
        let ast_source = "function consume%(values%(?))\nreturn 0\nend function\nfunction send%()\ndim actual%(5)\nconsume%(actual%)\nreturn 0\nend function\nprint send%()\nend\n";
        let semantic_source = "function consume%(values%(?))\nreturn 0\nend function\nend\n";
        let parsed = crate::parse_source(
            "semantic_unmatched_callable_bound_fallback.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_unmatched_callable_bound_fallback.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5)"), "{output}");
    }

    #[test]
    fn omitted_semantic_callable_array_does_not_reuse_local_ast_capacity() {
        let ast_source = "function consume%(values%(?))\nreturn 0\nend function\nfunction send%()\ndim actual%(5)\nconsume%(actual%)\nreturn 0\nend function\nprint send%()\nend\n";
        let semantic_source = "function consume%(values%(?))\nreturn 0\nend function\nfunction send%()\nconsume%(actual%)\nreturn 0\nend function\nprint send%()\nend\n";
        let parsed = crate::parse_source(
            "semantic_missing_callable_array.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "semantic_missing_callable_array.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let diagnostics = super::CodeGenerator::new().generate(&resolved).unwrap_err();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.message.contains("can't automatically size")
                    && diagnostic.message.contains("call site passes an array")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn basic_inferred_parameter_capacity_evaluates_callable_local_constants() {
        let ast_source = "function consume%(values%(?))\nreturn 0\nend function\nfunction send%()\nconst capacity = 2\ndim actual%(capacity)\nconsume%(actual%)\nreturn 0\nend function\nprint send%()\nend\n";
        let semantic_source = "function consume%(values%(?))\nreturn 0\nend function\nfunction send%()\nconst capacity = 2 + 3\ndim actual%(capacity)\nconsume%(actual%)\nreturn 0\nend function\nprint send%()\nend\n";
        let parsed = crate::parse_source(
            "semantic_callable_local_capacity_constant.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_callable_local_capacity_constant.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5)"), "{output}");
        assert!(!output.contains("DIM consumeValues0%(2)"), "{output}");
    }

    #[test]
    fn basic_inferred_capacity_parses_radix_callable_array_dimension() {
        let ast_source = "function consume%(values%(?))\nreturn 0\nend function\nfunction send%()\ndim actual%(2)\nconsume%(actual%)\nreturn 0\nend function\nprint send%()\nend\n";
        let semantic_source = "function consume%(values%(?))\nreturn 0\nend function\nfunction send%()\ndim actual%(&H5)\nconsume%(actual%)\nreturn 0\nend function\nprint send%()\nend\n";
        let parsed = crate::parse_source(
            "semantic_radix_callable_actual_capacity.bcl".to_string(),
            ast_source,
        )
        .unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let semantic = crate::semantic_ir::parse_and_adapt_named(
            "semantic_radix_callable_actual_capacity.bcl",
            semantic_source,
        )
        .unwrap();
        let resolved = crate::resolver::resolve_with_semantic(program, Some(semantic)).unwrap();

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM consumeValues0%(5)"), "{output}");
        assert!(!output.contains("DIM consumeValues0%(2)"), "{output}");
    }

    #[test]
    fn semantic_array_bound_constants_preserve_unknown_array_diagnostics() {
        let source = "const count = sizeof(missing%)\nprint count\nend\n";
        let parsed = crate::parse_source("unknown_semantic_bound.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let diagnostics = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .err()
        .expect("the resolver rejects the invalid array bound");
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.message.contains("missing%")
                    && diagnostic.message.contains("isn't a known array")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn semantic_array_bound_constants_preserve_invalid_axis_diagnostics() {
        let source = "dim grid%(2, 3)\nconst count = sizeof(grid%, 2)\nprint count\nend\n";
        let parsed =
            crate::parse_source("invalid_semantic_bound_axis.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let diagnostics = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .err()
        .expect("the resolver rejects the invalid array bound");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.message.contains("axis 2 doesn't exist") }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn semantic_sizeof_requires_axis_for_multidimensional_arrays() {
        let source = "dim grid%(2, 3)\nconst count = sizeof(grid%)\nprint count\nend\n";
        let parsed =
            crate::parse_source("missing_semantic_bound_axis.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let diagnostics = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .err()
        .expect("the resolver rejects the invalid array bound");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.message.contains("sizeof needs an axis argument") }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn semantic_array_bound_constants_preserve_nonliteral_axis_diagnostic() {
        let source = "dim grid%(2, 3)\nconst count = sizeof(grid%, 1.5)\nprint count\nend\n";
        let parsed =
            crate::parse_source("nonliteral_semantic_bound_axis.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .unwrap();
        let diagnostics = super::CodeGenerator::new().generate(&resolved).unwrap_err();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.message.contains("axis argument")
                    && diagnostic.message.contains("literal integer")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn semantic_array_bound_constants_reject_negative_axis() {
        let source = "dim grid%(2, 3)\nconst count = sizeof(grid%, -1)\nprint count\nend\n";
        let parsed =
            crate::parse_source("negative_semantic_bound_axis.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .unwrap();
        let diagnostics = super::CodeGenerator::new().generate(&resolved).unwrap_err();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.message.contains("axis argument")
                    && diagnostic.message.contains("literal integer")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn semantic_lbound_and_ubound_require_axis_for_multidimensional_arrays() {
        for builtin in ["lbound", "ubound"] {
            let source =
                format!("dim grid%(2, 3)\nconst result = {builtin}(grid%)\nprint result\nend\n");
            let parsed =
                crate::parse_source("missing_bound_axis.bcl".to_string(), &source).unwrap();
            let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
            let diagnostics = crate::resolver::resolve_with_semantic(
                program,
                Some(crate::semantic_ir::parse_and_adapt(&source).unwrap()),
            )
            .err()
        .expect("the resolver rejects the invalid array bound");
            assert!(
                diagnostics.iter().any(|diagnostic| {
                    diagnostic
                        .message
                        .contains(&format!("{builtin} needs an axis argument"))
                }),
                "{builtin}: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn sizeof_rejects_element_count_overflow_without_panicking() {
        let source =
            "dim huge%(9223372036854775807)\nconst count = sizeof(huge%)\nprint count\nend\n";
        let parsed = crate::parse_source("sizeof_overflow.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .unwrap();
        let diagnostics = super::CodeGenerator::new().generate(&resolved).unwrap_err();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.message.contains("element count")
                    && diagnostic.message.contains("overflows")
            }),
            "{diagnostics:?}"
        );
    }

    #[test]
    fn sizeof_accepts_largest_nonoverflowing_element_count() {
        let source =
            "dim huge%(9223372036854775806)\nconst count = sizeof(huge%)\nprint count\nend\n";
        let parsed = crate::parse_source("sizeof_max_count.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .unwrap();
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("CONSTCOUNT% = 9223372036854775807"),
            "{output}"
        );
    }

    #[test]
    fn ubound_accepts_largest_supported_index_without_incrementing() {
        let source =
            "dim huge%(9223372036854775807)\nconst highest = ubound(huge%)\nprint highest\nend\n";
        let parsed = crate::parse_source("ubound_max_index.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let resolved = crate::resolver::resolve_with_semantic(
            program,
            Some(crate::semantic_ir::parse_and_adapt(source).unwrap()),
        )
        .unwrap();
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(
            output.contains("CONSTHIGHEST% = 9223372036854775807"),
            "{output}"
        );
    }

    #[test]
    fn basic_generation_uses_semantic_callable_string_dim() {
        let source = "function f%()\ndim text as string\nreturn 0\nend function\nend\n";
        let parsed = crate::parse_source("semantic_dim.bcl".to_string(), source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(crate::semantic_ir::parse_and_adapt(source).unwrap());
        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains(" AS STRING"), "{output}");
    }

    #[test]
    fn basic_callable_dim_dispatches_name_and_bounds_from_semantic_ir() {
        let ast_source = "function first%()\ndim legacy%(2)\nreturn legacy%(0)\nend function\nprint first%()\nend\n";
        let semantic_source = "function first%()\ndim canonical%(9)\nreturn canonical%(0)\nend function\nprint first%()\nend\n";
        let parsed =
            crate::parse_source("basic_callable_semantic_dim.bcl".to_string(), ast_source).unwrap();
        let crate::lower::Lowered { program, .. } = crate::lower::lower(parsed).unwrap();
        let mut resolved = crate::resolver::resolve(program).unwrap();
        resolved.semantic_module = Some(
            crate::semantic_ir::parse_and_adapt_named(
                "basic_callable_semantic_dim.bcl",
                semantic_source,
            )
            .unwrap(),
        );

        let output = super::CodeGenerator::new().generate(&resolved).unwrap();
        assert!(output.contains("DIM firstCanonical0%(9)"), "{output}");
        assert!(!output.contains("legacy"), "{output}");
        assert!(output.contains("(9)"), "{output}");
    }
}

/// What a bare `exit` resolves to, tracked per enclosing loop. `for`/`next`
/// compiles to a native BASIC `FOR ... NEXT` block, so leaving it is just
/// BASIC's own `EXIT FOR` -- no label involved, unlike `while`/`do`, which
/// transpile to a GOTO chain and so need a real jump target.
#[derive(Debug, Clone)]
enum LoopExit {
    NativeFor,
    Goto(String),
}

pub struct CodeGenerator {
    next_label: usize,
    indent: usize,
    output: String,
    functions: Vec<FunctionInfo>,
    known_callables: HashSet<String>,
    line_numbers: bool,
    loop_exit_stack: Vec<LoopExit>,
    // The label a `continue` inside the current innermost loop should
    // GOTO -- always a plain label, unlike `loop_exit_stack` (no native-FOR
    // special case: BASIC has no native "skip to the increment" construct
    // the way it has `EXIT FOR`, so even a `for` loop's own continue target
    // is a GOTO to a label placed right before `NEXT`). Pushed/popped in
    // lockstep with `loop_exit_stack` at the same three call sites.
    loop_continue_stack: Vec<String>,
    // All BASIC names already claimed: global vars + every allocated param/result/local name.
    // RefCell because ident() must read and extend this set through a shared &self reference.
    taken_names: RefCell<HashSet<String>>,
    // Lowercase BASIC names of every record/file FIELD buffer variable
    // (from `records::lower`'s `Statement::Field`). These are structurally
    // global -- there is exactly one FIELD-bound buffer per record field,
    // shared by every function/procedure that touches that file -- so
    // ident() must never allocate a per-function local for one, regardless
    // of which scope the LSET/GET/PUT referencing it appears in.
    record_buffer_names: HashSet<String>,
    // Every `const`'s generated BASIC variable name (see `const_var_name`),
    // keyed by lowercase base name (never a suffix -- see
    // resolver AST compatibility metadata for why a reference never
    // carries one either). `ident()` returns this instead of a camelCased
    // rendering of the source name: real BASCOM rejects any identifier
    // containing an underscore outright, even used only as an assignment
    // target (confirmed under real BASCOM/dosbox-x), so a source name like
    // `HELLO_MSG` can't just have its case/underscores lightly touched up
    // the way an ordinary variable's can -- `const_var_name` builds an
    // entirely fresh, underscore-free name instead.
    const_var_names: HashMap<String, String>,
    // Module-level CONST initializer expressions retained by semantic IR.
    // The BASIC emitter consumes the semantic expression directly when it
    // can render it without syntax-specific rewriting.
    semantic_const_initializers: HashMap<String, crate::semantic_ir::Expression>,
    semantic_records: Vec<crate::semantic_ir::Record>,
    semantic_top_level_dims: HashMap<String, crate::semantic_ir::DimDeclaration>,
    // Lowercase BASIC names of the *subset* of `record_buffer_names` that
    // `records::lower` invented itself (via `buffer_ident`), as opposed to
    // a `FIELD` buffer name the author typed directly in raw-BASIC-
    // passthrough source. A synthesized name is already deliberately
    // camelCased and keeps that case in `ident()`; an author-typed one
    // still gets BASCAL's normal lowercase normalization, same as any
    // other identifier.
    synthesized_buffer_names: HashSet<String>,
    // Errors found while generating (e.g. an invalid `byref` call argument).
    // Collected rather than returned immediately since codegen methods are
    // called deep inside statement/expression recursion.
    diagnostics: Vec<Diagnostic>,
    // Declared rank (number of DIM dimensions) of every top-level array,
    // lowercase name -> rank. Used to check a call site's array argument
    // against the callee's own inferred parameter rank.
    top_level_array_ranks: HashMap<String, usize>,
    // Frozen per-axis bound text for every top-level array -- a literal
    // directly, or a generated temp's name if the bound wasn't already a
    // compile-time constant. Populated as each DIM is actually generated.
    // Every array needs this available, not just ones `sizeof()` is called
    // on directly: any array can be passed to a function, and the transpiler
    // auto-injects its bounds at the call site so the callee's own
    // `sizeof()` on that parameter has something to read.
    top_level_array_bounds: HashMap<String, Vec<String>>,
    // Lowercase names of every procedure named as an `on error goto` target
    // somewhere in the program. resolver::validate has already proven each
    // one contains no `return` and never falls off the end (every path
    // ends in resume/goto/end) -- so unlike an ordinary procedure, codegen
    // must NOT append an implicit trailing RETURN for one of these: it's
    // been proven unreachable, and appending it anyway would reintroduce
    // exactly the "RETURN with no GOSUB frame" crash risk the proof exists
    // to rule out.
    error_handler_procedures: HashSet<String>,
    // Synthetic catch labels of lexically enclosing structured tries. This
    // lets a nested try restore its enclosing handler before its finally
    // block runs and rethrow after it.
    try_handler_stack: Vec<String>,
    // Whether any `catch` anywhere in the program binds the optional
    // third (source-filename) variable -- decided once, up front, in
    // `generate()`. Gates all of the per-statement source-file tracking
    // and the `BCC_RESOLVE_SOURCE_FILE` lookup subroutine below, so a
    // program that never uses `catch err%, erl%, source$` gets byte-for-
    // byte the same output it always has.
    needs_source_lookup: bool,
    /// Errors found by rendering helpers that only borrow the generator
    /// immutably (array-bound builtins on a typed path); merged into
    /// `diagnostics` before generation finishes.
    deferred_errors: std::cell::RefCell<Vec<String>>,
    // The original `.bcl` filename `statement()` most recently emitted a
    // `source_file_marker` for -- lets it emit one only when the file
    // actually changes from one statement to the next. `None` until the
    // first statement, guaranteeing that one gets a marker of its own.
    current_marker_file: Option<String>,
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    source_name: BasicIdent,
    stem: String,
    label: String,
    result: BasicIdent,
    /// (resolved source binding, allocated lowered BASIC name) pairs, in declared order.
    params: Vec<(FunctionParameterInfo, BasicIdent)>,
    /// Array rank inferred per parameter from how it's indexed inside this
    /// function's own body (`None` if never directly indexed, or indexed
    /// inconsistently). Parallel to `params`.
    param_ranks: Vec<Option<usize>>,
    /// For each array parameter (rank `Some(n)` in `param_ranks`), the `n`
    /// transpiler-synthesized BASIC variable names that carry its per-axis
    /// bounds -- never written by the `.bcl` author, never appearing at a
    /// call site. The caller sets them (from the actual argument array's
    /// own resolved bounds) immediately before `GOSUB`, alongside the
    /// ordinary copy-in; the callee's body reads them back through
    /// `sizeof()`. Empty `Vec` for a scalar parameter. Parallel to `params`.
    param_bound_vars: Vec<Vec<String>>,
    /// Fixed storage capacity per axis for each array parameter -- how big
    /// the shared storage array named by `param_bound_vars`'s sibling
    /// lowered name actually gets `DIM`ed, once, at top-level. Resolved by
    /// `infer_array_param_capacities` before any `FunctionInfo` is built:
    /// either the largest array ever passed to this parameter across every
    /// call site in the program (when every axis is declared `?`), or an
    /// explicit literal the author wrote instead. Distinct from
    /// `param_bound_vars`, which tracks each individual call's *actual*
    /// size -- capacity is the fixed ceiling that actual size is checked
    /// against at runtime before every call. Empty `Vec` for a scalar
    /// parameter. Parallel to `params`.
    param_capacities: Vec<Vec<i64>>,
    /// Declared rank of every array this function DIMs locally, lowercase
    /// name -> rank.
    local_array_ranks: HashMap<String, usize>,
    semantic_const_initializers: HashMap<String, crate::semantic_ir::Expression>,
    semantic_parameters: Option<Vec<crate::semantic_ir::Parameter>>,
    semantic_dim_declarations: HashMap<String, crate::semantic_ir::DimDeclaration>,
    /// Frozen per-axis bound text for every array DIMed locally within this
    /// function. RefCell because it's populated lazily as DIM statements
    /// are generated, through a shared `&FunctionInfo` reference -- same
    /// pattern as `local_var_map`.
    local_array_bounds: RefCell<HashMap<String, Vec<String>>>,
    is_procedure: bool,
    receiver: Option<TypeSuffix>,
    globals: HashSet<String>,
    // Cache of source-variable-key → allocated lowered BASIC name for locals in this function.
    // RefCell because ident() populates this lazily through a shared &FunctionInfo reference.
    local_var_map: RefCell<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
struct FunctionParameterInfo {
    name: BasicIdent,
    mode: ParamMode,
    /// AST default retained only for the AST compatibility emitter. Normal
    /// codegen reads defaults from `FunctionInfo::semantic_parameters`.
    default: Option<Expr>,
}

/// Emit supported top-level BASIC statements directly from typed semantic
/// IR when the complete module statement stream is in this subset. Returning
/// `None` keeps mixed statement families on the compatibility path.
fn basic_semantic_dim(
    generator: &mut CodeGenerator,
    declarations: &HashMap<String, crate::semantic_ir::DimDeclaration>,
    statement: &crate::semantic_ir::SemanticStatement,
    current_function: Option<&FunctionInfo>,
) -> Option<Vec<String>> {
    let crate::semantic_ir::SemanticStatementKind::Dim(items) = &statement.kind else {
        return None;
    };
    let mut output = Vec::new();
    for item in items {
        let key = item.name.to_ascii_lowercase();
        let declaration = declarations.get(&key)?;
        if declaration.array_axes != item.array_axes
            || declaration
                .dimensions
                .iter()
                .any(|axis| matches!(axis, crate::semantic_ir::DimAxis::Inferred))
        {
            return None;
        }
        let ident = BasicIdent::parse(&declaration.name);
        // Only an explicit `as <type>` becomes a type clause; a bare `dim x`
        // stays `DIM x` (the resolver's default of single is not written out).
        let type_clause = (ident.suffix.is_none() && declaration.type_annotation.is_some())
            .then(|| semantic_dim_type_name(declaration.element_type))
            .flatten()
            .map(|value| format!(" AS {value}"))
            .unwrap_or_default();
        if declaration.array_axes == 0 {
            let base = generator.ident(&ident, current_function);
            output.push(format!("DIM {base}{type_clause}"));
            continue;
        }
        if declaration.dimensions.is_empty() {
            let base = generator.ident(&ident, current_function);
            output.push(format!("DIM {base}(){type_clause}"));
            continue;
        }
        if declaration.dimensions.len() != declaration.array_axes {
            return None;
        }
        let mut axes = Vec::with_capacity(declaration.dimensions.len());
        for axis in &declaration.dimensions {
            let axis = match axis {
                crate::semantic_ir::DimAxis::Fixed(value) => {
                    let literal = value.chars().next().is_some_and(|character| {
                        character.is_ascii_digit() || character == '&' || character == '"'
                    });
                    let rendered = if literal {
                        value.clone()
                    } else {
                        generator.ident(&BasicIdent::parse(value), current_function)
                    };
                    (rendered, literal)
                }
                crate::semantic_ir::DimAxis::Expression(expression) => {
                    let (prelude, rendered) = if let Some(rendered) =
                        generator.semantic_const_expression(expression, current_function)
                    {
                        (Vec::new(), rendered)
                    } else {
                        generator.semantic_expression_with_prelude(expression, current_function)?
                    };
                    output.extend(prelude);
                    let literal = matches!(
                        expression.kind,
                        crate::semantic_ir::ExpressionKind::Literal(_)
                    );
                    (rendered, literal)
                }
                crate::semantic_ir::DimAxis::Inferred => return None,
            };
            axes.push(axis);
        }
        let base = generator.ident(&ident, current_function);
        let rendered = axes
            .iter()
            .map(|(value, _)| value.clone())
            .collect::<Vec<_>>();
        output.push(format!("DIM {base}({}){type_clause}", rendered.join(", ")));
        let frozen = axes
            .into_iter()
            .map(|(value, literal)| {
                if literal {
                    value
                } else {
                    let temp = generator.next_temp_var();
                    output.push(format!("{temp} = {value}"));
                    temp
                }
            })
            .collect();
        if let Some(function) = current_function {
            function.local_array_bounds.borrow_mut().insert(key, frozen);
        } else {
            generator.top_level_array_bounds.insert(key, frozen);
        }
    }
    Some(output)
}

fn basic_semantic_try_stream_is_typed(module: &crate::semantic_ir::SemanticModule) -> bool {
    use crate::semantic_ir::SemanticStatementKind as Kind;
    fn try_support(statements: &[crate::semantic_ir::SemanticStatement]) -> (bool, bool) {
        let mut has_try = false;
        let mut supported = true;
        for statement in statements {
            let bodies: Vec<&[crate::semantic_ir::SemanticStatement]> = match &statement.kind {
                Kind::Try {
                    body,
                    catch,
                    finally_body,
                } => {
                    has_try = true;
                    let mut bodies = vec![body.as_slice(), finally_body.as_slice()];
                    if let Some(catch) = catch {
                        bodies.push(catch.body.as_slice());
                    }
                    bodies
                }
                Kind::Line(body) | Kind::While { body, .. } | Kind::For { body, .. } | Kind::Do { body, .. } => {
                    vec![body]
                }
                Kind::If { then_body, else_body, .. } => vec![then_body, else_body],
                Kind::SelectCase { cases, else_body, .. } => cases
                    .iter()
                    .map(|case| case.body.as_slice())
                    .chain(std::iter::once(else_body.as_slice()))
                    .collect(),
                _ => Vec::new(),
            };
            for body in bodies {
                let (nested_has_try, nested_supported) = try_support(body);
                has_try |= nested_has_try;
                supported &= nested_supported;
            }
        }
        (has_try, supported)
    }

    if module.statement_sources.len() != module.statements.len() {
        return !module
            .statements
            .iter()
            .any(|statement| try_support(std::slice::from_ref(statement)).0);
    }
    module
        .statements
        .iter()
        .zip(&module.statement_sources)
        .all(|(statement, source_index)| {
            let (has_try, supported) = try_support(std::slice::from_ref(statement));
            !has_try || (supported && module.sources.get(*source_index).is_some())
        })
}



fn basic_semantic_intrinsics(
    generator: &mut CodeGenerator,
    module: &crate::semantic_ir::SemanticModule,
    statements: &[crate::semantic_ir::SemanticStatement],
    allow_structured_try: bool,
) -> Option<Vec<String>> {
    use crate::semantic_ir::{
        PrintDestination, PrintToken as SemanticPrintToken, ResumeTarget,
        SemanticStatementKind as Kind,
    };
    fn render_target(generator: &CodeGenerator, name: &str) -> String {
        generator.semantic_label_target_text(name)
    }
    fn visit(
        generator: &mut CodeGenerator,
        module: &crate::semantic_ir::SemanticModule,
        statements: &[crate::semantic_ir::SemanticStatement],
        output: &mut Vec<String>,
        allow_structured_try: bool,
    ) -> bool {
        for statement in statements {
            match &statement.kind {
                Kind::Dim(_) => {
                    let declarations = module.top_level_dim_declarations();
                    let Some(lines) = basic_semantic_dim(generator, &declarations, statement, None)
                    else {
                        return false;
                    };
                    output.extend(lines);
                }
                Kind::FileDeclaration {
                    name,
                    record_type: Some(record_type),
                    path,
                    ..
                } => {
                    let Some(file) = module.lowered_record_files.iter().find(|file| {
                        file.owner.is_none()
                            && file.name.eq_ignore_ascii_case(&name.name)
                            && file.record_type.eq_ignore_ascii_case(&record_type.name)
                    }) else {
                        return false;
                    };
                    let Some((mut lines, path)) =
                        generator.semantic_expression_with_prelude(path, None)
                    else {
                        return false;
                    };
                    let record_length = file.record_length;
                    lines.push(format!(
                        "OPEN {path} FOR RANDOM AS #{} LEN = {record_length}",
                        file.channel
                    ));
                    let bindings = file
                        .fields
                        .iter()
                        .map(|field| {
                            format!(
                                "{} AS {}",
                                field.width,
                                generator.ident(&BasicIdent::parse(&field.buffer_name), None)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    lines.push(format!("FIELD #{}, {bindings}", file.channel));
                    output.extend(lines);
                }
                Kind::Line(body) => {
                    if !visit(generator, module, body, output, allow_structured_try) {
                        return false;
                    }
                }
                Kind::Expression(expression) => {
                    let Some(lines) =
                        basic_semantic_expression_statement(generator, expression, None)
                    else {
                        return false;
                    };
                    output.extend(lines);
                }
                Kind::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    let Some(condition) = generator.semantic_condition(condition, None) else {
                        return false;
                    };
                    let mut then_lines = Vec::new();
                    let mut else_lines = Vec::new();
                    if !visit(
                        generator,
                        module,
                        then_body,
                        &mut then_lines,
                        allow_structured_try,
                    ) || !visit(
                        generator,
                        module,
                        else_body,
                        &mut else_lines,
                        allow_structured_try,
                    ) {
                        return false;
                    }
                    let id = generator.next_label;
                    generator.next_label += 1;
                    let else_label = format!("IF_{id:04}_ELSE");
                    let end_label = format!("IF_{id:04}_END");
                    if else_body.is_empty() {
                        output.extend(condition.jump_lines(&end_label, false));
                        output.extend(then_lines.into_iter().map(|line| format!("    {line}")));
                        output.push(format!("{end_label}:"));
                        output.push("REM END IF".to_string());
                    } else {
                        output.extend(condition.jump_lines(&else_label, false));
                        output.extend(then_lines.into_iter().map(|line| format!("    {line}")));
                        output.push(format!("GOTO {end_label}"));
                        output.push(format!("{else_label}:"));
                        output.extend(else_lines.into_iter().map(|line| format!("    {line}")));
                        output.push(format!("{end_label}:"));
                        output.push("REM END IF".to_string());
                    }
                }
                Kind::For {
                    variable,
                    variable_type,
                    start,
                    bounds,
                    body,
                } => {
                    let start_suffix = start
                        .value_type
                        .suffix()
                        .map(|suffix| suffix.to_string())
                        .unwrap_or_default();
                    let bounds_contain_callable_call = match bounds {
                        crate::semantic_ir::ForBounds::To { limit, step } => {
                            generator.semantic_expression_contains_callable_call(limit)
                                || step.as_ref().is_some_and(|step| {
                                    generator.semantic_expression_contains_callable_call(step)
                                })
                        }
                        crate::semantic_ir::ForBounds::Downto { limit, .. } => {
                            generator.semantic_expression_contains_callable_call(limit)
                        }
                    };
                    let (mut for_prelude, start) =
                        if let Some(start) = generator.semantic_const_expression(start, None) {
                            (Vec::new(), start)
                        } else if let Some((prelude, start)) =
                            generator.semantic_expression_with_prelude(start, None)
                        {
                            (prelude, start)
                        } else {
                            return false;
                        };
                    let start = if bounds_contain_callable_call {
                        let snapshot = generator.next_temp_var_suffixed(&start_suffix);
                        for_prelude.push(format!("{snapshot} = {start}"));
                        snapshot
                    } else {
                        start
                    };
                    let (limit, step) = match bounds {
                        crate::semantic_ir::ForBounds::To {
                            limit: limit_expression,
                            step,
                        } => {
                            let limit = if let Some(limit) =
                                generator.semantic_const_expression(limit_expression, None)
                            {
                                limit
                            } else if let Some((prelude, limit)) =
                                generator.semantic_expression_with_prelude(limit_expression, None)
                            {
                                for_prelude.extend(prelude);
                                limit
                            } else {
                                return false;
                            };
                            let limit = if step.as_ref().is_some_and(|step| {
                                generator.semantic_expression_contains_callable_call(step)
                            }) {
                                let suffix = limit_expression
                                    .value_type
                                    .suffix()
                                    .map(|suffix| suffix.to_string())
                                    .unwrap_or_default();
                                let snapshot = generator.next_temp_var_suffixed(&suffix);
                                for_prelude.push(format!("{snapshot} = {limit}"));
                                snapshot
                            } else {
                                limit
                            };
                            let step = match step {
                                Some(step) => {
                                    let step = if let Some(step) =
                                        generator.semantic_const_expression(step, None)
                                    {
                                        step
                                    } else if let Some((prelude, step)) =
                                        generator.semantic_expression_with_prelude(step, None)
                                    {
                                        for_prelude.extend(prelude);
                                        step
                                    } else {
                                        return false;
                                    };
                                    format!(" STEP {step}")
                                }
                                None => String::new(),
                            };
                            (limit, step)
                        }
                        crate::semantic_ir::ForBounds::Downto { limit, step } => {
                            let limit = if let Some(limit) =
                                generator.semantic_const_expression(limit, None)
                            {
                                limit
                            } else if let Some((prelude, limit)) =
                                generator.semantic_expression_with_prelude(limit, None)
                            {
                                for_prelude.extend(prelude);
                                limit
                            } else {
                                return false;
                            };
                            (limit, format!(" STEP {step}"))
                        }
                    };
                    output.append(&mut for_prelude);
                    let continue_id = generator.next_label;
                    generator.next_label += 1;
                    let continue_label = format!("FOR_{continue_id:04}_CONTINUE");
                    generator.loop_exit_stack.push(LoopExit::NativeFor);
                    generator.loop_continue_stack.push(continue_label.clone());
                    let mut body_lines = Vec::new();
                    let body_supported = visit(
                        generator,
                        module,
                        body,
                        &mut body_lines,
                        allow_structured_try,
                    );
                    generator.loop_continue_stack.pop();
                    generator.loop_exit_stack.pop();
                    if !body_supported {
                        return false;
                    }
                    let mut variable_ident = BasicIdent::parse(variable);
                    variable_ident.suffix = variable_type
                        .suffix()
                        .and_then(crate::ast::TypeSuffix::from_char);
                    let variable = generator.ident(&variable_ident, None);
                    output.push(format!("FOR {variable} = {start} TO {limit}{step}"));
                    output.extend(body_lines.into_iter().map(|line| format!("    {line}")));
                    output.push(format!("{continue_label}:"));
                    output.push(format!("NEXT {variable}"));
                }
                Kind::While { condition, body } => {
                    let Some(condition) = generator.semantic_condition(condition, None) else {
                        return false;
                    };
                    let id = generator.next_label;
                    generator.next_label += 1;
                    let top_label = format!("WHILE_{id:04}_TOP");
                    let end_label = format!("WHILE_{id:04}_END");
                    generator
                        .loop_exit_stack
                        .push(LoopExit::Goto(end_label.clone()));
                    generator.loop_continue_stack.push(top_label.clone());
                    let mut body_lines = Vec::new();
                    let body_supported = visit(
                        generator,
                        module,
                        body,
                        &mut body_lines,
                        allow_structured_try,
                    );
                    generator.loop_continue_stack.pop();
                    generator.loop_exit_stack.pop();
                    if !body_supported {
                        return false;
                    }
                    output.push(format!("{top_label}:"));
                    output.extend(condition.jump_lines(&end_label, false));
                    output.extend(body_lines.into_iter().map(|line| format!("    {line}")));
                    output.push(format!("GOTO {top_label}"));
                    output.push(format!("{end_label}:"));
                    output.push("REM END WHILE".to_string());
                }
                Kind::Do {
                    pre_condition,
                    post_condition,
                    body,
                } => {
                    let pre = match pre_condition {
                        Some(condition) => {
                            let Some(rendered) = generator.semantic_condition(&condition.value, None)
                            else {
                                return false;
                            };
                            Some((condition.kind, rendered))
                        }
                        None => None,
                    };
                    let post = match post_condition {
                        Some(condition) => {
                            let Some(rendered) = generator.semantic_condition(&condition.value, None)
                            else {
                                return false;
                            };
                            Some((condition.kind, rendered))
                        }
                        None => None,
                    };
                    let id = generator.next_label;
                    generator.next_label += 1;
                    let top_label = format!("DO_{id:04}_TOP");
                    let end_label = format!("DO_{id:04}_END");
                    let continue_label = format!("DO_{id:04}_CONTINUE");
                    generator
                        .loop_exit_stack
                        .push(LoopExit::Goto(end_label.clone()));
                    generator.loop_continue_stack.push(continue_label.clone());
                    let mut body_lines = Vec::new();
                    let body_supported = visit(
                        generator,
                        module,
                        body,
                        &mut body_lines,
                        allow_structured_try,
                    );
                    generator.loop_continue_stack.pop();
                    generator.loop_exit_stack.pop();
                    if !body_supported {
                        return false;
                    }
                    output.push(format!("{top_label}:"));
                    if let Some((kind, condition)) = pre {
                        let invert = kind != crate::semantic_ir::LoopConditionKind::While;
                        output.extend(condition.jump_lines(&end_label, invert));
                    }
                    output.extend(body_lines.into_iter().map(|line| format!("    {line}")));
                    output.push(format!("{continue_label}:"));
                    if let Some((kind, condition)) = post {
                        let invert = kind == crate::semantic_ir::LoopConditionKind::While;
                        output.extend(
                            condition
                                .jump_lines(&top_label, invert)
                                .into_iter()
                                .map(|line| format!("    {line}")),
                        );
                    } else {
                        output.push(format!("GOTO {top_label}"));
                    }
                    output.push(format!("{end_label}:"));
                    output.push("REM END DO".to_string());
                }
                Kind::SelectCase {
                    selector,
                    cases,
                    else_body,
                } => {
                    let (selector_prelude, selector_text) = if let Some(rendered) =
                        generator.semantic_const_expression(selector, None)
                    {
                        (Vec::new(), rendered)
                    } else if let Some(rendered) =
                        generator.semantic_expression_with_prelude(selector, None)
                    {
                        rendered
                    } else {
                        return false;
                    };
                    let id = generator.next_label;
                    generator.next_label += 1;
                    let end_label = format!("SEL_{id:04}_END");
                    let temp_id = generator.next_label;
                    generator.next_label += 1;
                    let suffix = selector
                        .value_type
                        .suffix()
                        .map(|suffix| suffix.to_string())
                        .unwrap_or_default();
                    let temp = format!("BCCT{temp_id}{suffix}");
                    let case_labels = (0..cases.len())
                        .map(|index| format!("SEL_{id:04}_C{index}"))
                        .collect::<Vec<_>>();
                    let else_label = format!("SEL_{id:04}_ELSE");
                    let mut rendered_cases = Vec::new();
                    for clause in cases {
                        let mut values = Vec::new();
                        for value in &clause.values {
                            let condition = match value {
                                crate::semantic_ir::CaseValue::Value {
                                    first,
                                    range_end: None,
                                    ..
                                } => {
                                    let Some(value) =
                                        generator.semantic_const_expression(first, None)
                                    else {
                                        return false;
                                    };
                                    format!("{temp} = {value}")
                                }
                                crate::semantic_ir::CaseValue::Value {
                                    first,
                                    range_end: Some(last),
                                    ..
                                } => {
                                    let (Some(first), Some(last)) = (
                                        generator.semantic_const_expression(first, None),
                                        generator.semantic_const_expression(last, None),
                                    ) else {
                                        return false;
                                    };
                                    format!("{temp} >= {first} AND {temp} <= {last}")
                                }
                                crate::semantic_ir::CaseValue::Comparison {
                                    operator,
                                    value,
                                    ..
                                } => {
                                    let Some(value) =
                                        generator.semantic_const_expression(value, None)
                                    else {
                                        return false;
                                    };
                                    let operator = match operator {
                                        crate::semantic_ir::ComparisonOperator::NotEqual => "<>",
                                        crate::semantic_ir::ComparisonOperator::LessOrEqual => "<=",
                                        crate::semantic_ir::ComparisonOperator::GreaterOrEqual => {
                                            ">="
                                        }
                                        crate::semantic_ir::ComparisonOperator::Equal => "=",
                                        crate::semantic_ir::ComparisonOperator::Less => "<",
                                        crate::semantic_ir::ComparisonOperator::Greater => ">",
                                    };
                                    format!("{temp} {operator} {value}")
                                }
                            };
                            values.push(condition);
                        }
                        if values.is_empty() {
                            return false;
                        }
                        rendered_cases.push(values.join(" OR "));
                    }
                    let mut rendered_bodies = Vec::new();
                    for clause in cases {
                        let mut lines = Vec::new();
                        if !visit(
                            generator,
                            module,
                            &clause.body,
                            &mut lines,
                            allow_structured_try,
                        ) {
                            return false;
                        }
                        rendered_bodies.push(lines);
                    }
                    let mut rendered_else = Vec::new();
                    if !visit(
                        generator,
                        module,
                        else_body,
                        &mut rendered_else,
                        allow_structured_try,
                    ) {
                        return false;
                    }

                    output.extend(selector_prelude);
                    output.push(format!("{temp} = {selector_text}"));
                    for (index, condition) in rendered_cases.iter().enumerate() {
                        output.push(format!(
                            "IF ({condition}) <> 0 THEN GOTO {}",
                            case_labels[index]
                        ));
                    }
                    output.push(format!(
                        "GOTO {}",
                        if else_body.is_empty() {
                            &end_label
                        } else {
                            &else_label
                        }
                    ));
                    for (index, lines) in rendered_bodies.into_iter().enumerate() {
                        output.push(format!("{}:", case_labels[index]));
                        output.extend(lines.into_iter().map(|line| format!("    {line}")));
                        output.push(format!("    GOTO {end_label}"));
                    }
                    if !else_body.is_empty() {
                        output.push(format!("{else_label}:"));
                        output.extend(rendered_else.into_iter().map(|line| format!("    {line}")));
                    }
                    output.push(format!("{end_label}:"));
                    output.push("REM END SELECT".to_string());
                }
                Kind::Assignment {
                    target,
                    operator,
                    value,
                } => {
                    let Some(lines) =
                        basic_semantic_assignment(generator, target, *operator, value, None)
                    else {
                        return false;
                    };
                    output.extend(lines);
                }
                Kind::MidAssign {
                    target,
                    start,
                    length,
                    value,
                } => {
                    let Some(lines) = basic_semantic_mid_assign(
                        generator,
                        target,
                        start,
                        length.as_ref(),
                        value,
                        None,
                    ) else {
                        return false;
                    };
                    output.extend(lines);
                }
                Kind::Print {
                    destination,
                    tokens,
                } => {
                    let mut body = String::new();
                    let has_callable_expression = tokens.iter().any(|token| {
                        matches!(token, SemanticPrintToken::Expression(expression)
                            if generator.semantic_expression_contains_callable_call(expression))
                    });
                    let expression_count = tokens
                        .iter()
                        .filter(|token| matches!(token, SemanticPrintToken::Expression(_)))
                        .count();
                    let mut expression_prelude = Vec::new();
                    let mut after_separator = false;
                    for token in tokens {
                        match token {
                            SemanticPrintToken::Expression(expression) => {
                                let expression = if has_callable_expression {
                                    let Some((mut prelude, mut value)) = generator
                                        .semantic_expression_with_prelude(expression, None)
                                    else {
                                        return false;
                                    };
                                    if expression_count == 1
                                        && matches!(expression.kind, crate::semantic_ir::ExpressionKind::Call { .. })
                                        && prelude.last().is_some_and(|line| {
                                            line.strip_prefix(&format!("{value} = ")).is_some()
                                        })
                                    {
                                        let line = prelude.pop().expect("checked call result");
                                        value = line
                                            .split_once(" = ")
                                            .expect("checked call result assignment")
                                            .1
                                            .to_string();
                                        generator.next_label = generator.next_label.saturating_sub(1);
                                    }
                                    expression_prelude.extend(prelude);
                                    if expression_count == 1 {
                                        value
                                    } else {
                                        let suffix = expression
                                            .value_type
                                            .suffix()
                                            .map(|suffix| suffix.to_string())
                                            .unwrap_or_default();
                                        let snapshot = generator.next_temp_var_suffixed(&suffix);
                                        expression_prelude.push(format!("{snapshot} = {value}"));
                                        snapshot
                                    }
                                } else {
                                    let Some(expression) =
                                        generator.semantic_const_expression(expression, None)
                                    else {
                                        return false;
                                    };
                                    expression
                                };
                                if after_separator {
                                    body.push(' ');
                                }
                                body.push_str(&expression);
                                after_separator = false;
                            }
                            SemanticPrintToken::Comma { .. } => {
                                body.push(',');
                                after_separator = true;
                            }
                            SemanticPrintToken::Semicolon { .. } => {
                                body.push(';');
                                after_separator = true;
                            }
                        }
                    }
                    let (destination_prelude, rendered_destination) = match destination {
                        PrintDestination::Standard { .. } => (Some(Vec::new()), Some(String::new())),
                        PrintDestination::Using { format, .. } => generator
                            .semantic_expression_with_prelude(format, None)
                            .map(|(lines, format)| (Some(lines), Some(format!(" USING {format}"))))
                            .unwrap_or((None, None)),
                        PrintDestination::Channel { channel, using, .. } => {
                            let Some((mut lines, channel)) =
                                generator.semantic_expression_with_prelude(channel, None)
                            else {
                                return false;
                            };
                            match using {
                                Some(format) => {
                                    let Some((format_lines, format)) = generator
                                        .semantic_expression_with_prelude(format, None)
                                    else {
                                        return false;
                                    };
                                    lines.extend(format_lines);
                                    (Some(lines), Some(format!(" #{channel}, USING {format}")))
                                }
                                None => (Some(lines), Some(format!(" #{channel}"))),
                            }
                        }
                    };
                    let Some(destination_prelude) = destination_prelude else {
                        return false;
                    };
                    let Some(destination) = rendered_destination else {
                        return false;
                    };
                    output.extend(destination_prelude);
                    output.extend(expression_prelude);
                    let prefix = "PRINT";
                    let separator = if destination.starts_with(" USING") {
                        "; "
                    } else if destination.contains(", USING ") {
                        "; "
                    } else {
                        ", "
                    };
                    output.push(if body.is_empty() {
                        format!("{prefix}{destination}")
                    } else if destination.starts_with(" USING") || destination.contains(", USING ")
                    {
                        format!("{prefix}{destination}{separator}{body}")
                    } else if destination.is_empty() {
                        format!("{prefix} {body}")
                    } else {
                        format!("{prefix}{destination}{separator}{body}")
                    });
                }
                Kind::Lprint { using, tokens } => {
                    let (using_prelude, using) = match using {
                        Some(format) => {
                            let Some((lines, format)) =
                                generator.semantic_expression_with_prelude(format, None)
                            else {
                                return false;
                            };
                            (lines, Some(format))
                        }
                        None => (Vec::new(), None),
                    };
                    let Some((lines, body)) = basic_semantic_print_body(generator, tokens, None)
                    else {
                        return false;
                    };
                    output.extend(using_prelude);
                    output.extend(lines);
                    output.push(match (using, body.is_empty()) {
                        (Some(format), true) => format!("LPRINT USING {format}"),
                        (Some(format), false) => format!("LPRINT USING {format};{body}"),
                        (None, true) => "LPRINT".to_string(),
                        (None, false) => format!("LPRINT{body}"),
                    });
                }
                Kind::Write { channel, values } => {
                    let Some((mut lines, channel)) =
                        generator.semantic_expression_with_prelude(channel, None)
                    else {
                        return false;
                    };
                    let (value_lines, values) = match values {
                        crate::semantic_ir::WriteValues::Omitted => (Vec::new(), Vec::new()),
                        crate::semantic_ir::WriteValues::Values(values) => {
                            let Some(rendered) = basic_semantic_io_values(generator, values, None)
                            else {
                                return false;
                            };
                            rendered
                        }
                    };
                    lines.extend(value_lines);
                    let suffix = if values.is_empty() {
                        String::new()
                    } else {
                        format!(", {}", values.join(", "))
                    };
                    output.extend(lines);
                    output.push(format!("WRITE #{channel}{suffix}"));
                }
                Kind::Close(channel) => {
                    let Some((lines, channel)) =
                        generator.semantic_expression_with_prelude(channel, None)
                    else {
                        return false;
                    };
                    output.extend(lines);
                    output.push(format!("CLOSE #{channel}"));
                }
                Kind::Kill(path) => {
                    let Some((lines, path)) =
                        generator.semantic_expression_with_prelude(path, None)
                    else {
                        return false;
                    };
                    output.extend(lines);
                    output.push(format!("KILL {path}"));
                }
                Kind::Rename {
                    source,
                    destination,
                } => {
                    let Some((mut lines, source)) =
                        generator.semantic_expression_with_prelude(source, None)
                    else {
                        return false;
                    };
                    let Some((destination_lines, destination)) =
                        generator.semantic_expression_with_prelude(destination, None)
                    else {
                        return false;
                    };
                    lines.extend(destination_lines);
                    output.extend(lines);
                    output.push(format!("NAME {source} AS {destination}"));
                }
                Kind::Seek { channel, position } => {
                    let Some((mut lines, channel)) =
                        generator.semantic_expression_with_prelude(channel, None)
                    else {
                        return false;
                    };
                    let Some((position_lines, position)) =
                        generator.semantic_expression_with_prelude(position, None)
                    else {
                        return false;
                    };
                    lines.extend(position_lines);
                    output.extend(lines);
                    output.push(format!("SEEK #{channel}, {position}"));
                }
                Kind::Open {
                    path,
                    mode,
                    channel,
                    length,
                } => {
                    let Some((mut lines, path)) =
                        generator.semantic_expression_with_prelude(path, None)
                    else {
                        return false;
                    };
                    let Some((channel_lines, channel)) =
                        generator.semantic_expression_with_prelude(channel, None)
                    else {
                        return false;
                    };
                    lines.extend(channel_lines);
                    let length = match length {
                        Some(length) => {
                            let Some((length_lines, length)) =
                                generator.semantic_expression_with_prelude(length, None)
                            else {
                                return false;
                            };
                            lines.extend(length_lines);
                            format!(" LEN = {length}")
                        }
                        None => String::new(),
                    };
                    let mode = match mode.kind {
                        crate::semantic_ir::OpenModeKind::Input => "INPUT",
                        crate::semantic_ir::OpenModeKind::Output => "OUTPUT",
                        crate::semantic_ir::OpenModeKind::Append => "APPEND",
                        crate::semantic_ir::OpenModeKind::Random => "RANDOM",
                        crate::semantic_ir::OpenModeKind::Binary => "BINARY",
                    };
                    output.extend(lines);
                    output.push(format!("OPEN {path} FOR {mode} AS #{channel}{length}"));
                }
                Kind::LineInput { channel, target } => {
                    let Some((mut lines, channel)) =
                        generator.semantic_expression_with_prelude(channel, None)
                    else {
                        return false;
                    };
                    let Some((target_lines, target)) =
                        basic_semantic_lvalue(generator, target, None)
                    else {
                        return false;
                    };
                    lines.extend(target_lines);
                    output.extend(lines);
                    output.push(format!("LINE INPUT #{channel}, {target}"));
                }
                Kind::Input { source, targets } => {
                    let mut prelude = Vec::new();
                    let prefix = match source {
                        crate::semantic_ir::InputSource::Console(prompt) => {
                            let prompt = prompt
                                .as_ref()
                                .map(basic_semantic_input_prompt)
                                .unwrap_or_default();
                            format!("INPUT {prompt}")
                        }
                        crate::semantic_ir::InputSource::Channel(channel) => {
                            let Some((lines, channel)) =
                                generator.semantic_expression_with_prelude(channel, None)
                            else {
                                return false;
                            };
                            prelude = lines;
                            format!("INPUT #{channel}, ")
                        }
                    };
                    let mut rendered_targets = Vec::with_capacity(targets.len());
                    for target in targets {
                        let Some((lines, target)) =
                            basic_semantic_lvalue(generator, target, None)
                        else {
                            return false;
                        };
                        prelude.extend(lines);
                        rendered_targets.push(target);
                    }
                    output.extend(prelude);
                    output.push(format!("{prefix}{}", rendered_targets.join(", ")));
                }
                Kind::Get { channel, position, .. } | Kind::Put { channel, position, .. } => {
                    let command = if matches!(&statement.kind, Kind::Get { .. }) {
                        "GET"
                    } else {
                        "PUT"
                    };
                    let Some((mut lines, channel)) =
                        generator.semantic_expression_with_prelude(channel, None)
                    else {
                        return false;
                    };
                    let position = match position {
                        Some(position) => {
                            let value = match &position.position {
                                Some(position) => {
                                    let Some((prelude, value)) =
                                        generator.semantic_expression_with_prelude(position, None)
                                    else {
                                        return false;
                                    };
                                    lines.extend(prelude);
                                    Some(value)
                                }
                                None => Some(String::new()),
                            };
                            let Some(value) = value else {
                                return false;
                            };
                            let record = match &position.record {
                                Some(record) => {
                                    let Some((prelude, record)) =
                                        generator.semantic_expression_with_prelude(record, None)
                                    else {
                                        return false;
                                    };
                                    lines.extend(prelude);
                                    Some(record)
                                }
                                None => Some(String::new()),
                            };
                            let Some(record) = record else {
                                return false;
                            };
                            if record.is_empty() {
                                format!(", {value}")
                            } else {
                                format!(", {value}, {record}")
                            }
                        }
                        None => String::new(),
                    };
                    output.extend(lines);
                    if let Kind::Get {
                        require_existing: Some(length),
                        ..
                    } = &statement.kind
                    {
                        let record = position.trim_start_matches(", ");
                        output.push(format!(
                            "IF LOF(#{channel}) < ({record}) * {length} THEN ERROR 63"
                        ));
                    }
                    output.push(format!("{command} #{channel}{position}"));
                }
                Kind::Lset { target, value } | Kind::Rset { target, value } => {
                    let command = if matches!(&statement.kind, Kind::Lset { .. }) {
                        "LSET"
                    } else {
                        "RSET"
                    };
                    let rendered = if generator.semantic_expression_contains_callable_call(value) {
                        generator.semantic_expression_with_prelude(value, None)
                    } else {
                        generator
                            .semantic_const_expression(value, None)
                            .map(|value| (Vec::new(), value))
                    };
                    let Some((lines, value)) = rendered else {
                        return false;
                    };
                    let target = generator.ident(&BasicIdent::parse(&target.name), None);
                    output.extend(lines);
                    output.push(format!("{command} {target} = {value}"));
                }
                Kind::Field { channel, bindings, .. } => {
                    let Some(channel) = generator.semantic_const_expression(channel, None) else {
                        return false;
                    };
                    let Some(bindings) = bindings
                        .iter()
                        .map(|binding| {
                                    generator
                                        .semantic_const_expression(&binding.length, None)
                                        .map(|length| {
                                            let mut ident = BasicIdent::parse(&binding.name);
                                            ident.suffix = binding
                                                .type_suffix
                                                .as_deref()
                                                .and_then(|suffix| suffix.chars().next())
                                                .and_then(TypeSuffix::from_char);
                                            format!(
                                                "{length} AS {}",
                                                generator.ident(&ident, None)
                                            )
                                        })
                        })
                        .collect::<Option<Vec<_>>>()
                    else {
                        return false;
                    };
                    output.push(format!("FIELD #{channel}, {}", bindings.join(", ")));
                }
                Kind::OptionBase(base) => {
                    let Some(base) = generator.semantic_const_expression(base, None) else {
                        return false;
                    };
                    output.push(format!("OPTION BASE {base}"));
                }
                Kind::Erase(names) => output.push(format!(
                    "ERASE {}",
                    names
                        .iter()
                        .map(|name| generator.ident(&BasicIdent::parse(&name.name), None))
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
                Kind::Randomize(seed) => match seed {
                    crate::semantic_ir::RandomizeSeed::Default => {
                        output.push("RANDOMIZE".to_string())
                    }
                    crate::semantic_ir::RandomizeSeed::Value(value) => {
                        let Some((lines, value)) =
                            generator.semantic_expression_with_prelude(value, None)
                        else {
                            return false;
                        };
                        output.extend(lines);
                        output.push(format!("RANDOMIZE {value}"));
                    }
                },
                Kind::Swap { left, right } => {
                    let (Some((mut lines, left)), Some((right_lines, right))) = (
                        basic_semantic_lvalue(generator, left, None),
                        basic_semantic_lvalue(generator, right, None),
                    ) else {
                        return false;
                    };
                    lines.extend(right_lines);
                    output.extend(lines);
                    output.push(format!("SWAP {left}, {right}"));
                }
                Kind::Poke { address, value } => {
                    let Some((mut lines, address)) =
                        generator.semantic_expression_with_prelude(address, None)
                    else {
                        return false;
                    };
                    let Some((value_lines, value)) =
                        generator.semantic_expression_with_prelude(value, None)
                    else {
                        return false;
                    };
                    lines.extend(value_lines);
                    output.extend(lines);
                    output.push(format!("POKE {address}, {value}"));
                }
                Kind::Out { port, value } => {
                    let Some((mut lines, port)) =
                        generator.semantic_expression_with_prelude(port, None)
                    else {
                        return false;
                    };
                    let Some((value_lines, value)) =
                        generator.semantic_expression_with_prelude(value, None)
                    else {
                        return false;
                    };
                    lines.extend(value_lines);
                    output.extend(lines);
                    output.push(format!("OUT {port}, {value}"));
                }
                Kind::Width { channel, value } => {
                    let mut lines = Vec::new();
                    let channel = if let Some(channel) = channel {
                        let Some((channel_lines, channel)) =
                            generator.semantic_expression_with_prelude(channel, None)
                        else {
                            return false;
                        };
                        lines.extend(channel_lines);
                        Some(channel)
                    } else {
                        None
                    };
                    let Some((value_lines, value)) =
                        generator.semantic_expression_with_prelude(value, None)
                    else {
                        return false;
                    };
                    lines.extend(value_lines);
                    lines.push(match channel {
                        Some(channel) => format!("WIDTH #{channel}, {value}"),
                        None => format!("WIDTH {value}"),
                    });
                    output.extend(lines);
                }
                Kind::Locate { row, column } => {
                    let Some((mut row_lines, row)) =
                        generator.semantic_expression_with_prelude(row, None)
                    else {
                        return false;
                    };
                    let Some((column_lines, column)) =
                        generator.semantic_expression_with_prelude(column, None)
                    else {
                        return false;
                    };
                    row_lines.extend(column_lines);
                    output.extend(row_lines);
                    output.push(format!("LOCATE {row}, {column}"));
                }
                Kind::Color {
                    foreground,
                    background,
                } => {
                    let Some((mut lines, foreground)) =
                        generator.semantic_expression_with_prelude(foreground, None)
                    else {
                        return false;
                    };
                    match background {
                        Some(background) => {
                            let Some((background_lines, background)) =
                                generator.semantic_expression_with_prelude(background, None)
                            else {
                                return false;
                            };
                            lines.extend(background_lines);
                            output.extend(lines);
                            output.push(format!("COLOR {foreground}, {background}"));
                        }
                        None => {
                            output.extend(lines);
                            output.push(format!("COLOR {foreground}"));
                        }
                    }
                }
                Kind::Error(code) => {
                    let Some((lines, code)) =
                        generator.semantic_expression_with_prelude(code, None)
                    else {
                        return false;
                    };
                    output.extend(lines);
                    output.push(format!("ERROR {code}"));
                }
                Kind::Throw(value) => match value {
                    crate::semantic_ir::ThrowValue::Bare => output.push("ERROR ERR".to_string()),
                    crate::semantic_ir::ThrowValue::Value(value) => {
                        let Some((lines, value)) =
                            generator.semantic_expression_with_prelude(value, None)
                        else {
                            return false;
                        };
                        output.extend(lines);
                        output.push(format!("ERROR {value}"));
                    }
                },
                Kind::OnBranch {
                    selector,
                    branch,
                    targets,
                } => {
                    let Some((lines, selector)) =
                        generator.semantic_expression_with_prelude(selector, None)
                    else {
                        return false;
                    };
                    let command = match branch {
                        crate::semantic_ir::BranchKind::Goto => "GOTO",
                        crate::semantic_ir::BranchKind::Gosub => "GOSUB",
                    };
                    let targets = targets
                        .iter()
                        .map(|target| render_target(generator, &target.name))
                        .collect::<Vec<_>>();
                    output.extend(lines);
                    output.push(format!("ON {selector} {command} {}", targets.join(", ")));
                }
                Kind::Try {
                    body,
                    catch,
                    finally_body,
                } => {
                    if !allow_structured_try {
                        return false;
                    }
                    let catch_filters = match catch.as_ref() {
                        Some(catch) => {
                            let Some(filters) = catch
                                .filters
                                .iter()
                                .map(|filter| generator.semantic_const_expression(filter, None))
                                .collect::<Option<Vec<_>>>()
                            else {
                                return false;
                            };
                            Some(filters)
                        }
                        None => None,
                    };
                    let id = generator.next_label;
                    generator.next_label += 1;
                    let catch_label = format!("TRY_{id:04}_CATCH");
                    let catch_run_label = format!("TRY_{id:04}_CATCH_RUN");
                    let rethrow_label = format!("TRY_{id:04}_RETHROW");
                    let finally_label = format!("TRY_{id:04}_FINALLY");
                    let end_label = format!("TRY_{id:04}_END");
                    let pending_name = format!("BCCTRY{id:04}PENDING%");
                    let outer_handler = generator.try_handler_stack.last().cloned();
                    let restore_outer = |output: &mut Vec<String>| match &outer_handler {
                        Some(label) => output.push(format!("ON ERROR GOTO {label}")),
                        None => output.push("ON ERROR GOTO 0".to_string()),
                    };

                    output.push(format!("ON ERROR GOTO {catch_label}"));
                    output.push(format!("{pending_name} = 0"));
                    generator.try_handler_stack.push(catch_label.clone());
                    if !visit(generator, module, body, output, allow_structured_try) {
                        generator.try_handler_stack.pop();
                        return false;
                    }
                    generator.try_handler_stack.pop();
                    restore_outer(output);
                    output.push(format!("GOTO {finally_label}"));

                    output.push(format!("{catch_label}:"));
                    output.push(format!("{pending_name} = ERR"));
                    if let Some(catch) = catch {
                        let filters = catch_filters.as_ref().expect("filters were rendered");
                        if !filters.is_empty() {
                            let matched_label = format!("TRY_{id:04}_MATCHED");
                            let condition = filters
                                .iter()
                                .map(|filter| format!("(ERR = {filter})"))
                                .collect::<Vec<_>>()
                                .join(" OR ");
                            output.push(format!("IF {condition} THEN GOTO {matched_label}"));
                            output.push(format!("RESUME {finally_label}"));
                            output.push(format!("{matched_label}:"));
                        }
                        let mut error_ident = BasicIdent::parse(&catch.error);
                        error_ident.suffix = catch
                            .error_type
                            .suffix()
                            .and_then(TypeSuffix::from_char);
                        let mut line_ident = BasicIdent::parse(&catch.line);
                        line_ident.suffix =
                            catch.line_type.suffix().and_then(TypeSuffix::from_char);
                        let error_name = generator.ident(&error_ident, None);
                        let line_name = generator.ident(&line_ident, None);
                        output.push(format!("{error_name} = ERR"));
                        output.push(format!("{line_name} = ERL"));
                        if let Some(source) = &catch.source {
                            let mut source_ident = BasicIdent::parse(source);
                            source_ident.suffix = catch
                                .source_type
                                .and_then(crate::semantic_ir::SemanticValueType::suffix)
                                .and_then(TypeSuffix::from_char);
                            let source_name = generator.ident(&source_ident, None);
                            output.push("GOSUB BCC_RESOLVE_SOURCE_FILE".to_string());
                            output.push(format!("{source_name} = BCCSOURCEFILE$"));
                        }
                        output.push(format!("RESUME {catch_run_label}"));
                    } else {
                        output.push(format!("RESUME {finally_label}"));
                    }

                    if let Some(catch) = catch {
                        output.push(format!("{catch_run_label}:"));
                        output.push(format!("ON ERROR GOTO {rethrow_label}"));
                        generator.try_handler_stack.push(rethrow_label.clone());
                        if !visit(generator, module, &catch.body, output, allow_structured_try) {
                            generator.try_handler_stack.pop();
                            return false;
                        }
                        generator.try_handler_stack.pop();
                        output.push(format!("{pending_name} = 0"));
                        restore_outer(output);
                        output.push(format!("GOTO {finally_label}"));
                        output.push(format!("{rethrow_label}:"));
                        output.push(format!("{pending_name} = ERR"));
                        output.push(format!("RESUME {finally_label}"));
                    }

                    output.push(format!("{finally_label}:"));
                    restore_outer(output);
                    if !visit(
                        generator,
                        module,
                        finally_body,
                        output,
                        allow_structured_try,
                    ) {
                        return false;
                    }
                    output.push(format!("IF {pending_name} <> 0 THEN ERROR {pending_name}"));
                    output.push(format!("{end_label}:"));
                    output.push("REM END TRY".to_string());
                }
                Kind::Exit => output.push(match generator.loop_exit_stack.last() {
                    Some(LoopExit::NativeFor) => "EXIT FOR".to_string(),
                    Some(LoopExit::Goto(label)) => format!("GOTO {label}"),
                    None => "' warning: EXIT outside of a loop".to_string(),
                }),
                Kind::Continue => output.push(match generator.loop_continue_stack.last() {
                    Some(label) => format!("GOTO {label}"),
                    None => "' warning: CONTINUE outside of a loop".to_string(),
                }),
                Kind::Stop => output.push("STOP".to_string()),
                Kind::Cls => output.push("CLS".to_string()),
                Kind::Beep => output.push("BEEP".to_string()),
                Kind::System => output.push("SYSTEM".to_string()),
                Kind::Clear => output.push("CLEAR".to_string()),
                Kind::End => output.push("END".to_string()),
                // At module scope a bare RETURN is a GOSUB return. Preserve
                // the typed semantic node instead of reconstructing it from
                // the compatibility AST.
                Kind::Return(crate::semantic_ir::ReturnValue::Default) => {
                    output.push("RETURN".to_string())
                }
                Kind::Global { .. } => {}
                Kind::Const { name, value, .. } => {
                    let Some(value) = generator.semantic_const_expression(value, None) else {
                        return false;
                    };
                    let name = generator.ident(&BasicIdent::parse(&name.name), None);
                    output.push(format!("{name} = {value}"));
                }
                Kind::Comment { block, text } => {
                    output.extend(basic_semantic_comment(*block, text));
                }
                Kind::Label(label) => output.push(format!("{}:", user_label_token(&label.name))),
                Kind::Goto(label) => {
                    output.push(format!("GOTO {}", render_target(generator, &label.name)))
                }
                Kind::Gosub(label) => {
                    output.push(format!("GOSUB {}", render_target(generator, &label.name)))
                }
                Kind::Restore(restore_target) => output.push(match restore_target {
                    Some(target) => format!("RESTORE {}", render_target(generator, &target.name)),
                    None => "RESTORE".to_string(),
                }),
                Kind::Resume(target) => output.push(match target {
                    None => "RESUME".to_string(),
                    Some(ResumeTarget::Next) => "RESUME NEXT".to_string(),
                    Some(ResumeTarget::Label(target)) => {
                        format!("RESUME {}", render_target(generator, &target.name))
                    }
                }),
                Kind::OnErrorGoto(target) => output.push(match target {
                    crate::semantic_ir::ErrorHandlerTarget::Disable => {
                        "ON ERROR GOTO 0".to_string()
                    }
                    crate::semantic_ir::ErrorHandlerTarget::Label(target) => {
                        format!("ON ERROR GOTO {}", render_target(generator, &target.name))
                    }
                }),
                Kind::Data(values) => {
                    let Some(values) = values
                        .iter()
                        .map(|value| generator.semantic_const_expression(value, None))
                        .collect::<Option<Vec<_>>>()
                    else {
                        return false;
                    };
                    output.push(format!("DATA {}", values.join(", ")));
                }
                Kind::Read(targets) => {
                    let mut target_text = Vec::with_capacity(targets.len());
                    for target in targets {
                        let Some((prelude, rendered)) =
                            basic_semantic_lvalue(generator, target, None)
                        else {
                            return false;
                        };
                        output.extend(prelude);
                        target_text.push(rendered);
                    }
                    output.push(format!("READ {}", target_text.join(", ")));
                }
                _ => return false,
            }
        }
        true
    }

    let mut output = Vec::new();
    let initial_label = generator.next_label;
    let initial_loop_exit_depth = generator.loop_exit_stack.len();
    let initial_loop_continue_depth = generator.loop_continue_stack.len();
    let initial_taken_names = generator.taken_names.borrow().clone();
    let initial_top_level_array_bounds = generator.top_level_array_bounds.clone();
    let initial_diagnostic_count = generator.diagnostics.len();
    let initial_marker_file = generator.current_marker_file.clone();
    // The catch `source$` lookup maps `ERL` back to a file through markers
    // between top-level statements, so a whole-module stream is visited one
    // statement at a time in that case.
    let mark_sources = generator.needs_source_lookup
        && std::ptr::eq(statements, module.statements.as_slice())
        && module.statement_sources.len() == statements.len();
    let visited = if mark_sources {
        statements
            .iter()
            .zip(&module.statement_sources)
            .all(|(statement, source_index)| {
                if let Some(source) = module.sources.get(*source_index) {
                    if generator.current_marker_file.as_deref() != Some(source.filename.as_str()) {
                        generator.current_marker_file = Some(source.filename.clone());
                        output.push(source_file_marker(
                            &crate::diagnostics::display_source_filename(&source.filename),
                        ));
                    }
                }
                visit(
                    generator,
                    module,
                    std::slice::from_ref(statement),
                    &mut output,
                    allow_structured_try,
                )
            })
    } else {
        visit(
            generator,
            module,
            statements,
            &mut output,
            allow_structured_try,
        )
    };
    if !statements.is_empty() && visited {
        Some(output)
    } else {
        generator.current_marker_file = initial_marker_file;
        // A declined semantic stream must not perturb labels generated by the
        // compatibility emitter that will handle it instead, or leave loop
        // context behind for later statements/callables.
        generator.next_label = initial_label;
        generator.loop_exit_stack.truncate(initial_loop_exit_depth);
        generator
            .loop_continue_stack
            .truncate(initial_loop_continue_depth);
        *generator.taken_names.borrow_mut() = initial_taken_names;
        generator.top_level_array_bounds = initial_top_level_array_bounds;
        generator.diagnostics.truncate(initial_diagnostic_count);
        None
    }
}

/// Dispatch aligned top-level semantic statements independently. A failed
/// semantic node uses the matching legacy statement; inability to prove a
/// one-to-one source mapping declines the dispatcher before any output is
/// written, preserving the existing whole-stream compatibility path.
fn basic_semantic_statements_by_source(
    generator: &mut CodeGenerator,
    module: &crate::semantic_ir::SemanticModule,
    ast_statements: &[Stmt],
) -> Option<()> {
    if module.statement_sources.len() != module.statements.len() {
        return None;
    }

    let mut aligned = Vec::new();
    let mut used_ast = HashSet::new();
    for (root, source_index) in module.statements.iter().zip(&module.statement_sources) {
        let source = module.sources.get(*source_index)?;
        let children: Vec<&crate::semantic_ir::SemanticStatement> =
            if let crate::semantic_ir::SemanticStatementKind::Line(nodes) = &root.kind {
                nodes.iter().collect()
            } else {
                vec![root]
            };
        // Each semantic node maps to the one AST statement at its exact
        // source position, so a line whose AST form has extra siblings (for
        // example a label followed by a comment) still aligns.
        for node in children {
            let position = source_position(source, node.span)?;
            let matches = ast_statements
                .iter()
                .enumerate()
                .filter(|(_, stmt)| {
                    stmt.pos.filename == source.filename
                        && stmt.pos.line == position.line
                        && stmt.pos.column == position.column
                        && !matches!(stmt.kind, Statement::BlankLine)
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            // The legacy parser positions a trailing comment's `Raw` node at
            // its statement rather than at the apostrophe, so a semantic
            // comment falls back to the line's next unclaimed `Raw` comment.
            let matches = if matches.is_empty()
                && matches!(
                    node.kind,
                    crate::semantic_ir::SemanticStatementKind::Comment { block: false, .. }
                ) {
                ast_statements
                    .iter()
                    .enumerate()
                    .filter(|(index, stmt)| {
                        !used_ast.contains(index)
                            && stmt.pos.filename == source.filename
                            && stmt.pos.line == position.line
                            && matches!(&stmt.kind, Statement::Raw(text) if text.starts_with('\''))
                    })
                    .map(|(index, _)| index)
                    .take(1)
                    .collect::<Vec<_>>()
            } else {
                matches
            };
            // A trailing comment on a statement line has no legacy AST
            // counterpart (the legacy parser discards it), so it emits nothing.
            if matches.is_empty()
                && matches!(
                    node.kind,
                    crate::semantic_ir::SemanticStatementKind::Comment { block: false, .. }
                )
            {
                continue;
            }
            let [ast_index] = matches.as_slice() else {
                return None;
            };
            if !used_ast.insert(*ast_index)
                || aligned
                    .last()
                    .is_some_and(|(previous, _, _)| previous >= ast_index)
            {
                return None;
            }
            aligned.push((*ast_index, node, *source_index));
        }
    }
    if !ast_statements.is_empty() {
        generator.blank();
    }
    let mut aligned_iter = aligned.into_iter().peekable();
    for (ast_index, ast_statement) in ast_statements.iter().enumerate() {
        if aligned_iter
            .peek()
            .is_some_and(|(semantic_ast_index, _, _)| *semantic_ast_index == ast_index)
        {
            let (_, semantic, source_index) =
                aligned_iter.next().expect("peeked aligned statement");
            let source = &module.sources[source_index];
            if generator.needs_source_lookup
                && generator.current_marker_file.as_deref() != Some(source.filename.as_str())
            {
                generator.current_marker_file = Some(source.filename.clone());
                generator.line(&source_file_marker(
                    &crate::diagnostics::display_source_filename(&source.filename),
                ));
            }
            if let Some(lines) =
                basic_semantic_intrinsics(generator, module, std::slice::from_ref(semantic), true)
            {
                for line in lines {
                    generator.line(&line);
                }
            } else {
                generator.statement(ast_statement, None);
            }
        } else {
            // Legacy-only synthetic nodes such as blank-line markers retain
            // their original codegen behavior.
            generator.statement(ast_statement, None);
        }
    }
    Some(())
}

/// Record DSL lowering currently synthesizes several operations (GET/PUT,
/// CLOSE, and field assignments) that are not yet represented in semantic IR.
/// Keep the whole lowered AST stream authoritative when one of those nodes
/// occurs so a typed FileDeclaration cannot make the dispatcher silently
/// omit its generated record operations.
fn has_untyped_lowered_record_operations(
    statements: &[Stmt],
    files: &[crate::semantic_ir::LoweredRecordFile],
) -> bool {
    let is_record_channel = |channel: &Expr| {
        let Expr::Integer(channel) = channel else {
            return false;
        };
        files.iter().any(|file| file.owner.is_none() && file.channel == *channel)
    };
    let is_record_buffer = |name: &BasicIdent| {
        files.iter().any(|file| {
            file.owner.is_none()
                && file
                    .fields
                    .iter()
                    .any(|field| field.buffer_name.eq_ignore_ascii_case(&name.as_basic()))
        })
    };
    statements.iter().any(|statement| match &statement.kind {
        Statement::Get { channel, .. }
        | Statement::Put { channel, .. }
        | Statement::Close { channel } => is_record_channel(channel),
        Statement::Lset { var, .. } | Statement::Rset { var, .. } => is_record_buffer(var),
        Statement::If {
            then_body,
            else_body,
            ..
        } => {
            has_untyped_lowered_record_operations(then_body, files)
                || has_untyped_lowered_record_operations(else_body, files)
        }
        Statement::For { body, .. }
        | Statement::While { body, .. }
        | Statement::Do { body, .. } => has_untyped_lowered_record_operations(body, files),
        Statement::TryCatch {
            try_body,
            catch,
            finally_body,
        } => {
            has_untyped_lowered_record_operations(try_body, files)
                || catch.as_ref().is_some_and(|catch| {
                    has_untyped_lowered_record_operations(&catch.body, files)
                })
                || has_untyped_lowered_record_operations(finally_body, files)
        }
        Statement::SelectCase {
            cases, else_body, ..
        } => {
            cases.iter().any(|case| {
                has_untyped_lowered_record_operations(&case.body, files)
            }) || has_untyped_lowered_record_operations(else_body, files)
        }
        _ => false,
    })
}

fn basic_semantic_callable_statements_by_source<'a>(
    module: &'a crate::semantic_ir::SemanticModule,
    function: &FunctionDef,
) -> Option<Vec<Option<&'a crate::semantic_ir::SemanticStatement>>> {
    use crate::semantic_ir::{CallableKind, SemanticStatementKind as Kind};
    let callable = module.callables.iter().find(|callable| {
        crate::semantic_ir::callable_name_matches_function(&callable.name, function)
            && callable.receiver.is_some() == function.receiver.is_some()
            && callable
                .receiver
                .as_deref()
                .map_or(true, |receiver| match function.receiver {
                    Some(TypeSuffix::Integer) => receiver.eq_ignore_ascii_case("integer"),
                    Some(TypeSuffix::Long) => receiver.eq_ignore_ascii_case("long"),
                    Some(TypeSuffix::Single) => receiver.eq_ignore_ascii_case("single"),
                    Some(TypeSuffix::Double) => receiver.eq_ignore_ascii_case("double"),
                    Some(TypeSuffix::String) => receiver.eq_ignore_ascii_case("string"),
                    None => false,
                })
            && match (
                function.receiver.is_some(),
                function.is_procedure,
                callable.kind,
            ) {
                (
                    true,
                    false,
                    CallableKind::Method | CallableKind::FluentMethod | CallableKind::InlineMethod,
                )
                | (false, true, CallableKind::Procedure)
                | (false, false, CallableKind::Function) => true,
                _ => false,
            }
    })?;
    let source = module.sources.get(callable.source_index)?;
    let mut aligned = vec![None; function.body.len()];
    let mut previous = None;
    for root in &callable.body {
        let children: Vec<_> = if let Kind::Line(children) = &root.kind {
            children.iter().collect()
        } else {
            vec![root]
        };
        for semantic in children {
            let position = source_position(source, semantic.span)?;
            let candidates = function
                .body
                .iter()
                .enumerate()
                .filter(|(_, statement)| {
                    statement.pos.filename == position.filename
                        && statement.pos.line == position.line
                        && statement.pos.column == position.column
                        && !matches!(statement.kind, Statement::BlankLine)
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            let [index] = candidates.as_slice() else {
                return None;
            };
            if previous.is_some_and(|previous| previous >= *index) || aligned[*index].is_some() {
                return None;
            }
            aligned[*index] = Some(semantic);
            previous = Some(*index);
        }
    }
    Some(aligned)
}

fn basic_semantic_callable_try(
    generator: &mut CodeGenerator,
    semantic: &crate::semantic_ir::SemanticStatement,
    function: &FunctionInfo,
) -> Option<Vec<String>> {
    fn render_body(
        generator: &mut CodeGenerator,
        statements: &[crate::semantic_ir::SemanticStatement],
        function: &FunctionInfo,
    ) -> Option<Vec<String>> {
        statements
            .iter()
            .map(|statement| basic_semantic_callable_statement(generator, statement, function))
            .collect::<Option<Vec<_>>>()
            .map(|lines| lines.into_iter().flatten().collect())
    }

    fn inner(
        generator: &mut CodeGenerator,
        semantic: &crate::semantic_ir::SemanticStatement,
        function: &FunctionInfo,
    ) -> Option<Vec<String>> {
        let crate::semantic_ir::SemanticStatementKind::Try {
            body,
            catch,
            finally_body,
        } = &semantic.kind
        else {
            return None;
        };
        let catch_filters = match catch.as_ref() {
            Some(catch) => Some(
                catch
                    .filters
                    .iter()
                    .map(|filter| generator.semantic_const_expression(filter, Some(function)))
                    .collect::<Option<Vec<_>>>()?,
            ),
            None => None,
        };

        let id = generator.next_label;
        generator.next_label += 1;
        let catch_label = format!("TRY_{id:04}_CATCH");
        let catch_run_label = format!("TRY_{id:04}_CATCH_RUN");
        let rethrow_label = format!("TRY_{id:04}_RETHROW");
        let finally_label = format!("TRY_{id:04}_FINALLY");
        let end_label = format!("TRY_{id:04}_END");
        let pending_name = format!("BCCTRY{id:04}PENDING%");
        let outer_handler = generator.try_handler_stack.last().cloned();
        let restore_outer = |lines: &mut Vec<String>| match &outer_handler {
            Some(label) => lines.push(format!("ON ERROR GOTO {label}")),
            None => lines.push("ON ERROR GOTO 0".to_string()),
        };
        let nested = |lines: Vec<String>| {
            lines
                .into_iter()
                .map(|line| format!("    {line}"))
                .collect::<Vec<_>>()
        };

        let mut lines = vec![
            format!("ON ERROR GOTO {catch_label}"),
            format!("{pending_name} = 0"),
        ];
        generator.try_handler_stack.push(catch_label.clone());
        let try_lines = render_body(generator, body, function);
        generator.try_handler_stack.pop();
        lines.extend(nested(try_lines?));
        restore_outer(&mut lines);
        lines.push(format!("GOTO {finally_label}"));
        lines.push(format!("{catch_label}:"));
        lines.push(format!("{pending_name} = ERR"));

        if let Some(catch) = catch {
            let filters = catch_filters.as_ref().expect("catch filters were rendered");
            let mut catch_setup = Vec::new();
            if !filters.is_empty() {
                let matched_label = format!("TRY_{id:04}_MATCHED");
                let condition = filters
                    .iter()
                    .map(|filter| format!("(ERR = {filter})"))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                catch_setup.push(format!("IF {condition} THEN GOTO {matched_label}"));
                catch_setup.push(format!("RESUME {finally_label}"));
                catch_setup.push(format!("{matched_label}:"));
            }
            let mut error_ident = BasicIdent::parse(&catch.error);
            error_ident.suffix = catch
                .error_type
                .suffix()
                .and_then(TypeSuffix::from_char);
            let mut line_ident = BasicIdent::parse(&catch.line);
            line_ident.suffix = catch.line_type.suffix().and_then(TypeSuffix::from_char);
            let error_name = generator.ident(&error_ident, Some(function));
            let line_name = generator.ident(&line_ident, Some(function));
            catch_setup.push(format!("{error_name} = ERR"));
            catch_setup.push(format!("{line_name} = ERL"));
            if let Some(source) = &catch.source {
                let mut source_ident = BasicIdent::parse(source);
                source_ident.suffix = catch
                    .source_type
                    .and_then(crate::semantic_ir::SemanticValueType::suffix)
                    .and_then(TypeSuffix::from_char);
                let source_name = generator.ident(&source_ident, Some(function));
                catch_setup.push("GOSUB BCC_RESOLVE_SOURCE_FILE".to_string());
                catch_setup.push(format!("{source_name} = BCCSOURCEFILE$"));
            }
            catch_setup.push(format!("RESUME {catch_run_label}"));
            lines.extend(nested(catch_setup));

            lines.push(format!("{catch_run_label}:"));
            lines.push(format!("ON ERROR GOTO {rethrow_label}"));
            generator.try_handler_stack.push(rethrow_label.clone());
            let catch_lines = render_body(generator, &catch.body, function);
            generator.try_handler_stack.pop();
            lines.extend(nested(catch_lines?));
            lines.push(format!("    {pending_name} = 0"));
            let mut catch_restore = Vec::new();
            restore_outer(&mut catch_restore);
            lines.extend(nested(catch_restore));
            lines.push(format!("    GOTO {finally_label}"));
            lines.push(format!("{rethrow_label}:"));
            lines.push(format!("    {pending_name} = ERR"));
            lines.push(format!("    RESUME {finally_label}"));
        } else {
            lines.push(format!("    RESUME {finally_label}"));
        }

        lines.push(format!("{finally_label}:"));
        restore_outer(&mut lines);
        lines.extend(nested(render_body(generator, finally_body, function)?));
        lines.push(format!(
            "    IF {pending_name} <> 0 THEN ERROR {pending_name}"
        ));
        lines.push(format!("{end_label}:"));
        lines.push("REM END TRY".to_string());
        Some(lines)
    }

    let next_label = generator.next_label;
    let loop_exit_stack = generator.loop_exit_stack.clone();
    let loop_continue_stack = generator.loop_continue_stack.clone();
    let try_handler_stack = generator.try_handler_stack.clone();
    let taken_names = generator.taken_names.borrow().clone();
    let local_var_map = function.local_var_map.borrow().clone();
    let local_array_bounds = function.local_array_bounds.borrow().clone();
    let result = inner(generator, semantic, function);
    if result.is_none() {
        generator.next_label = next_label;
        generator.loop_exit_stack = loop_exit_stack;
        generator.loop_continue_stack = loop_continue_stack;
        generator.try_handler_stack = try_handler_stack;
        *generator.taken_names.borrow_mut() = taken_names;
        *function.local_var_map.borrow_mut() = local_var_map;
        *function.local_array_bounds.borrow_mut() = local_array_bounds;
    }
    result
}

fn basic_semantic_callable_leaf(
    generator: &mut CodeGenerator,
    semantic: &crate::semantic_ir::SemanticStatement,
    function: &FunctionInfo,
) -> Option<Vec<String>> {
    use crate::semantic_ir::{PrintDestination, ReturnValue, SemanticStatementKind as Kind};
    let render_target = |target: &crate::semantic_ir::NamedReference| {
        generator.semantic_label_target_text(&target.name)
    };
    match &semantic.kind {
        Kind::Try { .. } => basic_semantic_callable_try(generator, semantic, function),
        Kind::Dim(_) => basic_semantic_dim(
            generator,
            &function.semantic_dim_declarations,
            semantic,
            Some(function),
        ),
        Kind::MidAssign {
            target,
            start,
            length,
            value,
        } => basic_semantic_mid_assign(
            generator,
            target,
            start,
            length.as_ref(),
            value,
            Some(function),
        ),
        Kind::Assignment {
            target,
            operator,
            value,
        } => basic_semantic_assignment(generator, target, *operator, value, Some(function)),
        Kind::Expression(expression) => {
            basic_semantic_expression_statement(generator, expression, Some(function))
        }
        Kind::Const { name, value, .. } => {
            let value = generator.semantic_const_expression(value, Some(function))?;
            let name = generator.ident(&BasicIdent::parse(&name.name), Some(function));
            Some(vec![format!("{name} = {value}")])
        }
        Kind::Comment { block, text } => Some(basic_semantic_comment(*block, text)),
        Kind::Print {
            destination,
            tokens,
        } => {
            let (lines, body) = basic_semantic_print_body(generator, tokens, Some(function))?;
            let mut destination_lines = Vec::new();
            let line = match destination {
                PrintDestination::Standard { .. } => format!("PRINT{body}"),
                PrintDestination::Using { format, .. } => {
                    let (prelude, format) =
                        generator.semantic_expression_with_prelude(format, Some(function))?;
                    destination_lines.extend(prelude);
                    if body.is_empty() {
                        format!("PRINT USING {format}")
                    } else {
                        format!("PRINT USING {format};{body}")
                    }
                }
                PrintDestination::Channel { channel, using, .. } => {
                    let (prelude, channel) =
                        generator.semantic_expression_with_prelude(channel, Some(function))?;
                    destination_lines.extend(prelude);
                    match using {
                        Some(format) => {
                            let (prelude, format) = generator
                                .semantic_expression_with_prelude(format, Some(function))?;
                            destination_lines.extend(prelude);
                            if body.is_empty() {
                                format!("PRINT #{channel}, USING {format}")
                            } else {
                                format!("PRINT #{channel}, USING {format};{body}")
                            }
                        }
                        None if body.is_empty() => format!("PRINT #{channel}"),
                        None => format!("PRINT #{channel}, {}", body.trim_start()),
                    }
                }
            };
            destination_lines.extend(lines);
            let mut lines = destination_lines;
            lines.push(line);
            Some(lines)
        }
        Kind::Lprint { using, tokens } => {
            let (mut using_lines, using) = match using {
                Some(format) => {
                    let (lines, format) =
                        generator.semantic_expression_with_prelude(format, Some(function))?;
                    (lines, Some(format))
                }
                None => (Vec::new(), None),
            };
            let (mut lines, body) = basic_semantic_print_body(generator, tokens, Some(function))?;
            using_lines.append(&mut lines);
            let mut lines = using_lines;
            lines.push(match (using, body.is_empty()) {
                (Some(format), true) => format!("LPRINT USING {format}"),
                (Some(format), false) => format!("LPRINT USING {format};{}", body),
                (None, _) => format!("LPRINT{body}"),
            });
            Some(lines)
        }
        Kind::Input { source, targets } => {
            let mut lines = Vec::new();
            let prefix = match source {
                crate::semantic_ir::InputSource::Console(prompt) => {
                    let prompt = prompt
                        .as_ref()
                        .map(basic_semantic_input_prompt)
                        .unwrap_or_default();
                    format!("INPUT {prompt}")
                }
                crate::semantic_ir::InputSource::Channel(channel) => {
                    let (prelude, channel) =
                        generator.semantic_expression_with_prelude(channel, Some(function))?;
                    lines.extend(prelude);
                    format!("INPUT #{channel}, ")
                }
            };
            let mut rendered_targets = Vec::with_capacity(targets.len());
            for target in targets {
                let (prelude, target) =
                    basic_semantic_lvalue(generator, target, Some(function))?;
                lines.extend(prelude);
                rendered_targets.push(target);
            }
            lines.push(format!("{prefix}{}", rendered_targets.join(", ")));
            Some(lines)
        }
        Kind::LineInput { channel, target } => {
            let (mut lines, channel) =
                generator.semantic_expression_with_prelude(channel, Some(function))?;
            let (target_lines, target) =
                basic_semantic_lvalue(generator, target, Some(function))?;
            lines.extend(target_lines);
            lines.push(format!("LINE INPUT #{channel}, {target}"));
            Some(lines)
        }
        Kind::Write { channel, values } => {
            let (mut lines, channel) =
                generator.semantic_expression_with_prelude(channel, Some(function))?;
            let (value_lines, values) = match values {
                crate::semantic_ir::WriteValues::Omitted => (Vec::new(), Vec::new()),
                crate::semantic_ir::WriteValues::Values(values) => {
                    basic_semantic_io_values(generator, values, Some(function))?
                }
            };
            lines.extend(value_lines);
            let suffix = if values.is_empty() {
                String::new()
            } else {
                format!(", {}", values.join(", "))
            };
            lines.push(format!("WRITE #{channel}{suffix}"));
            Some(lines)
        }
        Kind::Close(channel) => {
            let (mut lines, channel) =
                generator.semantic_expression_with_prelude(channel, Some(function))?;
            lines.push(format!("CLOSE #{channel}"));
            Some(lines)
        }
        Kind::Open {
            path,
            mode,
            channel,
            length,
        } => {
            let (mut lines, path) =
                generator.semantic_expression_with_prelude(path, Some(function))?;
            let (channel_lines, channel) =
                generator.semantic_expression_with_prelude(channel, Some(function))?;
            lines.extend(channel_lines);
            let length = match length {
                Some(length) => {
                    let (length_lines, length) =
                        generator.semantic_expression_with_prelude(length, Some(function))?;
                    lines.extend(length_lines);
                    format!(" LEN = {length}")
                }
                None => String::new(),
            };
            let mode = match mode.kind {
                crate::semantic_ir::OpenModeKind::Input => "INPUT",
                crate::semantic_ir::OpenModeKind::Output => "OUTPUT",
                crate::semantic_ir::OpenModeKind::Append => "APPEND",
                crate::semantic_ir::OpenModeKind::Random => "RANDOM",
                crate::semantic_ir::OpenModeKind::Binary => "BINARY",
            };
            lines.push(format!("OPEN {path} FOR {mode} AS #{channel}{length}"));
            Some(lines)
        }
        Kind::Seek { channel, position } => {
            let (mut lines, channel) =
                generator.semantic_expression_with_prelude(channel, Some(function))?;
            let (position_lines, position) =
                generator.semantic_expression_with_prelude(position, Some(function))?;
            lines.extend(position_lines);
            lines.push(format!("SEEK #{channel}, {position}"));
            Some(lines)
        }
        Kind::Get { channel, position, .. } | Kind::Put { channel, position, .. } => {
            let command = if matches!(&semantic.kind, Kind::Get { .. }) {
                "GET"
            } else {
                "PUT"
            };
            let (mut lines, channel) =
                generator.semantic_expression_with_prelude(channel, Some(function))?;
            let position = match position {
                Some(position) => {
                    let value = match &position.position {
                        Some(value) => {
                            let (prelude, value) = generator
                                .semantic_expression_with_prelude(value, Some(function))?;
                            lines.extend(prelude);
                            value
                        }
                        None => String::new(),
                    };
                    let record = match &position.record {
                        Some(record) => {
                            let (prelude, record) = generator
                                .semantic_expression_with_prelude(record, Some(function))?;
                            lines.extend(prelude);
                            record
                        }
                        None => String::new(),
                    };
                    if record.is_empty() {
                        if value.is_empty() {
                            String::new()
                        } else {
                            format!(", {value}")
                        }
                    } else {
                        format!(", {value}, {record}")
                    }
                }
                None => String::new(),
            };
            if let Kind::Get {
                require_existing: Some(length),
                ..
            } = &semantic.kind
            {
                let record = position.trim_start_matches(", ");
                lines.push(format!(
                    "IF LOF(#{channel}) < ({record}) * {length} THEN ERROR 63"
                ));
            }
            lines.push(format!("{command} #{channel}{position}"));
            Some(lines)
        }
        Kind::Field { channel, bindings, .. } => {
            let channel = generator.semantic_const_expression(channel, Some(function))?;
            let bindings = bindings
                .iter()
                .map(|binding| {
                    generator
                    .semantic_const_expression(&binding.length, Some(function))
                    .map(|length| {
                        let mut ident = BasicIdent::parse(&binding.name);
                        ident.suffix = binding
                            .type_suffix
                            .as_deref()
                            .and_then(|suffix| suffix.chars().next())
                            .and_then(TypeSuffix::from_char);
                        format!(
                            "{length} AS {}",
                            generator.ident(&ident, Some(function))
                        )
                    })
                })
                .collect::<Option<Vec<_>>>()?;
            Some(vec![format!("FIELD #{channel}, {}", bindings.join(", "))])
        }
        Kind::Kill(path) => {
            let (mut lines, path) =
                generator.semantic_expression_with_prelude(path, Some(function))?;
            lines.push(format!("KILL {path}"));
            Some(lines)
        }
        Kind::Rename {
            source,
            destination,
        } => {
            let (mut lines, source) =
                generator.semantic_expression_with_prelude(source, Some(function))?;
            let (destination_lines, destination) =
                generator.semantic_expression_with_prelude(destination, Some(function))?;
            lines.extend(destination_lines);
            lines.push(format!("NAME {source} AS {destination}"));
            Some(lines)
        }
        Kind::Width { channel, value } => {
            let mut lines = Vec::new();
            let channel = if let Some(channel) = channel {
                let (channel_lines, channel) =
                    generator.semantic_expression_with_prelude(channel, Some(function))?;
                lines.extend(channel_lines);
                Some(channel)
            } else {
                None
            };
            let (value_lines, value) =
                generator.semantic_expression_with_prelude(value, Some(function))?;
            lines.extend(value_lines);
            lines.push(match channel {
                Some(channel) => format!("WIDTH #{channel}, {value}"),
                None => format!("WIDTH {value}"),
            });
            Some(lines)
        }
        Kind::Locate { row, column } => {
            let (mut lines, row) =
                generator.semantic_expression_with_prelude(row, Some(function))?;
            let (column_lines, column) =
                generator.semantic_expression_with_prelude(column, Some(function))?;
            lines.extend(column_lines);
            lines.push(format!("LOCATE {row}, {column}"));
            Some(lines)
        }
        Kind::Color {
            foreground,
            background,
        } => {
            let (mut lines, foreground) =
                generator.semantic_expression_with_prelude(foreground, Some(function))?;
            match background {
                Some(background) => {
                    let (background_lines, background) =
                        generator.semantic_expression_with_prelude(background, Some(function))?;
                    lines.extend(background_lines);
                    lines.push(format!("COLOR {foreground}, {background}"));
                    Some(lines)
                }
                None => {
                    lines.push(format!("COLOR {foreground}"));
                    Some(lines)
                }
            }
        }
        Kind::Swap { left, right } => {
            let (mut lines, left) = basic_semantic_lvalue(generator, left, Some(function))?;
            let (right_lines, right) = basic_semantic_lvalue(generator, right, Some(function))?;
            lines.extend(right_lines);
            lines.push(format!("SWAP {left}, {right}"));
            Some(lines)
        }
        Kind::Randomize(seed) => match seed {
            crate::semantic_ir::RandomizeSeed::Default => Some(vec!["RANDOMIZE".to_string()]),
            crate::semantic_ir::RandomizeSeed::Value(value) => {
                let (mut lines, value) =
                    generator.semantic_expression_with_prelude(value, Some(function))?;
                lines.push(format!("RANDOMIZE {value}"));
                Some(lines)
            }
        },
        Kind::Poke { address, value } => {
            let (mut lines, address) =
                generator.semantic_expression_with_prelude(address, Some(function))?;
            let (value_lines, value) =
                generator.semantic_expression_with_prelude(value, Some(function))?;
            lines.extend(value_lines);
            lines.push(format!("POKE {address}, {value}"));
            Some(lines)
        }
        Kind::Out { port, value } => {
            let (mut lines, port) =
                generator.semantic_expression_with_prelude(port, Some(function))?;
            let (value_lines, value) =
                generator.semantic_expression_with_prelude(value, Some(function))?;
            lines.extend(value_lines);
            lines.push(format!("OUT {port}, {value}"));
            Some(lines)
        }
        Kind::Lset { target, value } | Kind::Rset { target, value } => {
            let command = if matches!(&semantic.kind, Kind::Lset { .. }) {
                "LSET"
            } else {
                "RSET"
            };
            let target = generator.ident(&BasicIdent::parse(&target.name), Some(function));
            let (mut lines, value) = if generator.semantic_expression_contains_callable_call(value)
            {
                generator.semantic_expression_with_prelude(value, Some(function))?
            } else {
                (
                    Vec::new(),
                    generator.semantic_const_expression(value, Some(function))?,
                )
            };
            lines.push(format!("{command} {target} = {value}"));
            Some(lines)
        }
        Kind::Erase(names) => Some(vec![format!(
            "ERASE {}",
            names
                .iter()
                .map(|name| generator.ident(&BasicIdent::parse(&name.name), Some(function)))
                .collect::<Vec<_>>()
                .join(", ")
        )]),
        Kind::Label(label) => Some(vec![format!("{}:", user_label_token(&label.name))]),
        Kind::Goto(label) => Some(vec![format!("GOTO {}", render_target(label))]),
        Kind::Gosub(label) => Some(vec![format!("GOSUB {}", render_target(label))]),
        Kind::Restore(target) => Some(vec![match target {
            Some(target) => format!("RESTORE {}", render_target(target)),
            None => "RESTORE".to_string(),
        }]),
        Kind::Resume(target) => Some(vec![match target {
            None => "RESUME".to_string(),
            Some(crate::semantic_ir::ResumeTarget::Next) => "RESUME NEXT".to_string(),
            Some(crate::semantic_ir::ResumeTarget::Label(target)) => {
                format!("RESUME {}", render_target(target))
            }
        }]),
        Kind::OnErrorGoto(target) => Some(vec![match target {
            crate::semantic_ir::ErrorHandlerTarget::Disable => "ON ERROR GOTO 0".to_string(),
            crate::semantic_ir::ErrorHandlerTarget::Label(target) => {
                format!("ON ERROR GOTO {}", render_target(target))
            }
        }]),
        // Callable globals are already represented in the resolver's
        // semantic name scopes; BASIC has no corresponding body statement.
        Kind::Global { .. } => Some(Vec::new()),
        Kind::OnBranch {
            selector,
            branch,
            targets,
        } => {
            let command = match branch {
                crate::semantic_ir::BranchKind::Goto => "GOTO",
                crate::semantic_ir::BranchKind::Gosub => "GOSUB",
            };
            let targets = targets
                .iter()
                .map(|target| render_target(target))
                .collect::<Vec<_>>();
            let (mut lines, selector) =
                generator.semantic_expression_with_prelude(selector, Some(function))?;
            lines.push(format!("ON {selector} {command} {}", targets.join(", ")));
            Some(lines)
        }
        Kind::Exit => Some(vec![match generator.loop_exit_stack.last() {
            Some(LoopExit::NativeFor) => "EXIT FOR".to_string(),
            Some(LoopExit::Goto(label)) => format!("GOTO {label}"),
            None => "' warning: EXIT outside of a loop".to_string(),
        }]),
        Kind::Continue => Some(vec![match generator.loop_continue_stack.last() {
            Some(label) => format!("GOTO {label}"),
            None => "' warning: CONTINUE outside of a loop".to_string(),
        }]),
        Kind::End => Some(vec!["END".to_string()]),
        Kind::Stop => Some(vec!["STOP".to_string()]),
        Kind::Cls => Some(vec!["CLS".to_string()]),
        Kind::Beep => Some(vec!["BEEP".to_string()]),
        Kind::System => Some(vec!["SYSTEM".to_string()]),
        Kind::Clear => Some(vec!["CLEAR".to_string()]),
        Kind::Error(code) => {
            let (mut lines, code) =
                generator.semantic_expression_with_prelude(code, Some(function))?;
            lines.push(format!("ERROR {code}"));
            Some(lines)
        }
        Kind::Throw(value) => match value {
            crate::semantic_ir::ThrowValue::Bare => Some(vec!["ERROR ERR".to_string()]),
            crate::semantic_ir::ThrowValue::Value(value) => {
                let (mut lines, value) =
                    generator.semantic_expression_with_prelude(value, Some(function))?;
                lines.push(format!("ERROR {value}"));
                Some(lines)
            }
        },
        Kind::Read(targets) => {
            let mut lines = Vec::new();
            let mut target_text = Vec::with_capacity(targets.len());
            for target in targets {
                let (prelude, rendered) =
                    basic_semantic_lvalue(generator, target, Some(function))?;
                lines.extend(prelude);
                target_text.push(rendered);
            }
            lines.push(format!("READ {}", target_text.join(", ")));
            Some(lines)
        }
        Kind::Data(values) => Some(vec![format!(
            "DATA {}",
            values
                .iter()
                .map(|value| generator.semantic_const_expression(value, Some(function)))
                .collect::<Option<Vec<_>>>()?
                .join(", ")
        )]),
        Kind::Return(ReturnValue::Value(expression)) if !function.is_procedure => {
            let (mut lines, expression) =
                generator.semantic_expression_with_prelude(expression, Some(function))?;
            lines.push(format!("{} = {expression}", function.result.as_basic()));
            lines.push("RETURN".to_string());
            Some(lines)
        }
        Kind::Return(ReturnValue::Default) => Some(vec!["RETURN".to_string()]),
        _ => None,
    }
}

/// Render a typed `MID$` assignment when its scalar operands and lvalue can
/// be emitted without AST reconstruction. Operand values and array indices
/// are snapshotted in source order before the inline splice.
fn basic_semantic_mid_assign(
    generator: &mut CodeGenerator,
    target: &crate::semantic_ir::Expression,
    start: &crate::semantic_ir::Expression,
    length: Option<&crate::semantic_ir::Expression>,
    value: &crate::semantic_ir::Expression,
    current_function: Option<&FunctionInfo>,
) -> Option<Vec<String>> {
    fn supported_expression(expression: &crate::semantic_ir::Expression) -> bool {
        use crate::semantic_ir::ExpressionKind as Kind;
        match &expression.kind {
            Kind::Literal(_) | Kind::Boolean(_) | Kind::Name(_) => true,
            Kind::Parenthesized(inner) | Kind::Unary { operand: inner, .. } => {
                supported_expression(inner)
            }
            Kind::Binary { left, right, .. } => {
                supported_expression(left) && supported_expression(right)
            }
            Kind::Index { index, .. } => supported_expression(index),
            Kind::MultiIndex { indices, .. } => indices.iter().all(supported_expression),
            // A resolved scalar function call is rendered with an explicit
            // prelude and its result is captured before later operands.
            Kind::Call { arguments, .. } => arguments.iter().all(supported_expression),
            // Scalar record members have typed storage names; record-valued
            // expressions and method calls still require other machinery.
            Kind::Member {
                base: Some(base),
                arguments: None,
                ..
            } => matches!(base.kind, Kind::Name(_)),
            _ => false,
        }
    }
    if ![target, start, value]
        .into_iter()
        .chain(length)
        .all(supported_expression)
    {
        return None;
    }
    let mut lines = Vec::new();
    let (target_prelude, target_lvalue) = basic_semantic_lvalue(generator, target, current_function)?;
    lines.extend(target_prelude);
    let target_text = generator.next_temp_var_suffixed("$");
    lines.push(format!("{target_text} = {target_lvalue}"));
    let mut render_snapshot =
        |expression: &crate::semantic_ir::Expression, is_string: bool, lines: &mut Vec<String>| {
            let (prelude, rendered) =
                generator.semantic_expression_with_prelude(expression, current_function)?;
            lines.extend(prelude);
            let suffix = if is_string { "$" } else { "%" };
            let temporary = generator.next_temp_var_suffixed(&suffix);
            lines.push(format!("{temporary} = {rendered}"));
            Some(temporary)
        };
    let start_text = render_snapshot(start, false, &mut lines)?;
    let length_text = match length {
        Some(length) => Some(render_snapshot(length, false, &mut lines)?),
        None => None,
    };
    let value_text = render_snapshot(value, true, &mut lines)?;
    let length_text = match length_text {
        Some(length) => length,
        None => {
            let length = generator.next_temp_var_suffixed("%");
            lines.push(format!("{length} = LEN({value_text})"));
            length
        }
    };
    let id = generator.next_label;
    generator.next_label += 1;
    let trim_label = format!("MID_{id:04}_TRIM");
    let done_label = format!("MID_{id:04}_DONE");
    lines.push(format!(
        "IF LEN({value_text}) > {length_text} THEN GOTO {trim_label}"
    ));
    lines.push(format!("GOTO {done_label}"));
    lines.push(format!("{trim_label}:"));
    lines.push(format!("{value_text} = LEFT$({value_text}, {length_text})"));
    lines.push(format!("{done_label}:"));
    lines.push(format!(
        "{target_lvalue} = LEFT$({target_text}, {start_text} - 1) + {value_text} + MID$({target_text}, {start_text} + LEN({value_text}))"
    ));
    Some(lines)
}

/// Render a typed scalar or array lvalue, evaluating each array index once
/// in source order before returning the target text.
fn basic_semantic_lvalue(
    generator: &mut CodeGenerator,
    target: &crate::semantic_ir::Expression,
    current_function: Option<&FunctionInfo>,
) -> Option<(Vec<String>, String)> {
    match &target.kind {
        crate::semantic_ir::ExpressionKind::Name(_) => (
            Vec::new(),
            generator.semantic_const_expression(target, current_function)?,
        ),
        crate::semantic_ir::ExpressionKind::Member {
            base: Some(base),
            arguments: None,
            ..
        } if matches!(base.kind, crate::semantic_ir::ExpressionKind::Name(_)) => (
            Vec::new(),
            generator.semantic_const_expression(target, current_function)?,
        ),
        crate::semantic_ir::ExpressionKind::Index { name, index } => {
            let ident = BasicIdent::parse(name);
            if generator.ordinary_function_info(&ident).is_some()
                || generator.resolve_array_rank(&ident, current_function)? != 1
            {
                return None;
            }
            if !generator.semantic_expression_contains_callable_call(index) {
                return Some((
                    Vec::new(),
                    generator.semantic_const_expression(target, current_function)?,
                ));
            }
            let (mut prelude, rendered) =
                generator.semantic_expression_with_prelude(index, current_function)?;
            let suffix = index
                .value_type
                .suffix()
                .map(|suffix| suffix.to_string())
                .unwrap_or_else(|| "%".to_string());
            let temporary = generator.next_temp_var_suffixed(&suffix);
            prelude.push(format!("{temporary} = {rendered}"));
            (
                prelude,
                format!("{}({temporary})", generator.ident(&ident, current_function)),
            )
        }
        crate::semantic_ir::ExpressionKind::MultiIndex { name, indices } => {
            let ident = BasicIdent::parse(name);
            if generator.ordinary_function_info(&ident).is_some()
                || generator.resolve_array_rank(&ident, current_function)? != indices.len()
            {
                return None;
            }
            if !indices
                .iter()
                .any(|index| generator.semantic_expression_contains_callable_call(index))
            {
                return Some((
                    Vec::new(),
                    generator.semantic_const_expression(target, current_function)?,
                ));
            }
            let mut prelude = Vec::new();
            let mut rendered_indices = Vec::with_capacity(indices.len());
            for index in indices {
                let (index_prelude, rendered) =
                    generator.semantic_expression_with_prelude(index, current_function)?;
                prelude.extend(index_prelude);
                let suffix = index
                    .value_type
                    .suffix()
                    .map(|suffix| suffix.to_string())
                    .unwrap_or_else(|| "%".to_string());
                let temporary = generator.next_temp_var_suffixed(&suffix);
                prelude.push(format!("{temporary} = {rendered}"));
                rendered_indices.push(temporary);
            }
            (
                prelude,
                format!(
                    "{}({})",
                    generator.ident(&ident, current_function),
                    rendered_indices.join(", ")
                ),
            )
        }
        crate::semantic_ir::ExpressionKind::Call { name, arguments } => {
            // The generated frontend represents ordinary parenthesized
            // references as calls. Resolved array rank distinguishes an
            // lvalue subscript here from a callable invocation.
            let ident = BasicIdent::parse(name);
            if generator.resolve_array_rank(&ident, current_function)? != arguments.len()
                || generator.ordinary_function_info(&ident).is_some()
            {
                return None;
            }
            if !arguments
                .iter()
                .any(|index| generator.semantic_expression_contains_callable_call(index))
            {
                return Some((
                    Vec::new(),
                    generator.semantic_const_expression(target, current_function)?,
                ));
            }
            let mut prelude = Vec::new();
            let mut rendered_indices = Vec::with_capacity(arguments.len());
            for index in arguments {
                let (index_prelude, rendered) =
                    generator.semantic_expression_with_prelude(index, current_function)?;
                prelude.extend(index_prelude);
                let suffix = index
                    .value_type
                    .suffix()
                    .map(|suffix| suffix.to_string())
                    .unwrap_or_else(|| "%".to_string());
                let temporary = generator.next_temp_var_suffixed(&suffix);
                prelude.push(format!("{temporary} = {rendered}"));
                rendered_indices.push(temporary);
            }
            (
                prelude,
                format!(
                    "{}({})",
                    generator.ident(&ident, current_function),
                    rendered_indices.join(", ")
                ),
            )
        }
        _ => return None,
    }
    .into()
}

fn basic_semantic_callable_statement(
    generator: &mut CodeGenerator,
    semantic: &crate::semantic_ir::SemanticStatement,
    function: &FunctionInfo,
) -> Option<Vec<String>> {
    let initial_label = generator.next_label;
    let initial_loop_exit_depth = generator.loop_exit_stack.len();
    let initial_loop_continue_depth = generator.loop_continue_stack.len();
    let lines = basic_semantic_callable_statement_inner(generator, semantic, function);
    if lines.is_none() {
        generator.next_label = initial_label;
        generator.loop_exit_stack.truncate(initial_loop_exit_depth);
        generator
            .loop_continue_stack
            .truncate(initial_loop_continue_depth);
    }
    lines
}

fn basic_semantic_callable_statement_inner(
    generator: &mut CodeGenerator,
    semantic: &crate::semantic_ir::SemanticStatement,
    function: &FunctionInfo,
) -> Option<Vec<String>> {
    use crate::semantic_ir::SemanticStatementKind as Kind;
    match &semantic.kind {
        Kind::Line(statements) => {
            let mut lines = Vec::new();
            for statement in statements {
                lines.extend(basic_semantic_callable_statement(
                    generator, statement, function,
                )?);
            }
            Some(lines)
        }
        Kind::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            let condition = generator.semantic_condition(condition, Some(function))?;
            let mut then_lines = Vec::new();
            let mut else_lines = Vec::new();
            for statement in then_body {
                then_lines.extend(basic_semantic_callable_statement(
                    generator, statement, function,
                )?);
            }
            for statement in else_body {
                else_lines.extend(basic_semantic_callable_statement(
                    generator, statement, function,
                )?);
            }
            let id = generator.next_label;
            generator.next_label += 1;
            let else_label = format!("IF_{id:04}_ELSE");
            let end_label = format!("IF_{id:04}_END");
            let mut lines = Vec::new();
            if else_body.is_empty() {
                lines.extend(condition.jump_lines(&end_label, false));
                lines.extend(then_lines.into_iter().map(|line| format!("    {line}")));
                lines.push(format!("{end_label}:"));
            } else {
                lines.extend(condition.jump_lines(&else_label, false));
                lines.extend(then_lines.into_iter().map(|line| format!("    {line}")));
                lines.push(format!("GOTO {end_label}"));
                lines.push(format!("{else_label}:"));
                lines.extend(else_lines.into_iter().map(|line| format!("    {line}")));
                lines.push(format!("{end_label}:"));
            }
            lines.push("REM END IF".to_string());
            Some(lines)
        }
        Kind::For {
            variable,
            variable_type,
            start,
            bounds,
            body,
            ..
        } => {
            let start_suffix = start
                .value_type
                .suffix()
                .map(|suffix| suffix.to_string())
                .unwrap_or_default();
            let bounds_contain_callable_call = match bounds {
                crate::semantic_ir::ForBounds::To { limit, step } => {
                    generator.semantic_expression_contains_callable_call(limit)
                        || step.as_ref().is_some_and(|step| {
                            generator.semantic_expression_contains_callable_call(step)
                        })
                }
                crate::semantic_ir::ForBounds::Downto { limit, .. } => {
                    generator.semantic_expression_contains_callable_call(limit)
                }
            };
            let (mut for_prelude, start) =
                if let Some(start) = generator.semantic_const_expression(start, Some(function)) {
                    (Vec::new(), start)
                } else {
                    generator.semantic_expression_with_prelude(start, Some(function))?
                };
            let start = if bounds_contain_callable_call {
                let snapshot = generator.next_temp_var_suffixed(&start_suffix);
                for_prelude.push(format!("{snapshot} = {start}"));
                snapshot
            } else {
                start
            };
            let (limit, step) = match bounds {
                crate::semantic_ir::ForBounds::To {
                    limit: limit_expression,
                    step,
                } => {
                    let limit = if let Some(limit) =
                        generator.semantic_const_expression(limit_expression, Some(function))
                    {
                        limit
                    } else {
                        let (prelude, limit) = generator
                            .semantic_expression_with_prelude(limit_expression, Some(function))?;
                        for_prelude.extend(prelude);
                        limit
                    };
                    let limit = if step.as_ref().is_some_and(|step| {
                        generator.semantic_expression_contains_callable_call(step)
                    }) {
                        let suffix = limit_expression
                            .value_type
                            .suffix()
                            .map(|suffix| suffix.to_string())
                            .unwrap_or_default();
                        let snapshot = generator.next_temp_var_suffixed(&suffix);
                        for_prelude.push(format!("{snapshot} = {limit}"));
                        snapshot
                    } else {
                        limit
                    };
                    let step = match step {
                        Some(step) => {
                            let step = if let Some(step) =
                                generator.semantic_const_expression(step, Some(function))
                            {
                                step
                            } else {
                                let (prelude, step) = generator
                                    .semantic_expression_with_prelude(step, Some(function))?;
                                for_prelude.extend(prelude);
                                step
                            };
                            format!(" STEP {step}")
                        }
                        None => String::new(),
                    };
                    (limit, step)
                }
                crate::semantic_ir::ForBounds::Downto { limit, step } => {
                    let limit = if let Some(limit) =
                        generator.semantic_const_expression(limit, Some(function))
                    {
                        limit
                    } else {
                        let (prelude, limit) =
                            generator.semantic_expression_with_prelude(limit, Some(function))?;
                        for_prelude.extend(prelude);
                        limit
                    };
                    (limit, format!(" STEP {step}"))
                }
            };
            let continue_id = generator.next_label;
            generator.next_label += 1;
            let continue_label = format!("FOR_{continue_id:04}_CONTINUE");
            generator.loop_exit_stack.push(LoopExit::NativeFor);
            generator.loop_continue_stack.push(continue_label.clone());
            let mut body_lines = Vec::new();
            let body_result = body.iter().try_for_each(|statement| {
                body_lines.extend(basic_semantic_callable_statement(
                    generator, statement, function,
                )?);
                Some(())
            });
            generator.loop_continue_stack.pop();
            generator.loop_exit_stack.pop();
            body_result?;
            let mut variable_ident = BasicIdent::parse(variable);
            variable_ident.suffix = variable_type
                .suffix()
                .and_then(crate::ast::TypeSuffix::from_char);
            let variable = generator.ident(&variable_ident, Some(function));
            for_prelude.push(format!("FOR {variable} = {start} TO {limit}{step}"));
            let mut lines = for_prelude;
            lines.extend(body_lines.into_iter().map(|line| format!("    {line}")));
            lines.push(format!("{continue_label}:"));
            lines.push(format!("NEXT {variable}"));
            Some(lines)
        }
        Kind::While { condition, body } => {
            let condition = generator.semantic_condition(condition, Some(function))?;
            let id = generator.next_label;
            generator.next_label += 1;
            let top_label = format!("WHILE_{id:04}_TOP");
            let end_label = format!("WHILE_{id:04}_END");
            generator
                .loop_exit_stack
                .push(LoopExit::Goto(end_label.clone()));
            generator.loop_continue_stack.push(top_label.clone());
            let mut body_lines = Vec::new();
            let body_result = body.iter().try_for_each(|statement| {
                body_lines.extend(basic_semantic_callable_statement(
                    generator, statement, function,
                )?);
                Some(())
            });
            generator.loop_continue_stack.pop();
            generator.loop_exit_stack.pop();
            body_result?;
            let mut lines = vec![format!("{top_label}:")];
            lines.extend(condition.jump_lines(&end_label, false));
            lines.extend(body_lines.into_iter().map(|line| format!("    {line}")));
            lines.push(format!("GOTO {top_label}"));
            lines.push(format!("{end_label}:"));
            lines.push("REM END WHILE".to_string());
            Some(lines)
        }
        Kind::Do {
            pre_condition,
            post_condition,
            body,
        } => {
            let render_condition =
                |generator: &mut CodeGenerator, condition: &crate::semantic_ir::LoopCondition| {
                    generator
                        .semantic_condition(&condition.value, Some(function))
                        .map(|rendered| (condition.kind, rendered))
                };
            let pre = match pre_condition {
                Some(condition) => Some(render_condition(generator, condition)?),
                None => None,
            };
            let post = match post_condition {
                Some(condition) => Some(render_condition(generator, condition)?),
                None => None,
            };
            let id = generator.next_label;
            generator.next_label += 1;
            let top_label = format!("DO_{id:04}_TOP");
            let end_label = format!("DO_{id:04}_END");
            let continue_label = format!("DO_{id:04}_CONTINUE");
            generator
                .loop_exit_stack
                .push(LoopExit::Goto(end_label.clone()));
            generator.loop_continue_stack.push(continue_label.clone());
            let mut body_lines = Vec::new();
            let body_result = body.iter().try_for_each(|statement| {
                body_lines.extend(basic_semantic_callable_statement(
                    generator, statement, function,
                )?);
                Some(())
            });
            generator.loop_continue_stack.pop();
            generator.loop_exit_stack.pop();
            body_result?;
            let mut lines = vec![format!("{top_label}:")];
            if let Some((kind, condition)) = pre {
                let invert = kind != crate::semantic_ir::LoopConditionKind::While;
                lines.extend(condition.jump_lines(&end_label, invert));
            }
            lines.extend(body_lines.into_iter().map(|line| format!("    {line}")));
            lines.push(format!("{continue_label}:"));
            if let Some((kind, condition)) = post {
                let invert = kind == crate::semantic_ir::LoopConditionKind::While;
                lines.extend(
                    condition
                        .jump_lines(&top_label, invert)
                        .into_iter()
                        .map(|line| format!("    {line}")),
                );
            } else {
                lines.push(format!("GOTO {top_label}"));
            }
            lines.push(format!("{end_label}:"));
            lines.push("REM END DO".to_string());
            Some(lines)
        }
        Kind::SelectCase {
            selector,
            cases,
            else_body,
        } => {
            let (selector_prelude, selector_text) = if let Some(rendered) =
                generator.semantic_const_expression(selector, Some(function))
            {
                (Vec::new(), rendered)
            } else {
                generator.semantic_expression_with_prelude(selector, Some(function))?
            };
            let id = generator.next_label;
            generator.next_label += 1;
            let end_label = format!("SEL_{id:04}_END");
            let temp_id = generator.next_label;
            generator.next_label += 1;
            let suffix = selector
                .value_type
                .suffix()
                .map(|suffix| suffix.to_string())
                .unwrap_or_default();
            let temp = format!("BCCT{temp_id}{suffix}");
            let case_labels = (0..cases.len())
                .map(|index| format!("SEL_{id:04}_C{index}"))
                .collect::<Vec<_>>();
            let else_label = format!("SEL_{id:04}_ELSE");
            let mut rendered_cases = Vec::new();
            for clause in cases {
                let mut values = Vec::new();
                for value in &clause.values {
                    values.push(match value {
                        crate::semantic_ir::CaseValue::Value {
                            first,
                            range_end: None,
                            ..
                        } => {
                            format!(
                                "{temp} = {}",
                                generator.semantic_const_expression(first, Some(function))?
                            )
                        }
                        crate::semantic_ir::CaseValue::Value {
                            first,
                            range_end: Some(last),
                            ..
                        } => {
                            let first =
                                generator.semantic_const_expression(first, Some(function))?;
                            let last = generator.semantic_const_expression(last, Some(function))?;
                            format!("{temp} >= {first} AND {temp} <= {last}")
                        }
                        crate::semantic_ir::CaseValue::Comparison {
                            operator, value, ..
                        } => {
                            let value =
                                generator.semantic_const_expression(value, Some(function))?;
                            let operator = match operator {
                                crate::semantic_ir::ComparisonOperator::NotEqual => "<>",
                                crate::semantic_ir::ComparisonOperator::LessOrEqual => "<=",
                                crate::semantic_ir::ComparisonOperator::GreaterOrEqual => ">=",
                                crate::semantic_ir::ComparisonOperator::Equal => "=",
                                crate::semantic_ir::ComparisonOperator::Less => "<",
                                crate::semantic_ir::ComparisonOperator::Greater => ">",
                            };
                            format!("{temp} {operator} {value}")
                        }
                    });
                }
                if values.is_empty() {
                    return None;
                }
                rendered_cases.push(values.join(" OR "));
            }
            let mut rendered_bodies = Vec::new();
            for clause in cases {
                let mut lines = Vec::new();
                for statement in &clause.body {
                    lines.extend(basic_semantic_callable_statement(
                        generator, statement, function,
                    )?);
                }
                rendered_bodies.push(lines);
            }
            let mut rendered_else = Vec::new();
            for statement in else_body {
                rendered_else.extend(basic_semantic_callable_statement(
                    generator, statement, function,
                )?);
            }
            let mut lines = selector_prelude;
            lines.push(format!("{temp} = {selector_text}"));
            for (index, condition) in rendered_cases.iter().enumerate() {
                lines.push(format!(
                    "IF ({condition}) <> 0 THEN GOTO {}",
                    case_labels[index]
                ));
            }
            lines.push(format!(
                "GOTO {}",
                if else_body.is_empty() {
                    &end_label
                } else {
                    &else_label
                }
            ));
            for (index, body) in rendered_bodies.into_iter().enumerate() {
                lines.push(format!("{}:", case_labels[index]));
                lines.extend(body.into_iter().map(|line| format!("    {line}")));
                lines.push(format!("    GOTO {end_label}"));
            }
            if !else_body.is_empty() {
                lines.push(format!("{else_label}:"));
                lines.extend(rendered_else.into_iter().map(|line| format!("    {line}")));
            }
            lines.push(format!("{end_label}:"));
            lines.push("REM END SELECT".to_string());
            Some(lines)
        }
        _ => basic_semantic_callable_leaf(generator, semantic, function),
    }
}

fn basic_semantic_print_body(
    generator: &mut CodeGenerator,
    tokens: &[crate::semantic_ir::PrintToken],
    function: Option<&FunctionInfo>,
) -> Option<(Vec<String>, String)> {
    use crate::semantic_ir::PrintToken;
    let mut body = String::new();
    let has_callable_expression = tokens.iter().any(|token| {
        matches!(token, PrintToken::Expression(expression)
            if generator.semantic_expression_contains_callable_call(expression))
    });
    let expression_count = tokens
        .iter()
        .filter(|token| matches!(token, PrintToken::Expression(_)))
        .count();
    let mut lines = Vec::new();
    let mut after_separator = false;
    for token in tokens {
        match token {
            PrintToken::Expression(expression) => {
                let rendered = if has_callable_expression {
                    let (mut prelude, mut value) =
                        generator.semantic_expression_with_prelude(expression, function)?;
                    if expression_count == 1
                        && matches!(expression.kind, crate::semantic_ir::ExpressionKind::Call { .. })
                        && prelude.last().is_some_and(|line| {
                            line.strip_prefix(&format!("{value} = ")).is_some()
                        })
                    {
                        let line = prelude.pop().expect("checked call result");
                        value = line
                            .split_once(" = ")
                            .expect("checked call result assignment")
                            .1
                            .to_string();
                        generator.next_label = generator.next_label.saturating_sub(1);
                    } else if expression_count > 1 {
                        let suffix = expression
                            .value_type
                            .suffix()
                            .map(|suffix| suffix.to_string())
                            .unwrap_or_default();
                        let snapshot = generator.next_temp_var_suffixed(&suffix);
                        prelude.push(format!("{snapshot} = {value}"));
                        value = snapshot;
                    }
                    lines.extend(prelude);
                    value
                } else {
                    generator.semantic_const_expression(expression, function)?
                };
                if after_separator {
                    body.push(' ');
                }
                body.push_str(&rendered);
                after_separator = false;
            }
            PrintToken::Comma { .. } => {
                body.push(',');
                after_separator = true;
            }
            PrintToken::Semicolon { .. } => {
                body.push(';');
                after_separator = true;
            }
        }
    }
    Some((
        lines,
        if body.is_empty() {
            String::new()
        } else {
            format!(" {body}")
        },
    ))
}

fn basic_semantic_io_values(
    generator: &mut CodeGenerator,
    values: &[crate::semantic_ir::Expression],
    function: Option<&FunctionInfo>,
) -> Option<(Vec<String>, Vec<String>)> {
    let has_callable = values
        .iter()
        .any(|value| generator.semantic_expression_contains_callable_call(value));
    let mut lines = Vec::new();
    let mut rendered = Vec::with_capacity(values.len());
    for value in values {
        let text = if has_callable {
            let (mut prelude, mut text) =
                generator.semantic_expression_with_prelude(value, function)?;
            if values.len() == 1
                && matches!(value.kind, crate::semantic_ir::ExpressionKind::Call { .. })
                && prelude.last().is_some_and(|line| {
                    line.strip_prefix(&format!("{text} = ")).is_some()
                })
            {
                let line = prelude.pop().expect("checked call result");
                text = line
                    .split_once(" = ")
                    .expect("checked call result assignment")
                    .1
                    .to_string();
                generator.next_label = generator.next_label.saturating_sub(1);
            } else if values.len() > 1 {
                let suffix = value
                    .value_type
                    .suffix()
                    .map(|suffix| suffix.to_string())
                    .unwrap_or_default();
                let snapshot = generator.next_temp_var_suffixed(&suffix);
                prelude.push(format!("{snapshot} = {text}"));
                text = snapshot;
            }
            lines.extend(prelude);
            text
        } else {
            generator.semantic_const_expression(value, function)?
        };
        rendered.push(text);
    }
    Some((lines, rendered))
}

fn basic_semantic_input_prompt(prompt: &crate::semantic_ir::InputPrompt) -> String {
    let text = prompt
        .text
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .unwrap_or(&prompt.text)
        .replace("\"\"", "\"");
    format!("\"{}\"; ", escape_string(&text))
}

fn basic_semantic_assignment(
    generator: &mut CodeGenerator,
    target: &crate::semantic_ir::Expression,
    operator: crate::semantic_ir::AssignmentOperator,
    value: &crate::semantic_ir::Expression,
    current_function: Option<&FunctionInfo>,
) -> Option<Vec<String>> {
    use crate::semantic_ir::{AssignmentOperator, ExpressionKind};
    let (mut prelude, target) = basic_semantic_lvalue(generator, target, current_function)?;
    let (value_prelude, mut rendered_value) =
        if let Some(value) = generator.semantic_const_expression(value, current_function) {
            (Vec::new(), value)
        } else {
            generator.semantic_expression_with_prelude(value, current_function)?
        };
    prelude.extend(value_prelude);
    if let ExpressionKind::Call { name, .. } = &value.kind {
        if let Some(info) = generator.ordinary_function_info(&BasicIdent::parse(name)) {
            let result_name = info.result.as_basic();
            if prelude.last() == Some(&format!("{rendered_value} = {result_name}")) {
                prelude.pop();
                generator.next_label = generator.next_label.saturating_sub(1);
                rendered_value = result_name;
            }
        }
    }
    let expression = match operator {
        AssignmentOperator::Assign => rendered_value,
        AssignmentOperator::Add => format!("{target} + {rendered_value}"),
        AssignmentOperator::Subtract => format!("{target} - {rendered_value}"),
        AssignmentOperator::Multiply => format!("{target} * {rendered_value}"),
        AssignmentOperator::Divide => format!("{target} / {rendered_value}"),
    };
    prelude.push(format!("{target} = {expression}"));
    Some(prelude)
}

fn basic_semantic_comment(block: bool, text: &str) -> Vec<String> {
    if !block {
        let body = text
            .strip_prefix('\'')
            .or_else(|| text.strip_prefix("//"))
            .unwrap_or(text);
        return vec![format!("' {}", body.trim_start())];
    }

    let body = text
        .strip_prefix("/*")
        .and_then(|body| body.strip_suffix("*/"))
        .unwrap_or(text);
    let lines = body
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix('*')
                .map(|line| line.trim())
                .unwrap_or(trimmed)
                .to_string()
        })
        .collect::<Vec<_>>();
    let start = lines.iter().position(|line| !line.is_empty()).unwrap_or(0);
    let end = lines
        .iter()
        .rposition(|line| !line.is_empty())
        .map(|index| index + 1)
        .unwrap_or(start);
    lines[start..end]
        .iter()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("' {line}")
            }
        })
        .collect()
}

fn basic_semantic_expression_statement(
    generator: &mut CodeGenerator,
    expression: &crate::semantic_ir::Expression,
    function: Option<&FunctionInfo>,
) -> Option<Vec<String>> {
    use crate::semantic_ir::{ExpressionKind, SemanticValueType};
    if let ExpressionKind::Call { name, arguments } = &expression.kind {
        let callable = BasicIdent::parse(name);
        if let Some(info) = generator.ordinary_function_info(&callable).cloned() {
            if info.is_procedure {
                let parameters = info.semantic_parameters.as_ref()?;
                if arguments.len() > parameters.len()
                    || parameters.len() != info.params.len()
                    || info.params.iter().zip(parameters).enumerate().any(
                        |(index, ((ast, _), semantic))| {
                            !ast.name.as_basic().eq_ignore_ascii_case(&semantic.name)
                                || info.param_ranks.get(index).copied().flatten().unwrap_or(0)
                                    != semantic.array_axes
                        },
                    )
                    || parameters
                        .iter()
                        .skip(arguments.len())
                        .any(|parameter| parameter.default.is_none())
                {
                    return None;
                }
                let mut complete_arguments = arguments.clone();
                complete_arguments.extend(
                    parameters
                        .iter()
                        .skip(arguments.len())
                        .filter_map(|parameter| parameter.default.clone()),
                );
                let mut lines = Vec::new();
                let mut byref_copyback = Vec::new();
                let mut array_copyback = Vec::new();
                for (index, (argument, (_, lowered))) in
                    complete_arguments.iter().zip(&info.params).enumerate()
                {
                    let semantic_parameter = &parameters[index];
                    if semantic_parameter.array_axes > 0 {
                        let ExpressionKind::Name(argument_name) = &argument.kind else {
                            return None;
                        };
                        let source_ident = BasicIdent::parse(argument_name);
                        let rank = generator.resolve_array_rank(&source_ident, function)?;
                        if rank != semantic_parameter.array_axes {
                            return None;
                        }
                        let expected = semantic_parameter
                            .value_type
                            .suffix()
                            .and_then(TypeSuffix::from_char)?;
                        if generator.resolve_array_suffix(&source_ident, function)? != expected {
                            return None;
                        }
                        let source = generator.ident(&source_ident, function);
                        let bounds = info.param_bound_vars.get(index)?;
                        let capacities = info.param_capacities.get(index)?;
                        if bounds.len() != rank {
                            return None;
                        }
                        for (axis, bound_var) in bounds.iter().enumerate() {
                            let bound =
                                generator.resolve_axis_bound(&source_ident, axis, function)?;
                            lines.push(format!("{bound_var} = {bound}"));
                            if let Some(capacity) = capacities.get(axis) {
                                lines.push(format!(
                                    "IF {bound_var} > {capacity} THEN PRINT \"runtime error: `{}` of `{}` needs \"; {bound_var}; \" elements along axis {axis}, but its storage only holds {capacity}\" : STOP",
                                    semantic_parameter.name, info.source_name,
                                ));
                            }
                        }
                        let loop_vars = (0..rank)
                            .map(|_| generator.next_temp_var())
                            .collect::<Vec<_>>();
                        lines.extend(array_copy_lines(
                            &lowered.as_basic(),
                            &source,
                            bounds,
                            "copy array argument into transpiled procedure storage",
                            &loop_vars,
                        ));
                        if semantic_parameter.passing == Some(crate::semantic_ir::Passing::ByRef) {
                            array_copyback.push((
                                source,
                                lowered.as_basic(),
                                bounds.to_vec(),
                                rank,
                            ));
                        }
                        continue;
                    }
                    let expected = semantic_parameter
                        .value_type
                        .suffix()
                        .and_then(TypeSuffix::from_char)?;
                    if argument
                        .value_type
                        .suffix()
                        .and_then(TypeSuffix::from_char)?
                        != expected
                    {
                        return None;
                    }
                    if semantic_parameter.passing == Some(crate::semantic_ir::Passing::ByRef) {
                        let ExpressionKind::Name(argument_name) = &argument.kind else {
                            return None;
                        };
                        let caller = generator.ident(&BasicIdent::parse(argument_name), function);
                        lines.push(format!("{} = {caller}", lowered.as_basic()));
                        byref_copyback.push((caller, lowered.as_basic()));
                    } else {
                        let (prelude, value) =
                            generator.semantic_expression_with_prelude(argument, function)?;
                        lines.extend(prelude);
                        lines.push(format!("{} = {value}", lowered.as_basic()));
                    }
                }
                lines.push(format!("GOSUB {}", info.label));
                for (source, destination, bounds, rank) in array_copyback {
                    let loop_vars = (0..rank)
                        .map(|_| generator.next_temp_var())
                        .collect::<Vec<_>>();
                    lines.extend(array_copy_lines(
                        &source,
                        &destination,
                        &bounds,
                        "copy mutated array argument back to caller storage",
                        &loop_vars,
                    ));
                }
                lines.extend(
                    byref_copyback
                        .into_iter()
                        .map(|(caller, lowered)| format!("{caller} = {lowered}")),
                );
                return Some(lines);
            }
        }
    }
    if let ExpressionKind::Call { name, arguments } = &expression.kind {
        let callable = BasicIdent::parse(name);
        let has_typed_callable = generator.ordinary_function_info(&callable).is_some()
            || generator.function_info(&callable).is_some();
        if !has_typed_callable
            && arguments.iter().any(|argument| {
                matches!(&argument.kind, ExpressionKind::Name(argument_name)
                    if generator.resolve_array_rank(
                        &BasicIdent::parse(argument_name),
                        function,
                    ).is_some())
            })
        {
            // The call signature is unresolved, so the typed IR cannot tell
            // whether this array name is a whole-array argument or a scalar
            // value. Preserve the legacy statement emission in that case.
            return None;
        }
    }
    if let ExpressionKind::Member {
        base: Some(base),
        member,
        arguments: Some(arguments),
    } = &expression.kind
    {
        let receiver = match base.value_type {
            SemanticValueType::String => Some(TypeSuffix::String),
            SemanticValueType::Integer => Some(TypeSuffix::Integer),
            SemanticValueType::Long => Some(TypeSuffix::Long),
            SemanticValueType::Single => Some(TypeSuffix::Single),
            SemanticValueType::Double => Some(TypeSuffix::Double),
            SemanticValueType::Unknown | SemanticValueType::Boolean => None,
        }?;
        let info = generator.method_info(receiver, member)?.clone();
        let mut call_arguments = Vec::with_capacity(arguments.len() + 1);
        call_arguments.push(base.as_ref().clone());
        call_arguments.extend(arguments.iter().cloned());
        if call_arguments.len() != info.params.len() || info.param_ranks.iter().any(Option::is_some)
        {
            return None;
        }
        let mut lines = Vec::new();
        let mut byref_copyback = Vec::new();
        for (position, (argument, (parameter, lowered))) in
            call_arguments.iter().zip(&info.params).enumerate()
        {
            if parameter.mode == ParamMode::ByRef {
                let ExpressionKind::Name(name) = &argument.kind else {
                    return None;
                };
                let caller = generator.ident(&BasicIdent::parse(name), function);
                lines.push(format!("{} = {caller}", lowered.as_basic()));
                byref_copyback.push((caller, lowered.as_basic()));
            } else {
                let (prelude, rendered) =
                    generator.semantic_expression_with_prelude(argument, function)?;
                lines.extend(prelude);
                let semantic_position = position.checked_sub(usize::from(info.receiver.is_some()));
                let suffix = semantic_position
                    .and_then(|position| info.semantic_parameters.as_ref()?.get(position))
                    .and_then(|parameter| parameter.value_type.suffix())
                    .and_then(TypeSuffix::from_char)
                    .or(parameter.name.suffix)
                    .map(|suffix| suffix.to_string())
                    .unwrap_or_default();
                let temporary = generator.next_temp_var_suffixed(&suffix);
                lines.push(format!("{temporary} = {rendered}"));
                lines.push(format!("{} = {temporary}", lowered.as_basic()));
            }
        }
        lines.push(format!("GOSUB {}", info.label));
        for (caller, lowered) in byref_copyback {
            lines.push(format!("{caller} = {lowered}"));
        }
        return Some(lines);
    }

    let (mut lines, rendered) = generator.semantic_expression_with_prelude(expression, function)?;
    if matches!(expression.kind, ExpressionKind::Call { .. })
        && lines
            .last()
            .is_some_and(|line| line.starts_with(&format!("{rendered} = ")))
    {
        lines.pop();
        generator.next_label = generator.next_label.saturating_sub(1);
    } else if lines.is_empty() {
        lines.push(rendered);
    }
    Some(lines)
}

fn semantic_dim_type_name(
    value_type: crate::semantic_ir::SemanticValueType,
) -> Option<&'static str> {
    match value_type {
        crate::semantic_ir::SemanticValueType::String => Some("STRING"),
        crate::semantic_ir::SemanticValueType::Integer => Some("INTEGER"),
        crate::semantic_ir::SemanticValueType::Long => Some("LONG"),
        crate::semantic_ir::SemanticValueType::Single => Some("SINGLE"),
        crate::semantic_ir::SemanticValueType::Double => Some("DOUBLE"),
        crate::semantic_ir::SemanticValueType::Unknown
        | crate::semantic_ir::SemanticValueType::Boolean => None,
    }
}

fn semantic_const_suffix(
    value_type: crate::semantic_ir::SemanticValueType,
) -> Option<TypeSuffix> {
    match value_type {
        crate::semantic_ir::SemanticValueType::Double => Some(TypeSuffix::Single),
        crate::semantic_ir::SemanticValueType::String => Some(TypeSuffix::String),
        crate::semantic_ir::SemanticValueType::Integer => Some(TypeSuffix::Integer),
        crate::semantic_ir::SemanticValueType::Long => Some(TypeSuffix::Long),
        crate::semantic_ir::SemanticValueType::Single => Some(TypeSuffix::Single),
        crate::semantic_ir::SemanticValueType::Unknown
        | crate::semantic_ir::SemanticValueType::Boolean => None,
    }
}

fn source_position(
    source: &crate::semantic_ir::SemanticSource,
    span: crate::rdgen_frontend::SourceSpan,
) -> Option<crate::diagnostics::SourcePos> {
    source.source_position_at(span.start)
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            next_label: 1,
            indent: 0,
            output: String::new(),
            functions: Vec::new(),
            known_callables: HashSet::new(),
            line_numbers: false,
            loop_exit_stack: Vec::new(),
            loop_continue_stack: Vec::new(),
            taken_names: RefCell::new(HashSet::new()),
            record_buffer_names: HashSet::new(),
            const_var_names: HashMap::new(),
            semantic_const_initializers: HashMap::new(),
            semantic_records: Vec::new(),
            semantic_top_level_dims: HashMap::new(),
            synthesized_buffer_names: HashSet::new(),
            diagnostics: Vec::new(),
            top_level_array_ranks: HashMap::new(),
            top_level_array_bounds: HashMap::new(),
            error_handler_procedures: HashSet::new(),
            try_handler_stack: Vec::new(),
            needs_source_lookup: false,
            deferred_errors: std::cell::RefCell::new(Vec::new()),
            current_marker_file: None,
        }
    }

    pub fn with_synthesized_buffer_names(mut self, value: HashSet<String>) -> Self {
        self.synthesized_buffer_names = value;
        self
    }

    pub fn with_line_numbers(mut self, value: bool) -> Self {
        self.line_numbers = value;
        self
    }

    pub fn generate(
        mut self,
        resolved: &crate::resolver::ResolvedProgram,
    ) -> Result<String, Vec<Diagnostic>> {
        let program = &resolved.program;
        self.needs_source_lookup = resolved
            .semantic_module
            .as_ref()
            .map(crate::semantic_ir::SemanticModule::uses_catch_source_var)
            .unwrap_or(resolved.uses_catch_source_var);
        // Seed the name registry with every variable visible at global scope.
        // Function params/results are registered as each FunctionInfo is built so
        // later functions cannot collide with earlier ones either.
        let mut taken = resolved
            .semantic_name_scopes
            .as_ref()
            .map(|scopes| scopes.global_names.iter().cloned().collect())
            .unwrap_or_else(|| collect_program_names(program));
        let semantic_record_storage = resolved
            .semantic_module
            .as_ref()
            .map(semantic_record_storage_names)
            .unwrap_or_default();
        taken.extend(semantic_record_storage.iter().cloned());
        if let Some(scopes) = resolved.semantic_name_scopes.as_ref() {
            for names in scopes.callable_globals.values() {
                taken.extend(names.iter().cloned());
            }
        }
        for block in &resolved.common_blocks {
            for variable in &block.vars {
                taken.insert(variable.name.as_basic().to_ascii_lowercase());
            }
        }
        let known_callables: HashSet<String> =
            if let Some(module) = resolved.semantic_module.as_ref() {
                module
                    .callables
                    .iter()
                    .flat_map(|callable| {
                        let typed = callable.name.to_ascii_lowercase();
                        let base = typed
                            .trim_end_matches(['$', '%', '&', '!', '#'])
                            .to_string();
                        [typed, base].into_iter()
                    })
                    .chain(BASIC_BUILTINS.iter().map(|s| s.to_string()))
                    .collect()
            } else {
                program
                    .functions
                    .iter()
                    .map(|f| f.name.name.to_ascii_lowercase())
                    .chain(BASIC_BUILTINS.iter().map(|s| s.to_string()))
                    .collect()
            };
        let param_capacities = infer_array_param_capacities(
            program,
            resolved.semantic_module.as_ref(),
            &mut self.diagnostics,
        );
        let top_level_array_ranks = resolved
            .semantic_module
            .as_ref()
            .map(crate::semantic_ir::SemanticModule::top_level_array_ranks)
            .unwrap_or_else(|| resolved.top_level_array_ranks.clone());
        self.semantic_const_initializers = resolved
            .semantic_module
            .as_ref()
            .map(crate::semantic_ir::SemanticModule::top_level_const_initializers)
            .unwrap_or_default();
        self.semantic_records = resolved
            .semantic_module
            .as_ref()
            .map(|module| module.records.clone())
            .unwrap_or_default();
        self.semantic_top_level_dims = resolved
            .semantic_module
            .as_ref()
            .map(crate::semantic_ir::SemanticModule::top_level_dim_declarations)
            .unwrap_or_default();
        let record_buffer_names = resolved
            .semantic_module
            .as_ref()
            .map(semantic_record_buffer_names)
            .unwrap_or_else(|| resolved.record_buffer_names.clone());
        let mut functions = Vec::new();
        for f in &program.functions {
            let capacities = param_capacities
                .get(&f.name.name.to_ascii_lowercase())
                .cloned()
                .unwrap_or_default();
            let semantic_callable = resolved
                .semantic_module
                .as_ref()
                .and_then(|module| semantic_basic_callable_for_function(module, f));
            let semantic_array_ranks =
                semantic_callable.map(crate::semantic_ir::CallableSignature::array_ranks);
            let semantic_param_ranks = semantic_callable.map(|callable| {
                let (ranks, observed) = callable.parameter_array_ranks();
                for (index, uses) in observed.iter().enumerate() {
                    let parameter = &callable.parameters[index];
                    let position = semantic_callable_diagnostic_pos(
                        resolved.semantic_module.as_ref(),
                        semantic_callable,
                        parameter.span,
                    );
                    if uses.len() > 1 {
                        self.diagnostics.push(Diagnostic::error(
                            position.clone(),
                            format!(
                                "parameter `{}` of `{}` is indexed with different numbers of subscripts in different places -- BASCAL can't tell how many dimensions it has",
                                parameter.name, callable.name
                            ),
                        ));
                    } else if let Some(used) = uses.first() {
                        match parameter.array_axes {
                            0 => self.diagnostics.push(Diagnostic::error(
                                position.clone(),
                                format!(
                                    "parameter `{}` of `{}` is indexed as an array, but its declaration doesn't say so. Give it an explicit rank, e.g. `{}(?)`",
                                    parameter.name, callable.name, parameter.name
                                ),
                            )),
                            declared if declared != *used => self.diagnostics.push(Diagnostic::error(
                                position.clone(),
                                format!(
                                    "parameter `{}` of `{}` is declared with {declared} dimensions but indexed with {used} subscript{} in the body",
                                    parameter.name, callable.name, if *used == 1 { "" } else { "s" }
                                ),
                            )),
                            _ => {}
                        }
                    }
                }
                ranks
            });
            let semantic_globals = semantic_callable.map(|callable| {
                let mut globals: HashSet<String> =
                    crate::semantic_ir::SemanticModule::global_declarations_in(&callable.body)
                        .into_iter()
                        .collect();
                globals.extend(semantic_record_storage.iter().cloned());
                globals
            });
            let mut function_info = FunctionInfo::from_def(
                f,
                &mut taken,
                &known_callables,
                &mut self.diagnostics,
                capacities,
                semantic_param_ranks,
                semantic_array_ranks,
                semantic_globals,
                semantic_callable.map(|callable| callable.parameters.clone()),
                semantic_callable,
                resolved.semantic_module.as_ref(),
            );
            if let Some(callable) = semantic_callable {
                function_info.semantic_const_initializers = callable.const_initializers();
                function_info.semantic_dim_declarations = callable.dim_declarations();
            }
            functions.push(function_info);
        }
        self.functions = functions;
        self.error_handler_procedures = if let Some(module) = resolved.semantic_module.as_ref() {
            module
                .error_handler_targets()
                .into_iter()
                .map(|name| name.to_ascii_lowercase())
                .collect()
        } else {
            resolved.error_handler_procedures.clone()
        };
        self.top_level_array_ranks = top_level_array_ranks;
        self.record_buffer_names = record_buffer_names;
        if let Some(module) = resolved.semantic_module.as_ref() {
            for (name, value_type) in module.const_types() {
                let ident = BasicIdent::parse(&name);
                let Some(suffix) = semantic_const_suffix(value_type) else {
                    continue;
                };
                let generated = BasicIdent {
                    name: const_var_name(&ident.name),
                    suffix: Some(suffix),
                }
                .as_basic();
                taken.insert(generated.to_ascii_lowercase());
                self.const_var_names
                    .insert(ident.name.to_ascii_lowercase(), generated);
            }
        } else {
            // Compatibility path for callers that construct a resolved
            // legacy AST without a semantic module.  Recover only the
            // binding names/types from the AST declaration walk; normal
            // driver compilation uses the semantic branch above.
            let mut legacy_consts = HashMap::new();
            collect_consts(&program.statements, &mut legacy_consts);
            for function in &program.functions {
                collect_consts(&function.body, &mut legacy_consts);
            }
            for name in legacy_consts.keys() {
                let ident = BasicIdent::parse(name);
                let generated = BasicIdent {
                    name: const_var_name(&ident.name),
                    suffix: ident.suffix,
                }
                .as_basic();
                taken.insert(generated.to_ascii_lowercase());
                self.const_var_names
                    .insert(ident.name.to_ascii_lowercase(), generated);
            }
        }
        *self.taken_names.borrow_mut() = taken;

        self.known_callables = self
            .functions
            .iter()
            .map(|f| f.source_name.name.to_ascii_lowercase())
            .chain(BASIC_BUILTINS.iter().map(|s| s.to_string()))
            .collect();

        self.line("' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE");
        self.line("' Functions are transpiled to global variables, labels, and GOSUB");

        for block in &resolved.common_blocks {
            let vars = block
                .vars
                .iter()
                .map(|v| {
                    if v.is_array {
                        format!("{}()", v.name.as_basic())
                    } else {
                        v.name.as_basic()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            self.line(&format!("COMMON {vars}"));
        }

        if let Some(module) = resolved.semantic_module.as_ref() {
            let unresolved = module
                .dependencies
                .iter()
                .filter(|dependency| !dependency.resolved)
                .collect::<Vec<_>>();
            if !unresolved.is_empty() {
                self.line("' TODO: resolve BASCAL dependency selectors during link");
                for dependency in unresolved {
                    match dependency.kind {
                        crate::semantic_ir::DependencyKind::Require => {
                            self.line(&format!("' require {}", dependency.path))
                        }
                        crate::semantic_ir::DependencyKind::Import => {
                            self.line(&format!("' import {} (alias for require)", dependency.path))
                        }
                    }
                }
            }
        } else if !program.declarations.is_empty() {
            self.line("' TODO: resolve BASCAL dependency selectors during link");
            for declaration in &program.declarations {
                match declaration {
                    DependencyDecl::Require(symbol) => {
                        self.line(&format!("' require {}", symbol.raw))
                    }
                    DependencyDecl::Import(symbol) => {
                        self.line(&format!("' import {} (alias for require)", symbol.raw))
                    }
                }
            }
        }

        self.emit_array_param_storage_dims();

        // Prefer whole-stream typed emission: it has no AST alignment step
        // and lets the semantic IR own statement order and structure. The
        // aligned dispatcher remains a compatibility bridge for streams
        // containing nodes the typed emitter has not migrated yet. Record DSL
        // expansions are one such bridge until their GET/PUT transformations
        // are represented in typed IR.
        let record_stream_is_typed = resolved
            .semantic_module
            .as_ref()
            .is_some_and(|module| module.records_transpiled)
            || !has_untyped_lowered_record_operations(
            &program.statements,
            resolved
                .semantic_module
                .as_ref()
                .map(|module| module.lowered_record_files.as_slice())
                .unwrap_or_default(),
        );
        let semantic_try_stream_is_typed = resolved
            .semantic_module
            .as_ref()
            .is_some_and(|module| basic_semantic_try_stream_is_typed(module));
        let semantic_intrinsics = if record_stream_is_typed {
            resolved.semantic_module.as_ref().and_then(|module| {
                basic_semantic_intrinsics(
                    &mut self,
                    module,
                    &module.statements,
                    semantic_try_stream_is_typed,
                )
            })
        } else {
            None
        };
        let source_dispatch = if semantic_intrinsics.is_none() && record_stream_is_typed {
            resolved.semantic_module.as_ref().and_then(|module| {
                basic_semantic_statements_by_source(&mut self, module, &program.statements)
            })
        } else {
            None
        };
        if let Some(intrinsics) = semantic_intrinsics {
            self.blank();
            for intrinsic in intrinsics {
                self.line(&intrinsic);
            }
        } else if source_dispatch.is_some() {
            // The aligned dispatcher emitted each semantic or compatibility
            // statement in source order.
        } else if !program.statements.is_empty() {
            self.blank();
            self.statements(&program.statements, None);
        }

        if !program.functions.is_empty() {
            if !resolved
                .semantic_module
                .as_ref()
                .map(crate::semantic_ir::SemanticModule::ends_with_end)
                .unwrap_or_else(|| ends_with_end(&program.statements))
            {
                self.line("END");
            }
            for function in &program.functions {
                self.function(function, resolved.semantic_module.as_ref());
            }
        }
        for message in self.deferred_errors.take() {
            if !self
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message == message)
            {
                self.diagnostics.push(Diagnostic::error(
                    SourcePos::new("<validation>", 1, 1),
                    message,
                ));
            }
        }
        if !self.diagnostics.is_empty() {
            return Err(self.diagnostics);
        }
        if self.needs_source_lookup {
            // A first numbering pass, over the program exactly as built so
            // far, to learn which final line-number ranges belong to which
            // original `.bcl` file -- see `number_basic_lines`'s own doc
            // comment. Its rendered text is discarded: appending the
            // lookup subroutine strictly after everything already emitted
            // cannot change any number this pass already assigned (see
            // `emit_source_file_lookup_subroutine`'s doc comment), so a
            // second, real pass over the augmented program produces
            // identical numbers for every line that mattered here.
            let (_, breakpoints) = number_basic_lines(&self.output, self.line_numbers);
            if program.functions.is_empty()
                && !resolved
                    .semantic_module
                    .as_ref()
                    .map(crate::semantic_ir::SemanticModule::ends_with_end)
                    .unwrap_or_else(|| ends_with_end(&program.statements))
            {
                self.line("END");
            }
            self.emit_source_file_lookup_subroutine(&breakpoints);
        }
        Ok(number_basic_lines(&self.output, self.line_numbers).0)
    }

    /// Emits the one-time, top-level `DIM` for every array parameter's
    /// shared storage, sized to its resolved capacity (see
    /// `infer_array_param_capacities`). Must run exactly once per program,
    /// before any call -- classic BASIC has no `REDIM`, so this is the
    /// only `DIM` these storage arrays ever get; `call_lines` no longer
    /// DIMs them per call site.
    fn emit_array_param_storage_dims(&mut self) {
        let lines: Vec<String> = self
            .functions
            .iter()
            .flat_map(|info| {
                info.params
                    .iter()
                    .enumerate()
                    .filter_map(move |(index, (_, lowered))| {
                        info.param_ranks.get(index).copied().flatten()?;
                        let capacities = info.param_capacities.get(index)?;
                        if capacities.is_empty() {
                            return None;
                        }
                        let bounds = capacities
                            .iter()
                            .map(|c| c.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        Some(format!("DIM {}({bounds})", lowered.as_basic()))
                    })
            })
            .collect();
        if lines.is_empty() {
            return;
        }
        self.blank();
        self.line("' Storage for array parameters, sized to fit every call site");
        self.lines(lines);
    }

    fn function(
        &mut self,
        function: &FunctionDef,
        semantic_module: Option<&crate::semantic_ir::SemanticModule>,
    ) {
        let info = self
            .function_info(&function.name)
            .expect("function table should contain every function")
            .clone();
        let params = info
            .params
            .iter()
            .map(|(parameter, _)| parameter.name.as_basic())
            .collect::<Vec<_>>()
            .join(", ");
        let lowered_name = info.source_name.as_basic().to_ascii_lowercase();
        let kind = if info.is_procedure {
            "procedure"
        } else {
            "function"
        };
        self.blank();
        self.line(&format!("' {kind} {}({})", lowered_name, params));
        self.line(&format!("{}:", info.label));
        self.indent += 1;
        let semantic_dispatch = semantic_module
            .and_then(|module| basic_semantic_callable_statements_by_source(module, function));
        let semantic_source_filename = semantic_module
            .and_then(|module| semantic_basic_callable_for_function(module, function))
            .and_then(|callable| semantic_module?.sources.get(callable.source_index))
            .map(|source| source.filename.clone());
        // Prefer the callable's whole typed body: it needs no one-to-one AST
        // alignment (a `dim a, b` is one typed statement but several AST
        // ones, and an expanded record statement is many). If any statement is
        // unsupported, fall back to per-statement dispatch, then the AST body.
        let whole_body_lines = semantic_module
            .and_then(|module| semantic_basic_callable_for_function(module, function))
            .and_then(|callable| {
                let diagnostics = self.diagnostics.len();
                let mut lines = Vec::new();
                for statement in &callable.body {
                    match basic_semantic_callable_statement(self, statement, &info) {
                        Some(statement_lines) => lines.extend(statement_lines),
                        None => {
                            self.diagnostics.truncate(diagnostics);
                            return None;
                        }
                    }
                }
                Some(lines)
            });
        let function_body: &[Stmt] = if whole_body_lines.is_some() {
            &[]
        } else {
            &function.body
        };
        if let Some(lines) = whole_body_lines {
            if let Some(filename) = semantic_source_filename.as_deref() {
                if self.needs_source_lookup && self.current_marker_file.as_deref() != Some(filename)
                {
                    self.current_marker_file = Some(filename.to_string());
                    self.line(&source_file_marker(
                        &crate::diagnostics::display_source_filename(filename),
                    ));
                }
            }
            for line in lines {
                self.line(&line);
            }
        }
        for (index, statement) in function_body.iter().enumerate() {
            let semantic_statement = semantic_dispatch
                .as_ref()
                .and_then(|items| items.get(index).copied().flatten());
            let semantic_lines = semantic_dispatch
                .as_ref()
                .and_then(|items| items.get(index).copied().flatten())
                .and_then(|semantic| basic_semantic_callable_statement(self, semantic, &info));
            if let Some(lines) = semantic_lines {
                let filename = semantic_statement
                    .and(semantic_source_filename.as_deref())
                    .unwrap_or(&statement.pos.filename);
                if self.needs_source_lookup && self.current_marker_file.as_deref() != Some(filename)
                {
                    self.current_marker_file = Some(filename.to_string());
                    self.line(&source_file_marker(
                        &crate::diagnostics::display_source_filename(filename),
                    ));
                }
                for line in lines {
                    self.line(&line);
                }
            } else {
                self.statement(statement, Some(&info));
            }
        }
        // A procedure named as an `on error goto` target is entered via a
        // raw GOTO, never a GOSUB -- resolver::validate has already proven
        // it contains no `return` and never falls off the end, so the
        // usual implicit trailing RETURN below would be both unreachable
        // and, if that proof were ever wrong, a "RETURN without GOSUB"
        // crash. Skip it entirely for these, same as a raw label.
        let is_unreturnable_error_handler = info.is_procedure
            && self
                .error_handler_procedures
                .contains(&info.source_name.name.to_ascii_lowercase());
        let ends_with_return = semantic_module
            .as_ref()
            .and_then(|module| {
                semantic_basic_callable_for_function(module, function)
                    .map(|callable| ends_with_semantic_return(&callable.body))
            })
            .unwrap_or_else(|| {
                semantic_dispatch.as_ref().map_or_else(
                    || ends_with_return(&function.body),
                    |statements| ends_with_emitted_return(&function.body, statements),
                )
            });
        if !ends_with_return && !is_unreturnable_error_handler {
            self.line("RETURN");
        }
        self.indent -= 1;
        self.line(&format!("' end {kind} {}", lowered_name));
    }

    fn statements(&mut self, statements: &[Stmt], current_function: Option<&FunctionInfo>) {
        for statement in statements {
            self.statement(statement, current_function);
        }
    }

    fn statement(&mut self, statement: &Stmt, current_function: Option<&FunctionInfo>) {
        if self.needs_source_lookup
            && self.current_marker_file.as_deref() != Some(statement.pos.filename.as_str())
        {
            self.current_marker_file = Some(statement.pos.filename.clone());
            self.line(&source_file_marker(
                &crate::diagnostics::display_source_filename(&statement.pos.filename),
            ));
        }
        match &statement.kind {
            Statement::Dim {
                name,
                is_array,
                sizes,
            } => {
                let base = self.ident(name, current_function);
                let key = name.as_basic().to_ascii_lowercase();
                let semantic_declaration = current_function
                    .and_then(|function| function.semantic_dim_declarations.get(&key))
                    .or_else(|| {
                        current_function
                            .is_none()
                            .then(|| self.semantic_top_level_dims.get(&key))
                            .flatten()
                    });
                let semantic_axes = semantic_declaration.and_then(|declaration| {
                    if declaration
                        .dimensions
                        .iter()
                        .any(|axis| matches!(axis, crate::semantic_ir::DimAxis::Inferred))
                    {
                        return None;
                    }
                    declaration
                        .dimensions
                        .iter()
                        .map(|axis| match axis {
                            crate::semantic_ir::DimAxis::Fixed(value) => {
                                let literal = value.chars().next().is_some_and(|character| {
                                    character.is_ascii_digit()
                                        || character == '&'
                                        || character == '"'
                                });
                                let rendered = if literal {
                                    value.clone()
                                } else {
                                    self.ident(&BasicIdent::parse(value), None)
                                };
                                Some((rendered, literal))
                            }
                            crate::semantic_ir::DimAxis::Expression(expression) => self
                                .semantic_const_expression(expression, current_function)
                                .map(|value| {
                                    let literal = matches!(
                                        expression.kind,
                                        crate::semantic_ir::ExpressionKind::Literal(_)
                                    );
                                    (value, literal)
                                }),
                            crate::semantic_ir::DimAxis::Inferred => None,
                        })
                        .collect::<Option<Vec<_>>>()
                });
                if let (Some(declaration), Some(axes)) = (semantic_declaration, semantic_axes) {
                    let type_clause = name
                        .suffix
                        .is_none()
                        .then(|| semantic_dim_type_name(declaration.element_type))
                        .flatten()
                        .map(|value| format!(" AS {value}"))
                        .unwrap_or_default();
                    if declaration.array_axes == 0 {
                        self.line(&format!("DIM {base}{type_clause}"));
                    } else if axes.is_empty() {
                        self.line(&format!("DIM {base}(){type_clause}"));
                    } else {
                        let rendered = axes
                            .iter()
                            .map(|(value, _)| value.clone())
                            .collect::<Vec<_>>();
                        self.line(&format!("DIM {base}({}){type_clause}", rendered.join(", ")));
                        let frozen = axes
                            .into_iter()
                            .map(|(value, literal)| {
                                if literal {
                                    value
                                } else {
                                    let temp = self.next_temp_var();
                                    self.line(&format!("{temp} = {value}"));
                                    temp
                                }
                            })
                            .collect();
                        if let Some(function) = current_function {
                            function.local_array_bounds.borrow_mut().insert(key, frozen);
                        } else {
                            self.top_level_array_bounds.insert(key, frozen);
                        }
                    }
                    return;
                }
                let key = name.as_basic().to_ascii_lowercase();
                let declaration = current_function
                    .and_then(|function| function.semantic_dim_declarations.get(&key))
                    .or_else(|| self.semantic_top_level_dims.get(&key));
                let type_clause = if name.suffix.is_none() {
                    declaration
                        .and_then(|declaration| {
                            semantic_dim_type_name(declaration.element_type)
                        })
                        .map(|value| format!(" AS {value}"))
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                if sizes.is_empty() {
                    if *is_array {
                        self.line(&format!("DIM {base}(){type_clause}"));
                    } else {
                        self.line(&format!("DIM {base}{type_clause}"));
                    }
                } else {
                    let mut rendered = Vec::new();
                    for s in sizes {
                        let (prelude, val) = self.expr(s, current_function);
                        self.lines(prelude);
                        rendered.push(val);
                    }
                    self.line(&format!("DIM {base}({}){type_clause}", rendered.join(", ")));

                    // Every DIMed array's bounds get frozen here -- a
                    // literal bound needs no extra line, but a non-literal
                    // one is captured into a temp so it's still readable
                    // later (by `sizeof()`, or by the auto-injected bound
                    // passed to a function this array gets passed into).
                    let key = name.as_basic().to_ascii_lowercase();
                    let frozen: Vec<String> = sizes
                        .iter()
                        .zip(rendered.iter())
                        .map(|(s, val)| {
                            if matches!(s, Expr::Integer(_)) {
                                val.clone()
                            } else {
                                let temp = self.next_temp_var();
                                self.line(&format!("{temp} = {val}"));
                                temp
                            }
                        })
                        .collect();
                    if let Some(info) = current_function {
                        info.local_array_bounds.borrow_mut().insert(key, frozen);
                    } else {
                        self.top_level_array_bounds.insert(key, frozen);
                    }
                }
            }
            Statement::Open {
                mode,
                file,
                channel,
                len,
            } => {
                let (file_prelude, file) = self.expr(file, current_function);
                let (channel_prelude, channel) = self.expr(channel, current_function);
                self.lines(file_prelude);
                self.lines(channel_prelude);
                let mode_str = match mode {
                    OpenMode::Input => "INPUT",
                    OpenMode::Output => "OUTPUT",
                    OpenMode::Append => "APPEND",
                    OpenMode::Random => "RANDOM",
                    OpenMode::Binary => "BINARY",
                };
                let len_clause = if let Some(len_expr) = len {
                    let (len_pre, len_val) = self.expr(len_expr, current_function);
                    self.lines(len_pre);
                    format!(" LEN = {len_val}")
                } else {
                    String::new()
                };
                self.line(&format!(
                    "OPEN {file} FOR {mode_str} AS #{channel}{len_clause}"
                ));
            }
            Statement::FileDecl { .. } => {
                unreachable!("record/file DSL must be lowered before codegen")
            }
            Statement::LineInput { channel, target } => {
                let (channel_prelude, channel) = self.expr(channel, current_function);
                let (target_prelude, target) = self.expr(target, current_function);
                self.lines(channel_prelude);
                self.lines(target_prelude);
                self.line(&format!("LINE INPUT #{channel}, {target}"));
            }
            Statement::PrintFile { channel, tokens } => {
                let (channel_prelude, channel) = self.expr(channel, current_function);
                self.lines(channel_prelude);
                let body = self.render_print_tokens(tokens, current_function);
                if body.is_empty() {
                    self.line(&format!("PRINT #{channel}"));
                } else {
                    self.line(&format!("PRINT #{channel}, {body}"));
                }
            }
            Statement::Close { channel } => {
                let (channel_prelude, channel) = self.expr(channel, current_function);
                self.lines(channel_prelude);
                self.line(&format!("CLOSE #{channel}"));
            }
            Statement::Kill { file } => {
                let (prelude, file) = self.expr(file, current_function);
                self.lines(prelude);
                self.line(&format!("KILL {file}"));
            }
            Statement::Name { from, to } => {
                let (from_prelude, from) = self.expr(from, current_function);
                let (to_prelude, to) = self.expr(to, current_function);
                self.lines(from_prelude);
                self.lines(to_prelude);
                self.line(&format!("NAME {from} AS {to}"));
            }
            Statement::Assignment { target, value } => {
                let (target_prelude, target) = self.expr(target, current_function);
                let (value_prelude, value) = self.expr(value, current_function);
                self.lines(target_prelude);
                self.lines(value_prelude);
                self.line(&format!("{target} = {value}"));
            }
            Statement::MidAssign {
                target,
                start,
                len,
                value,
            } => {
                // Preserve left-to-right evaluation and snapshot every
                // value before later operands can mutate it. Array indices
                // already arrive in `target_prelude` as single-evaluation
                // temporaries; reusing target_text preserves that lvalue.
                let (target_prelude, target_text) = self.expr(target, current_function);
                let (start_prelude, start_text) = self.expr(start, current_function);
                let len_rendered = len.as_ref().map(|e| self.expr(e, current_function));
                let (value_prelude, value_text) = self.expr(value, current_function);
                self.lines(target_prelude);
                let target_value = self.next_temp_var_suffixed("$");
                self.line(&format!("{target_value} = {target_text}"));
                self.lines(start_prelude);
                let start_value = self.next_temp_var();
                self.line(&format!("{start_value} = {start_text}"));
                let len_text = match len_rendered {
                    Some((len_prelude, len_text)) => {
                        self.lines(len_prelude);
                        let len_value = self.next_temp_var();
                        self.line(&format!("{len_value} = {len_text}"));
                        Some(len_value)
                    }
                    None => None,
                };
                self.lines(value_prelude);
                let replacement = self.next_temp_var_suffixed("$");
                self.line(&format!("{replacement} = {value_text}"));
                let length_value = len_text.unwrap_or_else(|| format!("LEN({replacement})"));

                let label = self.next_label;
                self.next_label += 1;
                let trim_label = format!("MID_{label:04}_TRIM");
                let done_label = format!("MID_{label:04}_DONE");
                self.line(&format!(
                    "IF LEN({replacement}) > {length_value} THEN GOTO {trim_label}"
                ));
                self.line(&format!("GOTO {done_label}"));
                self.line(&format!("{trim_label}:"));
                self.line(&format!(
                    "{replacement} = LEFT$({replacement}, {length_value})"
                ));
                self.line(&format!("{done_label}:"));
                self.line(&format!(
                    "{target_text} = LEFT$({target_value}, {start_value} - 1) + {replacement} + MID$({target_value}, {start_value} + LEN({replacement}))"
                ));
            }
            Statement::Print { tokens } => {
                let body = self.render_print_tokens(tokens, current_function);
                if body.is_empty() {
                    self.line("PRINT");
                } else {
                    self.line(&format!("PRINT {body}"));
                }
            }
            Statement::PrintUsing { format, tokens } => {
                let (fmt_pre, fmt_str) = self.expr(format, current_function);
                self.lines(fmt_pre);
                let body = self.render_print_tokens(tokens, current_function);
                if body.is_empty() {
                    self.line(&format!("PRINT USING {fmt_str}"));
                } else {
                    self.line(&format!("PRINT USING {fmt_str}; {body}"));
                }
            }
            Statement::PrintFileUsing {
                channel,
                format,
                tokens,
            } => {
                let (ch_pre, ch) = self.expr(channel, current_function);
                let (fmt_pre, fmt_str) = self.expr(format, current_function);
                self.lines(ch_pre);
                self.lines(fmt_pre);
                let body = self.render_print_tokens(tokens, current_function);
                if body.is_empty() {
                    self.line(&format!("PRINT #{ch}, USING {fmt_str}"));
                } else {
                    self.line(&format!("PRINT #{ch}, USING {fmt_str}; {body}"));
                }
            }
            Statement::ReturnVoid => {
                self.line("RETURN");
            }
            Statement::Return { value } => {
                let Some(info) = current_function else {
                    let (prelude, value) = self.expr(value, current_function);
                    self.lines(prelude);
                    self.line(&format!("RETURN {}", value));
                    return;
                };
                if info.is_procedure {
                    self.line("RETURN");
                } else {
                    let (prelude, value) = self.expr(value, current_function);
                    self.lines(prelude);
                    self.line(&format!("{} = {}", info.result.as_basic(), value));
                    self.line("RETURN");
                }
            }
            Statement::If {
                condition,
                then_body,
                else_body,
            } => self.if_statement(condition, then_body, else_body, current_function),
            Statement::For {
                var,
                start,
                end,
                step,
                body,
            } => {
                let (start_prelude, start) = self.expr(start, current_function);
                let (end_prelude, end) = self.expr(end, current_function);
                let step = step.as_ref().map(|step| self.expr(step, current_function));
                self.lines(start_prelude);
                self.lines(end_prelude);
                let step = if let Some((step_prelude, step)) = step {
                    self.lines(step_prelude);
                    format!(" STEP {step}")
                } else {
                    String::new()
                };
                self.line(&format!(
                    "FOR {} = {start} TO {end}{step}",
                    self.ident(var, current_function)
                ));
                self.indent += 1;
                // BASIC has no native "skip straight to the increment"
                // construct the way it has `EXIT FOR`, so even a `for`
                // loop's own `continue` target needs a real label -- placed
                // right before `NEXT`, so jumping there runs the increment
                // and loop condition exactly like falling off the end of
                // the body normally would.
                let continue_id = self.next_label;
                self.next_label += 1;
                let continue_label = format!("FOR_{continue_id:04}_CONTINUE");
                self.loop_exit_stack.push(LoopExit::NativeFor);
                self.loop_continue_stack.push(continue_label.clone());
                self.statements(body, current_function);
                self.loop_continue_stack.pop();
                self.loop_exit_stack.pop();
                self.indent -= 1;
                self.line(&format!("{continue_label}:"));
                self.line(&format!("NEXT {}", self.ident(var, current_function)));
            }
            Statement::While { condition, body } => {
                let id = self.next_label;
                self.next_label += 1;
                let top_label = format!("WHILE_{id:04}_TOP");
                let end_label = format!("WHILE_{id:04}_END");
                self.line(&format!("{top_label}:"));
                self.condition_jump(condition, &end_label, false, current_function);
                self.indent += 1;
                self.loop_exit_stack.push(LoopExit::Goto(end_label.clone()));
                // A `while` loop's own per-iteration bookkeeping *is*
                // re-checking the condition, which is exactly what
                // `top_label` already does -- no separate continue label
                // needed, unlike `for`/`do`.
                self.loop_continue_stack.push(top_label.clone());
                self.statements(body, current_function);
                self.loop_continue_stack.pop();
                self.loop_exit_stack.pop();
                self.line(&format!("GOTO {top_label}"));
                self.indent -= 1;
                self.line(&format!("{end_label}:"));
                self.line("REM END WHILE");
            }
            Statement::Do {
                condition,
                body,
                post_condition,
            } => {
                let id = self.next_label;
                self.next_label += 1;
                let top_label = format!("DO_{id:04}_TOP");
                let end_label = format!("DO_{id:04}_END");
                let continue_label = format!("DO_{id:04}_CONTINUE");
                self.line(&format!("{top_label}:"));
                if let Some(cond) = condition {
                    // is_while -> exit when false (invert=false); is_until -> exit when true (invert=true).
                    self.condition_jump(&cond.expr, &end_label, !cond.is_while, current_function);
                }
                self.indent += 1;
                self.loop_exit_stack.push(LoopExit::Goto(end_label.clone()));
                // `continue_label` sits right after the body, before
                // whatever decides to repeat -- the post-condition check
                // below if there is one, or the unconditional GOTO back to
                // `top_label` otherwise. Jumping straight to `top_label`
                // instead would skip a post-condition check entirely,
                // silently turning `do ... loop until done` into an
                // infinite loop on `continue`.
                self.loop_continue_stack.push(continue_label.clone());
                self.statements(body, current_function);
                self.loop_continue_stack.pop();
                self.loop_exit_stack.pop();
                self.line(&format!("{continue_label}:"));
                if let Some(cond) = post_condition {
                    // is_while -> repeat when true (invert=true); is_until -> repeat when false (invert=false).
                    self.condition_jump(&cond.expr, &top_label, cond.is_while, current_function);
                } else {
                    // No post-condition: loop back to re-check the pre-condition (if
                    // any) or repeat unconditionally (bare `do ... end do`, relying on
                    // `exit do` to leave). Previously this only fired for a bare
                    // `do`, so a pre-condition-only `do while`/`do until` loop never
                    // actually looped — it ran the body at most once and fell
                    // through, regardless of the condition.
                    self.line(&format!("GOTO {top_label}"));
                }
                self.indent -= 1;
                self.line(&format!("{end_label}:"));
                self.line("REM END DO");
            }
            Statement::ExprStmt(expr_stmt) => self.expr_statement(expr_stmt, current_function),
            Statement::End => self.line("END"),
            Statement::Stop => self.line("STOP"),
            Statement::Cls => self.line("CLS"),
            Statement::Beep => self.line("BEEP"),
            Statement::System => self.line("SYSTEM"),
            Statement::OptionBase(expr) => {
                let (prelude, base) = self.expr(expr, current_function);
                self.lines(prelude);
                self.line(&format!("OPTION BASE {base}"));
            }
            Statement::Erase(vars) => {
                let names: Vec<String> = vars
                    .iter()
                    .map(|v| self.ident(v, current_function))
                    .collect();
                self.line(&format!("ERASE {}", names.join(", ")));
            }
            Statement::Randomize(expr) => {
                if let Some(expr) = expr {
                    let (prelude, expr) = self.expr(expr, current_function);
                    self.lines(prelude);
                    self.line(&format!("RANDOMIZE {expr}"));
                } else {
                    self.line("RANDOMIZE");
                }
            }
            Statement::Swap(a, b) => {
                let (a_prelude, a) = self.expr(a, current_function);
                let (b_prelude, b) = self.expr(b, current_function);
                self.lines(a_prelude);
                self.lines(b_prelude);
                self.line(&format!("SWAP {a}, {b}"));
            }
            Statement::Poke { address, value } => {
                let (addr_prelude, addr) = self.expr(address, current_function);
                let (val_prelude, val) = self.expr(value, current_function);
                self.lines(addr_prelude);
                self.lines(val_prelude);
                self.line(&format!("POKE {addr}, {val}"));
            }
            Statement::Out { port, value } => {
                let (port_prelude, port) = self.expr(port, current_function);
                let (val_prelude, val) = self.expr(value, current_function);
                self.lines(port_prelude);
                self.lines(val_prelude);
                self.line(&format!("OUT {port}, {val}"));
            }
            Statement::Width { channel, cols } => match channel {
                Some(ch) => {
                    let (ch_prelude, ch_s) = self.expr(ch, current_function);
                    self.lines(ch_prelude);
                    let (cols_prelude, cols_s) = self.expr(cols, current_function);
                    self.lines(cols_prelude);
                    self.line(&format!("WIDTH #{ch_s}, {cols_s}"));
                }
                None => {
                    let (cols_prelude, cols_s) = self.expr(cols, current_function);
                    self.lines(cols_prelude);
                    self.line(&format!("WIDTH {cols_s}"));
                }
            },
            Statement::Clear => {
                self.line("CLEAR");
            }
            Statement::Label(name) => {
                self.line(&format!("{}:", user_label_token(name)));
            }
            Statement::Goto(target) => {
                self.line(&format!("GOTO {}", self.label_target_text(target)));
            }
            Statement::Gosub(target) => {
                self.line(&format!("GOSUB {}", self.label_target_text(target)));
            }
            Statement::OnErrorGoto { target } => {
                self.line(&format!("ON ERROR GOTO {}", self.label_target_text(target)));
            }
            Statement::Resume(kind) => match kind {
                ResumeTarget::Same => self.line("RESUME"),
                ResumeTarget::Next => self.line("RESUME NEXT"),
                ResumeTarget::Line(expr) => {
                    self.line(&format!("RESUME {}", self.label_target_text(expr)));
                }
            },
            Statement::ErrorStmt { code } => {
                let (prelude, code) = self.expr(code, current_function);
                self.lines(prelude);
                self.line(&format!("ERROR {code}"));
            }
            Statement::ThrowStmt { code } => {
                let (prelude, code) = match code {
                    Some(code) => self.expr(code, current_function),
                    None => (Vec::new(), "ERR".to_string()),
                };
                self.lines(prelude);
                self.line(&format!("ERROR {code}"));
            }
            Statement::Input { prompt, vars } => {
                let mut rendered = Vec::new();
                for var in vars {
                    let (prelude, var) = self.expr(var, current_function);
                    self.lines(prelude);
                    rendered.push(var);
                }
                let prompt_part = match prompt {
                    Some(p) => format!("\"{p}\"; "),
                    None => String::new(),
                };
                self.line(&format!("INPUT {}{}", prompt_part, rendered.join(", ")));
            }
            Statement::InputFile { channel, vars } => {
                let (channel_prelude, channel) = self.expr(channel, current_function);
                self.lines(channel_prelude);
                let mut rendered = Vec::new();
                for var in vars {
                    let (prelude, var) = self.expr(var, current_function);
                    self.lines(prelude);
                    rendered.push(var);
                }
                self.line(&format!("INPUT #{channel}, {}", rendered.join(", ")));
            }
            Statement::Data(values) => {
                let mut rendered = Vec::new();
                for val in values {
                    let (prelude, val) = self.expr(val, current_function);
                    self.lines(prelude);
                    rendered.push(val);
                }
                self.line(&format!("DATA {}", rendered.join(", ")));
            }
            Statement::Read(vars) => {
                let mut rendered = Vec::new();
                for var in vars {
                    let (prelude, var) = self.expr(var, current_function);
                    self.lines(prelude);
                    rendered.push(var);
                }
                self.line(&format!("READ {}", rendered.join(", ")));
            }
            Statement::Restore(target) => {
                if let Some(target) = target {
                    self.line(&format!("RESTORE {}", self.label_target_text(target)));
                } else {
                    self.line("RESTORE");
                }
            }
            Statement::Const { name, value } => {
                // Real MBASIC/BASCOM has no CONST statement at all (a
                // QuickBASIC-era addition) -- a plain assignment to the
                // const's generated name (see `const_var_name`/`ident()`)
                // is the only thing that compiles there. `const` in .bcl
                // source is purely a naming/intent signal to the reader;
                // nothing in generated BASIC needs to express "this
                // shouldn't be reassigned" for it to behave correctly.
                let key = name.as_basic().to_ascii_lowercase();
                // Semantic names preserve source spelling without the BASIC
                // suffix synthesized for an untyped AST const declaration.
                let semantic_key = name.name.to_ascii_lowercase();
                let semantic_value = current_function
                    .and_then(|function| {
                        function
                            .semantic_const_initializers
                            .get(&key)
                            .or_else(|| function.semantic_const_initializers.get(&semantic_key))
                    })
                    .or_else(|| {
                        current_function
                            .is_none()
                            .then(|| {
                                self.semantic_const_initializers
                                    .get(&key)
                                    .or_else(|| self.semantic_const_initializers.get(&semantic_key))
                            })
                            .flatten()
                    })
                    .and_then(|expression| {
                        self.semantic_const_expression(expression, current_function)
                    });
                if let Some(value) = semantic_value {
                    self.line(&format!("{} = {value}", self.ident(name, current_function)));
                } else {
                    let (prelude, value) = self.expr(value, current_function);
                    self.lines(prelude);
                    self.line(&format!("{} = {value}", self.ident(name, current_function)));
                }
            }
            Statement::Write { channel, exprs } => {
                let (channel_prelude, channel) = self.expr(channel, current_function);
                self.lines(channel_prelude);
                let mut rendered = Vec::new();
                for item in exprs {
                    let (prelude, item) = self.expr(item, current_function);
                    self.lines(prelude);
                    rendered.push(item);
                }
                self.line(&format!("WRITE #{channel}, {}", rendered.join(", ")));
            }
            Statement::Field {
                channel, fields, ..
            } => {
                let (ch_pre, ch) = self.expr(channel, current_function);
                self.lines(ch_pre);
                let mut parts = Vec::new();
                for (width, var) in fields {
                    let (w_pre, w) = self.expr(width, current_function);
                    self.lines(w_pre);
                    parts.push(format!("{w} AS {}", self.ident(var, current_function)));
                }
                self.line(&format!("FIELD #{ch}, {}", parts.join(", ")));
            }
            Statement::Get {
                channel,
                record,
                var,
                require_existing,
                record_length,
            } => {
                let (ch_pre, ch) = self.expr(channel, current_function);
                self.lines(ch_pre);
                match (record, var) {
                    (None, None) => self.line(&format!("GET #{ch}")),
                    (Some(rec), None) => {
                        let (r_pre, r) = self.expr(rec, current_function);
                        self.lines(r_pre);
                        if *require_existing {
                            let length = record_length
                                .expect("DSL partial update must carry its record length");
                            self.line(&format!("IF LOF(#{ch}) < ({r}) * {length} THEN ERROR 63"));
                        }
                        self.line(&format!("GET #{ch}, {r}"));
                    }
                    (None, Some(v)) => {
                        let (v_pre, v) = self.expr(v, current_function);
                        self.lines(v_pre);
                        self.line(&format!("GET #{ch}, , {v}"));
                    }
                    (Some(rec), Some(v)) => {
                        let (r_pre, r) = self.expr(rec, current_function);
                        let (v_pre, v) = self.expr(v, current_function);
                        self.lines(r_pre);
                        self.lines(v_pre);
                        self.line(&format!("GET #{ch}, {r}, {v}"));
                    }
                }
            }
            Statement::Put {
                channel,
                record,
                var,
                ..
            } => {
                let (ch_pre, ch) = self.expr(channel, current_function);
                self.lines(ch_pre);
                match (record, var) {
                    (None, None) => self.line(&format!("PUT #{ch}")),
                    (Some(rec), None) => {
                        let (r_pre, r) = self.expr(rec, current_function);
                        self.lines(r_pre);
                        self.line(&format!("PUT #{ch}, {r}"));
                    }
                    (None, Some(v)) => {
                        let (v_pre, v) = self.expr(v, current_function);
                        self.lines(v_pre);
                        self.line(&format!("PUT #{ch}, , {v}"));
                    }
                    (Some(rec), Some(v)) => {
                        let (r_pre, r) = self.expr(rec, current_function);
                        let (v_pre, v) = self.expr(v, current_function);
                        self.lines(r_pre);
                        self.lines(v_pre);
                        self.line(&format!("PUT #{ch}, {r}, {v}"));
                    }
                }
            }
            Statement::Lset { var, value } => {
                let (v_pre, v) = self.expr(value, current_function);
                self.lines(v_pre);
                self.line(&format!("LSET {} = {v}", self.ident(var, current_function)));
            }
            Statement::Rset { var, value } => {
                let (v_pre, v) = self.expr(value, current_function);
                self.lines(v_pre);
                self.line(&format!("RSET {} = {v}", self.ident(var, current_function)));
            }
            Statement::Seek { channel, position } => {
                let (ch_pre, ch) = self.expr(channel, current_function);
                let (pos_pre, pos) = self.expr(position, current_function);
                self.lines(ch_pre);
                self.lines(pos_pre);
                self.line(&format!("SEEK #{ch}, {pos}"));
            }
            Statement::Lprint(tokens) => {
                let body = self.render_print_tokens(tokens, current_function);
                if body.is_empty() {
                    self.line("LPRINT");
                } else {
                    self.line(&format!("LPRINT {body}"));
                }
            }
            Statement::LprintUsing { format, tokens } => {
                let (fmt_pre, fmt_str) = self.expr(format, current_function);
                self.lines(fmt_pre);
                let body = self.render_print_tokens(tokens, current_function);
                if body.is_empty() {
                    self.line(&format!("LPRINT USING {fmt_str}"));
                } else {
                    self.line(&format!("LPRINT USING {fmt_str}; {body}"));
                }
            }
            Statement::Exit => match self.loop_exit_stack.last() {
                Some(LoopExit::NativeFor) => self.line("EXIT FOR"),
                Some(LoopExit::Goto(label)) => {
                    let label = label.clone();
                    self.line(&format!("GOTO {label}"));
                }
                None => self.line("' warning: EXIT outside of a loop"),
            },
            Statement::Continue => match self.loop_continue_stack.last() {
                Some(label) => {
                    let label = label.clone();
                    self.line(&format!("GOTO {label}"));
                }
                None => self.line("' warning: CONTINUE outside of a loop"),
            },
            Statement::SelectCase {
                expr,
                cases,
                else_body,
            } => {
                self.select_case(expr, cases, else_body, current_function);
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                self.try_catch(try_body, catch.as_ref(), finally_body, current_function);
            }
            Statement::Locate { row, col } => {
                let (row_prelude, row) = self.expr(row, current_function);
                let (col_prelude, col) = self.expr(col, current_function);
                self.lines(row_prelude);
                self.lines(col_prelude);
                self.line(&format!("LOCATE {row}, {col}"));
            }
            Statement::Color { fg, bg } => {
                let (fg_prelude, fg) = self.expr(fg, current_function);
                self.lines(fg_prelude);
                if let Some(bg) = bg {
                    let (bg_prelude, bg) = self.expr(bg, current_function);
                    self.lines(bg_prelude);
                    self.line(&format!("COLOR {fg}, {bg}"));
                } else {
                    self.line(&format!("COLOR {fg}"));
                }
            }
            Statement::OnBranch {
                expr,
                targets,
                is_gosub,
            } => {
                let (prelude, expr) = self.expr(expr, current_function);
                self.lines(prelude);
                let rendered: Vec<String> =
                    targets.iter().map(|t| self.label_target_text(t)).collect();
                let keyword = if *is_gosub { "GOSUB" } else { "GOTO" };
                self.line(&format!("ON {expr} {keyword} {}", rendered.join(", ")));
            }
            Statement::GlobalDecl(_) => {}
            Statement::Raw(raw) => self.line(raw),
            Statement::BlockComment(lines) => {
                for line in lines {
                    if line.is_empty() {
                        self.blank();
                    } else {
                        self.line(&format!("' {line}"));
                    }
                }
            }
            Statement::BlankLine => self.blank(),
        }
    }

    fn expr_statement(&mut self, expr_stmt: &Expr, current_function: Option<&FunctionInfo>) {
        if let Expr::ScalarMethodCall { base, method, args } = expr_stmt {
            if let Some(receiver) = self.expr_receiver_type(base) {
                if let Some(info) = self.method_info(receiver, method).cloned() {
                    let mut call_args = Vec::with_capacity(args.len() + 1);
                    call_args.push((**base).clone());
                    call_args.extend(args.iter().cloned());
                    let lines = self.call_lines(&info, &call_args, current_function);
                    self.lines(lines);
                    return;
                }
            }
        }
        if let Some((name, args)) = callable_expr(expr_stmt) {
            if let Some(info) = self.ordinary_function_info(name).cloned() {
                self.emit_call_statement(&info, args, current_function);
                return;
            }
        }

        let (prelude, expr_stmt) = self.expr(expr_stmt, current_function);
        self.lines(prelude);
        self.line(&expr_stmt);
    }

    /// Emits guarded-jump code for `condition`: jumps to `target` when the
    /// condition is false (`invert = false`) or true (`invert = true`),
    /// otherwise falls through to whatever is emitted next.
    ///
    /// Detects a top-level `&&`/`||` chain (only ever produced by
    /// `Parser::parse_condition`) and emits one short-circuit guard line per
    /// operand instead of rendering the whole condition as a single BASIC
    /// expression. Each operand's own prelude (e.g. a `GOSUB` for a function
    /// call) is emitted immediately before that operand's guard line, not
    /// hoisted to the top — this is what makes a later operand's side
    /// effects genuinely not run once an earlier operand already decided
    /// the outcome, the actual point of the feature.
    ///
    /// Any condition that isn't a chain falls back to exactly the
    /// single-`IF` behavior this replaced, so every existing condition is
    /// byte-for-byte unchanged.
    fn condition_jump(
        &mut self,
        condition: &Expr,
        target: &str,
        invert: bool,
        current_function: Option<&FunctionInfo>,
    ) {
        let chain_op = match condition {
            Expr::Binary {
                op: op @ (BinaryOp::AndAnd | BinaryOp::OrOr),
                ..
            } => Some(*op),
            _ => None,
        };

        let Some(chain_op) = chain_op else {
            let (prelude, text) = self.expr(condition, current_function);
            self.lines(prelude);
            let polarity = if invert { "<> 0" } else { "= 0" };
            self.line(&format!("IF ({text}) {polarity} THEN GOTO {target}"));
            return;
        };

        let mut operands = Vec::new();
        flatten_chain(condition, chain_op, &mut operands);

        let is_and = matches!(chain_op, BinaryOp::AndAnd);
        let polarity = if is_and { "= 0" } else { "<> 0" };
        // De Morgan duality: an AND-chain under `invert` behaves like a
        // plain OR-chain (needs a "some operand already decided true" skip
        // label) and vice versa — captured by this single XOR flag.
        let simple = is_and != invert;

        let jump_dest = if simple {
            target.to_string()
        } else {
            let id = self.next_label;
            self.next_label += 1;
            format!("SC_{id:04}_CONT")
        };

        for operand in &operands {
            let (prelude, text) = self.expr(operand, current_function);
            self.lines(prelude);
            self.line(&format!("IF ({text}) {polarity} THEN GOTO {jump_dest}"));
        }

        if !simple {
            self.line(&format!("GOTO {target}"));
            self.line(&format!("{jump_dest}:"));
        }
    }

    fn if_statement(
        &mut self,
        condition: &Expr,
        then_body: &[Stmt],
        else_body: &[Stmt],
        current_function: Option<&FunctionInfo>,
    ) {
        let id = self.next_label;
        self.next_label += 1;
        let else_label = format!("IF_{id:04}_ELSE");
        let end_label = format!("IF_{id:04}_END");

        if else_body.is_empty() {
            self.condition_jump(condition, &end_label, false, current_function);
            self.indent += 1;
            self.statements(then_body, current_function);
            self.indent -= 1;
            self.line(&format!("{end_label}:"));
            self.line("REM END IF");
        } else {
            self.condition_jump(condition, &else_label, false, current_function);
            self.indent += 1;
            self.statements(then_body, current_function);
            self.line(&format!("GOTO {end_label}"));
            self.indent -= 1;
            self.line(&format!("{else_label}:"));
            self.indent += 1;
            self.statements(else_body, current_function);
            self.indent -= 1;
            self.line(&format!("{end_label}:"));
            self.line("REM END IF");
        }
    }

    fn select_case(
        &mut self,
        expr: &Expr,
        cases: &[CaseClause],
        else_body: &[Stmt],
        current_function: Option<&FunctionInfo>,
    ) {
        let id = self.next_label;
        self.next_label += 1;
        let end_label = format!("SEL_{id:04}_END");

        // Store the select expression in a temp variable to avoid re-evaluation.
        // The temp variable must carry the same type suffix as the expression.
        let (prelude, expr_str) = self.expr(expr, current_function);
        self.lines(prelude);
        let suffix = expr_type_suffix(expr);
        let temp = {
            let id = self.next_label;
            self.next_label += 1;
            format!("BCCT{id}{suffix}")
        };
        self.line(&format!("{temp} = {expr_str}"));

        // Emit dispatch: one IF/GOTO per case clause.
        let case_labels: Vec<String> = (0..cases.len())
            .map(|i| format!("SEL_{id:04}_C{i}"))
            .collect();
        let else_label = format!("SEL_{id:04}_ELSE");

        for (i, clause) in cases.iter().enumerate() {
            let cond = clause
                .values
                .iter()
                .map(|v| self.case_value_cond(v, &temp, current_function))
                .collect::<Vec<_>>()
                .join(" OR ");
            self.line(&format!("IF ({cond}) <> 0 THEN GOTO {}", case_labels[i]));
        }
        self.line(&format!(
            "GOTO {}",
            if else_body.is_empty() {
                &end_label
            } else {
                &else_label
            }
        ));

        // Emit each case body.
        for (i, clause) in cases.iter().enumerate() {
            self.line(&format!("{}:", case_labels[i]));
            self.indent += 1;
            self.statements(&clause.body, current_function);
            self.line(&format!("GOTO {end_label}"));
            self.indent -= 1;
        }

        // Emit else body.
        if !else_body.is_empty() {
            self.line(&format!("{else_label}:"));
            self.indent += 1;
            self.statements(else_body, current_function);
            self.indent -= 1;
        }

        self.line(&format!("{end_label}:"));
        self.line("REM END SELECT");
    }

    /// `try ... catch err%, erl% ... end try` -- see `Statement::TryCatch`'s
    /// own doc comment for the semantics. Transpiles straight onto real
    /// BASIC's own `ON ERROR GOTO`/`RESUME <label>`: the catch label is a
    /// synthetic one like `if_statement`/`select_case`'s own, not a named
    /// procedure, so none of `resolver::validate`'s named-error-handler-
    /// target rules apply here. `RESUME <label>` (not a plain `GOTO`) at
    /// the end of the catch body is required, not stylistic -- a bare
    /// `GOTO` out of a handler leaves BASIC's own "currently trapping"
    /// state set, so a *later*, unrelated error elsewhere in the program
    /// would silently fail to trap at all (verified under dosbox-x; see
    /// tutorial/labels_and_error_handling.bcl's matching comment).
    fn try_catch(
        &mut self,
        try_body: &[Stmt],
        catch: Option<&TryCatchHandler>,
        finally_body: &[Stmt],
        current_function: Option<&FunctionInfo>,
    ) {
        let id = self.next_label;
        self.next_label += 1;
        let catch_label = format!("TRY_{id:04}_CATCH");
        let catch_run_label = format!("TRY_{id:04}_CATCH_RUN");
        let rethrow_label = format!("TRY_{id:04}_RETHROW");
        let finally_label = format!("TRY_{id:04}_FINALLY");
        let end_label = format!("TRY_{id:04}_END");
        let pending_name = format!("BCCTRY{id:04}PENDING%");
        let outer_handler = self.try_handler_stack.last().cloned();
        let restore_outer = |this: &mut Self| match &outer_handler {
            Some(label) => this.line(&format!("ON ERROR GOTO {label}")),
            None => this.line("ON ERROR GOTO 0"),
        };

        self.line(&format!("ON ERROR GOTO {catch_label}"));
        self.line(&format!("{pending_name} = 0"));
        self.try_handler_stack.push(catch_label.clone());
        self.indent += 1;
        self.statements(try_body, current_function);
        self.indent -= 1;
        self.try_handler_stack.pop();
        restore_outer(self);
        self.line(&format!("GOTO {finally_label}"));

        self.line(&format!("{catch_label}:"));
        self.indent += 1;
        self.line(&format!("{pending_name} = ERR"));
        if let Some(catch) = catch {
            // `catch err%(53, 76), erl%` (GitHub issue #76): a non-empty
            // filter list checks ERR *before* binding anything or running
            // the catch body at all -- a miss skips straight to `finally`
            // with `pending_name` still `= ERR` from above, so it
            // auto-rethrows exactly the way an explicit `throw` already
            // does (see the doc comment at the bottom of this function for
            // that shared finally-exit rethrow). An empty filter list (the
            // ordinary, pre-#76 `catch err%, erl%` shape) keeps the exact
            // unconditional `RESUME` this always emitted, byte-for-byte.
            if !catch.error_filter.is_empty() {
                let matched_label = format!("TRY_{id:04}_MATCHED");
                let mut prelude_lines = Vec::new();
                let mut condition_parts = Vec::new();
                for filter_expr in &catch.error_filter {
                    let (prelude, code) = self.expr(filter_expr, current_function);
                    prelude_lines.extend(prelude);
                    condition_parts.push(format!("(ERR = {code})"));
                }
                self.lines(prelude_lines);
                let condition = condition_parts.join(" OR ");
                self.line(&format!("IF {condition} THEN GOTO {matched_label}"));
                self.line(&format!("RESUME {finally_label}"));
                self.line(&format!("{matched_label}:"));
            }
            let err_name = self.ident(&catch.err_var, current_function);
            let erl_name = self.ident(&catch.erl_var, current_function);
            self.line(&format!("{err_name} = ERR"));
            self.line(&format!("{erl_name} = ERL"));
            if let Some(source_var) = &catch.source_var {
                let source_name = self.ident(source_var, current_function);
                self.line("GOSUB BCC_RESOLVE_SOURCE_FILE");
                self.line(&format!("{source_name} = BCCSOURCEFILE$"));
            }
            // RESUME clears BASIC's active-handler state before the user
            // catch runs, so a throw from it can be caught and unwound.
            self.line(&format!("RESUME {catch_run_label}"));
        } else {
            self.line(&format!("RESUME {finally_label}"));
        }
        self.indent -= 1;

        if let Some(catch) = catch {
            self.line(&format!("{catch_run_label}:"));
            self.line(&format!("ON ERROR GOTO {rethrow_label}"));
            self.indent += 1;
            self.try_handler_stack.push(rethrow_label.clone());
            self.statements(&catch.body, current_function);
            self.try_handler_stack.pop();
            self.line(&format!("{pending_name} = 0"));
            restore_outer(self);
            self.line(&format!("GOTO {finally_label}"));
            self.indent -= 1;

            self.line(&format!("{rethrow_label}:"));
            self.indent += 1;
            self.line(&format!("{pending_name} = ERR"));
            self.line(&format!("RESUME {finally_label}"));
            self.indent -= 1;
        }

        self.line(&format!("{finally_label}:"));
        restore_outer(self);
        self.indent += 1;
        self.statements(finally_body, current_function);
        self.line(&format!("IF {pending_name} <> 0 THEN ERROR {pending_name}"));
        self.indent -= 1;

        self.line(&format!("{end_label}:"));
        self.line("REM END TRY");
    }

    fn case_value_cond(
        &mut self,
        value: &CaseValue,
        temp: &str,
        current_function: Option<&FunctionInfo>,
    ) -> String {
        match value {
            CaseValue::Single(expr) => {
                let (_, s) = self.expr(expr, current_function);
                format!("{temp} = {s}")
            }
            CaseValue::Range { from, to } => {
                let (_, from) = self.expr(from, current_function);
                let (_, to) = self.expr(to, current_function);
                format!("{temp} >= {from} AND {temp} <= {to}")
            }
            CaseValue::Is { op, value } => {
                let (_, val) = self.expr(value, current_function);
                format!("{temp} {} {val}", binary_op(*op))
            }
        }
    }

    fn expr(
        &mut self,
        node: &Expr,
        current_function: Option<&FunctionInfo>,
    ) -> (Vec<String>, String) {
        match node {
            Expr::Integer(value) => (Vec::new(), value.to_string()),
            Expr::Float(value) => (Vec::new(), value.to_string()),
            Expr::HexLit(s) => (Vec::new(), s.clone()),
            Expr::String(value) => (Vec::new(), format!("\"{}\"", escape_string(value))),
            Expr::Ident(ident) => {
                let is_param = current_function
                    .is_some_and(|f| f.params.iter().any(|(src, _)| same_ident(&src.name, ident)));
                // `ERR`/`ERL` are the one pair of zero-arg builtins in
                // `known_callables` that are *always* suffixless in real
                // BASIC -- unlike `DATE$`/`TIME$`/`INKEY$`, whose own
                // legitimate spelling already carries a `$` suffix, so a
                // suffixed `err%`/`erl%` here can only be a genuine user
                // variable (e.g. `try`/`catch`'s own `catch err%, erl%`
                // locals -- see `Statement::TryCatch`), never the real
                // pseudo-variable, which this branch would otherwise
                // silently misrender as the literal text "ERR"/"ERL",
                // discarding whatever value the real local actually holds.
                let is_suffixed_err_or_erl = ident.suffix.is_some()
                    && (ident.name.eq_ignore_ascii_case("err")
                        || ident.name.eq_ignore_ascii_case("erl"));
                let emitted = if !is_param
                    && !is_suffixed_err_or_erl
                    && self
                        .known_callables
                        .contains(&ident.name.to_ascii_lowercase())
                {
                    self.canonical_callable(ident)
                } else {
                    self.ident(ident, current_function)
                };
                (Vec::new(), emitted)
            }
            Expr::ArrayRef { name, indices } => {
                if let Some(info) = self.ordinary_function_info(name).cloned() {
                    let call = self.call_lines(&info, indices, current_function);
                    return (call, info.result.as_basic());
                }

                let mut prelude = Vec::new();
                let mut rendered_indices = Vec::new();
                for index in indices {
                    let (index_prelude, index) = self.expr(index, current_function);
                    prelude.extend(index_prelude);
                    rendered_indices.push(index);
                }
                let base = if self
                    .known_callables
                    .contains(&name.name.to_ascii_lowercase())
                {
                    self.canonical_callable(name)
                } else {
                    self.ident(name, current_function)
                };
                (
                    prelude,
                    format!("{}({})", base, rendered_indices.join(", ")),
                )
            }
            Expr::Call { name, args } => {
                let array_bound_builtin = ["sizeof", "lbound", "ubound"]
                    .iter()
                    .find(|builtin| name.name.eq_ignore_ascii_case(builtin));
                if let Some(&builtin_name) = array_bound_builtin {
                    let resolved = match args.first() {
                        Some(Expr::Ident(array_name)) if args.len() <= 2 => match builtin_name {
                            "lbound" => {
                                self.resolve_lbound(array_name, args.get(1), current_function)
                            }
                            "ubound" => {
                                self.resolve_ubound(array_name, args.get(1), current_function)
                            }
                            _ => self.resolve_sizeof(array_name, args.get(1), current_function),
                        },
                        _ => Err(format!(
                            "{builtin_name} expects an array name, e.g. `{builtin_name}(arr%)` \
                             or `{builtin_name}(grid%, 1)`"
                        )),
                    };
                    return match resolved {
                        Ok(text) => (Vec::new(), text),
                        Err(message) => {
                            self.diagnostics.push(Diagnostic::error(
                                SourcePos::new("<validation>", 1, 1),
                                message,
                            ));
                            (Vec::new(), "1".to_string())
                        }
                    };
                }
                if let Some(info) = self.ordinary_function_info(name).cloned() {
                    (
                        self.call_lines(&info, args, current_function),
                        info.result.as_basic(),
                    )
                } else {
                    let mut prelude = Vec::new();
                    let mut rendered_args = Vec::new();
                    for arg in args {
                        let (arg_prelude, arg) = self.expr(arg, current_function);
                        prelude.extend(arg_prelude);
                        rendered_args.push(arg);
                    }
                    let key = name.name.to_ascii_lowercase();
                    let emit_name = if self.known_callables.contains(&key) {
                        self.canonical_callable(name)
                    } else {
                        // Not a recognized callable, so this is really a
                        // multi-index array element access parsed as a Call
                        // (see make_paren_ident_expr in parser.rs) -- needs
                        // the same param/local scope resolution as the
                        // single-index ArrayRef case above, or a function
                        // parameter's or local array's mangled storage name
                        // would be silently skipped.
                        self.ident(name, current_function)
                    };
                    (
                        prelude,
                        format!("{}({})", emit_name, rendered_args.join(", ")),
                    )
                }
            }
            Expr::Unary { op, expr } => {
                let (prelude, inner) = self.expr(expr, current_function);
                let rendered = match op {
                    UnaryOp::Neg => format!("-{inner}"),
                    UnaryOp::Not => format!("NOT ({inner})"),
                };
                (prelude, rendered)
            }
            Expr::Binary { left, op, right } => {
                let (mut prelude, left_str) = self.expr(left, current_function);
                let (right_prelude, right_str) = self.expr(right, current_function);
                prelude.extend(right_prelude);
                let left_r = if matches!(left.as_ref(), Expr::Binary { .. }) {
                    format!("({left_str})")
                } else {
                    left_str
                };
                let right_r = if matches!(right.as_ref(), Expr::Binary { .. }) {
                    format!("({right_str})")
                } else {
                    right_str
                };
                (prelude, format!("{left_r} {} {right_r}", binary_op(*op)))
            }
            Expr::ScalarMethodCall { base, method, args } => {
                let Some(receiver) = self.expr_receiver_type(base) else {
                    self.diagnostics.push(Diagnostic::error(
                        SourcePos::new("<validation>", 1, 1),
                        format!("method receiver for `.{method}()` must be scalar"),
                    ));
                    return (Vec::new(), "0".to_string());
                };
                let Some(info) = self.method_info(receiver, method).cloned() else {
                    self.diagnostics.push(Diagnostic::error(
                        SourcePos::new("<validation>", 1, 1),
                        format!("unknown method `.{method}()`"),
                    ));
                    return (Vec::new(), "0".to_string());
                };
                let mut call_args = Vec::with_capacity(args.len() + 1);
                call_args.push((**base).clone());
                call_args.extend(args.iter().cloned());
                (
                    self.call_lines(&info, &call_args, current_function),
                    info.result.as_basic(),
                )
            }
            Expr::FileIndex { .. }
            | Expr::FieldAccess { .. }
            | Expr::MethodCall { .. }
            | Expr::RecordLit { .. } => {
                unreachable!("record/file DSL must be lowered before codegen")
            }
        }
    }

    /// Render typed semantic expressions that may call a declared scalar
    /// function. Call operands are emitted in source order, then the result
    /// is snapshotted so a later call cannot overwrite the callee's shared
    /// BASIC result variable before its parent expression uses it.
    fn semantic_expression_with_prelude(
        &mut self,
        expression: &crate::semantic_ir::Expression,
        current_function: Option<&FunctionInfo>,
    ) -> Option<(Vec<String>, String)> {
        use crate::semantic_ir::ExpressionKind;
        match &expression.kind {
            ExpressionKind::Parenthesized(inner) => {
                let (lines, rendered) =
                    self.semantic_expression_with_prelude(inner, current_function)?;
                Some((lines, format!("({rendered})")))
            }
            ExpressionKind::Unary { operator, operand } => {
                let (lines, operand) =
                    self.semantic_expression_with_prelude(operand, current_function)?;
                let rendered = match operator.to_ascii_lowercase().as_str() {
                    "-" => format!("-{operand}"),
                    "not" => format!("NOT ({operand})"),
                    _ => return None,
                };
                Some((lines, rendered))
            }
            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                let (mut lines, left_text) =
                    self.semantic_expression_with_prelude(left, current_function)?;
                let (right_lines, right_text) =
                    self.semantic_expression_with_prelude(right, current_function)?;
                lines.extend(right_lines);
                let left = if matches!(left.kind, ExpressionKind::Binary { .. }) {
                    format!("({left_text})")
                } else {
                    left_text
                };
                let right = if matches!(right.kind, ExpressionKind::Binary { .. }) {
                    format!("({right_text})")
                } else {
                    right_text
                };
                let op = match operator.as_str() {
                    "+" | "-" | "*" | "/" | "\\" | "^" | "=" | "<>" | "<" | "<=" | ">" | ">=" => {
                        operator.to_string()
                    }
                    value if value.eq_ignore_ascii_case("and") => "AND".to_string(),
                    value if value.eq_ignore_ascii_case("or") => "OR".to_string(),
                    value if value.eq_ignore_ascii_case("xor") => "XOR".to_string(),
                    value if value.eq_ignore_ascii_case("mod") => "MOD".to_string(),
                    _ => return None,
                };
                Some((lines, format!("{left} {op} {right}")))
            }
            ExpressionKind::Call { name, arguments } => {
                let ident = BasicIdent::parse(name);
                let Some(info) = self.ordinary_function_info(&ident).cloned() else {
                    // Ordinary-call syntax on a scalar method (`ucase$(s$)`)
                    // is a method call on the first argument when its type is
                    // the method's receiver, as `records::lower` decides for
                    // the AST.
                    if self.ordinary_call_is_scalar_method(&ident, arguments) {
                        let first = &arguments[0];
                        let method_call = crate::semantic_ir::Expression {
                            kind: ExpressionKind::Member {
                                base: Some(Box::new(first.clone())),
                                member: ident.name.clone(),
                                arguments: Some(arguments[1..].to_vec()),
                            },
                            ..expression.clone()
                        };
                        return self
                            .semantic_expression_with_prelude(&method_call, current_function);
                    }
                    if self.function_info(&ident).is_some() {
                        return None;
                    }
                    if ["sizeof", "lbound", "ubound"]
                        .iter()
                        .any(|builtin| name.eq_ignore_ascii_case(builtin))
                    {
                        let rendered =
                            self.semantic_const_expression(expression, current_function)?;
                        let suffix = expression
                            .value_type
                            .suffix()
                            .map(|suffix| suffix.to_string())
                            .unwrap_or_default();
                        let result = self.next_temp_var_suffixed(&suffix);
                        return Some((vec![format!("{result} = {rendered}")], result));
                    }
                    if let Some(rank) = self.resolve_array_rank(&ident, current_function) {
                        if rank != arguments.len() {
                            return None;
                        }
                        let mut lines = Vec::new();
                        let mut indices = Vec::with_capacity(arguments.len());
                        for argument in arguments {
                            let (prelude, rendered) =
                                self.semantic_expression_with_prelude(argument, current_function)?;
                            lines.extend(prelude);
                            let suffix = argument
                                .value_type
                                .suffix()
                                .map(|suffix| suffix.to_string())
                                .unwrap_or_else(|| "%".to_string());
                            let index = self.next_temp_var_suffixed(&suffix);
                            lines.push(format!("{index} = {rendered}"));
                            indices.push(index);
                        }
                        let array = format!(
                            "{}({})",
                            self.ident(&ident, current_function),
                            indices.join(", ")
                        );
                        let suffix = expression
                            .value_type
                            .suffix()
                            .map(|suffix| suffix.to_string())
                            .unwrap_or_default();
                        let result = self.next_temp_var_suffixed(&suffix);
                        lines.push(format!("{result} = {array}"));
                        return Some((lines, result));
                    }
                    let mut lines = Vec::new();
                    let mut rendered_arguments = Vec::with_capacity(arguments.len());
                    for argument in arguments {
                        let (prelude, rendered) =
                            self.semantic_expression_with_prelude(argument, current_function)?;
                        lines.extend(prelude);
                        let suffix = argument
                            .value_type
                            .suffix()
                            .map(|suffix| suffix.to_string())
                            .unwrap_or_default();
                        let temporary = self.next_temp_var_suffixed(&suffix);
                        lines.push(format!("{temporary} = {rendered}"));
                        rendered_arguments.push(temporary);
                    }
                    let suffix = expression
                        .value_type
                        .suffix()
                        .map(|suffix| suffix.to_string())
                        .unwrap_or_default();
                    let result = self.next_temp_var_suffixed(&suffix);
                    lines.push(format!(
                        "{result} = {}({})",
                        self.canonical_callable(&ident),
                        rendered_arguments.join(", ")
                    ));
                    return Some((lines, result));
                };
                let parameters = info.semantic_parameters.as_ref()?;
                if arguments.len() > info.params.len() || parameters.len() != info.params.len() {
                    return None;
                }
                let mut complete_arguments = arguments.clone();
                for parameter in parameters.iter().skip(arguments.len()) {
                    complete_arguments.push(parameter.default.clone()?);
                }
                let mut lines = Vec::new();
                let mut rendered_arguments = Vec::with_capacity(complete_arguments.len());
                let mut array_arguments = vec![None; complete_arguments.len()];
                let mut byref_copyback = Vec::new();
                for (index, (argument, (param, lowered))) in
                    complete_arguments.iter().zip(&info.params).enumerate()
                {
                    let whole_array = match &argument.kind {
                        ExpressionKind::Name(name) => Some(BasicIdent::parse(name)),
                        ExpressionKind::Call { name, arguments } if arguments.is_empty() => {
                            Some(BasicIdent::parse(name))
                        }
                        _ => None,
                    }
                    .filter(|name| self.resolve_array_rank(name, current_function).is_some());
                    if let Some(target_rank) = info.param_ranks.get(index).copied().flatten() {
                        let source_name = whole_array?;
                        let source_rank =
                            self.resolve_array_rank(&source_name, current_function)?;
                        if source_rank != target_rank {
                            self.diagnostics.push(Diagnostic::error(
                                SourcePos::new("<validation>", 1, 1),
                                format!(
                                    "`{source_name}` has {source_rank} dimension{} here, but parameter `{}` of `{}` requires {target_rank} -- passing it would generate incorrect BASIC",
                                    if source_rank == 1 { "" } else { "s" }, param.name, info.source_name,
                                ),
                            ));
                            return None;
                        }
                        array_arguments[index] = Some((
                            self.ident(&source_name, current_function),
                            source_name,
                            target_rank,
                        ));
                        rendered_arguments.push(None);
                        continue;
                    }
                    if whole_array.is_some() {
                        return None;
                    }
                    if param.mode == ParamMode::ByRef {
                        let ExpressionKind::Name(name) = &argument.kind else {
                            return None;
                        };
                        if name.eq_ignore_ascii_case("true") || name.eq_ignore_ascii_case("false") {
                            return None;
                        }
                        let caller = self.ident(&BasicIdent::parse(name), current_function);
                        rendered_arguments.push(Some(caller.clone()));
                        byref_copyback.push((caller, lowered.as_basic()));
                    } else {
                        let (argument_lines, rendered) =
                            self.semantic_expression_with_prelude(argument, current_function)?;
                        let has_prelude = !argument_lines.is_empty();
                        lines.extend(argument_lines);
                        if has_prelude {
                            let suffix = param
                                .name
                                .suffix
                                .map(|suffix| suffix.to_string())
                                .unwrap_or_default();
                            let temporary = self.next_temp_var_suffixed(&suffix);
                            lines.push(format!("{temporary} = {rendered}"));
                            rendered_arguments.push(Some(temporary));
                        } else {
                            rendered_arguments.push(Some(rendered));
                        }
                    }
                }
                for (argument, (_, lowered)) in rendered_arguments.iter().zip(&info.params) {
                    if let Some(argument) = argument {
                        lines.push(format!("{} = {argument}", lowered.as_basic()));
                    }
                }
                for (index, array_argument) in array_arguments.iter().enumerate() {
                    let Some((actual_array, source_name, rank)) = array_argument else {
                        continue;
                    };
                    let (_, lowered) = info.params.get(index)?;
                    let bound_vars = info
                        .param_bound_vars
                        .get(index)
                        .cloned()
                        .unwrap_or_default();
                    let capacities = info
                        .param_capacities
                        .get(index)
                        .cloned()
                        .unwrap_or_default();
                    for (axis, bound_var) in bound_vars.iter().enumerate() {
                        let bound = self.resolve_axis_bound(source_name, axis, current_function)
                            .unwrap_or_else(|| {
                                self.diagnostics.push(Diagnostic::error(
                                    SourcePos::new("<validation>", 1, 1),
                                    format!("could not determine the size of `{source_name}` along axis {axis} to pass to `{}`", info.source_name),
                                ));
                                "1".to_string()
                            });
                        lines.push(format!("{bound_var} = {bound}"));
                        if let Some(capacity) = capacities.get(axis) {
                            let param_name = info
                                .params
                                .get(index)
                                .map(|(param, _)| param.name.as_basic())
                                .unwrap_or_default();
                            lines.push(format!(
                                "IF {bound_var} > {capacity} THEN PRINT \"runtime error: `{param_name}` of `{}` needs \"; {bound_var}; \" elements along axis {axis}, but its storage only holds {capacity}\" : STOP",
                                info.source_name,
                            ));
                        }
                    }
                    let loop_vars = (0..*rank).map(|_| self.next_temp_var()).collect::<Vec<_>>();
                    lines.extend(array_copy_lines(
                        &lowered.as_basic(),
                        actual_array,
                        &bound_vars,
                        "copy array argument into transpiled function storage",
                        &loop_vars,
                    ));
                }
                lines.push(format!("GOSUB {}", info.label));
                for (index, array_argument) in array_arguments.iter().enumerate() {
                    let Some((actual_array, _source_name, rank)) = array_argument else {
                        continue;
                    };
                    if info
                        .params
                        .get(index)
                        .is_some_and(|(param, _)| param.mode == ParamMode::ByRef)
                    {
                        let (_, lowered) = info.params.get(index)?;
                        let bound_vars = info
                            .param_bound_vars
                            .get(index)
                            .cloned()
                            .unwrap_or_default();
                        let loop_vars =
                            (0..*rank).map(|_| self.next_temp_var()).collect::<Vec<_>>();
                        lines.extend(array_copy_lines(
                            actual_array,
                            &lowered.as_basic(),
                            &bound_vars,
                            "copy mutated array argument back to caller storage",
                            &loop_vars,
                        ));
                    }
                }
                for (caller, lowered) in byref_copyback {
                    lines.push(format!("{caller} = {lowered}"));
                }
                let suffix = match info.result.suffix {
                    Some(TypeSuffix::Integer) => "%",
                    Some(TypeSuffix::String) => "$",
                    Some(TypeSuffix::Single) => "!",
                    Some(TypeSuffix::Double) => "#",
                    Some(TypeSuffix::Long) => "&",
                    None => "",
                };
                let result = self.next_temp_var_suffixed(suffix);
                lines.push(format!("{result} = {}", info.result.as_basic()));
                Some((lines, result))
            }
            ExpressionKind::Member {
                base: Some(base),
                member,
                arguments: Some(arguments),
            } => {
                let receiver = match base.value_type {
                    crate::semantic_ir::SemanticValueType::String => TypeSuffix::String,
                    crate::semantic_ir::SemanticValueType::Integer => TypeSuffix::Integer,
                    crate::semantic_ir::SemanticValueType::Long => TypeSuffix::Long,
                    crate::semantic_ir::SemanticValueType::Single => TypeSuffix::Single,
                    crate::semantic_ir::SemanticValueType::Double => TypeSuffix::Double,
                    crate::semantic_ir::SemanticValueType::Unknown
                    | crate::semantic_ir::SemanticValueType::Boolean => return None,
                };
                let info = self.method_info(receiver, member)?.clone();
                let parameters = info.semantic_parameters.as_ref()?;
                if arguments.len() > parameters.len()
                    || parameters.len() + 1 != info.params.len()
                    || parameters
                        .iter()
                        .skip(arguments.len())
                        .any(|parameter| parameter.default.is_none())
                {
                    return None;
                }
                let mut explicit_arguments = arguments.clone();
                explicit_arguments.extend(
                    parameters
                        .iter()
                        .skip(arguments.len())
                        .filter_map(|parameter| parameter.default.clone()),
                );
                if explicit_arguments.len() != parameters.len()
                    || info.param_ranks.iter().skip(1).any(Option::is_some)
                {
                    return None;
                }
                let mut lines = Vec::new();
                let mut byref_copyback = Vec::new();
                let (receiver_parameter, receiver_lowered) = info.params.first()?;
                if receiver_parameter.mode == ParamMode::ByRef {
                    let ExpressionKind::Name(name) = &base.kind else {
                        return None;
                    };
                    let caller = self.ident(&BasicIdent::parse(name), current_function);
                    lines.push(format!("{} = {caller}", receiver_lowered.as_basic()));
                    byref_copyback.push((caller, receiver_lowered.as_basic()));
                } else {
                    let (prelude, rendered) =
                        self.semantic_expression_with_prelude(base, current_function)?;
                    lines.extend(prelude);
                    lines.push(format!("{} = {rendered}", receiver_lowered.as_basic()));
                }
                for (index, (argument, ((parameter, lowered), semantic))) in explicit_arguments
                    .iter()
                    .zip(info.params.iter().skip(1).zip(parameters))
                    .enumerate()
                {
                    let expected = semantic
                        .value_type
                        .suffix()
                        .and_then(TypeSuffix::from_char)?;
                    // BASIC converts between numeric types on assignment, so
                    // only a string/number mismatch is a real type error here.
                    let supplied = argument
                        .value_type
                        .suffix()
                        .and_then(TypeSuffix::from_char)?;
                    if (supplied == TypeSuffix::String) != (expected == TypeSuffix::String)
                        || info
                            .param_ranks
                            .get(index + 1)
                            .copied()
                            .flatten()
                            .unwrap_or(0)
                            != semantic.array_axes
                    {
                        return None;
                    }
                    if parameter.mode == ParamMode::ByRef {
                        let ExpressionKind::Name(name) = &argument.kind else {
                            return None;
                        };
                        let caller = self.ident(&BasicIdent::parse(name), current_function);
                        lines.push(format!("{} = {caller}", lowered.as_basic()));
                        byref_copyback.push((caller, lowered.as_basic()));
                    } else {
                        let (prelude, rendered) =
                            self.semantic_expression_with_prelude(argument, current_function)?;
                        lines.extend(prelude);
                        let temporary = self.next_temp_var_suffixed(
                            &parameter
                                .name
                                .suffix
                                .map(|suffix| suffix.to_string())
                                .unwrap_or_default(),
                        );
                        lines.push(format!("{temporary} = {rendered}"));
                        lines.push(format!("{} = {temporary}", lowered.as_basic()));
                    }
                }
                lines.push(format!("GOSUB {}", info.label));
                lines.extend(
                    byref_copyback
                        .into_iter()
                        .map(|(caller, lowered)| format!("{caller} = {lowered}")),
                );
                let suffix = match info.result.suffix {
                    Some(TypeSuffix::Integer) => "%",
                    Some(TypeSuffix::String) => "$",
                    Some(TypeSuffix::Single) => "!",
                    Some(TypeSuffix::Double) => "#",
                    Some(TypeSuffix::Long) => "&",
                    None => "",
                };
                let result = self.next_temp_var_suffixed(suffix);
                lines.push(format!("{result} = {}", info.result.as_basic()));
                Some((lines, result))
            }
            _ => self
                .semantic_const_expression(expression, current_function)
                .map(|rendered| (Vec::new(), rendered)),
        }
    }

    /// Render a branch condition, transpiling a `&&`/`||` chain to per-operand
    /// guards. Declines (`None`) when any operand cannot be rendered.
    fn semantic_condition(
        &mut self,
        condition: &crate::semantic_ir::Expression,
        current_function: Option<&FunctionInfo>,
    ) -> Option<SemanticCondition> {
        let chain_operator = match &condition.kind {
            crate::semantic_ir::ExpressionKind::Binary { operator, .. }
                if operator == "&&" || operator == "||" =>
            {
                Some(operator.clone())
            }
            _ => None,
        };
        let render = |generator: &mut CodeGenerator,
                          expression: &crate::semantic_ir::Expression|
         -> Option<(Vec<String>, String)> {
            if let Some(text) = generator.semantic_const_expression(expression, current_function) {
                Some((Vec::new(), text))
            } else {
                generator.semantic_expression_with_prelude(expression, current_function)
            }
        };
        let Some(chain_operator) = chain_operator else {
            let (prelude, text) = render(self, condition)?;
            return Some(SemanticCondition::Expression { prelude, text });
        };
        let mut flattened = Vec::new();
        flatten_semantic_chain(condition, &chain_operator, &mut flattened);
        let mut operands = Vec::with_capacity(flattened.len());
        for operand in flattened {
            operands.push(render(self, operand)?);
        }
        let continue_id = self.next_label;
        self.next_label += 1;
        Some(SemanticCondition::Chain {
            is_and: chain_operator == "&&",
            continue_id,
            operands,
        })
    }

    /// Whether an ordinary-syntax call `name(first, ...)` is a call to a
    /// scalar method: no ordinary function claims `name`, and the first
    /// argument's type is the receiver of a method of that name and suffix.
    fn ordinary_call_is_scalar_method(
        &self,
        ident: &BasicIdent,
        arguments: &[crate::semantic_ir::Expression],
    ) -> bool {
        self.ordinary_function_info(ident).is_none()
            && arguments
                .first()
                .and_then(|first| semantic_receiver_suffix(first.value_type))
                .is_some_and(|receiver| {
                    self.method_info(receiver, &ident.name)
                        .is_some_and(|method| method.source_name.suffix == ident.suffix)
                })
    }

    fn semantic_expression_contains_callable_call(
        &self,
        expression: &crate::semantic_ir::Expression,
    ) -> bool {
        use crate::semantic_ir::ExpressionKind;
        match &expression.kind {
            ExpressionKind::Call { name, arguments } => {
                let ident = BasicIdent::parse(name);
                self.ordinary_function_info(&ident).is_some()
                    || self.ordinary_call_is_scalar_method(&ident, arguments)
                    || arguments
                        .iter()
                        .any(|argument| self.semantic_expression_contains_callable_call(argument))
            }
            ExpressionKind::Parenthesized(inner) | ExpressionKind::Unary { operand: inner, .. } => {
                self.semantic_expression_contains_callable_call(inner)
            }
            ExpressionKind::Binary { left, right, .. } => {
                self.semantic_expression_contains_callable_call(left)
                    || self.semantic_expression_contains_callable_call(right)
            }
            ExpressionKind::Index { index, .. } => {
                self.semantic_expression_contains_callable_call(index)
            }
            ExpressionKind::MultiIndex { indices, .. } => indices
                .iter()
                .any(|index| self.semantic_expression_contains_callable_call(index)),
            ExpressionKind::Member {
                base,
                member,
                arguments,
            } => {
                let is_scalar_method_call = base.as_ref().is_some_and(|base| {
                    let receiver = match base.value_type {
                        crate::semantic_ir::SemanticValueType::String => Some(TypeSuffix::String),
                        crate::semantic_ir::SemanticValueType::Integer => Some(TypeSuffix::Integer),
                        crate::semantic_ir::SemanticValueType::Long => Some(TypeSuffix::Long),
                        crate::semantic_ir::SemanticValueType::Single => Some(TypeSuffix::Single),
                        crate::semantic_ir::SemanticValueType::Double => Some(TypeSuffix::Double),
                        crate::semantic_ir::SemanticValueType::Unknown
                        | crate::semantic_ir::SemanticValueType::Boolean => None,
                    };
                    matches!(arguments, Some(_))
                        && receiver
                            .is_some_and(|receiver| self.method_info(receiver, member).is_some())
                });
                is_scalar_method_call
                    || base
                        .as_ref()
                        .is_some_and(|base| self.semantic_expression_contains_callable_call(base))
                    || arguments.as_ref().is_some_and(|arguments| {
                        arguments.iter().any(|argument| {
                            self.semantic_expression_contains_callable_call(argument)
                        })
                    })
            }
            ExpressionKind::RecordLiteral(fields)
            | ExpressionKind::PartialRecordLiteral(fields) => fields
                .iter()
                .any(|field| self.semantic_expression_contains_callable_call(&field.value)),
            ExpressionKind::Name(_) | ExpressionKind::Literal(_) | ExpressionKind::Boolean(_) => {
                false
            }
        }
    }

    fn semantic_const_expression(
        &self,
        expression: &crate::semantic_ir::Expression,
        current_function: Option<&FunctionInfo>,
    ) -> Option<String> {
        use crate::semantic_ir::ExpressionKind;
        match &expression.kind {
            ExpressionKind::Literal(value) => Some(value.clone()),
            ExpressionKind::Boolean(value) => Some(if *value { "-1" } else { "0" }.to_string()),
            ExpressionKind::Name(name) => {
                if name.eq_ignore_ascii_case("true") {
                    return Some("-1".to_string());
                }
                if name.eq_ignore_ascii_case("false") {
                    return Some("0".to_string());
                }
                if name.contains('.')
                    && expression.value_type != crate::semantic_ir::SemanticValueType::Unknown
                {
                    let suffix = match expression.value_type.suffix()? {
                        '%' => TypeSuffix::Integer,
                        '$' => TypeSuffix::String,
                        '!' => TypeSuffix::Single,
                        '#' => TypeSuffix::Double,
                        '&' => TypeSuffix::Long,
                        _ => return None,
                    };
                    let parts = name.split('.').collect::<Vec<_>>();
                    let storage = BasicIdent {
                        name: camel_join(&parts),
                        suffix: Some(suffix),
                    };
                    return Some(self.ident(&storage, current_function));
                }
                let ident = BasicIdent::parse(name);
                if current_function.is_some_and(|function| {
                    function
                        .params
                        .iter()
                        .any(|(parameter, _)| same_ident(&parameter.name, &ident))
                }) {
                    return Some(self.ident(&ident, current_function));
                }
                let base = ident.name.to_ascii_lowercase();
                let base = base.trim_end_matches(['$', '%', '&', '!', '#']);
                let suffixed_error_local = ident.suffix.is_some()
                    && (ident.name.eq_ignore_ascii_case("err")
                        || ident.name.eq_ignore_ascii_case("erl"));
                if !suffixed_error_local
                    && (self
                        .known_callables
                        .contains(&ident.name.to_ascii_lowercase())
                        || self.known_callables.contains(base))
                {
                    Some(self.canonical_callable(&ident))
                } else {
                    Some(self.ident(&ident, current_function))
                }
            }
            ExpressionKind::Parenthesized(inner) => self
                .semantic_const_expression(inner, current_function)
                .map(|value| format!("({value})")),
            ExpressionKind::Unary { operator, operand } => {
                let operand = self.semantic_const_expression(operand, current_function)?;
                match operator.to_ascii_lowercase().as_str() {
                    "-" => Some(format!("-{operand}")),
                    "not" => Some(format!("NOT ({operand})")),
                    _ => None,
                }
            }
            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                let left_text = self.semantic_const_expression(left, current_function)?;
                let right_text = self.semantic_const_expression(right, current_function)?;
                let left = if matches!(left.kind, ExpressionKind::Binary { .. }) {
                    format!("({left_text})")
                } else {
                    left_text
                };
                let right = if matches!(right.kind, ExpressionKind::Binary { .. }) {
                    format!("({right_text})")
                } else {
                    right_text
                };
                let operator = match operator.as_str() {
                    "+" => "+",
                    "-" => "-",
                    "*" => "*",
                    "/" => "/",
                    "\\" => "\\",
                    "^" => "^",
                    "=" => "=",
                    "<>" => "<>",
                    "<" => "<",
                    "<=" => "<=",
                    ">" => ">",
                    ">=" => ">=",
                    value if value.eq_ignore_ascii_case("and") => "AND",
                    value if value.eq_ignore_ascii_case("or") => "OR",
                    value if value.eq_ignore_ascii_case("xor") => "XOR",
                    value if value.eq_ignore_ascii_case("mod") => "MOD",
                    "&&" | "||" => return None,
                    _ => return None,
                };
                Some(format!("{left} {operator} {right}"))
            }
            ExpressionKind::Member {
                base: Some(base),
                member,
                arguments: None,
            } => {
                let ExpressionKind::Name(base_name) = &base.kind else {
                    return None;
                };
                let record_type = base.record_type.as_deref()?;
                let record = self
                    .semantic_records
                    .iter()
                    .find(|record| record.name.eq_ignore_ascii_case(record_type))?;
                let field = semantic_record_field(&self.semantic_records, record, member)?;
                let suffix = match field.field_type {
                    crate::semantic_ir::RecordFieldType::String { .. } => TypeSuffix::String,
                    crate::semantic_ir::RecordFieldType::Int16 { .. }
                    | crate::semantic_ir::RecordFieldType::Int { .. } => TypeSuffix::Integer,
                    crate::semantic_ir::RecordFieldType::Int32 { .. } => TypeSuffix::Long,
                    crate::semantic_ir::RecordFieldType::Float32 { .. } => TypeSuffix::Single,
                    crate::semantic_ir::RecordFieldType::Float64 { .. } => TypeSuffix::Double,
                    crate::semantic_ir::RecordFieldType::Record { .. } => return None,
                };
                let storage = BasicIdent {
                    name: camel_join(&[base_name, &field.name]),
                    suffix: Some(suffix),
                };
                Some(self.ident(&storage, current_function))
            }
            ExpressionKind::Call { name, arguments } => {
                let ident = BasicIdent::parse(name);
                if ["sizeof", "lbound", "ubound"]
                    .iter()
                    .any(|builtin| name.eq_ignore_ascii_case(builtin))
                {
                    let Some(array_name) = arguments.first().and_then(semantic_array_designator)
                    else {
                        return None;
                    };
                    if arguments.len() > 2 {
                        return None;
                    }
                    let axis = match arguments.get(1) {
                        None => None,
                        Some(axis) => Some(Expr::Integer(semantic_integer_literal_axis(axis)?)),
                    };
                    let array = BasicIdent::parse(array_name);
                    let resolved = match name.to_ascii_lowercase().as_str() {
                        "sizeof" => self.resolve_sizeof(&array, axis.as_ref(), current_function),
                        "lbound" => self.resolve_lbound(&array, axis.as_ref(), current_function),
                        _ => self.resolve_ubound(&array, axis.as_ref(), current_function),
                    };
                    return match resolved {
                        Ok(text) => Some(text),
                        Err(message) => {
                            self.deferred_errors.borrow_mut().push(message);
                            Some("1".to_string())
                        }
                    };
                }
                let arguments = arguments
                    .iter()
                    .map(|argument| self.semantic_const_expression(argument, current_function))
                    .collect::<Option<Vec<_>>>()?;
                if let Some(rank) = self.resolve_array_rank(&ident, current_function) {
                    if rank != arguments.len() {
                        return None;
                    }
                    return Some(format!(
                        "{}({})",
                        self.ident(&ident, current_function),
                        arguments.join(", ")
                    ));
                }
                if self.function_info(&ident).is_some() {
                    return None;
                }
                Some(format!(
                    "{}({})",
                    self.canonical_callable(&ident),
                    arguments.join(", ")
                ))
            }
            ExpressionKind::Index { name, index } => {
                let ident = BasicIdent::parse(name);
                if self.ordinary_function_info(&ident).is_some() {
                    return None;
                }
                let index = self.semantic_const_expression(index, current_function)?;
                Some(format!("{}({index})", self.ident(&ident, current_function)))
            }
            ExpressionKind::MultiIndex { name, indices } => {
                let ident = BasicIdent::parse(name);
                if self.ordinary_function_info(&ident).is_some() {
                    return None;
                }
                let indices = indices
                    .iter()
                    .map(|index| self.semantic_const_expression(index, current_function))
                    .collect::<Option<Vec<_>>>()?;
                if indices.len() != self.resolve_array_rank(&ident, current_function)? {
                    return None;
                }
                Some(format!(
                    "{}({})",
                    self.ident(&ident, current_function),
                    indices.join(", ")
                ))
            }
            _ => None,
        }
    }

    fn call_lines(
        &mut self,
        info: &FunctionInfo,
        args: &[Expr],
        current_function: Option<&FunctionInfo>,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        let mut rendered_args = Vec::new();
        for arg in args {
            let (arg_prelude, rendered_arg) = self.expr(arg, current_function);
            lines.extend(arg_prelude);
            rendered_args.push(rendered_arg);
        }

        // Resolve, once up front, which arguments are being passed as
        // whole arrays -- either `arr%()` (empty parens; array-ness is
        // explicit in the syntax) or a bare identifier that resolves to a
        // declared array *and* whose corresponding parameter is itself
        // declared as an array (array-ness is inferred purely from the
        // callee's signature in that case, so a bare name only counts when
        // the callee already expects an array there). `Some((resolved
        // caller-side mangled name, original source identifier, reconciled
        // rank))` when so -- the original identifier is kept alongside the
        // mangled one because resolving its bounds (below) has to look it
        // up by its *source* name, not the per-function generated one.
        // `None` for a plain scalar argument, or an array-shaped argument
        // whose rank didn't match (reported once, here).
        let array_args: Vec<Option<(String, BasicIdent, usize)>> = args
            .iter()
            .enumerate()
            .map(|(index, arg)| {
                let target_rank = info.param_ranks.get(index).copied().flatten();
                let source_name: &BasicIdent = match arg {
                    Expr::ArrayRef { name, indices } if indices.is_empty() => name,
                    Expr::Ident(name)
                        if target_rank.is_some()
                            && self.resolve_array_rank(name, current_function).is_some() =>
                    {
                        name
                    }
                    _ => return None,
                };
                let source_rank = self.resolve_array_rank(source_name, current_function);
                match (target_rank, source_rank) {
                    (Some(t), Some(s)) if t != s => {
                        let param_name = info
                            .params
                            .get(index)
                            .map(|(p, _)| p.name.as_basic())
                            .unwrap_or_default();
                        self.diagnostics.push(Diagnostic::error(
                            SourcePos::new("<validation>", 1, 1),
                            format!(
                                "`{}` has {} dimension{} here, but parameter `{}` of `{}` is \
                                 indexed with {} -- passing it would generate incorrect BASIC",
                                source_name,
                                s,
                                if s == 1 { "" } else { "s" },
                                param_name,
                                info.source_name,
                                t,
                            ),
                        ));
                        None
                    }
                    _ => {
                        let rank = target_rank.or(source_rank).unwrap_or(1);
                        Some((
                            self.ident(source_name, current_function),
                            source_name.clone(),
                            rank,
                        ))
                    }
                }
            })
            .collect();

        for (index, rendered_arg) in rendered_args.iter().enumerate() {
            if let Some((_, lowered)) = info.params.get(index) {
                if array_args[index].is_none() {
                    lines.push(format!("{} = {rendered_arg}", lowered.as_basic()));
                }
            } else {
                lines.push(format!(
                    "' warning: extra argument {} for {} ignored by current lowering",
                    index + 1,
                    info.source_name
                ));
            }
        }

        for (index, (param, lowered)) in info.params.iter().enumerate().skip(args.len()) {
            let semantic_default = index
                .checked_sub(usize::from(info.receiver.is_some()))
                .and_then(|position| {
                    info.semantic_parameters
                        .as_ref()
                        .and_then(|parameters| parameters.get(position))
                })
                .and_then(|parameter| parameter.default.as_ref());
            if let Some(default) = semantic_default {
                if let Some((default_prelude, rendered_default)) =
                    self.semantic_expression_with_prelude(default, current_function)
                {
                    lines.extend(default_prelude);
                    lines.push(format!("{} = {rendered_default}", lowered.as_basic()));
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        SourcePos::new("<validation>", 1, 1),
                        format!(
                            "typed default for parameter `{}` of `{}` isn't supported by the BASIC backend",
                            param.name, info.source_name
                        ),
                    ));
                }
            } else if info.semantic_parameters.is_none() {
                if let Some(default) = &param.default {
                    let (default_prelude, rendered_default) = self.expr(default, current_function);
                    lines.extend(default_prelude);
                    lines.push(format!("{} = {rendered_default}", lowered.as_basic()));
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        SourcePos::new("<validation>", 1, 1),
                        format!(
                            "`{}` expects {} argument(s), got {}",
                            info.source_name,
                            info.params.len(),
                            args.len()
                        ),
                    ));
                }
            } else {
                self.diagnostics.push(Diagnostic::error(
                    SourcePos::new("<validation>", 1, 1),
                    format!(
                        "`{}` expects {} argument(s), got {}",
                        info.source_name,
                        info.params.len(),
                        args.len()
                    ),
                ));
            }
        }

        for (index, _arg) in args.iter().enumerate() {
            if let Some((_, lowered)) = info.params.get(index) {
                if let Some((actual_array, source_name, rank)) = &array_args[index] {
                    let bound_vars = info
                        .param_bound_vars
                        .get(index)
                        .cloned()
                        .unwrap_or_default();
                    let capacities = info
                        .param_capacities
                        .get(index)
                        .cloned()
                        .unwrap_or_default();
                    for (axis, bound_var) in bound_vars.iter().enumerate() {
                        let bound = self
                            .resolve_axis_bound(source_name, axis, current_function)
                            .unwrap_or_else(|| {
                                self.diagnostics.push(Diagnostic::error(
                                    SourcePos::new("<validation>", 1, 1),
                                    format!(
                                        "could not determine the size of `{}` along axis {} \
                                         to pass to `{}`",
                                        source_name, axis, info.source_name,
                                    ),
                                ));
                                "1".to_string()
                            });
                        lines.push(format!("{bound_var} = {bound}"));
                        // Parameter storage is DIMed once, at top-level, sized to fit
                        // every call site the transpiler could resolve at compile time
                        // (see `infer_array_param_capacities`). This is the runtime
                        // backstop for whatever that inference couldn't prove safe --
                        // a call passing more elements than the storage was built for
                        // would otherwise silently corrupt whatever memory follows it.
                        if let Some(capacity) = capacities.get(axis) {
                            let param_name = info
                                .params
                                .get(index)
                                .map(|(p, _)| p.name.as_basic())
                                .unwrap_or_default();
                            lines.push(format!(
                                "IF {bound_var} > {capacity} THEN PRINT \"runtime error: `{param_name}` of `{}` needs \"; {bound_var}; \" elements along axis {axis}, but its storage only holds {capacity}\" : STOP",
                                info.source_name,
                            ));
                        }
                    }
                    let loop_vars: Vec<String> = (0..*rank).map(|_| self.next_temp_var()).collect();
                    lines.extend(array_copy_lines(
                        &lowered.as_basic(),
                        actual_array,
                        &bound_vars,
                        "copy array argument into transpiled function storage",
                        &loop_vars,
                    ));
                }
            }
        }

        lines.push(format!("GOSUB {}", info.label));

        for (index, arg) in args.iter().enumerate() {
            if let Some((param, lowered)) = info.params.get(index) {
                if param.mode != ParamMode::ByRef {
                    continue;
                }
                if let Some((actual_array, _source_name, rank)) = &array_args[index] {
                    let bound_vars = info
                        .param_bound_vars
                        .get(index)
                        .cloned()
                        .unwrap_or_default();
                    let loop_vars: Vec<String> = (0..*rank).map(|_| self.next_temp_var()).collect();
                    lines.extend(array_copy_lines(
                        actual_array,
                        &lowered.as_basic(),
                        &bound_vars,
                        "copy mutated array argument back to caller storage",
                        &loop_vars,
                    ));
                } else if let Expr::Ident(ident) = arg {
                    let caller_name = self.ident(ident, current_function);
                    lines.push(format!("{} = {}", caller_name, lowered.as_basic()));
                } else {
                    self.diagnostics.push(Diagnostic::error(
                        SourcePos::new("<validation>", 1, 1),
                        format!(
                            "`byref` parameter `{}` of `{}` was called with an argument that \
                             isn't a plain variable -- byref requires somewhere to write the \
                             result back to",
                            param.name, info.source_name
                        ),
                    ));
                }
            }
        }

        lines
    }

    fn emit_call_statement(
        &mut self,
        info: &FunctionInfo,
        args: &[Expr],
        current_function: Option<&FunctionInfo>,
    ) {
        let lines = self.call_lines(info, args, current_function);
        self.lines(lines);
    }

    fn next_temp_var(&mut self) -> String {
        self.next_temp_var_suffixed("%")
    }

    fn next_temp_var_suffixed(&mut self, suffix: &str) -> String {
        let id = self.next_label;
        self.next_label += 1;
        format!("BCCT{id}{suffix}")
    }

    fn canonical_callable(&self, name: &BasicIdent) -> String {
        BasicIdent {
            name: name.name.to_ascii_uppercase(),
            suffix: name.suffix,
        }
        .as_basic()
    }

    fn ident(&self, ident: &BasicIdent, current_function: Option<&FunctionInfo>) -> String {
        let source_key = ident.as_basic().to_ascii_lowercase();
        if self.record_buffer_names.contains(&source_key) {
            // FIELD buffers are structurally global -- there is exactly
            // one FIELD-bound buffer per record field, shared by every
            // function/procedure that touches that file -- so this must
            // be checked before, and instead of, per-function allocation,
            // regardless of scope.
            return if self.synthesized_buffer_names.contains(&source_key) {
                // Transpiler-built (via `buffer_ident`): already
                // deliberately camelCased, so its case is preserved
                // rather than flattened by the normalization below.
                ident.as_basic()
            } else {
                // Author-typed in raw-BASIC-passthrough source: still
                // gets BASCAL's normal lowercase normalization, same as
                // any other identifier.
                BasicIdent {
                    name: ident.name.to_ascii_lowercase(),
                    suffix: ident.suffix,
                }
                .as_basic()
            };
        }
        if let Some(generated) = self.const_var_names.get(&ident.name.to_ascii_lowercase()) {
            // Constants resolve program-wide to the fixed name generated by
            // `const_var_name`, even when their declaration appears in a
            // callable body.
            return generated.clone();
        }
        if let Some(info) = current_function {
            // Params have already-allocated lowered names.
            if let Some((_, lowered)) = info
                .params
                .iter()
                .find(|(source, _)| same_ident(&source.name, ident))
            {
                return lowered.as_basic();
            }
            if !info.globals.contains(&source_key) {
                // Check per-function cache first.
                {
                    let cache = info.local_var_map.borrow();
                    if let Some(cached) = cache.get(&source_key) {
                        return cached.clone();
                    }
                }
                // Allocate a name that doesn't clash with any already-claimed BASIC name.
                let preferred_stem = camel_join(&[&info.stem, &ident.name]);
                let lowered = {
                    let taken = self.taken_names.borrow();
                    allocate_unique(&preferred_stem, ident.suffix, &taken)
                };
                let lowered_basic = lowered.as_basic();
                self.taken_names
                    .borrow_mut()
                    .insert(lowered_basic.to_ascii_lowercase());
                info.local_var_map
                    .borrow_mut()
                    .insert(source_key, lowered_basic.clone());
                return lowered_basic;
            }
        }
        if ident.name.eq_ignore_ascii_case("err") || ident.name.eq_ignore_ascii_case("erl") {
            // Real BASCOM reserves ERR/ERL outright -- confirmed under
            // real BASCOM/dosbox-x that it rejects even a *suffixed*
            // reference like `err%` (`catch`'s own conventional binding
            // name, mirroring the real pseudo-variable it captures) as an
            // assignment target, not just as an expression operand. A
            // function-local of this name never hits this rename: it
            // already gets a per-function-unique generated name above
            // (e.g. `showErr0%`) that never collides with the bare literal
            // in the first place. Only a top-level (or explicit `global`)
            // one reaches here, so BASCAL source keeps writing `err%`/
            // `erl%` -- the natural, conventional choice -- while the
            // generated BASIC uses this fixed rename instead.
            return BasicIdent {
                name: format!("BCC{}", ident.name.to_ascii_uppercase()),
                suffix: ident.suffix,
            }
            .as_basic();
        }
        BasicIdent {
            name: ident.name.to_ascii_lowercase(),
            suffix: ident.suffix,
        }
        .as_basic()
    }

    fn function_info(&self, name: &BasicIdent) -> Option<&FunctionInfo> {
        self.functions
            .iter()
            .find(|function| same_ident(&function.source_name, name))
    }

    /// Same lookup as `function_info`, but excluding methods
    /// (`receiver.is_some()`) -- used wherever an *ordinary* call site
    /// (`Expr::Call`/`Expr::ArrayRef`, or a bare call statement) is being
    /// resolved, so a method never gets matched here with zero type
    /// checking on its receiver (`function_info` alone would happily match
    /// a method by name+suffix and hand its whole indices/args list
    /// straight to `call_lines`, silently binding the first argument to
    /// `self` regardless of its actual type -- confirmed: `ltrim$(n%)`,
    /// `n%` an Integer, compiled and ran with no diagnostic at all before
    /// this fix, assigning `n%` into `ltrim`'s string `self` param).
    /// `records::Lowerer::try_ordinary_call_as_method` is the one, real,
    /// type-checked path from ordinary-call syntax to a method now -- it
    /// runs before this codegen pass ever sees the program, rewriting an
    /// eligible call into a genuine `Expr::ScalarMethodCall`, which this
    /// function never needs to see at all.
    fn ordinary_function_info(&self, name: &BasicIdent) -> Option<&FunctionInfo> {
        self.functions
            .iter()
            .find(|function| function.receiver.is_none() && same_ident(&function.source_name, name))
    }

    fn method_info(&self, receiver: TypeSuffix, name: &str) -> Option<&FunctionInfo> {
        self.functions.iter().find(|function| {
            function.receiver == Some(receiver)
                && function.source_name.name.eq_ignore_ascii_case(name)
        })
    }

    fn expr_receiver_type(&self, expr: &Expr) -> Option<TypeSuffix> {
        match expr {
            Expr::String(_) => Some(TypeSuffix::String),
            Expr::Integer(_) | Expr::HexLit(_) => Some(TypeSuffix::Integer),
            Expr::Float(_) => Some(TypeSuffix::Single),
            Expr::Ident(id) | Expr::Call { name: id, .. } | Expr::ArrayRef { name: id, .. } => {
                Some(id.suffix.unwrap_or(TypeSuffix::Single))
            }
            Expr::Unary { expr, .. } => self.expr_receiver_type(expr),
            Expr::Binary { left, .. } => self.expr_receiver_type(left),
            Expr::ScalarMethodCall { base, method, .. } => {
                let receiver = self.expr_receiver_type(base)?;
                self.method_info(receiver, method)?.source_name.suffix
            }
            _ => None,
        }
    }

    /// Renders a `goto`/`gosub`/`on error goto`/`resume`/`on ... goto`/
    /// `on ... gosub` target. The parser only ever produces a bare label
    /// identifier here, or (for `on error goto` only) the integer `0`
    /// sentinel that disables the error trap.
    ///
    /// A target naming a declared `function`/`procedure` needs special
    /// handling: unlike an ordinary `name:` label (emitted, and later
    /// number-resolved, using the exact text the author wrote), a
    /// function/procedure entry point is emitted under its own synthesized
    /// `FN_<stem>` label (see `FunctionInfo::from_def`) -- an ordinary call
    /// site already knows to emit that directly, but a raw label reference
    /// like this one has no reason to guess it, so look it up through the
    /// function table instead of rendering the identifier text as-is.
    fn label_target_text(&self, target: &Expr) -> String {
        match target {
            Expr::Ident(ident) => self.label_target_ident(ident),
            Expr::Integer(0) => "0".to_string(),
            _ => unreachable!(
                "goto/gosub/on/resume targets are label identifiers (or the `on error goto 0` sentinel), enforced at parse time"
            ),
        }
    }

    fn label_target_ident(&self, ident: &BasicIdent) -> String {
        match self.function_info(ident) {
            Some(info) => info.label.clone(),
            None => user_label_token(&ident.as_basic()),
        }
    }

    fn semantic_label_target_text(&self, name: &str) -> String {
        self.label_target_ident(&BasicIdent::parse(name))
    }

    /// Declared rank of the array named `name`, resolved in whatever scope
    /// it's actually visible in: a local `dim` inside `current_function`, a
    /// parameter of `current_function` being forwarded onward (using that
    /// parameter's own inferred rank), or a top-level `dim`. `None` means
    /// unknown -- nothing to check a call site's argument against.
    fn resolve_array_rank(
        &self,
        name: &BasicIdent,
        current_function: Option<&FunctionInfo>,
    ) -> Option<usize> {
        let key = name.as_basic().to_ascii_lowercase();
        if let Some(info) = current_function {
            if let Some(rank) = info.local_array_ranks.get(&key) {
                return Some(*rank);
            }
            if let Some(index) = info
                .params
                .iter()
                .position(|(p, _)| same_ident(&p.name, name))
            {
                return info.param_ranks.get(index).copied().flatten();
            }
        }
        self.top_level_array_ranks.get(&key).copied()
    }

    fn resolve_array_suffix(
        &self,
        name: &BasicIdent,
        current_function: Option<&FunctionInfo>,
    ) -> Option<TypeSuffix> {
        let key = name.as_basic().to_ascii_lowercase();
        let suffix_from_type = |value_type: crate::semantic_ir::SemanticValueType| match value_type {
            crate::semantic_ir::SemanticValueType::String => Some(TypeSuffix::String),
            crate::semantic_ir::SemanticValueType::Integer => Some(TypeSuffix::Integer),
            crate::semantic_ir::SemanticValueType::Long => Some(TypeSuffix::Long),
            crate::semantic_ir::SemanticValueType::Single => Some(TypeSuffix::Single),
            crate::semantic_ir::SemanticValueType::Double => Some(TypeSuffix::Double),
            crate::semantic_ir::SemanticValueType::Unknown
            | crate::semantic_ir::SemanticValueType::Boolean => None,
        };
        if let Some(function) = current_function {
            if let Some(index) = function
                .params
                .iter()
                .position(|(parameter, _)| same_ident(&parameter.name, name))
            {
                if let Some(suffix) = function
                    .semantic_parameters
                    .as_ref()?
                    .get(index)?
                    .value_type
                    .suffix()
                    .and_then(TypeSuffix::from_char)
                {
                    return Some(suffix);
                }
            }
            if let Some(declaration) = function.semantic_dim_declarations.get(&key) {
                return suffix_from_type(declaration.element_type);
            }
        }
        if let Some(declaration) = self.semantic_top_level_dims.get(&key) {
            return suffix_from_type(declaration.element_type);
        }
        None
    }

    /// Bound text for one axis of a known array: a frozen DIM-time bound
    /// for a directly-`dim`ed array (local or top-level), or -- for an
    /// array *parameter* -- the transpiler-synthesized hidden variable that
    /// the caller sets (from the actual argument's own resolved bound)
    /// immediately before `GOSUB`. `None` means the axis genuinely can't
    /// be resolved (unknown array, or an unsized `dim arr%()` with no
    /// bounds to freeze).
    fn resolve_axis_bound(
        &self,
        name: &BasicIdent,
        axis: usize,
        current_function: Option<&FunctionInfo>,
    ) -> Option<String> {
        if let Some(info) = current_function {
            if let Some(index) = info
                .params
                .iter()
                .position(|(p, _)| same_ident(&p.name, name))
            {
                return info
                    .param_bound_vars
                    .get(index)
                    .and_then(|bounds| bounds.get(axis))
                    .cloned();
            }
            let key = name.as_basic().to_ascii_lowercase();
            if let Some(bound) = info
                .local_array_bounds
                .borrow()
                .get(&key)
                .and_then(|b| b.get(axis))
            {
                return Some(bound.clone());
            }
        }
        let key = name.as_basic().to_ascii_lowercase();
        self.top_level_array_bounds
            .get(&key)
            .and_then(|b| b.get(axis))
            .cloned()
    }

    /// Shared by `sizeof`/`LBOUND`/`UBOUND`: validates `name` is a known
    /// array and `axis_expr` names one of its real axes (a literal
    /// integer, defaulting to `0` for a 1-D array; required and checked
    /// in range for 2-D+), then resolves that axis's raw `DIM` bound the
    /// same way `resolve_axis_bound` always has -- a real top-level
    /// array's literal-or-captured-at-DIM-time bound, or (inside a
    /// function, for one of its own array parameters) the hidden
    /// auto-injected bound variable the caller sets immediately before
    /// `GOSUB`. `UBOUND` exposes this value directly; `sizeof` adds 1 to
    /// it (see `resolve_sizeof`'s own doc comment); `LBOUND` ignores it
    /// entirely and always resolves to the literal `0` (see
    /// `resolve_lbound`'s own doc comment) -- but still needs the same
    /// validation, so an unknown array or a bad axis is still a clear
    /// error rather than a silently-wrong `0`.
    fn resolve_array_bound_for_builtin(
        &self,
        builtin_name: &str,
        name: &BasicIdent,
        axis_expr: Option<&Expr>,
        current_function: Option<&FunctionInfo>,
    ) -> Result<String, String> {
        let rank = self
            .resolve_array_rank(name, current_function)
            .ok_or_else(|| {
                format!(
                    "`{name}` isn't a known array, so `{builtin_name}` can't determine its size"
                )
            })?;

        let axis = match axis_expr {
            Some(Expr::Integer(n)) => *n as usize,
            Some(_) => {
                return Err(format!(
                    "the axis argument to `{builtin_name}` must be a literal integer"
                ));
            }
            None if rank == 1 => 0,
            None => {
                return Err(format!(
                    "`{name}` has {rank} dimensions -- {builtin_name} needs an axis argument, \
                     e.g. `{builtin_name}({name}, 0)`"
                ));
            }
        };
        if axis >= rank {
            return Err(format!(
                "`{name}` only has {rank} dimension{} -- axis {axis} doesn't exist",
                if rank == 1 { "" } else { "s" }
            ));
        }

        self.resolve_axis_bound(name, axis, current_function)
            .ok_or_else(|| format!("could not determine the size of `{name}`"))
    }

    /// Resolves `UBOUND(name)` / `UBOUND(name, axis)` -- the array's real
    /// declared bound along an axis (its highest valid index), exactly
    /// the value `resolve_array_bound_for_builtin` gives back.
    fn resolve_ubound(
        &self,
        name: &BasicIdent,
        axis_expr: Option<&Expr>,
        current_function: Option<&FunctionInfo>,
    ) -> Result<String, String> {
        self.resolve_array_bound_for_builtin("UBOUND", name, axis_expr, current_function)
    }

    /// Resolves `LBOUND(name)` / `LBOUND(name, axis)` -- always the
    /// literal `0`, since BASCAL only supports base-0 array indexing
    /// (`OPTION BASE` is rejected outright -- see GitHub issue #50). Still
    /// runs the same array-known/axis-valid validation
    /// `resolve_array_bound_for_builtin` does for `UBOUND`/`sizeof`, so
    /// `LBOUND(nope%)` is a clear error, not a silently-wrong `0`.
    fn resolve_lbound(
        &self,
        name: &BasicIdent,
        axis_expr: Option<&Expr>,
        current_function: Option<&FunctionInfo>,
    ) -> Result<String, String> {
        self.resolve_array_bound_for_builtin("LBOUND", name, axis_expr, current_function)?;
        Ok("0".to_string())
    }

    /// Resolves `sizeof(name)` / `sizeof(name, axis)` to the text that
    /// should replace the call in generated code -- the array's real
    /// element *count* along this axis, i.e. `UBOUND(name, axis) + 1`
    /// (real BASIC's own inclusive-bound convention: a `DIM arr%(N)` axis
    /// holds `N + 1` elements, indices `0..=N`, so the bound alone is one
    /// short of the count). Adding 1 to an already-resolved integer
    /// literal keeps the common case's generated code a plain literal
    /// rather than a `(9 + 1)` expression; a captured runtime bound (a
    /// variable name, from a non-literal `DIM` size, or an array
    /// parameter's own hidden bound variable) still needs the arithmetic
    /// spelled out.
    fn resolve_sizeof(
        &self,
        name: &BasicIdent,
        axis_expr: Option<&Expr>,
        current_function: Option<&FunctionInfo>,
    ) -> Result<String, String> {
        let bound =
            self.resolve_array_bound_for_builtin("SIZEOF", name, axis_expr, current_function)?;
        Ok(match bound.parse::<i64>() {
            Ok(n) => n
                .checked_add(1)
                .ok_or_else(|| {
                    format!("the element count of `{name}` overflows the supported integer range")
                })?
                .to_string(),
            Err(_) => format!("({bound} + 1)"),
        })
    }

    fn render_print_tokens(
        &mut self,
        tokens: &[PrintToken],
        current_function: Option<&FunctionInfo>,
    ) -> String {
        let mut out = String::new();
        // after_sep: push a space BEFORE the next Expr (readable: `; x%` not `;x%`)
        // Starts false so the very first Expr gets no leading space.
        let mut after_sep = false;
        for token in tokens {
            match token {
                PrintToken::Expr(e) => {
                    let (prelude, rendered) = self.expr(e, current_function);
                    self.lines(prelude);
                    if after_sep {
                        out.push(' ');
                    }
                    out.push_str(&rendered);
                    after_sep = false;
                }
                PrintToken::Semi => {
                    out.push(';');
                    after_sep = true;
                }
                PrintToken::Comma => {
                    out.push(',');
                    after_sep = true;
                }
            }
        }
        out
    }

    fn lines(&mut self, lines: Vec<String>) {
        for line in lines {
            self.line(&line);
        }
    }

    fn line(&mut self, line: &str) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
        self.output.push_str(line);
        self.output.push('\n');
    }

    fn blank(&mut self) {
        self.output.push('\n');
    }

    /// Appends the shared `catch err%, erl%, source$` lookup subroutine,
    /// entered via `GOSUB` from every catch site that binds a source-
    /// filename variable (see `try_catch`). `breakpoints` comes from a
    /// throwaway first `number_basic_lines` pass over the program built so
    /// far (see `generate()`'s own call site) -- each entry is the highest
    /// final line number reached while still inside one contiguous run of
    /// lines from the same original `.bcl` file, in ascending order, so
    /// `ERL <= bound` correctly identifies the file for any line number
    /// the program could ever actually assign to `ERL`: BASIC only ever
    /// reports a line number that exists, and this backend always emits
    /// one file's statements as one contiguous run (see
    /// `source_file_marker`'s own doc comment), so the breakpoints
    /// partition every possible `ERL` value without gaps.
    ///
    /// Appending this strictly after everything `generate()` has emitted
    /// so far -- and only once every function/procedure (each of which
    /// always ends in a real `RETURN`, never fallthrough) or a synthesized
    /// top-level `END` guarantees this is never reached except via its own
    /// `GOSUB` -- is what lets the breakpoints computed against the
    /// pre-append text stay correct once this subroutine's own lines are
    /// numbered alongside everything else: sequential numbering assigns
    /// identical numbers to every pre-existing line either way, since
    /// nothing here can turn an earlier line into (or out of) a branch
    /// target.
    fn emit_source_file_lookup_subroutine(&mut self, breakpoints: &[(usize, String)]) {
        self.blank();
        self.line("' catch's optional source$ binding: map ERL back to its original .bcl file");
        self.line("BCC_RESOLVE_SOURCE_FILE:");
        self.indent += 1;
        match breakpoints.split_last() {
            Some((last, rest)) => {
                for (bound, file) in rest {
                    self.line(&format!(
                        "IF ERL <= {bound} THEN BCCSOURCEFILE$ = \"{}\" : RETURN",
                        escape_string(file)
                    ));
                }
                self.line(&format!("BCCSOURCEFILE$ = \"{}\"", escape_string(&last.1)));
            }
            None => self.line("BCCSOURCEFILE$ = \"\""),
        }
        self.line("RETURN");
        self.indent -= 1;
    }
}

fn semantic_record_field<'a>(
    records: &'a [crate::semantic_ir::Record],
    record: &'a crate::semantic_ir::Record,
    member: &str,
) -> Option<&'a crate::semantic_ir::RecordField> {
    for combined in &record.combines {
        let combined_record = records
            .iter()
            .find(|candidate| candidate.name.eq_ignore_ascii_case(combined))?;
        if let Some(field) = semantic_record_field(records, combined_record, member) {
            return Some(field);
        }
    }
    record
        .fields
        .iter()
        .find(|field| field.name.eq_ignore_ascii_case(member))
}

fn collect_semantic_record_fields<'a>(
    records: &'a [crate::semantic_ir::Record],
    record: &'a crate::semantic_ir::Record,
    fields: &mut Vec<&'a crate::semantic_ir::RecordField>,
) {
    for combined in &record.combines {
        if let Some(combined_record) = records
            .iter()
            .find(|candidate| candidate.name.eq_ignore_ascii_case(combined))
        {
            collect_semantic_record_fields(records, combined_record, fields);
        }
    }
    fields.extend(record.fields.iter());
}

/// A hidden, never-emitted marker line `statement()` inserts right before
/// the first statement of every run of lines from a new original `.bcl`
/// file (tracked via `CodeGenerator::current_marker_file`), only when
/// `needs_source_lookup` is set. `number_basic_lines` reads these to learn
/// which file each final line number came from, then strips them before
/// producing real output -- they use a `'` (comment) prefix as a safety
/// net (so anything that ever saw one unstripped would still see valid,
/// harmless BASIC) plus a `\u{1}` control byte real `.bcl` source could
/// never lex as a comment's own text, so a marker can never collide with
/// an author's own comment.
fn source_file_marker(file: &str) -> String {
    format!("'\u{1}{file}")
}

fn parse_source_file_marker(line: &str) -> Option<&str> {
    line.trim_start().strip_prefix("'\u{1}")
}

impl FunctionInfo {
    fn from_def(
        function: &FunctionDef,
        taken: &mut HashSet<String>,
        known_callables: &HashSet<String>,
        diagnostics: &mut Vec<Diagnostic>,
        mut param_capacities: Vec<Vec<i64>>,
        semantic_param_ranks: Option<Vec<Option<usize>>>,
        semantic_array_ranks: Option<HashMap<String, usize>>,
        semantic_globals: Option<HashSet<String>>,
        semantic_parameters: Option<Vec<crate::semantic_ir::Parameter>>,
        semantic_signature: Option<&crate::semantic_ir::CallableSignature>,
        semantic_module: Option<&crate::semantic_ir::SemanticModule>,
    ) -> Self {
        // A scalar method's semantic name is bare; keep the result suffix the
        // callable table and call sites are keyed by.
        let source_name = semantic_signature
            .map(|callable| {
                crate::semantic_ir::callable_ident_for_function(&callable.name, function)
            })
            .unwrap_or_else(|| function.name.clone());
        let receiver = semantic_signature
            .map(|callable| {
                match callable
                    .receiver
                    .as_deref()
                    .map(str::to_ascii_lowercase)
                    .as_deref()
                {
                    Some("integer") => Some(TypeSuffix::Integer),
                    Some("long") => Some(TypeSuffix::Long),
                    Some("single") => Some(TypeSuffix::Single),
                    Some("double") => Some(TypeSuffix::Double),
                    Some("string") => Some(TypeSuffix::String),
                    _ => None,
                }
            })
            .unwrap_or(function.receiver);
        let stem = sanitize_symbol(&source_name.name);
        let parameter_specs = semantic_parameters
            .as_ref()
            .map(|parameters| {
                parameters
                    .iter()
                    .map(|parameter| {
                        let mut name = BasicIdent::parse(&parameter.name);
                        let suffix = parameter
                            .value_type
                            .suffix()
                            .and_then(TypeSuffix::from_char);
                        if suffix.is_none() {
                            diagnostics.push(Diagnostic::error(
                                semantic_callable_diagnostic_pos(
                                    semantic_module,
                                    semantic_signature,
                                    parameter.span,
                                ),
                                format!(
                                    "parameter `{}` of `{}` has no resolved BASIC scalar type",
                                    parameter.name, source_name
                                ),
                            ));
                        }
                        name.suffix = suffix;
                        let mode = match parameter.passing {
                            Some(crate::semantic_ir::Passing::ByRef) => ParamMode::ByRef,
                            Some(crate::semantic_ir::Passing::ByVal) | None => ParamMode::ByVal,
                        };
                        (name, mode, None)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| {
                function
                    .params
                    .iter()
                    .map(|parameter| {
                        (
                            parameter.name.clone(),
                            parameter.mode,
                            parameter.default.clone(),
                        )
                    })
                    .collect()
            });
        let mut params: Vec<(FunctionParameterInfo, BasicIdent)> = parameter_specs
            .iter()
            .map(|(name, mode, default)| {
                let preferred = camel_join(&[&stem, &name.name]);
                let lowered = allocate_unique(&preferred, name.suffix, taken);
                taken.insert(lowered.as_basic().to_ascii_lowercase());
                (
                    FunctionParameterInfo {
                        name: name.clone(),
                        mode: *mode,
                        default: default.clone(),
                    },
                    lowered,
                )
            })
            .collect();
        // A complete resolved signature is authoritative, including scalar
        // parameters (`None` rank). Infer from calls only for AST-only callers
        // or an adapter result that does not cover the whole declaration.
        let mut param_ranks = semantic_parameters
            .as_ref()
            .map(|parameters| {
                parameters
                    .iter()
                    .map(|parameter| (parameter.array_axes > 0).then_some(parameter.array_axes))
                    .collect()
            })
            .unwrap_or_else(|| {
                match semantic_param_ranks.filter(|ranks| ranks.len() == parameter_specs.len()) {
                    Some(ranks) => ranks,
                    None => infer_param_ranks(function, known_callables, diagnostics),
                }
            });
        let mut param_bound_vars: Vec<Vec<String>> = params
            .iter()
            .zip(param_ranks.iter())
            .map(|((param, _), rank)| match rank {
                Some(rank) => (0..*rank)
                    .map(|axis| {
                        let preferred =
                            camel_join(&[&stem, &param.name.name, &format!("dim{axis}")]);
                        let lowered = allocate_unique(&preferred, Some(TypeSuffix::Integer), taken);
                        taken.insert(lowered.as_basic().to_ascii_lowercase());
                        lowered.as_basic()
                    })
                    .collect(),
                None => Vec::new(),
            })
            .collect();
        if let Some(receiver) = receiver {
            let self_param = FunctionParameterInfo {
                name: BasicIdent {
                    name: "self".to_string(),
                    suffix: Some(receiver),
                },
                mode: ParamMode::ByVal,
                default: None,
            };
            let preferred = camel_join(&[&stem, "self"]);
            let lowered = allocate_unique(&preferred, Some(receiver), taken);
            taken.insert(lowered.as_basic().to_ascii_lowercase());
            params.insert(0, (self_param, lowered));
            param_ranks.insert(0, None);
            param_bound_vars.insert(0, Vec::new());
            param_capacities.insert(0, Vec::new());
        }
        let local_array_ranks = match (semantic_signature, semantic_array_ranks) {
            (Some(_), Some(ranks)) => ranks,
            (Some(_), None) => HashMap::new(),
            (None, Some(ranks)) => ranks,
            (None, None) => dim_ranks_in_body(&function.body),
        };
        let result_suffix = semantic_signature
            .map(|callable| {
                callable
                    .result_type
                    .as_deref()
                    .and_then(|suffix| suffix.chars().next())
                    .and_then(TypeSuffix::from_char)
            })
            .unwrap_or_else(|| {
                semantic_signature
                    .is_none()
                    .then_some(source_name.suffix)
                    .flatten()
            });
        let is_procedure = semantic_signature
            .map(|callable| callable.kind == crate::semantic_ir::CallableKind::Procedure)
            .unwrap_or(function.is_procedure);
        if semantic_signature.is_some() && !is_procedure && result_suffix.is_none() {
            diagnostics.push(Diagnostic::error(
                semantic_callable_diagnostic_pos(
                    semantic_module,
                    semantic_signature,
                    semantic_signature.map_or_else(
                        || crate::rdgen_frontend::SourceSpan { start: 0, end: 0 },
                        |callable| callable.name_span,
                    ),
                ),
                format!("function `{source_name}` has no resolved BASIC result type"),
            ));
        }
        let result = allocate_unique(&camel_join(&[&stem, "result"]), result_suffix, taken);
        taken.insert(result.as_basic().to_ascii_lowercase());
        let globals = match (semantic_signature, semantic_globals) {
            (Some(_), Some(globals)) => globals,
            (Some(_), None) => HashSet::new(),
            (None, Some(globals)) => globals,
            (None, None) => collect_globals(&function.body),
        };
        Self {
            source_name,
            stem: stem.clone(),
            label: function_label(&stem, result_suffix, receiver),
            result,
            params,
            param_ranks,
            param_bound_vars,
            param_capacities,
            local_array_ranks,
            semantic_const_initializers: HashMap::new(),
            semantic_parameters,
            semantic_dim_declarations: HashMap::new(),
            local_array_bounds: RefCell::new(HashMap::new()),
            is_procedure,
            receiver,
            globals,
            local_var_map: RefCell::new(HashMap::new()),
        }
    }
}

/// Visits every `Expr` node (recursively, including sub-expressions) that
/// appears anywhere in `body` -- every statement kind, every clause. Used to
/// find every array-element access to a given parameter, wherever it
/// appears in a function body, so its declared rank can be inferred from
/// how many indices it's actually used with.
pub(crate) fn visit_body_exprs<'a>(body: &'a [Stmt], f: &mut impl FnMut(&'a Expr)) {
    for stmt in body {
        visit_statement_exprs(stmt, f);
    }
}

fn visit_statement_exprs<'a>(stmt: &'a Stmt, f: &mut impl FnMut(&'a Expr)) {
    match &stmt.kind {
        Statement::Dim { sizes, .. } => {
            for e in sizes {
                visit_expr(e, f);
            }
        }
        Statement::Open {
            file, channel, len, ..
        } => {
            visit_expr(file, f);
            visit_expr(channel, f);
            if let Some(e) = len {
                visit_expr(e, f);
            }
        }
        Statement::FileDecl { path, .. } => visit_expr(path, f),
        Statement::LineInput { channel, target } => {
            visit_expr(channel, f);
            visit_expr(target, f);
        }
        Statement::PrintFile { channel, tokens } => {
            visit_expr(channel, f);
            visit_print_tokens(tokens, f);
        }
        Statement::PrintUsing { format, tokens } => {
            visit_expr(format, f);
            visit_print_tokens(tokens, f);
        }
        Statement::PrintFileUsing {
            channel,
            format,
            tokens,
        } => {
            visit_expr(channel, f);
            visit_expr(format, f);
            visit_print_tokens(tokens, f);
        }
        Statement::Close { channel } => visit_expr(channel, f),
        Statement::Kill { file } => visit_expr(file, f),
        Statement::Name { from, to } => {
            visit_expr(from, f);
            visit_expr(to, f);
        }
        Statement::Assignment { target, value } => {
            visit_expr(target, f);
            visit_expr(value, f);
        }
        Statement::MidAssign {
            target,
            start,
            len,
            value,
        } => {
            visit_expr(target, f);
            visit_expr(start, f);
            if let Some(e) = len {
                visit_expr(e, f);
            }
            visit_expr(value, f);
        }
        Statement::Print { tokens } => visit_print_tokens(tokens, f),
        Statement::Return { value } => visit_expr(value, f),
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            visit_expr(condition, f);
            visit_body_exprs(then_body, f);
            visit_body_exprs(else_body, f);
        }
        Statement::For {
            start,
            end,
            step,
            body,
            ..
        } => {
            visit_expr(start, f);
            visit_expr(end, f);
            if let Some(e) = step {
                visit_expr(e, f);
            }
            visit_body_exprs(body, f);
        }
        Statement::While { condition, body } => {
            visit_expr(condition, f);
            visit_body_exprs(body, f);
        }
        Statement::Do {
            condition,
            body,
            post_condition,
        } => {
            if let Some(c) = condition {
                visit_expr(&c.expr, f);
            }
            visit_body_exprs(body, f);
            if let Some(c) = post_condition {
                visit_expr(&c.expr, f);
            }
        }
        Statement::ExprStmt(e) => visit_expr(e, f),
        Statement::OptionBase(e) => visit_expr(e, f),
        Statement::Erase(_) => {}
        Statement::Randomize(e) => {
            if let Some(e) = e {
                visit_expr(e, f);
            }
        }
        Statement::Swap(a, b) => {
            visit_expr(a, f);
            visit_expr(b, f);
        }
        Statement::Poke { address, value } => {
            visit_expr(address, f);
            visit_expr(value, f);
        }
        Statement::Goto(e) | Statement::Gosub(e) => visit_expr(e, f),
        Statement::OnErrorGoto { target } => visit_expr(target, f),
        Statement::Resume(kind) => {
            if let ResumeTarget::Line(e) = kind {
                visit_expr(e, f);
            }
        }
        Statement::ErrorStmt { code } => visit_expr(code, f),
        Statement::ThrowStmt { code } => {
            if let Some(code) = code {
                visit_expr(code, f);
            }
        }
        Statement::Input { vars, .. } => {
            for e in vars {
                visit_expr(e, f);
            }
        }
        Statement::InputFile { channel, vars } => {
            visit_expr(channel, f);
            for e in vars {
                visit_expr(e, f);
            }
        }
        Statement::Data(values) | Statement::Read(values) => {
            for e in values {
                visit_expr(e, f);
            }
        }
        Statement::Restore(e) => {
            if let Some(e) = e {
                visit_expr(e, f);
            }
        }
        Statement::Const { value, .. } => visit_expr(value, f),
        Statement::Write { channel, exprs } => {
            visit_expr(channel, f);
            for e in exprs {
                visit_expr(e, f);
            }
        }
        Statement::Field {
            channel, fields, ..
        } => {
            visit_expr(channel, f);
            for (w, _) in fields {
                visit_expr(w, f);
            }
        }
        Statement::Get {
            channel,
            record,
            var,
            ..
        }
        | Statement::Put {
            channel,
            record,
            var,
            ..
        } => {
            visit_expr(channel, f);
            if let Some(e) = record {
                visit_expr(e, f);
            }
            if let Some(e) = var {
                visit_expr(e, f);
            }
        }
        Statement::Lset { value, .. } | Statement::Rset { value, .. } => visit_expr(value, f),
        Statement::Seek { channel, position } => {
            visit_expr(channel, f);
            visit_expr(position, f);
        }
        Statement::Lprint(tokens) => visit_print_tokens(tokens, f),
        Statement::LprintUsing { format, tokens } => {
            visit_expr(format, f);
            visit_print_tokens(tokens, f);
        }
        Statement::SelectCase {
            expr,
            cases,
            else_body,
        } => {
            visit_expr(expr, f);
            for case in cases {
                for v in &case.values {
                    match v {
                        CaseValue::Single(e) | CaseValue::Is { value: e, .. } => visit_expr(e, f),
                        CaseValue::Range { from, to } => {
                            visit_expr(from, f);
                            visit_expr(to, f);
                        }
                    }
                }
                visit_body_exprs(&case.body, f);
            }
            visit_body_exprs(else_body, f);
        }
        Statement::TryCatch {
            try_body,
            catch,
            finally_body,
            ..
        } => {
            visit_body_exprs(try_body, f);
            if let Some(catch) = catch {
                for filter_expr in &catch.error_filter {
                    visit_expr(filter_expr, f);
                }
                visit_body_exprs(&catch.body, f);
            }
            visit_body_exprs(finally_body, f);
        }
        Statement::Locate { row, col } => {
            visit_expr(row, f);
            visit_expr(col, f);
        }
        Statement::Color { fg, bg } => {
            visit_expr(fg, f);
            if let Some(e) = bg {
                visit_expr(e, f);
            }
        }
        Statement::OnBranch { expr, targets, .. } => {
            visit_expr(expr, f);
            for e in targets {
                visit_expr(e, f);
            }
        }
        Statement::Out { port, value } => {
            visit_expr(port, f);
            visit_expr(value, f);
        }
        Statement::Width { channel, cols } => {
            if let Some(e) = channel {
                visit_expr(e, f);
            }
            visit_expr(cols, f);
        }
        Statement::End
        | Statement::Stop
        | Statement::Cls
        | Statement::Beep
        | Statement::System
        | Statement::Clear
        | Statement::ReturnVoid
        | Statement::GlobalDecl(_)
        | Statement::Raw(_)
        | Statement::BlockComment(_)
        | Statement::Label(_)
        | Statement::BlankLine
        | Statement::Exit
        | Statement::Continue => {}
    }
}

fn visit_print_tokens<'a>(tokens: &'a [PrintToken], f: &mut impl FnMut(&'a Expr)) {
    for t in tokens {
        if let PrintToken::Expr(e) = t {
            visit_expr(e, f);
        }
    }
}

fn visit_expr<'a>(expr: &'a Expr, f: &mut impl FnMut(&'a Expr)) {
    f(expr);
    match expr {
        Expr::Integer(_) | Expr::Float(_) | Expr::HexLit(_) | Expr::String(_) | Expr::Ident(_) => {}
        Expr::ArrayRef { indices, .. } => {
            for e in indices {
                visit_expr(e, f);
            }
        }
        Expr::Call { args, .. } => {
            for e in args {
                visit_expr(e, f);
            }
        }
        Expr::Unary { expr, .. } => visit_expr(expr, f),
        Expr::Binary { left, right, .. } => {
            visit_expr(left, f);
            visit_expr(right, f);
        }
        Expr::FileIndex { index, .. } => visit_expr(index, f),
        Expr::FieldAccess { base, .. } => visit_expr(base, f),
        Expr::MethodCall { base, args, .. } => {
            visit_expr(base, f);
            for e in args {
                visit_expr(e, f);
            }
        }
        Expr::ScalarMethodCall { base, args, .. } => {
            visit_expr(base, f);
            for e in args {
                visit_expr(e, f);
            }
        }
        Expr::RecordLit { fields, .. } => {
            for (_, e) in fields {
                visit_expr(e, f);
            }
        }
    }
}

/// Infers each parameter's array rank (number of subscripts) from how it's
/// actually indexed inside the function's own body -- there's no type
/// annotation to read it from directly. `None` means either the parameter
/// is never directly indexed in this body (e.g. it's only ever forwarded
/// on as a whole array to another call), or it's indexed inconsistently,
/// which is reported as its own diagnostic.
fn infer_param_ranks(
    function: &FunctionDef,
    known_callables: &HashSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Option<usize>> {
    function
        .params
        .iter()
        .map(|param| {
            let mut ranks: HashSet<usize> = HashSet::new();
            visit_body_exprs(&function.body, &mut |e| {
                // `make_paren_ident_expr` in parser.rs only produces
                // ArrayRef for empty parens or exactly one index; anything
                // else -- including every 2+ dimensional array access --
                // parses as a Call, disambiguated from a real function call
                // later by name. Both shapes need checking here.
                let (name, count) = match e {
                    Expr::ArrayRef { name, indices } if !indices.is_empty() => {
                        (name, indices.len())
                    }
                    Expr::Call { name, args }
                        if !known_callables.contains(&name.name.to_ascii_lowercase()) =>
                    {
                        (name, args.len())
                    }
                    _ => return,
                };
                if same_ident(name, &param.name) {
                    ranks.insert(count);
                }
            });
            let usage_rank = match ranks.len() {
                0 => None,
                1 => ranks.into_iter().next(),
                _ => {
                    diagnostics.push(Diagnostic::error(
                        SourcePos::new("<validation>", 1, 1),
                        format!(
                            "parameter `{}` of `{}` is indexed with different numbers of \
                             subscripts in different places -- BASCAL can't tell how many \
                             dimensions it has",
                            param.name, function.name
                        ),
                    ));
                    None
                }
            };

            // The declaration (`arr%(?)`, `arr%(?, ?)`, ...) is the
            // authoritative rank when present. Body usage is still checked
            // against it -- an array parameter with no declared rank at
            // all is rejected outright, since there's no other way to
            // learn a parameter's rank from its declaration.
            match (param.rank(), usage_rank) {
                (Some(declared), Some(used)) if declared != used => {
                    diagnostics.push(Diagnostic::error(
                        SourcePos::new("<validation>", 1, 1),
                        format!(
                            "parameter `{}` of `{}` is declared with {} dimension{} but indexed \
                             with {} subscript{} in the body",
                            param.name,
                            function.name,
                            declared,
                            if declared == 1 { "" } else { "s" },
                            used,
                            if used == 1 { "" } else { "s" },
                        ),
                    ));
                    None
                }
                (Some(declared), _) => Some(declared),
                (None, Some(used)) => {
                    let placeholders = vec!["?"; used].join(", ");
                    diagnostics.push(Diagnostic::error(
                        SourcePos::new("<validation>", 1, 1),
                        format!(
                            "parameter `{}` of `{}` is indexed as a {}-D array in the body, but \
                             its declaration doesn't say so -- write `{}({})`",
                            param.name,
                            function.name,
                            used,
                            param.name.as_basic(),
                            placeholders,
                        ),
                    ));
                    None
                }
                (None, None) => None,
            }
        })
        .collect()
}

/// Whether any `catch` anywhere in `program` binds the optional third
/// (source-filename) variable -- decides whether `generate()` needs to
/// track per-statement source files at all (see `TryCatchHandler::
/// source_var`'s own doc comment). Recurses into every nested statement
/// body, top-level and every function/procedure's own alike, so a program
/// gets exactly the same output it always has unless it actually writes
/// `catch err%, erl%, source$` somewhere.
pub(crate) fn program_uses_catch_source_var(program: &Program) -> bool {
    statements_use_catch_source_var(&program.statements)
        || program
            .functions
            .iter()
            .any(|f| statements_use_catch_source_var(&f.body))
}

fn statements_use_catch_source_var(statements: &[Stmt]) -> bool {
    statements.iter().any(|stmt| match &stmt.kind {
        Statement::TryCatch {
            try_body,
            catch,
            finally_body,
        } => {
            catch.as_ref().is_some_and(|c| c.source_var.is_some())
                || statements_use_catch_source_var(try_body)
                || catch
                    .as_ref()
                    .is_some_and(|c| statements_use_catch_source_var(&c.body))
                || statements_use_catch_source_var(finally_body)
        }
        Statement::If {
            then_body,
            else_body,
            ..
        } => {
            statements_use_catch_source_var(then_body) || statements_use_catch_source_var(else_body)
        }
        Statement::For { body, .. }
        | Statement::While { body, .. }
        | Statement::Do { body, .. } => statements_use_catch_source_var(body),
        Statement::SelectCase {
            cases, else_body, ..
        } => {
            cases
                .iter()
                .any(|case| statements_use_catch_source_var(&case.body))
                || statements_use_catch_source_var(else_body)
        }
        _ => false,
    })
}

/// Declared rank (number of DIM dimensions) of every array DIMed anywhere
/// in `body`, lowercase name -> rank. `dim arr%()` (no bounds written) has
/// no rank recorded here -- there's nothing to check it against.
pub(crate) fn dim_ranks_in_body(body: &[Stmt]) -> HashMap<String, usize> {
    let mut ranks = HashMap::new();
    collect_dim_ranks(body, &mut ranks);
    ranks
}

fn collect_dim_ranks(body: &[Stmt], out: &mut HashMap<String, usize>) {
    for stmt in body {
        match &stmt.kind {
            Statement::Dim {
                name,
                is_array,
                sizes,
            } => {
                if *is_array && !sizes.is_empty() {
                    out.insert(name.as_basic().to_ascii_lowercase(), sizes.len());
                }
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_dim_ranks(then_body, out);
                collect_dim_ranks(else_body, out);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_dim_ranks(body, out),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_dim_ranks(&case.body, out);
                }
                collect_dim_ranks(else_body, out);
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                collect_dim_ranks(try_body, out);
                if let Some(catch) = catch {
                    collect_dim_ranks(&catch.body, out);
                }
                collect_dim_ranks(finally_body, out);
            }
            _ => {}
        }
    }
}

// ── array parameter storage capacity inference ──────────────────────────
//
// Every array parameter's shared storage array is DIMed exactly once, at
// top-level, before any call happens (classic BASIC has no REDIM, so a
// shared storage slot can never be resized once DIMed). Its size has to be
// a fixed capacity, decided once, big enough for the largest thing any
// call site ever passes it. This section computes that capacity: for a
// `?` axis, the max of every call site's resolved bound, but only when
// every one of those bounds is itself resolvable at compile time (a
// literal, a `const`, or -- when the array being passed is itself another
// function's array parameter being forwarded onward -- that parameter's
// own already-resolved capacity). An axis that can't be resolved this way
// needs an explicit literal capacity written in the declaration instead.

/// One axis's resolved bound for a single call site's array argument.
enum ArgBound {
    /// This argument isn't array-shaped at all (a scalar, or a rank
    /// mismatch that a later, more specific diagnostic will catch) --
    /// contributes nothing, doesn't count as a data point either way.
    NotAnArray,
    /// It's an array, but this axis's bound can't be pinned to a concrete
    /// integer -- a genuinely dynamic (runtime) value, or a forwarded
    /// parameter whose own capacity hasn't resolved yet this round.
    Unresolvable,
    Resolved(i64),
}

#[derive(Clone)]
enum CapacityCallArgument {
    Legacy(Expr),
    Semantic(crate::semantic_ir::Expression),
}

/// Evaluates `expr` to a concrete integer if it's a compile-time constant:
/// a literal, a reference to an unambiguous `const` (recursively), or
/// +/-/*// on two such values. Anything else (a plain variable, a function
/// call, an ambiguous multiply-defined `const` name) is `None` -- a
/// genuine runtime value, not something this pass can reason about.
fn const_eval(expr: &Expr, consts: &HashMap<String, Vec<Expr>>, depth: u32) -> Option<i64> {
    if depth > 32 {
        return None; // guards against a self-referential `const`
    }
    match expr {
        Expr::Integer(n) => Some(*n),
        Expr::Ident(name) => {
            let key = name.as_basic().to_ascii_lowercase();
            match consts.get(&key) {
                Some(defs) if defs.len() == 1 => const_eval(&defs[0], consts, depth + 1),
                _ => None,
            }
        }
        Expr::Unary {
            op: UnaryOp::Neg,
            expr,
        } => const_eval(expr, consts, depth + 1).map(|v| -v),
        Expr::Binary { left, op, right } => {
            let l = const_eval(left, consts, depth + 1)?;
            let r = const_eval(right, consts, depth + 1)?;
            match op {
                BinaryOp::Add => Some(l.wrapping_add(r)),
                BinaryOp::Sub => Some(l.wrapping_sub(r)),
                BinaryOp::Mul => Some(l.wrapping_mul(r)),
                BinaryOp::Div if r != 0 => Some(l / r),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Every `const` declaration anywhere in `body` (recursing into nested
/// blocks), keyed by lowercase name. More than one definition under the
/// same name is tracked (not merged) so `const_eval` can refuse to guess
/// which one a reference means.
pub(crate) fn collect_consts(body: &[Stmt], out: &mut HashMap<String, Vec<Expr>>) {
    for stmt in body {
        match &stmt.kind {
            Statement::Const { name, value } => {
                out.entry(name.as_basic().to_ascii_lowercase())
                    .or_default()
                    .push(value.clone());
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_consts(then_body, out);
                collect_consts(else_body, out);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_consts(body, out),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_consts(&case.body, out);
                }
                collect_consts(else_body, out);
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                collect_consts(try_body, out);
                if let Some(catch) = catch {
                    collect_consts(&catch.body, out);
                }
                collect_consts(finally_body, out);
            }
            _ => {}
        }
    }
}

/// Every `dim`ed array's full size-expression list anywhere in `body`
/// (recursing into nested blocks), keyed by lowercase name -- the same
/// traversal as `collect_dim_ranks`, but keeping the bound expressions
/// themselves instead of just their count.
fn collect_dim_sizes(body: &[Stmt], out: &mut HashMap<String, Vec<Expr>>) {
    for stmt in body {
        match &stmt.kind {
            Statement::Dim {
                name,
                is_array,
                sizes,
            } => {
                if *is_array && !sizes.is_empty() {
                    out.insert(name.as_basic().to_ascii_lowercase(), sizes.clone());
                }
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_dim_sizes(then_body, out);
                collect_dim_sizes(else_body, out);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_dim_sizes(body, out),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_dim_sizes(&case.body, out);
                }
                collect_dim_sizes(else_body, out);
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                collect_dim_sizes(try_body, out);
                if let Some(catch) = catch {
                    collect_dim_sizes(&catch.body, out);
                }
                collect_dim_sizes(finally_body, out);
            }
            _ => {}
        }
    }
}

/// Every call site anywhere in the program that calls one of the
/// program's own functions: `(enclosing function name, lowercase -- None
/// for top-level, callee name, argument list)`. A single-argument call
/// (`f%(x%)`) parses as `Expr::ArrayRef`, not `Expr::Call` (see
/// `make_paren_ident_expr` in `parser.rs`), so both shapes are checked.
fn collect_call_sites(
    program: &Program,
    function_names: &HashSet<String>,
) -> Vec<(Option<String>, BasicIdent, Vec<Expr>)> {
    let mut sites = Vec::new();
    let mut scan = |scope: Option<String>, body: &[Stmt]| {
        let mut visit = |e: &Expr| match e {
            Expr::Call { name, args }
                if function_names.contains(&name.name.to_ascii_lowercase()) =>
            {
                sites.push((scope.clone(), name.clone(), args.clone()));
            }
            Expr::ArrayRef { name, indices }
                if function_names.contains(&name.name.to_ascii_lowercase()) =>
            {
                sites.push((scope.clone(), name.clone(), indices.clone()));
            }
            _ => {}
        };
        visit_body_exprs(body, &mut visit);
    };
    scan(None, &program.statements);
    for f in &program.functions {
        scan(Some(f.name.name.to_ascii_lowercase()), &f.body);
    }
    sites
}

fn collect_semantic_call_sites(
    module: &crate::semantic_ir::SemanticModule,
    function_names: &HashSet<String>,
) -> Vec<(Option<String>, String, Vec<CapacityCallArgument>)> {
    use crate::semantic_ir::{
        Expression, ExpressionKind, SemanticStatement, SemanticStatementKind as Kind,
    };

    fn visit_expression(
        expression: &Expression,
        scope: &Option<String>,
        function_names: &HashSet<String>,
        sites: &mut Vec<(Option<String>, String, Vec<CapacityCallArgument>)>,
    ) {
        match &expression.kind {
            ExpressionKind::Call { name, arguments } => {
                let callee = BasicIdent::parse(name).name.to_ascii_lowercase();
                if function_names.contains(&callee) {
                    sites.push((
                        scope.clone(),
                        callee,
                        arguments
                            .iter()
                            .cloned()
                            .map(CapacityCallArgument::Semantic)
                            .collect(),
                    ));
                }
                for argument in arguments {
                    visit_expression(argument, scope, function_names, sites);
                }
            }
            ExpressionKind::Index { index, .. } => {
                visit_expression(index, scope, function_names, sites)
            }
            ExpressionKind::MultiIndex { indices, .. } => {
                for index in indices {
                    visit_expression(index, scope, function_names, sites);
                }
            }
            ExpressionKind::Parenthesized(inner) | ExpressionKind::Unary { operand: inner, .. } => {
                visit_expression(inner, scope, function_names, sites)
            }
            ExpressionKind::Binary { left, right, .. } => {
                visit_expression(left, scope, function_names, sites);
                visit_expression(right, scope, function_names, sites);
            }
            ExpressionKind::Member {
                base, arguments, ..
            } => {
                if let Some(base) = base {
                    visit_expression(base, scope, function_names, sites);
                }
                if let Some(arguments) = arguments {
                    for argument in arguments {
                        visit_expression(argument, scope, function_names, sites);
                    }
                }
            }
            ExpressionKind::RecordLiteral(fields)
            | ExpressionKind::PartialRecordLiteral(fields) => {
                for field in fields {
                    visit_expression(&field.value, scope, function_names, sites);
                }
            }
            ExpressionKind::Name(_) | ExpressionKind::Literal(_) | ExpressionKind::Boolean(_) => {}
        }
    }

    fn visit_body(
        statements: &[SemanticStatement],
        scope: &Option<String>,
        function_names: &HashSet<String>,
        sites: &mut Vec<(Option<String>, String, Vec<CapacityCallArgument>)>,
    ) {
        fn visit_print_tokens(
            tokens: &[crate::semantic_ir::PrintToken],
            scope: &Option<String>,
            function_names: &HashSet<String>,
            sites: &mut Vec<(Option<String>, String, Vec<CapacityCallArgument>)>,
        ) {
            for token in tokens {
                if let crate::semantic_ir::PrintToken::Expression(expression) = token {
                    visit_expression(expression, scope, function_names, sites);
                }
            }
        }
        for statement in statements {
            match &statement.kind {
                Kind::Line(body) => visit_body(body, scope, function_names, sites),
                Kind::Assignment { target, value, .. } => {
                    visit_expression(target, scope, function_names, sites);
                    visit_expression(value, scope, function_names, sites);
                }
                Kind::MidAssign {
                    target,
                    start,
                    length,
                    value,
                } => {
                    visit_expression(target, scope, function_names, sites);
                    visit_expression(start, scope, function_names, sites);
                    if let Some(length) = length {
                        visit_expression(length, scope, function_names, sites);
                    }
                    visit_expression(value, scope, function_names, sites);
                }
                Kind::Expression(expression)
                | Kind::OptionBase(expression)
                | Kind::Error(expression)
                | Kind::Kill(expression)
                | Kind::Close(expression) => {
                    visit_expression(expression, scope, function_names, sites);
                }
                Kind::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    visit_expression(condition, scope, function_names, sites);
                    visit_body(then_body, scope, function_names, sites);
                    visit_body(else_body, scope, function_names, sites);
                }
                Kind::While { condition, body } => {
                    visit_expression(condition, scope, function_names, sites);
                    visit_body(body, scope, function_names, sites);
                }
                Kind::For {
                    start,
                    bounds,
                    body,
                    ..
                } => {
                    visit_expression(start, scope, function_names, sites);
                    match bounds {
                        crate::semantic_ir::ForBounds::To { limit, step } => {
                            visit_expression(limit, scope, function_names, sites);
                            if let Some(step) = step {
                                visit_expression(step, scope, function_names, sites);
                            }
                        }
                        crate::semantic_ir::ForBounds::Downto { limit, .. } => {
                            visit_expression(limit, scope, function_names, sites)
                        }
                    }
                    visit_body(body, scope, function_names, sites);
                }
                Kind::Do {
                    pre_condition,
                    post_condition,
                    body,
                } => {
                    for condition in pre_condition.iter().chain(post_condition.iter()) {
                        visit_expression(&condition.value, scope, function_names, sites);
                    }
                    visit_body(body, scope, function_names, sites);
                }
                Kind::SelectCase {
                    selector,
                    cases,
                    else_body,
                } => {
                    visit_expression(selector, scope, function_names, sites);
                    for case in cases {
                        for value in &case.values {
                            match value {
                                crate::semantic_ir::CaseValue::Comparison { value, .. } => {
                                    visit_expression(value, scope, function_names, sites)
                                }
                                crate::semantic_ir::CaseValue::Value {
                                    first, range_end, ..
                                } => {
                                    visit_expression(first, scope, function_names, sites);
                                    if let Some(last) = range_end {
                                        visit_expression(last, scope, function_names, sites);
                                    }
                                }
                            }
                        }
                        visit_body(&case.body, scope, function_names, sites);
                    }
                    visit_body(else_body, scope, function_names, sites);
                }
                Kind::Try {
                    body,
                    catch,
                    finally_body,
                } => {
                    visit_body(body, scope, function_names, sites);
                    if let Some(catch) = catch {
                        for filter in &catch.filters {
                            visit_expression(filter, scope, function_names, sites);
                        }
                        visit_body(&catch.body, scope, function_names, sites);
                    }
                    visit_body(finally_body, scope, function_names, sites);
                }
                Kind::Return(crate::semantic_ir::ReturnValue::Value(value))
                | Kind::Throw(crate::semantic_ir::ThrowValue::Value(value)) => {
                    visit_expression(value, scope, function_names, sites)
                }
                Kind::OnBranch { selector, .. } => {
                    visit_expression(selector, scope, function_names, sites)
                }
                Kind::Print {
                    destination,
                    tokens,
                } => {
                    match destination {
                        crate::semantic_ir::PrintDestination::Standard { .. } => {}
                        crate::semantic_ir::PrintDestination::Channel {
                            channel, using, ..
                        } => {
                            visit_expression(channel, scope, function_names, sites);
                            if let Some(using) = using {
                                visit_expression(using, scope, function_names, sites);
                            }
                        }
                        crate::semantic_ir::PrintDestination::Using { format, .. } => {
                            visit_expression(format, scope, function_names, sites)
                        }
                    }
                    visit_print_tokens(tokens, scope, function_names, sites);
                }
                Kind::Input { source, targets } => {
                    if let crate::semantic_ir::InputSource::Channel(channel) = source {
                        visit_expression(channel, scope, function_names, sites);
                    }
                    for target in targets {
                        visit_expression(target, scope, function_names, sites);
                    }
                }
                Kind::Write { channel, values } => {
                    visit_expression(channel, scope, function_names, sites);
                    if let crate::semantic_ir::WriteValues::Values(values) = values {
                        for value in values {
                            visit_expression(value, scope, function_names, sites);
                        }
                    }
                }
                Kind::Open {
                    path,
                    channel,
                    length,
                    ..
                } => {
                    visit_expression(path, scope, function_names, sites);
                    visit_expression(channel, scope, function_names, sites);
                    if let Some(length) = length {
                        visit_expression(length, scope, function_names, sites);
                    }
                }
                Kind::Seek { channel, position } => {
                    visit_expression(channel, scope, function_names, sites);
                    visit_expression(position, scope, function_names, sites);
                }
                Kind::Rename {
                    source,
                    destination,
                } => {
                    visit_expression(source, scope, function_names, sites);
                    visit_expression(destination, scope, function_names, sites);
                }
                Kind::Data(values) | Kind::Read(values) => {
                    for value in values {
                        visit_expression(value, scope, function_names, sites);
                    }
                }
                Kind::Dim(items) => {
                    for item in items {
                        for dimension in &item.dimensions {
                            if let crate::semantic_ir::DimAxis::Expression(expression) = dimension {
                                visit_expression(expression, scope, function_names, sites);
                            }
                        }
                    }
                }
                Kind::Const { value, .. } => visit_expression(value, scope, function_names, sites),
                Kind::Swap { left, right } => {
                    visit_expression(left, scope, function_names, sites);
                    visit_expression(right, scope, function_names, sites);
                }
                Kind::Randomize(crate::semantic_ir::RandomizeSeed::Value(value)) => {
                    visit_expression(value, scope, function_names, sites)
                }
                Kind::Poke { address, value } => {
                    visit_expression(address, scope, function_names, sites);
                    visit_expression(value, scope, function_names, sites);
                }
                Kind::Out { port, value } => {
                    visit_expression(port, scope, function_names, sites);
                    visit_expression(value, scope, function_names, sites);
                }
                Kind::Width { channel, value } => {
                    if let Some(channel) = channel {
                        visit_expression(channel, scope, function_names, sites);
                    }
                    visit_expression(value, scope, function_names, sites);
                }
                Kind::LineInput { channel, target } => {
                    visit_expression(channel, scope, function_names, sites);
                    visit_expression(target, scope, function_names, sites);
                }
                Kind::Get { channel, position, .. } | Kind::Put { channel, position, .. } => {
                    visit_expression(channel, scope, function_names, sites);
                    if let Some(position) = position {
                        if let Some(value) = &position.position {
                            visit_expression(value, scope, function_names, sites);
                        }
                        if let Some(value) = &position.record {
                            visit_expression(value, scope, function_names, sites);
                        }
                    }
                }
                Kind::Lset { value, .. } | Kind::Rset { value, .. } => {
                    visit_expression(value, scope, function_names, sites)
                }
                Kind::Locate { row, column } => {
                    visit_expression(row, scope, function_names, sites);
                    visit_expression(column, scope, function_names, sites);
                }
                Kind::Color {
                    foreground,
                    background,
                } => {
                    visit_expression(foreground, scope, function_names, sites);
                    if let Some(background) = background {
                        visit_expression(background, scope, function_names, sites);
                    }
                }
                Kind::Lprint { using, tokens } => {
                    if let Some(using) = using {
                        visit_expression(using, scope, function_names, sites);
                    }
                    visit_print_tokens(tokens, scope, function_names, sites);
                }
                Kind::FileDeclaration { path, .. } => {
                    visit_expression(path, scope, function_names, sites)
                }
                Kind::Field { channel, bindings, .. } => {
                    visit_expression(channel, scope, function_names, sites);
                    for binding in bindings {
                        visit_expression(&binding.length, scope, function_names, sites);
                    }
                }
                Kind::Unsupported
                | Kind::Label(_)
                | Kind::Comment { .. }
                | Kind::Exit
                | Kind::Continue
                | Kind::End
                | Kind::Goto(_)
                | Kind::Gosub(_)
                | Kind::Resume(_)
                | Kind::OnErrorGoto(_)
                | Kind::Erase(_)
                | Kind::Restore(_)
                | Kind::Global { .. }
                | Kind::Randomize(crate::semantic_ir::RandomizeSeed::Default)
                | Kind::Return(crate::semantic_ir::ReturnValue::Default)
                | Kind::Throw(crate::semantic_ir::ThrowValue::Bare)
                | Kind::Stop
                | Kind::Clear
                | Kind::Cls
                | Kind::Beep
                | Kind::System => {}
            }
        }
    }

    let mut sites = Vec::new();
    let top_level = None;
    visit_body(&module.statements, &top_level, function_names, &mut sites);
    for callable in &module.callables {
        let scope = Some(BasicIdent::parse(&callable.name).name.to_ascii_lowercase());
        visit_body(&callable.body, &scope, function_names, &mut sites);
    }
    sites
}

/// Resolves one call site's argument to a concrete per-axis bound, if
/// possible -- see `ArgBound`.
fn resolve_call_arg_bound(
    scope: &Option<String>,
    arg: &Expr,
    axis: usize,
    resolved: &HashMap<String, Vec<Vec<Option<i64>>>>,
    local_dim_sizes: &HashMap<String, HashMap<String, Vec<Expr>>>,
    semantic_local_dim_sizes: Option<&HashMap<String, HashMap<String, Vec<Option<i64>>>>>,
    top_level_dim_sizes: &HashMap<String, Vec<Expr>>,
    semantic_top_level_dim_sizes: Option<&HashMap<String, Vec<Option<i64>>>>,
    functions_by_name: &HashMap<String, &FunctionDef>,
    consts: &HashMap<String, Vec<Expr>>,
) -> ArgBound {
    let source_name: &BasicIdent = match arg {
        Expr::ArrayRef { name, indices } if indices.is_empty() => name,
        Expr::Ident(name) => name,
        _ => return ArgBound::NotAnArray,
    };
    let key = source_name.as_basic().to_ascii_lowercase();

    if let Some(func) = scope {
        if let Some(def) = functions_by_name.get(func) {
            if let Some(idx) = def
                .params
                .iter()
                .position(|p| p.name.as_basic().to_ascii_lowercase() == key)
            {
                return if resolved
                    .get(func)
                    .and_then(|parameters| parameters.get(idx))
                    .is_some_and(|axes| !axes.is_empty())
                {
                    match resolved
                        .get(func)
                        .and_then(|v| v.get(idx))
                        .and_then(|a| a.get(axis))
                    {
                        Some(Some(v)) => ArgBound::Resolved(*v),
                        _ => ArgBound::Unresolvable,
                    }
                } else {
                    ArgBound::NotAnArray
                };
            }
        }
        if let Some(semantic_local) = semantic_local_dim_sizes.and_then(|scopes| scopes.get(func)) {
            if let Some(sizes) = semantic_local.get(&key) {
                return match sizes.get(axis).copied().flatten() {
                    Some(value) => ArgBound::Resolved(value),
                    None => ArgBound::Unresolvable,
                };
            }
            if let Some(semantic_top_level) = semantic_top_level_dim_sizes {
                return semantic_top_level
                    .get(&key)
                    .map(|sizes| match sizes.get(axis).copied().flatten() {
                        Some(value) => ArgBound::Resolved(value),
                        None => ArgBound::Unresolvable,
                    })
                    .unwrap_or(ArgBound::NotAnArray);
            }
        }
        if let Some(sizes) = local_dim_sizes.get(func).and_then(|m| m.get(&key)) {
            return match sizes.get(axis).and_then(|e| const_eval(e, consts, 0)) {
                Some(v) => ArgBound::Resolved(v),
                None => ArgBound::Unresolvable,
            };
        }
    }
    if let Some(semantic_top_level) = semantic_top_level_dim_sizes {
        return semantic_top_level
            .get(&key)
            .map(|sizes| match sizes.get(axis).copied().flatten() {
                Some(value) => ArgBound::Resolved(value),
                None => ArgBound::Unresolvable,
            })
            .unwrap_or(ArgBound::NotAnArray);
    }
    if let Some(sizes) = top_level_dim_sizes.get(&key) {
        return match sizes.get(axis).and_then(|e| const_eval(e, consts, 0)) {
            Some(v) => ArgBound::Resolved(v),
            None => ArgBound::Unresolvable,
        };
    }
    ArgBound::NotAnArray
}

fn resolve_semantic_call_arg_bound(
    scope: &Option<String>,
    arg: &crate::semantic_ir::Expression,
    module: &crate::semantic_ir::SemanticModule,
    resolved: &HashMap<String, Vec<Vec<Option<i64>>>>,
    semantic_local_dim_sizes: &HashMap<String, HashMap<String, Vec<Option<i64>>>>,
    semantic_top_level_dim_sizes: &HashMap<String, Vec<Option<i64>>>,
    axis: usize,
) -> ArgBound {
    use crate::semantic_ir::ExpressionKind;
    let mut expression = arg;
    while let ExpressionKind::Parenthesized(inner) = &expression.kind {
        expression = inner;
    }
    let name = match &expression.kind {
        ExpressionKind::Name(name) => name,
        // The grammar preserves `array%()` as a zero-argument Call until
        // declaration facts disambiguate it; BASCAL permits that spelling
        // for an array passed as a whole call argument.
        ExpressionKind::Call { name, arguments } if arguments.is_empty() => name,
        _ => return ArgBound::NotAnArray,
    };
    let key = BasicIdent::parse(name).as_basic().to_ascii_lowercase();

    if let Some(scope) = scope {
        let callable_name = BasicIdent::parse(scope).name.to_ascii_lowercase();
        if let Some(callable) = module.callables.iter().find(|callable| {
            callable.receiver.is_none()
                && BasicIdent::parse(&callable.name)
                    .name
                    .eq_ignore_ascii_case(&callable_name)
                && matches!(
                    callable.kind,
                    crate::semantic_ir::CallableKind::Function
                        | crate::semantic_ir::CallableKind::Procedure
                )
        }) {
            if let Some(index) = callable.parameters.iter().position(|parameter| {
                BasicIdent::parse(&parameter.name)
                    .as_basic()
                    .eq_ignore_ascii_case(&key)
            }) {
                return if resolved
                    .get(&callable_name)
                    .and_then(|parameters| parameters.get(index))
                    .is_some_and(|axes| !axes.is_empty())
                {
                    match resolved
                        .get(&callable_name)
                        .and_then(|parameters| parameters.get(index))
                        .and_then(|axes| axes.get(axis))
                    {
                        Some(Some(value)) => ArgBound::Resolved(*value),
                        _ => ArgBound::Unresolvable,
                    }
                } else {
                    ArgBound::NotAnArray
                };
            }
        }
        if let Some(sizes) = semantic_local_dim_sizes
            .get(&callable_name)
            .and_then(|declarations| declarations.get(&key))
        {
            return match sizes.get(axis).copied().flatten() {
                Some(value) => ArgBound::Resolved(value),
                None => ArgBound::Unresolvable,
            };
        }
    }

    semantic_top_level_dim_sizes
        .get(&key)
        .map(|sizes| match sizes.get(axis).copied().flatten() {
            Some(value) => ArgBound::Resolved(value),
            None => ArgBound::Unresolvable,
        })
        .unwrap_or(ArgBound::NotAnArray)
}

/// A rendered branch condition: a single BASIC expression, or a `&&`/`||`
/// chain transpiled to one short-circuit guard per operand, so a later operand's
/// side effects genuinely do not run once an earlier operand has decided the
/// outcome (the semantic counterpart of `CodeGenerator::condition_jump`).
enum SemanticCondition {
    Expression {
        prelude: Vec<String>,
        text: String,
    },
    Chain {
        is_and: bool,
        continue_id: usize,
        operands: Vec<(Vec<String>, String)>,
    },
}

impl SemanticCondition {
    /// Lines that jump to `target` when the condition is false (or true, if
    /// `invert`) and fall through otherwise.
    fn jump_lines(&self, target: &str, invert: bool) -> Vec<String> {
        match self {
            SemanticCondition::Expression { prelude, text } => {
                let polarity = if invert { "<> 0" } else { "= 0" };
                let mut lines = prelude.clone();
                lines.push(format!("IF ({text}) {polarity} THEN GOTO {target}"));
                lines
            }
            SemanticCondition::Chain {
                is_and,
                continue_id,
                operands,
            } => {
                let polarity = if *is_and { "= 0" } else { "<> 0" };
                // De Morgan duality: an AND chain under `invert` behaves like
                // an OR chain and needs a skip label.
                let simple = *is_and != invert;
                let continue_label = format!("SC_{continue_id:04}_CONT");
                let destination = if simple { target } else { &continue_label };
                let mut lines = Vec::new();
                for (prelude, text) in operands {
                    lines.extend(prelude.iter().cloned());
                    lines.push(format!("IF ({text}) {polarity} THEN GOTO {destination}"));
                }
                if !simple {
                    lines.push(format!("GOTO {target}"));
                    lines.push(format!("{continue_label}:"));
                }
                lines
            }
        }
    }
}

fn flatten_semantic_chain<'a>(
    expression: &'a crate::semantic_ir::Expression,
    operator: &str,
    out: &mut Vec<&'a crate::semantic_ir::Expression>,
) {
    if let crate::semantic_ir::ExpressionKind::Binary {
        left,
        operator: this,
        right,
    } = &expression.kind
    {
        if this == operator {
            flatten_semantic_chain(left, operator, out);
            flatten_semantic_chain(right, operator, out);
            return;
        }
    }
    out.push(expression);
}

/// The source position of `span` inside a typed callable, falling back to the
/// synthetic validation position when the module cannot place it.
fn semantic_callable_diagnostic_pos(
    module: Option<&crate::semantic_ir::SemanticModule>,
    callable: Option<&crate::semantic_ir::CallableSignature>,
    span: crate::rdgen_frontend::SourceSpan,
) -> SourcePos {
    module
        .zip(callable)
        .and_then(|(module, callable)| module.callable_position(callable, span))
        .unwrap_or_else(|| SourcePos::new("<validation>", 1, 1))
}

/// The scalar type a value of this semantic type has as a method receiver.
fn semantic_receiver_suffix(value_type: crate::semantic_ir::SemanticValueType) -> Option<TypeSuffix> {
    use crate::semantic_ir::SemanticValueType as Value;
    match value_type {
        Value::String => Some(TypeSuffix::String),
        Value::Integer => Some(TypeSuffix::Integer),
        Value::Long => Some(TypeSuffix::Long),
        Value::Single => Some(TypeSuffix::Single),
        Value::Double => Some(TypeSuffix::Double),
        Value::Unknown | Value::Boolean => None,
    }
}

fn semantic_basic_callable_for_function<'a>(
    module: &'a crate::semantic_ir::SemanticModule,
    function: &FunctionDef,
) -> Option<&'a crate::semantic_ir::CallableSignature> {
    let candidates = module
        .callables
        .iter()
        .filter(|callable| {
            crate::semantic_ir::callable_name_matches_function(&callable.name, function)
                && callable.receiver.is_some() == function.receiver.is_some()
                && matches!(
                    (function.receiver.is_some(), callable.kind),
                    (
                        true,
                        crate::semantic_ir::CallableKind::Method
                            | crate::semantic_ir::CallableKind::FluentMethod
                            | crate::semantic_ir::CallableKind::InlineMethod
                    ) | (
                        false,
                        crate::semantic_ir::CallableKind::Procedure
                            | crate::semantic_ir::CallableKind::Function
                    )
                )
        })
        .collect::<Vec<_>>();
    candidates
        .iter()
        .copied()
        .find(
            |callable| match (function.receiver, callable.receiver.as_deref()) {
                (None, None) => true,
                (Some(TypeSuffix::Integer), Some(value)) => value.eq_ignore_ascii_case("integer"),
                (Some(TypeSuffix::Long), Some(value)) => value.eq_ignore_ascii_case("long"),
                (Some(TypeSuffix::Single), Some(value)) => value.eq_ignore_ascii_case("single"),
                (Some(TypeSuffix::Double), Some(value)) => value.eq_ignore_ascii_case("double"),
                (Some(TypeSuffix::String), Some(value)) => value.eq_ignore_ascii_case("string"),
                _ => false,
            },
        )
        .or_else(|| (candidates.len() == 1).then(|| candidates[0]))
}

fn semantic_dimension_capacity(
    module: &crate::semantic_ir::SemanticModule,
    dimension: &crate::semantic_ir::DimAxis,
) -> Option<i64> {
    match dimension {
        crate::semantic_ir::DimAxis::Inferred => None,
        crate::semantic_ir::DimAxis::Fixed(value) => crate::semantic_ir::parse_integer_value(value),
        crate::semantic_ir::DimAxis::Expression(expression) => {
            module.evaluate_integer_expression(expression)
        }
    }
}

/// Resolves every array parameter's per-axis storage capacity across the
/// whole program: lowercase function name -> per-parameter (empty for a
/// scalar) -> per-axis capacity. An axis whose capacity couldn't be
/// resolved (an unresolvable `?`, reported as a diagnostic) comes back as
/// `0` -- codegen still runs to completion so every diagnostic in the
/// program gets collected, but the result is discarded once any
/// diagnostic exists.
fn infer_array_param_capacities(
    program: &Program,
    semantic_module: Option<&crate::semantic_ir::SemanticModule>,
    diagnostics: &mut Vec<Diagnostic>,
) -> HashMap<String, Vec<Vec<i64>>> {
    let function_names: HashSet<String> = if let Some(module) = semantic_module {
        let mut names: HashSet<_> = module
            .callables
            .iter()
            .map(|callable| BasicIdent::parse(&callable.name).name.to_ascii_lowercase())
            .collect();
        names.extend(
            program
                .functions
                .iter()
                .filter(|function| semantic_basic_callable_for_function(module, function).is_none())
                .map(|function| function.name.name.to_ascii_lowercase()),
        );
        names
    } else {
        program
            .functions
            .iter()
            .map(|function| function.name.name.to_ascii_lowercase())
            .collect()
    };
    let functions_by_name: HashMap<String, &FunctionDef> = program
        .functions
        .iter()
        .map(|f| (f.name.name.to_ascii_lowercase(), f))
        .collect();

    let mut consts = HashMap::new();
    if let Some(module) = semantic_module {
        for (name, value) in module.integer_constants() {
            consts.insert(name.clone(), vec![Expr::Integer(value)]);
            if !name.ends_with(['$', '%', '&', '!', '#']) {
                consts.insert(format!("{name}%"), vec![Expr::Integer(value)]);
            }
        }
        for function in &program.functions {
            if basic_semantic_callable_statements_by_source(module, function).is_none() {
                collect_consts(&function.body, &mut consts);
            }
        }
    } else {
        collect_consts(&program.statements, &mut consts);
        for f in &program.functions {
            collect_consts(&f.body, &mut consts);
        }
    }

    let mut top_level_dim_sizes = HashMap::new();
    if semantic_module.is_none() {
        collect_dim_sizes(&program.statements, &mut top_level_dim_sizes);
    }
    let semantic_top_level_dim_sizes = semantic_module.map(|module| {
        module
            .top_level_dim_declarations()
            .into_iter()
            .filter(|(_, declaration)| declaration.array_axes > 0)
            .map(|(name, declaration)| {
                let dimensions = declaration
                    .dimensions
                    .iter()
                    .map(|dimension| semantic_dimension_capacity(module, dimension))
                    .collect();
                (name, dimensions)
            })
            .collect::<HashMap<_, Vec<Option<i64>>>>()
    });

    let mut local_dim_sizes: HashMap<String, HashMap<String, Vec<Expr>>> = HashMap::new();
    for f in &program.functions {
        if semantic_module
            .is_none_or(|module| basic_semantic_callable_statements_by_source(module, f).is_none())
        {
            let mut sizes = HashMap::new();
            collect_dim_sizes(&f.body, &mut sizes);
            local_dim_sizes.insert(f.name.name.to_ascii_lowercase(), sizes);
        }
    }
    let semantic_local_dim_sizes = semantic_module.map(|module| {
        module
            .callables
            .iter()
            .filter(|callable| {
                callable.receiver.is_none()
                    && matches!(
                        callable.kind,
                        crate::semantic_ir::CallableKind::Function
                            | crate::semantic_ir::CallableKind::Procedure
                    )
            })
            .map(|callable| {
                let mut scoped_module = module.clone();
                scoped_module.statements = callable.body.clone();
                let dimensions = callable
                    .dim_declarations()
                    .into_iter()
                    .filter(|(_, declaration)| declaration.array_axes > 0)
                    .map(|(name, declaration)| {
                        let capacities = declaration
                            .dimensions
                            .iter()
                            .map(|dimension| semantic_dimension_capacity(&scoped_module, dimension))
                            .collect();
                        (name, capacities)
                    })
                    .collect();
                (
                    BasicIdent::parse(&callable.name).name.to_ascii_lowercase(),
                    dimensions,
                )
            })
            .collect::<HashMap<_, HashMap<String, Vec<Option<i64>>>>>()
    });

    let call_sites = if let Some(module) = semantic_module {
        let mut sites = collect_semantic_call_sites(module, &function_names);
        let legacy_sites = collect_call_sites(program, &function_names);
        for (scope, callee, arguments) in legacy_sites {
            let Some(scope_name) = scope.as_deref() else {
                continue;
            };
            let Some(function) = program
                .functions
                .iter()
                .find(|function| function.name.name.eq_ignore_ascii_case(scope_name))
            else {
                continue;
            };
            // Preserve AST call-site analysis only for callable bodies that
            // cannot be aligned to semantic IR and therefore remain on the
            // backend's explicit compatibility path.
            if basic_semantic_callable_statements_by_source(module, function).is_none() {
                sites.push((
                    scope,
                    callee.name.to_ascii_lowercase(),
                    arguments
                        .into_iter()
                        .map(CapacityCallArgument::Legacy)
                        .collect(),
                ));
            }
        }
        sites
    } else {
        collect_call_sites(program, &function_names)
            .into_iter()
            .map(|(scope, callee, arguments)| {
                (
                    scope,
                    callee.name.to_ascii_lowercase(),
                    arguments
                        .into_iter()
                        .map(CapacityCallArgument::Legacy)
                        .collect(),
                )
            })
            .collect()
    };

    let mut resolved: HashMap<String, Vec<Vec<Option<i64>>>> = program
        .functions
        .iter()
        .map(|f| {
            let per_param = semantic_module
                .and_then(|module| semantic_basic_callable_for_function(module, f))
                .map(|callable| {
                    callable
                        .parameters
                        .iter()
                        .map(|parameter| {
                            parameter
                                .dimensions
                                .iter()
                                .map(|dimension| {
                                    semantic_dimension_capacity(
                                        semantic_module.expect("semantic callable module"),
                                        dimension,
                                    )
                                })
                                .collect()
                        })
                        .collect()
                })
                .unwrap_or_else(|| {
                    f.params
                        .iter()
                        .map(|parameter| parameter.axes.clone().unwrap_or_default())
                        .collect()
                });
            (f.name.name.to_ascii_lowercase(), per_param)
        })
        .collect();

    // Fixed-point: each round resolves whatever `?` axes it can from
    // already-known bounds (literals, consts, or another parameter's
    // already-resolved capacity). Since BASCAL rejects every call cycle,
    // direct or indirect, the dependency chain through forwarded array
    // parameters is finite, so this always terminates.
    loop {
        let mut changed = false;
        for f in &program.functions {
            let fname = f.name.name.to_ascii_lowercase();
            let parameter_count = semantic_module
                .and_then(|module| semantic_basic_callable_for_function(module, f))
                .map(|callable| callable.parameters.len())
                .unwrap_or(f.params.len());
            for param_index in 0..parameter_count {
                let axis_count = resolved
                    .get(&fname)
                    .and_then(|parameters| parameters.get(param_index))
                    .map(Vec::len)
                    .unwrap_or_default();
                if axis_count == 0 {
                    continue;
                }
                for axis in 0..axis_count {
                    if resolved[&fname][param_index][axis].is_some() {
                        continue;
                    }
                    let mut max_value: Option<i64> = None;
                    let mut all_resolved = true;
                    let mut any_call_site = false;
                    for (scope, callee, call_args) in &call_sites {
                        if callee != &fname {
                            continue;
                        }
                        let Some(arg) = call_args.get(param_index) else {
                            continue;
                        };
                        let bound = match arg {
                            CapacityCallArgument::Legacy(argument) => resolve_call_arg_bound(
                                scope,
                                argument,
                                axis,
                                &resolved,
                                &local_dim_sizes,
                                semantic_local_dim_sizes.as_ref(),
                                &top_level_dim_sizes,
                                semantic_top_level_dim_sizes.as_ref(),
                                &functions_by_name,
                                &consts,
                            ),
                            CapacityCallArgument::Semantic(argument) => {
                                let module = semantic_module
                                    .expect("semantic callsites require a semantic module");
                                resolve_semantic_call_arg_bound(
                                    scope,
                                    argument,
                                    module,
                                    &resolved,
                                    semantic_local_dim_sizes
                                        .as_ref()
                                        .expect("semantic callable DIM facts are available"),
                                    semantic_top_level_dim_sizes
                                        .as_ref()
                                        .expect("semantic top-level DIM facts are available"),
                                    axis,
                                )
                            }
                        };
                        match bound {
                            ArgBound::NotAnArray => {}
                            ArgBound::Unresolvable => {
                                any_call_site = true;
                                all_resolved = false;
                            }
                            ArgBound::Resolved(v) => {
                                any_call_site = true;
                                max_value = Some(max_value.map_or(v, |m: i64| m.max(v)));
                            }
                        }
                    }
                    if any_call_site && all_resolved {
                        if let Some(v) = max_value {
                            resolved.get_mut(&fname).unwrap()[param_index][axis] = Some(v);
                            changed = true;
                        }
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }

    // Compile-time-provable overflow check: for every axis (inferred or
    // explicit), any call site whose bound *does* resolve gets compared
    // against the final capacity right now, instead of waiting to catch
    // it with the runtime check `call_lines` emits for the general case.
    for f in &program.functions {
        let fname = f.name.name.to_ascii_lowercase();
        let semantic_parameters = semantic_module
            .and_then(|module| semantic_basic_callable_for_function(module, f))
            .map(|callable| callable.parameters.as_slice());
        let parameter_count = semantic_parameters
            .map(|parameters| parameters.len())
            .unwrap_or(f.params.len());
        for param_index in 0..parameter_count {
            let param_name = semantic_parameters
                .and_then(|parameters| parameters.get(param_index))
                .map(|parameter| parameter.name.clone())
                .or_else(|| {
                    f.params
                        .get(param_index)
                        .map(|parameter| parameter.name.as_basic())
                })
                .unwrap_or_default();
            let axis_count = resolved
                .get(&fname)
                .and_then(|parameters| parameters.get(param_index))
                .map(Vec::len)
                .unwrap_or_default();
            if axis_count == 0 {
                continue;
            }
            for axis in 0..axis_count {
                let Some(capacity) = resolved[&fname][param_index][axis] else {
                    continue;
                };
                for (scope, callee, call_args) in &call_sites {
                    if callee != &fname {
                        continue;
                    }
                    let Some(arg) = call_args.get(param_index) else {
                        continue;
                    };
                    let bound = match arg {
                        CapacityCallArgument::Legacy(argument) => resolve_call_arg_bound(
                            scope,
                            argument,
                            axis,
                            &resolved,
                            &local_dim_sizes,
                            semantic_local_dim_sizes.as_ref(),
                            &top_level_dim_sizes,
                            semantic_top_level_dim_sizes.as_ref(),
                            &functions_by_name,
                            &consts,
                        ),
                        CapacityCallArgument::Semantic(argument) => {
                            let module = semantic_module
                                .expect("semantic callsites require a semantic module");
                            resolve_semantic_call_arg_bound(
                                scope,
                                argument,
                                module,
                                &resolved,
                                semantic_local_dim_sizes
                                    .as_ref()
                                    .expect("semantic callable DIM facts are available"),
                                semantic_top_level_dim_sizes
                                    .as_ref()
                                    .expect("semantic top-level DIM facts are available"),
                                axis,
                            )
                        }
                    };
                    if let ArgBound::Resolved(actual) = bound {
                        if actual > capacity {
                            diagnostics.push(Diagnostic::error(
                                SourcePos::new("<validation>", 1, 1),
                                format!(
                                    "a call to `{}` passes {} elements along axis {} of `{}`, \
                                     but its storage is only sized for {} -- give `{}` a \
                                     bigger explicit capacity",
                                    f.name, actual, axis, param_name, capacity, param_name,
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    // Any `?` axis still unresolved at this point genuinely can't be
    // inferred -- either no call site could be resolved, or the parameter
    // is never called at all.
    for f in &program.functions {
        let semantic_callable =
            semantic_module.and_then(|module| semantic_basic_callable_for_function(module, f));
        let fname = semantic_callable
            .map(|callable| BasicIdent::parse(&callable.name).name.to_ascii_lowercase())
            .unwrap_or_else(|| f.name.name.to_ascii_lowercase());
        let callable_name = semantic_callable
            .map(|callable| callable.name.clone())
            .unwrap_or_else(|| f.name.as_basic());
        let parameter_names: Vec<String> = semantic_callable
            .map(|callable| {
                callable
                    .parameters
                    .iter()
                    .map(|parameter| parameter.name.clone())
                    .collect()
            })
            .unwrap_or_else(|| {
                f.params
                    .iter()
                    .map(|parameter| parameter.name.as_basic())
                    .collect()
            });
        for (param_index, param_name) in parameter_names.iter().enumerate() {
            let axis_count = resolved
                .get(&fname)
                .and_then(|parameters| parameters.get(param_index))
                .map(Vec::len)
                .unwrap_or_default();
            if axis_count == 0 {
                continue;
            }
            for axis in 0..axis_count {
                if resolved[&fname][param_index][axis].is_some() {
                    continue;
                }
                let any_call_site = call_sites.iter().any(|(_, callee, call_args)| {
                    callee == &fname && call_args.get(param_index).is_some()
                });
                let message = if any_call_site {
                    format!(
                        "can't automatically size `{}`'s storage along axis {} of `{}` -- at \
                         least one call site passes an array whose size isn't a compile-time \
                         constant. Give it an explicit capacity instead of `?`, e.g. `{}(100)`",
                        param_name, axis, callable_name, param_name,
                    )
                } else {
                    format!(
                        "can't automatically size `{}`'s storage along axis {} of `{}` -- `{}` \
                         is never called, so there's no call site to infer a capacity from. \
                         Give it an explicit capacity instead of `?`, e.g. `{}(100)`",
                        param_name, axis, callable_name, callable_name, param_name,
                    )
                };
                diagnostics.push(Diagnostic::error(
                    SourcePos::new("<validation>", 1, 1),
                    message,
                ));
            }
        }
    }

    resolved
        .into_iter()
        .map(|(name, per_param)| {
            let capacities = per_param
                .into_iter()
                .map(|axes| axes.into_iter().map(|axis| axis.unwrap_or(0)).collect())
                .collect();
            (name, capacities)
        })
        .collect()
}

fn collect_globals(body: &[Stmt]) -> HashSet<String> {
    let mut globals = HashSet::new();
    for stmt in body {
        match &stmt.kind {
            Statement::GlobalDecl(ident) => {
                globals.insert(ident.as_basic().to_ascii_lowercase());
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                globals.extend(collect_globals(then_body));
                globals.extend(collect_globals(else_body));
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => {
                globals.extend(collect_globals(body));
            }
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    globals.extend(collect_globals(&case.body));
                }
                globals.extend(collect_globals(else_body));
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                globals.extend(collect_globals(try_body));
                if let Some(catch) = catch {
                    globals.extend(collect_globals(&catch.body));
                }
                globals.extend(collect_globals(finally_body));
            }
            _ => {}
        }
    }
    globals
}

/// Collect the lowercase BASIC name of every record/file FIELD buffer
/// variable in the whole program (top-level statements and every function
/// body). `records::lower` always names a given file/field pair's buffer
/// identically everywhere it's referenced, so this set is exactly the set
/// of names `ident()` must resolve to their bare global form, no matter
/// which function/procedure body an LSET/GET/PUT referencing one appears in.
pub(crate) fn collect_record_buffer_names(program: &Program) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_record_buffer_names_in(&program.statements, &mut names);
    for func in &program.functions {
        collect_record_buffer_names_in(&func.body, &mut names);
    }
    names
}

fn semantic_record_buffer_names(module: &crate::semantic_ir::SemanticModule) -> HashSet<String> {
    let mut names = module.record_buffer_names();
    for file in &module.lowered_record_files {
        names.extend(
            file.fields
                .iter()
                .map(|field| field.buffer_name.to_ascii_lowercase()),
        );
    }
    names
}

fn collect_record_buffer_names_in(stmts: &[Stmt], names: &mut HashSet<String>) {
    for stmt in stmts {
        match &stmt.kind {
            Statement::Field { fields, .. } => {
                for (_, var) in fields {
                    names.insert(var.as_basic().to_ascii_lowercase());
                }
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_record_buffer_names_in(then_body, names);
                collect_record_buffer_names_in(else_body, names);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => {
                collect_record_buffer_names_in(body, names);
            }
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_record_buffer_names_in(&case.body, names);
                }
                collect_record_buffer_names_in(else_body, names);
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                collect_record_buffer_names_in(try_body, names);
                if let Some(catch) = catch {
                    collect_record_buffer_names_in(&catch.body, names);
                }
                collect_record_buffer_names_in(finally_body, names);
            }
            _ => {}
        }
    }
}

/// Name (sans type suffix) of the optional require-able `com.bascal.stdlib.
/// midAssign` compatibility function. The BASIC backend handles statement-
/// form MID$ assignment inline; the JVM backend recognizes explicitly
/// required legacy helper definitions.
pub(crate) const MID_ASSIGN_HELPER_NAME: &str = "midAssign";

/// Returns a `BasicIdent` whose BASIC form is not present in `taken`.
/// Always uses the indexed form `preferredStem0`, `1`, … so that allocated
/// names are visually distinct from bare global names and can never coincide
/// with an unindexed global even if no collision exists today.
/// A short, alphanumeric-only tag distinguishing one type suffix from
/// another in a generated GOSUB label (see `function_label`) -- `allocate_unique`
/// already keeps ordinary variable names apart this way via the real BASIC
/// suffix character embedded in `BasicIdent::as_basic()`'s own rendering,
/// but a label is plain emitted text (`self.line(&format!("{}:", info.label))`
/// in `emit_function_def`), not a `BasicIdent`, so it needs its own safe
/// (non-`%`/`$`/`!`/`#`/`&`) tag instead.
fn label_suffix_tag(suffix: TypeSuffix) -> &'static str {
    match suffix {
        TypeSuffix::Integer => "i",
        TypeSuffix::Long => "l",
        TypeSuffix::Single => "f",
        TypeSuffix::Double => "d",
        TypeSuffix::String => "s",
    }
}

/// The GOSUB label a function/procedure/method's own body starts at --
/// keyed by `stem` (the base name alone) *plus* the result suffix and (for
/// a method) the receiver, so two functions/methods that only differ by
/// suffix and/or receiver never collide on the same label. Two ordinary
/// functions differing only by suffix (`function foo%(x%)` / `function
/// foo$(x%)`) used to collide this way -- both got `FN_foo`, and whichever
/// claimed it last silently won every call site, with no diagnostic at
/// all (confirmed via real `fbc` execution: `print foo%(5)` actually ran
/// `foo$`'s body). A procedure has no result suffix at all
/// (`function.name.suffix` is `None`), so its label is keyed on the bare
/// stem alone -- safe, since `reject_duplicate_functions` already rejects
/// two procedures (or a procedure and a function, which real BASIC's own
/// name resolution can't tell apart either) sharing one name.
fn function_label(stem: &str, suffix: Option<TypeSuffix>, receiver: Option<TypeSuffix>) -> String {
    let mut label = format!("FN_{stem}");
    if let Some(suffix) = suffix {
        label.push('_');
        label.push_str(label_suffix_tag(suffix));
    }
    if let Some(receiver) = receiver {
        label.push_str("_of_");
        label.push_str(label_suffix_tag(receiver));
    }
    label
}

fn allocate_unique(
    preferred_stem: &str,
    suffix: Option<TypeSuffix>,
    taken: &HashSet<String>,
) -> BasicIdent {
    for i in 0u32.. {
        let candidate = BasicIdent {
            name: format!("{preferred_stem}{i}"),
            suffix,
        };
        if !taken.contains(&candidate.as_basic().to_ascii_lowercase()) {
            return candidate;
        }
    }
    unreachable!("allocate_unique exhausted u32 candidates")
}

/// Collect the lowercase BASIC form of every variable name used at global
/// (program-level) scope, plus any names declared as `global` inside
/// functions.  This forms the initial "taken" set before function params and
/// results are allocated.
fn collect_program_names(program: &Program) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_names_from_stmts(&program.statements, &mut names);
    for block in &program.common {
        for var in &block.vars {
            names.insert(var.name.as_basic().to_ascii_lowercase());
        }
    }
    for func in &program.functions {
        collect_global_decl_names(&func.body, &mut names);
    }
    names
}

fn collect_names_from_stmts(stmts: &[Stmt], names: &mut HashSet<String>) {
    for stmt in stmts {
        collect_names_from_stmt(stmt, names);
    }
}

fn collect_names_from_stmt(stmt: &Stmt, names: &mut HashSet<String>) {
    match &stmt.kind {
        Statement::Assignment { target, value } => {
            collect_names_from_expr(target, names);
            collect_names_from_expr(value, names);
        }
        Statement::MidAssign {
            target,
            start,
            len,
            value,
        } => {
            collect_names_from_expr(target, names);
            collect_names_from_expr(start, names);
            if let Some(e) = len {
                collect_names_from_expr(e, names);
            }
            collect_names_from_expr(value, names);
        }
        Statement::Dim { name, .. } => {
            names.insert(name.as_basic().to_ascii_lowercase());
        }
        Statement::Const { name, value } => {
            names.insert(name.as_basic().to_ascii_lowercase());
            collect_names_from_expr(value, names);
        }
        Statement::Print { tokens } | Statement::Lprint(tokens) => {
            for t in tokens {
                if let PrintToken::Expr(e) = t {
                    collect_names_from_expr(e, names);
                }
            }
        }
        Statement::PrintUsing { format, tokens } | Statement::LprintUsing { format, tokens } => {
            collect_names_from_expr(format, names);
            for t in tokens {
                if let PrintToken::Expr(e) = t {
                    collect_names_from_expr(e, names);
                }
            }
        }
        Statement::PrintFile { channel, tokens } => {
            collect_names_from_expr(channel, names);
            for t in tokens {
                if let PrintToken::Expr(e) = t {
                    collect_names_from_expr(e, names);
                }
            }
        }
        Statement::PrintFileUsing {
            channel,
            format,
            tokens,
        } => {
            collect_names_from_expr(channel, names);
            collect_names_from_expr(format, names);
            for t in tokens {
                if let PrintToken::Expr(e) = t {
                    collect_names_from_expr(e, names);
                }
            }
        }
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            collect_names_from_expr(condition, names);
            collect_names_from_stmts(then_body, names);
            collect_names_from_stmts(else_body, names);
        }
        Statement::For {
            var,
            start,
            end,
            step,
            body,
        } => {
            names.insert(var.as_basic().to_ascii_lowercase());
            collect_names_from_expr(start, names);
            collect_names_from_expr(end, names);
            if let Some(s) = step {
                collect_names_from_expr(s, names);
            }
            collect_names_from_stmts(body, names);
        }
        Statement::While { condition, body } => {
            collect_names_from_expr(condition, names);
            collect_names_from_stmts(body, names);
        }
        Statement::Do {
            condition,
            body,
            post_condition,
        } => {
            if let Some(c) = condition {
                collect_names_from_expr(&c.expr, names);
            }
            collect_names_from_stmts(body, names);
            if let Some(c) = post_condition {
                collect_names_from_expr(&c.expr, names);
            }
        }
        Statement::SelectCase {
            expr,
            cases,
            else_body,
        } => {
            collect_names_from_expr(expr, names);
            for case in cases {
                for v in &case.values {
                    match v {
                        CaseValue::Single(e) | CaseValue::Is { value: e, .. } => {
                            collect_names_from_expr(e, names);
                        }
                        CaseValue::Range { from, to } => {
                            collect_names_from_expr(from, names);
                            collect_names_from_expr(to, names);
                        }
                    }
                }
                collect_names_from_stmts(&case.body, names);
            }
            collect_names_from_stmts(else_body, names);
        }
        Statement::TryCatch {
            try_body,
            catch,
            finally_body,
        } => {
            collect_names_from_stmts(try_body, names);
            if let Some(catch) = catch {
                names.insert(catch.err_var.as_basic().to_ascii_lowercase());
                for filter_expr in &catch.error_filter {
                    collect_names_from_expr(filter_expr, names);
                }
                names.insert(catch.erl_var.as_basic().to_ascii_lowercase());
                if let Some(source_var) = &catch.source_var {
                    names.insert(source_var.as_basic().to_ascii_lowercase());
                }
                collect_names_from_stmts(&catch.body, names);
            }
            collect_names_from_stmts(finally_body, names);
        }
        Statement::ExprStmt(e) => collect_names_from_expr(e, names),
        Statement::Return { value } => collect_names_from_expr(value, names),
        Statement::Input { vars, .. } | Statement::Read(vars) => {
            for e in vars {
                collect_names_from_expr(e, names);
            }
        }
        Statement::InputFile { channel, vars } => {
            collect_names_from_expr(channel, names);
            for e in vars {
                collect_names_from_expr(e, names);
            }
        }
        Statement::Data(values) => {
            for e in values {
                collect_names_from_expr(e, names);
            }
        }
        Statement::Open { file, channel, .. } => {
            collect_names_from_expr(file, names);
            collect_names_from_expr(channel, names);
        }
        Statement::FileDecl { .. } => {
            unreachable!("record/file DSL must be lowered before codegen")
        }
        Statement::Close { channel } => collect_names_from_expr(channel, names),
        Statement::LineInput { channel, target } => {
            collect_names_from_expr(channel, names);
            collect_names_from_expr(target, names);
        }
        Statement::Write { channel, exprs } => {
            collect_names_from_expr(channel, names);
            for e in exprs {
                collect_names_from_expr(e, names);
            }
        }
        Statement::Field {
            channel, fields, ..
        } => {
            collect_names_from_expr(channel, names);
            for (w, v) in fields {
                collect_names_from_expr(w, names);
                names.insert(v.as_basic().to_ascii_lowercase());
            }
        }
        Statement::Get {
            channel,
            record,
            var,
            ..
        }
        | Statement::Put {
            channel,
            record,
            var,
            ..
        } => {
            collect_names_from_expr(channel, names);
            if let Some(e) = record {
                collect_names_from_expr(e, names);
            }
            if let Some(e) = var {
                collect_names_from_expr(e, names);
            }
        }
        Statement::Lset { var, value } | Statement::Rset { var, value } => {
            names.insert(var.as_basic().to_ascii_lowercase());
            collect_names_from_expr(value, names);
        }
        Statement::Seek { channel, position } => {
            collect_names_from_expr(channel, names);
            collect_names_from_expr(position, names);
        }
        Statement::Locate { row, col } => {
            collect_names_from_expr(row, names);
            collect_names_from_expr(col, names);
        }
        Statement::Color { fg, bg } => {
            collect_names_from_expr(fg, names);
            if let Some(e) = bg {
                collect_names_from_expr(e, names);
            }
        }
        Statement::Poke { address, value } => {
            collect_names_from_expr(address, names);
            collect_names_from_expr(value, names);
        }
        Statement::Out { port, value } => {
            collect_names_from_expr(port, names);
            collect_names_from_expr(value, names);
        }
        Statement::Width { channel, cols } => {
            if let Some(c) = channel {
                collect_names_from_expr(c, names);
            }
            collect_names_from_expr(cols, names);
        }
        Statement::Swap(a, b) => {
            collect_names_from_expr(a, names);
            collect_names_from_expr(b, names);
        }
        Statement::Randomize(e) => {
            if let Some(e) = e {
                collect_names_from_expr(e, names);
            }
        }
        Statement::OnBranch { expr, targets, .. } => {
            collect_names_from_expr(expr, names);
            for t in targets {
                collect_names_from_expr(t, names);
            }
        }
        Statement::OnErrorGoto { target } | Statement::ErrorStmt { code: target } => {
            collect_names_from_expr(target, names);
        }
        Statement::ThrowStmt { code: Some(code) } => collect_names_from_expr(code, names),
        Statement::ThrowStmt { code: None } => {}
        Statement::Goto(e) | Statement::Gosub(e) | Statement::Restore(Some(e)) => {
            collect_names_from_expr(e, names);
        }
        Statement::Resume(kind) => {
            if let ResumeTarget::Line(e) = kind {
                collect_names_from_expr(e, names);
            }
        }
        Statement::OptionBase(e) => collect_names_from_expr(e, names),
        Statement::Kill { file } => collect_names_from_expr(file, names),
        Statement::Name { from, to } => {
            collect_names_from_expr(from, names);
            collect_names_from_expr(to, names);
        }
        Statement::GlobalDecl(ident) => {
            names.insert(ident.as_basic().to_ascii_lowercase());
        }
        Statement::Erase(_)
        | Statement::End
        | Statement::Stop
        | Statement::Cls
        | Statement::Beep
        | Statement::Clear
        | Statement::System
        | Statement::Exit
        | Statement::Continue
        | Statement::Restore(None)
        | Statement::ReturnVoid
        | Statement::Raw(_)
        | Statement::BlockComment(_)
        | Statement::Label(_)
        | Statement::BlankLine => {}
    }
}

fn collect_names_from_expr(expr: &Expr, names: &mut HashSet<String>) {
    match expr {
        Expr::Ident(ident) => {
            names.insert(ident.as_basic().to_ascii_lowercase());
        }
        Expr::ArrayRef { name, indices } => {
            names.insert(name.as_basic().to_ascii_lowercase());
            for i in indices {
                collect_names_from_expr(i, names);
            }
        }
        Expr::Call { args, .. } => {
            for a in args {
                collect_names_from_expr(a, names);
            }
        }
        Expr::Unary { expr, .. } => collect_names_from_expr(expr, names),
        Expr::Binary { left, right, .. } => {
            collect_names_from_expr(left, names);
            collect_names_from_expr(right, names);
        }
        Expr::Integer(_) | Expr::Float(_) | Expr::HexLit(_) | Expr::String(_) => {}
        Expr::FileIndex { .. }
        | Expr::FieldAccess { .. }
        | Expr::MethodCall { .. }
        | Expr::RecordLit { .. } => {
            unreachable!("record/file DSL must be lowered before codegen")
        }
        Expr::ScalarMethodCall { base, args, .. } => {
            collect_names_from_expr(base, names);
            for arg in args {
                collect_names_from_expr(arg, names);
            }
        }
    }
}

/// Collect names from `GlobalDecl` statements anywhere in a function body.
/// These become global-scope names in the emitted BASIC, so they must be
/// excluded from the "taken" set to avoid double-counting but we still need
/// the bare name reserved.
fn collect_global_decl_names(body: &[Stmt], names: &mut HashSet<String>) {
    for stmt in body {
        match &stmt.kind {
            Statement::GlobalDecl(ident) => {
                names.insert(ident.as_basic().to_ascii_lowercase());
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_global_decl_names(then_body, names);
                collect_global_decl_names(else_body, names);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => {
                collect_global_decl_names(body, names);
            }
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_global_decl_names(&case.body, names);
                }
                collect_global_decl_names(else_body, names);
            }
            Statement::TryCatch {
                try_body,
                catch,
                finally_body,
            } => {
                collect_global_decl_names(try_body, names);
                if let Some(catch) = catch {
                    collect_global_decl_names(&catch.body, names);
                }
                collect_global_decl_names(finally_body, names);
            }
            _ => {}
        }
    }
}

fn same_ident(left: &BasicIdent, right: &BasicIdent) -> bool {
    left.suffix == right.suffix && left.name.eq_ignore_ascii_case(&right.name)
}

fn callable_expr(expr: &Expr) -> Option<(&BasicIdent, &[Expr])> {
    match expr {
        Expr::Call { name, args } => Some((name, args)),
        Expr::ArrayRef { name, indices } => Some((name, indices)),
        _ => None,
    }
}

fn semantic_integer_literal_axis(expression: &crate::semantic_ir::Expression) -> Option<i64> {
    use crate::semantic_ir::ExpressionKind;
    match &expression.kind {
        ExpressionKind::Literal(value) => value.parse().ok(),
        ExpressionKind::Parenthesized(inner) => semantic_integer_literal_axis(inner),
        _ => None,
    }
}

fn semantic_array_designator(expression: &crate::semantic_ir::Expression) -> Option<&str> {
    use crate::semantic_ir::ExpressionKind;
    match &expression.kind {
        ExpressionKind::Name(name) => Some(name),
        ExpressionKind::Parenthesized(inner) => semantic_array_designator(inner),
        _ => None,
    }
}

fn semantic_record_storage_names(module: &crate::semantic_ir::SemanticModule) -> HashSet<String> {
    let mut names = HashSet::new();
    for (variable, record_type) in module.record_variable_types() {
        let Some(record) = module
            .records
            .iter()
            .find(|record| record.name.eq_ignore_ascii_case(&record_type))
        else {
            continue;
        };
        let mut fields = Vec::new();
        collect_semantic_record_fields(&module.records, record, &mut fields);
        for field in fields {
            let suffix = match field.field_type {
                crate::semantic_ir::RecordFieldType::String { .. } => TypeSuffix::String,
                crate::semantic_ir::RecordFieldType::Int16 { .. }
                | crate::semantic_ir::RecordFieldType::Int { .. } => TypeSuffix::Integer,
                crate::semantic_ir::RecordFieldType::Int32 { .. } => TypeSuffix::Long,
                crate::semantic_ir::RecordFieldType::Float32 { .. } => TypeSuffix::Single,
                crate::semantic_ir::RecordFieldType::Float64 { .. } => TypeSuffix::Double,
                crate::semantic_ir::RecordFieldType::Record { .. } => continue,
            };
            let storage = BasicIdent {
                name: camel_join(&[&variable, &field.name]),
                suffix: Some(suffix),
            };
            names.insert(storage.as_basic().to_ascii_lowercase());
        }
    }
    names
}

/// Bound for one axis of an array argument: the arguments immediately
/// following the array argument are its per-axis element counts, in the
/// same order as `DIM`'s own bounds -- `axis` 0 is the first of these.
/// Nested copy loop, one FOR per axis, innermost body doing the actual
/// element assignment. `bounds` and `loop_vars` are parallel, one entry per
/// dimension -- rank 1 (the common case) produces exactly the same output
/// as the original single-loop version.
fn array_copy_lines(
    destination: &str,
    source: &str,
    bounds: &[String],
    comment: &str,
    loop_vars: &[String],
) -> Vec<String> {
    let rank = loop_vars.len();
    let mut lines = vec![
        String::new(),
        format!("' {comment}: {source}() -> {destination}()"),
    ];
    for (level, (var, bound)) in loop_vars.iter().zip(bounds.iter()).enumerate() {
        lines.push(format!("{}FOR {var} = 0 TO {bound}", "    ".repeat(level)));
    }
    let index_list = loop_vars.join(", ");
    lines.push(format!(
        "{}{destination}({index_list}) = {source}({index_list})",
        "    ".repeat(rank)
    ));
    for (level, var) in loop_vars.iter().enumerate().rev() {
        lines.push(format!("{}NEXT {var}", "    ".repeat(level)));
    }
    lines.push(String::new());
    lines
}

pub(crate) fn sanitize_symbol(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

/// Joins identifier fragments into one camelCase symbol, with no separator
/// -- BASIC is case-insensitive, but camelCase reads far better in generated
/// output than an underscore chain, and (for real MBASIC/BASCOM targets)
/// underscores in an identifier that's read as an expression operand are a
/// hard compile error, not just a style choice. The first non-empty
/// fragment is lowercased in full; every fragment after that only has its
/// own first character forced to uppercase, with the rest left exactly as
/// given -- never force-lowercased. That matters because a later fragment
/// is sometimes itself an already-camelCased compound built by an earlier
/// `camel_join` call (e.g. records.rs building `sName` before handing it
/// to codegen's own `ident()`, which joins it onto a function stem); force-
/// lowercasing the remainder would flatten `sName` into `Sname`, silently
/// erasing the word boundary it already carried. Non-alphanumeric
/// characters are dropped, since BASIC identifiers are letters and digits
/// only. Collision-freedom is still guaranteed the same way it always was
/// -- by `allocate_unique` checking the result against `taken`, not by
/// this function.
/// The generated BASIC variable name for the const named `base_name`
/// (lowercase, no suffix; CONST bindings are globally keyed by bare name):
/// `CONST` followed by every underscore-separated word of the const's own
/// name, uppercased and run together -- so `HELLO_MSG` becomes
/// `CONSTHELLOMSG`, with no underscore anywhere in the result. Real BASCOM
/// rejects any identifier containing an underscore outright, confirmed
/// under real BASCOM/dosbox-x -- even used only as an assignment target,
/// which ruled out the simpler fix of just keeping the source spelling's
/// case/word-breaks (the previous, buggy `camel_join`-based scheme this
/// replaces). The `CONST` prefix keeps this scheme's own output
/// unmistakable in a `--target basic` listing and collision-free against
/// an ordinarily-cased user variable that happens to share the same
/// letters.
pub(crate) fn const_var_name(base_name: &str) -> String {
    let mut out = String::from("CONST");
    for part in base_name.split('_') {
        out.push_str(&part.to_ascii_uppercase());
    }
    out
}

pub(crate) fn camel_join(parts: &[&str]) -> String {
    let mut out = String::new();
    for part in parts {
        let clean: String = part.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        if clean.is_empty() {
            continue;
        }
        if out.is_empty() {
            out.push_str(&clean.to_ascii_lowercase());
        } else {
            let mut chars = clean.chars();
            if let Some(first) = chars.next() {
                out.push(first.to_ascii_uppercase());
                out.push_str(chars.as_str());
            }
        }
    }
    out
}

/// Flattens a left-associated `&&`/`||` chain (as built by
/// `Parser::parse_condition`) into its operands, left to right.
fn flatten_chain<'a>(expr: &'a Expr, op: BinaryOp, out: &mut Vec<&'a Expr>) {
    match expr {
        Expr::Binary { left, op: o, right } if *o == op => {
            flatten_chain(left, op, out);
            out.push(right);
        }
        _ => out.push(expr),
    }
}

fn binary_op(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Eq => "=",
        BinaryOp::Ne => "<>",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        BinaryOp::And => "AND",
        BinaryOp::Or => "OR",
        BinaryOp::Xor => "XOR",
        BinaryOp::Mod => "MOD",
        BinaryOp::IntDiv => "\\",
        BinaryOp::Pow => "^",
        BinaryOp::AndAnd | BinaryOp::OrOr => {
            unreachable!(
                "&&/|| only valid as an if/while/do condition chain — codegen bug if reached here"
            )
        }
    }
}

fn escape_string(value: &str) -> String {
    value.replace('"', "\"\"")
}

fn ends_with_end(statements: &[Stmt]) -> bool {
    statements
        .iter()
        .rev()
        .find(|s| !matches!(&***s, Statement::BlankLine))
        .is_some_and(|s| matches!(&**s, Statement::End))
}

fn ends_with_return(statements: &[Stmt]) -> bool {
    statements
        .iter()
        .rev()
        .find(|s| !matches!(&***s, Statement::BlankLine))
        .is_some_and(|s| matches!(&**s, Statement::Return { .. } | Statement::ReturnVoid))
}

fn ends_with_emitted_return(
    ast_statements: &[Stmt],
    semantic_statements: &[Option<&crate::semantic_ir::SemanticStatement>],
) -> bool {
    for (index, ast_statement) in ast_statements.iter().enumerate().rev() {
        if let Some(semantic) = semantic_statements.get(index).copied().flatten() {
            if matches!(
                &semantic.kind,
                crate::semantic_ir::SemanticStatementKind::Comment { .. }
            ) {
                continue;
            }
            return matches!(
                &semantic.kind,
                crate::semantic_ir::SemanticStatementKind::Return(_)
            );
        }
        match &**ast_statement {
            Statement::BlankLine | Statement::Raw(_) | Statement::BlockComment(_) => continue,
            Statement::Return { .. } | Statement::ReturnVoid => return true,
            _ => return false,
        }
    }
    false
}

fn ends_with_semantic_return(statements: &[crate::semantic_ir::SemanticStatement]) -> bool {
    fn last_statement(
        statements: &[crate::semantic_ir::SemanticStatement],
    ) -> Option<&crate::semantic_ir::SemanticStatement> {
        for statement in statements.iter().rev() {
            match &statement.kind {
                crate::semantic_ir::SemanticStatementKind::Line(body) => {
                    if let Some(last) = last_statement(body) {
                        return Some(last);
                    }
                }
                crate::semantic_ir::SemanticStatementKind::Comment { .. } => continue,
                _ => return Some(statement),
            }
        }
        None
    }

    last_statement(statements).is_some_and(|statement| {
        matches!(
            &statement.kind,
            crate::semantic_ir::SemanticStatementKind::Return(_)
        )
    })
}

/// Renders `source` (the generator's raw, unnumbered text) into real BASIC
/// line numbers, and reports which original `.bcl` file each numbered line
/// came from -- see `emit_source_file_lookup_subroutine`'s own doc comment
/// for how that second part is used and why it stays correct even though
/// this function is called a second time after that subroutine's own text
/// is appended.
fn number_basic_lines(source: &str, full: bool) -> (String, Vec<(usize, String)>) {
    let lines = source.lines().collect::<Vec<_>>();

    // The `source_file_marker` in effect for each line -- `None` before
    // the first one, or when the program never needed any (the common
    // case: `needs_source_lookup` is false, so `source` has no markers at
    // all and every entry here is `None`).
    let mut line_file: Vec<Option<&str>> = Vec::with_capacity(lines.len());
    let mut current_file: Option<&str> = None;
    for line in &lines {
        if let Some(file) = parse_source_file_marker(line) {
            current_file = Some(file);
        }
        line_file.push(current_file);
    }

    // Lines that survive into the output (non-blank, non-label-only,
    // non-marker)
    let emitted: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| {
            !line.trim().is_empty()
                && is_label_line(line).is_none()
                && parse_source_file_marker(line).is_none()
        })
        .map(|(i, _)| i)
        .collect();

    // In full mode every emitted line is a target; in sparse mode only lines
    // that are actually jumped to receive a number.
    let target_indices: std::collections::HashSet<usize> = if full {
        emitted.iter().copied().collect()
    } else {
        lines
            .iter()
            .enumerate()
            .filter_map(|(index, line)| {
                is_label_line(line)?;
                next_emitted_line_index(&lines, index + 1)
            })
            .collect()
    };

    // Assign sequential line numbers (step 10) to target lines in source order
    let mut index_to_number: HashMap<usize, usize> = HashMap::new();
    let mut current_number = 10usize;
    for &index in &emitted {
        if target_indices.contains(&index) {
            index_to_number.insert(index, current_number);
            current_number += 10;
        }
    }

    // Map each label name to the line number of the first emitted line after it
    let label_numbers: HashMap<String, usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            let label = is_label_line(line)?;
            let target = next_emitted_line_index(&lines, index + 1)?;
            let number = *index_to_number.get(&target)?;
            Some((label.to_string(), number))
        })
        .collect();

    // Names that came from a user `name:` label. Their references are
    // sentinel-wrapped and resolved structurally; every other label is an
    // internal, distinctively-prefixed control-flow label whose references are
    // still resolved by whole-word text match.
    let user_labels: HashSet<String> = lines
        .iter()
        .filter(|line| is_user_label_line(line))
        .filter_map(|line| is_label_line(line))
        .map(str::to_string)
        .collect();

    // Every numbered line's (number, file) pair, in ascending-number
    // (= source) order, collapsed to just the highest number reached
    // within each contiguous same-file run -- see
    // `emit_source_file_lookup_subroutine`'s own doc comment for why an
    // ascending `ERL <= bound` chain built from exactly these pairs
    // correctly identifies the file for any line number the program could
    // ever actually assign to `ERL`.
    let mut breakpoints: Vec<(usize, String)> = Vec::new();
    for &index in &emitted {
        let (Some(&number), Some(file)) = (index_to_number.get(&index), line_file[index]) else {
            continue;
        };
        match breakpoints.last_mut() {
            Some((last_number, last_file)) if last_file == file => *last_number = number,
            _ => breakpoints.push((number, file.to_string())),
        }
    }

    // Walk every intermediate line in order so blank lines pass through.
    // Label-only and marker lines are dropped; everything else is emitted.
    // Consecutive blank lines are folded into a single blank.
    let mut output = String::new();
    let mut last_was_blank = false;
    for (index, &raw) in lines.iter().enumerate() {
        if is_label_line(raw).is_some() || parse_source_file_marker(raw).is_some() {
            continue;
        }
        if raw.trim().is_empty() {
            if !last_was_blank {
                output.push('\n');
                last_was_blank = true;
            }
            continue;
        }
        last_was_blank = false;
        // Sparse-mode target lines are GOTO entry points: trim to column-0 after the
        // number.  Every other line keeps the structural indentation that codegen built
        // up via self.indent, so IF/WHILE/FOR bodies stay visually nested.
        let mut text = if index_to_number.contains_key(&index) && !full {
            raw.trim().to_string()
        } else {
            raw.to_string()
        };
        // User-label references are sentinel-wrapped: an exact structural
        // replace, safe to run unconditionally because the sentinel cannot
        // appear in a comment or a string literal.
        for (label, number) in &label_numbers {
            if user_labels.contains(label) {
                text = text.replace(&user_label_token(label), &number.to_string());
            }
        }
        // Internal control-flow labels are still resolved by whole-word text
        // match. Comment lines are user text, not code — never rewrite label
        // words inside them, even if a label name happens to appear as an
        // ordinary word in the comment.
        if !text.trim_start().starts_with('\'') {
            for (label, number) in &label_numbers {
                if !user_labels.contains(label) {
                    text = replace_label_word(&text, label, &number.to_string());
                }
            }
        }
        if let Some(&number) = index_to_number.get(&index) {
            output.push_str(&format!("{number} {text}\n"));
        } else {
            output.push_str(&format!("{text}\n"));
        }
    }
    (output, breakpoints)
}

fn next_emitted_line_index(lines: &[&str], start: usize) -> Option<usize> {
    for (index, line) in lines.iter().enumerate().skip(start) {
        if is_label_line(line).is_some()
            || line.trim().is_empty()
            || parse_source_file_marker(line).is_some()
        {
            continue;
        }
        return Some(index);
    }
    None
}

fn expr_type_suffix(expr: &Expr) -> &'static str {
    match expr {
        Expr::String(_) => "$",
        Expr::Integer(_) => "%",
        Expr::Float(_) => "!",
        Expr::Ident(ident)
        | Expr::Call { name: ident, .. }
        | Expr::ArrayRef { name: ident, .. } => match ident.suffix {
            Some(TypeSuffix::String) => "$",
            Some(TypeSuffix::Single) => "!",
            Some(TypeSuffix::Double) => "#",
            Some(TypeSuffix::Long) => "&",
            _ => "%",
        },
        Expr::HexLit(_) => "%",
        Expr::Unary { expr, .. } => expr_type_suffix(expr),
        Expr::Binary { left, .. } => expr_type_suffix(left),
        Expr::FileIndex { .. }
        | Expr::FieldAccess { .. }
        | Expr::MethodCall { .. }
        | Expr::ScalarMethodCall { .. }
        | Expr::RecordLit { .. } => {
            unreachable!("record/file DSL must be lowered before codegen")
        }
    }
}

/// True for any generated line that is *only* a label declaration: either a
/// transpiler-internal control-flow label (`IF_0004_END:`, `WHILE_0002_TOP:`,
/// ...) or a user-written `name:` label from BASCAL source (`Statement::Label`).
/// Both kinds are resolved to real BASIC line numbers by `number_basic_lines`
/// and then dropped from the output — codegen never emits any other line
/// that is nothing but an identifier followed by a colon.
fn is_label_line(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let label = trimmed.strip_suffix(':')?;
    // A user label (`Statement::Label`) is emitted sentinel-wrapped; strip the
    // delimiters so the bare name is what gets mapped to a line number.
    let label = strip_user_label_token(label).unwrap_or(label);
    if !label.is_empty() && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        Some(label)
    } else {
        None
    }
}

/// Sentinel delimiters wrapping a *user* label (`Statement::Label`) at both its
/// declaration line and every `GOTO` / `GOSUB` / `ON ERROR GOTO` / `RESUME` /
/// `RESTORE` / `ON ... GOTO` reference. `\x01` / `\x02` cannot occur in
/// generated code, in a re-emitted comment, or inside a string literal, so
/// `number_basic_lines` resolves these references by an exact structural match
/// on the sentinel instead of by whole-word text substitution. That is what
/// lets a user label named `done` or `loop` coexist with the same word
/// appearing in a `PRINT` string or a comment without being corrupted.
const USER_LABEL_OPEN: char = '\u{1}';
const USER_LABEL_CLOSE: char = '\u{2}';

fn user_label_token(name: &str) -> String {
    format!("{USER_LABEL_OPEN}{name}{USER_LABEL_CLOSE}")
}

fn strip_user_label_token(text: &str) -> Option<&str> {
    text.strip_prefix(USER_LABEL_OPEN)?
        .strip_suffix(USER_LABEL_CLOSE)
}

/// True for a label *declaration* line that came from a user `name:` label
/// (as opposed to a transpiler-internal control-flow label like
/// `WHILE_0001_TOP:`).
fn is_user_label_line(line: &str) -> bool {
    line.trim()
        .strip_suffix(':')
        .and_then(strip_user_label_token)
        .is_some()
}

/// Replace whole-word occurrences of `label` in `text` with `replacement`,
/// skipping anything inside a `"..."` string literal. A plain `str::replace`
/// would also match `label` as a substring of an unrelated longer identifier,
/// or inside program output text (e.g. a label named `done` corrupting
/// `PRINT "done"`) — user-chosen label names are short, ordinary words, so
/// that collision is a real risk in a way it never was for the transpiler's
/// own distinctively-prefixed internal labels.
fn replace_label_word(text: &str, label: &str, replacement: &str) -> String {
    if label.is_empty() {
        return text.to_string();
    }
    let is_ident_char = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let mut out = String::with_capacity(text.len());
    let mut in_string = false;
    let mut i = 0;
    while i < text.len() {
        let ch = text[i..].chars().next().unwrap();
        if ch == '"' {
            in_string = !in_string;
            out.push(ch);
            i += 1;
            continue;
        }
        if !in_string && text[i..].starts_with(label) {
            let before_ok = i == 0 || !is_ident_char(text[..i].chars().next_back().unwrap());
            let after_idx = i + label.len();
            let after_ok = after_idx >= text.len()
                || !is_ident_char(text[after_idx..].chars().next().unwrap());
            if before_ok && after_ok {
                out.push_str(replacement);
                i = after_idx;
                continue;
            }
        }
        let ch_len = ch.len_utf8();
        out.push_str(&text[i..i + ch_len]);
        i += ch_len;
    }
    out
}

/// Pre-generation validation: report every global variable whose name matches
/// a transpiler-generated local name (`stem_var_0suffix`), which would silently
/// produce a BASIC program with two distinct roles sharing the same identifier.
///
/// Checks the result variable, every parameter, and every local variable
/// reference in each function body.
pub(crate) fn check_generated_name_conflicts(program: &Program) -> Vec<Diagnostic> {
    let globals = collect_program_names(program);

    // Names the transpiler will never treat as locals.
    let builtin_stems: HashSet<&str> = BASIC_BUILTINS.iter().copied().collect();
    let function_stems: HashSet<String> = program
        .functions
        .iter()
        .map(|f| sanitize_symbol(&f.name.name))
        .collect();

    let mut diagnostics = Vec::new();

    for func in &program.functions {
        let stem = sanitize_symbol(&func.name.name);
        let param_keys: HashSet<String> = func
            .params
            .iter()
            .map(|p| p.name.as_basic().to_ascii_lowercase())
            .collect();
        let global_decls = collect_globals(&func.body);

        // ── result variable ────────────────────────────────────────────────
        check_one_conflict(
            &globals,
            &stem,
            "result",
            func.name.suffix,
            &func.name,
            &format!("result variable for `{}`", func.name.as_basic()),
            &mut diagnostics,
        );

        // ── parameters ────────────────────────────────────────────────────
        for param in &func.params {
            check_one_conflict(
                &globals,
                &stem,
                &sanitize_symbol(&param.name.name),
                param.name.suffix,
                &func.name,
                &format!(
                    "parameter `{}` of `{}`",
                    param.name.as_basic(),
                    func.name.as_basic()
                ),
                &mut diagnostics,
            );
        }

        // ── locals referenced in the body ──────────────────────────────────
        let mut body_names: HashSet<String> = HashSet::new();
        collect_names_from_stmts(&func.body, &mut body_names);

        for key in &body_names {
            if param_keys.contains(key) || global_decls.contains(key) {
                continue;
            }
            let local = BasicIdent::parse(key);
            let bare = local.name.to_ascii_lowercase();
            if builtin_stems.contains(bare.as_str()) || function_stems.contains(&bare) {
                continue;
            }
            check_one_conflict(
                &globals,
                &stem,
                &sanitize_symbol(&local.name),
                local.suffix,
                &func.name,
                &format!(
                    "local variable `{}` in `{}`",
                    local.as_basic(),
                    func.name.as_basic()
                ),
                &mut diagnostics,
            );
        }
    }

    diagnostics
}

/// Semantic-IR entry point for generated-name validation. The legacy AST
/// overload remains for parser-only compatibility callers.
pub(crate) fn check_generated_name_conflicts_semantic(
    module: &crate::semantic_ir::SemanticModule,
    common_blocks: &[CommonBlock],
) -> Vec<Diagnostic> {
    use crate::semantic_ir::CallableKind;
    let mut globals: HashSet<String> = module.name_scopes().global_names.into_iter().collect();
    for callable in &module.callables {
        globals.extend(crate::semantic_ir::SemanticModule::global_declarations_in(
            &callable.body,
        ));
    }
    for block in common_blocks {
        for variable in &block.vars {
            globals.insert(variable.name.as_basic().to_ascii_lowercase());
        }
    }
    let builtin_stems: HashSet<&str> = BASIC_BUILTINS.iter().copied().collect();
    let function_stems: HashSet<String> = module
        .callables
        .iter()
        .map(|callable| sanitize_symbol(&BasicIdent::parse(&callable.name).name))
        .collect();
    let mut diagnostics = Vec::new();
    for callable in &module.callables {
        if !matches!(
            callable.kind,
            CallableKind::Function
                | CallableKind::Procedure
                | CallableKind::Method
                | CallableKind::FluentMethod
                | CallableKind::InlineMethod
        ) {
            continue;
        }
        let function = BasicIdent::parse(&callable.name);
        let stem = sanitize_symbol(&function.name);
        let param_keys: HashSet<String> = callable
            .parameters
            .iter()
            .map(|parameter| {
                BasicIdent::parse(&parameter.name)
                    .as_basic()
                    .to_ascii_lowercase()
            })
            .collect();
        let global_decls =
            crate::semantic_ir::SemanticModule::global_declarations_in(&callable.body);
        check_one_conflict(
            &globals,
            &stem,
            "result",
            function.suffix,
            &function,
            &format!("result variable for `{}`", function.as_basic()),
            &mut diagnostics,
        );
        for parameter in &callable.parameters {
            let parameter_ident = BasicIdent::parse(&parameter.name);
            check_one_conflict(
                &globals,
                &stem,
                &sanitize_symbol(&parameter_ident.name),
                parameter_ident.suffix,
                &function,
                &format!(
                    "parameter `{}` of `{}`",
                    parameter_ident.as_basic(),
                    function.as_basic()
                ),
                &mut diagnostics,
            );
        }
        for key in crate::semantic_ir::SemanticModule::names_in_statements(&callable.body) {
            if param_keys.contains(&key) || global_decls.contains(&key) {
                continue;
            }
            let local = BasicIdent::parse(&key);
            let bare = local.name.to_ascii_lowercase();
            if builtin_stems.contains(bare.as_str()) || function_stems.contains(&bare) {
                continue;
            }
            check_one_conflict(
                &globals,
                &stem,
                &sanitize_symbol(&local.name),
                local.suffix,
                &function,
                &format!(
                    "local variable `{}` in `{}`",
                    local.as_basic(),
                    function.as_basic()
                ),
                &mut diagnostics,
            );
        }
    }
    diagnostics
}

fn check_one_conflict(
    globals: &HashSet<String>,
    stem: &str,
    var_stem: &str,
    suffix: Option<TypeSuffix>,
    func_name: &BasicIdent,
    description: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let candidate = BasicIdent {
        name: format!("{}0", camel_join(&[stem, var_stem])),
        suffix,
    };
    if globals.contains(&candidate.as_basic().to_ascii_lowercase()) {
        diagnostics.push(Diagnostic::error(
            SourcePos::new("<validation>", 1, 1),
            format!(
                "global `{}` conflicts with the transpiler-generated name for {}; \
                 rename the global or the function `{}`",
                candidate.as_basic(),
                description,
                func_name.as_basic(),
            ),
        ));
    }
}
