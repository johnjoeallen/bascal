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
//! `STRING_BUFFER_SIZE`/`render_string_expr`), `+` string concatenation,
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
//! `build_function_table`/`emit_function_def`), a suffixless (default-typed)
//! numeric variable (real MBASIC/BASCOM's own unoverridden default,
//! single-precision -- see `effective_suffix`), twenty-five BASIC
//! intrinsics implemented natively -- `LEN`, `ASC`, `CHR$`, `MID$`,
//! `LEFT$`, `RIGHT$`, `STR$`, `VAL`, `INSTR`, `SQR`, `ABS`, `INT`, `FIX`,
//! `SGN`, `CINT`, `CLNG`, `CSNG`, `CDBL`, `SIN`, `COS`, `TAN`, `ATN`,
//! `LOG`, `EXP`, `RND` (see `render_numeric_call`/`render_string_call`/
//! `MID_HELPER`/`INSTR_HELPER`/`SGN_HELPER`/`RND_HELPER`) -- plus
//! `RANDOMIZE` (a statement, not an expression; see `Statement::Randomize`'s
//! own handling in `emit_statement`) -- and
//! random-access record I/O: `OPEN ... FOR RANDOM`/`BINARY`, `CLOSE`,
//! `FIELD`, `GET`/`PUT` (whole-record form only), `LSET`/`RSET`, and
//! `MKI$`/`MKL$`/`MKS$`/`MKD$`/`CVI`/`CVL`/`CVS`/`CVD` (see
//! `FileIoLayout`/`apply_field_statement`/`emit_get_or_put`/`FILE_IO_HELPER`
//! -- two real, documented divergences from real MBASIC/BASCOM live
//! there: `MKS$`/`MKD$`/`CVS`/`CVD` use plain IEEE 754 instead of real
//! BASIC's Microsoft Binary Format, and multi-byte values are packed in
//! the host's native byte order, assumed little-endian). NOT yet
//! supported: `procedure` (no return value), `byref`/array parameters, a
//! function body that doesn't provably `return` on every path (see
//! `body_always_returns`), sequential file I/O (`OPEN FOR
//! INPUT`/`OUTPUT`/`APPEND`), and a `FIELD`/`OPEN`/`GET`/`PUT` channel or
//! `FIELD` width that isn't a literal integer -- all rejected with a
//! diagnostic rather than guessed at. Recursion (direct or indirect) is
//! rejected at the resolver level before codegen ever runs, for every
//! target, not just this one. Everything else (other statement kinds,
//! arrays, any BASIC intrinsic beyond the six above) reports a "not
//! supported yet" diagnostic rather than panicking or emitting wrong code
//! -- this is a walking skeleton to prove the CLI/dispatch plumbing
//! (`Target::C`, `--target c`, `invoke_gcc`) end-to-end, not a real
//! backend. Tutorials that compile end to end today: `tutorial/01_hello.bcl`,
//! `tutorial/03_arithmetic.bcl`, `tutorial/04_conditions.bcl`,
//! `tutorial/05_loops.bcl`, `tutorial/06_select_case.bcl`,
//! `tutorial/07_functions.bcl` (including its two `require`d
//! `com.bascal.stdlib` library functions, `ucase$`/`lcase$` -- library
//! merging itself needed no C-backend-specific work at all, since
//! `lib.rs`'s `require`/`import` resolution already merges a required
//! file's functions into `Program.functions` before either backend's
//! codegen ever runs), and `tutorial/15_random_and_record_files.bcl`
//! (both its hand-written Part 1 and DSL-based Part 2 -- gcc-compiled
//! and run, every value matches).
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

use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::ast::{
    BasicIdent, BinaryOp, CaseClause, CaseValue, DoCondition, Expr, FunctionDef, OpenMode,
    ParamMode, PrintToken, Program, RecordFieldType, Statement, TypeSuffix, UnaryOp,
};
use crate::diagnostics::{Diagnostic, SourcePos};

/// One user-defined function's C-callable shape, built by
/// `build_function_table` before any codegen runs -- looked up by
/// `Expr::Call` sites (in `render_numeric_expr`/`render_string_expr`) via
/// `fn_key`, same (name, suffix) keying `codegen_basic::same_ident` uses,
/// since a call site's own suffix is part of its identifier syntax and
/// must match the declaration's exactly.
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
}

struct FnParam {
    /// The C identifier used for this parameter *inside the function
    /// body* -- for a string parameter, that's the local buffer holding
    /// the function's own byval copy, NOT the raw incoming pointer
    /// parameter (see `emit_function_def`'s copy-in preamble).
    c_name: String,
    is_string: bool,
    /// Only meaningful when `!is_string`.
    is_float: bool,
}

type FunctionTable = HashMap<(String, Option<TypeSuffix>), FnSig>;

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
    };
    let mut visit = |expr: &Expr| {
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
const MID_HELPER: &str = "#define BCC_STRBUF_COUNT 8\nstatic char bcc_strbuf[BCC_STRBUF_COUNT][256];\nstatic int bcc_strbuf_next = 0;\n\nstatic char* bcc_strbuf_take(void) {\n    char* buf = bcc_strbuf[bcc_strbuf_next];\n    bcc_strbuf_next = (bcc_strbuf_next + 1) % BCC_STRBUF_COUNT;\n    return buf;\n}\n\nstatic const char* bcc_mid(const char* s, int start, int length) {\n    char* out = bcc_strbuf_take();\n    int len = (int)strlen(s);\n    int from = start - 1;\n    if (from < 0) from = 0;\n    if (from > len) from = len;\n    int avail = len - from;\n    if (length < 0) length = 0;\n    if (length > avail) length = avail;\n    snprintf(out, 256, \"%.*s\", length, s + from);\n    return out;\n}\n\nstatic const char* bcc_chr(int code) {\n    char* out = bcc_strbuf_take();\n    snprintf(out, 256, \"%c\", code);\n    return out;\n}\n\nstatic const char* bcc_stri(int value) {\n    char* out = bcc_strbuf_take();\n    snprintf(out, 256, \"% d\", value);\n    return out;\n}\n\nstatic const char* bcc_strd(double value) {\n    char* out = bcc_strbuf_take();\n    snprintf(out, 256, \"% g\", value);\n    return out;\n}\n\n";

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
const FILE_IO_HELPER: &str = "#define BCC_MAX_CHANNELS 32\nstatic FILE* bcc_files[BCC_MAX_CHANNELS];\n\nstatic void bcc_read_string_field(char* field, const unsigned char* source, size_t width) {\n    memcpy(field, source, width);\n    field[width] = 0;\n    while (width > 0 && field[width - 1] == ' ') field[--width] = 0;\n}\n\nstatic void bcc_mki(char* out, int value) {\n    int16_t v = (int16_t)value;\n    memcpy(out, &v, 2);\n}\n\nstatic void bcc_mkl(char* out, int value) {\n    int32_t v = (int32_t)value;\n    memcpy(out, &v, 4);\n}\n\nstatic void bcc_mks(char* out, double value) {\n    float v = (float)value;\n    memcpy(out, &v, 4);\n}\n\nstatic void bcc_mkd(char* out, double value) {\n    memcpy(out, &value, 8);\n}\n\nstatic int bcc_cvi(const char* s) {\n    int16_t v;\n    memcpy(&v, s, 2);\n    return (int)v;\n}\n\nstatic int bcc_cvl(const char* s) {\n    int32_t v;\n    memcpy(&v, s, 4);\n    return (int)v;\n}\n\nstatic float bcc_cvs(const char* s) {\n    float v;\n    memcpy(&v, s, 4);\n    return v;\n}\n\nstatic double bcc_cvd(const char* s) {\n    double v;\n    memcpy(&v, s, 8);\n    return v;\n}\n\nstatic int bcc_read_record(FILE* file, void* buffer, size_t reclen, long record) {\n    if (fseek(file, (record - 1) * (long)reclen, SEEK_SET) != 0) return 0;\n    return fread(buffer, 1, reclen, file) == reclen;\n}\n\nstatic void bcc_write_record(FILE* file, const void* buffer, size_t reclen, long record) {\n    fseek(file, (record - 1) * (long)reclen, SEEK_SET);\n    fwrite(buffer, 1, reclen, file);\n}\n\nstatic void bcc_pad_string_field(unsigned char* dest, const char* value, size_t width) {\n    size_t len = strlen(value);\n    if (len > width) len = width;\n    memcpy(dest, value, len);\n    memset(dest + len, ' ', width - len);\n}\n\n";

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
const COLOR_HELPER: &str = "static const int bcc_ansi_fg[16] = {30, 34, 32, 36, 31, 35, 33, 37, 90, 94, 92, 96, 91, 95, 93, 97};\nstatic const int bcc_ansi_bg[8] = {40, 44, 42, 46, 41, 45, 43, 47};\n\nstatic void bcc_color(int fg, int bg) {\n    printf(\"\\x1b[%dm\", bcc_ansi_fg[fg & 15]);\n    if (bg >= 0) {\n        printf(\"\\x1b[%dm\", bcc_ansi_bg[bg & 7]);\n    }\n}\n\n";

/// `input [prompt$;] var` -- reads one whole line into a shared,
/// fixed-size scratch buffer (matching every string in this backend
/// already being a fixed `char[STRING_BUFFER_SIZE]`), stripping the
/// trailing newline `fgets` leaves in. Every `INPUT` in the program reuses
/// this same buffer -- safe because each `Statement::Input` fully consumes
/// it (parses it into the target variable) before the next one runs; there
/// is never a live reference to a stale read left lying around.
const INPUT_HELPER: &str = "static char bcc_input_buf[256];\n\nstatic void bcc_read_line(void) {\n    if (fgets(bcc_input_buf, sizeof(bcc_input_buf), stdin) == NULL) {\n        bcc_input_buf[0] = 0;\n        return;\n    }\n    bcc_input_buf[strcspn(bcc_input_buf, \"\\r\\n\")] = 0;\n}\n\n";

/// `INSTR(s$, needle$)` -- the 1-based position of the first match, or 0.
/// Scoped to this 2-argument form only, matching what `docs/language/
/// arrays-and-strings.html` documents -- real BASCOM's optional leading
/// `start%` argument (`INSTR(start%, s$, needle$)`) isn't implemented,
/// since nothing in this repo exercises it. `strstr` already does the
/// actual search; this just converts its pointer result to BASIC's
/// 1-based index convention (or 0 for "not found", instead of C's `NULL`).
const INSTR_HELPER: &str = "static int bcc_instr(const char* s, const char* needle) {\n    const char* found = strstr(s, needle);\n    return found ? (int)(found - s) + 1 : 0;\n}\n\n";

/// `SGN(x)` -- -1/0/1 by the sign of `x`. No single C library function
/// does this (unlike `SQR`/`ABS`/`INT`/`FIX`, which map straight onto
/// `sqrt`/`fabs`/`floor`/`trunc`), so it gets a small helper of its own.
const SGN_HELPER: &str = "static int bcc_sgn(double v) {\n    if (v > 0) return 1;\n    if (v < 0) return -1;\n    return 0;\n}\n\n";

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
const RND_HELPER: &str = "static double bcc_rnd_last = 0.0;\n\nstatic double bcc_rnd(double x) {\n    if (x < 0) {\n        srand((unsigned int)(-x));\n    }\n    if (x != 0) {\n        bcc_rnd_last = (double)rand() / ((double)RAND_MAX + 1.0);\n    }\n    return bcc_rnd_last;\n}\n\n";

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
const SEQ_FILE_HELPER: &str = "static int bcc_eof(FILE* file) {\n    int c = fgetc(file);\n    if (c == EOF) return -1;\n    ungetc(c, file);\n    return 0;\n}\n\nstatic void bcc_line_input_file(FILE* file, char* buf, size_t bufsize) {\n    if (fgets(buf, (int)bufsize, file) == NULL) {\n        buf[0] = 0;\n        return;\n    }\n    buf[strcspn(buf, \"\\r\\n\")] = 0;\n}\n\nstatic void bcc_read_file_field(FILE* file, char* buf, size_t bufsize) {\n    int c = fgetc(file);\n    while (c == ' ') c = fgetc(file);\n    size_t len = 0;\n    if (c == '\"') {\n        c = fgetc(file);\n        while (c != EOF && c != '\"') {\n            if (len + 1 < bufsize) buf[len++] = (char)c;\n            c = fgetc(file);\n        }\n        c = fgetc(file);\n        while (c != EOF && c != ',' && c != '\\n') c = fgetc(file);\n    } else {\n        while (c != EOF && c != ',' && c != '\\n' && c != '\\r') {\n            if (len + 1 < bufsize) buf[len++] = (char)c;\n            c = fgetc(file);\n        }\n        if (c == '\\r') {\n            int c2 = fgetc(file);\n            if (c2 != '\\n' && c2 != EOF) ungetc(c2, file);\n        }\n    }\n    buf[len] = 0;\n}\n\nstatic char bcc_file_field_buf[256];\n\n";

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
    /// Accumulated C source for every synthesized record helper, spliced
    /// into `generate`'s output right after `FILE_IO_HELPER` once emission
    /// of the whole program is done.
    helper_defs: String,
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
    fn collect(statements: &[Statement], layouts: &mut HashMap<Vec<u32>, (String, Vec<bool>)>) {
        for statement in statements {
            match statement {
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
fn program_has_statement(program: &Program, pred: &dyn Fn(&Statement) -> bool) -> bool {
    fn walk(statements: &[Statement], pred: &dyn Fn(&Statement) -> bool) -> bool {
        statements.iter().any(|statement| {
            pred(statement)
                || match statement {
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
fn count_gosubs(statements: &[Statement]) -> usize {
    statements
        .iter()
        .map(|statement| {
            let self_count = usize::from(matches!(statement, Statement::Gosub(_)));
            let nested_count = match statement {
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
                _ => 0,
            };
            self_count + nested_count
        })
        .sum()
}

fn program_uses_color(program: &Program) -> bool {
    program_has_statement(program, &|s| matches!(s, Statement::Color { .. }))
}

fn program_uses_input(program: &Program) -> bool {
    program_has_statement(program, &|s| matches!(s, Statement::Input { .. }))
}

/// Whether `program` has any `RANDOMIZE` at all -- decides whether
/// `generate()` needs `<stdlib.h>` for `srand()` (independent of whether
/// `RND` itself is ever called -- see `scan_builtin_usage`'s own
/// `needs_stdlib_h` set for that half).
fn program_uses_randomize(program: &Program) -> bool {
    program_has_statement(program, &|s| matches!(s, Statement::Randomize(_)))
}

/// Whether `program` has a `RANDOMIZE` that needs `time(NULL)` -- bare
/// `RANDOMIZE` or `RANDOMIZE TIMER` (see `Statement::Randomize`'s own
/// handling in `emit_statement` for why both fall back to the same
/// time-based seed) -- as opposed to `RANDOMIZE <numeric seed>`, which
/// needs no `<time.h>` at all.
fn program_uses_randomize_time(program: &Program) -> bool {
    program_has_statement(program, &|s| {
        matches!(s, Statement::Randomize(None))
            || matches!(
                s,
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
            s,
            Statement::Write { .. }
                | Statement::InputFile { .. }
                | Statement::LineInput { .. }
                | Statement::PrintFile { .. }
        ) || matches!(
            s,
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
    fn walk(statements: &[Statement], layout: &mut FileIoLayout) -> Result<(), String> {
        for statement in statements {
            match statement {
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
/// comment), neither are `byref` or array parameters (real
/// pass-by-reference/array-decay semantics are a real chunk of work,
/// deferred), and a function whose body doesn't end with an explicit
/// `return` on its last top-level statement is rejected outright rather
/// than guessing a fallback value -- unlike the BASIC backend (which can
/// fall back on "whatever the shared result variable last held," matching
/// real MBASIC/BASCOM's own GOSUB-without-RETURN behavior), a real C
/// function falling off the end without `return`-ing a value is undefined
/// behavior, not a defined-if-surprising fallback.
fn build_function_table(functions: &[FunctionDef]) -> Result<FunctionTable, String> {
    let mut table = FunctionTable::new();
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
        let mut params = Vec::with_capacity(func.params.len());
        for param in &func.params {
            if param.mode != ParamMode::ByVal {
                return Err(format!(
                    "`byref` parameters aren't supported by the minimal C backend yet (`{}`'s \
                     parameter `{}`) -- only byval scalar parameters are",
                    func.name, param.name
                ));
            }
            if param.axes.is_some() {
                return Err(format!(
                    "array parameters aren't supported by the minimal C backend yet (`{}`'s \
                     parameter `{}`)",
                    func.name, param.name
                ));
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
        table.insert(
            fn_key(&func.name),
            FnSig {
                c_name: function_c_name(&func.name),
                is_void: func.is_procedure,
                is_string,
                is_float: numeric.is_some_and(|(_, f)| f),
                params,
            },
        );
    }
    Ok(table)
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
fn body_always_returns(body: &[Statement]) -> bool {
    let last = body.iter().rev().find(|s| {
        !matches!(s, Statement::BlankLine | Statement::BlockComment(_))
            && !matches!(s, Statement::Raw(text) if text.trim_start().starts_with('\''))
    });
    match last {
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
fn collect_global_decl_idents(body: &[Statement], out: &mut Vec<BasicIdent>) {
    for stmt in body {
        match stmt {
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

pub(crate) fn generate(program: &Program) -> Result<String, Vec<Diagnostic>> {
    let functions =
        build_function_table(&program.functions).map_err(|message| vec![unsupported(&message)])?;
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

    let mut prototypes = String::new();
    let mut function_defs = String::new();
    for func in &program.functions {
        let sig = &functions[&fn_key(&func.name)];
        prototypes.push_str(&function_signature(func, sig));
        prototypes.push_str(";\n");
        emit_function_def(
            func,
            sig,
            &functions,
            &mut function_defs,
            &mut needs_math,
            &mut needs_string,
            &mut temp_counter,
            &mut function_view,
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

    let gosub_count = count_gosubs(&program.statements);
    let mut gosub_id: usize = 0;
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
    let needs_seq_io =
        builtin_usage.needs_seq_file_helper || program_uses_sequential_file_io(program);
    let needs_randomize = program_uses_randomize(program);
    let needs_randomize_time = program_uses_randomize_time(program);

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
    } else if builtin_usage.needs_stdlib_h || needs_input || needs_randomize {
        // `input`'s numeric targets parse via `atoi`/`atof` (see
        // `INPUT_HELPER`'s call site in `emit_statement`); `RANDOMIZE`
        // needs `srand()` (see `Statement::Randomize`'s own handling in
        // `emit_statement`), independent of whether `RND` itself
        // (`builtin_usage.needs_stdlib_h`) is ever called.
        includes.push_str("#include <stdlib.h>\n");
    }
    if needs_randomize_time {
        includes.push_str("#include <time.h>\n");
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

    let mut out = includes;
    out.push('\n');
    if builtin_usage.needs_ring_buffer_helpers {
        out.push_str(MID_HELPER);
    }
    if builtin_usage.needs_instr_helper {
        out.push_str(INSTR_HELPER);
    }
    if builtin_usage.needs_sgn_helper {
        out.push_str(SGN_HELPER);
    }
    if builtin_usage.needs_rnd_helper {
        out.push_str(RND_HELPER);
    }
    if gosub_count > 0 {
        out.push_str(GOSUB_HELPER);
    }
    if file_io.used {
        out.push_str(FILE_IO_HELPER);
        out.push_str(&file_io.helper_defs);
    }
    if needs_seq_io {
        out.push_str(SEQ_FILE_HELPER);
    }
    if needs_color {
        out.push_str(COLOR_HELPER);
    }
    if needs_input {
        out.push_str(INPUT_HELPER);
    }
    if !globals_decl.is_empty() {
        out.push_str(&globals_decl);
        out.push('\n');
    }
    if !prototypes.is_empty() {
        out.push_str(&prototypes);
        out.push('\n');
    }
    out.push_str(&function_defs);
    out.push_str(&format!(
        "int main(void) {{\n{}}}\n",
        reindent_c_body(&body)
    ));
    Ok(out)
}

/// One function's C prototype/definition header -- `<ret> <name>(<params>)`,
/// no trailing `;`/body, shared by the forward-declaration pass and
/// `emit_function_def` so the two can never drift out of sync with each
/// other. A string-returning function is actually `void`-returning in C
/// (see the module doc comment/`emit_function_def`): its BASCAL return
/// value comes out through an extra trailing `char* bcc_out` parameter
/// instead, matching the buffer-out convention already used by every
/// other string value in this backend. A string parameter's C parameter
/// is `const char*` named `<c_name>_in` -- not `<c_name>` itself -- because
/// the function body needs its own byval-copied local under the plain
/// `<c_name>` (see `emit_function_def`'s copy-in preamble): aliasing the
/// parameter directly would let the callee mutate the caller's buffer,
/// breaking byval semantics.
fn function_signature(func: &FunctionDef, sig: &FnSig) -> String {
    let ret_type = if sig.is_void || sig.is_string {
        "void"
    } else {
        numeric_c_type(func.name.suffix.expect("validated by build_function_table"))
            .expect("validated by build_function_table")
            .0
    };
    let mut params: Vec<String> = func
        .params
        .iter()
        .zip(&sig.params)
        .map(|(param, fp)| {
            if fp.is_string {
                format!("const char* {}_in", fp.c_name)
            } else {
                let suffix = param
                    .name
                    .suffix
                    .expect("validated by build_function_table");
                let (c_type, _) =
                    numeric_c_type(suffix).expect("validated by build_function_table");
                format!("{c_type} {}", fp.c_name)
            }
        })
        .collect();
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
    out: &mut String,
    needs_math: &mut bool,
    needs_string: &mut bool,
    temp_counter: &mut usize,
    file_io: &mut FileIoLayout,
) -> Result<(), String> {
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
    numeric_locals.retain(|k, _| !param_keys.contains(k) && !global_keys.contains(k));
    string_locals.retain(|k| !param_keys.contains(k) && !global_keys.contains(k));

    let mut body = String::new();
    for fp in &sig.params {
        if fp.is_string {
            body.push_str(&format!("    char {0}[{STRING_BUFFER_SIZE}];\n", fp.c_name));
            body.push_str(&format!(
                "    snprintf({0}, sizeof({0}), \"%s\", {0}_in);\n",
                fp.c_name
            ));
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
    if sig.params.iter().any(|p| p.is_string)
        || !numeric_locals.is_empty()
        || !string_locals.is_empty()
    {
        body.push('\n');
    }

    // GOSUB is scoped to top-level code only (see `Statement::Gosub`'s own
    // doc comment) -- a function/procedure body never legally reaches the
    // GOSUB arm's `gosub_id`/`gosub_count` reads at all, since it errors
    // out immediately whenever `current_function` is `Some` (always true
    // here), so a throwaway `0`/local counter is safe.
    let mut unused_gosub_id: usize = 0;
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
        )?;
    }

    out.push_str(&function_signature(func, sig));
    out.push_str(&format!(" {{\n{}}}\n\n", reindent_c_body(&body)));
    Ok(())
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
    statement: &Statement,
    numeric_out: &mut BTreeMap<String, &'static str>,
    string_out: &mut BTreeSet<String>,
) {
    match statement {
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
        _ => {}
    }
}

fn register_var(
    ident: &BasicIdent,
    numeric_out: &mut BTreeMap<String, &'static str>,
    string_out: &mut BTreeSet<String>,
) {
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
    statement: &Statement,
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
) -> Result<(), String> {
    match statement {
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
                OpenMode::Random | OpenMode::Binary => {
                    out.push_str(&format!(
                        "    bcc_files[{idx}] = fopen({file_text}, \"rb+\");\n"
                    ));
                    out.push_str(&format!(
                        "    if (!bcc_files[{idx}]) bcc_files[{idx}] = fopen({file_text}, \"wb+\");\n"
                    ));
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
        // `INPUT #`'s comma-delimited fields need. Scoped to a bare string
        // variable target, matching keyboard `INPUT`'s own restriction.
        Statement::LineInput { channel, target } => {
            let ch = literal_channel(channel)?;
            let idx = ch - 1;
            let Expr::Ident(ident) = target else {
                return Err(
                    "LINE INPUT #'s target isn't supported by the minimal C backend yet -- only \
                     a bare string variable is"
                        .to_string(),
                );
            };
            if ident.suffix != Some(TypeSuffix::String) {
                return Err(
                    "LINE INPUT # requires a string (`$`-suffixed) variable".to_string(),
                );
            }
            let c_name = c_var_name(ident, TypeSuffix::String);
            out.push_str(&format!(
                "    bcc_line_input_file(bcc_files[{idx}], {c_name}, sizeof({c_name}));\n"
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
                if is_string_expr(expr) {
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
            let is_rset = matches!(statement, Statement::Rset { .. });
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
                out.push_str("    return;\n");
            } else {
                let (text, is_float) = render_numeric_expr(value, needs_math, functions)?;
                let coerced = coerce_numeric(text, is_float, sig.is_float, needs_math);
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
            out.push_str("    return;\n");
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
            if args.len() != sig.params.len() {
                return Err(format!(
                    "`{name}` expects {} argument(s), got {}",
                    sig.params.len(),
                    args.len()
                ));
            }
            let mut prelude = Vec::new();
            let mut arg_texts = Vec::with_capacity(args.len());
            for (arg, param) in args.iter().zip(&sig.params) {
                if param.is_string {
                    let (arg_prelude, text) =
                        render_string_expr(arg, needs_math, temp_counter, functions)?;
                    prelude.extend(arg_prelude);
                    arg_texts.push(text);
                } else {
                    let (text, is_float) = render_numeric_expr(arg, needs_math, functions)?;
                    arg_texts.push(coerce_numeric(text, is_float, param.is_float, needs_math));
                }
            }
            for line in prelude {
                out.push_str(&line);
            }
            if sig.is_string {
                let temp = format!("bt_s_{temp_counter}");
                *temp_counter += 1;
                out.push_str(&format!("    char {temp}[{STRING_BUFFER_SIZE}];\n"));
                arg_texts.push(temp);
            }
            out.push_str(&format!("    {}({});\n", sig.c_name, arg_texts.join(", ")));
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
        // `label:`/`goto label` -- Phase 1 of raw BASIC's label-based
        // control flow (see the C-target labels/GOTO tracking issue):
        // close to a 1:1 mapping onto C's own `goto`/label, since both
        // languages have the identical primitive. `gosub`/`return` and
        // `on error goto`/`resume` need a real "remember where to resume"
        // execution model C's `goto` doesn't give for free, so they're
        // deliberately still unsupported (falling through to the generic
        // error below) -- a separate, larger piece of work.
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
        // ordinary variable -- like every other zero-arg pseudo-variable
        // (`ERR`/`ERL`/`DATE$`/...), it isn't otherwise supported by this
        // backend yet.
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
        other => Err(format!(
            "{other:?} is not supported by the minimal C backend yet -- only `print`, `end`, \
             `dim`, `if`, `for`, `while`, `do`, `exit`, `select case`, `return`, a bare \
             function/procedure call, and assignment/`const` of scalar variables (%, &, !, #, \
             $) are implemented so far"
        )),
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
    else_body: &[Statement],
    out: &mut String,
    needs_math: &mut bool,
    needs_string: &mut bool,
    temp_counter: &mut usize,
    functions: &FunctionTable,
    current_function: Option<&FnSig>,
    file_io: &mut FileIoLayout,
    gosub_count: usize,
    gosub_id: &mut usize,
) -> Result<(), String> {
    let is_string = is_string_expr(expr);
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
    if args.len() != sig.params.len() {
        return Err(format!(
            "`{name}` expects {} argument(s), got {}",
            sig.params.len(),
            args.len()
        ));
    }
    let mut prelude = Vec::new();
    let mut arg_texts = Vec::with_capacity(args.len());
    for (arg, param) in args.iter().zip(&sig.params) {
        if param.is_string {
            let (arg_prelude, text) = render_string_expr(arg, needs_math, temp_counter, functions)?;
            prelude.extend(arg_prelude);
            arg_texts.push(text);
        } else {
            let (text, is_float) = render_numeric_expr(arg, needs_math, functions)?;
            arg_texts.push(coerce_numeric(text, is_float, param.is_float, needs_math));
        }
    }
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
fn render_string_expr(
    expr: &Expr,
    needs_math: &mut bool,
    temp_counter: &mut usize,
    functions: &FunctionTable,
) -> Result<(Vec<String>, String), String> {
    match expr {
        Expr::String(s) => Ok((Vec::new(), format!("\"{}\"", escape_c_string_literal(s)))),
        Expr::Ident(ident) if ident.suffix == Some(TypeSuffix::String) => {
            Ok((Vec::new(), c_var_name(ident, TypeSuffix::String)))
        }
        Expr::Binary {
            op: BinaryOp::Add,
            left,
            right,
        } if is_string_expr(left) || is_string_expr(right) => {
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
        Expr::Call { name, args } if name.suffix == Some(TypeSuffix::String) => {
            render_string_call(name, args, needs_math, temp_counter, functions)
        }
        Expr::ArrayRef { name, indices }
            if name.suffix == Some(TypeSuffix::String) && is_known_callable(name, functions) =>
        {
            render_string_call(name, indices, needs_math, temp_counter, functions)
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
            PrintToken::Expr(expr) if is_string_expr(expr) => {
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

fn render_numeric_call(
    name: &BasicIdent,
    args: &[Expr],
    needs_math: &mut bool,
    functions: &FunctionTable,
) -> Result<(String, bool), String> {
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
    if args.len() != sig.params.len() {
        return Err(format!(
            "`{name}` expects {} argument(s), got {}",
            sig.params.len(),
            args.len()
        ));
    }
    let mut arg_texts = Vec::with_capacity(args.len());
    for (arg, param) in args.iter().zip(&sig.params) {
        if param.is_string {
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
fn render_numeric_expr(
    expr: &Expr,
    needs_math: &mut bool,
    functions: &FunctionTable,
) -> Result<(String, bool), String> {
    match expr {
        Expr::Integer(n) => Ok((n.to_string(), false)),
        Expr::Float(f) => Ok((format!("{f:?}"), true)),
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
            if is_string_expr(left) || is_string_expr(right) {
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
        Expr::Call { name, args } => render_numeric_call(name, args, needs_math, functions),
        Expr::ArrayRef { name, indices }
            if functions.contains_key(&fn_key(name)) || name.name.eq_ignore_ascii_case("rnd") =>
        {
            render_numeric_call(name, indices, needs_math, functions)
        }
        _ => Err(
            "this expression isn't supported in a numeric context by the minimal C backend yet \
             -- render_numeric_expr only covers numeric literals, numeric scalar variables (%, \
             &, !, #), arithmetic, comparisons, AND/OR/XOR/NOT, and function calls (a string \
             variable is a type error here, not just unimplemented -- see render_string_expr \
             for string expressions); arrays aren't supported in either context yet"
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
        _ => Err(
            "a string argument to a function called from a numeric context must be a plain \
             string literal, string variable, or CHR$/MID$/LEFT$/STR$ call (no concatenation or \
             user-defined function calls) -- not supported by the minimal C backend yet"
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
fn ends_with_end(statements: &[Statement]) -> bool {
    statements
        .iter()
        .rev()
        .find(|s| {
            !matches!(s, Statement::BlankLine | Statement::BlockComment(_))
                && !matches!(s, Statement::Raw(text) if text.trim_start().starts_with('\''))
        })
        .is_some_and(|s| matches!(s, Statement::End))
}

fn unsupported(message: &str) -> Diagnostic {
    Diagnostic::error(SourcePos::new("<target>", 1, 1), message.to_string())
}
