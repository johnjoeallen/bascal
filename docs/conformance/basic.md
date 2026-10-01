# [Conformance tests](../)

## BASIC-specific

| Test description | Result |
| --- | :---: |
| Scalar methods match BASCOM | PASS |
| Constants and printing match BASCOM | PASS |
| MID$ assignment matches BASCOM | PASS |
| nested_typed_on_gosub_matches_c_target | PASS |
| String self-concatenation matches BASCOM | PASS |
| Standard-library functions match BASCOM | PASS |
| Tie-break rounding matches BASCOM | PASS |
| c_callable_end_exits_the_process | PASS |
| c_driver_runs_callable_nested_typed_output_open | PASS |
| c_driver_runs_nested_semantic_input_through_typed_ir | PASS |
| c_driver_runs_nested_typed_data_read | PASS |
| c_driver_runs_nested_typed_file_write_after_output_open | PASS |
| c_driver_transpiles_callable_array_indices_from_typed_ir | PASS |
| c_numeric_user_function_calls_compile_and_run | PASS |
| c_print_without_trailing_newline_flushes_stdout | PASS |
| c_record_dsl_open_registers_synthesized_error_runtime | PASS |
| c_record_dsl_partial_update_preserves_omitted_field_runtime | PASS |
| c_record_variable_write_roundtrips_all_typed_fields | PASS |
| c_semantic_callable_dynamic_array_evaluates_typed_bound_function_once | PASS |
| c_semantic_callable_dynamic_array_uses_typed_parameter | PASS |
| c_semantic_dynamic_array_allocates_inside_typed_branch | PASS |
| c_semantic_dynamic_array_evaluates_typed_bound_function_once | PASS |
| c_semantic_dynamic_array_preserves_fixed_octal_axis_at_runtime | PASS |
| c_semantic_dynamic_array_preserves_fixed_radix_axis_at_runtime | PASS |
| c_semantic_dynamic_array_preserves_zero_upper_bound | PASS |
| c_semantic_dynamic_dim_evaluates_typed_arithmetic_bound | PASS |
| c_semantic_dynamic_long_array_uses_suffix_inferred_element_type | PASS |
| c_semantic_dynamic_multi_index_arrays_compile_and_run | PASS |
| c_semantic_file_input_stores_indexed_array_values | PASS |
| c_semantic_lset_packed_numeric_record_dsl_roundtrip | PASS |
| c_semantic_lset_packed_numeric_runtime_roundtrip | PASS |
| c_semantic_multi_index_arrays_compile_and_run | PASS |
| c_semantic_multidimensional_bounds_each_evaluate_once | PASS |
| c_semantic_print_channel_writes_formatted_tokens | PASS |
| c_semantic_seek_positions_random_record_channel | PASS |
| c_semantic_skipped_dynamic_dim_does_not_evaluate_typed_bound | PASS |
| c_semantic_throw_propagates_through_try_reachable_procedure | PASS |
| c_string_intrinsics_compile_and_run_through_semantic_ir | PASS |
| c_string_user_function_calls_compile_and_run | PASS |
| Every supported fixture transpiles successfully | PASS |
| Built-in scalar methods under FreeBASIC | PASS |
| freebasic_runs_continue_across_every_loop_kind_when_available | PASS |
| MID$ assignment edge cases under FreeBASIC | PASS |
| remline output under FreeBASIC | PASS |
| Self-referential string concatenation under FreeBASIC | PASS |
| freebasic_runs_semantic_mid_assign_array_argument_when_available | PASS |
| freebasic_runs_semantic_mid_assign_byref_operand_when_available | PASS |
| freebasic_runs_semantic_mid_assign_dynamic_target_when_available | PASS |
| Sort driver under FreeBASIC | PASS |
| Standard-library functions under FreeBASIC | PASS |
| gcc_runs_date_dollar_under_c_target_when_available | PASS |
| gcc_runs_inventory_list_all_without_a_garbled_press_any_key_prompt_when_available | PASS |
| gcc_runs_mid_assign_conformance_fixture_under_c_target_when_available | PASS |
| Existing random-file records compile on all targets | PASS |

<nav class="conformance-nav" aria-label="Conformance results navigation">
  <a href="../tutorials/">← Previous: Tutorials</a>
  <a href="../">← Overview</a>
  <a href="../c/">Next: C-specific →</a>
</nav>
