# [Conformance tests](../)

## JVM-specific

| Test description | Result |
| --- | :---: |
| card_catalog_example_runs_under_jvm_when_available | PASS |
| Hello-world assembly and execution | PASS |
| jvm_byref_scalar_parameters_write_back_to_the_caller_when_available | PASS |
| Expected failure for array byval clone assembly | FAIL |
| Catch filters and source bindings | PASS |
| jvm_color_uses_the_correct_cga_to_ansi_mapping_when_available | PASS |
| jvm_date_dollar_matches_mm_dd_yyyy_format_when_available | PASS |
| jvm_double_to_string_rounds_to_six_significant_digits_when_available | PASS |
| Expected diagnostic for MID$ assignment | FAIL |
| Expected diagnostic for random/record file I/O | FAIL |
| Expected diagnostic for sequential file I/O | FAIL |
| jvm_field_buffer_registers_when_the_file_open_is_wrapped_in_try_catch | PASS |
| jvm_function_call_as_bare_statement_discards_its_result_when_available | PASS |
| jvm_get_on_a_fresh_empty_file_does_not_throw_when_available | PASS |
| jvm_inkey_polls_without_crashing_under_piped_input_when_available | PASS |
| jvm_inkey_setup_does_not_disable_output_postprocessing | PASS |
| jvm_input_after_inkey_reads_the_typed_value_when_available | PASS |
| jvm_instr_and_stop_and_system_compile | PASS |
| jvm_inventory_list_all_without_a_garbled_press_any_key_prompt_when_available | PASS |
| Typed non-integer arrays and array parameters | PASS |
| jvm_print_without_trailing_newline_flushes_stdout | PASS |
| jvm_random_access_file_round_trips_when_available | PASS |
| jvm_random_and_record_files_tutorial_runs_when_available | PASS |
| jvm_select_case_registers_variables_declared_only_inside_a_case_clause | PASS |
| jvm_tab_spc_and_dynamic_locate_match_c_backend_when_available | PASS |
| Structured TRY/CATCH/FINALLY execution | PASS |
| Numeric literals and arithmetic | PASS |
| Portable error-handling tutorial | PASS |
| Scalar function calls and returns | PASS |
| Scalar variables and constants | PASS |
| Scoped GOTO labels | PASS |
| Structured branches and WHILE loops | PASS |
| Existing random-file records compile on all targets | FAIL |

<nav class="conformance-nav" aria-label="Conformance results navigation">
  <a href="../c/">← Previous: C-specific</a>
  <a href="../">← Overview</a>
  <a href="../records/">Next: Files and records →</a>
</nav>
