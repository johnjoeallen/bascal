# [Conformance tests](../)

## JVM-specific

| Test description | Result |
| --- | :---: |
| card_catalog_example_runs_under_jvm_when_available | PASS |
| Hello-world assembly and execution | PASS |
| jvm_byref_scalar_parameters_write_back_to_the_caller_when_available | PASS |
| jvm_byval_array_procedure_receives_a_copy | PASS |
| jvm_callable_dynamic_array_preserves_fixed_radix_axis_when_available | PASS |
| jvm_callable_dynamic_dim_allocates_inside_typed_branch_when_available | PASS |
| jvm_callable_dynamic_dim_evaluates_typed_bound_function_once_when_available | PASS |
| jvm_callable_dynamic_dim_uses_callable_typed_parameter_when_available | PASS |
| jvm_callable_dynamic_multidimensional_array_uses_typed_local_slot_when_available | PASS |
| jvm_callable_dynamic_string_array_uses_local_typed_slot_when_available | PASS |
| jvm_callable_end_exits_the_process_when_available | PASS |
| jvm_callable_typed_restore_updates_shared_data_cursor_when_available | PASS |
| Catch filters and source bindings | PASS |
| jvm_color_uses_the_correct_cga_to_ansi_mapping_when_available | PASS |
| jvm_conditional_typed_read_does_not_advance_when_branch_is_skipped | PASS |
| jvm_date_dollar_matches_mm_dd_yyyy_format_when_available | PASS |
| jvm_double_to_string_rounds_to_six_significant_digits_when_available | PASS |
| jvm_dynamic_array_preserves_fixed_octal_axis_when_available | PASS |
| jvm_dynamic_array_preserves_fixed_radix_axis_when_available | PASS |
| jvm_dynamic_array_preserves_zero_upper_bound_when_available | PASS |
| jvm_dynamic_color_expressions_run_when_available | PASS |
| jvm_dynamic_dim_evaluates_side_effecting_typed_bound_once_when_available | PASS |
| jvm_dynamic_dim_evaluates_typed_bound_at_statement_position_when_available | PASS |
| jvm_dynamic_dim_transpiles_typed_arithmetic_bound_when_available | PASS |
| jvm_dynamic_long_array_uses_typed_capacity_and_element_type_when_available | PASS |
| jvm_dynamic_multidimensional_array_uses_typed_axes_when_available | PASS |
| jvm_dynamic_multidimensional_bounds_each_evaluate_once_when_available | PASS |
| jvm_dynamic_string_array_uses_typed_capacity_when_available | PASS |
| Expected diagnostic for random/record file I/O | FAIL |
| jvm_expected_failure_read_without_data_has_specific_diagnostic | PASS |
| Expected diagnostic for sequential file I/O | FAIL |
| jvm_field_buffer_registers_when_the_file_open_is_wrapped_in_try_catch | PASS |
| jvm_for_limit_is_captured_before_body_mutation_when_available | PASS |
| jvm_for_limit_is_captured_before_mutating_method_step_when_available | PASS |
| jvm_for_start_is_captured_before_mutating_method_bound_when_available | PASS |
| jvm_function_and_scalar_method_default_arguments_run_when_available | PASS |
| jvm_function_call_as_bare_statement_discards_its_result_when_available | PASS |
| jvm_get_on_a_fresh_empty_file_does_not_throw_when_available | PASS |
| jvm_inkey_polls_without_crashing_under_piped_input_when_available | PASS |
| jvm_inkey_setup_does_not_disable_output_postprocessing | PASS |
| jvm_input_after_inkey_reads_the_typed_value_when_available | PASS |
| jvm_instr_and_stop_and_system_compile | PASS |
| jvm_inventory_list_all_without_a_garbled_press_any_key_prompt_when_available | PASS |
| jvm_multi_index_arrays_run_when_available | PASS |
| jvm_nested_dynamic_dim_allocates_only_in_typed_branch_when_available | PASS |
| jvm_nested_typed_output_open_truncates_and_writes_when_available | PASS |
| jvm_nested_typed_read_consumes_data_items_when_available | PASS |
| jvm_nested_typed_read_stores_into_array_elements_when_available | PASS |
| jvm_nested_typed_write_emits_csv_file_bytes_when_available | PASS |
| Typed non-integer arrays and array parameters | PASS |
| jvm_print_without_trailing_newline_flushes_stdout | PASS |
| jvm_random_access_file_round_trips_when_available | PASS |
| jvm_random_and_record_files_tutorial_runs_when_available | PASS |
| jvm_scalar_method_calls_copy_byval_array_arguments_when_available | PASS |
| jvm_scalar_method_calls_pass_byref_array_arguments_when_available | PASS |
| jvm_scalar_method_calls_write_back_byref_string_arguments_when_available | PASS |
| jvm_scalar_method_double_receiver_widens_integer_argument_when_available | PASS |
| jvm_select_case_registers_variables_declared_only_inside_a_case_clause | PASS |
| jvm_semantic_block_if_runs_when_available | PASS |
| jvm_semantic_channel_print_writes_typed_file_bytes_when_available | PASS |
| jvm_semantic_do_transfers_run_when_available | PASS |
| jvm_semantic_double_comparisons_follow_ieee_ordering_when_available | PASS |
| jvm_semantic_for_method_step_is_evaluated_once_when_available | PASS |
| jvm_semantic_for_transfers_run_when_available | PASS |
| jvm_semantic_function_calls_copy_typed_byval_arrays_when_available | PASS |
| jvm_semantic_function_calls_pass_typed_byref_arrays_when_available | PASS |
| jvm_semantic_method_loop_conditions_run_each_iteration_when_available | PASS |
| jvm_semantic_mid_assignment_runs_when_available | PASS |
| jvm_semantic_nested_field_layout_round_trips_when_available | PASS |
| jvm_semantic_nested_field_sets_round_trip_when_available | PASS |
| jvm_semantic_nested_file_line_input_reads_a_typed_string_when_available | PASS |
| jvm_semantic_nested_get_put_round_trip_when_available | PASS |
| jvm_semantic_nested_input_reads_value_when_available | PASS |
| jvm_semantic_nested_locate_color_run_when_available | PASS |
| jvm_semantic_nested_random_file_lifecycle_runs_when_available | PASS |
| jvm_semantic_nested_rename_moves_file_when_available | PASS |
| jvm_semantic_nested_return_writes_back_byref_when_available | PASS |
| jvm_semantic_nested_swap_runs_when_available | PASS |
| jvm_semantic_random_open_evaluates_channel_once_when_available | PASS |
| jvm_semantic_seek_runs_at_module_and_nested_scope_when_available | PASS |
| jvm_semantic_select_case_runs_when_available | PASS |
| jvm_semantic_single_arithmetic_preserves_float32_precision_when_available | PASS |
| jvm_semantic_while_runs_when_available | PASS |
| jvm_semantic_while_transfers_run_when_available | PASS |
| jvm_skipped_callable_dynamic_dim_does_not_evaluate_typed_bound_when_available | PASS |
| jvm_skipped_dynamic_dim_does_not_evaluate_typed_bound_when_available | PASS |
| jvm_string_and_constant_default_arguments_run_when_available | PASS |
| jvm_tab_spc_and_dynamic_locate_match_c_backend_when_available | PASS |
| Structured TRY/CATCH/FINALLY execution | PASS |
| jvm_typed_data_in_callable_body_is_added_to_shared_pool_when_available | PASS |
| jvm_typed_data_in_conditional_block_is_included_in_pool | PASS |
| jvm_typed_on_goto_assigns_unique_labels_for_multiple_statements_when_available | PASS |
| jvm_typed_on_goto_converts_long_selector_for_later_target_when_available | PASS |
| jvm_typed_on_goto_falls_through_outside_target_range_when_available | PASS |
| jvm_typed_on_goto_selector_above_target_count_falls_through_when_available | PASS |
| jvm_typed_on_goto_selector_one_selects_first_label_when_available | PASS |
| jvm_typed_on_goto_uses_one_based_label_dispatch_when_available | PASS |
| jvm_typed_read_array_index_expression_is_evaluated_once_when_available | PASS |
| jvm_typed_read_converts_long_double_and_string_data_when_available | PASS |
| jvm_typed_read_reports_out_of_data_when_available | PASS |
| jvm_typed_read_stores_string_data_into_array_elements_when_available | PASS |
| jvm_typed_restore_resets_data_cursor_to_start_and_label_when_available | PASS |
| jvm_typed_restore_resolves_labels_in_nested_blocks_when_available | PASS |
| jvm_typed_single_read_preserves_binary32_rounding_when_available | PASS |
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
