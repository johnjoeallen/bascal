[Home](../../) / [Manual](../) / Statement Quick Reference

[← Command-Line Reference](command-line-reference.md) [Standard Library Functions →](standard-library-functions.md)

<div class="prose" markdown="1">

| Statement       | Syntax                                                                  | Description                                                                                           |
|-----------------|-------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------|
| `BEEP`          | `BEEP`                                                                  | Sound the system bell                                                                                 |
| `CLEAR`         | `CLEAR`                                                                 | Reset all variables and close all files                                                               |
| `CLS`           | `CLS`                                                                   | Clear the screen                                                                                      |
| `CLOSE`         | `CLOSE #n`                                                              | Close file channel *n*                                                                                |
| `COLOR`         | `COLOR fg[, bg]`                                                        | Set foreground/background colour                                                                      |
| `CONST`         | `CONST name = expr`                                                     | Declare a named constant                                                                              |
| `DATA`          | `DATA val[, ...]`                                                       | Embed literal data values                                                                             |
| `DIM`           | `DIM name[(d1[, d2, ...])][, name2...]`                                 | Declare one or more variables or 1-D/multi-D arrays                                                   |
| `ERASE`         | `ERASE arr[, ...]`                                                      | Free memory used by arrays                                                                            |
| `DO`            | `DO [WHILE/UNTIL cond]` … `END DO`, or `DO` … `LOOP [WHILE/UNTIL cond]` | Pre-check or post-check conditional loop                                                              |
| `END`           | `END`                                                                   | End of program                                                                                        |
| `EXIT`          | `exit`                                                                  | Exit the innermost enclosing FOR/WHILE/DO loop                                                        |
| `FOR`           | `FOR v = start TO end [STEP s]` … `END FOR`                             | Counted loop                                                                                          |
| `FUNCTION`      | `FUNCTION name%(params)` … `END FUNCTION`                               | Define a function with a return value                                                                 |
| Label           | `name:`                                                                 | Declare a branch target for GOTO/GOSUB/ON.../RESUME                                                   |
| `GOSUB`         | `GOSUB label`                                                           | Call BASIC subroutine                                                                                 |
| `GOTO`          | `GOTO label`                                                            | Unconditional branch                                                                                  |
| `IF`            | `IF cond THEN` … \[`ELSEIF` …\] \[`ELSE` …\] `END IF`                   | Conditional block                                                                                     |
| `INPUT`         | `INPUT [prompt;] var[, ...]`                                            | Read from keyboard                                                                                    |
| `KILL`          | `KILL file$`                                                            | Delete a file                                                                                         |
| `INPUT #`       | `INPUT #n, var[, ...]`                                                  | Read from file                                                                                        |
| `LET`           | `LET var = expr`                                                        | Assignment (keyword optional)                                                                         |
| Compound assign | `var += / -= / *= / /= expr`                                            | Assignment shorthand for `var = var op expr`                                                          |
| `MID$` (stmt)   | `MID$(str$, start[, len]) = repl$`                                      | In-place substring replacement                                                                        |
| `LINE INPUT`    | `LINE INPUT #n, var$`                                                   | Read full line from file                                                                              |
| `LOCATE`        | `LOCATE row, col`                                                       | Position cursor                                                                                       |
| `LPRINT`        | `LPRINT expr[, ...]`                                                    | Print to printer                                                                                      |
| `NAME`          | `NAME old$ AS new$`                                                     | Rename a file                                                                                         |
| `OPTION BASE`   | `OPTION BASE 0\|1`                                                      | Not supported (and unlikely to ever be) — see [OPTION BASE](variables-and-constants.md#option-base) |
| `ON...GOTO`     | `ON expr GOTO label1, label2, ...`                                      | Computed GOTO                                                                                         |
| `ON...GOSUB`    | `ON expr GOSUB label1, label2, ...`                                     | Computed GOSUB                                                                                        |
| `ON ERROR GOTO` | `ON ERROR GOTO label`                                                   | Install error handler (`ON ERROR GOTO 0` disables)                                                    |
| `ERROR`         | `ERROR n`                                                               | Trigger runtime error code *n*                                                                        |
| `RESUME`        | `RESUME` / `RESUME NEXT` / `RESUME label`                               | Resume after error handler                                                                            |
| `TRY` / `CATCH` | `try ... catch err%(errFileNotFound%), erl%, source$ ... end try`    | Portable error recovery; optional code filter plus code, line, and source file; no `resume`          |
| `THROW`         | `throw` / `throw code%`                                                 | Re-raise the current error, or raise a new one, from inside `catch` (portable across both targets)    |
| `OPEN`          | `OPEN file$ FOR INPUT/OUTPUT/APPEND AS #n`                              | Open file                                                                                             |
| `OUT`           | `OUT port, val`                                                         | Write byte to hardware I/O port                                                                       |
| `POKE`          | `POKE address, val`                                                     | Write byte to memory address                                                                          |
| `PRINT`         | `PRINT expr[, ...]`                                                     | Print to screen                                                                                       |
| `PROGRAM`       | `program name` / `program name shared sharedname`                       | Declare this file as a runnable program (mandatory in the file passed to `bcc`)                       |
| `LIBRARY`       | `library name`                                                          | Declare this file as a library module (mandatory in every `require`/`import` target)                  |
| `PROCEDURE`     | `PROCEDURE name(params)` … `END PROCEDURE`                              | Define a procedure (no return value)                                                                  |
| `PRINT #`       | `PRINT #n, expr[, ...]`                                                 | Print to file                                                                                         |
| `PRINT USING`   | `PRINT USING fmt$; expr[; ...]`                                         | Formatted print (also `LPRINT USING`, `PRINT #n, USING`)                                              |
| `RANDOMIZE`     | `RANDOMIZE [seed]`                                                      | Seed random number generator                                                                          |
| `READ`          | `READ var[, ...]`                                                       | Read from DATA stream                                                                                 |
| `REQUIRE`       | `require path.symbol`                                                   | Load dependency module                                                                                |
| `RESTORE`       | `RESTORE [label]`                                                       | Reset DATA pointer                                                                                    |
| `RETURN`        | `RETURN expr` / `RETURN`                                                | Return value from function; bare form exits a procedure early                                         |
| `SELECT CASE`   | `SELECT CASE expr` … `END SELECT`                                       | Multi-way branch                                                                                      |
| `STOP`          | `STOP`                                                                  | Stop program execution                                                                                |
| `SHARED`        | `shared name`                                                           | Declare this file as the shared-variables file `name` (shared vars listed via `dim`)                  |
| `SWAP`          | `SWAP a, b`                                                             | Exchange two variable values                                                                          |
| `SYSTEM`        | `SYSTEM`                                                                | Exit to operating system                                                                              |
| `WHILE`         | `WHILE cond` … `END WHILE` (or `WEND`)                                  | Condition-at-top loop                                                                                 |
| `WIDTH`         | `WIDTH [#n,] cols`                                                      | Set line width for console or file channel                                                            |
| `WRITE #`       | `WRITE #n, expr[, ...]`                                                 | Write to file (quoted format)                                                                         |

### System pseudo-variables (no parentheses)

| Name    | Type   | Returns                        |
|---------|--------|--------------------------------|
| `DATE$` | String | Current date as `MM-DD-YYYY`   |
| `TIME$` | String | Current time as `HH:MM:SS`     |
| `TIMER` | Single | Seconds elapsed since midnight |

### Boolean literals

| Name    | Transpiles to |
|---------|---------------|
| `TRUE`  | `-1`          |
| `FALSE` | `0`           |

</div>

[← Command-Line Reference](command-line-reference.md) [Standard Library Functions →](standard-library-functions.md)
