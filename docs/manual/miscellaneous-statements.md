[Home](../../) / [Manual](../) / Miscellaneous Statements

[← Data Statements](data-statements.md) [Dependencies — REQUIRE and IMPORT →](dependencies-require-and-import.md)

<div class="prose" markdown="1">

### MID\$ (statement form)

Overwrites a run of characters inside a string with a same-length replacement — `target$` keeps its original length; see [MID\$ assignment](standard-library-functions.md#mid-assignment) for how this actually gets transpiled.

```bascal
mid$(target$, start[, length]) = replacement$
```

`start` is 1-based. The optional `length` caps how many characters are replaced; if omitted, replacement continues to the end of `target$` or until `replacement$` runs out of characters, whichever comes first.

```bascal
s$ = "Hello World"
mid$(s$, 7, 5) = "BASIC"   ' s$ → "Hello BASIC"
mid$(s$, 1)    = "Goodbye"  ' s$ → "GoodbyeBASIC" (no length cap)
```

This is distinct from the `mid$()` *function*, which extracts a substring without modifying the original. BASCAL handles the statement form as an ordinary assignment whose left-hand side is `mid$(...)`.

### SWAP

Exchanges the values of two variables — no explicit temporary needed.

From `tutorial/09_data.bcl`:

```bascal
a% = 42
b% = 17
PRINT "Before SWAP: a=" + STR$(a%) + " b=" + STR$(b%)
SWAP a%, b%
PRINT "After SWAP:  a=" + STR$(a%) + " b=" + STR$(b%)
' Before SWAP: a=42 b=17
' After SWAP:  a=17 b=42
```

SWAP works on strings and array elements too:

```bascal
SWAP first$, last$               ' exchange string variables
SWAP country$(i%), country$(j%)  ' exchange array elements (used in bubble sort)
```

### RANDOMIZE

Seeds the random number generator. With no argument, the runtime may prompt for a seed or use a default.

```bascal
RANDOMIZE           ' prompt or default
RANDOMIZE TIMER     ' time-based seed for different sequences each run
RANDOMIZE 99        ' fixed seed for reproducible output
```

### Labels

```bascal
name:
```

Declares a branch target for `goto`/`gosub`/`on error goto`/`resume`/ `on ... goto`/`on ... gosub` to jump to. **BASCAL manages line numbers itself — you cannot target a raw line number.** Every one of those statements requires a label name; the transpiler assigns the actual BASIC line number when it renders output, exactly the way it already numbers the branch targets inside `if`/`while`/`do`/`select case`.

```bascal
goto skip
print "not reached"
skip:
print "reached"
```

A label can share its line with the statement that follows it — the `:` doubles as that statement's separator, same as anywhere else in BASCAL:

```bascal
skip: print "reached"
```

### GOTO

Transfers control to a label. Prefer `if`, loops, and functions; `GOTO` is primarily useful for error handlers.

```bascal
GOTO doCleanup
```

### GOSUB / RETURN (BASIC-level)

Calls a BASIC subroutine at a label. Note this is the raw BASIC `GOSUB`, distinct from the function-call mechanism BASCAL generates internally.

```bascal
GOSUB writeLog
```

### ON ... GOTO / ON ... GOSUB

Computed branch: the integer expression selects the *n*th target (1-based). Each target is a label, not a line number.

```bascal
ON choice% GOTO firstCase, secondCase, thirdCase
ON mode%   GOSUB modeIdle, modeRun, modeError
```

If the expression evaluates to 0 or exceeds the number of targets, execution continues with the next statement. Still accepted, but see [Legacy-Form Warnings](#legacy-form-warnings) below — `select case` expresses the same dispatch structurally.

### Legacy-Form Warnings

BASCAL stays a strict superset of classic BASIC — every legacy form on this page keeps compiling exactly as it always has. But a few of them have a direct, unambiguous BASCAL structured equivalent, and `bcc` names it as an advisory warning printed to stderr (never a compile error) whenever it sees one, to steer new or edited source toward the structured spelling:

- `ON ... GOTO` / `ON ... GOSUB` — prefer `select case`.
- a chain of two or more `IF ... THEN GOTO` / `ELSEIF ... THEN GOTO` links, each branching to a different label — prefer `select case`. A single, unchained `IF cond THEN GOTO label` (an ordinary early-exit/error-check branch) is left alone.
- a `GOTO` back to a label already seen earlier in the same block — a hand-wired loop — prefer `do ... loop` or `while ... end while`.
- a hand-typed `FIELD` statement — prefer the [`record`/`file`](record-files.md) DSL, which expresses the same random-access layout without manually tracking `FIELD`/`LSET`/`GET`/`PUT` offsets.

Nothing else on this page is warned about — `GOTO`/`GOSUB` to a label, labels themselves, and `ON ERROR GOTO`/`RESUME`/`ERROR` have no BASCAL structured equivalent, so they're accepted silently.

### ON ERROR GOTO / RESUME / ERROR

MS-BASIC structured error handling: trap runtime errors, handle them, and resume execution.

#### ON ERROR GOTO

Installs an error handler at a given label. Any subsequent runtime error causes execution to jump there. `ON ERROR GOTO 0` is the one place a numeric argument is still legal — `0` isn't a line number, it's the sentinel that disables the trap.

```bascal
on error goto errHandler   ' jump to errHandler on any error
on error goto 0            ' disable the error trap
```

The target can be a raw `name:` label or a `procedure`. A `procedure` target gets extra compile-time checking, because it's reached with a plain `GOTO` (never a `GOSUB`), so there's no call frame for `RETURN` to pop: bcc rejects any `return` inside such a procedure's body, and rejects the body unless every path is proven to end in `resume`/`resume next`/`resume <label>` (an `if`/`select case` only counts if every branch, including a mandatory `else`/`case else`, diverges the same way). A procedure that passes both checks also can't be called like an ordinary procedure anywhere else in the program — it's proven to never return, so it could never come back to a normal caller. See `errorTrap()` in `tutorial/inventory.bcl` for a worked example.

#### RESUME

Resumes execution after an error handler has run.

```bascal
resume             ' retry the statement that caused the error
resume next         ' continue at the statement after the failing one
resume afterError   ' jump to a specific label
```

`RESUME` without an argument retries the failing statement (useful for recoverable errors like "disk full — retry after making space"). `RESUME NEXT` skips the failing statement.

#### ERROR

Triggers a runtime error with the given code, as if that error occurred naturally. Useful for re-raising an error in a handler or testing.

```bascal
error 53     ' simulate "file not found"
error code%  ' variable error code
```

#### ERR and ERL

Inside an error handler, the system pseudo-variables `err` and `erl` hold the error code and the BASIC line number where the error occurred. Write them without a type suffix; BASIC treats them as numeric:

```bascal
on error goto handleErr
' ...
goto afterErr
handleErr:
' reached via ON ERROR GOTO
if err = 53 then
    print "File not found"
    resume next
end if
error err   ' re-raise unhandled errors
afterErr:
end
```

#### Typical pattern

```bascal
on error goto errHandler
open fileName$ for input as #1
' ... file processing ...
close #1
on error goto 0
goto done

errHandler:
if err = 53 then
    print "Cannot open "; fileName$
    resume next
else
    error err
end if

done:
end
```

### TRY / CATCH / THROW

Portable structured error recovery: works unchanged under both `--target basic` and `--target C`. A failed statement anywhere in the `try` body abandons the rest of it and runs `catch` once, then always runs the rest of the program after `end try` — never back inside `try`. There is no `resume` equivalent: recovery always continues right after `end try`, not at the failing statement or "next".

```bascal
try
    open fileName$ for input as #1
    ' ... file processing ...
    close #1
catch err%(errFileNotFound%, errFileAlreadyOpen%), erl%, source$
    print "Error "; err%; " at line "; erl%
end try
```

`err%`, `erl%`, and `source$` are ordinary locals scoped to the `catch` block alone — not aliases for the ambient `err`/`erl` pseudo-variables `on error goto` uses above. They're populated once, at the start of `catch`, with the error code, line, and `.bcl` source file that raised the error, and go out of scope at `end try`.

Attach a parenthesized, comma-separated filter list to the error-code variable to handle only selected codes. After `require com.bascal.stdlib.error`, use the library’s named common-error constants: `catch err%(errFileNotFound%, errFileAlreadyOpen%), erl%, source$`. The list accepts numeric expressions, not only literals. If no filter expression matches the raised error, BASCAL skips the catch body and re-raises the error after `finally` (or immediately when there is no `finally`). `catch` is optional; a `try` with only a `finally` body still abandons `try` on error, it just has nothing custom to run before `finally` does.

#### THROW

`throw` is the portable equivalent of `ERROR err` above — most often used inside a `catch` to re-raise an error it doesn't know how to handle, so it keeps propagating instead of being silently swallowed. With no argument it rethrows whatever error is currently active; with an expression, it raises that code as a new error instead.

```bascal
try
    open fileName$ for input as #1
catch err%, erl%
    if err% = 53 then
        print "Cannot open "; fileName$
    else
        throw          ' not a case this catch handles -- keep propagating
    end if
end try

throw 53   ' raise a new error, same as `error 53`
```

### POKE

Writes a byte value to a memory address. The address is an integer expression.

```bascal
poke &H0400, 3      ' write to segment-zero address using hex literal
poke address%, val%
```

Generates `POKE address, value`. Use with care — writing to arbitrary addresses is hardware-specific and may crash the runtime.

### OUT

Writes a byte to a hardware I/O port. Syntax mirrors `POKE`.

```bascal
out 888, 3          ' send value 3 to port 888 (parallel port control)
out port%, val%
```

Generates `OUT port, value`. Port numbers and semantics are hardware-specific.

### WIDTH

Sets the output line width for the screen or an open file channel.

```bascal
width 80            ' set console line width to 80 characters
width 40            ' narrow console mode
width #1, 132       ' set line width for file channel #1
```

The optional `#n,` prefix targets a file channel; without it, the console width is set. Generates `WIDTH cols` or `WIDTH #n, cols`.

### CLEAR

Resets all program variables to their zero/empty defaults and closes all open files. Useful at the start of a program or before reinitialising state.

```bascal
clear
```

Generates `CLEAR`.

### DATE<span class="math inline">, *TIME*</span>, TIMER

Read-only system pseudo-variables that return the current date, time, and elapsed time. No parentheses — they are used as plain identifiers.

```bascal
print "Today is "; date$         ' e.g.  06-11-2026
print "Time:    "; time$         ' e.g.  14:35:02
randomize timer                  ' time-based RNG seed
```

| Name    | Returns                                         |
|---------|-------------------------------------------------|
| `DATE$` | Current date as `MM-DD-YYYY` string             |
| `TIME$` | Current time as `HH:MM:SS` string               |
| `TIMER` | Seconds since midnight (single-precision float) |

These are passed through verbatim as `DATE$`, `TIME$`, and `TIMER` in the generated BASIC.

### STOP

Terminates the program immediately; may invoke the debugger in some implementations.

```bascal
STOP
```

### SYSTEM

Exits to the operating system immediately.

```bascal
SYSTEM
```

### END

Signals the end of the main program body. Functions are emitted after `END` in the generated output.

```bascal
END
```

</div>

[← Data Statements](data-statements.md) [Dependencies — REQUIRE and IMPORT →](dependencies-require-and-import.md)
