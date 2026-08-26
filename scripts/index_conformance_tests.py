#!/usr/bin/env python3
"""Index conformance tests and tutorials by stable, source-derived IDs."""
from pathlib import Path
import re

root = Path(__file__).resolve().parents[1]
ids = {}
valid_statuses = {"PASS", "FAIL", "UNSUPPORTED", "DEFERRED", "WILL NOT IMPLEMENT", "NOT APPLICABLE", "UNKNOWN"}
descriptions = {
    "builtin_scalar_methods_match_c_target": "Scalar methods match C output",
    "builtin_scalar_methods_match_real_bascom": "Scalar methods match BASCOM",
    "const_and_print_matches_real_bascom": "Constants and printing match BASCOM",
    "mid_assign_matches_real_bascom": "MID$ assignment matches BASCOM",
    "self_referential_string_concatenation_matches_c_target": "String self-concatenation matches C output",
    "self_referential_string_concatenation_matches_real_bascom": "String self-concatenation matches BASCOM",
    "stdlib_functions_match_c_target": "Standard-library functions match C output",
    "stdlib_functions_match_real_bascom": "Standard-library functions match BASCOM",
    "tie_break_rounding_matches_real_bascom": "Tie-break rounding matches BASCOM",
    "c_target_builds_and_runs_noninteractive_tutorials": "Deterministic tutorials build and execute",
    "c_target_rejects_labels_and_error_handling_tutorial": "C rejects classic labels and error handling",
    "compiles_every_example_bcl_file": "Every supported fixture transpiles successfully",
    "freebasic_runs_builtin_scalar_methods_when_available": "Built-in scalar methods under FreeBASIC",
    "freebasic_runs_mid_assign_edge_cases_when_available": "MID$ assignment edge cases under FreeBASIC",
    "freebasic_runs_remline_when_available": "remline output under FreeBASIC",
    "freebasic_runs_self_referential_string_concatenation_when_available": "Self-referential string concatenation under FreeBASIC",
    "freebasic_runs_sort_driver_when_available": "Sort driver under FreeBASIC",
    "freebasic_runs_stdlib_functions_when_available": "Standard-library functions under FreeBASIC",
    "gcc_runs_inventory_tutorial_under_c_target_when_available": "Interactive inventory case study",
    "gcc_runs_remline_under_c_target_when_available": "remline case study output",
    "gcc_runs_try_catch_through_nested_procedure_calls_under_c_target_when_available": "Nested procedure TRY/CATCH propagation",
    "jvm_try_catch_finally_runs_when_available": "Structured TRY/CATCH/FINALLY execution",
    "jvm_catch_filters_and_source_bindings_run_when_available": "Catch filters and source bindings",
    "portable_error_handling_tutorial_runs_when_available": "Portable error-handling tutorial",
    "hello_world_transpiles_assembles_and_runs_when_available": "Hello-world assembly and execution",
    "numeric_literals_and_arithmetic_run_when_available": "Numeric literals and arithmetic",
    "scalar_variables_and_constants_run_when_available": "Scalar variables and constants",
    "structured_branches_and_while_loops_run_when_available": "Structured branches and WHILE loops",
    "scalar_functions_run_when_available": "Scalar function calls and returns",
    "scoped_goto_runs_when_available": "Scoped GOTO labels",
    "jvm_non_integer_arrays_run_when_available": "Typed non-integer arrays and array parameters",
    "c_target_random_file_is_binary_compatible_with_real_bascom_c_writes": "C random-file output is readable by BASCOM",
    "c_target_random_file_is_binary_compatible_with_real_bascom_bascom_writes": "BASCOM random-file output is readable by C",
    "c_target_random_access_file_is_binary_compatible_with_real_bascom_bascom_writes": "BASCOM random-file output is readable by C",
    "c_target_random_access_file_is_binary_compatible_with_real_bascom_c_writes": "C random-file output is readable by BASCOM",
    "existing_random_access_file_record_usage_still_compiles_on_basic_and_c": "Existing random-file records compile on BASIC and C",
    "jvm_backend_does_not_yet_support_random_access_file_records": "JVM random-file records produce the expected diagnostic",
    "jvm_random_file_binary_compatibility_is_pending": "Random file binary compatibility",
    "jvm_byval_arrays_expected_failure_is_non_blocking": "Expected failure for array byval clone assembly",
    "jvm_expected_failure_mid_assignment_is_non_blocking": "Expected diagnostic for MID$ assignment",
    "jvm_expected_failure_random_record_io_is_non_blocking": "Expected diagnostic for random/record file I/O",
    "jvm_expected_failure_sequential_file_io_is_non_blocking": "Expected diagnostic for sequential file I/O",
    "conformance_fixtures_transpile_on_their_supported_backends": "Every supported fixture transpiles successfully",
    "array_of_records_currently_fails": "Array of records is not yet supported",
    "bare_dynamic_string_field_currently_fails_to_parse": "Bare dynamic record string fields are not yet supported",
    "nested_record_field_currently_fails": "Nested record fields are not yet supported",
    "plain_dim_of_record_type_currently_fails": "Arrays of records are not yet supported",
    "record_valued_parameter_currently_fails": "Record-valued parameters are not yet supported",
    "record_valued_return_currently_fails": "Record-valued returns are not yet supported",
    "standalone_record_literal_currently_fails": "Standalone record literals are not yet supported",
}
# Most tests in this module exercise both the BASIC and C targets.  Keep
# target/file-specific cases in their precise groups rather than inheriting
# the module-wide default, so generated backend pages do not show unrelated
# tests or spurious UNKNOWN cells.
group_overrides = {
    "builtin_scalar_methods_match_real_bascom": ["basic"],
    "const_and_print_matches_real_bascom": ["basic"],
    "mid_assign_matches_real_bascom": ["basic"],
    "stdlib_functions_match_real_bascom": ["basic"],
    "self_referential_string_concatenation_matches_real_bascom": ["basic"],
    "tie_break_rounding_matches_real_bascom": ["basic"],
    "builtin_scalar_methods_match_c_target": ["c"],
    "stdlib_functions_match_c_target": ["c"],
    "self_referential_string_concatenation_matches_c_target": ["c"],
    "c_target_builds_and_runs_noninteractive_tutorials": ["c"],
    "c_target_rejects_labels_and_error_handling_tutorial": ["c"],
    "gcc_runs_try_catch_through_nested_procedure_calls_under_c_target_when_available": ["c"],
    "gcc_runs_inventory_tutorial_under_c_target_when_available": ["c"],
    "gcc_runs_remline_under_c_target_when_available": ["c"],
    "existing_random_access_file_record_usage_still_compiles_on_basic_and_c": ["basic", "c", "records"],
    "jvm_backend_does_not_yet_support_random_access_file_records": ["jvm", "records"],
    "c_target_random_access_file_is_binary_compatible_with_real_bascom_bascom_writes": ["core", "records"],
    "c_target_random_access_file_is_binary_compatible_with_real_bascom_c_writes": ["core", "records"],
}
status_overrides = {
    "c_target_random_access_file_is_binary_compatible_with_real_bascom_bascom_writes": {"basic": "PASS", "c": "PASS", "jvm": "FAIL"},
    "c_target_random_access_file_is_binary_compatible_with_real_bascom_c_writes": {"basic": "PASS", "c": "PASS", "jvm": "FAIL"},
}

for path in sorted((root / "tests").glob("*.rs")):
    text = path.read_text()
    if "Conformance groups:" not in text:
        continue
    module = path.stem
    for name in re.findall(r"#\[test\]\s*(?:#\[[^\]]+\]\s*)*fn\s+([A-Za-z0-9_]+)", text):
        test_id = f"test.{module}.{name}"
        if test_id in ids:
            raise SystemExit(f"duplicate conformance ID: {test_id}")
        module_groups = re.search(r"^// Conformance groups:\s*(.+)$", text, re.MULTILINE).group(1).split(", ")
        groups = group_overrides.get(name, module_groups)
        expected = {
            backend: (
                "PASS"
                if backend in groups or "core" in groups
                else "NOT APPLICABLE"
                if "files" in groups or "records" in groups
                else "UNKNOWN"
            )
            for backend in ("basic", "c", "jvm")
        }
        if name == "conformance_fixtures_transpile_on_their_supported_backends":
            expected = {backend: "PASS" for backend in ("basic", "c", "jvm")}
        expected.update(status_overrides.get(name, {}))
        ids[test_id] = ("test", module, name, groups, expected)

import tomllib
meta = tomllib.loads((root / "tutorial" / "conformance.toml").read_text())
for item in meta["tutorial"]:
    source = Path(item["source"]).with_suffix("").as_posix().replace("/", ".")
    test_id = f"tutorial.{source}"
    if test_id in ids:
        raise SystemExit(f"duplicate conformance ID: {test_id}")
    status = dict(item.get("expected", item.get("status", {})))
    for backend in ("basic", "c", "jvm"):
        if backend not in status:
            if backend in item.get("wont", []):
                status[backend] = "WILL NOT IMPLEMENT"
            elif backend in item.get("na", []):
                status[backend] = "UNSUPPORTED"
            elif backend in item.get("backends", []):
                status[backend] = "PASS"
            else:
                status[backend] = "DEFERRED"
    invalid = {backend: state for backend, state in status.items() if state.upper() not in valid_statuses}
    if invalid:
        raise SystemExit(f"{test_id}: invalid status values: {invalid}")
    ids[test_id] = ("tutorial", item["source"], item["name"], ["tutorials"], status)

metadata = tomllib.loads((root / "conformance" / "metadata.toml").read_text())
for item in metadata["test"]:
    test_id = item["id"]
    if test_id in ids:
        raise SystemExit(f"duplicate conformance ID: {test_id}")
    ids[test_id] = ("test", "metadata", item["description"], item["groups"], item["expected"])

out = ["# Generated by scripts/index_conformance_tests.py", ""]
for test_id, (kind, source, name, groups, status) in sorted(ids.items()):
    group_text = "[" + ", ".join(f'\"{group}\"' for group in groups) + "]"
    description = descriptions.get(name, name).replace('"', '\\"')
    out.append(f'[[test]]\nid = "{test_id}"\nkind = "{kind}"\nsource = "{source}"\nname = "{name}"\ndescription = "{description}"\ngroups = {group_text}\nexpected = {{ basic = "{status.get("basic", "UNKNOWN")}", c = "{status.get("c", "UNKNOWN")}", jvm = "{status.get("jvm", "UNKNOWN")}" }}\n')
(root / "conformance").mkdir(exist_ok=True)
(root / "conformance" / "test-index.toml").write_text("\n".join(out))
print(f"indexed {len(ids)} conformance tests/tutorials")
