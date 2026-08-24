//! Minimal native-C backend.
//!
//! Deliberately narrow: this only understands a top-level `print` of
//! string/numeric literals -- including negation, every arithmetic
//! operator (`+`/`-`/`*`/`/`/`\`/MOD/`^`), every comparison operator
//! (`=`/`<>`/`<`/`<=`/`>`/`>=`), every bitwise/logical operator
//! (`AND`/`OR`/`XOR`/`NOT` -- genuinely bitwise, not short-circuit
//! booleans) of them, and `&&`/`||` (BASCAL's own, *already*
//! short-circuit operators -- unlike bitwise `AND`/`OR`, C's native
//! `&&`/`||` are the direct, correct translation here) -- `end`, `dim`,
//! `const`, assignment/reading of *scalar* variables (numeric: `%`/`&`/
//! `!`/`#`; string: `$`, fixed-size `char[256]` buffers written only via
//! `snprintf`, never `strcpy`/`strcat` -- see
//! `STRING_BUFFER_SIZE`/`render_string_expr`); `dim`-declared arrays too
//! (a real, native multi-dimensional C array -- see `ArrayInfo`/
//! `collect_array_declarations` -- but only with a literal or top-level-
//! `const`-literal bound in every dimension, since a real C array needs a
//! compile-time-known size), indexed reads/writes, and `sizeof(arr%)`/
//! `sizeof(grid%, axis)`; `swap` of two scalars or array elements (see
//! `render_lvalue`); `+` string concatenation,
//! `if`/`elseif`/`else`/`end if` (including the single-line form, and
//! nesting), `for`/`next`, `while`/`wend`, every `do`/`loop` pre-/
//! post-check variant, `exit` (maps to a plain C `break;` -- C's native
//! loops already give it the right "innermost enclosing loop" target for
//! free), `select case` (single-value, `to` range, and `is <op>` clauses
//! on a numeric selector; exact-match-only on a string selector, via
//! `strcmp` -- see `emit_select_case`), `function` declarations with
//! byval scalar parameters (numeric and string), `return`, local
//! variables (real C function-local scope -- no name-mangling needed,
//! unlike the BASIC backend's GOSUB-against-shared-globals approach), and
//! `global` to opt into reading/writing a top-level variable instead (see
//! `build_function_table`/`emit_function_def`); `procedure` declarations
//! too (a real `void` C function reusing the same machinery as `function`
//! -- its body may fall through with no explicit `return`, matching real
//! BASIC's implicit `RETURN` for `PROCEDURE`), and a bare call statement
//! (calling a `procedure`, or discarding a `function`'s return value); a
//! suffixless (default-typed) numeric variable (real MBASIC/BASCOM's own
//! unoverridden default, single-precision -- see `effective_suffix`);
//! `require`/`import` cross-file resolution (needed no C-backend-specific
//! work at all -- `lib.rs`'s own resolution already merges a required
//! file's functions/procedures into `Program.functions` before either
//! backend's codegen ever runs); `label:`/`goto label` (a direct 1:1
//! mapping onto C's own `goto`/label -- works inside a `function`/
//! `procedure` body too, not just at top level) and top-level-only BASIC
//! `gosub label`/bare `return` (distinct from a `function`/`procedure`'s
//! own `return` -- built on a small return-address-ID stack, since C's
//! `goto` has no "remember where to resume" of its own; see
//! `Statement::Gosub`'s own doc comment for why it's scoped to top-level
//! code); `on error goto label`/`on error goto 0`, `resume`/`resume
//! next`/`resume label`, `error code`, and bare `err`/`erl` (see
//! `ErrorDataCtx`/`emit_raise_block` -- the same return-address-ID-stack
//! idea as `gosub`/`return`, just read in the opposite direction: a raise
//! site's ID is written when it fires and read back later by whichever
//! `resume` handles it, rather than a `gosub`'s ID being written at the
//! call and read immediately by its own `return`; also top-level-code-only
//! for the identical reason. A label handler target only -- a `procedure`
//! target, which real BASIC also allows, isn't supported yet. A failed
//! sequential `open ... for input` now raises real BASIC's own error 53
//! this way too, instead of silently leaving a `NULL` `FILE*` behind (see
//! `Statement::Open`'s `OpenMode::Input` arm). `erl` reads a stable
//! per-raise-site ID, not a real BASIC line number -- this backend
//! doesn't track BASIC line numbers at all, a real, documented
//! divergence); `data`/`read`/`restore` (see
//! `collect_data_items_and_labels`/`DATA_HELPER` -- scalar targets only,
//! no array reads; a `data` item must be a literal number or string;
//! `restore label` resolves to a fixed item-count position at compile
//! time, no runtime lookup needed -- works inside a `function`/
//! `procedure` body too, unlike the error-handling trio, since
//! `bcc_data`/`bcc_data_ptr` are plain file-scope globals reachable from
//! anywhere); twenty-five BASIC intrinsics implemented natively -- `LEN`,
//! `ASC`, `CHR$`, `MID$`, `LEFT$`, `RIGHT$`, `STR$`, `VAL`, `INSTR`, `SQR`,
//! `ABS`, `INT`, `FIX`, `SGN`, `CINT`, `CLNG`, `CSNG`, `CDBL`, `SIN`,
//! `COS`, `TAN`, `ATN`, `LOG`, `EXP`, `RND` (see
//! `render_numeric_call`/`render_string_call`/`MID_HELPER`/
//! `INSTR_HELPER`/`SGN_HELPER`/`RND_HELPER`) -- plus the statement form
//! `RANDOMIZE` (see `Statement::Randomize`'s own handling in
//! `emit_statement`); random-access record I/O: `OPEN ... FOR
//! RANDOM`/`BINARY`, `CLOSE`, `FIELD`, `GET`/`PUT` (whole-record form
//! only), `LSET`/`RSET`, and `MKI$`/`MKL$`/`MKS$`/`MKD$`/`CVI`/`CVL`/
//! `CVS`/`CVD` (see
//! `FileIoLayout`/`apply_field_statement`/`emit_get_or_put`/`FILE_IO_HELPER`
//! -- two real, documented divergences from real MBASIC/BASCOM live
//! there: `MKS$`/`MKD$`/`CVS`/`CVD` use plain IEEE 754 instead of real
//! BASIC's Microsoft Binary Format, and multi-byte values are packed in
//! the host's native byte order, assumed little-endian); sequential file
//! I/O: `OPEN ... FOR INPUT`/`OUTPUT`/`APPEND`, `CLOSE`, `PRINT #`,
//! `WRITE #`/`INPUT #` (a matched quoted, comma-separated format each can
//! read back), `LINE INPUT #`, and `EOF(#ch)` (see `SEQ_FILE_HELPER`);
//! interactive `INPUT` (one bare scalar variable per statement, with an
//! optional prompt -- see `INPUT_HELPER`); and screen I/O: `cls`,
//! `locate row, col`, `color fg[, bg]`, and `beep` (see `COLOR_HELPER`).
//! NOT yet supported: array parameters (`byref` or `byval`) -- passing a
//! whole array into a `function`/`procedure` is a separate wall from
//! declaring/indexing one at all, which *is* supported (see above); a
//! `dim` array bound that isn't a literal or a top-level `const` with an
//! integer-literal value (see `resolve_array_bound_literal` -- a real C
//! array needs a compile-time-known size, unlike real BASIC's own `dim
//! arr%(n%)` for a runtime-computed `n%`); `byref` scalar parameters
//! (byval-only so far); a `function` body that doesn't provably `return`
//! on every path (see `body_always_returns` -- a `procedure` has no such
//! requirement); a `procedure` as an `on error goto` target (a label
//! target is supported -- see above); a `FIELD`/`OPEN`/`GET`/`PUT`
//! channel or `FIELD` width that isn't a literal integer; and
//! `gosub`/`on error goto`/`resume`/`error` used inside a `function`/
//! `procedure` body (`label`/`goto`/`read`/`restore` all work there fine
//! -- only the return-address-ID-stack techniques are scoped to top-level
//! code, since a `return` inside a function/procedure body always means
//! that callable's own return, leaving no unambiguous "this GOSUB's/raise
//! site's own RETURN/RESUME" to dispatch to) -- all rejected with a
//! diagnostic rather than guessed at. Recursion (direct or indirect) is
//! rejected at the resolver level before codegen ever runs, for every
//! target, not just this one. Everything else (`on ... goto`/`on ...
//! gosub`, `mid$` statement-form assignment, `poke`/`out`, `print using`,
//! ...) reports a "not supported yet" diagnostic rather than panicking or
//! emitting wrong code -- this is a deliberately minimal backend, not a
//! complete one; see the GitHub issue tracker's `c-target` label for the
//! current, itemized list. Tutorials that compile end to end today:
//! `tutorial/01_hello.bcl`, `tutorial/02_variables.bcl`,
//! `tutorial/03_arithmetic.bcl`, `tutorial/04_conditions.bcl`,
//! `tutorial/05_loops.bcl`, `tutorial/06_select_case.bcl`,
//! `tutorial/07_functions.bcl` (including its two `require`d
//! `com.bascal.stdlib` library functions, `ucase$`/`lcase$`),
//! `tutorial/09_data.bcl`, `tutorial/10_files.bcl`, `tutorial/11_screen.bcl`,
//! `tutorial/13_shared/start.bcl` + `tutorial/13_shared/show.bcl`,
//! `tutorial/15_random_and_record_files.bcl` (both its hand-written Part 1
//! and DSL-based Part 2), `tutorial/16_short_circuit.bcl`,
//! `tutorial/17_labels_and_error_handling.bcl`, and `tutorial/18_stdlib.bcl`
//! -- each gcc-compiled and run, not just transpiled (see
//! `docs/manual/command-line-reference.html#backends` for the up to date
//! list). `tutorial/08_arrays.bcl`, `12_require.bcl` (its required library
//! takes an array parameter), `14_procedures.bcl` (byref array params), and
//! `19_inventory.bcl` still don't, blocked by
//! the gaps listed above.
//!
//! Numeric `print` output is plain `%d`/`%g` `printf` formatting -- it does
//! not reproduce real MBASIC/BASCOM's own numeric `PRINT` convention (a
//! leading space standing in for a sign, a trailing space after the number).
//! Matching that exactly is future work, not attempted here.
//!
//! When this grows beyond `print`/`end`: record layout must NOT be expressed
//! as a plain C `struct` -- alignment padding would break binary
//! compatibility with the packed, no-padding layout `FIELD`/`GET`/`PUT` use
//! on the BASIC side. Every record field needs to be (de)serialized
//! explicitly at the byte offsets `records.rs` already computes for the
//! BASIC backend, the same offsets both backends should share.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::ast::{
    BasicIdent, BinaryOp, CaseClause, CaseValue, DoCondition, Expr, FunctionDef, OpenMode,
    ParamMode, PrintToken, Program, RecordFieldType, ResumeTarget, Statement, Stmt, TypeSuffix,
    UnaryOp,
};
use crate::diagnostics::{Diagnostic, SourcePos};

/// One user-defined function's C-callable shape, built by
/// `build_function_table` before any codegen runs -- looked up by
/// `Expr::Call` sites (in `render_numeric_expr`/`render_string_expr`) via
/// `fn_key`, same (name, suffix) keying `codegen_basic::same_ident` uses,
/// since a call site's own suffix is part of its identifier syntax and
/// must match the declaration's exactly.
#[derive(Clone)]
struct FnSig {
    /// The real C function's name (see `function_c_name`) -- distinct from
    /// `c_var_name`'s `bv_...` namespace so a function and a variable that
    /// happen to share a BASIC name can never collide as C identifiers.
    c_name: String,
    /// A `procedure` -- no return value at all, a real `void` C function.
    /// `is_string`/`is_float` are meaningless when this is set.
    is_void: bool,
    is_string: bool,
    /// Only meaningful when `!is_string && !is_void`.
    is_float: bool,
    params: Vec<FnParam>,
    result_suffix: Option<TypeSuffix>,
}

#[derive(Clone)]
struct FnParam {
    /// The C identifier used for this parameter *inside the function
    /// body* -- for a byval string parameter, that's the local buffer
    /// holding the function's own byval copy, NOT the raw incoming pointer
    /// parameter (see `emit_function_def`'s copy-in preamble); same story
    /// for a byref string/array parameter's own local (see `by_ref`'s doc
    /// comment).
    c_name: String,
    is_string: bool,
    /// Only meaningful when `!is_string`.
    is_float: bool,
    default: Option<Expr>,
    suffix: TypeSuffix,
    /// `byref` (real BASCAL, not this backend's own pointer plumbing) --
    /// compiles to a real C pointer parameter named `<c_name>_in`, plus
    /// copy-in/copy-out around it: a scalar gets a local `<c_name>` copied
    /// in from `*<c_name>_in` at function entry and written back to
    /// `*<c_name>_in` at every `return` (see `emit_byref_scalar_copyback`);
    /// an array needs no such round trip at all -- a real pointer already
    /// gives the callee live read/write access to the caller's own
    /// storage, so `array`'s own by-ref case just uses the incoming
    /// pointer directly, under the plain, un-suffixed `<c_name>` (see
    /// `emit_function_def`/`function_signature`). Meaningless combined
    /// with a `byval` array parameter, which is always copied in
    /// regardless of this flag's *absence* there -- `array` doesn't carry
    /// its own separate by-ref flag since this one already means the same
    /// thing for both a scalar and an array parameter.
    by_ref: bool,
    /// `Some` for an array parameter (`arr%(?)`/`arr%(100)`) -- `None` for
    /// an ordinary scalar. Only rank-1 numeric arrays are supported (see
    /// `build_function_table`); every array parameter also gets a hidden
    /// second C parameter, `<c_name>_len0`, carrying its real element
    /// count at that call site (see `function_signature`) -- `sizeof(...)`
    /// inside the function body reads it back (see `ArrayInfo::runtime_len`
    /// and the per-function-extended `FunctionTable` `emit_function_def`
    /// builds for exactly this purpose).
    array: Option<ArrayParamInfo>,
}

/// An array parameter's own extra bookkeeping -- see `FnParam::array`'s
/// doc comment for the byref/byval split.
#[derive(Clone)]
struct ArrayParamInfo {
    /// The parameter's own explicit literal capacity (`arr%(100)` -- one
    /// more than the declared bound, matching `ArrayInfo::bounds`' own
    /// inclusive-bound convention), or `None` for an inferred `arr%(?)`
    /// one -- see `apply_byval_array_capacities`, which fills in
    /// `byval_capacity` from this (or, when this is `None`, from the
    /// largest call site it can resolve).
    declared_capacity: Option<i64>,
    /// The fixed size of the local C array a `byval` array parameter's
    /// own copy-in buffer is declared with (see `emit_function_def`) --
    /// `0` (never read) for a `byref` array parameter, which uses the
    /// caller's own storage directly instead of copying into a local
    /// buffer at all. Filled in by `apply_byval_array_capacities`, once,
    /// right after `build_function_table` runs (which can only see a
    /// parameter's own declared axis, not the call sites that decide an
    /// inferred `?` axis's real capacity).
    byval_capacity: i64,
}

type FunctionMap = HashMap<(String, Option<TypeSuffix>), FnSig>;

/// A `dim`-declared array's compile-time-known shape: `bounds` holds one
/// *inclusive* bound per axis, in declaration order (real BASIC's own
/// `dim arr%(N)` convention -- `N+1` elements, indexed `0..=N`; see
/// `docs/manual/variables-and-constants.html#option-base`), so a given
/// axis's element count is always `bounds[axis] + 1`. `is_string` picks
/// `char[.][STRING_BUFFER_SIZE]` vs. a numeric element type at declaration
/// time (see `collect_array_declarations`'s own call site in `generate`).
/// Scoped deliberately narrow, matching this backend's usual style: only
/// top-level arrays with a literal-or-const-literal size in every
/// dimension are tracked at all -- see `resolve_array_bound_literal`.
#[derive(Clone)]
struct ArrayInfo {
    bounds: Vec<i64>,
    /// `None` for a string array (`char[.][STRING_BUFFER_SIZE]` elements);
    /// `Some((c_type, is_float))` for a numeric one -- the same pair
    /// `numeric_c_type` gives that scalar suffix.
    element_type: Option<(&'static str, bool)>,
    /// `None` for a genuine top-level `dim`'d array -- `bounds` really is a
    /// compile-time constant there, so `sizeof(...)` just reads it
    /// directly (see `render_numeric_call`). `Some` only for a synthetic
    /// `ArrayInfo` `emit_function_def`/the call-emission sites register
    /// (per function, see `FunctionTable`'s own doc comment on the
    /// per-function-extended table) standing in for one of *that
    /// function's own* array parameters: the real element count along
    /// each axis isn't a compile-time constant at all there (the same
    /// function body is reused by every call site, which can each pass a
    /// differently-sized array), so this instead names the hidden
    /// `<c_name>_len0`-style runtime C parameter carrying it -- one
    /// string per axis, same order as `bounds` (which, in this case,
    /// holds only a dummy per-axis placeholder value; only their `len()`
    /// -- the rank -- is real).
    runtime_len: Option<Vec<String>>,
}

type ArrayTable = HashMap<String, ArrayInfo>;

/// Everything `Expr::Call`/`Expr::ArrayRef` resolution needs to know about
/// names declared elsewhere in the program: `funcs` is the original
/// per-BASCAL-function signature table (see `build_function_table`),
/// `arrays` is `dim`-declared arrays (see `collect_array_declarations`).
/// Bundled into one struct, rather than a second parameter threaded
/// through every `render_numeric_expr`/`render_string_expr`/... call site
/// (dozens of them, most several calls deep), since every caller that
/// already has a `functions: &FunctionTable` in scope needs `arrays` at
/// exactly the same points it needs `funcs` -- `Expr::ArrayRef`'s own
/// parse-time ambiguity (see `is_known_callable`'s doc comment) means a
/// single identifier lookup has to check both tables together anyway.
struct FunctionTable {
    funcs: FunctionMap,
    methods: HashMap<(TypeSuffix, String), FnSig>,
    arrays: ArrayTable,
}

impl FunctionTable {
    fn get(&self, key: &(String, Option<TypeSuffix>)) -> Option<&FnSig> {
        self.funcs.get(key)
    }

    fn contains_key(&self, key: &(String, Option<TypeSuffix>)) -> bool {
        self.funcs.contains_key(key)
    }

    fn method(&self, receiver: TypeSuffix, name: &str) -> Option<&FnSig> {
        self.methods.get(&(receiver, name.to_ascii_lowercase()))
    }

    fn signature_for(&self, func: &FunctionDef) -> Option<&FnSig> {
        match func.receiver {
            Some(receiver) => self.method(receiver, &func.name.name),
            None => self.get(&fn_key(&func.name)),
        }
    }
}

impl std::ops::Index<&(String, Option<TypeSuffix>)> for FunctionTable {
    type Output = FnSig;

    fn index(&self, key: &(String, Option<TypeSuffix>)) -> &FnSig {
        &self.funcs[key]
    }
}

/// Builds the *per-function* extended `FunctionTable` a function/procedure
/// with its own array parameter and/or local (non-top-level) `dim`'d
/// array needs while its own body is being emitted (see `generate`'s own
/// call site) -- `None` when `sig` has no array parameter and `func.body`
/// has no local array `dim` either, the common case, so the caller can
/// skip the clone entirely.
///
/// The extension is one synthetic `ArrayInfo` per array parameter, plus
/// `local_arrays` (every local `dim name(...)` array `collect_array_declarations`
/// finds in this function's own body -- see `generate`'s call site), all
/// keyed the same way a real top-level `dim`'d array already is (its own
/// `c_name`) -- once inserted, every existing array-aware lookup
/// (`render_lvalue`, `render_numeric_expr`/`render_string_expr`'s
/// `Expr::ArrayRef`/`Expr::Call` arms, `render_array_index_expr`,
/// `sizeof(...)` in `render_numeric_call`) treats a reference to the
/// parameter's or local array's own name exactly like a reference to a
/// real top-level array, with no changes to any of them: reading/writing
/// an indexed element already just needs to know the element's C type and
/// the array's rank, both given here (directly from `local_arrays` for a
/// local array -- it already has a real compile-time bound, same as a
/// top-level one), and `sizeof(...)` already just needs *some* string to
/// substitute for the element count, which `ArrayInfo::runtime_len` gives
/// as the hidden `<c_name>_len0` parameter `function_signature` declares
/// for every array parameter (see its own doc comment) -- correct for
/// `byval` (the local copy always holds exactly that many real elements)
/// and `byref` alike (the caller's own real array always does too). A
/// local array's own `runtime_len` is always `None`, same as a top-level
/// one -- its bound is a compile-time literal either way.
///
/// A same-named collision between a local array and a top-level one or an
/// array parameter is resolved by `extend`'s own last-write-wins
/// semantics (the local array's own entry, inserted last, wins) --
/// harmless in practice: the two live in genuinely separate C scopes (a
/// real top-level `static` array vs. this function's own true C local),
/// so shadowing the lookup for the duration of this function's own body
/// is exactly the right BASIC scoping behavior anyway.
///
/// Cloning `functions.funcs` here (rather than threading a second,
/// `arrays`-only table through every call site that already takes
/// `functions: &FunctionTable`) keeps this scoped to a single call site --
/// see `FnParam`'s own doc comment for the fuller design rationale.
fn function_scoped_table(
    functions: &FunctionTable,
    sig: &FnSig,
    local_arrays: &ArrayTable,
) -> Option<FunctionTable> {
    if !sig.params.iter().any(|p| p.array.is_some()) && local_arrays.is_empty() {
        return None;
    }
    let mut arrays = functions.arrays.clone();
    for param in &sig.params {
        if param.array.is_none() {
            continue;
        }
        arrays.insert(
            param.c_name.clone(),
            ArrayInfo {
                bounds: vec![0],
                element_type: Some(("int", param.is_float)),
                runtime_len: Some(vec![format!("{}_len0", param.c_name)]),
            },
        );
    }
    arrays.extend(local_arrays.iter().map(|(k, v)| (k.clone(), v.clone())));
    Some(FunctionTable {
        funcs: functions.funcs.clone(),
        methods: functions.methods.clone(),
        arrays,
    })
}

/// Renders one user-function/procedure call's whole argument list against
/// its own `FnParam` signature -- shared by the three call-emission sites
/// (`render_numeric_call`, `render_string_call`, and the bare
/// `Statement::ExprStmt` call statement), each of which used to inline an
/// almost-identical loop before every one of them also needed to handle
/// `byref`/array parameters.
///
/// A plain `byval` scalar parameter is unchanged from before: the argument
/// expression is rendered normally (any prelude it needs is collected and
/// returned). A `byref` scalar parameter's argument must be a bare
/// variable (matching real BASIC's own byref/byval rule, `docs/manual/
/// arrays.html#byref-byval` -- there's nowhere else to write the result
/// back to): its address is passed for a numeric one, or the plain buffer
/// name for a string one (already a `char*`, so no `&` -- taking its
/// address would instead give a `char(*)[256]`, the wrong pointer type
/// entirely). An array parameter's argument (`byval` or `byref` alike)
/// must be a bare reference to a *known* array -- either a real top-level
/// `dim`'d one, or (when this call site itself lives inside another
/// function's body) that function's own array parameter, thanks to the
/// per-function-extended `FunctionTable` `function_scoped_table` builds
/// (see its own doc comment) -- and renders to *two* C arguments: the
/// array's own name (already a real C array, which decays to a pointer
/// exactly like a `byref`/`byval` parameter's own incoming one expects)
/// and its real element count (a compile-time literal for a real `dim`'d
/// array, or the forwarded `<c_name>_len0` hidden parameter for a
/// forwarded array parameter -- see `ArrayInfo::runtime_len`).
fn render_call_args(
    args: &[&Expr],
    params: &[FnParam],
    needs_math: &mut bool,
    temp_counter: &mut usize,
    functions: &FunctionTable,
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut prelude = Vec::new();
    let mut arg_texts = Vec::with_capacity(args.len());
    for (&arg, param) in args.iter().zip(params) {
        if param.array.is_some() {
            let Expr::Ident(ident) = arg else {
                return Err(
                    "an array argument isn't supported by the minimal C backend yet -- only a \
                     bare array name is"
                        .to_string(),
                );
            };
            let key = array_c_name(ident);
            let info = functions.arrays.get(&key).ok_or_else(|| {
                format!("`{ident}` isn't a known array, so it can't be passed as an array argument")
            })?;
            let len_text = info
                .runtime_len
                .as_ref()
                .and_then(|v| v.first())
                .cloned()
                .unwrap_or_else(|| (info.bounds[0] + 1).to_string());
            arg_texts.push(key);
            arg_texts.push(len_text);
        } else if param.by_ref {
            let Expr::Ident(ident) = arg else {
                return Err(
                    "a `byref` parameter was called with an argument that isn't a plain \
                     variable -- byref requires somewhere to write the result back to"
                        .to_string(),
                );
            };
            if param.is_string {
                arg_texts.push(c_var_name(ident, TypeSuffix::String));
            } else {
                let suffix = effective_suffix(ident.suffix);
                arg_texts.push(format!("&{}", c_var_name(ident, suffix)));
            }
        } else if param.is_string {
            let (arg_prelude, text) = render_string_expr(arg, needs_math, temp_counter, functions)?;
            prelude.extend(arg_prelude);
            arg_texts.push(text);
        } else {
            let (text, is_float) = render_numeric_expr(arg, needs_math, functions)?;
            arg_texts.push(coerce_numeric(text, is_float, param.is_float, needs_math));
        }
    }
    Ok((prelude, arg_texts))
}

/// The lookup key for a function table entry, or an `Expr::Call` site --
/// case-insensitive name plus suffix, matching `codegen_basic::same_ident`
/// (a bare `PartialEq`/`Hash` derive on `BasicIdent` would be
/// case-*sensitive*, which is wrong for BASIC identifiers).
fn fn_key(ident: &BasicIdent) -> (String, Option<TypeSuffix>) {
    (ident.name.to_ascii_lowercase(), ident.suffix)
}

/// Whether `name` is *plausibly* a call at all -- a known user-defined
/// function, or one of the handful of BASIC intrinsics this backend
/// implements natively (`LEN`/`ASC`/`CHR$`/`MID$`/`LEFT$` -- see
/// `render_numeric_call`/`render_string_call`). Used only to disambiguate
/// `Expr::ArrayRef` (a single-argument or zero-argument call parses as
/// this, not `Expr::Call` -- see `make_paren_ident_expr` in `parser.rs`)
/// from a genuine, unsupported array access sharing the same shape:
/// deliberately permissive (an unknown builtin name still routes to
/// `render_numeric_call`/`render_string_call`, which reject it with a
/// precise error) rather than trying to enumerate every way a call could
/// be invalid here too.
fn is_known_callable(name: &BasicIdent, functions: &FunctionTable) -> bool {
    functions.contains_key(&fn_key(name))
        || matches!(
            name.name.to_ascii_lowercase().as_str(),
            "len" | "asc" | "chr" | "mid" | "left" | "str" | "cvi" | "cvl" | "cvs" | "cvd"
        )
}

/// The C identifier a BASCAL function maps to. Same `bf_<tag>_<name>`
/// shape as `c_var_name`'s `bv_<tag>_<name>`, but its own prefix -- a
/// function and a variable sharing a BASIC name (legal, since they're
/// different namespaces in BASIC) must never collide as C identifiers.
fn function_c_name(ident: &BasicIdent) -> String {
    let tag = match ident.suffix {
        Some(TypeSuffix::Integer) => 'i',
        Some(TypeSuffix::Long) => 'l',
        Some(TypeSuffix::Single) => 'f',
        Some(TypeSuffix::Double) => 'd',
        Some(TypeSuffix::String) => 's',
        None => 'i',
    };
    format!("bf_{tag}_{}", ident.name.to_ascii_lowercase())
}

/// Which of the small set of BASIC intrinsics this backend implements
/// natively (`LEN`, `ASC`, `CHR$`, `MID$`, `LEFT$` -- see
/// `render_numeric_call`/`render_string_call`) the program actually calls
/// anywhere, computed once, up front, by one AST scan (reusing
/// `codegen_basic::visit_body_exprs`, which already knows how to walk
/// every expression in every statement kind) rather than threading yet
/// another mutable flag through the entire codegen call graph the way
/// `needs_math`/`needs_string` already are: those two are set reactively,
/// exactly where the triggering construct is actually emitted, but this
/// only needs a yes/no answer *before* any code is emitted at all (to
/// decide whether to emit the `bcc_mid`/`bcc_chr` helper functions and the
/// `<string.h>` include), so a simple up-front scan is simpler than
/// plumbing.
struct BuiltinUsage {
    needs_string_h: bool,
    needs_ring_buffer_helpers: bool,
    /// Set by `VAL`, whose C translation (`atof`) is declared in
    /// `<stdlib.h>`. `generate()` already pulls this header in whenever
    /// `file_io.used`, for unrelated reasons -- this flag covers a
    /// program that calls `VAL` without using any random-access I/O.
    needs_stdlib_h: bool,
    /// Set by `INSTR`, whose C translation calls the `bcc_instr` helper
    /// (see `INSTR_HELPER`), which itself needs `strstr` from `<string.h>`.
    needs_instr_helper: bool,
    /// Set by `EOF(...)`, whose C translation calls the `bcc_eof` helper
    /// (see `SEQ_FILE_HELPER`). The rest of sequential file I/O is made of
    /// statement forms, caught by `program_uses_sequential_file_io`
    /// instead -- `EOF` is the one part that's an expression, so it needs
    /// its own flag here, in the same expression-visiting pass as `INSTR`.
    needs_seq_file_helper: bool,
    /// Set by `SGN`, whose C translation calls the `bcc_sgn` helper (see
    /// `SGN_HELPER`) -- `SQR`/`ABS`/`INT`/`FIX` need no helper of their
    /// own, just `<math.h>` (already covered by `needs_math`).
    needs_sgn_helper: bool,
    /// Set by `RND`, whose C translation calls the `bcc_rnd` helper (see
    /// `RND_HELPER`), which itself needs `rand()` from `<stdlib.h>` --
    /// folded into the same `needs_stdlib_h` gate `VAL`'s `atof` already
    /// sets, since both live behind the same include.
    needs_rnd_helper: bool,
    /// Set by bare `INKEY$` (an `Expr::Ident`, not a call -- see
    /// `render_string_expr`'s own `Expr::Ident` arm), whose C translation
    /// calls the `bcc_inkey` helper (see `INKEY_PROTO`/`INKEY_BODY`),
    /// which needs `<termios.h>`/`<unistd.h>` for its own non-blocking
    /// terminal raw-mode read.
    needs_inkey_helper: bool,
}

fn scan_builtin_usage(program: &Program) -> BuiltinUsage {
    let mut usage = BuiltinUsage {
        needs_string_h: false,
        needs_ring_buffer_helpers: false,
        needs_stdlib_h: false,
        needs_instr_helper: false,
        needs_seq_file_helper: false,
        needs_sgn_helper: false,
        needs_rnd_helper: false,
        needs_inkey_helper: false,
    };
    let mut visit = |expr: &Expr| {
        if let Expr::Ident(ident) = expr {
            if ident.suffix == Some(TypeSuffix::String) && ident.name.eq_ignore_ascii_case("inkey")
            {
                usage.needs_inkey_helper = true;
            }
        }
        if let Expr::Call { name, .. } | Expr::ArrayRef { name, .. } = expr {
            match name.name.to_ascii_lowercase().as_str() {
                "len" | "asc" => usage.needs_string_h = true,
                "mid" | "left" | "right" | "chr" | "str" => {
                    usage.needs_string_h = true;
                    usage.needs_ring_buffer_helpers = true;
                }
                "val" => usage.needs_stdlib_h = true,
                "instr" => {
                    usage.needs_string_h = true;
                    usage.needs_instr_helper = true;
                }
                "eof" => usage.needs_seq_file_helper = true,
                "sgn" => usage.needs_sgn_helper = true,
                "rnd" => {
                    usage.needs_stdlib_h = true;
                    usage.needs_rnd_helper = true;
                }
                _ => {}
            }
        }
        // A string comparison (`=`/`<>`/`<`/`<=`/`>`/`>=` where either
        // side is string-typed) compiles to `strcmp` -- see
        // `render_numeric_expr`'s own comparison-operator arm.
        if let Expr::Binary {
            left,
            op:
                BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge,
            right,
        } = expr
        {
            if is_string_expr(left) || is_string_expr(right) {
                usage.needs_string_h = true;
            }
        }
    };
    crate::codegen_basic::visit_body_exprs(&program.statements, &mut visit);
    for func in &program.functions {
        crate::codegen_basic::visit_body_exprs(&func.body, &mut visit);
    }
    usage
}

/// Every `Expr::Call`/`Expr::ArrayRef` site in the whole program -- top
/// level plus every function/procedure body -- as `(callee name, argument
/// list)` pairs, regardless of which of the two shapes parsed it (a
/// zero- or one-argument call always parses as `Expr::ArrayRef`, not
/// `Expr::Call` -- see `is_known_callable`'s own doc comment): used by
/// `apply_byval_array_capacities`, which filters these down to just the
/// ones actually calling a function with a `byval` array parameter.
fn collect_call_sites(program: &Program) -> Vec<(BasicIdent, Vec<Expr>)> {
    let mut sites: Vec<(BasicIdent, Vec<Expr>)> = Vec::new();
    let mut visit = |expr: &Expr| match expr {
        Expr::Call { name, args } => sites.push((name.clone(), args.clone())),
        Expr::ArrayRef { name, indices } => sites.push((name.clone(), indices.clone())),
        _ => {}
    };
    crate::codegen_basic::visit_body_exprs(&program.statements, &mut visit);
    for func in &program.functions {
        crate::codegen_basic::visit_body_exprs(&func.body, &mut visit);
    }
    sites
}

/// Fills in `byval_capacity` on every `byval` array parameter's
/// `ArrayParamInfo` (see its own doc comment) -- the fixed size of the
/// local C array `emit_function_def` copies the caller's array into. Run
/// once, right after `build_function_table`, before any codegen: a
/// `byref` array parameter is skipped entirely (it has no local buffer to
/// size at all -- see `FnParam::by_ref`'s doc comment).
///
/// Deliberately narrow, mirroring `codegen_basic::infer_array_param_capacities`'s
/// own call-site scan but far simpler: only a call site whose argument is
/// a bare reference to a *top-level* `dim`'d array (`arrays`, this
/// backend's only concept of "a compile-time-known array size" -- see the
/// module doc comment) is understood. An explicit capacity
/// (`arr%(100)`) is used as-is (still cross-checked against every
/// resolvable call site, the same compile-time-provable overflow check
/// `codegen_basic` does); an inferred one (`arr%(?)`) needs at least one
/// resolvable call site, and every call site to that parameter must
/// resolve -- an unresolvable one (an expression, or a forwarded array
/// parameter from another function) is rejected outright rather than
/// silently under-sizing the buffer, since there would be no way to prove
/// the buffer is big enough.
fn apply_byval_array_capacities(
    funcs: &mut FunctionMap,
    program: &Program,
    arrays: &ArrayTable,
) -> Result<(), String> {
    let call_sites = collect_call_sites(program);
    let keys: Vec<(String, Option<TypeSuffix>)> = funcs.keys().cloned().collect();
    for key in keys {
        let param_count = funcs[&key].params.len();
        for idx in 0..param_count {
            let (needs_capacity, declared) = match &funcs[&key].params[idx].array {
                Some(arr) if !funcs[&key].params[idx].by_ref => (true, arr.declared_capacity),
                _ => (false, None),
            };
            if !needs_capacity {
                continue;
            }
            let mut max_actual: Option<i64> = None;
            let mut any_site = false;
            let mut any_unresolved = false;
            for (name, args) in &call_sites {
                if fn_key(name) != key {
                    continue;
                }
                let Some(arg) = args.get(idx) else {
                    continue;
                };
                any_site = true;
                match arg {
                    Expr::Ident(arg_ident) => match arrays.get(&array_c_name(arg_ident)) {
                        Some(info) => {
                            let actual = info.bounds[0] + 1;
                            max_actual = Some(max_actual.map_or(actual, |m: i64| m.max(actual)));
                            if let Some(cap) = declared {
                                if actual > cap {
                                    return Err(format!(
                                        "a call to `{name}` passes {actual} elements to its \
                                         array parameter, but its storage is only sized for \
                                         {cap} -- give it a bigger explicit capacity"
                                    ));
                                }
                            }
                        }
                        None => any_unresolved = true,
                    },
                    _ => any_unresolved = true,
                }
            }
            let capacity = match declared {
                Some(cap) => cap,
                None if any_site && !any_unresolved => max_actual.unwrap_or(1),
                None => {
                    return Err(format!(
                        "can't automatically size `{name}`'s storage for its array parameter -- \
                         at least one call site (or none at all) can't be resolved to a \
                         top-level array at compile time. Give it an explicit capacity instead \
                         of `?`, e.g. `(100)`",
                        name = key.0
                    ));
                }
            };
            funcs
                .get_mut(&key)
                .unwrap()
                .params
                .get_mut(idx)
                .unwrap()
                .array
                .as_mut()
                .unwrap()
                .byval_capacity = capacity.max(1);
        }
    }
    Ok(())
}

/// `bcc_mid`/`bcc_chr` -- `MID$`/`LEFT$` and `CHR$` -- both return
/// `const char*` from a small fixed pool of `BCC_STRBUF_COUNT` static
/// buffers, cycling through them round-robin, rather than the
/// out-parameter-plus-caller-supplied-temp convention every *other*
/// string value in this backend uses (`render_string_expr`'s own
/// concatenation/user-function-call cases). That convention needs a
/// prelude -- a temp buffer declared, then a statement writing into it,
/// *before* the line that uses the result -- which `render_numeric_expr`
/// has nowhere to put (it returns a single expression, no prelude
/// mechanism at all): `LEN(MID$(s$, i%, 1))`/`ASC(...)` need to use a
/// nested `MID$`/`CHR$` call as a plain sub-expression, no statement
/// support needed. A self-contained returned pointer sidesteps the whole
/// problem, at the cost of a real, narrow trade-off: the buffer a given
/// call's result lives in gets reused after `BCC_STRBUF_COUNT` further
/// `bcc_mid`/`bcc_chr` calls -- fine for the handful of calls a single
/// expression or statement ordinarily makes, but a result must be
/// consumed (assigned, concatenated, printed) well before that many more
/// such calls happen, or it silently reads back changed. User-defined
/// string-returning functions don't share this pool -- see
/// `function_signature`'s `bcc_out` convention -- so nesting one of
/// *those* inside `LEN`/`ASC` still isn't supported (see
/// `render_prelude_free_string_arg`).
const MID_STATE: &str = "#define BCC_STRBUF_COUNT 8\nstatic char bcc_strbuf[BCC_STRBUF_COUNT][256];\nstatic int bcc_strbuf_next = 0;\n\n";
const MID_PROTOS: &str = "static char* bcc_strbuf_take(void);\nstatic const char* bcc_mid(const char* s, int start, int length);\nstatic const char* bcc_chr(int code);\nstatic const char* bcc_stri(int value);\nstatic const char* bcc_strd(double value);\n";
const MID_BODY: &str = "static char* bcc_strbuf_take(void) {\n    char* buf = bcc_strbuf[bcc_strbuf_next];\n    bcc_strbuf_next = (bcc_strbuf_next + 1) % BCC_STRBUF_COUNT;\n    return buf;\n}\n\nstatic const char* bcc_mid(const char* s, int start, int length) {\n    char* out = bcc_strbuf_take();\n    int len = (int)strlen(s);\n    int from = start - 1;\n    if (from < 0) from = 0;\n    if (from > len) from = len;\n    int avail = len - from;\n    if (length < 0) length = 0;\n    if (length > avail) length = avail;\n    snprintf(out, 256, \"%.*s\", length, s + from);\n    return out;\n}\n\nstatic const char* bcc_chr(int code) {\n    char* out = bcc_strbuf_take();\n    snprintf(out, 256, \"%c\", code);\n    return out;\n}\n\nstatic const char* bcc_stri(int value) {\n    char* out = bcc_strbuf_take();\n    snprintf(out, 256, \"% d\", value);\n    return out;\n}\n\nstatic const char* bcc_strd(double value) {\n    char* out = bcc_strbuf_take();\n    snprintf(out, 256, \"% g\", value);\n    return out;\n}\n\n";

/// Random-access record I/O runtime support: `bcc_files` is a fixed-size
/// table of open `FILE*` handles, sized `[BCC_MAX_CHANNELS]` and indexed
/// by `channel - 1` everywhere a channel is used -- BASIC channels are
/// 1-based (`#1`, never `#0`), but the C array is 0-based, so every
/// `bcc_files[...]` reference in emitted code subtracts 1 from the
/// BASCAL channel number first (done once, in Rust, since the channel is
/// always a compile-time literal -- see `literal_channel`). `FileIoLayout`'s
/// `channel_fields` map, by contrast, is still keyed by the raw 1-based
/// channel number -- it's a `HashMap`, not an array, so there's no
/// out-of-bounds concern to avoid by shifting its keys too.
/// `literal_channel` rejects any channel outside `1..=BCC_MAX_CHANNELS`
/// before it ever reaches here, so nothing at this level needs its own
/// bounds check. `bcc_mki`/`bcc_mkl`/
/// `bcc_mks`/`bcc_mkd` and `bcc_cvi`/`bcc_cvl`/`bcc_cvs`/`bcc_cvd` are
/// `MKI$`/`MKL$`/`MKS$`/`MKD$`/`CVI`/`CVL`/`CVS`/`CVD` -- raw
/// fixed-width-integer/float byte packing and unpacking via `memcpy`,
/// `<stdint.h>`'s `int16_t`/`int32_t` guaranteeing the exact widths real
/// MBASIC/BASCOM's own `MKI$`/`MKL$` use regardless of platform `int`
/// width. Two real, deliberate divergences from real MBASIC/BASCOM,
/// both documented rather than silently wrong: `MKS$`/`MKD$`/`CVS`/`CVD`
/// use plain IEEE 754 `float`/`double` (a `memcpy` of C's own
/// representation) instead of real BASIC's Microsoft Binary Format --
/// implementing byte-for-byte MBF conversion is real, narrow bit-twiddling
/// work not attempted here, so a `float64` record field written by this
/// backend is *not* binary-compatible with one written by real BASCOM
/// (an `int16`/`int32`/`string(N)` field still is, since those don't
/// involve MBF at all); and every multi-byte value is packed/unpacked in
/// the host's native byte order, assumed little-endian (true of every
/// realistic `--target c` deployment platform today -- x86/x86-64/ARM --
/// not big-endian mainframes, matching real BASIC's own on-disk
/// little-endian layout only on those platforms).
const FILE_IO_STATE: &str = "#define BCC_MAX_CHANNELS 32\nstatic FILE* bcc_files[BCC_MAX_CHANNELS];\n\n";
const FILE_IO_PROTOS: &str = "static void bcc_read_string_field(char* field, const unsigned char* source, size_t width);\nstatic void bcc_mki(char* out, int value);\nstatic void bcc_mkl(char* out, int value);\nstatic void bcc_mks(char* out, double value);\nstatic void bcc_mkd(char* out, double value);\nstatic int bcc_cvi(const char* s);\nstatic int bcc_cvl(const char* s);\nstatic float bcc_cvs(const char* s);\nstatic double bcc_cvd(const char* s);\nstatic int bcc_read_record(FILE* file, void* buffer, size_t reclen, long record);\nstatic void bcc_write_record(FILE* file, const void* buffer, size_t reclen, long record);\nstatic void bcc_pad_string_field(unsigned char* dest, const char* value, size_t width);\n";
const FILE_IO_BODY: &str = "static void bcc_read_string_field(char* field, const unsigned char* source, size_t width) {\n    memcpy(field, source, width);\n    field[width] = 0;\n    while (width > 0 && field[width - 1] == ' ') field[--width] = 0;\n}\n\nstatic void bcc_mki(char* out, int value) {\n    int16_t v = (int16_t)value;\n    memcpy(out, &v, 2);\n}\n\nstatic void bcc_mkl(char* out, int value) {\n    int32_t v = (int32_t)value;\n    memcpy(out, &v, 4);\n}\n\nstatic void bcc_mks(char* out, double value) {\n    float v = (float)value;\n    memcpy(out, &v, 4);\n}\n\nstatic void bcc_mkd(char* out, double value) {\n    memcpy(out, &value, 8);\n}\n\nstatic int bcc_cvi(const char* s) {\n    int16_t v;\n    memcpy(&v, s, 2);\n    return (int)v;\n}\n\nstatic int bcc_cvl(const char* s) {\n    int32_t v;\n    memcpy(&v, s, 4);\n    return (int)v;\n}\n\nstatic float bcc_cvs(const char* s) {\n    float v;\n    memcpy(&v, s, 4);\n    return v;\n}\n\nstatic double bcc_cvd(const char* s) {\n    double v;\n    memcpy(&v, s, 8);\n    return v;\n}\n\nstatic int bcc_read_record(FILE* file, void* buffer, size_t reclen, long record) {\n    if (fseek(file, (record - 1) * (long)reclen, SEEK_SET) != 0) return 0;\n    return fread(buffer, 1, reclen, file) == reclen;\n}\n\nstatic void bcc_write_record(FILE* file, const void* buffer, size_t reclen, long record) {\n    fseek(file, (record - 1) * (long)reclen, SEEK_SET);\n    fwrite(buffer, 1, reclen, file);\n}\n\nstatic void bcc_pad_string_field(unsigned char* dest, const char* value, size_t width) {\n    size_t len = strlen(value);\n    if (len > width) len = width;\n    memcpy(dest, value, len);\n    memset(dest + len, ' ', width - len);\n}\n\n";

/// `COLOR fg[, bg]`'s runtime helper -- real BASCOM's classic CGA palette
/// (0-15 foreground, 0-7 background) has no single portable C equivalent,
/// so this backend targets plain ANSI SGR escape sequences instead (widely
/// supported on POSIX terminals and modern Windows terminals -- not a
/// platform-specific console API). ANSI's own base 8 colors are in a
/// *different order* than CGA's (ANSI red=1/blue=4, CGA blue=1/red=4), and
/// CGA's bright colors (8-15) are ANSI's "aixterm" bright range (90-97),
/// not the base range with bold added -- `bcc_ansi_fg`/`bcc_ansi_bg` are
/// direct CGA-index -> real-ANSI-code lookup tables encoding both of
/// those, rather than a formula. `bcc_color`'s `bg` of `-1` means "COLOR
/// fg" alone was given -- leave the background alone, matching real
/// BASCOM's own COLOR (an omitted argument doesn't reset it).
///
/// `COLOR` is the one screen-appearance change this backend makes that
/// outlives the program itself if left alone: ANSI SGR color codes stay
/// in effect in the *terminal*, not just this process, so a program that
/// never explicitly resets them (real BASCOM programs rarely do -- DOS's
/// own console reset itself on the next COMMAND.COM prompt) would leave
/// the user's shell colored after exit. `bcc_color_reset` (`\x1b[0m`,
/// ANSI's own "all attributes off") is registered via `atexit` the first
/// time `COLOR` is actually used, guarded by `bcc_color_used` so it's
/// only registered once -- this way it fires on every exit path alike
/// (falling off the end of `main`, `STOP`/`SYSTEM`'s `exit(0)`, an
/// uncaught error's own `exit(1)`) with no need to duplicate a reset call
/// at each one individually.
const COLOR_PROTO: &str = "static void bcc_color(int fg, int bg);\n";
const COLOR_BODY: &str = "static const int bcc_ansi_fg[16] = {30, 34, 32, 36, 31, 35, 33, 37, 90, 94, 92, 96, 91, 95, 93, 97};\nstatic const int bcc_ansi_bg[8] = {40, 44, 42, 46, 41, 45, 43, 47};\nstatic int bcc_color_used = 0;\n\nstatic void bcc_color_reset(void) {\n    printf(\"\\x1b[0m\");\n}\n\nstatic void bcc_color(int fg, int bg) {\n    if (!bcc_color_used) {\n        atexit(bcc_color_reset);\n        bcc_color_used = 1;\n    }\n    printf(\"\\x1b[%dm\", bcc_ansi_fg[fg & 15]);\n    if (bg >= 0) {\n        printf(\"\\x1b[%dm\", bcc_ansi_bg[bg & 7]);\n    }\n}\n\n";

/// `input [prompt$;] var` -- reads one whole line into a shared,
/// fixed-size scratch buffer (matching every string in this backend
/// already being a fixed `char[STRING_BUFFER_SIZE]`), stripping the
/// trailing newline `fgets` leaves in. Every `INPUT` in the program reuses
/// this same buffer -- safe because each `Statement::Input` fully consumes
/// it (parses it into the target variable) before the next one runs; there
/// is never a live reference to a stale read left lying around.
const INPUT_STATE: &str = "static char bcc_input_buf[256];\n\n";
const INPUT_PROTO: &str = "static void bcc_read_line(void);\n";
const INPUT_BODY: &str = "static void bcc_read_line(void) {\n    if (fgets(bcc_input_buf, sizeof(bcc_input_buf), stdin) == NULL) {\n        bcc_input_buf[0] = 0;\n        return;\n    }\n    bcc_input_buf[strcspn(bcc_input_buf, \"\\r\\n\")] = 0;\n}\n\n";

/// `INSTR(s$, needle$)` -- the 1-based position of the first match, or 0.
/// Scoped to this 2-argument form only, matching what `docs/language/
/// arrays-and-strings.html` documents -- real BASCOM's optional leading
/// `start%` argument (`INSTR(start%, s$, needle$)`) isn't implemented,
/// since nothing in this repo exercises it. `strstr` already does the
/// actual search; this just converts its pointer result to BASIC's
/// 1-based index convention (or 0 for "not found", instead of C's `NULL`).
const INSTR_PROTO: &str = "static int bcc_instr(const char* s, const char* needle);\n";
const INSTR_BODY: &str = "static int bcc_instr(const char* s, const char* needle) {\n    const char* found = strstr(s, needle);\n    return found ? (int)(found - s) + 1 : 0;\n}\n\n";

/// `SGN(x)` -- -1/0/1 by the sign of `x`. No single C library function
/// does this (unlike `SQR`/`ABS`/`INT`/`FIX`, which map straight onto
/// `sqrt`/`fabs`/`floor`/`trunc`), so it gets a small helper of its own.
const SGN_PROTO: &str = "static int bcc_sgn(double v);\n";
const SGN_BODY: &str = "static int bcc_sgn(double v) {\n    if (v > 0) return 1;\n    if (v < 0) return -1;\n    return 0;\n}\n\n";

/// `RND(x)` -- real BASIC's argument-selects-behavior convention: `x < 0`
/// reseeds the sequence from `x` and returns the first draw of that new,
/// deterministic sequence (so the same negative `x` always reproduces the
/// same value); `x == 0` repeats the value `RND` last returned, drawing
/// nothing new; `x > 0` (including the omitted/no-arg call, which
/// `render_numeric_call` passes through as a literal `1.0`) draws and
/// returns the next value in the sequence. Built on C's own `rand()`, not
/// a from-scratch PRNG -- a real, documented divergence from real
/// MBASIC/BASCOM, same category as `MKS$`/`CVS`'s IEEE-754-vs-MBF gap
/// (see `FILE_IO_HELPER`'s own doc comment): the exact sequence of values
/// `RND` produces here will never match real BASCOM's, only the
/// documented argument semantics above. `RANDOMIZE` (see
/// `Statement::Randomize`'s own handling in `emit_statement`) reseeds the
/// same underlying `rand()` stream via `srand()`.
const RND_PROTO: &str = "static double bcc_rnd(double x);\n";
const RND_BODY: &str = "static double bcc_rnd_last = 0.0;\n\nstatic double bcc_rnd(double x) {\n    if (x < 0) {\n        srand((unsigned int)(-x));\n    }\n    if (x != 0) {\n        bcc_rnd_last = (double)rand() / ((double)RAND_MAX + 1.0);\n    }\n    return bcc_rnd_last;\n}\n\n";

/// `INKEY$` -- real BASIC's own non-blocking single-keypress read: return
/// the next waiting key as a one-character string, or `""` immediately if
/// none is waiting, never blocking either way (that's what the `do ...
/// loop until k$ <> ""` polling idiom every BASCAL tutorial/example using
/// it relies on assumes). Plain C has no such primitive at all -- POSIX's
/// own is a raw terminal mode plus a non-blocking `read()`, toggled *in
/// and back out* around each individual call: `tcgetattr`/`tcsetattr`
/// into `ICANON`/`ECHO`-off "raw" mode with `VMIN=0`/`VTIME=0` (`read`
/// returns immediately with whatever's available, zero bytes if nothing
/// is), the one `read()`, then `tcsetattr` straight back to whatever the
/// terminal's own settings were before this call. Deliberately scoped
/// this tightly rather than switching into raw mode once and leaving it
/// -- `INPUT`'s own `bcc_read_line` (`fgets`) shares the same stdin file
/// descriptor and needs real `ICANON` line-buffering/editing to work at
/// all; leaving the terminal permanently raw after the first `INKEY$`
/// call silently broke every `INPUT` after it (fgets would return
/// whatever partial, unbuffered bytes happened to be sitting there
/// instead of a real line). The per-call toggle costs two extra syscalls
/// each time, negligible against a human's own keystroke timing. A real,
/// documented divergence from real BASIC: this only ever works against
/// an interactive terminal (POSIX `<termios.h>`), unlike BASCOM's own
/// DOS-console INKEY$.
const INKEY_PROTO: &str = "static const char* bcc_inkey(void);\n";
const INKEY_BODY: &str = "static const char* bcc_inkey(void) {\n    struct termios orig, raw;\n    tcgetattr(STDIN_FILENO, &orig);\n    raw = orig;\n    raw.c_lflag &= ~(ICANON | ECHO);\n    raw.c_cc[VMIN] = 0;\n    raw.c_cc[VTIME] = 0;\n    tcsetattr(STDIN_FILENO, TCSANOW, &raw);\n\n    static char buf[2];\n    unsigned char c;\n    ssize_t n = read(STDIN_FILENO, &c, 1);\n    if (n == 1) {\n        buf[0] = (char)c;\n        buf[1] = 0;\n    } else {\n        buf[0] = 0;\n    }\n\n    tcsetattr(STDIN_FILENO, TCSANOW, &orig);\n    return buf;\n}\n\n";

/// `ON ERROR GOTO`/`RESUME`/`ERROR`/`ERR`/`ERL`'s runtime state -- see
/// `emit_raise_block`'s own doc comment for how these are used.
/// `bcc_err` is `ERR`; `bcc_erl` is `ERL` (see `render_numeric_expr`'s
/// `Expr::Ident` arm) -- the real `.bcl` source line of the raise site,
/// a compile-time literal each `emit_raise_block` call bakes in.
/// `bcc_resume_id` is a *different* per-raise-site value: a small
/// sequential index (not a line number, and not shown to BASCAL code at
/// all) that `RESUME`/`RESUME NEXT` switches on to `goto` back to the
/// right site -- kept separate from `bcc_erl` because two raise sites can
/// share a source line (so `bcc_erl` alone couldn't dispatch uniquely),
/// and because a `RESUME` dispatch table keyed on line numbers would leave
/// gaps a real line-number-keyed `switch` still has to handle correctly.
const ERROR_HANDLING_GLOBALS: &str = "static int bcc_err = 0;\nstatic int bcc_on_error_target = -1;\nstatic int bcc_in_handler = 0;\nstatic int bcc_resume_id = -1;\nstatic int bcc_erl = 0;\n\n";

/// A `try`-reachable procedure's own C return type (see
/// `collect_try_reachable_procedures`'s own doc comment) -- `status` is
/// `0` on normal completion, or the raised error code otherwise. Deliberately
/// a named struct, not a bare `int`, even though a `procedure` carries no
/// other payload: this is the first of the `bcc_result_*` family GitHub
/// issue #60 originally proposed (a `function`'s own `bcc_result_int`/
/// `_long`/`_single`/`_double`, each pairing this same `status` field with
/// a real `value`, are issue #67's follow-up) -- sharing the shape now
/// means a `procedure` promoted to a `function` later never needs a
/// second signature change, and every call site's status-check code
/// (`.status`) already reads identically either way.
const TRY_RESULT_TYPE: &str = "typedef struct { int status; } bcc_result_void;\n\n";

/// `DATA`/`READ`/`RESTORE`'s runtime state: `bcc_data` (declared
/// separately, right before this, with the program's actual literal
/// items -- see `collect_data_items_and_labels`/`generate`'s own
/// `data_array_decl`) is one flat, program-order array of every `DATA`
/// item's raw text, `BCC_DATA_COUNT` items long; `bcc_data_ptr` is the
/// single global read cursor real BASIC's own `READ`/`RESTORE` share.
/// `bcc_read_data` is `READ`'s only access to it -- out-of-DATA is a
/// plain fatal exit (real BASIC's own error 4, "Out of DATA", but not
/// routed through `emit_raise_block`'s trappable-error machinery: no
/// tutorial or test here needs a *recoverable* out-of-DATA, and every
/// raise site there is a fixed, known program position `RESUME`/`RESUME
/// NEXT` can jump back to by ID, which an arbitrary `READ` call site
/// would need its own ID for too -- deliberately not attempted).
const DATA_STATE: &str = "static int bcc_data_ptr = 0;\n\n";
const DATA_PROTO: &str = "static const char* bcc_read_data(void);\n";
const DATA_BODY: &str = "static const char* bcc_read_data(void) {\n    if (bcc_data_ptr >= BCC_DATA_COUNT) {\n        fprintf(stderr, \"Out of DATA\\n\");\n        exit(1);\n    }\n    return bcc_data[bcc_data_ptr++];\n}\n\n";

/// `GOSUB`/`RETURN`'s runtime state: a return-address stack, exactly how a
/// real BASIC interpreter itself implements GOSUB (push where to resume,
/// jump; on RETURN, pop and jump back) -- not a real address, just a small
/// integer ID `Statement::Gosub`'s own doc comment explains the rest of.
/// Depth is fixed and unchecked (an overflow silently overwrites the
/// stack) -- same "trusts the program, no runtime bounds check" category
/// as this backend's other fixed-size buffers (`STRING_BUFFER_SIZE`,
/// `BCC_MAX_CHANNELS`); 64 nested GOSUBs is far beyond anything a real
/// BASIC program written by hand would ever reach.
const GOSUB_HELPER: &str =
    "#define BCC_MAX_GOSUB_DEPTH 64\nstatic int bcc_gosub_stack[BCC_MAX_GOSUB_DEPTH];\nstatic int bcc_gosub_sp = 0;\n\n";

/// Sequential file I/O's runtime helpers -- `OPEN FOR INPUT/OUTPUT/APPEND`
/// shares `bcc_files`/`FILE_IO_HELPER` with random-access I/O (see
/// `Statement::Open`'s own doc comment), but reading a sequential file back
/// needs its own machinery `FILE_IO_HELPER` doesn't have:
///
/// `bcc_eof(file)` -- peeks one character with `fgetc`/`ungetc` (the
/// standard portable "is the next read going to hit EOF" idiom -- C has no
/// direct "am I at EOF" query that doesn't require having already tried a
/// failed read) to answer `EOF(#ch)` without disturbing the stream position.
/// Returns real BASIC's own -1 (true)/0 (false) convention, not C's 1/0 --
/// see `render_numeric_expr`'s comparison-operator arm for why every
/// boolean-shaped value in this backend has to be -1/0: something built on
/// top of this result (`NOT`'s bitwise complement, `AND`/`OR`) assumes it,
/// and `~1` (C's 1-for-true) is `-2`, not `0`, so plain C truthiness here
/// silently breaks `NOT`/`AND`/`OR` without ever failing to compile.
///
/// `bcc_line_input_file(file, buf, bufsize)` -- backs `LINE INPUT #`,
/// exactly `bcc_read_line`'s `fgets`-plus-`strcspn` shape (see
/// `INPUT_HELPER`) but reading from a given file instead of always `stdin`.
///
/// `bcc_read_file_field(file, buf, bufsize)` -- backs `INPUT #`, reading
/// one comma-or-newline-delimited field from the format `WRITE #` produces
/// (see `Statement::Write`'s own doc comment): a `"`-quoted run reads
/// everything up to the closing quote verbatim (any BASIC-style embedded
/// `""` escaping is out of scope, same category as this backend's other
/// unchecked-range gaps); an unquoted run reads up to the next comma or
/// line ending. Either way the trailing delimiter (`,`, or `\r`/`\n`) is
/// consumed so the next field read starts clean, matching real
/// `INPUT #`'s own field-at-a-time consumption.
const SEQ_FILE_STATE: &str = "static char bcc_file_field_buf[256];\n\n";
const SEQ_FILE_PROTOS: &str = "static int bcc_eof(FILE* file);\nstatic void bcc_line_input_file(FILE* file, char* buf, size_t bufsize);\nstatic void bcc_read_file_field(FILE* file, char* buf, size_t bufsize);\n";
const SEQ_FILE_BODY: &str = "static int bcc_eof(FILE* file) {\n    int c = fgetc(file);\n    if (c == EOF) return -1;\n    ungetc(c, file);\n    return 0;\n}\n\nstatic void bcc_line_input_file(FILE* file, char* buf, size_t bufsize) {\n    if (fgets(buf, (int)bufsize, file) == NULL) {\n        buf[0] = 0;\n        return;\n    }\n    buf[strcspn(buf, \"\\r\\n\")] = 0;\n}\n\nstatic void bcc_read_file_field(FILE* file, char* buf, size_t bufsize) {\n    int c = fgetc(file);\n    while (c == ' ') c = fgetc(file);\n    size_t len = 0;\n    if (c == '\"') {\n        c = fgetc(file);\n        while (c != EOF && c != '\"') {\n            if (len + 1 < bufsize) buf[len++] = (char)c;\n            c = fgetc(file);\n        }\n        c = fgetc(file);\n        while (c != EOF && c != ',' && c != '\\n') c = fgetc(file);\n    } else {\n        while (c != EOF && c != ',' && c != '\\n' && c != '\\r') {\n            if (len + 1 < bufsize) buf[len++] = (char)c;\n            c = fgetc(file);\n        }\n        if (c == '\\r') {\n            int c2 = fgetc(file);\n            if (c2 != '\\n' && c2 != EOF) ungetc(c2, file);\n        }\n    }\n    buf[len] = 0;\n}\n\n";

/// One field's layout within a `FIELD`-declared channel record buffer --
/// its C variable name (always a string, per `records::buffer_ident`),
/// byte width, and cumulative byte offset within the record. Built by
/// `apply_field_statement`.
#[derive(Clone)]
struct FieldEntry {
    c_name: String,
    width: u32,
    offset: u32,
    is_string: bool,
    ty: Option<RecordFieldType>,
}

/// The random-access record I/O layout *currently in effect*, tracked as
/// mutable state threaded through statement emission (`emit_statement`'s
/// own `functions`/`current_function` parameters, same shape) rather than
/// computed once by an up-front whole-program scan: a channel can be
/// `FIELD`ed more than once over a program's lifetime with a genuinely
/// different layout each time (reopened for a different purpose, or --
/// exactly what `tutorial/15_random_and_record_files.bcl` does -- the
/// same file reopened under a different set of buffer variable names
/// later in the same program), and each `GET`/`PUT`/`LSET`/`RSET` needs
/// the layout *most recently established for that channel*, not
/// whichever `FIELD` statement happens to be textually last in the whole
/// program. An up-front scan that only kept "the last `FIELD` per
/// channel" got this wrong -- Part 1 and Part 2 of that tutorial both
/// `FIELD` channel `#1`, with different variable names, and Part 1's own
/// `GET`/`PUT` calls were silently reading/writing Part 2's buffers
/// instead of their own.
struct FileIoLayout {
    /// Channel number -> that channel's *currently FIELD'd* fields, in
    /// `FIELD`-declaration order (needed by `GET`/`PUT` to split/join the
    /// whole record buffer). Updated in place by `apply_field_statement`
    /// each time a `FIELD` statement is actually emitted, reflecting
    /// program order, not just replaced once at scan time.
    channel_fields: HashMap<i64, Vec<FieldEntry>>,
    /// C variable name -> its *most recently FIELD'd* width -- looked up
    /// by `LSET`/`RSET`, which only know their target variable, not which
    /// channel/`FIELD` it came from. No ordering ambiguity here the way
    /// `channel_fields` has: two different `FIELD` statements naming the
    /// same C variable would have to agree on its width anyway (the
    /// variable's underlying C buffer has only one size), so simple
    /// insert-and-overwrite is correct regardless of program order.
    field_widths: HashMap<String, u32>,
    /// Whether the program uses random-access record I/O at all (any of
    /// `OPEN`/`CLOSE`/`FIELD`/`GET`/`PUT`/`LSET`/`RSET`) -- set as
    /// emission encounters one, consulted afterward (same "mutate during
    /// emission, read once emission's done" pattern `needs_math`/
    /// `needs_string` already use) to decide whether `generate` emits
    /// `FILE_IO_HELPER` and pulls in `<string.h>`/`<stdint.h>` for it.
    used: bool,
    /// Channel number -> the record/file DSL's declared record type name
    /// for its *currently FIELD'd* layout (see `Statement::Field`'s own
    /// doc comment), or `None` for a raw, hand-typed `FIELD` the DSL never
    /// produced. Purely cosmetic: only consulted by `record_helper_name`
    /// to name a synthesized whole-record helper after its actual type
    /// instead of an anonymous channel number.
    channel_record_type: HashMap<i64, Option<String>>,
    /// Fixed FIELD shapes declared by the record DSL anywhere in the
    /// program.  A raw FIELD with exactly one of these shapes can use the
    /// record type's readable helper name too; the packing convention is
    /// identical, and this avoids leaking channel/generation names into
    /// generated C for a program that already gives the shape a name.
    known_record_layouts: HashMap<Vec<u32>, (String, Vec<bool>)>,
    /// Record types whose reusable pack/unpack helpers have already been
    /// emitted. A helper takes the channel's current FIELD buffers as
    /// arguments, so every file declared with the same record type can
    /// share it even though those buffers have different C variable names.
    record_helpers: std::collections::HashSet<String>,
    /// Monotonically increasing layout generation per channel. Raw BASIC
    /// can re-FIELD a channel with a different shape, so its synthesized
    /// field-layout helper needs a fresh name each time that happens.
    channel_generation: HashMap<i64, u32>,
    /// Accumulated C source for every synthesized record helper's
    /// *implementation*, spliced into `generate`'s output after `main`,
    /// alongside `FILE_IO_BODY`, once emission of the whole program is
    /// done.
    helper_defs: String,
    /// One-line forward declaration per synthesized record helper in
    /// `helper_defs`, spliced in near the top (alongside `FILE_IO_PROTOS`)
    /// so `main`'s own calls to it, which appear earlier in the file than
    /// `helper_defs`' bodies do, still compile.
    helper_protos: String,
    /// Buffer C variable name -> the *unpacked* source expression a
    /// record/file DSL write is packing into it, captured by `Statement::
    /// Lset`'s C backend arm instead of being packed into the buffer
    /// immediately. The following `PUT` (always emitted right after,
    /// see `records::Lowerer::lower_whole_write` and friends) drains this
    /// to pass each field to the typed DSL PUT helper (`ensure_dsl_record_
    /// helpers`) as a native value, so the helper -- not generated code in
    /// `main` -- owns the packing. Never populated for a raw, hand-written
    /// `FIELD`'s `LSET`, which packs into its buffer immediately as before.
    pending_field_values: HashMap<String, Expr>,
}

/// Applies one `FIELD #ch, w1 AS v1$, ...` statement's layout into
/// `layout`, in place -- called from `emit_statement`'s own `Statement::
/// Field` arm at the point the statement is actually emitted (see
/// `FileIoLayout`'s own doc comment for why this has to happen live,
/// in program order, rather than as a one-time up-front scan). Errors
/// out (rather than silently guessing) when the channel or a field's
/// width isn't a literal integer: the layout has to be known at compile
/// time here (there's no runtime "ask the FIELD table" mechanism the way
/// real BASIC's own interpreter has), so a computed channel/width --
/// always a literal in BASCAL's own record/file DSL output, see
/// `records::lower_file_decl` -- isn't supported by the minimal C
/// backend yet.
fn apply_field_statement(
    channel: &Expr,
    fields: &[(Expr, BasicIdent)],
    record_type: &Option<String>,
    string_fields: &Option<Vec<bool>>,
    field_types: &Option<Vec<RecordFieldType>>,
    layout: &mut FileIoLayout,
) -> Result<(), String> {
    layout.used = true;
    let Expr::Integer(ch) = channel else {
        return Err(
            "`FIELD`'s channel must be a literal integer -- the minimal C backend needs to know \
             the record layout at compile time"
                .to_string(),
        );
    };
    if !(1..=BCC_MAX_CHANNELS).contains(ch) {
        return Err(format!(
            "file channel #{ch} is out of range -- the minimal C backend supports channels 1 \
             through {BCC_MAX_CHANNELS}"
        ));
    }
    let mut entries = Vec::with_capacity(fields.len());
    let mut offset = 0u32;
    for (width_expr, var) in fields {
        let Expr::Integer(width) = width_expr else {
            return Err(
                "a `FIELD` width must be a literal integer -- the minimal C backend needs to \
                 know the record layout at compile time"
                    .to_string(),
            );
        };
        if var.suffix != Some(TypeSuffix::String) {
            return Err(format!(
                "`FIELD` variable `{var}` must be a string (`$`) -- real MBASIC/BASCOM's \
                 `FIELD` only ever declares string variables"
            ));
        }
        // Every FIELD'd variable's C storage is the same
        // `char[STRING_BUFFER_SIZE]` buffer as any other string -- GET
        // splits raw record bytes into it via `memcpy` (see
        // `emit_get_or_put`), so a field wider than that buffer would
        // silently overflow it rather than truncate the way
        // `snprintf`-based string handling elsewhere in this backend
        // safely does.
        if *width < 0 || *width as usize >= STRING_BUFFER_SIZE {
            return Err(format!(
                "`FIELD` variable `{var}`'s width ({width}) must be less than \
                 {STRING_BUFFER_SIZE} -- every string in the minimal C backend, FIELD'd \
                 variables included, is a fixed {STRING_BUFFER_SIZE}-byte buffer"
            ));
        }
        let width = *width as u32;
        let c_name = c_var_name(var, TypeSuffix::String);
        layout.field_widths.insert(c_name.clone(), width);
        entries.push(FieldEntry {
            c_name,
            width,
            offset,
            is_string: string_fields.as_ref().and_then(|items| items.get(entries.len())).copied().unwrap_or(false),
            ty: field_types.as_ref().and_then(|items| items.get(entries.len())).copied(),
        });
        offset += width;
    }
    if layout.channel_fields.contains_key(ch) {
        *layout.channel_generation.entry(*ch).or_insert(0) += 1;
    } else {
        layout.channel_generation.entry(*ch).or_insert(0);
    }
    let inferred_layout = layout
        .known_record_layouts
        .get(&entries.iter().map(|entry| entry.width).collect::<Vec<_>>())
        .cloned();
    if string_fields.is_none() {
        if let Some((_, string_fields)) = &inferred_layout {
            for (entry, is_string) in entries.iter_mut().zip(string_fields) {
                entry.is_string = *is_string;
            }
        }
    }
    layout.channel_fields.insert(*ch, entries);
    // Only ever the literal `record_type` this `FIELD` statement itself
    // declared -- never `inferred_layout`'s name. A raw, hand-written
    // `FIELD` that happens to match a declared record type's byte widths
    // still gets that inference for `is_string` above (harmless, purely a
    // read-back detail), but must *not* be treated as if the record DSL
    // produced it: its buffers are real, user-visible BASIC variables, and
    // `emit_get_or_put`/`Statement::Lset` both gate the typed DSL PUT
    // helper path on this map alone.
    layout.channel_record_type.insert(*ch, record_type.clone());
    Ok(())
}

/// Collect named record layouts before emitting statements. The record DSL
/// leaves its type name and per-field `is_string` shape on synthesized
/// FIELD statements; an otherwise raw FIELD with the same byte widths
/// reuses that shape purely to classify its own fields correctly (a
/// numeric-packed field reads back differently from a genuine string one)
/// -- never to name or share the record type's *helper pair*, which stays
/// gated on `record_type` being literally present on the FIELD statement
/// itself (see `apply_field_statement`'s own note on `channel_record_type`).
fn known_record_layouts(program: &Program) -> HashMap<Vec<u32>, (String, Vec<bool>)> {
    fn collect(statements: &[Stmt], layouts: &mut HashMap<Vec<u32>, (String, Vec<bool>)>) {
        for statement in statements {
            match &statement.kind {
                Statement::Field {
                    fields,
                    record_type: Some(name),
                    string_fields,
                    ..
                } => {
                    let widths = fields
                        .iter()
                        .filter_map(|(width, _)| match width {
                            Expr::Integer(value) if *value >= 0 => Some(*value as u32),
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    if widths.len() == fields.len() {
                        let strings = string_fields
                            .clone()
                            .unwrap_or_else(|| vec![false; fields.len()]);
                        layouts.entry(widths).or_insert_with(|| (name.clone(), strings));
                    }
                }
                Statement::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    collect(then_body, layouts);
                    collect(else_body, layouts);
                }
                Statement::For { body, .. }
                | Statement::While { body, .. }
                | Statement::Do { body, .. } => {
                    collect(body, layouts);
                }
                Statement::SelectCase {
                    cases, else_body, ..
                } => {
                    for case in cases {
                        collect(&case.body, layouts);
                    }
                    collect(else_body, layouts);
                }
                _ => {}
            }
        }
    }

    let mut layouts = HashMap::new();
    collect(&program.statements, &mut layouts);
    for function in &program.functions {
        collect(&function.body, &mut layouts);
    }
    layouts
}

/// Whether `program` uses `color` anywhere (top level or inside any
/// function/procedure) -- decides whether `generate()` splices in
/// `COLOR_HELPER` at all, the same "only pull in what's actually used"
/// policy `scan_builtin_usage`/`file_io.used` already follow for their own
/// helper blocks. `cls`/`beep`/`locate` need no such check: each compiles
/// to a self-contained `printf` call with no shared helper of its own.
/// Whether any statement anywhere in `program` (top level or inside any
/// function/procedure, any nesting depth) matches `pred` -- the shared
/// walker behind `program_uses_color`/`program_uses_input`'s own "is this
/// helper block even needed" checks, the same "only pull in what's
/// actually used" policy `scan_builtin_usage`/`file_io.used` already
/// follow for theirs. `pred` is only ever asked about the statement
/// itself, never its own nested body (an `if`'s `then_body`, a `for`'s
/// `body`, ...) -- this function does that recursion once, centrally.
fn program_has_statement(program: &Program, pred: &dyn Fn(&Stmt) -> bool) -> bool {
    fn walk(statements: &[Stmt], pred: &dyn Fn(&Stmt) -> bool) -> bool {
        statements.iter().any(|statement| {
            pred(statement)
                || match &statement.kind {
                    Statement::If {
                        then_body,
                        else_body,
                        ..
                    } => walk(then_body, pred) || walk(else_body, pred),
                    Statement::For { body, .. }
                    | Statement::While { body, .. }
                    | Statement::Do { body, .. } => walk(body, pred),
                    Statement::SelectCase {
                        cases, else_body, ..
                    } => cases.iter().any(|case| walk(&case.body, pred)) || walk(else_body, pred),
                    Statement::TryCatch {
                        try_body,
                        catch_body,
                        ..
                    } => walk(try_body, pred) || walk(catch_body, pred),
                    _ => false,
                }
        })
    }
    walk(&program.statements, pred) || program.functions.iter().any(|f| walk(&f.body, pred))
}

/// Total number of `GOSUB` statements in `statements`, walking the same
/// nesting shape `program_has_statement` does (`if`/`for`/`while`/`do`/
/// `select case` bodies) -- but only over top-level statements, never
/// into `program.functions`, since GOSUB is rejected outright inside a
/// function/procedure body (see `Statement::Gosub`'s own doc comment).
/// Computed once, before emission starts, so `generate()` knows
/// `gosub_count` up front; the *order* this walks statements in has to
/// exactly match the order `emit_statement`'s own recursion visits them
/// in, since that's what lets a plain incrementing counter (`gosub_id`)
/// assign each GOSUB the same ID both here (implicitly, via position in
/// the `0..gosub_count` range) and during real emission.
fn count_gosubs(statements: &[Stmt]) -> usize {
    statements
        .iter()
        .map(|statement| {
            let self_count = usize::from(matches!(&**statement, Statement::Gosub(_)));
            let nested_count = match &statement.kind {
                Statement::If {
                    then_body,
                    else_body,
                    ..
                } => count_gosubs(then_body) + count_gosubs(else_body),
                Statement::For { body, .. }
                | Statement::While { body, .. }
                | Statement::Do { body, .. } => count_gosubs(body),
                Statement::SelectCase {
                    cases, else_body, ..
                } => {
                    cases
                        .iter()
                        .map(|case| count_gosubs(&case.body))
                        .sum::<usize>()
                        + count_gosubs(else_body)
                }
                Statement::TryCatch {
                    try_body,
                    catch_body,
                    ..
                } => count_gosubs(try_body) + count_gosubs(catch_body),
                _ => 0,
            };
            self_count + nested_count
        })
        .sum()
}

/// Total number of *raise sites* in `statements`, walking the same nesting
/// shape `count_gosubs` does -- an explicit `ERROR <code>` statement, or a
/// sequential `OPEN ... FOR INPUT` (whose C translation now checks for a
/// failed `fopen` and raises real BASIC's own error 53, "file not found",
/// instead of silently leaving a NULL `FILE*` behind -- see
/// `Statement::Open`'s own `OpenMode::Input` arm). Only top-level
/// statements are counted, matching `ON ERROR GOTO`/`RESUME`/`ERROR`'s own
/// top-level-only restriction (see `Statement::OnErrorGoto`'s doc
/// comment): a raise site needs `RESUME`/`RESUME NEXT` to be able to jump
/// back to it by ID, and `RESUME` itself is rejected inside a function/
/// procedure body. Computed once, before emission starts (see
/// `ErrorDataCtx`), the same "count now, assign IDs during real emission
/// in the identical order" split `count_gosubs`/`gosub_id` already use.
fn count_raise_sites(statements: &[Stmt]) -> usize {
    statements
        .iter()
        .map(|statement| {
            let self_count = usize::from(
                matches!(&**statement, Statement::ErrorStmt { .. })
                    || matches!(
                        &**statement,
                        Statement::Open {
                            mode: OpenMode::Input,
                            ..
                        }
                    ),
            );
            let nested_count = match &statement.kind {
                Statement::If {
                    then_body,
                    else_body,
                    ..
                } => count_raise_sites(then_body) + count_raise_sites(else_body),
                Statement::For { body, .. }
                | Statement::While { body, .. }
                | Statement::Do { body, .. } => count_raise_sites(body),
                Statement::SelectCase {
                    cases, else_body, ..
                } => {
                    cases
                        .iter()
                        .map(|case| count_raise_sites(&case.body))
                        .sum::<usize>()
                        + count_raise_sites(else_body)
                }
                Statement::TryCatch {
                    try_body,
                    catch_body,
                    ..
                } => count_raise_sites(try_body) + count_raise_sites(catch_body),
                _ => 0,
            };
            self_count + nested_count
        })
        .sum()
}

/// Total number of top-level `try`/`catch` blocks in `statements`,
/// walking the same nesting shape `count_raise_sites` does -- but never
/// descending into a `try`/`catch`'s own `try_body`/`catch_body` looking
/// for *more* `try`/`catch` blocks, since `resolver::reject_nested_try_
/// catch` already guarantees there can't be any there. Used only to size
/// `generate`'s own `dispatch_labels` up front; `Statement::TryCatch`'s
/// own arm in `emit_statement` assigns the matching id to each one during
/// real emission, via `ctx.try_id`, in this same left-to-right, depth-
/// first order.
fn count_try_catch_blocks(statements: &[Stmt]) -> usize {
    statements
        .iter()
        .map(|statement| {
            let self_count = usize::from(matches!(&**statement, Statement::TryCatch { .. }));
            let nested_count = match &statement.kind {
                Statement::If {
                    then_body,
                    else_body,
                    ..
                } => count_try_catch_blocks(then_body) + count_try_catch_blocks(else_body),
                Statement::For { body, .. }
                | Statement::While { body, .. }
                | Statement::Do { body, .. } => count_try_catch_blocks(body),
                Statement::SelectCase {
                    cases, else_body, ..
                } => {
                    cases
                        .iter()
                        .map(|case| count_try_catch_blocks(&case.body))
                        .sum::<usize>()
                        + count_try_catch_blocks(else_body)
                }
                _ => 0,
            };
            self_count + nested_count
        })
        .sum()
}

/// Every `procedure` (never a `function` -- see `collect_try_reachable_
/// procedures`'s own doc comment for why those stay out of scope for now)
/// directly called as a bare statement inside `statements`, walking the
/// same nesting shape `count_try_catch_blocks` does. Shared by both the
/// initial try-body seed pass (`collect_try_body_seeds`) and the BFS
/// expansion over each newly reachable procedure's own body (which never
/// contains a `try`/`catch` itself -- top-level-only, so no arm for it is
/// needed here at all).
fn collect_called_procedures(
    statements: &[Stmt],
    functions: &FunctionTable,
    out: &mut Vec<(String, Option<TypeSuffix>)>,
) {
    for stmt in statements {
        match &stmt.kind {
            Statement::ExprStmt(Expr::Call { name, .. })
            | Statement::ExprStmt(Expr::ArrayRef { name, .. }) => {
                let key = fn_key(name);
                if functions.get(&key).is_some_and(|sig| sig.is_void) {
                    out.push(key);
                }
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_called_procedures(then_body, functions, out);
                collect_called_procedures(else_body, functions, out);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_called_procedures(body, functions, out),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_called_procedures(&case.body, functions, out);
                }
                collect_called_procedures(else_body, functions, out);
            }
            _ => {}
        }
    }
}

/// Seeds `out` with every procedure directly called inside a top-level
/// `try`'s own `try_body` -- deliberately never `catch_body`: a call
/// there runs with no trap active (see the `Statement::TryCatch` arm in
/// `emit_statement`, which resets `bcc_on_error_target` to `-1` at catch
/// entry, real BASIC's own "no nested-trap recovery" rule), so a
/// procedure only ever called from a `catch_body` has no active `try` to
/// propagate a raise to and doesn't need this treatment at all.
fn collect_try_body_seeds(
    statements: &[Stmt],
    functions: &FunctionTable,
    out: &mut Vec<(String, Option<TypeSuffix>)>,
) {
    for stmt in statements {
        match &stmt.kind {
            Statement::TryCatch { try_body, .. } => {
                collect_called_procedures(try_body, functions, out);
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_try_body_seeds(then_body, functions, out);
                collect_try_body_seeds(else_body, functions, out);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_try_body_seeds(body, functions, out),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_try_body_seeds(&case.body, functions, out);
                }
                collect_try_body_seeds(else_body, functions, out);
            }
            _ => {}
        }
    }
}

/// Every procedure (transitively) on a call path from some top-level
/// `try`'s own `try_body` -- issue #60's C-target propagation, scoped
/// down to procedures only for now. A `function` used inside an
/// expression would need every generated/wrapper function to return a
/// `{status, value}` struct and every call in the *entire program* --
/// not just ones near a `try` -- hoisted to a checked temp, since a call
/// can't sit inline inside an arbitrary expression once its status has
/// to be checked the instant it returns (see the design notes on GitHub
/// issue #60). A `procedure` has no value to carry at all, so none of
/// that applies: its C signature just becomes `int` (a status) instead
/// of `void`, with no expression-flattening required anywhere.
///
/// Once in this set, a procedure gets `error`/failed `open ... for
/// input` allowed inside its own body (previously rejected -- see their
/// own arms in `emit_statement`) and returns a nonzero status instead of
/// (uselessly, cross-function) `goto`-ing a handler directly; every call
/// site to it decides `goto`/propagate/discard purely from its own
/// lexical context (`ErrorDataCtx`'s `current_try_catch`/
/// `current_function_reachable`) -- see `Statement::ExprStmt`'s own
/// procedure-call arm.
fn collect_try_reachable_procedures(
    program: &Program,
    functions: &FunctionTable,
) -> HashSet<(String, Option<TypeSuffix>)> {
    let mut worklist = Vec::new();
    collect_try_body_seeds(&program.statements, functions, &mut worklist);

    let mut reachable = HashSet::new();
    while let Some(key) = worklist.pop() {
        if !reachable.insert(key.clone()) {
            continue;
        }
        if let Some(func) = program.functions.iter().find(|f| fn_key(&f.name) == key) {
            collect_called_procedures(&func.body, functions, &mut worklist);
        }
    }
    reachable
}

/// Every distinct `ON ERROR GOTO <label>` target in `statements` (`ON
/// ERROR GOTO 0`, the disable sentinel, is a numeric literal, not an
/// identifier, so it's naturally excluded), assigned a stable integer ID
/// in first-seen program order -- used both by `ON ERROR GOTO` itself
/// (`bcc_on_error_target = <id>`) and by every raise site's own `switch
/// (bcc_on_error_target) { case <id>: goto bcc_lbl_<label>; ... }` (see
/// `emit_raise_block`), since a raise site has no other way to jump to
/// whichever label the *most recently executed* `ON ERROR GOTO` installed
/// -- that's only known at runtime, not at the raise site's own,
/// textually earlier, position.
fn collect_on_error_handler_ids(statements: &[Stmt]) -> HashMap<String, usize> {
    let mut ids = HashMap::new();
    collect_on_error_handler_ids_into(statements, &mut ids);
    ids
}

fn collect_on_error_handler_ids_into(statements: &[Stmt], ids: &mut HashMap<String, usize>) {
    for statement in statements {
        if let Statement::OnErrorGoto {
            target: Expr::Ident(ident),
        } = &**statement
        {
            let key = ident.name.to_ascii_lowercase();
            if !ids.contains_key(&key) {
                let next_id = ids.len();
                ids.insert(key, next_id);
            }
        }
        match &statement.kind {
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_on_error_handler_ids_into(then_body, ids);
                collect_on_error_handler_ids_into(else_body, ids);
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => collect_on_error_handler_ids_into(body, ids),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_on_error_handler_ids_into(&case.body, ids);
                }
                collect_on_error_handler_ids_into(else_body, ids);
            }
            Statement::TryCatch {
                try_body,
                catch_body,
                ..
            } => {
                collect_on_error_handler_ids_into(try_body, ids);
                collect_on_error_handler_ids_into(catch_body, ids);
            }
            _ => {}
        }
    }
}

/// Renders one `DATA` item's literal text as a quoted C string -- `READ`
/// converts it to its target variable's actual type at read time (`atoi`/
/// Every top-level `const` in `statements` whose value is a plain integer
/// literal (`const n% = 5`) or its negation (`const n% = -5`) -- real
/// BASCAL `const`s aren't compile-time-folded anywhere else in this
/// codebase (see `Statement::Const`'s own handling in `emit_statement`,
/// which just codegens it as an ordinary assignment), but a `dim`'s own
/// array-bound expression needs an actual compile-time integer, since a
/// real C array's size has to be one -- this is the one place that
/// integer value gets recovered. Keyed the same way `fn_key`/`var_key`
/// already do (lowercased name, suffix), matching how a `dim`'s size
/// expression -- a bare `Expr::Ident` -- would reference it.
fn collect_top_level_int_consts(
    statements: &[Stmt],
) -> HashMap<(String, Option<TypeSuffix>), i64> {
    let mut consts = HashMap::new();
    for statement in statements {
        if let Statement::Const { name, value } = &**statement {
            let literal = match value {
                Expr::Integer(n) => Some(*n),
                Expr::Unary {
                    op: UnaryOp::Neg,
                    expr,
                } => match expr.as_ref() {
                    Expr::Integer(n) => Some(-n),
                    _ => None,
                },
                _ => None,
            };
            if let Some(n) = literal {
                consts.insert((name.name.to_ascii_lowercase(), name.suffix), n);
            }
        }
    }
    consts
}

/// The C variable name of every top-level `const`, regardless of its
/// value's shape (unlike `collect_top_level_int_consts`, which only
/// tracks the integer-literal-valued subset `dim`'s own array-bound
/// resolution needs) -- a top-level `const` is implicitly visible from
/// every function/procedure body with no `global` declaration needed
/// (see this file's own module doc comment and `tutorial/inventory.bcl`'s
/// header note on `const`), so `emit_function_def` needs this to avoid
/// declaring a same-named, always-zero/empty local shadow for a bare
/// reference to one -- the same declaration-collision category `global`-
/// declared names are already excluded for (see `collect_global_decl_
/// idents`), just for a name that never needed the `global` keyword at
/// all to begin with.
fn collect_top_level_const_c_names(statements: &[Stmt]) -> BTreeSet<String> {
    statements
        .iter()
        .filter_map(|statement| match &**statement {
            Statement::Const { name, .. } => Some(c_var_name(name, effective_suffix(name.suffix))),
            _ => None,
        })
        .collect()
}

/// Resolves one `dim` array-bound expression to a compile-time `i64` --
/// either a literal integer, or a bare reference to a top-level `const`
/// with an integer-literal value (see `collect_top_level_int_consts`;
/// `tutorial/09_data.bcl`'s own `dim country$(numCapitals%)`, where
/// `numCapitals%` is exactly such a `const`, is why the second form
/// matters, not just the first). Nothing else is supported -- a runtime-
/// computed bound (real BASIC's own `dim arr%(n%)` for a plain variable
/// `n%` set at runtime) has no fixed C array size to declare, and this
/// backend doesn't attempt a dynamic-allocation fallback.
fn resolve_array_bound_literal(
    expr: &Expr,
    consts: &HashMap<(String, Option<TypeSuffix>), i64>,
) -> Result<i64, String> {
    match expr {
        Expr::Integer(n) => Ok(*n),
        Expr::Ident(ident) => consts
            .get(&(ident.name.to_ascii_lowercase(), ident.suffix))
            .copied()
            .ok_or_else(|| {
                format!(
                    "`dim`'s array bound `{ident}` isn't supported by the minimal C backend yet \
                     -- only a literal integer or a top-level `const` with an integer-literal \
                     value is (a real C array needs a compile-time-known size)"
                )
            }),
        _ => Err(
            "`dim`'s array bound isn't supported by the minimal C backend yet -- only a literal \
             integer or a top-level `const` with an integer-literal value is (a real C array \
             needs a compile-time-known size)"
                .to_string(),
        ),
    }
}

/// Every top-level `dim name(...)` array declaration in `statements`,
/// resolved to its compile-time shape (see `ArrayInfo`) -- walks the same
/// nested control-flow shapes `count_gosubs`/`count_raise_sites` already
/// do, though no BASCAL tutorial or test here actually nests one. Keyed
/// by the same C identifier `c_var_name` would give the array's own
/// scalar elements, so declaration and every later indexed read/write
/// (`Expr::ArrayRef`, `Statement::Assignment`'s array-target arm, ...)
/// agree on the same name.
fn collect_array_declarations(
    statements: &[Stmt],
    consts: &HashMap<(String, Option<TypeSuffix>), i64>,
) -> Result<ArrayTable, String> {
    let mut arrays = ArrayTable::new();
    collect_array_declarations_into(statements, consts, &mut arrays)?;
    Ok(arrays)
}

fn collect_array_declarations_into(
    statements: &[Stmt],
    consts: &HashMap<(String, Option<TypeSuffix>), i64>,
    arrays: &mut ArrayTable,
) -> Result<(), String> {
    for statement in statements {
        match &statement.kind {
            Statement::Dim {
                name,
                is_array: true,
                sizes,
            } => {
                let is_string = name.suffix == Some(TypeSuffix::String);
                let suffix = effective_suffix(name.suffix);
                let element_type = if is_string { None } else { numeric_c_type(suffix) };
                if !is_string && element_type.is_none() {
                    return Err(format!(
                        "`dim {name}` isn't supported by the minimal C backend yet -- only \
                         numeric or string array elements (%, &, !, #, $) are"
                    ));
                }
                let mut bounds = Vec::with_capacity(sizes.len());
                for size in sizes {
                    let bound = resolve_array_bound_literal(size, consts)?;
                    if bound < 0 {
                        return Err(format!("`dim {name}`'s array bound can't be negative"));
                    }
                    bounds.push(bound);
                }
                let c_name = c_var_name(
                    name,
                    if is_string {
                        TypeSuffix::String
                    } else {
                        suffix
                    },
                );
                arrays.insert(
                    c_name,
                    ArrayInfo {
                        bounds,
                        element_type,
                        runtime_len: None,
                    },
                );
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_array_declarations_into(then_body, consts, arrays)?;
                collect_array_declarations_into(else_body, consts, arrays)?;
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => {
                collect_array_declarations_into(body, consts, arrays)?;
            }
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_array_declarations_into(&case.body, consts, arrays)?;
                }
                collect_array_declarations_into(else_body, consts, arrays)?;
            }
            Statement::TryCatch {
                try_body,
                catch_body,
                ..
            } => {
                collect_array_declarations_into(try_body, consts, arrays)?;
                collect_array_declarations_into(catch_body, consts, arrays)?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// `atof` for a numeric target, a plain copy for a string one), exactly
/// the same "raw text in, target decides the type" convention `INPUT`
/// already uses for `bcc_input_buf` (see `Statement::Input`'s own arm) --
/// so a `DATA` item's own type never has to be tracked separately at all.
/// Only literal shapes are accepted: `Expr::String`, `Expr::Integer`,
/// `Expr::Float`, and a negative `Expr::Unary { op: Neg, .. }` wrapping
/// one of the two numeric kinds -- real BASIC's own `DATA` items are
/// always literals, never a general expression.
fn render_data_item(expr: &Expr) -> Result<String, String> {
    match expr {
        Expr::String(s) => Ok(format!("\"{}\"", escape_c_string_literal(s))),
        Expr::Integer(n) => Ok(format!("\"{n}\"")),
        Expr::Float(f) => Ok(format!("\"{f:?}\"")),
        Expr::Unary {
            op: UnaryOp::Neg,
            expr,
        } => match expr.as_ref() {
            Expr::Integer(n) => Ok(format!("\"-{n}\"")),
            Expr::Float(f) => Ok(format!("\"-{f:?}\"")),
            _ => Err(
                "DATA items aren't supported by the minimal C backend yet -- only literal \
                 numbers and strings are"
                    .to_string(),
            ),
        },
        _ => Err(
            "DATA items aren't supported by the minimal C backend yet -- only literal numbers \
             and strings are"
                .to_string(),
        ),
    }
}

/// Walks `statements` in the same textual/execution order `count_gosubs`/
/// `count_raise_sites` already recurse in, collecting every `DATA` item's
/// rendered text (flattened across every `DATA` statement in the program,
/// in order -- real BASIC's own single global read pointer walks exactly
/// this sequence) and, for every `Label`, the item count seen so far --
/// `RESTORE <label>` resolves directly to this count at compile time (see
/// `Statement::Restore`'s own arm), no runtime lookup needed, since a
/// label always denotes a fixed position in program order.
fn collect_data_items_and_labels(
    statements: &[Stmt],
) -> Result<(Vec<String>, HashMap<String, usize>), String> {
    let mut items = Vec::new();
    let mut labels = HashMap::new();
    collect_data_items_and_labels_into(statements, &mut items, &mut labels)?;
    Ok((items, labels))
}

fn collect_data_items_and_labels_into(
    statements: &[Stmt],
    items: &mut Vec<String>,
    labels: &mut HashMap<String, usize>,
) -> Result<(), String> {
    for statement in statements {
        match &statement.kind {
            Statement::Data(values) => {
                for value in values {
                    items.push(render_data_item(value)?);
                }
            }
            Statement::Label(name) => {
                labels.insert(name.to_ascii_lowercase(), items.len());
            }
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_data_items_and_labels_into(then_body, items, labels)?;
                collect_data_items_and_labels_into(else_body, items, labels)?;
            }
            Statement::For { body, .. }
            | Statement::While { body, .. }
            | Statement::Do { body, .. } => {
                collect_data_items_and_labels_into(body, items, labels)?;
            }
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_data_items_and_labels_into(&case.body, items, labels)?;
                }
                collect_data_items_and_labels_into(else_body, items, labels)?;
            }
            Statement::TryCatch {
                try_body,
                catch_body,
                ..
            } => {
                collect_data_items_and_labels_into(try_body, items, labels)?;
                collect_data_items_and_labels_into(catch_body, items, labels)?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Threaded through `emit_statement`/`emit_select_case` alongside the
/// GOSUB machinery (`gosub_count`/`gosub_id`), bundled into one struct
/// instead of three more bare parameters: `handler_ids` and `data_labels`
/// are read-only precomputed tables shared, unchanged, by every call site
/// (top-level and every function/procedure body alike -- see
/// `collect_on_error_handler_ids`/`collect_data_items_and_labels`);
/// `raise_site_count`/`raise_id` are the `ON ERROR GOTO` analogue of
/// `gosub_count`/`gosub_id` -- `raise_site_count` is the precomputed total
/// (`count_raise_sites`), `raise_id` a live counter incremented once per
/// raise site actually emitted, in the same order `count_raise_sites`
/// walks in. Both are `0`/thrown away for a function/procedure body's own
/// pass, since a raise site can only ever be top-level code (see
/// `Statement::OnErrorGoto`'s own doc comment).
///
/// `dispatch_labels` is `emit_raise_block`'s own switch table: index `id`
/// is the `goto` target for `bcc_on_error_target == id`, named handlers
/// first (`handler_ids`, sorted by id) then every top-level `try`/`catch`
/// block appended right after, so a `try`'s own id is always
/// `handler_ids.len() + <its position among try/catch blocks>` -- see
/// `generate`'s own construction of this `Vec`. `try_id` is a live
/// counter over that same try/catch id range, the `Statement::TryCatch`
/// analogue of `raise_id`, seeded at `handler_ids.len()` so the first
/// `try`/`catch` emitted gets exactly the id `dispatch_labels` reserved
/// for it. Empty/thrown away for a function/procedure body's own pass,
/// since `try`/`catch` (like the rest of this error-handling family) is
/// top-level-only.
struct ErrorDataCtx<'a> {
    handler_ids: &'a HashMap<String, usize>,
    dispatch_labels: &'a [String],
    raise_site_count: usize,
    raise_id: usize,
    try_id: usize,
    data_labels: &'a HashMap<String, usize>,
    /// Every procedure a raise can propagate a nonzero status out of --
    /// see `collect_try_reachable_procedures`'s own doc comment. Read at
    /// every bare procedure-call statement to decide whether that call
    /// even returns a status worth checking at all.
    try_reachable: &'a HashSet<(String, Option<TypeSuffix>)>,
    /// Whether the function/procedure body currently being emitted is
    /// itself in `try_reachable` -- read by `Statement::ReturnVoid` (does
    /// an explicit bare `return` need `return 0;` instead of `return;`?)
    /// and by `Statement::ErrorStmt`/`Statement::Open`'s `OpenMode::Input`
    /// arm (is a raise here even allowed, now that raise sites inside a
    /// function/procedure body are permitted for reachable procedures
    /// specifically, still rejected for every other one). Always `false`
    /// at top level, where neither question is ever asked.
    current_function_reachable: bool,
    /// The enclosing `try`'s own catch label, only while emitting that
    /// `try`'s `try_body` specifically -- `None` everywhere else,
    /// including that same `try`'s own `catch_body` (see
    /// `collect_try_body_seeds`'s own doc comment for why a call there
    /// never needs to `goto` back into the `try` it's already inside).
    /// Read by a reachable procedure's own call sites to decide `goto`
    /// (inside the owning `try_body`) vs. propagate-by-`return` (inside
    /// another reachable procedure's own body, `current_function_
    /// reachable`) vs. silently discard the status (neither -- some
    /// unrelated call path this whole mechanism doesn't apply to).
    current_try_catch: Option<String>,
}

/// Emits the shared "an error just occurred" block a raise site (`ERROR`
/// or a failed sequential `OPEN ... FOR INPUT`) drops into: record the
/// code, the real `.bcl` source line it raised from (`bcc_erl` -- what
/// `ERL` reads, see `render_numeric_expr`'s `Expr::Ident` arm; `err_line`
/// is a compile-time literal baked in from that statement's own `Stmt.pos`
/// at the call site, so this is the actual source line, not a synthetic
/// stand-in) and which site raised for `RESUME`/`RESUME NEXT` dispatch
/// purposes (`bcc_resume_id` -- see `ERROR_HANDLING_GLOBALS`'s own doc
/// comment for why this is a separate value from `bcc_erl`), then either
/// escalate to a fatal, uncaught-error exit (no handler installed, or
/// already inside one -- real BASIC has no nested-trap recovery either) or
/// dispatch to whichever label the most recently executed `ON ERROR GOTO`
/// (or the currently active `try`/`catch`) installed. `dispatch_labels`
/// empty means the program has no `ON ERROR GOTO`/`try`/`catch` at all --
/// the `switch` below then has no cases and is unreachable in practice
/// (`bcc_on_error_target` can only ever be `-1`), but still valid,
/// warning-free C.
fn emit_raise_block(
    out: &mut String,
    err_code_text: &str,
    raise_id: usize,
    err_line: usize,
    dispatch_labels: &[String],
) {
    out.push_str(&format!("    bcc_err = {err_code_text};\n"));
    out.push_str(&format!("    bcc_resume_id = {raise_id};\n"));
    out.push_str(&format!("    bcc_erl = {err_line};\n"));
    out.push_str("    if (bcc_on_error_target < 0 || bcc_in_handler) {\n");
    out.push_str("        fprintf(stderr, \"unhandled BASIC error %d\\n\", bcc_err);\n");
    out.push_str("        exit(1);\n");
    out.push_str("    }\n");
    out.push_str("    bcc_in_handler = 1;\n");
    out.push_str("    switch (bcc_on_error_target) {\n");
    for (id, label) in dispatch_labels.iter().enumerate() {
        out.push_str(&format!("    case {id}: goto {label};\n"));
    }
    out.push_str("    }\n");
}

/// The "an error just occurred" block for a raise site inside a `try`-
/// reachable procedure's own body (see `collect_try_reachable_
/// procedures`) -- `emit_raise_block`'s counterpart for that context.
/// Records the code/line the same way, then either escalates to the
/// identical fatal, uncaught-error exit, or -- since a `goto` here could
/// never reach a handler that lives in a different C function -- returns
/// a nonzero status for the immediate caller to notice and keep
/// propagating (see `Statement::ExprStmt`'s own procedure-call arm). No
/// retry/after labels, no `bcc_resume_id` dispatch: `resume`/`resume
/// next` are still rejected inside a function/procedure body (see
/// `Statement::Resume`'s own arm), so nothing here would ever read
/// either.
fn emit_raise_in_procedure_block(out: &mut String, err_code_text: &str, err_line: usize) {
    out.push_str(&format!("    bcc_err = {err_code_text};\n"));
    out.push_str(&format!("    bcc_erl = {err_line};\n"));
    out.push_str("    if (bcc_on_error_target < 0 || bcc_in_handler) {\n");
    out.push_str("        fprintf(stderr, \"unhandled BASIC error %d\\n\", bcc_err);\n");
    out.push_str("        exit(1);\n");
    out.push_str("    }\n");
    out.push_str("    bcc_in_handler = 1;\n");
    out.push_str("    return (bcc_result_void){ .status = bcc_err };\n");
}

fn program_uses_color(program: &Program) -> bool {
    program_has_statement(program, &|s| matches!(&**s, Statement::Color { .. }))
}

fn program_uses_input(program: &Program) -> bool {
    program_has_statement(program, &|s| matches!(&**s, Statement::Input { .. }))
}

/// Whether `program` has any `RANDOMIZE` at all -- decides whether
/// `generate()` needs `<stdlib.h>` for `srand()` (independent of whether
/// `RND` itself is ever called -- see `scan_builtin_usage`'s own
/// `needs_stdlib_h` set for that half).
fn program_uses_randomize(program: &Program) -> bool {
    program_has_statement(program, &|s| matches!(&**s, Statement::Randomize(_)))
}

/// Whether `program` has a `RANDOMIZE` that needs `time(NULL)` -- bare
/// `RANDOMIZE` or `RANDOMIZE TIMER` (see `Statement::Randomize`'s own
/// handling in `emit_statement` for why both fall back to the same
/// time-based seed) -- as opposed to `RANDOMIZE <numeric seed>`, which
/// needs no `<time.h>` at all.
fn program_uses_randomize_time(program: &Program) -> bool {
    program_has_statement(program, &|s| {
        matches!(&**s, Statement::Randomize(None))
            || matches!(
                &**s,
                Statement::Randomize(Some(Expr::Ident(ident)))
                    if ident.suffix.is_none() && ident.name.eq_ignore_ascii_case("timer")
            )
    })
}

/// Whether `program` uses sequential file I/O anywhere -- decides whether
/// `generate()` splices in `SEQ_FILE_HELPER`. `WRITE #`/`INPUT #`/
/// `LINE INPUT #`/`PRINT #`/a non-random-access `OPEN` are all statement
/// forms, caught directly by `program_has_statement`; `EOF(...)` is an
/// expression (a plain numeric-function call), so it's caught separately
/// by `scan_builtin_usage`'s `needs_seq_file_helper` flag instead -- see
/// its own match arm.
fn program_uses_sequential_file_io(program: &Program) -> bool {
    program_has_statement(program, &|s| {
        matches!(
            &**s,
            Statement::Write { .. }
                | Statement::InputFile { .. }
                | Statement::LineInput { .. }
                | Statement::PrintFile { .. }
        ) || matches!(
            &**s,
            Statement::Open { mode, .. }
                if !matches!(mode, OpenMode::Random | OpenMode::Binary)
        )
    })
}

/// Applies every top-level `FIELD` statement (the record/file DSL's own
/// synthesized ones included) into `layout`, in program order, *before*
/// `generate()` emits any function/procedure body. Without this, a
/// function referencing a channel's `FIELD` layout -- declared at the top
/// level, always executed before the function is ever called, but *textually
/// emitted after* every function in the generated C (see `generate()`) --
/// would see no layout for that channel at all: `FileIoLayout` is normally
/// only ever populated live, as `Statement::Field` is actually emitted (see
/// `FileIoLayout`'s own doc comment for why), and a function's own body is
/// emitted in a separate pass that runs first.
///
/// This does not attempt to give a function the *exact* layout active at
/// its own particular call site -- a channel re-`FIELD`ed partway through
/// `main` could in principle need a different layout depending on when a
/// function happens to be called, which this backend has no way to express
/// per-call-site anyway (a function is emitted once, not once per call).
/// It uses whichever layout is current once every top-level statement has
/// been walked -- correct for the overwhelmingly common shape (each
/// channel `FIELD`ed once, used for the program's whole run), and a
/// reasonable, well-defined default otherwise.
///
/// `generate()` runs this against its own *separate*, throwaway
/// `FileIoLayout` (never the one used for the real, order-sensitive
/// top-level pass) -- replaying the exact same `FIELD` statements against
/// a single shared instance would double the generation counter for every
/// channel `FIELD`ed more than once, corrupting the synthesized helper
/// names the real pass computes (a real regression this had, caught by
/// `c_target_field_layout_tracks_program_order_not_last_field_wins`).
/// `generate()` then merges only this throwaway instance's already-emitted
/// helper text and dedup-tracking into the real one before the top-level
/// pass runs, so the two passes end up sharing helper functions by name
/// (deterministic, since both walk the identical `FIELD` sequence) without
/// either emitting a duplicate definition.
fn apply_field_layouts_before_functions(
    program: &Program,
    layout: &mut FileIoLayout,
) -> Result<(), String> {
    fn walk(statements: &[Stmt], layout: &mut FileIoLayout) -> Result<(), String> {
        for statement in statements {
            match &statement.kind {
                Statement::Field {
                    channel,
                    fields,
                    record_type,
                    string_fields,
                    field_types,
                } => {
                    apply_field_statement(channel, fields, record_type, string_fields, field_types, layout)?;
                }
                Statement::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    walk(then_body, layout)?;
                    walk(else_body, layout)?;
                }
                Statement::For { body, .. }
                | Statement::While { body, .. }
                | Statement::Do { body, .. } => {
                    walk(body, layout)?;
                }
                Statement::SelectCase {
                    cases, else_body, ..
                } => {
                    for case in cases {
                        walk(&case.body, layout)?;
                    }
                    walk(else_body, layout)?;
                }
                Statement::TryCatch {
                    try_body,
                    catch_body,
                    ..
                } => {
                    walk(try_body, layout)?;
                    walk(catch_body, layout)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    walk(&program.statements, layout)
}

/// Validates every function's shape up front and builds the lookup table
/// `render_numeric_expr`/`render_string_expr` use for `Expr::Call`.
/// Deliberately narrow, same "reject rather than emit wrong code" policy
/// as everywhere else in this backend: **procedures** (no return value)
/// aren't supported yet (only functions are -- see the module doc
/// comment); `byref` scalar parameters compile to a real C pointer plus
/// copy-in/copy-out (see `FnParam::by_ref`); an array parameter is
/// supported too, but only a rank-1 numeric one (`arr%(?)`/`arr%(100)`) --
/// a higher rank needs its inner axes' capacities fixed at compile time to
/// give the parameter's real C pointer type a shape at all (real C allows
/// only the outermost dimension of a multi-dimensional array parameter to
/// vary), which no BASCAL tutorial or test here actually needs, so it's
/// left unattempted; a string array parameter is rejected the same way. A
/// function whose body doesn't end with an explicit `return` on its last
/// top-level statement is rejected outright rather than guessing a
/// fallback value -- unlike the BASIC backend (which can fall back on
/// "whatever the shared result variable last held," matching real
/// MBASIC/BASCOM's own GOSUB-without-RETURN behavior), a real C function
/// falling off the end without `return`-ing a value is undefined behavior,
/// not a defined-if-surprising fallback.
fn build_function_table(functions: &[FunctionDef]) -> Result<(FunctionMap, HashMap<(TypeSuffix, String), FnSig>), String> {
    let mut table = FunctionMap::new();
    let mut methods = HashMap::new();
    for func in functions {
        // A `procedure` never carries a type suffix (enforced by the
        // parser) and has no return type at all -- a real `void` C
        // function. Everything else about it (parameters, body emission)
        // is shared with `function` below.
        let is_string = !func.is_procedure && func.name.suffix == Some(TypeSuffix::String);
        let numeric = if func.is_procedure {
            None
        } else {
            func.name.suffix.and_then(numeric_c_type)
        };
        if !func.is_procedure && !is_string && numeric.is_none() {
            return Err(format!(
                "function `{}` isn't supported by the minimal C backend yet -- give it an \
                 explicit numeric or string return type suffix",
                func.name
            ));
        }
        let mut params = Vec::with_capacity(func.params.len() + usize::from(func.receiver.is_some()));
        if let Some(receiver) = func.receiver {
            params.push(FnParam {
                c_name: c_var_name(&BasicIdent { name: "self".to_string(), suffix: Some(receiver) }, receiver),
                is_string: receiver == TypeSuffix::String,
                is_float: numeric_c_type(receiver).is_some_and(|(_, f)| f),
                default: None,
                suffix: receiver,
                by_ref: false,
                array: None,
            });
        }
        for param in &func.params {
            let by_ref = param.mode == ParamMode::ByRef;
            if let Some(axes) = &param.axes {
                if axes.len() != 1 {
                    return Err(format!(
                        "array parameters with more than one dimension aren't supported by the \
                         minimal C backend yet (`{}`'s parameter `{}` has {} dimensions)",
                        func.name,
                        param.name,
                        axes.len()
                    ));
                }
                let Some(suffix) = param.name.suffix else {
                    return Err(format!(
                        "array parameter `{}` of `{}` isn't supported by the minimal C backend \
                         yet -- give it an explicit numeric type suffix",
                        param.name, func.name
                    ));
                };
                let Some((_, is_float)) = numeric_c_type(suffix) else {
                    return Err(format!(
                        "string array parameters aren't supported by the minimal C backend yet \
                         (`{}`'s parameter `{}`) -- only numeric array parameters are",
                        func.name, param.name
                    ));
                };
                params.push(FnParam {
                    c_name: c_var_name(&param.name, suffix),
                    is_string: false,
                    is_float,
                    default: None,
                    suffix,
                    by_ref,
                    array: Some(ArrayParamInfo {
                        declared_capacity: axes[0].map(|n| n + 1),
                        byval_capacity: 0,
                    }),
                });
                continue;
            }
            let Some(suffix) = param.name.suffix else {
                return Err(format!(
                    "parameter `{}` of `{}` isn't supported by the minimal C backend yet -- give \
                     it an explicit type suffix",
                    param.name, func.name
                ));
            };
            let param_is_string = suffix == TypeSuffix::String;
            let param_numeric = numeric_c_type(suffix);
            if !param_is_string && param_numeric.is_none() {
                return Err(format!(
                    "parameter `{}` of `{}` isn't supported by the minimal C backend yet",
                    param.name, func.name
                ));
            }
            params.push(FnParam {
                c_name: c_var_name(&param.name, suffix),
                is_string: param_is_string,
                is_float: param_numeric.is_some_and(|(_, f)| f),
                default: param.default.clone(),
                suffix,
                by_ref,
                array: None,
            });
        }
        // A procedure may fall through to its end with no explicit
        // `return` at all -- real BASIC's own "implicit RETURN" rule for
        // PROCEDURE (see tutorial 14) -- unlike a function, which must
        // always produce a value on every path.
        if !func.is_procedure && !body_always_returns(&func.body) {
            return Err(format!(
                "function `{}` isn't supported by the minimal C backend yet -- its body must \
                 end with an explicit `return` as its last top-level statement (the minimal C \
                 backend doesn't attempt to reproduce BASIC's undefined-fallthrough behavior)",
                func.name
            ));
        }
        let sig = FnSig {
                c_name: func.receiver.map_or_else(
                    || function_c_name(&func.name),
                    |receiver| format!("{}_{}", function_c_name(&func.name), type_tag(receiver)),
                ),
                is_void: func.is_procedure,
                is_string,
                is_float: numeric.is_some_and(|(_, f)| f),
                params,
                result_suffix: func.name.suffix,
            };
        if let Some(receiver) = func.receiver {
            methods.insert((receiver, func.name.name.to_ascii_lowercase()), sig);
        } else {
            table.insert(fn_key(&func.name), sig);
        }
    }
    Ok((table, methods))
}

fn type_tag(suffix: TypeSuffix) -> char {
    match suffix {
        TypeSuffix::Integer => 'i',
        TypeSuffix::Long => 'l',
        TypeSuffix::Single => 'f',
        TypeSuffix::Double => 'd',
        TypeSuffix::String => 's',
    }
}

fn call_args_with_defaults<'a>(sig: &'a FnSig, args: &'a [Expr], name: &BasicIdent) -> Result<Vec<&'a Expr>, String> {
    if args.len() > sig.params.len() {
        return Err(format!("`{name}` expects {} argument(s), got {}", sig.params.len(), args.len()));
    }
    let mut result: Vec<&Expr> = args.iter().collect();
    for param in sig.params.iter().skip(args.len()) {
        let Some(default) = &param.default else {
            return Err(format!("`{name}` expects {} argument(s), got {}", sig.params.len(), args.len()));
        };
        result.push(default);
    }
    Ok(result)
}

/// Proves a function body always reaches a `return` on every path -- see
/// `build_function_table`'s doc comment for why this is enforced rather
/// than falling back to a default value. Only looks at the last
/// non-comment/blank statement (same "trailing" convention as
/// `ends_with_end`): if control reaches past everything before it, this
/// is the statement that decides the outcome, so nothing earlier can
/// matter. A plain `return` trivially qualifies; an `if`/`else` or
/// `select case`/`case else` qualifies only when *every* branch
/// (recursively) does too, and only when an `else`/`case else` is even
/// present -- an `if` with no `else` can always fall through. `for`/
/// `while`/`do` never qualify on their own (a loop might run zero times,
/// or `exit` before ever returning), matching every tutorial-07-style
/// function in practice: they all put their own trailing `return` after
/// the loop rather than relying on one inside it.
fn body_always_returns(body: &[Stmt]) -> bool {
    let last = body.iter().rev().find(|s| {
        !matches!(&***s, Statement::BlankLine | Statement::BlockComment(_))
            && !matches!(&***s, Statement::Raw(text) if text.trim_start().starts_with('\''))
    });
    match last.map(|s| &s.kind) {
        Some(Statement::Return { .. }) => true,
        Some(Statement::If {
            then_body,
            else_body,
            ..
        }) => {
            !else_body.is_empty()
                && body_always_returns(then_body)
                && body_always_returns(else_body)
        }
        Some(Statement::SelectCase {
            cases, else_body, ..
        }) => {
            !else_body.is_empty()
                && cases.iter().all(|clause| body_always_returns(&clause.body))
                && body_always_returns(else_body)
        }
        _ => false,
    }
}

/// Collects every identifier named by a `global` declaration anywhere in
/// `body` (any nesting depth) -- mirrors
/// `codegen_basic::collect_global_decl_names`'s recursive walk, adapted to
/// return full `BasicIdent`s (suffix included) rather than just name
/// strings, since the caller (`emit_function_def`) needs the suffix to
/// register each one in the right (numeric vs. string) global-variable
/// map.
fn collect_global_decl_idents(body: &[Stmt], out: &mut Vec<BasicIdent>) {
    for stmt in body {
        match &stmt.kind {
            Statement::GlobalDecl(ident) => out.push(ident.clone()),
            Statement::If {
                then_body,
                else_body,
                ..
            } => {
                collect_global_decl_idents(then_body, out);
                collect_global_decl_idents(else_body, out);
            }
            Statement::For { body, .. } | Statement::While { body, .. } => {
                collect_global_decl_idents(body, out);
            }
            Statement::Do { body, .. } => collect_global_decl_idents(body, out),
            Statement::SelectCase {
                cases, else_body, ..
            } => {
                for case in cases {
                    collect_global_decl_idents(&case.body, out);
                }
                collect_global_decl_idents(else_body, out);
            }
            _ => {}
        }
    }
}

/// `generate`'s output: a single, self-contained `.c` file, always. A
/// compiled program's own logic still isn't buried in runtime-support
/// boilerplate (ring-buffer helpers, `GOSUB`/error-handling/`RANDOMIZE`/
/// `DATA` state, record I/O helpers, `INSTR`/`SGN`/`RND`, screen-I/O ANSI
/// helpers, ...) -- but instead of splitting that boilerplate into a
/// separate paired `bcc_runtime.h` file (the design this replaced -- see
/// GitHub issue #28), every needed helper's one-line forward declaration
/// goes near the top of `app`, right after whatever state it needs, and
/// its full implementation is appended at the very end, after `main`.
/// Opening the file still shows the program itself first; there's just
/// one file to ever look at, write, or `gcc` -- no sibling file that can
/// go stale or get clobbered by a second program compiled into the same
/// directory (each compile fully regenerates its own single file, so
/// nothing is ever left inconsistent with the `.c` it came from). Only
/// the specific `bcc_*` helpers this particular program actually needs
/// appear at all (same per-feature gating as before).
pub(crate) struct GeneratedC {
    pub(crate) app: String,
}

pub(crate) fn generate(program: &Program) -> Result<GeneratedC, Vec<Diagnostic>> {
    let (mut funcs, methods) =
        build_function_table(&program.functions).map_err(|message| vec![unsupported(&message)])?;
    let int_consts = collect_top_level_int_consts(&program.statements);
    let arrays = collect_array_declarations(&program.statements, &int_consts)
        .map_err(|message| vec![unsupported(&message)])?;
    apply_byval_array_capacities(&mut funcs, program, &arrays)
        .map_err(|message| vec![unsupported(&message)])?;
    let functions = FunctionTable { funcs, methods, arrays };
    let try_reachable = collect_try_reachable_procedures(program, &functions);
    let top_level_const_names = collect_top_level_const_c_names(&program.statements);
    let known_layouts = known_record_layouts(program);
    let new_file_io = |known_record_layouts: HashMap<Vec<u32>, (String, Vec<bool>)>| FileIoLayout {
        channel_fields: HashMap::new(),
        field_widths: HashMap::new(),
        used: false,
        channel_record_type: HashMap::new(),
        known_record_layouts,
        record_helpers: std::collections::HashSet::new(),
        channel_generation: HashMap::new(),
        helper_defs: String::new(),
        helper_protos: String::new(),
        pending_field_values: HashMap::new(),
    };
    let mut file_io = new_file_io(known_layouts.clone());

    // BASIC variables "spring into existence on first use" -- there's no
    // separate declaration step to hook into, so every scalar variable
    // touched anywhere at top level (by `dim`, an assignment target, or a
    // read) is collected up front and declared, zero-initialized (for a
    // string, that means an all-zero buffer -- a valid, empty C string),
    // at file scope, regardless of where it's first mentioned in source
    // order. A top-level BASCAL variable IS a real global in exactly the
    // sense a BASIC one is, so file-scope `static` storage (visible to
    // every function, not just `main`) is the correct C shape for it --
    // unlike a function's own locals (see `emit_function_def`), which stay
    // genuinely scoped to that one C function, no name-mangling needed,
    // since C's own lexical scoping already gives BASCAL's
    // local-unless-declared-`global` function semantics for free. Also
    // folds in every variable named by a `global` declaration inside any
    // function body, even one no top-level statement ever touches --
    // otherwise it would never get a file-scope declaration to refer to at
    // all. This pass is deliberately infallible -- it only ever *adds* a
    // declaration for a variable shape it understands; anything it doesn't
    // (arrays, ...) is silently skipped here and reported as a real error
    // later, when `emit_statement`/`render_numeric_expr`/
    // `render_string_expr` actually tries to use it.
    let mut numeric_vars = BTreeMap::new();
    let mut string_vars = BTreeSet::new();
    for statement in &program.statements {
        collect_vars_in_statement(statement, &mut numeric_vars, &mut string_vars);
    }
    for func in &program.functions {
        let mut globals = Vec::new();
        collect_global_decl_idents(&func.body, &mut globals);
        for ident in &globals {
            register_var(ident, &mut numeric_vars, &mut string_vars);
        }
    }
    // A bare array name used as a call argument (`printArray%(data%)`) is,
    // syntactically, just `Expr::Ident("data%")` -- indistinguishable, to
    // `collect_vars_in_statement`'s own expression walk, from an ordinary
    // scalar read, since it has no idea `arrays` (built above) even exists.
    // Left alone, that would register a same-named *scalar* global
    // alongside the real array declaration below -- two conflicting C
    // declarations for the same identifier. Array names always win here:
    // a real `dim`'d array already gets its own declaration, so any
    // same-named scalar entry only this pass's own blind spot could have
    // produced is dropped.
    numeric_vars.retain(|k, _| !functions.arrays.contains_key(k));
    string_vars.retain(|k| !functions.arrays.contains_key(k));

    let mut needs_math = false;
    let mut needs_string = false;
    let mut temp_counter = 0;

    // A function/procedure body is emitted (below) before the top-level
    // `FIELD` declarations that establish the layout it needs -- see
    // `apply_field_layouts_before_functions`'s own doc comment. Pre-scan
    // into a *separate* throwaway layout, used only for function bodies,
    // rather than `file_io` itself: that instance's own doc comment
    // explains why sharing one would corrupt the real, order-sensitive
    // pass's generation counting.
    let mut function_view = new_file_io(known_layouts);
    apply_field_layouts_before_functions(program, &mut function_view)
        .map_err(|message| vec![unsupported(&message)])?;

    // Precomputed once, shared read-only by both the function-body pass
    // below and the real top-level pass further down -- see
    // `ErrorDataCtx`'s own doc comment. `raise_site_count` is a plain
    // `usize`, not part of the shared table, since only the real
    // top-level pass ever assigns raise-site IDs (`ON ERROR GOTO`/
    // `RESUME`/`ERROR` are rejected outright inside a function/procedure
    // body -- see `Statement::OnErrorGoto`'s own arm in `emit_statement`).
    let on_error_handler_ids = collect_on_error_handler_ids(&program.statements);
    let (data_items, data_labels) = collect_data_items_and_labels(&program.statements)
        .map_err(|message| vec![unsupported(&message)])?;
    let raise_site_count = count_raise_sites(&program.statements);

    // `emit_raise_block`'s own switch table: named `ON ERROR GOTO` targets
    // first (sorted by id, matching `on_error_handler_ids`), then one
    // entry per top-level `try`/`catch` block, in the same left-to-right,
    // depth-first program order `Statement::TryCatch`'s own arm in
    // `emit_statement` assigns `ctx.try_id` in -- see `ErrorDataCtx`'s own
    // doc comment for why a `try`'s id is always `on_error_handler_ids.
    // len() + <its position among try/catch blocks>`.
    let try_catch_count = count_try_catch_blocks(&program.statements);
    let dispatch_labels: Vec<String> = {
        let mut sorted: Vec<(&String, &usize)> = on_error_handler_ids.iter().collect();
        sorted.sort_by_key(|(_, id)| **id);
        let mut labels: Vec<String> = sorted
            .into_iter()
            .map(|(name, _)| format!("bcc_lbl_{name}"))
            .collect();
        for i in 0..try_catch_count {
            labels.push(format!("bcc_try_{}_catch", on_error_handler_ids.len() + i));
        }
        labels
    };

    // `bcc_data`/`bcc_read_data` (see `DATA_HELPER`) are only declared
    // below when `data_items` is non-empty -- a `READ` in a program with
    // no `DATA` at all would otherwise fail with an opaque "undeclared
    // `bcc_read_data`" C compile error instead of a clear diagnostic here.
    if data_items.is_empty()
        && program_has_statement(program, &|s| matches!(&**s, Statement::Read(_)))
    {
        return Err(vec![unsupported(
            "`read` found but the program has no `data` items at all -- real BASIC's own \
             \"Out of DATA\" error, caught here at compile time instead",
        )]);
    }

    let mut prototypes = String::new();
    let mut function_defs = String::new();
    for func in &program.functions {
        let sig = functions.signature_for(func).expect("function table should contain declaration");
        let is_try_reachable = try_reachable.contains(&fn_key(&func.name));
        prototypes.push_str(&function_signature(func, sig, is_try_reachable));
        prototypes.push_str(";\n");
        // A function/procedure with its own array parameter(s) needs a
        // *per-function* extended `FunctionTable` while its own body is
        // being emitted -- see `function_scoped_table`'s own doc comment
        // for why (in short: it registers each such parameter into the
        // same `arrays` lookup `render_lvalue`/`render_numeric_expr`/
        // `sizeof(...)` already use for a real top-level `dim`'d array,
        // so indexing/`sizeof` on a parameter "just works" with no extra
        // threading through any of those). Every other function still
        // uses the shared, un-extended `functions` table -- cheap to
        // check, since most functions have no array parameter at all.
        let local_arrays = collect_array_declarations(&func.body, &int_consts)
            .map_err(|message| vec![unsupported(&message)])?;
        let scoped_table = function_scoped_table(&functions, sig, &local_arrays);
        let table_for_body = scoped_table.as_ref().unwrap_or(&functions);
        emit_function_def(
            func,
            sig,
            table_for_body,
            &local_arrays,
            &mut function_defs,
            &mut needs_math,
            &mut needs_string,
            &mut temp_counter,
            &mut function_view,
            &on_error_handler_ids,
            &data_labels,
            &try_reachable,
            &top_level_const_names,
        )
        .map_err(|message| vec![unsupported(&message)])?;
    }
    // Fold in only the helper text and dedup-tracking function bodies
    // produced -- never `function_view`'s own layout state, which the real
    // pass below still needs to build up itself, live, in program order.
    // Safe to share by name: both passes walk the identical `FIELD`
    // sequence (`function_view`'s over `program.statements` alone, the
    // real pass's below over the same statements), so any helper name the
    // real pass computes for a channel's Nth distinct shape is guaranteed
    // to already be exactly what `function_view` computed for that same
    // Nth shape.
    file_io.used = function_view.used;
    file_io.record_helpers = function_view.record_helpers;
    file_io.helper_defs = function_view.helper_defs;
    file_io.helper_protos = function_view.helper_protos;

    let gosub_count = count_gosubs(&program.statements);
    let mut gosub_id: usize = 0;
    let mut ctx = ErrorDataCtx {
        handler_ids: &on_error_handler_ids,
        dispatch_labels: &dispatch_labels,
        raise_site_count,
        raise_id: 0,
        try_id: on_error_handler_ids.len(),
        data_labels: &data_labels,
        try_reachable: &try_reachable,
        current_function_reachable: false,
        current_try_catch: None,
    };
    let mut body = String::new();
    for statement in &program.statements {
        emit_statement(
            statement,
            &mut body,
            &mut needs_math,
            &mut needs_string,
            &mut temp_counter,
            &functions,
            None,
            &mut file_io,
            gosub_count,
            &mut gosub_id,
            &mut ctx,
        )
        .map_err(|message| vec![unsupported(&message)])?;
    }
    // `Statement::End` already emits its own `return 0;` -- only add the
    // implicit fallthrough one when the program didn't already end with an
    // explicit `end` (comments/blank lines don't count), otherwise `main`
    // would end in two `return 0;` statements back to back.
    if !ends_with_end(&program.statements) {
        body.push_str("    return 0;\n");
    }

    let builtin_usage = scan_builtin_usage(program);
    let needs_color = program_uses_color(program);
    let needs_input = program_uses_input(program);
    let needs_stop_or_system = program_has_statement(program, &|s| {
        matches!(&**s, Statement::Stop | Statement::System)
    });
    let needs_seq_io =
        builtin_usage.needs_seq_file_helper || program_uses_sequential_file_io(program);
    let needs_randomize = program_uses_randomize(program);
    let needs_randomize_time = program_uses_randomize_time(program);
    // A raise site (`ERROR`, or a sequential `OPEN ... FOR INPUT` that can
    // now fail with error 53) always needs the error-handling globals,
    // whether or not the program actually installs a handler (an
    // un-trapped raise still needs `bcc_on_error_target`/`bcc_in_handler`
    // to know it's un-trapped -- see `emit_raise_block`). `ON ERROR GOTO`/
    // `RESUME` alone (no raise site at all) is a degenerate but valid
    // program -- the trap would just never fire -- so it's covered here
    // too, or `bcc_on_error_target = ...`/the `switch (bcc_resume_id)`
    // dispatch would reference undeclared globals. Same story for `try`/
    // `catch` on its own: its own arm in `emit_statement` always touches
    // `bcc_on_error_target`/`bcc_in_handler`, even for a `try_body` with
    // no *direct* raise site of its own -- e.g. one that only calls a
    // `try`-reachable procedure, whose actual raise site lives inside
    // that procedure's body instead, uncounted by `raise_site_count`
    // (which only ever counts top-level raise sites).
    let needs_error_handling = raise_site_count > 0
        || program_has_statement(program, &|s| {
            matches!(
                &**s,
                Statement::OnErrorGoto { .. } | Statement::Resume(_) | Statement::TryCatch { .. }
            )
        });

    // <math.h> is only pulled in when something (currently just `\`) needs
    // round() from it, <string.h> only when a string `select case` needs
    // strcmp() from it, a LEN/ASC/CHR$/MID$/LEFT$ call needs strlen() (see
    // `scan_builtin_usage`), `input`/sequential file I/O need `strcspn`
    // (see `INPUT_HELPER`/`SEQ_FILE_HELPER`), or random-access record I/O
    // needs memcpy() (see `FILE_IO_HELPER`), and <stdint.h> only for that
    // same record I/O's exact-width `int16_t`/`int32_t` packing -- most
    // programs won't need any of them.
    let mut includes = String::from("#include <stdio.h>\n");
    if needs_math {
        includes.push_str("#include <math.h>\n");
    }
    if needs_string || builtin_usage.needs_string_h || needs_input || needs_seq_io || file_io.used
    {
        includes.push_str("#include <string.h>\n");
    }
    if file_io.used {
        includes.push_str("#include <stdint.h>\n");
        includes.push_str("#include <stdlib.h>\n");
    } else if builtin_usage.needs_stdlib_h
        || needs_input
        || needs_randomize
        || needs_error_handling
        || needs_stop_or_system
        || needs_color
        || !data_items.is_empty()
    {
        // `input`'s numeric targets parse via `atoi`/`atof` (see
        // `INPUT_HELPER`'s call site in `emit_statement`); `RANDOMIZE`
        // needs `srand()` (see `Statement::Randomize`'s own handling in
        // `emit_statement`), independent of whether `RND` itself
        // (`builtin_usage.needs_stdlib_h`) is ever called; an untrapped
        // raise site's fatal path needs `exit()` (see `emit_raise_block`);
        // `READ`'s numeric targets parse via `atoi`/`atof` too, and
        // `DATA_HELPER`'s own out-of-DATA path needs `exit()`; `STOP`/
        // `SYSTEM` need `exit()` too (see their own arm in
        // `emit_statement`); `COLOR` needs `atexit()` (see `COLOR_BODY`'s
        // own doc comment).
        includes.push_str("#include <stdlib.h>\n");
    }
    if needs_randomize_time {
        includes.push_str("#include <time.h>\n");
    }
    if builtin_usage.needs_inkey_helper {
        includes.push_str("#include <termios.h>\n");
        includes.push_str("#include <unistd.h>\n");
    }

    let mut globals_decl = String::new();
    for (c_name, c_type) in &numeric_vars {
        globals_decl.push_str(&format!("static {c_type} {c_name} = 0;\n"));
    }
    for c_name in &string_vars {
        globals_decl.push_str(&format!(
            "static char {c_name}[{STRING_BUFFER_SIZE}] = {{0}};\n"
        ));
    }
    // `dim`-declared arrays -- a real, native multi-dimensional C array
    // (`int grid[10][10]`, not a manually-flattened, hand-strided single
    // dimension), one `[N + 1]` per axis matching real BASIC's own
    // inclusive-bound convention (see `ArrayInfo`'s own doc comment).
    // Sorted by name for the same deterministic-output reason
    // `numeric_vars`/`string_vars` already are (`BTreeMap`/`BTreeSet`
    // there; `arrays` is a plain `HashMap`, so sorted here instead).
    let mut sorted_arrays: Vec<(&String, &ArrayInfo)> = functions.arrays.iter().collect();
    sorted_arrays.sort_by_key(|(name, _)| name.as_str());
    for (c_name, info) in sorted_arrays {
        let dims: String = info
            .bounds
            .iter()
            .map(|bound| format!("[{}]", bound + 1))
            .collect();
        match info.element_type {
            Some((c_type, _)) => {
                globals_decl.push_str(&format!("static {c_type} {c_name}{dims} = {{0}};\n"));
            }
            None => {
                globals_decl.push_str(&format!(
                    "static char {c_name}{dims}[{STRING_BUFFER_SIZE}] = {{0}};\n"
                ));
            }
        }
    }

    // Every `bcc_*` runtime-support helper this program actually needs,
    // gated exactly as before -- but split three ways instead of pushed
    // into one `runtime` accumulator kept in a separate file: `state`
    // (macros/static data a helper's *own* function bodies need, plus
    // anything the user's emitted code itself references by name, e.g.
    // `bcc_data_ptr`/`bcc_files`/`bcc_input_buf` -- see each `_STATE`
    // constant's own doc comment) goes right after `includes`, before
    // anything that could reference it; `protos` (one-line forward
    // declarations) goes right after `state`, so `main`/the user's own
    // functions can call a helper by name even though its *body* won't
    // appear until after them; `body` (the actual implementations) is
    // appended at the very end of `app`, after `main`, so opening the
    // generated file shows the user's own program first -- see GitHub
    // issue #28, and the "helper functions declared at the top,
    // implementations at the bottom" design this replaced the old
    // paired-`bcc_runtime.h`-file split with.
    let mut runtime_state = String::new();
    let mut runtime_protos = String::new();
    let mut runtime_body = String::new();
    if builtin_usage.needs_ring_buffer_helpers {
        runtime_state.push_str(MID_STATE);
        runtime_protos.push_str(MID_PROTOS);
        runtime_body.push_str(MID_BODY);
    }
    if builtin_usage.needs_instr_helper {
        runtime_protos.push_str(INSTR_PROTO);
        runtime_body.push_str(INSTR_BODY);
    }
    if builtin_usage.needs_sgn_helper {
        runtime_protos.push_str(SGN_PROTO);
        runtime_body.push_str(SGN_BODY);
    }
    if builtin_usage.needs_rnd_helper {
        runtime_protos.push_str(RND_PROTO);
        runtime_body.push_str(RND_BODY);
    }
    if builtin_usage.needs_inkey_helper {
        runtime_protos.push_str(INKEY_PROTO);
        runtime_body.push_str(INKEY_BODY);
    }
    if gosub_count > 0 {
        runtime_state.push_str(GOSUB_HELPER);
    }
    if needs_error_handling {
        runtime_state.push_str(ERROR_HANDLING_GLOBALS);
    }
    if !try_reachable.is_empty() {
        runtime_state.push_str(TRY_RESULT_TYPE);
    }
    // `bcc_data_ptr` is the only piece of `DATA`/`READ`/`RESTORE` state
    // the user's own emitted code touches directly (`RESTORE` assigns it
    // straight by name -- see `Statement::Restore`'s own arm), so it's
    // the only piece that has to live in `state`. `bcc_data[]` itself
    // (and its `BCC_DATA_COUNT`) is only ever touched from inside
    // `bcc_read_data`, so it moves down alongside that function's own
    // body instead, right before `DATA_BODY`.
    if !data_items.is_empty() {
        runtime_state.push_str(DATA_STATE);
        runtime_protos.push_str(DATA_PROTO);
        runtime_body.push_str(&format!("#define BCC_DATA_COUNT {}\n", data_items.len()));
        runtime_body.push_str(&format!(
            "static const char* bcc_data[BCC_DATA_COUNT] = {{ {} }};\n\n",
            data_items.join(", ")
        ));
        runtime_body.push_str(DATA_BODY);
    }
    if file_io.used {
        runtime_state.push_str(FILE_IO_STATE);
        runtime_protos.push_str(FILE_IO_PROTOS);
        runtime_protos.push_str(&file_io.helper_protos);
        runtime_body.push_str(FILE_IO_BODY);
        runtime_body.push_str(&file_io.helper_defs);
    }
    if needs_seq_io {
        runtime_state.push_str(SEQ_FILE_STATE);
        runtime_protos.push_str(SEQ_FILE_PROTOS);
        runtime_body.push_str(SEQ_FILE_BODY);
    }
    if needs_color {
        runtime_protos.push_str(COLOR_PROTO);
        runtime_body.push_str(COLOR_BODY);
    }
    if needs_input {
        runtime_state.push_str(INPUT_STATE);
        runtime_protos.push_str(INPUT_PROTO);
        runtime_body.push_str(INPUT_BODY);
    }

    let mut app = includes;
    app.push('\n');
    if !runtime_state.is_empty() {
        app.push_str(&runtime_state);
    }
    if !runtime_protos.is_empty() {
        app.push_str(&runtime_protos);
        app.push('\n');
    }
    if !globals_decl.is_empty() {
        app.push_str(&globals_decl);
        app.push('\n');
    }
    if !prototypes.is_empty() {
        app.push_str(&prototypes);
        app.push('\n');
    }
    app.push_str(&function_defs);
    if builtin_usage.needs_inkey_helper {
        // `INPUT`'s own `bcc_read_line` (`fgets`) reads through C's
        // normal *buffered* stdio, which slurps ahead into its own
        // internal buffer -- bytes past what one `fgets` call needed
        // become invisible to `bcc_inkey`'s raw `read(STDIN_FILENO, ...)`
        // syscall on the same fd, since that bypasses stdio's buffer
        // entirely. Disabling stdin buffering here keeps both reading
        // styles consistent (every `fgets` then does its own raw reads
        // too), so a program mixing `INKEY$` with `INPUT` -- like
        // tutorial/inventory.bcl's own menu-key-then-part-number flow --
        // doesn't silently lose keystrokes typed right after an `INPUT`
        // line into a buffer `INKEY$` can never see.
        body.insert_str(0, "    setvbuf(stdin, NULL, _IONBF, 0);\n");
    }
    app.push_str(&format!(
        "int main(void) {{\n{}}}\n",
        reindent_c_body(&body)
    ));
    if !runtime_body.is_empty() {
        app.push('\n');
        app.push_str(&runtime_body);
    }

    Ok(GeneratedC { app })
}

/// One function's C prototype/definition header -- `<ret> <name>(<params>)`,
/// no trailing `;`/body, shared by the forward-declaration pass and
/// `emit_function_def` so the two can never drift out of sync with each
/// other. A string-returning function is actually `void`-returning in C
/// (see the module doc comment/`emit_function_def`): its BASCAL return
/// value comes out through an extra trailing `char* bcc_out` parameter
/// instead, matching the buffer-out convention already used by every
/// other string value in this backend.
///
/// A byval string parameter's C parameter is `const char*` named
/// `<c_name>_in` -- not `<c_name>` itself -- because the function body
/// needs its own byval-copied local under the plain `<c_name>` (see
/// `emit_function_def`'s copy-in preamble): aliasing the parameter
/// directly would let the callee mutate the caller's buffer, breaking
/// byval semantics. A `byref` scalar parameter (numeric or string) is
/// named `<c_name>_in` for the identical reason, just one level removed:
/// it's a real C pointer (`T*`/`char*`), and the body still needs its own
/// plain-named local to read/write normally -- copied in from `*<c_name>_in`
/// at function entry and written back at every `return` (see
/// `emit_byref_scalar_copyback`). An array parameter (`byval` or `byref`
/// alike) gets a second, hidden `int <c_name>_len0` parameter carrying its
/// real element count (see `FnParam::array`'s own doc comment); its first
/// parameter is named `<c_name>_in` only for `byval` (which, like a byval
/// string, still needs its own local copy -- see `emit_function_def`) --
/// a `byref` array parameter has no local copy to alias away from at all
/// (a real pointer already gives the callee live access to the caller's
/// own storage), so its incoming pointer is simply named `<c_name>`
/// directly, used as-is throughout the body.
fn function_signature(func: &FunctionDef, sig: &FnSig, is_try_reachable: bool) -> String {
    let ret_type = if is_try_reachable {
        // Only ever true alongside `sig.is_void` -- see
        // `collect_called_procedures`, which only ever adds a real
        // `procedure` to the reachable set.
        "bcc_result_void"
    } else if sig.is_void || sig.is_string {
        "void"
    } else {
        numeric_c_type(func.name.suffix.expect("validated by build_function_table"))
            .expect("validated by build_function_table")
            .0
    };
    let mut params: Vec<String> = Vec::new();
    for fp in &sig.params {
        if fp.array.is_some() {
            let (c_type, _) = numeric_c_type(fp.suffix).expect("validated by build_function_table");
            let pointer_name = if fp.by_ref {
                fp.c_name.clone()
            } else {
                format!("{}_in", fp.c_name)
            };
            params.push(format!("{c_type}* {pointer_name}, int {}_len0", fp.c_name));
        } else if fp.by_ref {
            if fp.is_string {
                params.push(format!("char* {}_in", fp.c_name));
            } else {
                let (c_type, _) = numeric_c_type(fp.suffix).expect("validated by build_function_table");
                params.push(format!("{c_type}* {}_in", fp.c_name));
            }
        } else if fp.is_string {
            params.push(format!("const char* {}_in", fp.c_name));
        } else {
            let (c_type, _) = numeric_c_type(fp.suffix).expect("validated by build_function_table");
            params.push(format!("{c_type} {}", fp.c_name));
        }
    }
    if sig.is_string {
        params.push("char* bcc_out".to_string());
    }
    let params_text = if params.is_empty() {
        "void".to_string()
    } else {
        params.join(", ")
    };
    format!("{ret_type} {}({params_text})", sig.c_name)
}

/// Emits one function's full C definition -- signature (via
/// `function_signature`) plus `{ ...body... }`.
///
/// The body's own local variables are collected the same way `generate`
/// collects `main`'s (every scalar touched anywhere in the body), but
/// then two categories are subtracted before declaring them: this
/// function's own parameters (already declared by the signature -- for a
/// string parameter, `c_name` denotes its byval-copy local, declared and
/// initialized from the incoming `<c_name>_in` pointer right here, before
/// any other local) and every name this function declared `global` (see
/// `collect_global_decl_idents`) -- those already have a file-scope
/// declaration from `generate`, and declaring a same-named local here
/// would shadow it instead of referring to it, breaking the whole point
/// of `global`.
fn emit_function_def(
    func: &FunctionDef,
    sig: &FnSig,
    functions: &FunctionTable,
    local_arrays: &ArrayTable,
    out: &mut String,
    needs_math: &mut bool,
    needs_string: &mut bool,
    temp_counter: &mut usize,
    file_io: &mut FileIoLayout,
    handler_ids: &HashMap<String, usize>,
    data_labels: &HashMap<String, usize>,
    try_reachable: &HashSet<(String, Option<TypeSuffix>)>,
    top_level_const_names: &BTreeSet<String>,
) -> Result<(), String> {
    let is_try_reachable = try_reachable.contains(&fn_key(&func.name));
    let mut numeric_locals = BTreeMap::new();
    let mut string_locals = BTreeSet::new();
    for stmt in &func.body {
        collect_vars_in_statement(stmt, &mut numeric_locals, &mut string_locals);
    }
    let mut global_idents = Vec::new();
    collect_global_decl_idents(&func.body, &mut global_idents);
    let global_keys: BTreeSet<String> = global_idents
        .iter()
        .map(|ident| c_var_name(ident, effective_suffix(ident.suffix)))
        .collect();
    let param_keys: BTreeSet<String> = sig.params.iter().map(|p| p.c_name.clone()).collect();
    numeric_locals.retain(|k, _| {
        !param_keys.contains(k) && !global_keys.contains(k) && !top_level_const_names.contains(k)
    });
    string_locals.retain(|k| {
        !param_keys.contains(k) && !global_keys.contains(k) && !top_level_const_names.contains(k)
    });

    let mut body = String::new();
    for fp in &sig.params {
        if let Some(arr) = &fp.array {
            // A `byref` array parameter needs no local buffer at all -- its
            // incoming pointer, named plainly `<c_name>` (see
            // `function_signature`), already gives live read/write access
            // to the caller's own storage, exactly matching real BASIC's
            // own byref array semantics. A `byval` one gets its own local
            // copy, sized to fit the largest call site this program could
            // resolve (see `apply_byval_array_capacities`), filled in from
            // the incoming pointer up to its real element count (the
            // hidden `<c_name>_len0` parameter) -- so later writes inside
            // the function body never reach the caller's own array.
            if !fp.by_ref {
                let (c_type, _) =
                    numeric_c_type(fp.suffix).expect("validated by build_function_table");
                body.push_str(&format!(
                    "    {c_type} {0}[{1}] = {{0}};\n",
                    fp.c_name, arr.byval_capacity
                ));
                body.push_str(&format!(
                    "    for (int bcc_i = 0; bcc_i < {0}_len0; bcc_i++) {{ {0}[bcc_i] = \
                     {0}_in[bcc_i]; }}\n",
                    fp.c_name
                ));
            }
        } else if fp.is_string {
            body.push_str(&format!("    char {0}[{STRING_BUFFER_SIZE}];\n", fp.c_name));
            body.push_str(&format!(
                "    snprintf({0}, sizeof({0}), \"%s\", {0}_in);\n",
                fp.c_name
            ));
        } else if fp.by_ref {
            // A `byref` scalar parameter's own copy-in: the body reads/
            // writes the plain local `<c_name>` completely normally (no
            // dereferencing anywhere else in this file needs to change at
            // all) -- `emit_byref_scalar_copyback` writes it back through
            // `<c_name>_in` at every `return`.
            let (c_type, _) = numeric_c_type(fp.suffix).expect("validated by build_function_table");
            body.push_str(&format!("    {c_type} {0} = *{0}_in;\n", fp.c_name));
        }
    }
    for (c_name, c_type) in &numeric_locals {
        body.push_str(&format!("    {c_type} {c_name} = 0;\n"));
    }
    for c_name in &string_locals {
        body.push_str(&format!(
            "    char {c_name}[{STRING_BUFFER_SIZE}] = {{0}};\n"
        ));
    }
    // A local `dim name(...)` array -- declared as a real true-C-local
    // array, right here, exactly like `numeric_locals`/`string_locals`
    // above (never hoisted to file scope the way a top-level `dim`'d one
    // is -- see `collect_array_declarations`'s own doc comment and
    // `generate`'s call site). Sorted by name for the same deterministic-
    // output reason the top-level globals loop already is.
    let mut sorted_local_arrays: Vec<(&String, &ArrayInfo)> = local_arrays.iter().collect();
    sorted_local_arrays.sort_by_key(|(name, _)| name.as_str());
    for (c_name, info) in &sorted_local_arrays {
        let dims: String = info
            .bounds
            .iter()
            .map(|bound| format!("[{}]", bound + 1))
            .collect();
        match info.element_type {
            Some((c_type, _)) => {
                body.push_str(&format!("    {c_type} {c_name}{dims} = {{0}};\n"));
            }
            None => {
                body.push_str(&format!(
                    "    char {c_name}{dims}[{STRING_BUFFER_SIZE}] = {{0}};\n"
                ));
            }
        }
    }
    if sig.params.iter().any(|p| p.is_string || p.array.is_some() || p.by_ref)
        || !numeric_locals.is_empty()
        || !string_locals.is_empty()
        || !sorted_local_arrays.is_empty()
    {
        body.push('\n');
    }

    // GOSUB is scoped to top-level code only (see `Statement::Gosub`'s own
    // doc comment) -- a function/procedure body never legally reaches the
    // GOSUB arm's `gosub_id`/`gosub_count` reads at all, since it errors
    // out immediately whenever `current_function` is `Some` (always true
    // here), so a throwaway `0`/local counter is safe. Same story for
    // `ON ERROR GOTO`/`RESUME`/`ERROR`/`try`/`catch` and `ctx.raise_id`/
    // `ctx.raise_site_count`/`ctx.try_id`/`ctx.dispatch_labels` -- all are
    // rejected outright inside a function/procedure body too (see their
    // own arms in `emit_statement`), so throwaway values are safe.
    // `handler_ids`/`data_labels` are real, though, not thrown away: `READ`/
    // `RESTORE` (unlike the error-handling trio) work fine inside a
    // function/procedure body, since `bcc_data`/`bcc_data_ptr` are plain
    // file-scope globals reachable from anywhere -- `RESTORE <label>`
    // there needs the real label table to resolve correctly.
    let mut unused_gosub_id: usize = 0;
    let mut ctx = ErrorDataCtx {
        handler_ids,
        dispatch_labels: &[],
        raise_site_count: 0,
        raise_id: 0,
        try_id: 0,
        data_labels,
        try_reachable,
        current_function_reachable: is_try_reachable,
        current_try_catch: None,
    };
    for stmt in &func.body {
        emit_statement(
            stmt,
            &mut body,
            needs_math,
            needs_string,
            temp_counter,
            functions,
            Some(sig),
            file_io,
            0,
            &mut unused_gosub_id,
            &mut ctx,
        )?;
    }

    // Covers a `procedure`'s implicit fallthrough return (no explicit
    // trailing `return` at all -- see `Statement::ReturnVoid`'s own arm in
    // `emit_statement`, which only handles an *explicit* one, wherever it
    // appears in the body). A `function`, by contrast, always ends with an
    // explicit `return` on every path (enforced by `body_always_returns`
    // in `build_function_table`), so this is unreachable there -- still
    // harmless to emit (plain `gcc`, with no `-Wall`/`-Werror`, doesn't
    // reject or even warn about trailing unreachable code -- see
    // `invoke_gcc` in `main.rs`), and simpler than trying to prove it's
    // dead first just to skip it.
    emit_byref_scalar_copyback(sig, &mut body);
    // A `try`-reachable procedure's own C return type is `bcc_result_void`
    // now (see `function_signature`), not `void` -- unlike a plain
    // `procedure`, falling off the end here without an explicit `return`
    // is real undefined behavior in C, not a free, do-nothing exit, so
    // this always needs its own explicit trailing success return.
    if is_try_reachable {
        body.push_str("    return (bcc_result_void){ .status = 0 };\n");
    }

    out.push_str(&function_signature(func, sig, is_try_reachable));
    out.push_str(&format!(" {{\n{}}}\n\n", reindent_c_body(&body)));
    Ok(())
}

/// Writes every `byref` *scalar* parameter's current value back through
/// its own incoming pointer (`<c_name>_in`) -- the other half of the
/// copy-in/copy-out convention `emit_function_def`'s own prologue sets up
/// (see `FnParam::by_ref`'s doc comment). Called at every real exit point
/// of a function/procedure body: `Statement::Return`'s own arm, the
/// `current_function.is_some()` branch of `Statement::ReturnVoid`'s own
/// arm (a `procedure`'s explicit bare `return`, wherever in the body it
/// appears), and once more at the very end of `emit_function_def` itself
/// (a `procedure`'s *implicit* fallthrough return -- see its own call
/// site's comment). A `byref` *array* parameter needs no such copyback at
/// all -- it was never copied into a local buffer in the first place (see
/// `emit_function_def`'s own prologue), so every write already landed
/// directly in the caller's own storage, live, the moment it happened.
fn emit_byref_scalar_copyback(sig: &FnSig, out: &mut String) {
    for fp in &sig.params {
        if fp.array.is_some() || !fp.by_ref {
            continue;
        }
        if fp.is_string {
            out.push_str(&format!(
                "    snprintf({0}_in, {STRING_BUFFER_SIZE}, \"%s\", {0});\n",
                fp.c_name
            ));
        } else {
            out.push_str(&format!("    *{0}_in = {0};\n", fp.c_name));
        }
    }
}

/// Re-indents an already-fully-generated C body per its actual nesting
/// depth. Every `emit_*`/`render_*` function above pushes each of its own
/// lines flush against the same flat 4-space prefix, regardless of how
/// deeply nested the construct producing it is -- threading an indent-depth
/// counter through every one of those call sites (`emit_statement` alone
/// recurses into itself from five different arms) would be a lot of
/// mechanical churn for a purely cosmetic property. Doing it as one pass
/// over the finished text instead, walking `{`/`}` block structure line by
/// line, gets the same result far more cheaply: a line starting with `}`
/// dedents *before* printing (so it lines up with the construct it closes,
/// e.g. `} else {`), otherwise it prints at the current depth; then the
/// line's own net brace balance (`brace_delta`) updates depth for the next
/// line. Depth starts at 1 -- every line here is already inside `main`'s
/// own `{ ... }`, added by `generate`'s caller after this returns.
///
/// Two things a naive version of this would get wrong: a `{`/`}` inside a
/// C string literal (a `print`ed BASCAL string can itself contain either
/// character, e.g. `print "{x}"` becomes `printf("{x}\n")`) must not be
/// mistaken for block structure -- `brace_delta` tracks string-literal
/// state (respecting `\"`/`\\` escapes) to skip those. And a whole-line
/// `//` comment (every comment this backend emits is one -- see
/// `Statement::BlockComment`/`Statement::Raw`) is skipped outright rather
/// than scanned for braces/quotes: comment text is arbitrary BASIC source
/// text, not C, and could contain an unbalanced `{`, `}`, or `"` of its
/// own that would otherwise desync every line indented after it.
fn reindent_c_body(raw: &str) -> String {
    let mut depth: usize = 1;
    let mut out = String::with_capacity(raw.len());
    for raw_line in raw.lines() {
        let content = raw_line.trim();
        if content.is_empty() {
            out.push('\n');
            continue;
        }
        if content.starts_with("//") {
            out.push_str(&"    ".repeat(depth));
            out.push_str(content);
            out.push('\n');
            continue;
        }
        let leading_close = content.starts_with('}');
        let print_depth = if leading_close {
            depth.saturating_sub(1)
        } else {
            depth
        };
        out.push_str(&"    ".repeat(print_depth));
        out.push_str(content);
        out.push('\n');
        depth = (depth as isize + brace_delta(content)).max(0) as usize;
    }
    out
}

/// Net `{`/`}` balance of one already-generated C line, ignoring either
/// character where it instead appears inside a C string literal (`"..."`,
/// respecting `\"`/`\\` escapes) -- see `reindent_c_body`.
fn brace_delta(line: &str) -> isize {
    let mut delta = 0isize;
    let mut in_string = false;
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if in_string {
            match c {
                '\\' => {
                    chars.next();
                }
                '"' => in_string = false,
                _ => {}
            }
        } else {
            match c {
                '"' => in_string = true,
                '{' => delta += 1,
                '}' => delta -= 1,
                _ => {}
            }
        }
    }
    delta
}

/// The C identifier a BASIC scalar variable maps to. Prefixed (`bv_...`,
/// "bascal variable") so a BASIC name that happens to collide with a C
/// keyword (a variable named `int`, say) can't produce invalid C, and
/// tagged with `suffix` so `x%`/`x&`/`x!`/`x#` -- distinct BASIC
/// variables, despite sharing a base name -- map to distinct C names
/// instead of colliding. Lowercases the name first since BASIC identifiers
/// are case-insensitive; BASCAL source can't contain an underscore in an
/// identifier (rejected at parse time, see `reject_underscored_identifiers`
/// in `lib.rs`), so the lowercased name alone is already collision-free as
/// a C identifier fragment.
/// The concrete numeric type a *scalar variable reference* has, filling
/// in real MBASIC/BASCOM's own default when BASCAL source leaves the
/// suffix off entirely (`i` in `for i = 3 downto 1`, not `i%`/`i!`/etc.).
/// Real BASIC's default is single-precision floating point, overridable
/// per-first-letter with `DEFINT`/`DEFSNG`/`DEFDBL`/`DEFLNG`/`DEFSTR` --
/// but BASCAL exposes no such statement to `.bcl` authors at all (checked
/// directly: no `DEFxxx` keyword anywhere in `parser.rs`), so a
/// BASCAL-generated `.bas` file's suffixless variables always fall back
/// to that same single, un-overridden default when run under real
/// BASCOM -- making `Single` the one and only correct fill-in here, not
/// a guess among several real possibilities. Scoped to variable
/// references only (`Ident`/`Dim`/`For`/assignment targets) -- function/
/// parameter *declarations* still require an explicit suffix (see
/// `build_function_table`), since nothing here needs that relaxed too.
fn effective_suffix(suffix: Option<TypeSuffix>) -> TypeSuffix {
    suffix.unwrap_or(TypeSuffix::Single)
}

fn c_var_name(ident: &BasicIdent, suffix: TypeSuffix) -> String {
    let tag = match suffix {
        TypeSuffix::Integer => 'i',
        TypeSuffix::Long => 'l',
        TypeSuffix::Single => 'f',
        TypeSuffix::Double => 'd',
        TypeSuffix::String => 's',
    };
    format!("bv_{tag}_{}", ident.name.to_ascii_lowercase())
}

/// The `functions.arrays` lookup key for a `dim`-declared array's own
/// name -- `c_var_name` with the same is-it-a-string-suffix choice
/// `collect_array_declarations_into` made when it first declared this
/// array, so a later indexed read/write agrees with the declaration on
/// exactly which key names it.
fn array_c_name(ident: &BasicIdent) -> String {
    let suffix = if ident.suffix == Some(TypeSuffix::String) {
        TypeSuffix::String
    } else {
        effective_suffix(ident.suffix)
    };
    c_var_name(ident, suffix)
}

/// Renders `name(idx1[, idx2, ...])` -- a `dim`-declared array's own
/// indexed element -- as the matching native C indexing expression
/// (`bv_i_name[idx1][idx2]`, real C multi-dimensional indexing, not
/// manual stride arithmetic -- see the array declaration itself in
/// `generate`). Used identically for a read (`render_numeric_expr`/
/// `render_string_expr`'s own `Expr::ArrayRef` arms) and a write
/// (`Statement::Assignment`'s array-target arm) -- both just need the
/// same lvalue-shaped C expression text. Each index is coerced to `int`
/// via a narrowing cast, matching real BASIC's own array subscripts
/// (always evaluated as integers); out-of-declared-bounds indexing isn't
/// checked at either compile or run time, the same unchecked-range-style
/// gap this backend already accepts everywhere else (see `ASC`'s own
/// doc comment in `render_numeric_call`).
fn render_array_index_expr(
    name: &BasicIdent,
    indices: &[Expr],
    needs_math: &mut bool,
    functions: &FunctionTable,
) -> Result<String, String> {
    let key = array_c_name(name);
    let info = functions
        .arrays
        .get(&key)
        .expect("caller already checked functions.arrays.contains_key(&array_c_name(name))");
    if indices.len() != info.bounds.len() {
        return Err(format!(
            "`{name}` is a {}-dimensional array, but this reference has {} index/indices",
            info.bounds.len(),
            indices.len()
        ));
    }
    let mut out = key;
    for index in indices {
        let (index_text, index_is_float) = render_numeric_expr(index, needs_math, functions)?;
        let index_text = if index_is_float {
            *needs_math = true;
            format!("((int)round((double)({index_text})))")
        } else {
            format!("({index_text})")
        };
        out.push('[');
        out.push_str(&index_text);
        out.push(']');
    }
    Ok(out)
}

/// The C type and `printf`/`render_numeric_expr` float-ness for a numeric
/// scalar variable's suffix. `%`/`&` (BASIC's 16-bit integer and 32-bit
/// long) are deliberately collapsed to the same plain C `int`/`%d` bucket
/// -- the same simplification this backend already makes for arithmetic
/// results (see `IntDiv`/`Mod` above), rather than introducing a `long`/
/// `%ld` type-tracking dimension for one BASIC type that's rarely
/// distinguished from `%` in practice. Returns `None` for `$`/no suffix --
/// `$` is a real, supported type, just not a *numeric* one, so it's
/// handled by a separate path (`render_string_expr`/`emit_assignment`'s
/// string branch), not here; suffixless (default-type) variables aren't
/// supported at all yet.
fn numeric_c_type(suffix: TypeSuffix) -> Option<(&'static str, bool)> {
    match suffix {
        TypeSuffix::Integer | TypeSuffix::Long => Some(("int", false)),
        TypeSuffix::Single => Some(("float", true)),
        TypeSuffix::Double => Some(("double", true)),
        TypeSuffix::String => None,
    }
}

fn collect_vars_in_statement(
    statement: &Stmt,
    numeric_out: &mut BTreeMap<String, &'static str>,
    string_out: &mut BTreeSet<String>,
) {
    match &statement.kind {
        Statement::Dim {
            name,
            is_array: false,
            sizes,
        } if sizes.is_empty() => {
            register_var(name, numeric_out, string_out);
        }
        Statement::Assignment {
            target: Expr::Ident(name),
            value,
        }
        | Statement::Const { name, value } => {
            register_var(name, numeric_out, string_out);
            collect_vars_in_expr(value, numeric_out, string_out);
        }
        Statement::Print { tokens } => {
            for token in tokens {
                if let PrintToken::Expr(expr) = token {
                    collect_vars_in_expr(expr, numeric_out, string_out);
                }
            }
        }
        Statement::Input { vars, .. } => {
            for var in vars {
                collect_vars_in_expr(var, numeric_out, string_out);
            }
        }
        Statement::Open { file, .. } => {
            collect_vars_in_expr(file, numeric_out, string_out);
        }
        Statement::LineInput { target, .. } => {
            collect_vars_in_expr(target, numeric_out, string_out);
        }
        Statement::PrintFile { tokens, .. } => {
            for token in tokens {
                if let PrintToken::Expr(expr) = token {
                    collect_vars_in_expr(expr, numeric_out, string_out);
                }
            }
        }
        Statement::InputFile { vars, .. } => {
            for var in vars {
                collect_vars_in_expr(var, numeric_out, string_out);
            }
        }
        Statement::Write { exprs, .. } => {
            for expr in exprs {
                collect_vars_in_expr(expr, numeric_out, string_out);
            }
        }
        Statement::Locate { row, col } => {
            collect_vars_in_expr(row, numeric_out, string_out);
            collect_vars_in_expr(col, numeric_out, string_out);
        }
        Statement::Color { fg, bg } => {
            collect_vars_in_expr(fg, numeric_out, string_out);
            if let Some(bg) = bg {
                collect_vars_in_expr(bg, numeric_out, string_out);
            }
        }
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            collect_vars_in_expr(condition, numeric_out, string_out);
            for stmt in then_body {
                collect_vars_in_statement(stmt, numeric_out, string_out);
            }
            for stmt in else_body {
                collect_vars_in_statement(stmt, numeric_out, string_out);
            }
        }
        Statement::For {
            var,
            start,
            end,
            step,
            body,
        } => {
            register_var(var, numeric_out, string_out);
            collect_vars_in_expr(start, numeric_out, string_out);
            collect_vars_in_expr(end, numeric_out, string_out);
            if let Some(step) = step {
                collect_vars_in_expr(step, numeric_out, string_out);
            }
            for stmt in body {
                collect_vars_in_statement(stmt, numeric_out, string_out);
            }
        }
        Statement::While { condition, body } => {
            collect_vars_in_expr(condition, numeric_out, string_out);
            for stmt in body {
                collect_vars_in_statement(stmt, numeric_out, string_out);
            }
        }
        Statement::Do {
            condition,
            body,
            post_condition,
        } => {
            if let Some(cond) = condition {
                collect_vars_in_expr(&cond.expr, numeric_out, string_out);
            }
            for stmt in body {
                collect_vars_in_statement(stmt, numeric_out, string_out);
            }
            if let Some(cond) = post_condition {
                collect_vars_in_expr(&cond.expr, numeric_out, string_out);
            }
        }
        Statement::SelectCase {
            expr,
            cases,
            else_body,
        } => {
            collect_vars_in_expr(expr, numeric_out, string_out);
            for clause in cases {
                for value in &clause.values {
                    match value {
                        CaseValue::Single(e) | CaseValue::Is { value: e, .. } => {
                            collect_vars_in_expr(e, numeric_out, string_out);
                        }
                        CaseValue::Range { from, to } => {
                            collect_vars_in_expr(from, numeric_out, string_out);
                            collect_vars_in_expr(to, numeric_out, string_out);
                        }
                    }
                }
                for stmt in &clause.body {
                    collect_vars_in_statement(stmt, numeric_out, string_out);
                }
            }
            for stmt in else_body {
                collect_vars_in_statement(stmt, numeric_out, string_out);
            }
        }
        // A function body's own `return <expr>`/bare `return`/procedure
        // call -- irrelevant at top level (`main` has none of these), but
        // this same function also walks function bodies (see
        // `emit_function_def`), where they do appear. `GlobalDecl` itself
        // needs no arm here: `collect_global_decl_idents` handles it
        // separately, and this function's own generic identifier-read
        // handling (`Statement::Ident`... there is none, reads only ever
        // happen inside an expression) never sees a bare `global x%`
        // statement as a variable use in the first place.
        Statement::Return { value } => collect_vars_in_expr(value, numeric_out, string_out),
        Statement::ExprStmt(expr) => collect_vars_in_expr(expr, numeric_out, string_out),
        // Every `FIELD` variable is a real string variable underneath
        // (see `FieldEntry`/`records::buffer_ident`) and needs the same
        // top-of-scope declaration as any other -- `scan_field_layout`
        // separately validates/records its byte layout, but declaring
        // the C storage for it is this pass's job, same as everything
        // else here.
        Statement::Field { fields, .. } => {
            for (_, var) in fields {
                register_var(var, numeric_out, string_out);
            }
        }
        Statement::Read(vars) => {
            for var in vars {
                collect_vars_in_expr(var, numeric_out, string_out);
            }
        }
        // `ON ERROR GOTO`/`RESUME`'s own targets are label names, not
        // variables (`OnErrorGoto`'s `Expr::Integer(0)` sentinel isn't
        // one either) -- neither needs an arm here, falling through to
        // the catch-all below. `ERROR <code>`'s code *can* be a real
        // variable (`error err` re-raises the current one -- `err` itself
        // is skipped by `register_var`'s own guard, not specially handled
        // here).
        Statement::ErrorStmt { code } => collect_vars_in_expr(code, numeric_out, string_out),
        // `err_var`/`erl_var` are hoisted like any other local (see
        // `emit_function_def`'s own doc comment on why every local is
        // declared, zero-initialized, at the top of its enclosing C
        // function rather than where first assigned) -- codegen_c.rs's
        // own `Statement::TryCatch` arm in `emit_statement` only ever
        // *assigns* them, at the top of the generated catch block; this
        // is what makes them exist as declared C locals at all.
        Statement::TryCatch {
            try_body,
            err_var,
            erl_var,
            catch_body,
        } => {
            for stmt in try_body {
                collect_vars_in_statement(stmt, numeric_out, string_out);
            }
            register_var(err_var, numeric_out, string_out);
            register_var(erl_var, numeric_out, string_out);
            for stmt in catch_body {
                collect_vars_in_statement(stmt, numeric_out, string_out);
            }
        }
        _ => {}
    }
}

fn collect_vars_in_expr(
    expr: &Expr,
    numeric_out: &mut BTreeMap<String, &'static str>,
    string_out: &mut BTreeSet<String>,
) {
    match expr {
        Expr::Ident(ident) => register_var(ident, numeric_out, string_out),
        Expr::Unary { expr, .. } => collect_vars_in_expr(expr, numeric_out, string_out),
        Expr::Binary { left, right, .. } => {
            collect_vars_in_expr(left, numeric_out, string_out);
            collect_vars_in_expr(right, numeric_out, string_out);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                collect_vars_in_expr(arg, numeric_out, string_out);
            }
        }
        // A single-argument (or zero-argument) function call parses as
        // `Expr::ArrayRef`, not `Expr::Call` -- see `make_paren_ident_expr`
        // in `parser.rs`, same ambiguity `codegen_basic::collect_call_sites`
        // documents and checks both shapes for. Collecting `indices`'
        // variable reads unconditionally (without checking whether `name`
        // is actually a known function) is safe either way: if this
        // really is a plain array reference instead (still unsupported by
        // this backend), the extra collected variable is harmless --
        // codegen will reject the array access itself later regardless.
        Expr::ArrayRef { indices, .. } => {
            for idx in indices {
                collect_vars_in_expr(idx, numeric_out, string_out);
            }
        }
        Expr::ScalarMethodCall { base, args, .. } => {
            collect_vars_in_expr(base, numeric_out, string_out);
            for arg in args {
                collect_vars_in_expr(arg, numeric_out, string_out);
            }
        }
        _ => {}
    }
}

fn register_var(
    ident: &BasicIdent,
    numeric_out: &mut BTreeMap<String, &'static str>,
    string_out: &mut BTreeSet<String>,
) {
    // Bare `ERR`/`ERL` are system pseudo-variables (see
    // `render_numeric_expr`'s `Expr::Ident` arm), not ordinary globals --
    // registering one here would declare a same-named, always-zero shadow
    // that every read/write of the real `bcc_err`/`bcc_erl` state would
    // silently miss instead.
    if ident.suffix.is_none()
        && (ident.name.eq_ignore_ascii_case("err") || ident.name.eq_ignore_ascii_case("erl"))
    {
        return;
    }
    // `INKEY$` is a real function call in disguise (see `render_string_
    // expr`'s own `Expr::Ident` arm) -- registering it here would declare
    // an always-empty shadow `char[256]` that every `k$ = inkey$` read
    // would silently miss, turning `do ... loop until k$ <> ""` into an
    // infinite loop (the actual bug this was fixed alongside).
    if ident.suffix == Some(TypeSuffix::String) && ident.name.eq_ignore_ascii_case("inkey") {
        return;
    }
    match ident.suffix {
        Some(TypeSuffix::String) => {
            string_out.insert(c_var_name(ident, TypeSuffix::String));
        }
        // `None` (suffixless) falls through to the numeric path below via
        // `effective_suffix` -- see its own doc comment for why `Single`
        // is the correct fill-in, not just a placeholder guess.
        suffix => {
            let suffix = effective_suffix(suffix);
            if let Some((c_type, _)) = numeric_c_type(suffix) {
                numeric_out.insert(c_var_name(ident, suffix), c_type);
            }
        }
    }
}

fn emit_statement(
    statement: &Stmt,
    out: &mut String,
    needs_math: &mut bool,
    needs_string: &mut bool,
    temp_counter: &mut usize,
    functions: &FunctionTable,
    current_function: Option<&FnSig>,
    file_io: &mut FileIoLayout,
    // GOSUB/RETURN support (top-level only -- see `Statement::Gosub`'s own
    // doc comment): `gosub_count` is the total number of top-level GOSUB
    // call sites in the whole program (from `count_gosubs`, computed once
    // before emission starts), and `gosub_id` is a counter incremented
    // once per GOSUB actually emitted, in the same left-to-right,
    // depth-first order `count_gosubs` walks the tree in -- guaranteeing
    // the Nth GOSUB encountered during emission gets the same ID
    // `count_gosubs` implicitly reserved for it (IDs `0..gosub_count`).
    // Always `0`/a throwaway counter inside a function/procedure body,
    // where GOSUB is rejected outright before either is ever read.
    gosub_count: usize,
    gosub_id: &mut usize,
    ctx: &mut ErrorDataCtx,
) -> Result<(), String> {
    match &statement.kind {
        Statement::Print { tokens } => {
            let (prelude, mut format, args, needs_newline) =
                render_print_tokens(tokens, needs_math, temp_counter, functions)?;
            for line in prelude {
                out.push_str(&line);
            }
            if needs_newline {
                format.push_str("\\n");
            }
            let mut call = format!("printf(\"{format}\"");
            for arg in &args {
                call.push_str(", ");
                call.push_str(arg);
            }
            call.push(')');
            out.push_str(&format!("    {call};\n"));
            Ok(())
        }
        Statement::End => {
            out.push_str("    return 0;\n");
            Ok(())
        }
        // `STOP`/`SYSTEM` -- both terminate the whole program outright,
        // real BASIC's own "halt, don't just fall through" statements
        // (`STOP` is real BASIC's breakpoint-style halt, resumable with
        // `CONT` only in an interactive session -- meaningless for a
        // compiled program, so indistinguishable from `END` here; `SYSTEM`
        // exits back to the OS/DOS shell). `exit(0)`, not `return 0`
        // (`Statement::End`'s own choice): unlike `END`, either of these
        // can appear inside a function/procedure body too, where
        // `return 0` would just return from that one call frame -- wrong,
        // since both need to halt the entire process regardless of call
        // depth.
        Statement::Stop | Statement::System => {
            out.push_str("    exit(0);\n");
            Ok(())
        }
        // `input [prompt$;] var` -- interactive keyboard input, distinct
        // from `input #` (sequential-file input, still unsupported). Real
        // BASIC always shows a trailing `? ` after the prompt (or alone,
        // with none given) -- BASCAL's own parser already discards the
        // `;`/`,` distinction after the prompt (see `Statement::Input`'s
        // own shape), so the BASIC backend already normalizes both
        // spellings to the same `; ` form; this matches that. Scoped to
        // exactly one variable per `input` -- real multi-variable,
        // comma-split-from-one-line `INPUT` isn't supported yet (no real
        // tutorial or example program in this repo uses it).
        Statement::Input { prompt, vars } => {
            if vars.len() != 1 {
                return Err(
                    "`input` with more than one variable isn't supported by the minimal C \
                     backend yet -- give each variable its own `input` statement"
                        .to_string(),
                );
            }
            let Expr::Ident(ident) = &vars[0] else {
                return Err(
                    "`input`'s target isn't supported by the minimal C backend yet -- only a \
                     bare scalar variable is"
                        .to_string(),
                );
            };
            let prompt_text = match prompt {
                Some(p) => format!("{}? ", escape_c_string_literal(p)),
                None => "? ".to_string(),
            };
            out.push_str(&format!("    printf(\"{prompt_text}\");\n"));
            out.push_str("    bcc_read_line();\n");
            if ident.suffix == Some(TypeSuffix::String) {
                let c_name = c_var_name(ident, TypeSuffix::String);
                out.push_str(&format!(
                    "    snprintf({c_name}, sizeof({c_name}), \"%s\", bcc_input_buf);\n"
                ));
            } else {
                let suffix = effective_suffix(ident.suffix);
                let c_name = c_var_name(ident, suffix);
                let (_, is_float) = numeric_c_type(suffix)
                    .expect("effective_suffix never returns TypeSuffix::String");
                if is_float {
                    out.push_str(&format!("    {c_name} = atof(bcc_input_buf);\n"));
                } else {
                    out.push_str(&format!("    {c_name} = atoi(bcc_input_buf);\n"));
                }
            }
            Ok(())
        }
        // Screen I/O -- see `COLOR_HELPER`'s own doc comment for why
        // `color` needs a runtime helper while these three don't: each is
        // a self-contained ANSI escape sequence with nothing to look up.
        Statement::Cls => {
            out.push_str("    printf(\"\\x1b[2J\\x1b[H\");\n");
            Ok(())
        }
        Statement::Beep => {
            out.push_str("    printf(\"\\a\");\n");
            Ok(())
        }
        // ANSI's own cursor-position escape is already `row;col`, 1-based,
        // exactly matching BASIC's own `LOCATE row, col` -- no reordering
        // or offset needed, unlike `COLOR`'s palette remapping.
        Statement::Locate { row, col } => {
            let (row_text, row_is_float) = render_numeric_expr(row, needs_math, functions)?;
            let row_text = coerce_numeric(row_text, row_is_float, false, needs_math);
            let (col_text, col_is_float) = render_numeric_expr(col, needs_math, functions)?;
            let col_text = coerce_numeric(col_text, col_is_float, false, needs_math);
            out.push_str(&format!("    printf(\"\\x1b[%d;%dH\", {row_text}, {col_text});\n"));
            Ok(())
        }
        Statement::Color { fg, bg } => {
            let (fg_text, fg_is_float) = render_numeric_expr(fg, needs_math, functions)?;
            let fg_text = coerce_numeric(fg_text, fg_is_float, false, needs_math);
            let bg_text = match bg {
                Some(bg) => {
                    let (text, is_float) = render_numeric_expr(bg, needs_math, functions)?;
                    coerce_numeric(text, is_float, false, needs_math)
                }
                None => "-1".to_string(),
            };
            out.push_str(&format!("    bcc_color({fg_text}, {bg_text});\n"));
            Ok(())
        }
        // Declarations are hoisted to the top of `main` up front (see
        // `collect_numeric_vars_in_statement` in `generate`), matching
        // BASIC's "springs into existence on first use" semantics -- `dim`
        // of a scalar is therefore a pure no-op here, already handled
        // wherever it happens to appear in source order.
        Statement::Dim {
            name,
            is_array: false,
            sizes,
        } if sizes.is_empty() => {
            let suffix = effective_suffix(name.suffix);
            if numeric_c_type(suffix).is_some() || suffix == TypeSuffix::String {
                Ok(())
            } else {
                Err(format!(
                    "`dim {name}` isn't supported by the minimal C backend yet -- only scalar \
                     variables (%, &, !, #, $) are"
                ))
            }
        }
        // `dim` of an array is likewise a no-op here -- its C declaration
        // was already hoisted into `generate`'s globals (see
        // `collect_array_declarations`/`ArrayInfo`), which also already
        // validated its shape (element type, literal-or-const bounds), so
        // reaching this arm at all means it's known-good.
        Statement::Dim {
            is_array: true, ..
        } => Ok(()),
        // `arr%(i%) = value` / `country$(i%) = value` -- an indexed
        // write into a `dim`-declared array (see `ArrayInfo`). Real C
        // multi-dimensional indexing (`arr[i]`, `grid[r][c]`) needs no
        // manual stride arithmetic, same as a read (see
        // `render_array_index_expr`). A string element can't be assigned
        // via plain `=` (C arrays don't support whole-array assignment),
        // so it goes through the same `snprintf` convention every other
        // string write in this backend already uses.
        Statement::Assignment {
            target: Expr::ArrayRef { name, indices },
            value,
        } if functions.arrays.contains_key(&array_c_name(name)) => {
            let c_expr = render_array_index_expr(name, indices, needs_math, functions)?;
            let info = &functions.arrays[&array_c_name(name)];
            if info.element_type.is_none() {
                let (prelude, text) =
                    render_string_expr(value, needs_math, temp_counter, functions)?;
                for line in prelude {
                    out.push_str(&line);
                }
                out.push_str(&format!(
                    "    snprintf({c_expr}, sizeof({c_expr}), \"%s\", {text});\n"
                ));
            } else {
                let (value_text, value_is_float) =
                    render_numeric_expr(value, needs_math, functions)?;
                let target_is_float = info.element_type.is_some_and(|(_, f)| f);
                let value_text =
                    coerce_numeric(value_text, value_is_float, target_is_float, needs_math);
                out.push_str(&format!("    {c_expr} = {value_text};\n"));
            }
            Ok(())
        }
        Statement::Assignment {
            target: Expr::Ident(name),
            value,
        } => emit_assignment(name, value, out, needs_math, temp_counter, functions),
        // Real MBASIC/BASCOM has no CONST statement at all -- `const` in
        // `.bcl` source is purely a naming/intent signal to the reader
        // (BASCAL's resolver already enforces it's never reassigned before
        // codegen ever runs), so it codegens exactly like an ordinary
        // assignment, same as the BASIC backend's own treatment of it.
        Statement::Const { name, value } => {
            emit_assignment(name, value, out, needs_math, temp_counter, functions)
        }
        // Unlike the BASIC backend, which has to transpile `if`/`elseif`/
        // `else` into a GOTO/label chain (real MBASIC/BASCOM has no block
        // `IF`), C has native `if`/`else`, so this is a direct structural
        // translation -- no labels needed. `elseif` doesn't need separate
        // handling either: the parser already desugars it into a single
        // nested `Statement::If` inside `else_body`, which the recursive
        // `emit_statement` call below just walks into naturally, producing
        // (harmless, if not maximally idiomatic) `} else {\n if (...) {`
        // nesting rather than a flat `else if` chain. Every line pushed
        // here (like everywhere else in this file) is flush against the
        // same flat indent regardless of nesting depth -- `generate`
        // re-indents the whole body in one pass afterward, see
        // `reindent_c_body`, rather than this function tracking depth
        // itself.
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            let (cond_text, _) = render_numeric_expr(condition, needs_math, functions)?;
            out.push_str(&format!("    if ({cond_text}) {{\n"));
            for stmt in then_body {
                emit_statement(
                    stmt,
                    out,
                    needs_math,
                    needs_string,
                    temp_counter,
                    functions,
                    current_function,
                    file_io,
                    gosub_count,
                    gosub_id,
                    ctx,
                )?;
            }
            if else_body.is_empty() {
                out.push_str("    }\n");
            } else {
                out.push_str("    } else {\n");
                for stmt in else_body {
                    emit_statement(
                        stmt,
                        out,
                        needs_math,
                        needs_string,
                        temp_counter,
                        functions,
                        current_function,
                        file_io,
                        gosub_count,
                        gosub_id,
                        ctx,
                    )?;
                }
                out.push_str("    }\n");
            }
            Ok(())
        }
        // BASIC evaluates a FOR loop's end/step expressions exactly once,
        // at loop entry -- not on every iteration, unlike a naive C `for`
        // whose condition re-reads whatever's in the expression each time.
        // If `end`/`step` refer to a variable the body mutates, that
        // matters: BASIC's bound stays fixed, so it's captured into its
        // own temp once, same as `render_string_expr`'s temps. The
        // increment direction (`<=` vs `>=`) is a runtime check on the
        // step's sign (`bt_step_n >= 0 ? ... : ...`), not assumed from the
        // step expression's syntactic shape, since a computed/variable
        // step's sign isn't known until runtime either.
        Statement::For {
            var,
            start,
            end,
            step,
            body,
        } => {
            let suffix = effective_suffix(var.suffix);
            let Some((c_type, target_is_float)) = numeric_c_type(suffix) else {
                return Err(format!(
                    "`for {var}` isn't supported by the minimal C backend yet -- the loop \
                     variable must be numeric (%, &, !, #)"
                ));
            };
            let loop_var = c_var_name(var, suffix);
            let (start_text, start_is_float) = render_numeric_expr(start, needs_math, functions)?;
            let start_text =
                coerce_numeric(start_text, start_is_float, target_is_float, needs_math);
            let (end_text, end_is_float) = render_numeric_expr(end, needs_math, functions)?;
            let end_text = coerce_numeric(end_text, end_is_float, target_is_float, needs_math);
            let step_text = match step {
                Some(step_expr) => {
                    let (text, is_float) = render_numeric_expr(step_expr, needs_math, functions)?;
                    coerce_numeric(text, is_float, target_is_float, needs_math)
                }
                None => "1".to_string(),
            };
            let limit = format!("bt_lim_{temp_counter}");
            let step_var = format!("bt_step_{temp_counter}");
            *temp_counter += 1;
            out.push_str(&format!("    {c_type} {limit} = {end_text};\n"));
            out.push_str(&format!("    {c_type} {step_var} = {step_text};\n"));
            out.push_str(&format!(
                "    for ({loop_var} = {start_text}; {step_var} >= 0 ? {loop_var} <= {limit} : \
                 {loop_var} >= {limit}; {loop_var} += {step_var}) {{\n"
            ));
            for stmt in body {
                emit_statement(
                    stmt,
                    out,
                    needs_math,
                    needs_string,
                    temp_counter,
                    functions,
                    current_function,
                    file_io,
                    gosub_count,
                    gosub_id,
                    ctx,
                )?;
            }
            out.push_str("    }\n");
            Ok(())
        }
        // Direct translation -- BASIC's WHILE/WEND is already exactly C's
        // `while`: pre-checked, continues while the condition is truthy
        // (nonzero), same "0 is false, anything else is true" rule C
        // already uses natively, no BASIC-specific semantics to bridge.
        Statement::While { condition, body } => {
            let (cond_text, _) = render_numeric_expr(condition, needs_math, functions)?;
            out.push_str(&format!("    while ({cond_text}) {{\n"));
            for stmt in body {
                emit_statement(
                    stmt,
                    out,
                    needs_math,
                    needs_string,
                    temp_counter,
                    functions,
                    current_function,
                    file_io,
                    gosub_count,
                    gosub_id,
                    ctx,
                )?;
            }
            out.push_str("    }\n");
            Ok(())
        }
        // Every `do` variant (pre-check `do while`/`do until`, post-check
        // `do ... loop while`/`loop until`, and a bare `do ... end do`
        // relying only on `exit`) is expressed uniformly as `while (1) { \
        // ...guards...; body; ...guards... }` rather than mapped
        // individually onto C's native `while`/`do-while` -- one shape
        // that's already correct for every case (including a
        // hypothetical pre-and-post-condition combination, if the parser
        // ever produced one) beats juggling several native-but-narrower
        // translations. A guard is `if (!(cond)) break;` for `while cond`
        // (exit when no longer true) or `if (cond) break;` for `until
        // cond` (exit once true) -- see `emit_do_guard`.
        Statement::Do {
            condition,
            body,
            post_condition,
        } => {
            out.push_str("    while (1) {\n");
            if let Some(cond) = condition {
                out.push_str(&emit_do_guard(cond, needs_math, functions)?);
            }
            for stmt in body {
                emit_statement(
                    stmt,
                    out,
                    needs_math,
                    needs_string,
                    temp_counter,
                    functions,
                    current_function,
                    file_io,
                    gosub_count,
                    gosub_id,
                    ctx,
                )?;
            }
            if let Some(cond) = post_condition {
                out.push_str(&emit_do_guard(cond, needs_math, functions)?);
            }
            out.push_str("    }\n");
            Ok(())
        }
        // `exit` is unqualified in BASCAL source (no "exit for"/"exit
        // while"/"exit do") -- the transpiler already knows which loop
        // it's in from context. The BASIC backend needs a loop_exit_stack
        // to track that (real MBASIC/BASCOM's own GOTO-chain-based loops
        // have no native "break"), but every C loop above is a real
        // native for/while loop, so plain `break;` already targets the
        // correct (innermost enclosing) one automatically -- no stack
        // needed here.
        Statement::Exit => {
            out.push_str("    break;\n");
            Ok(())
        }
        // A pure compile-time directive, not a runtime statement: it just
        // tells `emit_function_def`'s local-variable collection which
        // names to exclude from this function's own locals (see
        // `collect_global_decl_idents`), so the plain identifier used
        // anywhere else in the body naturally resolves to the file-scope
        // global of the same name via ordinary C lexical scoping -- no
        // per-use rewriting needed here at all.
        Statement::GlobalDecl(_) => Ok(()),
        // `OPEN ... FOR RANDOM/BINARY/INPUT/OUTPUT/APPEND AS #ch [LEN = n]`
        // -- every mode shares the same `bcc_files` channel table (see
        // `FILE_IO_HELPER`), so this statement alone is what sets
        // `file_io.used = true`, regardless of mode; a program that opens
        // a file only for sequential I/O still needs that table declared,
        // even though `apply_field_statement` (the only other place that
        // flag was set before) never runs for it. `LEN` is accepted but
        // not actually used for anything -- `GET`/`PUT`'s own record size
        // comes from the `FIELD` layout `scan_field_layout` already
        // computed, which is always consistent with `LEN` in BASCAL's own
        // record/file DSL output. Real BASIC's `OPEN FOR RANDOM` creates
        // the file if it doesn't already exist; C's `"rb+"` mode requires
        // the file to already exist, so a failed `"rb+"` open falls back
        // to `"wb+"` (create, read/write) rather than treating that as a
        // real error -- `OUTPUT`/`APPEND` already create-on-open via
        // C's own `"w"`/`"a"` modes, and `INPUT` (`"r"`) is the one mode
        // where a missing file is left as a genuine open failure (a
        // subsequent read/`EOF` against a NULL `FILE*` is this backend's
        // existing unchecked-range-style gap, same category as everywhere
        // else it trusts the program not to misuse a channel).
        Statement::Open {
            mode,
            file,
            channel,
            ..
        } => {
            let ch = literal_channel(channel)?;
            let idx = ch - 1;
            let (prelude, file_text) =
                render_string_expr(file, needs_math, temp_counter, functions)?;
            for line in prelude {
                out.push_str(&line);
            }
            file_io.used = true;
            match mode {
                // Neither attempt below is trappable (unlike `INPUT`'s own
                // arm just below) -- a `dim`/procedure-scoped `let p =
                // inv[n]` (`GET`) or `inv[n] = ...` (`PUT`) right after a
                // failed `OPEN` would otherwise dereference a NULL
                // `FILE*` and segfault (found via `tutorial/inventory.
                // bcl`'s own `initializeInventoryFileIfNew()`, called at
                // top level with no enclosing `try` -- a read-only
                // inven.dat makes both `fopen` attempts fail, "rb+"
                // needing write access and "wb+" needing to create/
                // truncate). A clean, fatal `exit(1)` here is a strict
                // improvement over that crash even without full trappable-
                // error support, which would need the same struct-return
                // propagation issue #67/#68 track for a raise inside an
                // arbitrary, possibly-non-reachable procedure like this one.
                OpenMode::Random | OpenMode::Binary => {
                    out.push_str(&format!(
                        "    bcc_files[{idx}] = fopen({file_text}, \"rb+\");\n"
                    ));
                    out.push_str(&format!(
                        "    if (!bcc_files[{idx}]) bcc_files[{idx}] = fopen({file_text}, \"wb+\");\n"
                    ));
                    out.push_str(&format!("    if (!bcc_files[{idx}]) {{\n"));
                    out.push_str(&format!(
                        "        fprintf(stderr, \"could not open %s for random access\\n\", {file_text});\n"
                    ));
                    out.push_str("        exit(1);\n");
                    out.push_str("    }\n");
                }
                // A missing file for `INPUT` mode raises real BASIC's own
                // error 53 ("file not found") -- but only at top level,
                // where `ON ERROR GOTO`/`RESUME` (and so a raise site's
                // own retry/skip labels -- see `ctx.raise_id`) are
                // actually supported (see `Statement::OnErrorGoto`'s own
                // doc comment); inside a function/procedure body this
                // still just leaves a `NULL` `FILE*` behind on failure,
                // the same pre-existing unchecked-range-style gap as
                // before this raise mechanism existed at all.
                OpenMode::Input if current_function.is_none() => {
                    let id = ctx.raise_id;
                    ctx.raise_id += 1;
                    out.push_str(&format!("    bcc_raise_retry_{id}: ;\n"));
                    out.push_str(&format!(
                        "    bcc_files[{idx}] = fopen({file_text}, \"r\");\n"
                    ));
                    out.push_str(&format!("    if (!bcc_files[{idx}]) {{\n"));
                    let mut raise = String::new();
                    emit_raise_block(&mut raise, "53", id, statement.pos.line, ctx.dispatch_labels);
                    for line in raise.lines() {
                        out.push_str("    ");
                        out.push_str(line);
                        out.push('\n');
                    }
                    out.push_str("    }\n");
                    out.push_str(&format!("    bcc_raise_after_{id}: ;\n"));
                }
                // Same raise, but from inside a `try`-reachable
                // procedure's own body -- no retry/after labels (no
                // `resume` to dispatch back to one, same as `error`'s own
                // in-procedure arm below), and status-return instead of
                // `emit_raise_block`'s `goto`, since a `goto` here could
                // never reach a handler in a different C function anyway.
                OpenMode::Input if ctx.current_function_reachable => {
                    out.push_str(&format!(
                        "    bcc_files[{idx}] = fopen({file_text}, \"r\");\n"
                    ));
                    out.push_str(&format!("    if (!bcc_files[{idx}]) {{\n"));
                    let mut raise = String::new();
                    emit_raise_in_procedure_block(&mut raise, "53", statement.pos.line);
                    for line in raise.lines() {
                        out.push_str("    ");
                        out.push_str(line);
                        out.push('\n');
                    }
                    out.push_str("    }\n");
                }
                OpenMode::Input => {
                    out.push_str(&format!(
                        "    bcc_files[{idx}] = fopen({file_text}, \"r\");\n"
                    ));
                }
                OpenMode::Output => {
                    out.push_str(&format!(
                        "    bcc_files[{idx}] = fopen({file_text}, \"w\");\n"
                    ));
                }
                OpenMode::Append => {
                    out.push_str(&format!(
                        "    bcc_files[{idx}] = fopen({file_text}, \"a\");\n"
                    ));
                }
            }
            Ok(())
        }
        Statement::Close { channel } => {
            let ch = literal_channel(channel)?;
            let idx = ch - 1;
            out.push_str(&format!("    fclose(bcc_files[{idx}]);\n"));
            out.push_str(&format!("    bcc_files[{idx}] = NULL;\n"));
            Ok(())
        }
        // `LINE INPUT #ch, var$` -- reads one whole line, same
        // `fgets`-plus-`strcspn` shape as keyboard `INPUT`'s own
        // `bcc_read_line` (see `INPUT_HELPER`), but straight from the
        // channel's `FILE*` via `bcc_line_input_file` (see
        // `SEQ_FILE_HELPER`) rather than always `stdin`, and straight into
        // the target's own buffer -- no shared scratch buffer needed, since
        // there's nothing further to parse out of a whole line the way
        // `INPUT #`'s comma-delimited fields need. The target can be a
        // bare string variable or a `dim`'d string array's own indexed
        // element (see `render_lvalue`, the same lvalue resolution
        // `Statement::Read`/`Statement::Swap` already use) -- either way
        // `sizeof(...)` on the resulting C lvalue is correct as-is: a
        // scalar string variable is a `char[STRING_BUFFER_SIZE]` local,
        // and a string array element indexes into a
        // `char[.][STRING_BUFFER_SIZE]` array, so `arr[i]` is itself a
        // `char[STRING_BUFFER_SIZE]`.
        Statement::LineInput { channel, target } => {
            let ch = literal_channel(channel)?;
            let idx = ch - 1;
            let (c_expr, element_type) = render_lvalue(target, needs_math, functions)?;
            if element_type.is_some() {
                return Err(
                    "LINE INPUT # requires a string (`$`-suffixed) target".to_string(),
                );
            }
            out.push_str(&format!(
                "    bcc_line_input_file(bcc_files[{idx}], {c_expr}, sizeof({c_expr}));\n"
            ));
            Ok(())
        }
        // `PRINT #ch, ...` -- identical rendering to plain `PRINT` (see its
        // own arm above, and `render_print_tokens`), just `fprintf`'d to
        // the channel's `FILE*` instead of `printf`'d to `stdout`.
        Statement::PrintFile { channel, tokens } => {
            let ch = literal_channel(channel)?;
            let idx = ch - 1;
            let (prelude, mut format, args, needs_newline) =
                render_print_tokens(tokens, needs_math, temp_counter, functions)?;
            for line in prelude {
                out.push_str(&line);
            }
            if needs_newline {
                format.push_str("\\n");
            }
            let mut call = format!("fprintf(bcc_files[{idx}], \"{format}\"");
            for arg in &args {
                call.push_str(", ");
                call.push_str(arg);
            }
            call.push(')');
            out.push_str(&format!("    {call};\n"));
            Ok(())
        }
        // `INPUT #ch, var[, ...]` -- reads each variable's own
        // comma-delimited field via `bcc_read_file_field` (see
        // `SEQ_FILE_HELPER`) into a shared scratch buffer, then parses it
        // the same way keyboard `INPUT`'s own arm above does (`snprintf`
        // straight through for a string target, `atoi`/`atof` for a
        // numeric one) -- safe to share one buffer across every field the
        // same way `INPUT_HELPER`'s `bcc_input_buf` is: each field is fully
        // consumed into its target variable before the next one is read.
        // Scoped to bare scalar variable targets, matching keyboard
        // `INPUT`'s own restriction.
        Statement::InputFile { channel, vars } => {
            let ch = literal_channel(channel)?;
            let idx = ch - 1;
            for var in vars {
                let Expr::Ident(ident) = var else {
                    return Err(
                        "INPUT #'s targets aren't supported by the minimal C backend yet -- \
                         only bare scalar variables are"
                            .to_string(),
                    );
                };
                out.push_str(&format!(
                    "    bcc_read_file_field(bcc_files[{idx}], bcc_file_field_buf, sizeof(bcc_file_field_buf));\n"
                ));
                if ident.suffix == Some(TypeSuffix::String) {
                    let c_name = c_var_name(ident, TypeSuffix::String);
                    out.push_str(&format!(
                        "    snprintf({c_name}, sizeof({c_name}), \"%s\", bcc_file_field_buf);\n"
                    ));
                } else {
                    let suffix = effective_suffix(ident.suffix);
                    let c_name = c_var_name(ident, suffix);
                    let (_, is_float) = numeric_c_type(suffix)
                        .expect("effective_suffix never returns TypeSuffix::String");
                    if is_float {
                        out.push_str(&format!(
                            "    {c_name} = atof(bcc_file_field_buf);\n"
                        ));
                    } else {
                        out.push_str(&format!(
                            "    {c_name} = atoi(bcc_file_field_buf);\n"
                        ));
                    }
                }
            }
            Ok(())
        }
        // `WRITE #ch, expr[, ...]` -- quoted-string, comma-separated
        // format that `INPUT #`/`bcc_read_file_field` can read back
        // exactly (see its own doc comment): each string argument is
        // wrapped in literal `"..."`, each numeric argument uses the same
        // `%d`/`%g` choice `render_print_tokens` does, items are joined
        // with `,` (no surrounding spaces -- real `WRITE #`'s own format),
        // and the line always ends in `\n` regardless of a trailing
        // `;`/`,` (unlike `PRINT`, `WRITE #` has no token-based suppression
        // syntax at all -- every `WRITE #` is a complete, newline-terminated
        // record).
        Statement::Write { channel, exprs } => {
            let ch = literal_channel(channel)?;
            let idx = ch - 1;
            let mut prelude = Vec::new();
            let mut format = String::new();
            let mut args = Vec::new();
            for (index, expr) in exprs.iter().enumerate() {
                if index > 0 {
                    format.push(',');
                }
                if is_string_expr_with_functions(expr, functions) {
                    let (expr_prelude, text) =
                        render_string_expr(expr, needs_math, temp_counter, functions)?;
                    prelude.extend(expr_prelude);
                    format.push_str("\\\"%s\\\"");
                    args.push(text);
                } else {
                    let (text, is_float) = render_numeric_expr(expr, needs_math, functions)?;
                    format.push_str(if is_float { "%g" } else { "%d" });
                    args.push(text);
                }
            }
            format.push_str("\\n");
            for line in prelude {
                out.push_str(&line);
            }
            let mut call = format!("fprintf(bcc_files[{idx}], \"{format}\"");
            for arg in &args {
                call.push_str(", ");
                call.push_str(arg);
            }
            call.push(')');
            out.push_str(&format!("    {call};\n"));
            Ok(())
        }
        // Pure compile-time bookkeeping, no runtime code -- records this
        // channel's field layout into `file_io` (see `FileIoLayout`'s own
        // doc comment for why this happens live, right here, rather than
        // as a one-time up-front scan); the fields' own C storage was
        // already declared via `collect_vars_in_statement`'s own `Field`
        // arm.
        Statement::Field {
            channel,
            fields,
            record_type,
            string_fields,
            field_types,
        } => apply_field_statement(channel, fields, record_type, string_fields, field_types, file_io),
        // `GET`/`PUT #ch, record` -- read/write one whole record (`GET`
        // splits the record's raw bytes across every `FIELD`'d variable
        // on that channel, at their declared offsets; `PUT` does the
        // reverse) into a single record-sized stack buffer, `fseek`ing
        // to `(record - 1) * record_width` first (BASIC record numbers
        // are 1-based). The bare `GET #ch`/`PUT #ch` (no record number,
        // "next sequential record") and `GET #ch, , var`/`PUT #ch, ,
        // var` (explicit target variable, not the FIELD buffer) forms
        // aren't supported -- BASCAL's own record/file DSL never
        // produces either, always giving an explicit record number and
        // no `var`.
        Statement::Get {
            channel,
            record,
            var: None,
            require_existing,
            ..
        } => emit_get_or_put(
            channel,
            record,
            needs_math,
            temp_counter,
            functions,
            file_io,
            out,
            true,
            *require_existing,
            None,
        ),
        Statement::Put {
            channel,
            record,
            var: None,
            provided_fields,
        } => emit_get_or_put(
            channel,
            record,
            needs_math,
            temp_counter,
            functions,
            file_io,
            out,
            false,
            false,
            provided_fields.as_deref(),
        ),
        // `LSET`/`RSET var = value` -- restricted to a `FIELD`'d
        // variable (looked up in `file_io.field_widths`; real BASIC
        // allows LSET/RSET on any string variable, justifying within
        // its *current* length, but every string variable in this
        // backend is a fixed `char[256]` with no meaningful "current
        // length" of its own, so only the FIELD'd case -- pad/truncate
        // to the field's *declared* width -- is implemented). `value`
        // being exactly `MKI$`/`MKL$`/`MKS$`/`MKD$` of a single
        // argument is special-cased to a direct `bcc_mkX` raw-byte pack
        // (see `FILE_IO_HELPER`) instead of the ordinary string-render
        // pipeline -- the packed bytes can contain a `0x00` byte
        // (e.g. `MKI$(1)` is bytes `01 00`), which a `snprintf`-based
        // copy would silently truncate at, being null-terminated-string
        // machinery, not byte-buffer machinery.
        Statement::Lset { var, value } | Statement::Rset { var, value } => {
            let is_rset = matches!(&**statement, Statement::Rset { .. });
            let c_name = c_var_name(var, TypeSuffix::String);
            let Some(&width) = file_io.field_widths.get(&c_name) else {
                return Err(format!(
                    "LSET/RSET on `{var}` isn't supported by the minimal C backend yet -- only a \
                     variable declared by a (literal-channel) FIELD is"
                ));
            };
            // A record/file DSL write (`db[i] = { ... }`/`?{ ... }`) always
            // `LSET`s straight into a `PUT` with nothing else reading the
            // buffer in between (see `records::Lowerer::lower_whole_write`
            // and friends) -- so for one of *those* buffers, defer instead
            // of packing: capture the still-native source value now, and
            // let the following `PUT` hand it straight to the typed helper
            // (`ensure_dsl_record_helpers`), which does the packing itself.
            // A raw, hand-written `LSET`/`RSET` never targets one of these
            // buffers (their names are DSL-synthesized), so this can't
            // misfire on ordinary user code.
            if !is_rset && is_dsl_record_buffer(file_io, &c_name) {
                let raw_value = match mk_pack_call(value) {
                    Some((_, _, arg)) => arg.clone(),
                    None => value.clone(),
                };
                file_io.pending_field_values.insert(c_name, raw_value);
                return Ok(());
            }
            if let Some((fn_name, target_is_float, arg)) = mk_pack_call(value) {
                if is_rset {
                    return Err(
                        "RSET with MKI$/MKL$/MKS$/MKD$ isn't supported by the minimal C backend \
                         yet -- real MBASIC/BASCOM packing functions are only ever paired with \
                         LSET"
                            .to_string(),
                    );
                }
                let (arg_text, arg_is_float) = render_numeric_expr(arg, needs_math, functions)?;
                let coerced = coerce_numeric(arg_text, arg_is_float, target_is_float, needs_math);
                out.push_str(&format!("    {fn_name}({c_name}, {coerced});\n"));
                return Ok(());
            }
            let (prelude, value_text) =
                render_string_expr(value, needs_math, temp_counter, functions)?;
            for line in prelude {
                out.push_str(&line);
            }
            let flag = if is_rset { "" } else { "-" };
            out.push_str(&format!(
                "    snprintf({c_name}, sizeof({c_name}), \"%{flag}*.*s\", {width}, {width}, {value_text});\n"
            ));
            Ok(())
        }
        // `select case` compiles to a native C `if`/`else if`/`else`
        // chain, same "no labels needed" story as `if`/`elseif`/`else`
        // above -- see `emit_select_case`.
        Statement::SelectCase {
            expr,
            cases,
            else_body,
        } => emit_select_case(
            expr,
            cases,
            else_body,
            out,
            needs_math,
            needs_string,
            temp_counter,
            functions,
            current_function,
            file_io,
            gosub_count,
            gosub_id,
            ctx,
        ),
        // Real C's own return-by-value works here directly: a numeric
        // return coerces the value the same way a plain assignment would
        // (`coerce_numeric` -- BASIC rounds a narrowing return the same
        // as any other narrowing assignment); a string return writes into
        // the synthesized `bcc_out` trailing parameter (see
        // `function_signature`) via `snprintf`, then `return;` bare,
        // since the underlying C function is actually `void` for a
        // string-returning BASCAL function.
        Statement::Return { value } => {
            let Some(sig) = current_function else {
                return Err(
                    "`return` outside of a function isn't supported by the minimal C backend"
                        .to_string(),
                );
            };
            if sig.is_string {
                let (prelude, text) =
                    render_string_expr(value, needs_math, temp_counter, functions)?;
                for line in prelude {
                    out.push_str(&line);
                }
                out.push_str(&format!(
                    "    snprintf(bcc_out, {STRING_BUFFER_SIZE}, \"%s\", {text});\n"
                ));
                emit_byref_scalar_copyback(sig, out);
                out.push_str("    return;\n");
            } else {
                let (text, is_float) = render_numeric_expr(value, needs_math, functions)?;
                let coerced = coerce_numeric(text, is_float, sig.is_float, needs_math);
                emit_byref_scalar_copyback(sig, out);
                out.push_str(&format!("    return {coerced};\n"));
            }
            Ok(())
        }
        // A bare `return` inside a `procedure` -- always legal even
        // mid-body, unlike falling off the end, which only a `void`
        // C function allows for free. Outside of any function/procedure
        // (`current_function.is_none()`), the exact same syntax means
        // something completely different: a BASIC-level GOSUB's own
        // `RETURN` (see `Statement::Gosub`'s own doc comment for the full
        // GOSUB/RETURN design) -- distinguishable here only by that
        // `current_function` context, never by anything in the AST node
        // itself, since the parser produces the identical `ReturnVoid` for
        // both (see `parser::parse_return`).
        Statement::ReturnVoid => {
            if current_function.is_none() {
                if gosub_count == 0 {
                    return Err(
                        "`return` outside of a function isn't supported by the minimal C backend"
                            .to_string(),
                    );
                }
                // Real GOSUB/RETURN is fully dynamic: which GOSUB call
                // site reached this particular RETURN is only known at
                // runtime (via `bcc_gosub_stack`'s popped value), not
                // statically -- the same RETURN can be reached from
                // multiple different GOSUBs, and a GOSUB target can fall
                // through into another one's body. So every RETURN gets
                // the identical dispatch: pop the ID, `switch` on it, jump
                // to that ID's own resume point right after its GOSUB
                // (see the `bcc_ret_<id>:` label `Statement::Gosub` emits)
                // -- covering every ID `0..gosub_count`, since any of them
                // could in principle be the one on top of the stack here.
                out.push_str("    switch (bcc_gosub_stack[--bcc_gosub_sp]) {\n");
                for id in 0..gosub_count {
                    out.push_str(&format!("    case {id}: goto bcc_ret_{id};\n"));
                }
                out.push_str("    }\n");
                return Ok(());
            }
            emit_byref_scalar_copyback(
                current_function.expect("checked by the `is_none()` branch above"),
                out,
            );
            if ctx.current_function_reachable {
                out.push_str("    return (bcc_result_void){ .status = 0 };\n");
            } else {
                out.push_str("    return;\n");
            }
            Ok(())
        }
        // `GOSUB label` -- BASIC-level subroutine call, distinct from a
        // `function`/`procedure` call: it jumps to `label` with no
        // parameters and no return value, and `RETURN` (see
        // `Statement::ReturnVoid`'s own arm above) resumes right after
        // whichever GOSUB reached it, which is only known at runtime.
        // Implemented with the same return-address-stack technique a real
        // BASIC interpreter uses: push a small integer ID (not a real
        // address -- `gosub_id`, assigned in strict left-to-right,
        // depth-first encounter order, matching `count_gosubs`' own
        // walk), jump to the target label, and mark the resume point
        // right after with a `bcc_ret_<id>:` label RETURN's `switch` jumps
        // back to.
        //
        // Scoped to top level only -- a GOSUB inside a `function`/
        // `procedure` body (`current_function.is_some()`) is rejected,
        // since RETURN there always means that function/procedure's own
        // return (see `Statement::ReturnVoid`), leaving no unambiguous
        // way to spell a GOSUB's own RETURN in the same body.
        Statement::Gosub(target) => {
            if current_function.is_some() {
                return Err(
                    "GOSUB inside a function/procedure isn't supported by the minimal C backend \
                     yet -- only a top-level GOSUB is"
                        .to_string(),
                );
            }
            let Expr::Ident(ident) = target else {
                return Err(
                    "GOSUB's target isn't supported by the minimal C backend yet -- only a bare \
                     label name is (enforced at parse time for every other BASCAL construct, so \
                     this shouldn't be reachable)"
                        .to_string(),
                );
            };
            let id = *gosub_id;
            *gosub_id += 1;
            out.push_str(&format!("    bcc_gosub_stack[bcc_gosub_sp++] = {id};\n"));
            out.push_str(&format!(
                "    goto bcc_lbl_{};\n",
                ident.name.to_ascii_lowercase()
            ));
            out.push_str(&format!("    bcc_ret_{id}:;\n"));
            Ok(())
        }
        // A bare call statement -- a `procedure` call, or a `function`
        // call whose result is deliberately discarded. Argument rendering
        // mirrors `render_numeric_call`/`render_string_call`'s own
        // user-function branch; the difference is there's no value to
        // hand back to a caller here; a string-returning function still
        // needs a `bcc_out` buffer to write into, just an unused one.
        // `Expr::ArrayRef` (not just `Expr::Call`) shows up here too -- a
        // zero-argument call always parses as `ArrayRef` regardless of
        // suffix (see `make_paren_ident_expr` in parser.rs), which a
        // parameterless `procedure foo()` call always is.
        Statement::ExprStmt(Expr::Call { name, args })
        | Statement::ExprStmt(Expr::ArrayRef {
            name,
            indices: args,
        }) => {
            let sig = functions.get(&fn_key(name)).ok_or_else(|| {
                format!(
                    "`{name}` isn't supported by the minimal C backend yet -- only a bare call \
                     to a known BASCAL function/procedure is supported as a standalone statement"
                )
            })?;
            let call_args = call_args_with_defaults(sig, args, name)?;
            let (prelude, mut arg_texts) =
                render_call_args(&call_args, &sig.params, needs_math, temp_counter, functions)?;
            for line in prelude {
                out.push_str(&line);
            }
            if sig.is_string {
                let temp = format!("bt_s_{temp_counter}");
                *temp_counter += 1;
                out.push_str(&format!("    char {temp}[{STRING_BUFFER_SIZE}];\n"));
                arg_texts.push(temp);
            }
            let call_text = format!("{}({})", sig.c_name, arg_texts.join(", "));
            // A `try`-reachable procedure's call sites (see
            // `collect_try_reachable_procedures`) decide purely from
            // their own lexical context, no analysis needed here: inside
            // the owning `try`'s own `try_body`, a nonzero status `goto`s
            // its catch label; inside another reachable procedure's own
            // body, it `return`s the whole status upward unchanged;
            // neither -- some call path this mechanism doesn't reach at
            // all (outside any `try`, or inside a non-reachable
            // procedure/function) -- just discards it, same as a plain
            // `void` call always has: the raise site itself already
            // fatal-exited if no trap was active (see `emit_raise_in_
            // procedure_block`), so a nonzero status can only ever reach
            // here when *something* up this call chain does check it.
            if ctx.try_reachable.contains(&fn_key(name)) {
                let status = format!("bcc_st_{temp_counter}");
                *temp_counter += 1;
                out.push_str(&format!("    bcc_result_void {status} = {call_text};\n"));
                if let Some(label) = &ctx.current_try_catch {
                    out.push_str(&format!("    if ({status}.status) goto {label};\n"));
                } else if ctx.current_function_reachable {
                    out.push_str(&format!("    if ({status}.status) return {status};\n"));
                }
            } else {
                out.push_str(&format!("    {call_text};\n"));
            }
            Ok(())
        }
        Statement::BlankLine => {
            out.push('\n');
            Ok(())
        }
        Statement::BlockComment(lines) => {
            for line in lines {
                out.push_str(&format!("    // {line}\n"));
            }
            Ok(())
        }
        // A `'`- or `//`-style single-line comment always parses to
        // `Statement::Raw("' <text>")` (see `parser.rs`) -- genuine raw
        // BASIC passthrough (hand-written GOTO/OPEN/etc.) would land here
        // too, but with no leading `'`, so only the comment shape is safe
        // to translate; anything else falls through to the generic error.
        Statement::Raw(text) if text.trim_start().starts_with('\'') => {
            let comment = text.trim_start().trim_start_matches('\'').trim_start();
            out.push_str(&format!("    // {comment}\n"));
            Ok(())
        }
        // `label:`/`goto label` -- raw BASIC's label-based control flow:
        // close to a 1:1 mapping onto C's own `goto`/label, since both
        // languages have the identical primitive. `gosub`/`return` (see
        // `GOSUB_HELPER`) and `on error goto`/`resume` (see
        // `ErrorDataCtx`/`emit_raise_block`) both need a real "remember
        // where to resume" execution model C's `goto` doesn't give for
        // free -- both built on the same return-address-ID-stack idea,
        // just with the roles reversed (GOSUB pushes an ID *forward* to a
        // fixed RETURN dispatch; a raise site's ID instead gets read
        // *backward* by a later RESUME).
        //
        // The label name gets the same `bcc_lbl_`-prefixed, lowercased
        // treatment `c_var_name` gives variables: C labels live in their
        // own namespace (no collision risk with a variable of the same
        // name), but an unprefixed label could still collide with a C
        // reserved word BASIC doesn't reserve (`int`, `for`, `do`, ...),
        // which isn't just a naming style choice here -- it would be a
        // real compile failure. The trailing `;` after the label makes it
        // a valid (empty) C statement even when it's the last thing in a
        // block, where a bare `label:}` is a syntax error.
        Statement::Label(name) => {
            out.push_str(&format!("    bcc_lbl_{}:;\n", name.to_ascii_lowercase()));
            Ok(())
        }
        Statement::Goto(target) => {
            let Expr::Ident(ident) = target else {
                return Err(
                    "GOTO's target isn't supported by the minimal C backend yet -- only a bare \
                     label name is (enforced at parse time for every other BASCAL construct, so \
                     this shouldn't be reachable)"
                        .to_string(),
                );
            };
            out.push_str(&format!(
                "    goto bcc_lbl_{};\n",
                ident.name.to_ascii_lowercase()
            ));
            Ok(())
        }
        // `RANDOMIZE` reseeds the same `rand()` stream `RND` draws from
        // (see `RND_HELPER`). Real BASIC's three forms: a numeric seed
        // (`RANDOMIZE 99`) reseeds deterministically -- straight to
        // `srand()`, same as `RND`'s own negative-argument case, just
        // without negating first (`RANDOMIZE`'s seed is already whatever
        // value the caller wants `srand()` to see). Bare `RANDOMIZE` real
        // BASIC prompts interactively for a seed -- not attempted here, no
        // interactive-input model exists in this backend yet -- so this
        // and `RANDOMIZE TIMER` both fall back to the same time-based
        // seed, a real, documented divergence for the bare form (matching
        // `TIMER`'s own real-elapsed-time reading, the closest available
        // stand-in for "vary run to run" without a prompt). `TIMER` itself
        // is recognized here by bare identifier name, not evaluated as an
        // ordinary variable -- like `ERR`/`ERL` (see `render_numeric_expr`'s
        // own `Expr::Ident` arm), just not itself readable as a value yet,
        // unlike those two.
        Statement::Randomize(seed) => {
            let is_timer = matches!(
                seed,
                Some(Expr::Ident(ident)) if ident.suffix.is_none() && ident.name.eq_ignore_ascii_case("timer")
            );
            match seed {
                None => out.push_str("    srand((unsigned int)time(NULL));\n"),
                Some(_) if is_timer => out.push_str("    srand((unsigned int)time(NULL));\n"),
                Some(expr) => {
                    let (text, is_float) = render_numeric_expr(expr, needs_math, functions)?;
                    let text = coerce_numeric(text, is_float, false, needs_math);
                    out.push_str(&format!("    srand((unsigned int)({text}));\n"));
                }
            }
            Ok(())
        }
        // `ON ERROR GOTO <label>`/`ON ERROR GOTO 0` -- installs/disables
        // the error trap (see `ErrorDataCtx`/`emit_raise_block`). Scoped
        // to top-level code only, same restriction (and the same reason)
        // `GOSUB` already has -- `RESUME`'s own dispatch below needs a
        // fixed, known set of raise sites to jump back to, which only
        // exists for top-level code (see `count_raise_sites`'s own doc
        // comment). A `procedure` target -- real BASIC also allows one,
        // reached with a plain `GOTO`, never a `GOSUB` (see the manual's
        // own `ON ERROR GOTO` section) -- isn't supported here yet; only
        // a label is.
        Statement::OnErrorGoto { target } => {
            if current_function.is_some() {
                return Err(
                    "`on error goto` isn't supported inside a function/procedure body by the \
                     minimal C backend yet -- only at top level"
                        .to_string(),
                );
            }
            match target {
                Expr::Integer(0) => {
                    out.push_str("    bcc_on_error_target = -1;\n");
                    Ok(())
                }
                Expr::Ident(ident) if functions.contains_key(&fn_key(ident)) => Err(format!(
                    "`on error goto {ident}` targets a `procedure` -- only a label target is \
                     supported by the minimal C backend yet"
                )),
                Expr::Ident(ident) => {
                    let id = ctx
                        .handler_ids
                        .get(&ident.name.to_ascii_lowercase())
                        .copied()
                        .expect("every ON ERROR GOTO target was already collected into handler_ids by collect_on_error_handler_ids");
                    out.push_str(&format!("    bcc_on_error_target = {id};\n"));
                    Ok(())
                }
                _ => Err(
                    "`on error goto`'s target isn't supported by the minimal C backend yet -- \
                     only a label name or `0` is"
                        .to_string(),
                ),
            }
        }
        // `RESUME`/`RESUME NEXT`/`RESUME <label>` -- see `ErrorDataCtx`'s
        // own doc comment for `ctx.raise_id`'s role, and
        // `Statement::ErrorStmt`/`Statement::Open`'s `OpenMode::Input` arm
        // for where a `bcc_raise_retry_<id>:`/`bcc_raise_after_<id>:`
        // label pair actually comes from. `RESUME`/`RESUME NEXT` dispatch
        // to whichever site raised via a `switch` on the live
        // `bcc_resume_id` global (set by that site's own `emit_raise_block`
        // call, read here, possibly much later in the emitted source, once
        // the handler body actually runs) -- `RESUME <label>` needs no
        // such dispatch, its target is a fixed label known at compile
        // time. All three clear `bcc_in_handler`, real BASIC's own "the
        // trap can fire again" reset, regardless of where they resume to.
        Statement::Resume(kind) => {
            if current_function.is_some() {
                return Err(
                    "`resume` isn't supported inside a function/procedure body by the minimal C \
                     backend yet -- only at top level"
                        .to_string(),
                );
            }
            match kind {
                ResumeTarget::Same | ResumeTarget::Next => {
                    out.push_str("    bcc_in_handler = 0;\n");
                    out.push_str("    switch (bcc_resume_id) {\n");
                    for id in 0..ctx.raise_site_count {
                        let label = if matches!(kind, ResumeTarget::Same) {
                            format!("bcc_raise_retry_{id}")
                        } else {
                            format!("bcc_raise_after_{id}")
                        };
                        out.push_str(&format!("    case {id}: goto {label};\n"));
                    }
                    out.push_str("    default: break;\n");
                    out.push_str("    }\n");
                    Ok(())
                }
                ResumeTarget::Line(Expr::Ident(ident)) => {
                    out.push_str("    bcc_in_handler = 0;\n");
                    out.push_str(&format!(
                        "    goto bcc_lbl_{};\n",
                        ident.name.to_ascii_lowercase()
                    ));
                    Ok(())
                }
                ResumeTarget::Line(_) => Err(
                    "`resume`'s target isn't supported by the minimal C backend yet -- only a \
                     bare label name is (enforced at parse time, so this shouldn't be reachable)"
                        .to_string(),
                ),
            }
        }
        // `ERROR <code>` -- triggers a runtime error as if it occurred
        // naturally (see `emit_raise_block`). `code` can be any numeric
        // expression, most commonly `err` itself, to re-raise an error a
        // handler decided it can't actually handle (see the manual's own
        // "typical pattern" example).
        Statement::ErrorStmt { code } => {
            if current_function.is_some() && !ctx.current_function_reachable {
                return Err(
                    "`error` isn't supported inside a function/procedure body by the minimal C \
                     backend yet -- only at top level, or inside a procedure called (directly or \
                     transitively) from a `try`'s own try_body"
                        .to_string(),
                );
            }
            let (code_text, code_is_float) = render_numeric_expr(code, needs_math, functions)?;
            let code_text = coerce_numeric(code_text, code_is_float, false, needs_math);
            if ctx.current_function_reachable {
                // No retry/after labels, no `bcc_raise_id` dispatch --
                // see `emit_raise_in_procedure_block`'s own doc comment.
                emit_raise_in_procedure_block(out, &code_text, statement.pos.line);
            } else {
                let id = ctx.raise_id;
                ctx.raise_id += 1;
                out.push_str(&format!("    bcc_raise_retry_{id}: ;\n"));
                emit_raise_block(out, &code_text, id, statement.pos.line, ctx.dispatch_labels);
                out.push_str(&format!("    bcc_raise_after_{id}: ;\n"));
            }
            Ok(())
        }
        // `DATA` items are pure compile-time literals, already fully
        // flattened into the static `bcc_data` array by
        // `collect_data_items_and_labels` before emission ever starts --
        // nothing left to emit here at the statement's own textual
        // position (`READ`/`RESTORE` are what actually touch `bcc_data`/
        // `bcc_data_ptr` at runtime).
        Statement::Data(_) => Ok(()),
        // `READ var[, ...]` -- pulls the next item(s) from the shared
        // `bcc_data`/`bcc_data_ptr` cursor (see `DATA_HELPER`), converting
        // each item's raw text to its target's actual type exactly the
        // way keyboard `INPUT` already converts `bcc_input_buf` (see
        // `Statement::Input`'s own arm) -- a `DATA` item's own type is
        // never tracked separately at all.
        // Each target's own C expression and element type come from the
        // same `render_lvalue` a scalar-or-array-element `SWAP`
        // operand already uses -- `READ`'s targets are exactly that same
        // shape (a bare scalar variable, or a `dim`-declared array's own
        // indexed element, e.g. `read country$(i%), capital$(i%)`).
        Statement::Read(vars) => {
            for var in vars {
                let (c_expr, element_type) = render_lvalue(var, needs_math, functions)?;
                match element_type {
                    None => out.push_str(&format!(
                        "    snprintf({c_expr}, sizeof({c_expr}), \"%s\", bcc_read_data());\n"
                    )),
                    Some((_, true)) => {
                        out.push_str(&format!("    {c_expr} = atof(bcc_read_data());\n"))
                    }
                    Some((_, false)) => {
                        out.push_str(&format!("    {c_expr} = atoi(bcc_read_data());\n"))
                    }
                }
            }
            Ok(())
        }
        // `RESTORE`/`RESTORE <label>` -- rewinds `bcc_data_ptr` to the
        // start, or to the item count `collect_data_items_and_labels`
        // already computed for that label at compile time -- a fixed
        // position in program order, so (unlike `ON ERROR GOTO`'s target)
        // this needs no runtime dispatch at all, just a literal integer.
        Statement::Restore(target) => match target {
            None => {
                out.push_str("    bcc_data_ptr = 0;\n");
                Ok(())
            }
            Some(Expr::Ident(ident)) => {
                let index = ctx
                    .data_labels
                    .get(&ident.name.to_ascii_lowercase())
                    .copied()
                    .ok_or_else(|| format!("`restore {ident}`: no such label"))?;
                out.push_str(&format!("    bcc_data_ptr = {index};\n"));
                Ok(())
            }
            Some(_) => Err(
                "RESTORE's target isn't supported by the minimal C backend yet -- only a bare \
                 label name is (enforced at parse time, so this shouldn't be reachable)"
                    .to_string(),
            ),
        },
        // `SWAP a, b` -- exchanges two lvalues (bare scalar variables, or
        // a `dim`-declared array's own indexed element -- see
        // `render_lvalue`) without a caller-visible temp. A numeric
        // swap goes through a real C temp of one side's own type (mixed
        // numeric types -- e.g. an `%` and a `!` -- still work, since C
        // freely converts between them on assignment, same simplification
        // `Statement::Swap`'s two sides get everywhere else in this
        // backend); a string swap can't use plain `=` at all (C arrays
        // don't support whole-array assignment), so it goes through a
        // temp `char[STRING_BUFFER_SIZE]` buffer and three `snprintf`
        // copies instead, the same convention every other string write in
        // this backend already uses.
        Statement::Swap(a, b) => {
            let (a_expr, a_type) = render_lvalue(a, needs_math, functions)?;
            let (b_expr, b_type) = render_lvalue(b, needs_math, functions)?;
            let temp = format!("bt_swap_{temp_counter}");
            *temp_counter += 1;
            match (a_type, b_type) {
                (None, None) => {
                    out.push_str(&format!("    char {temp}[{STRING_BUFFER_SIZE}];\n"));
                    out.push_str(&format!(
                        "    snprintf({temp}, sizeof({temp}), \"%s\", {a_expr});\n"
                    ));
                    out.push_str(&format!(
                        "    snprintf({a_expr}, sizeof({a_expr}), \"%s\", {b_expr});\n"
                    ));
                    out.push_str(&format!(
                        "    snprintf({b_expr}, sizeof({b_expr}), \"%s\", {temp});\n"
                    ));
                    Ok(())
                }
                (Some((c_type, _)), Some(_)) => {
                    out.push_str(&format!("    {c_type} {temp} = {a_expr};\n"));
                    out.push_str(&format!("    {a_expr} = {b_expr};\n"));
                    out.push_str(&format!("    {b_expr} = {temp};\n"));
                    Ok(())
                }
                _ => Err(
                    "SWAP's two operands must be the same kind (both string, or both numeric)"
                        .to_string(),
                ),
            }
        }
        // `try`/`catch` (issue #60), restricted to top-level code, exactly
        // like the rest of the `ON ERROR GOTO` family it's built on. Two
        // ways a raise inside `try_body` reaches the catch label below:
        // `emit_raise_block`'s own `goto` dispatch, for a raise from
        // `try_body`'s own top-level statements (`error`, or a failed
        // sequential `open ... for input`); or a nonzero status bubbling
        // straight up from a `try`-reachable procedure call inside
        // `try_body` (see `Statement::ExprStmt`'s own procedure-call arm,
        // driven by `ctx.current_try_catch` -- set here, restored after).
        // A raise from inside a called *function* still isn't caught --
        // GitHub issues #67/#68 track that remaining, larger piece.
        //
        // `bcc_on_error_target` is installed to this try's own id
        // (`ctx.try_id`, reserved up front by `generate`'s `dispatch_
        // labels` -- see `ErrorDataCtx`'s own doc comment) before
        // `try_body`, disabled again both after it completes normally
        // *and* right at catch entry -- the second reset matters just as
        // much as the first: without it, an error raised while running
        // catch_body itself (directly, or via another `try`-reachable
        // procedure call there) would still see this try's own id
        // installed and try to re-enter this same catch, instead of
        // correctly falling into the fatal "no nested-trap recovery"
        // path real BASIC has too. `bcc_in_handler` is cleared directly
        // (a plain global this backend fully controls, unlike real
        // BASIC's own trap state, which only `RESUME` can clear -- see
        // codegen_basic.rs's `try_catch` for why *that* backend needs
        // `RESUME <label>` instead of a bare `goto` here).
        Statement::TryCatch {
            try_body,
            err_var,
            erl_var,
            catch_body,
        } => {
            if current_function.is_some() {
                return Err(
                    "`try`/`catch` isn't supported inside a function/procedure body by the \
                     minimal C backend yet -- only at top level"
                        .to_string(),
                );
            }
            let id = ctx.try_id;
            ctx.try_id += 1;
            let catch_label = format!("bcc_try_{id}_catch");
            let end_label = format!("bcc_try_{id}_end");

            out.push_str(&format!("    bcc_on_error_target = {id};\n"));
            let outer_try_catch = ctx.current_try_catch.replace(catch_label.clone());
            for stmt in try_body {
                emit_statement(
                    stmt,
                    out,
                    needs_math,
                    needs_string,
                    temp_counter,
                    functions,
                    current_function,
                    file_io,
                    gosub_count,
                    gosub_id,
                    ctx,
                )?;
            }
            ctx.current_try_catch = outer_try_catch;
            out.push_str("    bcc_on_error_target = -1;\n");
            out.push_str(&format!("    goto {end_label};\n"));

            out.push_str(&format!("    {catch_label}: ;\n"));
            out.push_str("    bcc_in_handler = 0;\n");
            out.push_str("    bcc_on_error_target = -1;\n");
            let err_c = c_var_name(err_var, effective_suffix(err_var.suffix));
            let erl_c = c_var_name(erl_var, effective_suffix(erl_var.suffix));
            out.push_str(&format!("    {err_c} = bcc_err;\n"));
            out.push_str(&format!("    {erl_c} = bcc_erl;\n"));
            for stmt in catch_body {
                emit_statement(
                    stmt,
                    out,
                    needs_math,
                    needs_string,
                    temp_counter,
                    functions,
                    current_function,
                    file_io,
                    gosub_count,
                    gosub_id,
                    ctx,
                )?;
            }
            out.push_str(&format!("    {end_label}: ;\n"));
            Ok(())
        }
        other => Err(format!(
            "{other:?} is not supported by the minimal C backend yet -- only `print`, `end`, \
             `dim`, `if`, `for`, `while`, `do`, `exit`, `select case`, `return`, a bare \
             function/procedure call, and assignment/`const` of scalar variables (%, &, !, #, \
             $) are implemented so far"
        )),
    }
}

/// One assignable target, rendered as a C lvalue expression -- a bare
/// scalar variable, or a `dim`-declared array's own indexed element (see
/// `render_array_index_expr`) -- alongside its element type (`None` for
/// string, `Some((c_type, is_float))` for numeric). Shared by `SWAP`'s two
/// operands (so `Statement::Swap` can pick the right temp/swap strategy
/// for both together) and `READ`'s own targets (`read country$(i%),
/// capital$(i%)` needs exactly this same shape).
fn render_lvalue(
    expr: &Expr,
    needs_math: &mut bool,
    functions: &FunctionTable,
) -> Result<(String, Option<(&'static str, bool)>), String> {
    match expr {
        Expr::Ident(ident) if ident.suffix == Some(TypeSuffix::String) => {
            Ok((c_var_name(ident, TypeSuffix::String), None))
        }
        Expr::Ident(ident) => {
            let suffix = effective_suffix(ident.suffix);
            let element_type = numeric_c_type(suffix).ok_or_else(|| {
                format!(
                    "`{ident}` isn't supported by the minimal C backend yet -- only numeric or \
                     string scalar variables are"
                )
            })?;
            Ok((c_var_name(ident, suffix), Some(element_type)))
        }
        // A 2+-index array element parses as `Expr::Call`, not
        // `Expr::ArrayRef` -- see the identical arm's own comment in
        // `render_numeric_expr`.
        Expr::ArrayRef { name, indices } | Expr::Call { name, args: indices }
            if functions.arrays.contains_key(&array_c_name(name)) =>
        {
            let c_expr = render_array_index_expr(name, indices, needs_math, functions)?;
            let element_type = functions.arrays[&array_c_name(name)].element_type;
            Ok((c_expr, element_type))
        }
        _ => Err(
            "this target isn't supported by the minimal C backend yet -- only a bare scalar \
             variable or a dim'd array element is"
                .to_string(),
        ),
    }
}

/// Must match `FILE_IO_HELPER`'s own `#define BCC_MAX_CHANNELS 32` --
/// there's no way to interpolate a Rust `const` into that `const &str` C
/// template, so this is a second, deliberately-named copy of the same
/// number instead, kept in sync by hand (checked by
/// `literal_channel_respects_bcc_max_channels_bound` in `tests`).
const BCC_MAX_CHANNELS: i64 = 32;

/// A channel number that must be known at compile time -- `GET`/`PUT`
/// need to look up their channel's `FIELD` layout (`FileIoLayout`), and
/// `OPEN`/`CLOSE` index the fixed-size `bcc_files` table directly by
/// literal, both of which need an actual `i64`, not a rendered C
/// expression. Same restriction `scan_field_layout` already enforces for
/// `FIELD` itself, for the same reason -- BASCAL's own record/file DSL
/// only ever produces literal channel numbers (see
/// `records::lower_file_decl`), so this doesn't reject anything the DSL
/// itself would need.
///
/// Also range-checked here, once, against `BCC_MAX_CHANNELS` -- BASIC
/// channels are 1-based, so `0` is already meaningless, and a channel
/// above `BCC_MAX_CHANNELS` would index `bcc_files` out of bounds (real
/// undefined behavior, not just a wrong answer) if it weren't caught
/// before ever reaching generated code.
fn literal_channel(expr: &Expr) -> Result<i64, String> {
    let channel =
        match expr {
            Expr::Integer(n) => *n,
            _ => return Err(
                "a file channel number must be a literal integer -- the minimal C backend needs \
                 to know it at compile time"
                    .to_string(),
            ),
        };
    if !(1..=BCC_MAX_CHANNELS).contains(&channel) {
        return Err(format!(
            "file channel #{channel} is out of range -- the minimal C backend supports channels \
             1 through {BCC_MAX_CHANNELS}"
        ));
    }
    Ok(channel)
}

/// Whether `expr` is exactly `MKI$`/`MKL$`/`MKS$`/`MKD$` applied to a
/// single argument -- if so, the `bcc_mkX` helper to call (see
/// `FILE_IO_HELPER`), whether its C parameter is `double` (`true`, for
/// `MKS$`/`MKD$`) or `int`/`long` (`false`, for `MKI$`/`MKL$`, so
/// `coerce_numeric` rounds a narrowing argument the same as any other
/// narrowing conversion), and the argument expression itself. Used by
/// `Statement::Lset`'s special case -- see its own doc comment for why
/// packing bypasses the ordinary string-render pipeline entirely.
fn mk_pack_call(expr: &Expr) -> Option<(&'static str, bool, &Expr)> {
    // `MKI$`/`MKL$`/`MKS$`/`MKD$` all carry a `$` suffix and take exactly
    // one argument, so -- same ambiguity as CHR$/MID$/LEFT$ -- they parse
    // as `Expr::ArrayRef`, not `Expr::Call` (see `make_paren_ident_expr`
    // in `parser.rs`).
    let (name, args) = match expr {
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } => (name, args),
        _ => return None,
    };
    if args.len() != 1 {
        return None;
    }
    let fn_name = match name.name.to_ascii_lowercase().as_str() {
        "mki" => "bcc_mki",
        "mkl" => "bcc_mkl",
        "mks" => "bcc_mks",
        "mkd" => "bcc_mkd",
        _ => return None,
    };
    if name.suffix != Some(TypeSuffix::String) {
        return None;
    }
    Some((fn_name, matches!(fn_name, "bcc_mks" | "bcc_mkd"), &args[0]))
}

/// Whether `c_name` is one of the *currently FIELD'd* buffer variables of a
/// channel the record/file DSL itself declared (`channel_record_type` is
/// `Some` -- see `apply_field_statement`'s own note on why that map is
/// never populated from `known_record_layouts`' cosmetic inference). Used
/// by `Statement::Lset`'s C backend arm to decide whether to defer packing
/// to the typed PUT helper instead of packing into the buffer immediately.
fn is_dsl_record_buffer(file_io: &FileIoLayout, c_name: &str) -> bool {
    file_io.channel_fields.iter().any(|(ch, fields)| {
        file_io
            .channel_record_type
            .get(ch)
            .is_some_and(Option::is_some)
            && fields.iter().any(|field| field.c_name == c_name)
    })
}

/// Returns a C-safe suffix for helpers named after a BASCAL record type.
/// Record names are already identifier-like, but keeping this deliberately
/// narrow prevents a future parser relaxation from turning generated helper
/// names into invalid C.
fn record_helper_suffix(record_type: &str) -> String {
    record_type
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

/// The native C type backing a DSL record field's typed helper parameter --
/// exactly wide enough to `memcpy` straight into (or out of) the packed
/// on-disk representation, since `int16`/`int32`/`float32`/`float64` were
/// never anything but plain fixed-width binary in the first place (unlike
/// `MKI$`/`CVI` and friends, which only exist because real BASIC's `LSET`
/// requires a *string* to assign). `string(N)` fields never call this --
/// they use `const char*`/`char*` directly, same as everywhere else in
/// this backend.
fn record_field_c_type(ty: RecordFieldType) -> &'static str {
    match ty {
        RecordFieldType::Int16 => "int16_t",
        RecordFieldType::Int32 => "int32_t",
        RecordFieldType::Float32 => "float",
        RecordFieldType::Float64 => "double",
        RecordFieldType::Str(_) => unreachable!(
            "string record fields are passed as const char*/char*, never through record_field_c_type"
        ),
    }
}

/// Emits one reusable pair of pack/unpack functions for a *raw*,
/// hand-written `FIELD` layout -- keyed by channel generation, since a raw
/// `FIELD` doesn't declare a named type the way the record DSL does. `PUT`
/// always writes every bound buffer variable (real BASIC's `FIELD`/`LSET`
/// gives no partial-write concept), so this helper pair writes
/// unconditionally, with no NULL handling. The actual seeking and I/O is
/// delegated to `bcc_read_record`/`bcc_write_record`.
fn ensure_field_helpers(helper_key: &str, fields: &[FieldEntry], file_io: &mut FileIoLayout) {
    let suffix = record_helper_suffix(helper_key);
    if !file_io.record_helpers.insert(suffix.clone()) {
        return;
    }

    let reclen: u32 = fields.iter().map(|field| field.width).sum();
    let put_params = fields
        .iter()
        .enumerate()
        .map(|(index, _)| format!("const char* field_{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let get_params = fields
        .iter()
        .enumerate()
        .map(|(index, _)| format!("char* field_{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let put_separator = if put_params.is_empty() { "" } else { ", " };
    let get_separator = if get_params.is_empty() { "" } else { ", " };

    file_io.helper_protos.push_str(&format!(
        "static int bcc_put_record_{suffix}(FILE* file, long record{put_separator}{put_params});\n"
    ));
    file_io.helper_protos.push_str(&format!(
        "static int bcc_get_record_{suffix}(FILE* file, long record{get_separator}{get_params});\n"
    ));

    file_io.helper_defs.push_str(&format!(
        "static int bcc_put_record_{suffix}(FILE* file, long record{put_separator}{put_params}) {{\n    unsigned char buffer[{reclen}];\n"
    ));
    for (index, field) in fields.iter().enumerate() {
        file_io.helper_defs.push_str(&format!(
            "    memcpy(buffer + {offset}, field_{index}, {width});\n",
            offset = field.offset,
            width = field.width
        ));
    }
    file_io.helper_defs.push_str(&format!(
        "    bcc_write_record(file, buffer, {reclen}, record);\n    return 1;\n}}\n\n"
    ));

    file_io.helper_defs.push_str(&format!(
        "static int bcc_get_record_{suffix}(FILE* file, long record{get_separator}{get_params}) {{\n    unsigned char buffer[{reclen}];\n    if (!bcc_read_record(file, buffer, {reclen}, record)) return 0;\n"
    ));
    for (index, field) in fields.iter().enumerate() {
        if field.is_string {
            file_io.helper_defs.push_str(&format!(
                "    bcc_read_string_field(field_{index}, buffer + {offset}, {width});\n",
                offset = field.offset,
                width = field.width
            ));
        } else {
            file_io.helper_defs.push_str(&format!(
                "    memcpy(field_{index}, buffer + {offset}, {width});\n    field_{index}[{width}] = 0;\n",
                offset = field.offset,
                width = field.width
            ));
        }
    }
    file_io.helper_defs.push_str("    return 1;\n}\n\n");
}

/// Emits one reusable pair of pack/unpack functions for a record/file DSL
/// record type. Unlike `ensure_field_helpers`, `PUT` here is fully typed:
/// each non-string parameter is a pointer to its field's *native* C type
/// (`int16_t`/`int32_t`/`float`/`double`), packed with a plain `memcpy` --
/// no `bcc_mkX`/byte-string round trip -- and each `string(N)` parameter is
/// packed with `bcc_pad_string_field`. A NULL pointer marks a field omitted
/// by a partial (`?{ ... }`) update, exactly as `ensure_field_helpers`'s old
/// combined form did; the caller (`emit_get_or_put`) is responsible for
/// passing already-evaluated, correctly-typed values -- see its own
/// `pending_field_values` handling. `GET` stays blob-based (`char*` per
/// field, unconditionally read): its buffers still feed the record/file
/// DSL's ordinary `CVI`/`CVL`/`CVS`/`CVD`/trim-loop unpacking sequence,
/// shared with the BASIC backend, so it can't switch representations
/// without that shared sequence also changing.
fn ensure_dsl_record_helpers(record_type: &str, fields: &[FieldEntry], file_io: &mut FileIoLayout) {
    let suffix = record_helper_suffix(record_type);
    if !file_io.record_helpers.insert(suffix.clone()) {
        return;
    }

    let reclen: u32 = fields.iter().map(|field| field.width).sum();
    let put_params = fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            if field.is_string {
                format!("const char* field_{index}")
            } else {
                let ty = record_field_c_type(field.ty.expect(
                    "a DSL record field always carries its declared RecordFieldType",
                ));
                format!("const {ty}* field_{index}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    let get_params = fields
        .iter()
        .enumerate()
        .map(|(index, _)| format!("char* field_{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let put_separator = if put_params.is_empty() { "" } else { ", " };
    let get_separator = if get_params.is_empty() { "" } else { ", " };

    file_io.helper_protos.push_str(&format!(
        "static int bcc_put_record_{suffix}(FILE* file, long record{put_separator}{put_params});\n"
    ));
    file_io.helper_protos.push_str(&format!(
        "static int bcc_get_record_{suffix}(FILE* file, long record{get_separator}{get_params});\n"
    ));

    file_io.helper_defs.push_str(&format!(
        "static int bcc_put_record_{suffix}(FILE* file, long record{put_separator}{put_params}) {{\n    unsigned char buffer[{reclen}];\n"
    ));
    let missing_field = fields
        .iter()
        .enumerate()
        .map(|(index, _)| format!("!field_{index}"))
        .collect::<Vec<_>>()
        .join(" || ");
    let missing_field = if missing_field.is_empty() {
        "0".to_string()
    } else {
        missing_field
    };
    file_io.helper_defs.push_str(&format!(
        "    if (({missing_field}) && !bcc_read_record(file, buffer, {reclen}, record)) return 0;\n"
    ));
    for (index, field) in fields.iter().enumerate() {
        if field.is_string {
            file_io.helper_defs.push_str(&format!(
                "    if (field_{index}) bcc_pad_string_field(buffer + {offset}, field_{index}, {width});\n",
                offset = field.offset,
                width = field.width
            ));
        } else {
            file_io.helper_defs.push_str(&format!(
                "    (void)(field_{index} && memcpy(buffer + {offset}, field_{index}, {width}));\n",
                offset = field.offset,
                width = field.width
            ));
        }
    }
    file_io.helper_defs.push_str(&format!(
        "    bcc_write_record(file, buffer, {reclen}, record);\n    return 1;\n}}\n\n"
    ));

    file_io.helper_defs.push_str(&format!(
        "static int bcc_get_record_{suffix}(FILE* file, long record{get_separator}{get_params}) {{\n    unsigned char buffer[{reclen}];\n    if (!bcc_read_record(file, buffer, {reclen}, record)) return 0;\n"
    ));
    for (index, field) in fields.iter().enumerate() {
        if field.is_string {
            file_io.helper_defs.push_str(&format!(
                "    bcc_read_string_field(field_{index}, buffer + {offset}, {width});\n",
                offset = field.offset,
                width = field.width
            ));
        } else {
            file_io.helper_defs.push_str(&format!(
                "    memcpy(field_{index}, buffer + {offset}, {width});\n    field_{index}[{width}] = 0;\n",
                offset = field.offset,
                width = field.width
            ));
        }
    }
    file_io.helper_defs.push_str("    return 1;\n}\n\n");
}

/// `GET`/`PUT #ch, record` -- see `Statement::Get`/`Statement::Put`'s own
/// doc comment in `emit_statement` for the overall shape. `is_get`
/// selects which direction the `memcpy`s run: `GET` splits the freshly
/// `fread`-in record buffer out to every `FIELD`'d variable, `PUT`
/// gathers them back into the buffer before `fwrite`-ing it.
fn emit_get_or_put(
    channel: &Expr,
    record: &Option<Expr>,
    needs_math: &mut bool,
    temp_counter: &mut usize,
    functions: &FunctionTable,
    file_io: &mut FileIoLayout,
    out: &mut String,
    is_get: bool,
    require_existing: bool,
    provided_fields: Option<&[bool]>,
) -> Result<(), String> {
    let ch = literal_channel(channel)?;
    let idx = ch - 1;
    let Some(record) = record else {
        return Err(
            "GET/PUT with no record number (\"next sequential record\") isn't supported by the \
             minimal C backend yet -- always pass an explicit record number"
                .to_string(),
        );
    };
    let Some(fields) = file_io.channel_fields.get(&ch).cloned() else {
        return Err(format!(
            "GET/PUT on channel {ch} isn't supported by the minimal C backend yet -- no FIELD \
             was seen for it (with a literal channel number) before this point"
        ));
    };
    let (record_text, record_is_float) = render_numeric_expr(record, needs_math, functions)?;
    let record_text = coerce_numeric(record_text, record_is_float, false, needs_math);

    // The record DSL emits this GET as a BASIC-backend guard before a
    // partial PUT.  C performs that read inside the typed PUT helper, where
    // NULL field arguments make the operation atomic and avoid a second
    // record read.
    if is_get && require_existing {
        return Ok(());
    }

    let record_type = file_io.channel_record_type.get(&ch).cloned().flatten();
    if let Some(record_type) = record_type {
        let suffix = record_helper_suffix(&record_type);
        ensure_dsl_record_helpers(&record_type, &fields, file_io);
        if is_get {
            // GET stays blob-based -- see `ensure_dsl_record_helpers`'s own
            // doc comment for why -- so its call looks exactly like the raw
            // path below, just naming the DSL's own helper.
            let field_args = fields
                .iter()
                .map(|field| field.c_name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let separator = if field_args.is_empty() { "" } else { ", " };
            out.push_str(&format!(
                "    bcc_get_record_{suffix}(bcc_files[{idx}], {record_text}{separator}{field_args});\n"
            ));
            return Ok(());
        }

        // PUT is fully typed: pull each provided field's raw source value
        // straight out of `pending_field_values` (captured by `Statement::
        // Lset` instead of being packed into a buffer variable -- see that
        // arm's own doc comment) and pass it to the typed helper as a
        // native C value, materializing an addressable temporary for a
        // numeric field (the helper takes a pointer, so a NULL can still
        // mark a field omitted by a partial update). An omitted field
        // passes NULL directly, with no value to render at all.
        let mut field_args = Vec::with_capacity(fields.len());
        for (index, field) in fields.iter().enumerate() {
            let provided = !provided_fields
                .is_some_and(|provided| !provided.get(index).copied().unwrap_or(false));
            if !provided {
                field_args.push("NULL".to_string());
                continue;
            }
            let Some(raw_value) = file_io.pending_field_values.remove(&field.c_name) else {
                return Err(format!(
                    "internal error: record field `{}` (channel {ch}) has no pending value -- \
                     expected a `LSET` immediately before this `PUT`",
                    field.c_name
                ));
            };
            if field.is_string {
                let (prelude, text) =
                    render_string_expr(&raw_value, needs_math, temp_counter, functions)?;
                for line in prelude {
                    out.push_str(&line);
                }
                field_args.push(text);
            } else {
                let (text, value_is_float) =
                    render_numeric_expr(&raw_value, needs_math, functions)?;
                let ty = field
                    .ty
                    .expect("a DSL record field always carries its declared RecordFieldType");
                let target_is_float =
                    matches!(ty, RecordFieldType::Float32 | RecordFieldType::Float64);
                let coerced = coerce_numeric(text, value_is_float, target_is_float, needs_math);
                let tmp = format!("bcc_tmp_{}", *temp_counter);
                *temp_counter += 1;
                out.push_str(&format!(
                    "    {c_ty} {tmp} = {coerced};\n",
                    c_ty = record_field_c_type(ty)
                ));
                field_args.push(format!("&{tmp}"));
            }
        }
        let field_args = field_args.join(", ");
        let separator = if field_args.is_empty() { "" } else { ", " };
        let call = format!(
            "bcc_put_record_{suffix}(bcc_files[{idx}], {record_text}{separator}{field_args})"
        );
        if provided_fields.is_some_and(|provided| provided.iter().any(|present| !present)) {
            out.push_str(&format!(
                "    if (!{call}) {{ fprintf(stderr, \"BASCAL: record %ld does not exist\\n\", (long){record_text}); exit(1); }}\n"
            ));
        } else {
            out.push_str(&format!("    {call};\n"));
        }
        return Ok(());
    }

    let generation = file_io.channel_generation.get(&ch).copied().unwrap_or(0);
    let suffix = format!("fields_{ch}_{generation}");
    ensure_field_helpers(&suffix, &fields, file_io);
    let field_args = fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            if provided_fields
                .is_some_and(|provided| !provided.get(index).copied().unwrap_or(false))
            {
                "NULL".to_string()
            } else {
                field.c_name.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    let separator = if field_args.is_empty() { "" } else { ", " };
    let direction = if is_get { "get" } else { "put" };
    let call = format!(
        "bcc_{direction}_record_{suffix}(bcc_files[{idx}], {record_text}{separator}{field_args})"
    );
    if !is_get && provided_fields.is_some_and(|provided| provided.iter().any(|present| !present)) {
        out.push_str(&format!(
            "    if (!{call}) {{ fprintf(stderr, \"BASCAL: record %ld does not exist\\n\", (long){record_text}); exit(1); }}\n"
        ));
    } else {
        out.push_str(&format!("    {call};\n"));
    }
    Ok(())
}

/// Coerces a numeric value into a target of known float-ness, for any
/// narrowing (float/double source -> int target) assignment -- a bare `x%
/// = y / 2` (or a FOR loop's `i% = start`/limit/step, which needs the same
/// treatment). Real MBASIC/BASCOM **rounds** on this conversion, the same
/// as `CINT()`, confirmed directly against real BASCOM under dosbox-x:
/// `N% = 27 / 2` gives `N% = 14` (27/2 = 13.5, rounded up), not `13`
/// (C's own implicit double-to-int conversion truncates toward zero,
/// which would silently produce a different, wrong value here -- this
/// was caught by `for`/`while` loops actually exercising the case, e.g.
/// `n% = n% / 2` in a Collatz-sequence loop). Widening (int -> float/
/// double) needs no coercion: C converts an in-range integer to float/
/// double exactly, no rounding decision involved.
fn coerce_numeric(
    value_text: String,
    value_is_float: bool,
    target_is_float: bool,
    needs_math: &mut bool,
) -> String {
    if !target_is_float && value_is_float {
        *needs_math = true;
        format!("((int)round((double)({value_text})))")
    } else {
        value_text
    }
}

/// Shared by `Statement::Assignment` and `Statement::Const` -- both are
/// "evaluate `value`, store it in `name`'s variable," identical at the C
/// level (see the `Const` match arm's comment for why). Dispatches between
/// numeric and string variables; anything else (an array target, a
/// suffixless variable) is an error.
fn emit_assignment(
    name: &BasicIdent,
    value: &Expr,
    out: &mut String,
    needs_math: &mut bool,
    temp_counter: &mut usize,
    functions: &FunctionTable,
) -> Result<(), String> {
    if name.suffix == Some(TypeSuffix::String) {
        let (prelude, value_text) = render_string_expr(value, needs_math, temp_counter, functions)?;
        for line in prelude {
            out.push_str(&line);
        }
        let c_name = c_var_name(name, TypeSuffix::String);
        out.push_str(&format!(
            "    snprintf({c_name}, sizeof({c_name}), \"%s\", {value_text});\n"
        ));
        return Ok(());
    }
    let suffix = effective_suffix(name.suffix);
    match numeric_c_type(suffix) {
        Some((_, target_is_float)) => {
            let (value_text, value_is_float) = render_numeric_expr(value, needs_math, functions)?;
            let coerced = coerce_numeric(value_text, value_is_float, target_is_float, needs_math);
            out.push_str(&format!("    {} = {coerced};\n", c_var_name(name, suffix)));
            Ok(())
        }
        None => Err(format!(
            "assignment to `{name}` isn't supported by the minimal C backend yet -- give it an \
             explicit type suffix (%, &, !, #, $)"
        )),
    }
}

/// One `do`/`loop` guard line -- `if (!(cond)) break;` for a `while`
/// condition (exit once no longer true) or `if (cond) break;` for an
/// `until` condition (exit once true) -- shared by `Statement::Do`'s
/// pre-check and post-check cases (see its own comment for why both map
/// onto the same `while (1) { ...guards...; body; ...guards... }` shape).
fn emit_do_guard(
    cond: &DoCondition,
    needs_math: &mut bool,
    functions: &FunctionTable,
) -> Result<String, String> {
    let (cond_text, _) = render_numeric_expr(&cond.expr, needs_math, functions)?;
    Ok(if cond.is_while {
        format!("    if (!({cond_text})) break;\n")
    } else {
        format!("    if ({cond_text}) break;\n")
    })
}

/// `select case` evaluates its expression exactly once (into a fresh temp,
/// same "evaluate once" discipline as `for`'s start/end/step -- a
/// selector with side effects, e.g. a function call once those are
/// supported, must not run once per `case` clause), then compiles to a
/// native C `if`/`else if`/`else` chain testing that temp against each
/// clause's patterns -- no labels/GOTO dispatch needed, unlike the BASIC
/// backend's `select_case`, which has no block `IF` to build on. Wrapped
/// in its own `{ ... }` block so the temp's declaration can't collide
/// with another `select case` (or anything else) in the same scope.
///
/// The temp is numeric (`int`/`double`, matching `render_numeric_expr`'s
/// float-ness) or a `char[256]` buffer (matching every other string
/// value in this backend -- see `STRING_BUFFER_SIZE`), decided once via
/// `is_string_expr` on the selector and threaded into every clause's
/// pattern rendering (`render_case_value_cond`): a string selector can
/// only be tested with string-typed patterns and vice versa, same
/// type-consistency BASCAL's resolver already enforces before codegen
/// ever sees this.
fn emit_select_case(
    expr: &Expr,
    cases: &[CaseClause],
    else_body: &[Stmt],
    out: &mut String,
    needs_math: &mut bool,
    needs_string: &mut bool,
    temp_counter: &mut usize,
    functions: &FunctionTable,
    current_function: Option<&FnSig>,
    file_io: &mut FileIoLayout,
    gosub_count: usize,
    gosub_id: &mut usize,
    ctx: &mut ErrorDataCtx,
) -> Result<(), String> {
    let is_string = is_string_expr_with_functions(expr, functions);
    let temp = format!("bt_sel_{temp_counter}");
    *temp_counter += 1;

    out.push_str("    {\n");
    if is_string {
        let (prelude, text) = render_string_expr(expr, needs_math, temp_counter, functions)?;
        for line in prelude {
            out.push_str(&line);
        }
        out.push_str(&format!("    char {temp}[{STRING_BUFFER_SIZE}];\n"));
        out.push_str(&format!(
            "    snprintf({temp}, sizeof({temp}), \"%s\", {text});\n"
        ));
    } else {
        let (text, is_float) = render_numeric_expr(expr, needs_math, functions)?;
        let c_type = if is_float { "double" } else { "int" };
        out.push_str(&format!("    {c_type} {temp} = {text};\n"));
    }

    for (i, clause) in cases.iter().enumerate() {
        let mut conds = Vec::with_capacity(clause.values.len());
        for value in &clause.values {
            conds.push(render_case_value_cond(
                value,
                &temp,
                is_string,
                out,
                needs_math,
                needs_string,
                temp_counter,
                functions,
            )?);
        }
        let joined = conds.join(" || ");
        let keyword = if i == 0 { "if" } else { "} else if" };
        out.push_str(&format!("    {keyword} ({joined}) {{\n"));
        for stmt in &clause.body {
            emit_statement(
                stmt,
                out,
                needs_math,
                needs_string,
                temp_counter,
                functions,
                current_function,
                file_io,
                gosub_count,
                gosub_id,
                ctx,
            )?;
        }
    }
    if !cases.is_empty() {
        if !else_body.is_empty() {
            out.push_str("    } else {\n");
            for stmt in else_body {
                emit_statement(
                    stmt,
                    out,
                    needs_math,
                    needs_string,
                    temp_counter,
                    functions,
                    current_function,
                    file_io,
                    gosub_count,
                    gosub_id,
                    ctx,
                )?;
            }
        }
        out.push_str("    }\n");
    } else {
        for stmt in else_body {
            emit_statement(
                stmt,
                out,
                needs_math,
                needs_string,
                temp_counter,
                functions,
                current_function,
                file_io,
                gosub_count,
                gosub_id,
                ctx,
            )?;
        }
    }
    out.push_str("    }\n");
    Ok(())
}

/// Renders one `case` pattern (`CaseValue::Single`/`Range`/`Is`) as a C
/// boolean expression testing `temp` (the selector, already evaluated
/// once by `emit_select_case`). A string selector only supports
/// `Single` (exact match, via `strcmp(...) == 0` -- `needs_string` is
/// set here so `generate` knows to add `#include <string.h>`); `Range`/
/// `Is` on a string selector is rejected, not silently mistranslated,
/// since BASIC string comparison (`<`/`>` etc. on strings, or a numeric
/// `to` range against a string) isn't implemented by this backend at
/// all yet.
fn render_case_value_cond(
    value: &CaseValue,
    temp: &str,
    is_string: bool,
    out: &mut String,
    needs_math: &mut bool,
    needs_string: &mut bool,
    temp_counter: &mut usize,
    functions: &FunctionTable,
) -> Result<String, String> {
    match value {
        CaseValue::Single(expr) if is_string => {
            let (prelude, text) = render_string_expr(expr, needs_math, temp_counter, functions)?;
            for line in prelude {
                out.push_str(&line);
            }
            *needs_string = true;
            Ok(format!("(strcmp({temp}, {text}) == 0)"))
        }
        CaseValue::Single(expr) => {
            let (text, _) = render_numeric_expr(expr, needs_math, functions)?;
            Ok(format!("({temp} == {text})"))
        }
        CaseValue::Range { .. } if is_string => Err(
            "a `to` range in `select case` isn't supported on a string selector by the minimal \
             C backend yet -- only exact-match string case values are"
                .to_string(),
        ),
        CaseValue::Range { from, to } => {
            let (from_text, _) = render_numeric_expr(from, needs_math, functions)?;
            let (to_text, _) = render_numeric_expr(to, needs_math, functions)?;
            Ok(format!("({temp} >= {from_text} && {temp} <= {to_text})"))
        }
        CaseValue::Is { .. } if is_string => Err(
            "an `is` comparison in `select case` isn't supported on a string selector by the \
             minimal C backend yet -- only exact-match string case values are"
                .to_string(),
        ),
        CaseValue::Is { op, value } => {
            let (text, _) = render_numeric_expr(value, needs_math, functions)?;
            let c_op = match op {
                BinaryOp::Eq => "==",
                BinaryOp::Ne => "!=",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                _ => {
                    return Err(format!(
                        "`is {op:?}` isn't a valid select case comparison operator"
                    ))
                }
            };
            Ok(format!("({temp} {c_op} {text})"))
        }
    }
}

/// Whether `expr` is a string-typed expression -- a string literal, a
/// read of a `$`-suffixed variable, or `+` (concatenation) where either
/// side is. Used to route a `print`/assignment expression to
/// `render_string_expr` instead of `render_numeric_expr`; BASCAL's own
/// resolver has already rejected genuinely mixed-type `+` (a string plus a
/// number) before codegen ever runs, so this heuristic (check one side,
/// trust the program is well-typed) doesn't need to be a full type checker.
fn is_string_expr(expr: &Expr) -> bool {
    match expr {
        Expr::String(_) => true,
        Expr::Ident(ident) => ident.suffix == Some(TypeSuffix::String),
        Expr::Binary {
            op: BinaryOp::Add,
            left,
            right,
        } => is_string_expr(left) || is_string_expr(right),
        // A single-argument (or zero-argument) function call parses as
        // `Expr::ArrayRef`, not `Expr::Call` -- see `make_paren_ident_expr`
        // in `parser.rs`. Whether `name` is *actually* a known function
        // (vs. a genuine, unsupported array) isn't checked here -- same as
        // the `Expr::Call` arm below, which doesn't check either -- that
        // verification happens in `render_numeric_expr`/`render_string_expr`,
        // which do have the function table and can give a real "not a known
        // function" error; this only decides which of the two to route to.
        Expr::Call { name, .. } | Expr::ArrayRef { name, .. } => {
            name.suffix == Some(TypeSuffix::String)
        }
        _ => false,
    }
}

fn is_string_expr_with_functions(expr: &Expr, functions: &FunctionTable) -> bool {
    match expr {
        Expr::ScalarMethodCall { base, method, .. } => scalar_method_result(functions, base, method) == Some(TypeSuffix::String),
        Expr::Binary { op: BinaryOp::Add, left, right } =>
            is_string_expr_with_functions(left, functions) || is_string_expr_with_functions(right, functions),
        _ => is_string_expr(expr),
    }
}

fn scalar_expr_suffix(expr: &Expr, functions: &FunctionTable) -> Option<TypeSuffix> {
    match expr {
        Expr::String(_) => Some(TypeSuffix::String),
        Expr::Integer(_) | Expr::HexLit(_) => Some(TypeSuffix::Integer),
        Expr::Float(_) => Some(TypeSuffix::Single),
        Expr::Ident(id) | Expr::Call { name: id, .. } | Expr::ArrayRef { name: id, .. } => Some(effective_suffix(id.suffix)),
        Expr::Unary { expr, .. } => scalar_expr_suffix(expr, functions),
        Expr::Binary { left, .. } => scalar_expr_suffix(left, functions),
        Expr::ScalarMethodCall { base, method, .. } => scalar_method_result(functions, base, method),
        _ => None,
    }
}

fn scalar_method_result(functions: &FunctionTable, base: &Expr, method: &str) -> Option<TypeSuffix> {
    let receiver = scalar_expr_suffix(base, functions)?;
    functions.method(receiver, method)?.result_suffix
}

/// Every string variable/temporary is a fixed-size buffer -- real BASIC
/// strings are dynamically sized (heap-allocated, grow/shrink freely),
/// which this minimal backend doesn't attempt to replicate. `snprintf` is
/// used for every write into one of these buffers specifically so a string
/// longer than fits is *safely truncated*, never a buffer overflow --
/// unlike `strcpy`/`strcat`, which this backend deliberately never emits.
///
/// A Pascal-style representation (a length byte, heap storage rounded up
/// to 16-byte blocks so most assignments reuse the existing block instead
/// of reallocating) was considered and deliberately deferred: it doesn't
/// reduce allocation count below this scheme's current zero (every buffer
/// is a fixed-size local, allocated once, never resized), and it would
/// reintroduce exactly the malloc/realloc/free lifetime bugs (leaks,
/// use-after-free) this design exists to avoid, for no benefit while every
/// string is still a lone scalar. It's worth revisiting once this backend
/// supports arrays -- a flat `char[256][N]` string array is the case where
/// per-element right-sizing would actually matter -- but not before then.
const STRING_BUFFER_SIZE: usize = 256;

/// Renders a call to a string-returning user-defined function as an
/// expression -- shared by `render_string_expr`'s `Expr::Call` and
/// `Expr::ArrayRef` arms (see `is_string_expr`'s comment for why a
/// single-argument call parses as the latter). A string-returning
/// function is actually `void` in C, its BASCAL return value coming out
/// through a trailing `char* bcc_out` parameter instead of a real C
/// return value (see `function_signature`) -- so calling one as an
/// expression needs a prelude: a fresh temp buffer, plus the call itself
/// writing into it, exactly the same "materialize into a temp, use the
/// temp as the expression's value" shape `+` (concatenation) uses.
/// `RIGHT$(s$, n%)` -- the last `n` characters, or the whole string if
/// `n` is at least its length. No new C helper needed: it's exactly
/// `bcc_mid(s, strlen(s) - n + 1, n)`, and `bcc_mid`'s own clamping (see
/// `MID_HELPER`) already does the right thing at both extremes --
/// `n` &gt;= `strlen(s)` drives `start` to zero or negative, which clamps up
/// to the very first character, giving the whole string back; `n <= 0`
/// drives `length` to zero, clamped the same way `MID$`/`LEFT$`'s own
/// length argument already is. `s_text`/`n_text` are always safe to
/// reference twice here: both come from this backend's own render
/// functions, which already route any real work through a prelude
/// (assigned to a temp) before returning what's left -- a plain variable
/// name or literal, never an expression with a side effect to duplicate.
fn render_right_call(s_text: &str, n_text: &str) -> String {
    format!("bcc_mid({s_text}, (int)strlen({s_text}) - ({n_text}) + 1, {n_text})")
}

fn render_string_call(
    name: &BasicIdent,
    args: &[Expr],
    needs_math: &mut bool,
    temp_counter: &mut usize,
    functions: &FunctionTable,
) -> Result<(Vec<String>, String), String> {
    // `CHR$(code%)`/`MID$(s$, start%[, length%])`/`LEFT$(s$, n%)` (`LEFT$`
    // is exactly `MID$(s$, 1, n%)`) all delegate to the `bcc_chr`/`bcc_mid`
    // ring-buffer helpers (see `MID_HELPER`'s doc comment for why: they
    // return a self-contained `const char*` expression, no prelude of
    // their own needed, unlike every other non-trivial string value in
    // this backend). The 2-argument `MID$` form passes `INT_MAX` as the
    // length, relying on `bcc_mid`'s own clamp-to-available-length
    // behavior to reduce that to "everything from `start` to the end."
    // Only the *argument* expressions can still need a prelude of their
    // own (e.g. a concatenation as `MID$`'s source string) -- that's
    // still collected and threaded through normally.
    if name.name.eq_ignore_ascii_case("chr") && args.len() == 1 {
        let (text, is_float) = render_numeric_expr(&args[0], needs_math, functions)?;
        let coerced = coerce_numeric(text, is_float, false, needs_math);
        return Ok((Vec::new(), format!("bcc_chr({coerced})")));
    }
    // `STR$(n)` -- a number's string form, with real MBASIC/BASCOM's own
    // leading-space-for-non-negative convention (the same one the module
    // doc comment notes `print`'s own numeric formatting doesn't
    // reproduce yet -- `STR$` does, here, since it's cheap: C's `%` printf
    // flag on `%d`/`%g` already inserts exactly that leading space for a
    // non-negative value and a `-` for a negative one natively, no manual
    // sign handling needed).
    if name.name.eq_ignore_ascii_case("str") && args.len() == 1 {
        let (text, is_float) = render_numeric_expr(&args[0], needs_math, functions)?;
        return Ok((
            Vec::new(),
            if is_float {
                format!("bcc_strd({text})")
            } else {
                format!("bcc_stri({text})")
            },
        ));
    }
    if (name.name.eq_ignore_ascii_case("mid") && (args.len() == 2 || args.len() == 3))
        || (name.name.eq_ignore_ascii_case("left") && args.len() == 2)
    {
        let (prelude, s_text) = render_string_expr(&args[0], needs_math, temp_counter, functions)?;
        let is_left = name.name.eq_ignore_ascii_case("left");
        let (start_text, length_text) = if is_left {
            let (t, f) = render_numeric_expr(&args[1], needs_math, functions)?;
            ("1".to_string(), coerce_numeric(t, f, false, needs_math))
        } else {
            let (st, sf) = render_numeric_expr(&args[1], needs_math, functions)?;
            let start = coerce_numeric(st, sf, false, needs_math);
            let length = if args.len() == 3 {
                let (lt, lf) = render_numeric_expr(&args[2], needs_math, functions)?;
                coerce_numeric(lt, lf, false, needs_math)
            } else {
                "2147483647".to_string()
            };
            (start, length)
        };
        return Ok((
            prelude,
            format!("bcc_mid({s_text}, {start_text}, {length_text})"),
        ));
    }
    if name.name.eq_ignore_ascii_case("right") && args.len() == 2 {
        let (prelude, s_text) = render_string_expr(&args[0], needs_math, temp_counter, functions)?;
        let (nt, nf) = render_numeric_expr(&args[1], needs_math, functions)?;
        let n_text = coerce_numeric(nt, nf, false, needs_math);
        return Ok((prelude, render_right_call(&s_text, &n_text)));
    }
    let sig = functions.get(&fn_key(name)).ok_or_else(|| {
        format!(
            "`{name}` isn't supported by the minimal C backend yet -- only user-defined BASCAL \
             functions with a byval scalar signature are callable so far (no built-in BASIC \
             intrinsics like LEN/ASC/CHR$/MID$/LEFT$/RIGHT$ are, and are already handled above)"
        )
    })?;
    let call_args = call_args_with_defaults(sig, args, name)?;
    let (mut prelude, mut arg_texts) =
        render_call_args(&call_args, &sig.params, needs_math, temp_counter, functions)?;
    let temp = format!("bt_s_{temp_counter}");
    *temp_counter += 1;
    prelude.push(format!("    char {temp}[{STRING_BUFFER_SIZE}];\n"));
    arg_texts.push(temp.clone());
    prelude.push(format!("    {}({});\n", sig.c_name, arg_texts.join(", ")));
    Ok((prelude, temp))
}

/// Renders a string expression tree as C expression text usable as a
/// `char*` argument (an `snprintf`/`printf` `%s` argument, an assignment
/// source), plus any statements that must run first to materialize it.
///
/// String literals and `$`-suffixed variable reads need no prelude -- a
/// quoted C string literal and a buffer name (which decays to `char*`)
/// are both usable directly as a `%s` argument. `+` (concatenation)
/// does: C has no string-concatenation operator, so each concatenation
/// gets its own freshly declared temp buffer (`bt_s_<n>`, `temp_counter`
/// keeps every one unique within the program) and an `snprintf(dest,
/// sizeof(dest), "%s%s", left, right)` call -- safe against overflow the
/// same way `emit_assignment`'s string case is, and it's why plain C99
/// mid-block declarations (the temp buffer's `char bt_s_n[256];` line)
/// are emitted right where first needed rather than hoisted, unlike named
/// variables. A chain like `a$ + b$ + c$` (left-associative, same as
/// BASIC) therefore costs one temp buffer per `+`, not just one for the
/// whole chain -- more buffers than strictly necessary, but simple and
/// correct, consistent with this backend's other "correct over clever"
/// choices.
fn render_string_method_call(
    base: &Expr,
    method: &str,
    args: &[Expr],
    needs_math: &mut bool,
    temp_counter: &mut usize,
    functions: &FunctionTable,
) -> Result<(Vec<String>, String), String> {
    let receiver = scalar_expr_suffix(base, functions).ok_or_else(|| format!("method receiver for `.{method}()` must be scalar"))?;
    let sig = functions.method(receiver, method).ok_or_else(|| format!("unknown method `.{method}()`"))?;
    if !sig.is_string { return Err(format!("method `.{method}()` does not return a string")); }
    let mut call_args = Vec::with_capacity(args.len() + 1);
    call_args.push(base.clone());
    call_args.extend(args.iter().cloned());
    let call_args = call_args_with_defaults(sig, &call_args, &BasicIdent::parse(method))?;
    let mut prelude = Vec::new();
    let mut text = Vec::new();
    for (arg, param) in call_args.into_iter().zip(&sig.params) {
        if param.is_string {
            let (p, t) = render_string_expr(arg, needs_math, temp_counter, functions)?;
            prelude.extend(p); text.push(t);
        } else {
            let (t, f) = render_numeric_expr(arg, needs_math, functions)?;
            text.push(coerce_numeric(t, f, param.is_float, needs_math));
        }
    }
    let temp = format!("bt_s_{temp_counter}"); *temp_counter += 1;
    prelude.push(format!("    char {temp}[{STRING_BUFFER_SIZE}];\n"));
    text.push(temp.clone());
    prelude.push(format!("    {}({});\n", sig.c_name, text.join(", ")));
    Ok((prelude, temp))
}

fn render_string_expr(
    expr: &Expr,
    needs_math: &mut bool,
    temp_counter: &mut usize,
    functions: &FunctionTable,
) -> Result<(Vec<String>, String), String> {
    match expr {
        Expr::String(s) => Ok((Vec::new(), format!("\"{}\"", escape_c_string_literal(s)))),
        // `INKEY$` -- real BASIC's own non-blocking single-keypress read,
        // real BASCAL passes straight through too (see `bcc_inkey`'s own
        // doc comment for the terminal-raw-mode implementation). Checked
        // before the generic `Expr::Ident` arm below, which would
        // otherwise treat it as an ordinary (always-empty) string
        // variable -- see `register_var`'s matching skip.
        Expr::Ident(ident)
            if ident.suffix == Some(TypeSuffix::String)
                && ident.name.eq_ignore_ascii_case("inkey") =>
        {
            Ok((Vec::new(), "bcc_inkey()".to_string()))
        }
        Expr::Ident(ident) if ident.suffix == Some(TypeSuffix::String) => {
            Ok((Vec::new(), c_var_name(ident, TypeSuffix::String)))
        }
        Expr::Binary {
            op: BinaryOp::Add,
            left,
            right,
        } if is_string_expr_with_functions(left, functions) || is_string_expr_with_functions(right, functions) => {
            let (mut prelude, left_text) =
                render_string_expr(left, needs_math, temp_counter, functions)?;
            let (right_prelude, right_text) =
                render_string_expr(right, needs_math, temp_counter, functions)?;
            prelude.extend(right_prelude);
            let temp = format!("bt_s_{temp_counter}");
            *temp_counter += 1;
            prelude.push(format!("    char {temp}[{STRING_BUFFER_SIZE}];\n"));
            prelude.push(format!(
                "    snprintf({temp}, sizeof({temp}), \"%s%s\", {left_text}, {right_text});\n"
            ));
            Ok((prelude, temp))
        }
        // A string-returning function is actually `void` in C, its BASCAL
        // return value coming out through a trailing `char* bcc_out`
        // parameter instead of a real C return value (see
        // `function_signature`) -- so calling one as an *expression* needs
        // a prelude: a fresh temp buffer, plus the call itself writing
        // into it, exactly the same "materialize into a temp, use the
        // temp as the expression's value" shape `+` (concatenation) above
        // already uses.
        // A single-argument (or zero-argument) call to a string-returning
        // function parses as `Expr::ArrayRef`, not `Expr::Call` -- see
        // `make_paren_ident_expr` in `parser.rs`/`is_string_expr`'s own
        // comment -- so both shapes route to the same rendering.
        // `grid$(r%, c%)` -- a multi-dimensional string array read; see
        // the identical numeric-context arm's own comment in
        // `render_numeric_expr` for why 2+ indices parse as `Expr::Call`,
        // not `Expr::ArrayRef`.
        Expr::Call { name, args }
            if name.suffix == Some(TypeSuffix::String)
                && functions.arrays.contains_key(&array_c_name(name)) =>
        {
            let c_expr = render_array_index_expr(name, args, needs_math, functions)?;
            Ok((Vec::new(), c_expr))
        }
        Expr::Call { name, args } if name.suffix == Some(TypeSuffix::String) => {
            render_string_call(name, args, needs_math, temp_counter, functions)
        }
        Expr::ArrayRef { name, indices }
            if name.suffix == Some(TypeSuffix::String) && is_known_callable(name, functions) =>
        {
            render_string_call(name, indices, needs_math, temp_counter, functions)
        }
        // `country$(i%)` -- a `dim`-declared string array's own indexed
        // element (see `ArrayInfo`/`render_array_index_expr`). Checked
        // after the function-call arm above (`is_known_callable`), same
        // precedence `Expr::ArrayRef`'s own doc comment elsewhere in this
        // file already establishes for the numeric case.
        Expr::ArrayRef { name, indices } if functions.arrays.contains_key(&array_c_name(name)) => {
            let c_expr = render_array_index_expr(name, indices, needs_math, functions)?;
            Ok((Vec::new(), c_expr))
        }
        Expr::ScalarMethodCall { base, method, args } => {
            render_string_method_call(base, method, args, needs_math, temp_counter, functions)
        }
        _ => Err(
            "the minimal C backend's string expressions only support string literals, string \
             scalar variables ($), + (concatenation), and calls to user-defined BASCAL \
             functions so far"
                .to_string(),
        ),
    }
}

/// Builds any statements that must run before the `printf` call (e.g. a
/// string concatenation's temp-buffer setup), the `printf` format string
/// itself, its positional argument expressions, and whether the statement
/// wants a trailing newline -- same rule the BASIC backend's
/// `render_print_tokens` uses: a trailing `;`/`,` suppresses it, anything
/// else (including no separator at all) gets one.
///
/// A bare string literal contributes its (escaped) text directly to the
/// format string, with no `%s`/prelude needed. Any other string-typed
/// expression (`is_string_expr`: a string variable read, or `+`
/// concatenation) goes through `render_string_expr` instead, contributing
/// a `%s` placeholder, its value in `args`, and its prelude lines (if any)
/// to the output. Everything else goes through `render_numeric_expr`,
/// contributing a `%d`/`%g` placeholder plus C expression text. Anything
/// neither function understands (a call, an array) isn't supported yet and
/// is reported as an error rather than silently mishandled.
fn render_print_tokens(
    tokens: &[PrintToken],
    needs_math: &mut bool,
    temp_counter: &mut usize,
    functions: &FunctionTable,
) -> Result<(Vec<String>, String, Vec<String>, bool), String> {
    let mut prelude = Vec::new();
    let mut format = String::new();
    let mut args = Vec::new();
    let mut needs_newline = true;
    for (index, token) in tokens.iter().enumerate() {
        match token {
            PrintToken::Expr(Expr::String(s)) => {
                needs_newline = true;
                format.push_str(&escape_c_format_text(s));
            }
            // `TAB(n)`/`SPC(n)` -- print-position directives, not real
            // values (see tutorial/inventory.bcl's own header note on why
            // BASCOM itself rejects `"literal" + tab(n)`): only legal as
            // a bare, adjacent `PRINT` token, so they're intercepted
            // here, before falling through to `render_numeric_expr`
            // below, which has no general notion of either. `tab`/`spc`
            // are suffixless, so a single-arg call to either always
            // parses as `Expr::Call`, never `Expr::ArrayRef` (see
            // `make_paren_ident_expr` in parser.rs: `Expr::ArrayRef` only
            // for a *suffixed* name's single-arg call, or any name's
            // zero-arg call).
            //
            // `TAB(n)` moves the cursor to column `n` on the current
            // line, via the same ANSI cursor-column-absolute escape
            // family `LOCATE` already uses (`\x1b[<n>G`) -- consistent
            // with `LOCATE`'s own row/col passing straight through to
            // ANSI's identical 1-based column numbering, no reordering
            // or offset needed. `SPC(n)` prints `n` literal spaces, via
            // printf's own `%*s` field-width trick: a field width with
            // an empty string right-pads it with exactly that many
            // spaces.
            PrintToken::Expr(Expr::Call {
                name,
                args: call_args,
            }) if call_args.len() == 1 && name.name.eq_ignore_ascii_case("tab") => {
                let (n_text, n_is_float) =
                    render_numeric_expr(&call_args[0], needs_math, functions)?;
                let n_text = coerce_numeric(n_text, n_is_float, false, needs_math);
                needs_newline = true;
                format.push_str("\\x1b[%dG");
                args.push(n_text);
            }
            PrintToken::Expr(Expr::Call {
                name,
                args: call_args,
            }) if call_args.len() == 1 && name.name.eq_ignore_ascii_case("spc") => {
                let (n_text, n_is_float) =
                    render_numeric_expr(&call_args[0], needs_math, functions)?;
                let n_text = coerce_numeric(n_text, n_is_float, false, needs_math);
                needs_newline = true;
                format.push_str("%*s");
                args.push(n_text);
                args.push("\"\"".to_string());
            }
            PrintToken::Expr(expr) if is_string_expr_with_functions(expr, functions) => {
                let (expr_prelude, text) =
                    render_string_expr(expr, needs_math, temp_counter, functions)?;
                prelude.extend(expr_prelude);
                needs_newline = true;
                format.push_str("%s");
                args.push(text);
            }
            PrintToken::Expr(expr) => {
                let (text, is_float) = render_numeric_expr(expr, needs_math, functions)?;
                needs_newline = true;
                format.push_str(if is_float { "%g" } else { "%d" });
                args.push(text);
            }
            PrintToken::Semi | PrintToken::Comma => {
                needs_newline = index != tokens.len() - 1;
            }
        }
    }
    Ok((prelude, format, args, needs_newline))
}

/// Renders a numeric expression tree as C expression text, plus whether the
/// result is floating-point (picks `%g` vs `%d` in the caller). Covers
/// literals, numeric scalar variable reads (`%`/`&`/`!`/`#` -- the C
/// identifier and float-ness come from `c_var_name`/`numeric_c_type`; a
/// string variable read is a type error in a *numeric* context, so this
/// function still rejects `$` -- but strings themselves are handled fine,
/// just by `render_string_expr` instead, for expressions callers route
/// there via `is_string_expr`; no-suffix variables aren't supported at
/// all yet), negation, and every arithmetic operator: `+`/`-`/`*`
/// (direct translations, no semantic gap between BASIC and C), `/`
/// (explicit `(double)` casts on both operands, since BASIC's `/` always
/// performs floating-point division, even between two integers, unlike
/// plain C `/` between two `int`s), `\`/`MOD` (round each operand first,
/// per real MBASIC/BASCOM, then respectively truncate or take the
/// remainder of the integer quotient), and `^` (`pow()` from `<math.h>`,
/// right-associative -- already reflected in the tree shape by the time it
/// reaches here, same as the other operators' precedence). Every one of
/// these needed its exact BASIC semantics tracked down first (see the
/// per-arm comments below) rather than assuming "it's just the C operator"
/// -- several aren't.
///
/// `needs_math` is set whenever generated code calls into `<math.h>` (so
/// far: `\`/`MOD` via `round()`, `^` via `pow()`), so the caller knows to
/// add that `#include` -- most programs won't need it.
/// The `bcc_cvX` helper name for `CVI`/`CVL`/`CVS`/`CVD`, or `None` for
/// any other name -- see `render_numeric_call`'s own use of this.
fn cv_unpack_fn(name: &BasicIdent) -> Option<&'static str> {
    if name.suffix.is_some() {
        return None;
    }
    match name.name.to_ascii_lowercase().as_str() {
        "cvi" => Some("bcc_cvi"),
        "cvl" => Some("bcc_cvl"),
        "cvs" => Some("bcc_cvs"),
        "cvd" => Some("bcc_cvd"),
        _ => None,
    }
}

/// Shared by `sizeof`/`LBOUND`/`UBOUND` (see `render_numeric_call`'s own
/// call site): validates `args[0]` is a known array and `args.get(1)`
/// names one of its real axes (a literal integer, defaulting to `0` for a
/// 1-D array; required and checked in range for 2-D+), then resolves that
/// axis's raw bound as C expression text.
///
/// For a real top-level `dim`'d array this is `info.bounds[axis]`, a
/// compile-time-known literal. For one of the *current function's own
/// array parameters* (see `function_scoped_table`) there's no
/// compile-time-known bound at all -- the same function body is reused by
/// every call site, each of which can pass a differently-sized array -- so
/// its synthetic `ArrayInfo` carries the hidden `<c_name>_len0` runtime
/// parameter instead (see `ArrayInfo::runtime_len`'s own doc comment).
/// That parameter holds the real element *count* directly (set at the
/// call site by `render_call_args`/`apply_byval_array_capacities` as
/// `bound + 1`), so recovering the raw bound from it needs subtracting 1
/// back off.
///
/// `UBOUND` uses this value directly; `sizeof` adds 1 back to it (see
/// `render_numeric_call`'s own `sizeof` arm); `LBOUND` ignores it entirely
/// and always resolves to the literal `0` (`OPTION BASE` is rejected
/// outright -- see GitHub issue #50) -- but still needs the same
/// validation, so an unknown array or a bad axis is still a clear error
/// rather than a silently-wrong `0`.
fn resolve_array_bound_for_builtin(
    builtin_name: &str,
    args: &[Expr],
    functions: &FunctionTable,
) -> Result<String, String> {
    let Expr::Ident(array_name) = &args[0] else {
        return Err(format!(
            "`{builtin_name}` expects an array name, e.g. `{builtin_name}(arr%)` or \
             `{builtin_name}(grid%, 1)`"
        ));
    };
    let key = array_c_name(array_name);
    let info = functions.arrays.get(&key).ok_or_else(|| {
        format!("`{array_name}` isn't a known array, so `{builtin_name}` can't determine its size")
    })?;
    let axis = match args.get(1) {
        Some(Expr::Integer(n)) => *n as usize,
        Some(_) => {
            return Err(format!(
                "the axis argument to `{builtin_name}` must be a literal integer"
            ))
        }
        None if info.bounds.len() == 1 => 0,
        None => {
            return Err(format!(
                "`{array_name}` has {} dimensions -- {builtin_name} needs an axis argument, e.g. \
                 `{builtin_name}({array_name}, 0)`",
                info.bounds.len()
            ))
        }
    };
    if axis >= info.bounds.len() {
        return Err(format!(
            "`{array_name}` only has {} dimension{} -- axis {axis} doesn't exist",
            info.bounds.len(),
            if info.bounds.len() == 1 { "" } else { "s" }
        ));
    }
    if let Some(runtime) = info.runtime_len.as_ref().and_then(|v| v.get(axis)) {
        return Ok(format!("({runtime} - 1)"));
    }
    Ok(info.bounds[axis].to_string())
}

fn render_numeric_call(
    name: &BasicIdent,
    args: &[Expr],
    needs_math: &mut bool,
    functions: &FunctionTable,
) -> Result<(String, bool), String> {
    // `sizeof(arr%)` / `LBOUND(arr%)` / `UBOUND(arr%)`, and their `, axis`
    // forms -- see `resolve_array_bound_for_builtin`'s own doc comment for
    // the shared resolution all three build on. `OPTION BASE` is rejected
    // outright (see GitHub issue #50), so `LBOUND` is always the literal
    // `0`; `--target c` doesn't support `OPTION BASE` at all regardless.
    if let Some(builtin_name) = ["sizeof", "lbound", "ubound"]
        .into_iter()
        .find(|b| name.name.eq_ignore_ascii_case(b))
    {
        if !(1..=2).contains(&args.len()) {
            return Err(format!(
                "`{builtin_name}` expects an array name, e.g. `{builtin_name}(arr%)` or \
                 `{builtin_name}(grid%, 1)`"
            ));
        }
        if builtin_name == "lbound" {
            // Still validated below (unknown array / bad axis), just
            // discards the resolved bound -- see
            // `resolve_array_bound_for_builtin`'s own doc comment.
            resolve_array_bound_for_builtin(builtin_name, args, functions)?;
            return Ok(("0".to_string(), false));
        }
        let bound = resolve_array_bound_for_builtin(builtin_name, args, functions)?;
        if builtin_name == "ubound" {
            return Ok((bound, false));
        }
        // `sizeof()` returns the array's real element *count* along this
        // axis, not the raw bound `resolve_array_bound_for_builtin`
        // (and `UBOUND`) give back -- real BASIC's own inclusive-bound
        // convention means a `DIM arr%(N)` axis holds `N + 1` elements
        // (indices `0..=N`), matching `--target basic`'s own
        // `resolve_sizeof`. Adding 1 to an already-resolved integer
        // literal keeps the common case's generated code a plain literal
        // rather than a `(4 + 1)` expression; a runtime bound (an array
        // parameter's own hidden `_len0 - 1`, see
        // `resolve_array_bound_for_builtin`) still needs the arithmetic
        // spelled out.
        return Ok((
            match bound.parse::<i64>() {
                Ok(n) => (n + 1).to_string(),
                Err(_) => format!("({bound} + 1)"),
            },
            false,
        ));
    }
    // `LEN(s$)` -- `strlen`, cast to `int` so it prints under `%d` like
    // every other integer result here rather than a possibly-64-bit
    // `size_t`. `ASC(s$)` -- the ASCII code of the first character;
    // reading `s[0]` is always in-bounds (every string here is a
    // null-terminated `char[256]` buffer, so an empty string's `s[0]` is
    // just its own NUL terminator, giving `0`) -- unlike real BASIC,
    // which raises a runtime error on an empty string, this is a
    // behavioral gap, not a memory-safety one, same category as this
    // backend's other unchecked-range operators.
    if name.name.eq_ignore_ascii_case("len") && args.len() == 1 {
        let s = render_prelude_free_string_arg(&args[0], needs_math, functions)?;
        return Ok((format!("((int)strlen({s}))"), false));
    }
    if name.name.eq_ignore_ascii_case("asc") && args.len() == 1 {
        let s = render_prelude_free_string_arg(&args[0], needs_math, functions)?;
        return Ok((format!("((int)(unsigned char){s}[0])"), false));
    }
    // `VAL(s$)` -- parses a leading numeric prefix, real BASIC's own
    // "stop at the first character that doesn't extend a valid number"
    // behavior (`0` for a string with no such prefix at all) rather than
    // an all-or-nothing parse -- exactly what C's `atof` already does, no
    // helper needed. Always treated as float-typed here, same as the
    // `CVS`/`CVD` unpack functions below: a caller assigning the result
    // into an integer-suffixed variable still gets the correct rounding
    // via the ordinary `coerce_numeric` narrowing path every other
    // float-returning expression already goes through.
    if name.name.eq_ignore_ascii_case("val") && args.len() == 1 {
        let s = render_prelude_free_string_arg(&args[0], needs_math, functions)?;
        return Ok((format!("atof({s})"), true));
    }
    // `INSTR(s$, needle$)` -- the 1-based position of the first match, or
    // 0 for no match (see `INSTR_HELPER`). Only this 2-argument form is
    // supported -- see `INSTR_HELPER`'s own doc comment for why the
    // 3-argument start-position form is deliberately out of scope.
    if name.name.eq_ignore_ascii_case("instr") && args.len() == 2 {
        let s = render_prelude_free_string_arg(&args[0], needs_math, functions)?;
        let needle = render_prelude_free_string_arg(&args[1], needs_math, functions)?;
        return Ok((format!("bcc_instr({s}, {needle})"), false));
    }
    // `EOF(#ch)` -- non-zero once the channel's next read would hit end of
    // file (see `bcc_eof` in `SEQ_FILE_HELPER`). The channel has to be a
    // literal integer at compile time, same restriction `OPEN`/`CLOSE`/
    // `GET`/`PUT` already have (see `literal_channel`).
    if name.name.eq_ignore_ascii_case("eof") && args.len() == 1 {
        let ch = literal_channel(&args[0])?;
        let idx = ch - 1;
        return Ok((format!("bcc_eof(bcc_files[{idx}])"), false));
    }
    // `SQR(x)` -- real BASIC's SQR always returns a floating-point result
    // regardless of its argument's own type, exactly like `sqrt()`.
    if name.name.eq_ignore_ascii_case("sqr") && args.len() == 1 {
        let (inner, _) = render_numeric_expr(&args[0], needs_math, functions)?;
        *needs_math = true;
        return Ok((format!("sqrt((double)({inner}))"), true));
    }
    // `ABS(x)`/`INT(x)`/`FIX(x)` -- unlike `SQR`, these preserve their
    // argument's own int/float-ness (real BASIC's ABS/INT/FIX return the
    // same type they were given), so each computes through `fabs`/
    // `floor`/`trunc` on a `double` cast either way (simplest correct
    // shape, and evaluates the argument exactly once), then only casts
    // the *result* back to `int` when the argument itself was one --
    // `INT`'s `floor` (round toward negative infinity, not toward zero)
    // and `FIX`'s `trunc` (round toward zero) are genuinely different
    // rounding directions for a negative argument, same distinction
    // `\`/`MOD`'s own doc comment already draws for BASIC's own integer
    // division.
    if name.name.eq_ignore_ascii_case("abs") && args.len() == 1 {
        let (inner, is_float) = render_numeric_expr(&args[0], needs_math, functions)?;
        *needs_math = true;
        let call = format!("fabs((double)({inner}))");
        return Ok((
            if is_float {
                call
            } else {
                format!("(int)({call})")
            },
            is_float,
        ));
    }
    if name.name.eq_ignore_ascii_case("int") && args.len() == 1 {
        let (inner, is_float) = render_numeric_expr(&args[0], needs_math, functions)?;
        *needs_math = true;
        let call = format!("floor((double)({inner}))");
        return Ok((
            if is_float {
                call
            } else {
                format!("(int)({call})")
            },
            is_float,
        ));
    }
    if name.name.eq_ignore_ascii_case("fix") && args.len() == 1 {
        let (inner, is_float) = render_numeric_expr(&args[0], needs_math, functions)?;
        *needs_math = true;
        let call = format!("trunc((double)({inner}))");
        return Ok((
            if is_float {
                call
            } else {
                format!("(int)({call})")
            },
            is_float,
        ));
    }
    // `SGN(x)` -- -1/0/1 by sign, always an integer result regardless of
    // its argument's own type (see `bcc_sgn` in `SGN_HELPER`).
    if name.name.eq_ignore_ascii_case("sgn") && args.len() == 1 {
        let (inner, _) = render_numeric_expr(&args[0], needs_math, functions)?;
        *needs_math = true;
        return Ok((format!("bcc_sgn((double)({inner}))"), false));
    }
    // `RND`/`RND(x)` -- see `RND_HELPER`'s own doc comment for the
    // argument-selects-behavior semantics `bcc_rnd` implements. The
    // no-argument call is real BASIC's own shorthand for `RND(1)` -- "draw
    // the next value" -- so it's passed through as a literal `1.0`, no
    // different from a caller spelling out `RND(1)` themselves.
    if name.name.eq_ignore_ascii_case("rnd") && args.len() <= 1 {
        let arg = if let Some(first) = args.first() {
            let (inner, _) = render_numeric_expr(first, needs_math, functions)?;
            format!("(double)({inner})")
        } else {
            "1.0".to_string()
        };
        return Ok((format!("bcc_rnd({arg})"), true));
    }
    // `CINT(x)`/`CLNG(x)` -- round to the nearest integer (real BASIC's
    // own rounding, not truncation -- unlike `FIX`), the same
    // round-to-`int` shape `coerce_numeric` already gives any
    // float-to-int narrowing assignment; a caller writing `CINT`/`CLNG`
    // explicitly gets the identical result an implicit narrowing
    // assignment would, just spelled out. `CLNG`'s wider (32-bit) range
    // than `CINT`'s (16-bit) is a distinction real BASIC's INTEGER/LONG
    // types draw that this backend doesn't -- every integer here is
    // already a plain C `int` (see `numeric_c_type`), so the two compile
    // identically.
    if (name.name.eq_ignore_ascii_case("cint") || name.name.eq_ignore_ascii_case("clng"))
        && args.len() == 1
    {
        let (inner, _) = render_numeric_expr(&args[0], needs_math, functions)?;
        *needs_math = true;
        return Ok((format!("((int)round((double)({inner})))"), false));
    }
    // `CSNG(x)`/`CDBL(x)` -- force a float-typed result, same distinction
    // `numeric_c_type` already draws between `float` (`!`) and `double`
    // (`#`) variable storage; an already-numeric argument needs no value
    // transformation, just its own explicit cast, so a later narrowing
    // context (an assignment into an int-suffixed variable, say) still
    // goes through the ordinary `coerce_numeric` path as if this call
    // weren't there at all.
    if name.name.eq_ignore_ascii_case("csng") && args.len() == 1 {
        let (inner, _) = render_numeric_expr(&args[0], needs_math, functions)?;
        return Ok((format!("((float)({inner}))"), true));
    }
    if name.name.eq_ignore_ascii_case("cdbl") && args.len() == 1 {
        let (inner, _) = render_numeric_expr(&args[0], needs_math, functions)?;
        return Ok((format!("((double)({inner}))"), true));
    }
    // `SIN`/`COS`/`TAN`/`ATN`/`LOG`/`EXP` -- direct `<math.h>` mappings
    // (radians in and out, same convention real BASIC's own trig
    // functions use), each always float-typed regardless of its
    // argument's own type, same as `SQR`. `LOG` is the natural
    // logarithm (`log()`/`ln`), not base-10 -- real BASIC's own
    // convention (`LOG` is `ln`; base-10 would be `LOG10`, which BASIC
    // doesn't have a builtin for at all).
    for (basic_name, c_fn) in [
        ("sin", "sin"),
        ("cos", "cos"),
        ("tan", "tan"),
        ("atn", "atan"),
        ("log", "log"),
        ("exp", "exp"),
    ] {
        if name.name.eq_ignore_ascii_case(basic_name) && args.len() == 1 {
            let (inner, _) = render_numeric_expr(&args[0], needs_math, functions)?;
            *needs_math = true;
            return Ok((format!("{c_fn}((double)({inner}))"), true));
        }
    }
    // `CVI`/`CVL`/`CVS`/`CVD` unpack a `FIELD`'d variable's raw bytes
    // (see `FILE_IO_HELPER`'s `bcc_cvX` helpers and `Statement::Lset`'s
    // own doc comment for why packing/unpacking bypasses the ordinary
    // string machinery) -- restricted to exactly a bare variable
    // argument, matching `records::lower_whole_read`'s own always-`
    // Expr::Ident` usage; a general string expression here would still
    // need `render_string_expr`'s prelude, which this function has
    // nowhere to route (same restriction `render_prelude_free_string_arg`
    // documents).
    if let Some(fn_name) = cv_unpack_fn(name) {
        if args.len() == 1 {
            if let Expr::Ident(ident) = &args[0] {
                if ident.suffix == Some(TypeSuffix::String) {
                    let s = c_var_name(ident, TypeSuffix::String);
                    let is_float = matches!(fn_name, "bcc_cvs" | "bcc_cvd");
                    return Ok((format!("{fn_name}({s})"), is_float));
                }
            }
        }
        return Err(format!(
            "`{name}` isn't supported by the minimal C backend yet -- CVI/CVL/CVS/CVD only \
             support a bare FIELD'd string variable argument"
        ));
    }
    let sig = functions.get(&fn_key(name)).ok_or_else(|| {
        format!(
            "`{name}` isn't supported by the minimal C backend yet -- only user-defined BASCAL \
             functions with a byval scalar signature are callable so far (no built-in BASIC \
             intrinsics like CHR$/MID$/LEFT$/RIGHT$ are, and LEN/ASC are already handled above)"
        )
    })?;
    if sig.is_string {
        return Err(format!(
            "`{name}` returns a string, not a number, so it can't be used here"
        ));
    }
    let call_args = call_args_with_defaults(sig, args, name)?;
    let mut arg_texts = Vec::with_capacity(call_args.len());
    for (arg, param) in call_args.into_iter().zip(&sig.params) {
        if param.array.is_some() {
            let Expr::Ident(ident) = arg else {
                return Err(
                    "an array argument isn't supported by the minimal C backend yet -- only a \
                     bare array name is"
                        .to_string(),
                );
            };
            let key = array_c_name(ident);
            let info = functions.arrays.get(&key).ok_or_else(|| {
                format!(
                    "`{ident}` isn't a known array, so it can't be passed as an array argument"
                )
            })?;
            let len_text = info
                .runtime_len
                .as_ref()
                .and_then(|v| v.first())
                .cloned()
                .unwrap_or_else(|| (info.bounds[0] + 1).to_string());
            arg_texts.push(key);
            arg_texts.push(len_text);
        } else if param.by_ref {
            let Expr::Ident(ident) = arg else {
                return Err(
                    "a `byref` parameter was called with an argument that isn't a plain \
                     variable -- byref requires somewhere to write the result back to"
                        .to_string(),
                );
            };
            if param.is_string {
                arg_texts.push(c_var_name(ident, TypeSuffix::String));
            } else {
                let suffix = effective_suffix(ident.suffix);
                arg_texts.push(format!("&{}", c_var_name(ident, suffix)));
            }
        } else if param.is_string {
            arg_texts.push(render_prelude_free_string_arg(arg, needs_math, functions)?);
        } else {
            let (text, is_float) = render_numeric_expr(arg, needs_math, functions)?;
            arg_texts.push(coerce_numeric(text, is_float, param.is_float, needs_math));
        }
    }
    Ok((
        format!("{}({})", sig.c_name, arg_texts.join(", ")),
        sig.is_float,
    ))
}
fn render_numeric_method_call(
    base: &Expr,
    method: &str,
    args: &[Expr],
    needs_math: &mut bool,
    functions: &FunctionTable,
) -> Result<(String, bool), String> {
    let receiver = scalar_expr_suffix(base, functions).ok_or_else(|| format!("method receiver for `.{method}()` must be scalar"))?;
    let sig = functions.method(receiver, method).ok_or_else(|| format!("unknown method `.{method}()`"))?;
    if sig.is_string || sig.is_void { return Err(format!("method `.{method}()` does not return a number")); }
    let mut call_args = Vec::with_capacity(args.len() + 1);
    call_args.push(base.clone()); call_args.extend(args.iter().cloned());
    let call_args = call_args_with_defaults(sig, &call_args, &BasicIdent::parse(method))?;
    let mut text = Vec::new();
    for (arg, param) in call_args.into_iter().zip(&sig.params) {
        if param.is_string {
            text.push(render_prelude_free_string_arg(arg, needs_math, functions)?);
        } else {
            let (t, f) = render_numeric_expr(arg, needs_math, functions)?;
            text.push(coerce_numeric(t, f, param.is_float, needs_math));
        }
    }
    Ok((format!("{}({})", sig.c_name, text.join(", ")), sig.is_float))
}

fn render_numeric_expr(
    expr: &Expr,
    needs_math: &mut bool,
    functions: &FunctionTable,
) -> Result<(String, bool), String> {
    match expr {
        Expr::Integer(n) => Ok((n.to_string(), false)),
        Expr::Float(f) => Ok((format!("{f:?}"), true)),
        // `ERR`/`ERL`, bare (no type suffix) -- real BASIC's own
        // system pseudo-variables, holding the last raised error's code
        // and (for `ERL`) the line it occurred at. `ERL` reads `bcc_erl`,
        // the real `.bcl` source line baked in as a compile-time literal
        // at the raise site (see `emit_raise_block`'s own doc comment) --
        // not a BASIC-target `.bas` line number (this backend doesn't
        // generate one), but the actual source line, useful for locating
        // the fault in the program actually being edited. Checked before
        // the generic `Expr::Ident` arm below, which would otherwise treat
        // either name as an ordinary (always-zero) variable -- see
        // `register_var`'s matching skip.
        Expr::Ident(ident) if ident.suffix.is_none() && ident.name.eq_ignore_ascii_case("err") => {
            Ok(("bcc_err".to_string(), false))
        }
        Expr::Ident(ident) if ident.suffix.is_none() && ident.name.eq_ignore_ascii_case("erl") => {
            Ok(("bcc_erl".to_string(), false))
        }
        Expr::Ident(ident) => {
            let suffix = effective_suffix(ident.suffix);
            match numeric_c_type(suffix) {
                Some((_, is_float)) => Ok((c_var_name(ident, suffix), is_float)),
                None => Err(format!(
                    "`{ident}` isn't supported by the minimal C backend yet -- only numeric \
                     scalar variables (%, &, !, #, or suffixless) are"
                )),
            }
        }
        Expr::Unary {
            op: UnaryOp::Neg,
            expr,
        } => {
            let (inner, is_float) = render_numeric_expr(expr, needs_math, functions)?;
            Ok((format!("-({inner})"), is_float))
        }
        Expr::Binary {
            left,
            op: op @ (BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul),
            right,
        } => {
            let (left_text, left_float) = render_numeric_expr(left, needs_math, functions)?;
            let (right_text, right_float) = render_numeric_expr(right, needs_math, functions)?;
            let c_op = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                _ => unreachable!(),
            };
            Ok((
                format!("({left_text} {c_op} {right_text})"),
                left_float || right_float,
            ))
        }
        // Explicit `(double)` casts on both operands make this always a
        // floating-point division in C too, matching BASIC's `/` even when
        // both operands are integers (`5 / 2` is `2.5` in BASIC, but plain
        // C `/` between two `int`s would truncate to `2`).
        //
        // Division by a literal zero isn't specially detected: BASIC's `/`
        // raises a runtime "Division by zero" error, while C's `(double)x /
        // (double)0` silently produces `inf`/`nan` instead of crashing --
        // a real behavioral gap, just not a memory-safety one, and not
        // addressed here.
        Expr::Binary {
            left,
            op: BinaryOp::Div,
            right,
        } => {
            let (left_text, _) = render_numeric_expr(left, needs_math, functions)?;
            let (right_text, _) = render_numeric_expr(right, needs_math, functions)?;
            Ok((
                format!("((double){left_text} / (double){right_text})"),
                true,
            ))
        }
        // Real MBASIC/BASCOM's `\`: each operand is rounded to the nearest
        // integer first (verified against the GW-BASIC Reference Manual --
        // see the manual's Arithmetic Operators section), *then* the quotient
        // is truncated toward zero. `round()`'s ties-away-from-zero
        // tie-break is confirmed correct: real IBM Personal Computer BASIC
        // Compiler 2.00, run under dosbox-x, gives `2.5 \ 1 = 3` and
        // `-2.5 \ 1 = -3` -- round-half-to-even (the other common
        // convention) would instead round 2.5/-2.5 to their nearer *even*
        // neighbor, 2/-2, giving 2 and -2. C's `/` between two (rounded,
        // cast-to-`long`) integers
        // already truncates toward zero as of C99, so no extra truncation
        // step is needed once both operands are rounded. The final
        // `(int)` cast keeps the result a plain `int` so `%d` (not `%ld`)
        // is a correct printf format for it -- passing a `long` through a
        // `%d` vararg would be a real (if often silently-tolerated) type
        // mismatch. Overflow (a rounded operand or the quotient not
        // fitting in `long`/`int`) isn't specially detected, same as `/`'s
        // division-by-zero gap above.
        Expr::Binary {
            left,
            op: BinaryOp::IntDiv,
            right,
        } => {
            let (left_text, _) = render_numeric_expr(left, needs_math, functions)?;
            let (right_text, _) = render_numeric_expr(right, needs_math, functions)?;
            *needs_math = true;
            Ok((
                format!(
                    "((int)((long)round((double){left_text}) / (long)round((double){right_text})))"
                ),
                false,
            ))
        }
        // Real MBASIC/BASCOM's `MOD`: "the integer value that is the
        // remainder of an integer division" -- the same rounded, truncating
        // division `\` performs above (GW-BASIC Reference Manual examples:
        // `10.4 MOD 4 = 2` from `10 \ 4 = 2`; `25.68 MOD 6.99 = 5` from
        // `26 \ 7 = 3`, remainder 5). That's exactly C's `%` operator's own
        // definition since C99 (`a % b` has the same sign as `a`, matching
        // truncating division) -- no separate sign-handling logic needed,
        // just apply `%` to the same rounded, `long`-cast operands `\` uses.
        // MOD by a literal zero isn't specially detected: it's undefined
        // behavior in C (typically SIGFPE), where BASIC raises a runtime
        // "Division by zero" error instead -- not addressed here, same
        // category of gap as `/`'s and `\`'s.
        Expr::Binary {
            left,
            op: BinaryOp::Mod,
            right,
        } => {
            let (left_text, _) = render_numeric_expr(left, needs_math, functions)?;
            let (right_text, _) = render_numeric_expr(right, needs_math, functions)?;
            *needs_math = true;
            Ok((
                format!(
                    "((int)((long)round((double){left_text}) % (long)round((double){right_text})))"
                ),
                false,
            ))
        }
        // `^` (right-associative -- already reflected in the tree shape by
        // the time it reaches here, same as `+`/`-`/`*`'s precedence, so no
        // extra handling needed for that part) maps directly to pow() from
        // <math.h>, which always returns a `double`. A whole-number result
        // (`2 ^ 8` -> `256.0`) still prints as `256` under `%g`, not
        // `256.000000`, so this doesn't look any different from an integer
        // result in the common case. Negative bases with non-integer
        // exponents (e.g. `(-8) ^ (1/3)`) produce `nan` via real-valued
        // pow(), same as they'd error in BASIC -- a behavioral gap, not
        // addressed here, same category as the other operators' noted
        // gaps above.
        Expr::Binary {
            left,
            op: BinaryOp::Pow,
            right,
        } => {
            let (left_text, _) = render_numeric_expr(left, needs_math, functions)?;
            let (right_text, _) = render_numeric_expr(right, needs_math, functions)?;
            *needs_math = true;
            Ok((
                format!("pow((double){left_text}, (double){right_text})"),
                true,
            ))
        }
        // Real MBASIC/BASCOM's comparison operators evaluate to -1 (true)
        // or 0 (false) -- confirmed in the manual's own Comparison Operators
        // section -- not 1/0 like C's `==`/`<`/etc. `-(a == b)` gets there
        // directly: C's comparison already produces 0 or 1, and negating
        // that gives exactly 0 or -1. The result is always a plain `int`
        // (is_float = false), matching how a BASIC boolean gets used
        // (printed as an integer, fed into arithmetic or, eventually,
        // AND/OR -- see the bitwise-AND/OR project memory for why those
        // must NOT reuse C's `&&`/`||` the way this reuses `==`/`<`/etc.).
        Expr::Binary {
            left,
            op:
                op @ (BinaryOp::Eq
                | BinaryOp::Ne
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge),
            right,
        } => {
            let c_op = match op {
                BinaryOp::Eq => "==",
                BinaryOp::Ne => "!=",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
                _ => unreachable!(),
            };
            // A string comparison (real BASIC's `<`/`<=`/`>`/`>=` on
            // strings compare lexicographically, exactly what `strcmp`'s
            // own sign already gives) -- restricted, like every other
            // string operand in this function, to the prelude-free
            // shapes `render_prelude_free_string_arg` covers, since this
            // function has nowhere to route a prelude a fuller string
            // expression (`+` concatenation) would need.
            if is_string_expr_with_functions(left, functions) || is_string_expr_with_functions(right, functions) {
                let l = render_prelude_free_string_arg(left, needs_math, functions)?;
                let r = render_prelude_free_string_arg(right, needs_math, functions)?;
                return Ok((format!("(-(strcmp({l}, {r}) {c_op} 0))"), false));
            }
            let (left_text, _) = render_numeric_expr(left, needs_math, functions)?;
            let (right_text, _) = render_numeric_expr(right, needs_math, functions)?;
            Ok((format!("(-({left_text} {c_op} {right_text}))"), false))
        }
        // BASCAL's `&&`/`||` (distinct from bitwise `AND`/`OR` above --
        // restricted to `if`/`elseif`/`while`/`do` conditions, never
        // storable in a variable) are *already* real short-circuit
        // operators, unlike classic BASIC's bitwise-only `AND`/`OR` --
        // real C's own `&&`/`||` are the direct, correct translation
        // here (genuinely short-circuit, same as BASCAL's own), not the
        // bug `Eq`/`Ne`/etc. above's own comment warns `AND`/`OR` against
        // reusing. The BASIC backend needs a manual GOTO-chain
        // (`condition_jump`) to fake short-circuit evaluation, since
        // real MBASIC/BASCOM has no such primitive at all; C already
        // does. Result is `int` (0/1, not BASIC's -1/0) -- fine here
        // since, like the comparison operators above producing a value
        // that's *only* ever tested for zero-vs-nonzero in a condition,
        // this can't be stored into a variable and compared against a
        // literal `-1` elsewhere the way an ordinary BASIC boolean might.
        Expr::Binary {
            left,
            op: op @ (BinaryOp::AndAnd | BinaryOp::OrOr),
            right,
        } => {
            let (left_text, _) = render_numeric_expr(left, needs_math, functions)?;
            let (right_text, _) = render_numeric_expr(right, needs_math, functions)?;
            let c_op = match op {
                BinaryOp::AndAnd => "&&",
                BinaryOp::OrOr => "||",
                _ => unreachable!(),
            };
            Ok((format!("({left_text} {c_op} {right_text})"), false))
        }
        // Real MBASIC/BASCOM's AND/OR/XOR are genuinely bitwise, not
        // short-circuit booleans -- see the project memory saved
        // specifically for this. Verified against the GW-BASIC Reference
        // Manual: "Logical operators work by converting their operands to
        // 16-bit, signed, two's complement integers... the given operation
        // is performed on these integers bit by bit." The manual doesn't
        // say "rounded" here as explicitly as it does for `\`/MOD, but that
        // it means the same round()-style conversion is now confirmed, not
        // just assumed: real IBM Personal Computer BASIC Compiler 2.00,
        // run under dosbox-x, gives `2.5 AND 3 = 3` (round(2.5)=3, 3&3=3)
        // and `3.5 AND 3 = 0` (round(3.5)=4, 4&3=0) -- a plain truncating
        // conversion (int(2.5)=2, int(3.5)=3) would instead give 2 and 3.
        // This is exactly why C's `&`/`|`/`^` are the
        // right translation and C's `&&`/`||` would NOT be: this operates
        // on arbitrary integer *values* (`6 XOR 3 = 5`), not just BASIC's
        // -1/0 booleans -- though on -1/0 inputs specifically, plain
        // bitwise AND/OR/XOR already reproduces BASIC's truth table exactly
        // (two's complement -1 is all-ones), so no separate boolean-vs-
        // integer branch is needed.
        Expr::Binary {
            left,
            op: op @ (BinaryOp::And | BinaryOp::Or | BinaryOp::Xor),
            right,
        } => {
            let (left_text, _) = render_numeric_expr(left, needs_math, functions)?;
            let (right_text, _) = render_numeric_expr(right, needs_math, functions)?;
            *needs_math = true;
            let c_op = match op {
                BinaryOp::And => "&",
                BinaryOp::Or => "|",
                BinaryOp::Xor => "^",
                _ => unreachable!(),
            };
            Ok((
                format!(
                    "((int)((long)round((double){left_text}) {c_op} (long)round((double){right_text})))"
                ),
                false,
            ))
        }
        // `NOT` is bitwise complement, not boolean negation -- `NOT 1` is
        // `-2`, not `0` (the manual's own Logical Operators section makes a
        // point of this exact example, since it surprises anyone expecting
        // C-style `!`). Same round-to-integer step as AND/OR/XOR above.
        Expr::Unary {
            op: UnaryOp::Not,
            expr,
        } => {
            let (inner, _) = render_numeric_expr(expr, needs_math, functions)?;
            *needs_math = true;
            Ok((format!("((int)(~(long)round((double){inner})))"), false))
        }
        // A call to a user-defined numeric-returning BASCAL function --
        // and a single-argument (or zero-argument) one parses as
        // `Expr::ArrayRef`, not `Expr::Call` (see `make_paren_ident_expr`
        // in `parser.rs`/`is_string_expr`'s own comment), so both shapes
        // route to the same rendering, `render_numeric_call`. `RND()`
        // (real BASIC's own zero-argument spelling, equivalent to
        // `RND(1)` -- see `RND_HELPER`) is the one *builtin* that can take
        // this same zero-argument shape, so it needs the same routing
        // even though it's never in `functions` (that table only holds
        // BASCAL-defined `function`/`procedure` declarations).
        // `grid%(r%, c%)` -- a multi-dimensional (2+ index) array read.
        // Real function-call syntax and 2+-argument array indexing are
        // syntactically identical, so a 2+-index array reference parses
        // as a plain `Expr::Call`, not `Expr::ArrayRef` -- unlike the
        // single/zero-index case, which is genuinely ambiguous with a
        // call at parse time (see `make_paren_ident_expr` in
        // `parser.rs`) and so parses as `Expr::ArrayRef` instead, handled
        // below. Checked before the real call arm right after, same
        // function-wins-the-ambiguity precedence as that arm's own.
        Expr::Call { name, args } if functions.arrays.contains_key(&array_c_name(name)) => {
            let c_expr = render_array_index_expr(name, args, needs_math, functions)?;
            let is_float = functions.arrays[&array_c_name(name)]
                .element_type
                .is_some_and(|(_, f)| f);
            Ok((c_expr, is_float))
        }
        Expr::ScalarMethodCall { base, method, args } => {
            render_numeric_method_call(base, method, args, needs_math, functions)
        }
        // A call to a user-defined numeric-returning BASCAL function --
        // and a single-argument (or zero-argument) one parses as
        // `Expr::ArrayRef`, not `Expr::Call` (see `make_paren_ident_expr`
        // in `parser.rs`/`is_string_expr`'s own comment), so both shapes
        // route to the same rendering, `render_numeric_call`. `RND()`
        // (real BASIC's own zero-argument spelling, equivalent to
        // `RND(1)` -- see `RND_HELPER`) is the one *builtin* that can take
        // this same zero-argument shape, so it needs the same routing
        // even though it's never in `functions` (that table only holds
        // BASCAL-defined `function`/`procedure` declarations).
        Expr::Call { name, args } => render_numeric_call(name, args, needs_math, functions),
        Expr::ArrayRef { name, indices }
            if functions.contains_key(&fn_key(name)) || name.name.eq_ignore_ascii_case("rnd") =>
        {
            render_numeric_call(name, indices, needs_math, functions)
        }
        // `arr%(i%)` -- a `dim`-declared numeric array's own indexed
        // element (see `ArrayInfo`/`render_array_index_expr`). Checked
        // after the function-call arm above -- a same-named `function`/
        // `RND` always wins the ambiguity, matching
        // `codegen_basic::expr`'s own identical precedence for real
        // BASIC array syntax (indistinguishable from a call there too).
        Expr::ArrayRef { name, indices } if functions.arrays.contains_key(&array_c_name(name)) => {
            let c_expr = render_array_index_expr(name, indices, needs_math, functions)?;
            let is_float = functions.arrays[&array_c_name(name)]
                .element_type
                .is_some_and(|(_, f)| f);
            Ok((c_expr, is_float))
        }
        _ => Err(
            "this expression isn't supported in a numeric context by the minimal C backend yet \
             -- render_numeric_expr only covers numeric literals, numeric scalar variables (%, \
             &, !, #), arithmetic, comparisons, AND/OR/XOR/NOT, function calls, and dim'd array \
             elements (a string variable is a type error here, not just unimplemented -- see \
             render_string_expr for string expressions)"
                .to_string(),
        ),
    }
}

/// A string argument to a function called from a *numeric* context
/// (`render_numeric_expr`'s own `Expr::Call` arm, or `LEN`/`ASC`'s own
/// string argument in `render_numeric_call`) -- restricted to shapes that
/// need no prelude of their own, since `render_numeric_expr` has no
/// prelude mechanism to route setup code (e.g. `+` concatenation's temp
/// buffer) through: a plain literal, a bare `$`-suffixed variable read,
/// or a `CHR$`/`MID$`/`LEFT$` call (these three are also prelude-free
/// expressions in their own right -- see `MID_HELPER`'s doc comment --
/// *if* their own arguments are too, checked recursively here). A
/// string-returning **user-defined** function called from a *string*
/// context doesn't have this restriction -- see `render_string_expr`'s
/// own `Expr::Call` arm, which does have a prelude to work with -- but
/// one called from here still does, since it uses the
/// out-parameter-plus-temp-buffer convention (`function_signature`'s
/// `bcc_out`), not the ring buffer.
fn render_prelude_free_string_arg(
    expr: &Expr,
    needs_math: &mut bool,
    functions: &FunctionTable,
) -> Result<String, String> {
    match expr {
        Expr::String(s) => Ok(format!("\"{}\"", escape_c_string_literal(s))),
        Expr::Ident(ident) if ident.suffix == Some(TypeSuffix::String) => {
            Ok(c_var_name(ident, TypeSuffix::String))
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("chr") && args.len() == 1 => {
            let (text, is_float) = render_numeric_expr(&args[0], needs_math, functions)?;
            let coerced = coerce_numeric(text, is_float, false, needs_math);
            Ok(format!("bcc_chr({coerced})"))
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if (name.name.eq_ignore_ascii_case("mid") && (args.len() == 2 || args.len() == 3))
            || (name.name.eq_ignore_ascii_case("left") && args.len() == 2) =>
        {
            let s_text = render_prelude_free_string_arg(&args[0], needs_math, functions)?;
            let is_left = name.name.eq_ignore_ascii_case("left");
            let (start_text, length_text) = if is_left {
                let (t, f) = render_numeric_expr(&args[1], needs_math, functions)?;
                ("1".to_string(), coerce_numeric(t, f, false, needs_math))
            } else {
                let (st, sf) = render_numeric_expr(&args[1], needs_math, functions)?;
                let start = coerce_numeric(st, sf, false, needs_math);
                let length = if args.len() == 3 {
                    let (lt, lf) = render_numeric_expr(&args[2], needs_math, functions)?;
                    coerce_numeric(lt, lf, false, needs_math)
                } else {
                    "2147483647".to_string()
                };
                (start, length)
            };
            Ok(format!("bcc_mid({s_text}, {start_text}, {length_text})"))
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("right") && args.len() == 2 => {
            let s_text = render_prelude_free_string_arg(&args[0], needs_math, functions)?;
            let (nt, nf) = render_numeric_expr(&args[1], needs_math, functions)?;
            let n_text = coerce_numeric(nt, nf, false, needs_math);
            Ok(render_right_call(&s_text, &n_text))
        }
        Expr::Call { name, args }
        | Expr::ArrayRef {
            name,
            indices: args,
        } if name.name.eq_ignore_ascii_case("str") && args.len() == 1 => {
            let (text, is_float) = render_numeric_expr(&args[0], needs_math, functions)?;
            Ok(if is_float {
                format!("bcc_strd({text})")
            } else {
                format!("bcc_stri({text})")
            })
        }
        // `country$(i%)` / a 2+-index `grid$(r%, c%)` (the latter parses
        // as `Expr::Call`, not `Expr::ArrayRef` -- see the identical
        // arm's own comment in `render_numeric_expr`) -- a `dim`-declared
        // string array's own indexed element (see `ArrayInfo`), the same
        // shape a string comparison (`country$(i%) > country$(i% + 1)`,
        // `strcmp`'s own operands -- see this function's own call site
        // above) needs just as much as a plain string variable does.
        Expr::ArrayRef { name, indices } | Expr::Call { name, args: indices }
            if functions
                .arrays
                .get(&array_c_name(name))
                .is_some_and(|info| info.element_type.is_none()) =>
        {
            render_array_index_expr(name, indices, needs_math, functions)
        }
        _ => Err(
            "a string argument to a function called from a numeric context must be a plain \
             string literal, string variable, dim'd string array element, or \
             CHR$/MID$/LEFT$/STR$ call (no concatenation or user-defined function calls) -- not \
             supported by the minimal C backend yet"
                .to_string(),
        ),
    }
}

/// Escapes `value` as the body of a plain C string literal (no surrounding
/// quotes) -- correct for a string used as an ordinary argument (an
/// `snprintf` source, an assignment value, a concatenation operand: see
/// `render_string_expr`). Deliberately a separate function from
/// `codegen_basic::escape_string`, not a shared one: BASIC string literals
/// have no backslash escapes at all (a literal `"` is doubled, that's the
/// entire rule), while C needs `\"`, `\\`, and control bytes escaped.
/// Reusing the BASIC escaper here would silently emit invalid/wrong C the
/// moment a string contained a backslash or an unescaped control byte.
///
/// NOT correct for embedding text directly into a `printf`-style format
/// string itself (as opposed to one of its arguments) -- that additionally
/// needs a literal `%` doubled to `%%`, which this deliberately does NOT
/// do (a string used as a plain value, e.g. `grade$ = "100%"`, must keep
/// its single `%` -- doubling it here would be a correctness bug in the
/// other direction). See `escape_c_format_text` for the format-string case.
fn escape_c_string_literal(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Everything `escape_c_string_literal` does, plus doubling a literal `%`
/// to `%%` -- for text embedded directly into a `printf`-style format
/// string (see `render_print_tokens`'s `PrintToken::Expr(Expr::String(s))`
/// case), where an unescaped `%` would otherwise be read as a format
/// specifier instead of literal text -- a correctness bug (wrong output at
/// best, mismatched varargs / crash at worst).
fn escape_c_format_text(value: &str) -> String {
    escape_c_string_literal(value).replace('%', "%%")
}

/// Same rule as `codegen_basic::ends_with_end`: the last statement that
/// isn't a comment or blank line must be `end` for the program to already
/// have emitted its own `return 0;`.
fn ends_with_end(statements: &[Stmt]) -> bool {
    statements
        .iter()
        .rev()
        .find(|s| {
            !matches!(&***s, Statement::BlankLine | Statement::BlockComment(_))
                && !matches!(&***s, Statement::Raw(text) if text.trim_start().starts_with('\''))
        })
        .is_some_and(|s| matches!(&**s, Statement::End))
}

fn unsupported(message: &str) -> Diagnostic {
    Diagnostic::error(SourcePos::new("<target>", 1, 1), message.to_string())
}
