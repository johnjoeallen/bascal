# Conformance tests

## Core language

| Test description | BASIC | C | JVM |
| --- | :---: | :---: | :---: |
| All tutorial .bcl sources compile | PASS | PASS | DEFERRED |
| Built-in scalar method calls | PASS | PASS | PASS |
| C/JVM reject the classic labels/error-handling tutorial | NOT APPLICABLE | PASS | PASS |
| Deterministic tutorials build and execute | PASS | PASS | DEFERRED |
| Interactive inventory case study | PASS | PASS | DEFERRED |
| Standard-library function execution | PASS | PASS | PASS |
| Typed non-integer arrays and array parameters | DEFERRED | DEFERRED | PASS |
| BASCOM creates file; target validates binary compatibility | NOT APPLICABLE | PASS | NOT APPLICABLE |
| Target creates file; BASCOM validates binary compatibility | NOT APPLICABLE | PASS | NOT APPLICABLE |
| Every supported fixture transpiles successfully | PASS | PASS | PASS |
| Adventure game compiles through the front end without backend code generation | UNIMPLEMENTED | UNIMPLEMENTED | UNIMPLEMENTED |
| Array of records is not yet supported | PASS | PASS | PASS |
| Variable-length string records are rejected as random-access file types | PASS | PASS | PASS |
| In-memory records support variable-length string fields | PASS | PASS | PASS |
| Existing random-file records compile on all targets | PASS | PASS | FAIL |
| jvm_backend_compiles_random_access_file_records | PASS | PASS | PASS |
| Nested record fields are not yet supported | PASS | PASS | PASS |
| Arrays of records are not yet supported | PASS | PASS | PASS |
| Record-valued parameters are not yet supported | PASS | PASS | PASS |
| Record-valued returns are not yet supported | PASS | PASS | PASS |
| standalone_record_supports_init_assign_get_and_member_set | PASS | PASS | PASS |
| adventure_port_method_declarations_still_parse | PASS | PASS | PASS |
| assigning_between_unrelated_record_types_is_rejected | PASS | PASS | PASS |
| builtin_scalar_method_syntax_is_unaffected | PASS | PASS | PASS |
| colon_string_and_colon_dollar_are_equivalent | PASS | PASS | PASS |
| combines_cycle_is_rejected | PASS | PASS | PASS |
| combines_does_not_combine_methods | PASS | PASS | PASS |
| combines_does_not_imply_assignability | PASS | PASS | PASS |
| combines_duplicate_field_between_source_and_local_declaration_is_rejected | PASS | PASS | PASS |
| combines_duplicate_field_between_two_sources_is_rejected | PASS | PASS | PASS |
| combines_duplicate_field_through_transitive_paths_is_rejected | PASS | PASS | PASS |
| combines_inline_method_stays_with_its_own_declaring_record | PASS | PASS | PASS |
| combines_multiple_sources_all_contribute_fields | PASS | PASS | PASS |
| combines_of_an_undeclared_record_is_rejected | PASS | PASS | PASS |
| combines_resolves_transitively_through_multiple_levels | PASS | PASS | PASS |
| combines_runs_identically_on_c_and_jvm_when_available | PASS | PASS | PASS |
| combines_single_source_contributes_its_fields | PASS | PASS | PASS |
| combines_with_its_own_explicitly_declared_method_works | PASS | PASS | PASS |
| every_suffix_shorthand_maps_through_the_existing_type_table | PASS | PASS | PASS |
| external_record_receiver_method_compiles_and_runs | PASS | PASS | PASS |
| external_scalar_receiver_method_compiles | PASS | PASS | PASS |
| inline_and_external_same_signature_is_a_duplicate_definition | PASS | PASS | PASS |
| inline_record_method_compiles_and_runs | PASS | PASS | PASS |
| jvm_lowering_uses_invokestatic_with_explicit_receiver_never_invokevirtual | PASS | PASS | PASS |
| jvm_record_method_runs_when_available | PASS | PASS | PASS |
| legacy_dual_type_bracket_form_is_rejected | PASS | PASS | PASS |
| parser_accepts_bracket_receiver_with_dollar_shorthand_return | PASS | PASS | PASS |
| parser_accepts_bracket_receiver_with_named_return | PASS | PASS | PASS |
| parser_accepts_inline_record_method_with_dollar_shorthand_return | PASS | PASS | PASS |
| record_method_accepts_arguments | PASS | PASS | PASS |
| record_method_mutation_of_self_is_visible_to_the_caller | PASS | PASS | PASS |
| same_named_methods_on_unrelated_records_resolve_by_exact_receiver_type | PASS | PASS | PASS |
| scalar_method_chaining_still_works | PASS | PASS | PASS |
| self_field_access_reads_the_receivers_own_field | PASS | PASS | PASS |
| spec_card_display_example_runs_identically_across_backends | PASS | PASS | PASS |
| suffix_and_declared_return_type_must_agree | PASS | PASS | PASS |
| unknown_receiver_type_is_rejected | PASS | PASS | PASS |
| unrecognized_return_type_name_is_rejected | PASS | PASS | PASS |
| zero_argument_record_method_compiles_and_runs | PASS | PASS | PASS |

<nav class="conformance-nav" aria-label="Conformance results navigation">
  <a href="../">← Overview</a>
  <a href="tutorials/">Next: Tutorials →</a>
</nav>
