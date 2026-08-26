# Conformance results

This page records the backend conformance tests shipped with BASCAL. `PASS`
means the test currently passes in the normal test environment; `N/A` means
that the test does not apply to that backend (or requires an optional runtime
not present in the ordinary build). The matrix is kept alongside the tests in
`tests/` and should be updated whenever a backend capability changes.

| Test | BASIC | C | JVM |
| --- | :---: | :---: | :---: |
| `language_conformance::conformance_fixtures_transpile_on_their_supported_backends` | PASS | PASS | PASS |
| `examples::compiles_every_example_bcl_file` | PASS | N/A | N/A |
| `examples::c_target_builds_and_runs_noninteractive_tutorials` | N/A | PASS | N/A |
| `examples::c_target_rejects_labels_and_error_handling_tutorial` | N/A | PASS | N/A |
| `examples::gcc_runs_try_catch_through_nested_procedure_calls_under_c_target_when_available` | N/A | PASS | N/A |
| `examples::gcc_runs_inventory_tutorial_under_c_target_when_available` | N/A | PASS | N/A |
| `examples::gcc_runs_remline_under_c_target_when_available` | N/A | PASS | N/A |
| `examples::freebasic_runs_mid_assign_edge_cases_when_available` | PASS | N/A | N/A |
| `examples::freebasic_runs_self_referential_string_concatenation_when_available` | PASS | N/A | N/A |
| `examples::freebasic_runs_builtin_scalar_methods_when_available` | PASS | N/A | N/A |
| `examples::freebasic_runs_stdlib_functions_when_available` | PASS | N/A | N/A |
| `examples::freebasic_runs_remline_when_available` | PASS | N/A | N/A |
| `jvm_conformance::jvm_try_catch_finally_runs_when_available` | N/A | N/A | PASS |
| `jvm_conformance::jvm_non_integer_arrays_run_when_available` | N/A | N/A | PASS |
| `jvm_conformance::jvm_catch_filters_and_source_bindings_run_when_available` | N/A | N/A | PASS |
| `jvm_conformance::portable_error_handling_tutorial_runs_when_available` | N/A | N/A | PASS |
| `jvm_conformance::hello_world_transpiles_assembles_and_runs_when_available` | N/A | N/A | PASS |
| `jvm_conformance::numeric_literals_and_arithmetic_run_when_available` | N/A | N/A | PASS |
| `jvm_conformance::scalar_variables_and_constants_run_when_available` | N/A | N/A | PASS |
| `jvm_conformance::structured_branches_and_while_loops_run_when_available` | N/A | N/A | PASS |
| `jvm_conformance::scalar_functions_run_when_available` | N/A | N/A | PASS |
| `jvm_conformance::scoped_goto_runs_when_available` | N/A | N/A | PASS |
| `jvm_conformance::jvm_record_binary_compatibility_with_basic_and_c_is_pending` | N/A | N/A | N/A |
| `dosbox_conformance::const_and_print_matches_real_bascom` | PASS | PASS | N/A |
| `dosbox_conformance::mid_assign_matches_real_bascom` | PASS | PASS | N/A |
| `dosbox_conformance::stdlib_functions_match_real_bascom` | PASS | PASS | N/A |
| `dosbox_conformance::stdlib_functions_match_c_target` | N/A | PASS | N/A |
| `dosbox_conformance::self_referential_string_concatenation_matches_real_bascom` | PASS | PASS | N/A |
| `dosbox_conformance::self_referential_string_concatenation_matches_c_target` | N/A | PASS | N/A |
| `dosbox_conformance::builtin_scalar_methods_match_real_bascom` | PASS | PASS | N/A |
| `dosbox_conformance::builtin_scalar_methods_match_c_target` | N/A | PASS | N/A |
| `dosbox_conformance::c_target_random_access_file_is_binary_compatible_with_real_bascom_c_writes` | N/A | PASS | N/A |
| `dosbox_conformance::c_target_random_access_file_is_binary_compatible_with_real_bascom_bascom_writes` | N/A | PASS | N/A |
| `dosbox_conformance::tie_break_rounding_matches_real_bascom` | PASS | PASS | N/A |
| `record_general_purpose::standalone_record_literal_currently_fails` | PASS | PASS | PASS |
| `record_general_purpose::plain_dim_of_record_type_currently_fails` | PASS | PASS | PASS |
| `record_general_purpose::array_of_records_currently_fails` | PASS | PASS | PASS |
| `record_general_purpose::record_valued_parameter_currently_fails` | PASS | PASS | PASS |
| `record_general_purpose::record_valued_return_currently_fails` | PASS | PASS | PASS |
| `record_general_purpose::nested_record_field_currently_fails` | PASS | PASS | PASS |
| `record_general_purpose::bare_dynamic_string_field_currently_fails_to_parse` | PASS | PASS | PASS |
| `record_general_purpose::existing_random_access_file_record_usage_still_compiles_on_basic_and_c` | PASS | PASS | N/A |
| `record_general_purpose::jvm_backend_does_not_yet_support_random_access_file_records` | N/A | N/A | PASS |

The optional BASCOM/DOSBox and Krakatau/JVM tests are skipped when their
external tools are unavailable; those environments are exercised by the
dedicated conformance workflow. The one ignored JVM record test is an
intentional pending check for issue #105.
