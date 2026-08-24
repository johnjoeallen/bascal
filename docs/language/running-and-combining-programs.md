## Compiling and running from the command line

`bcc` is the whole toolchain: one invocation resolves every `require`, transpiles, and — if asked — compiles and runs the result.

    bcc hello.bcl                       # writes hello.bas next to it
    bcc hello.bcl -o build/              # writes build/hello.bas -- name inferred
    bcc hello.bcl -L lib --run          # add a library search path, then run it
    bcc hello.bcl --target c --binary   # emit C and compile it with gcc

`-o` always names a directory — existing, or just written with a trailing slash for one that doesn’t exist yet — never an exact output file path to spell out by hand. The file inside it is auto-named the same way an omitted `-o` would: the source’s own name, with the target’s extension. Missing parent directories are created along the way.

`-L dir` adds a directory `require` searches, beyond the source file’s own directory; repeat it for more than one.

`--target` (or `-t`) chooses `basic` or `c`; omit it and `bcc` falls back to whatever `BASCAL_TARGET` or a config file says, or `basic` failing all of that.

`--binary` additionally compiles the generated output with the target’s own toolchain (`fbc` for BASIC, `gcc` for C); `--run` implies `--binary` and then runs it, with the program’s own stdin/stdout/stderr connected directly to your terminal.

`bcc` skips redoing work that’s already up to date — pass `--clean` to force a full retranspile.

`--strict-vars` turns on Pascal-style mandatory declaration: every variable — any scalar or array, any type suffix, not just the arrays introduced in [Arrays and strings](arrays-and-strings.md) — must have a `dim`/`declare` (or be a `const`, a `for` loop’s own counter, or a function/procedure parameter), or the compile is rejected. `--strict-vars-warn` runs the identical check but only prints findings to stderr, without failing the build.

## Screen and console output

Beyond `print`, a handful of statements reach the console more directly — meaningful mainly for the BASIC target, where a real terminal or DOS screen is what’s listening.

    cls
    color 14, 1
    locate 1, 30
    print "  BASCAL DEMO  "
    beep

`cls` clears the screen; `color foreground, background` sets text colors from the classic CGA palette (0–15 foreground, 0–7 background); `locate row, col` positions the cursor (both 1-based); `beep` sounds the terminal bell. `lprint` sends output to the line printer instead of the screen, using the same argument list as `print`.

## Linking separately run programs together

Classic BASIC has a way for one program to hand off to another without either one going through disk I/O to pass data along: a `COMMON` block of variables, shared by name between programs that agree to declare it the same way, combined with `CHAIN` to actually transfer control. BASCAL structures the shared declaration itself, in its own small file.

    ' state.bcl
    shared state
    declare count%
    declare label$

    ' start.bcl
    program start shared state
    count% = 1
    label$ = "first"
    print count%; " "; label$
    end

A shared file declares nothing but variables — no functions, no executable statements — and its `shared <name>` header must match its own filename. Any program that wants those variables declares them with `program name shared sharedname`; the compiler emits one matching `COMMON` statement at the very top of the generated BASIC, ahead of everything else, so every program sharing that file agrees on the same layout.

`CHAIN "show"` — the classic statement that actually transfers control from one compiled program to another, carrying the `COMMON` block’s values across — isn’t a BASCAL statement in its own right; add it directly to the generated BASIC where a genuine handoff between two separately-run programs is needed. It names the compiled program, not the `.bas` it was compiled from: verified against real BASCOM 2.00 under DOSBox-X, `CHAIN "show.bas"` tries to run the source text itself and corrupts, while `CHAIN "show"` (or, spelled out, `CHAIN "show.exe"`) correctly runs the compiled program. BASCAL’s own part of the job is keeping both programs’ view of the shared variables honest, not deciding when to jump between them.

[← The standard library](standard-library.md)[The targets beneath the program →](the-basic-beneath.md)
