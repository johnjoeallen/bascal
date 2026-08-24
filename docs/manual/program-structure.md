[Home](../../) / [Manual](../) / Program Structure

[← Getting Started](getting-started.md) [Data Types and Type Suffixes →](data-types-and-type-suffixes.md)

<div class="prose" markdown="1">

Every `.bcl` file is exactly one of three things, declared by a mandatory header on its first non-comment, non-blank line:

| Header         | File is a...                                                                  | May be `require`d?                                         | May itself `require`? |
|----------------|-------------------------------------------------------------------------------|------------------------------------------------------------|-----------------------|
| `program name` | runnable program (the file you hand to `bcc`)                                 | no                                                         | yes                   |
| `library name` | library module                                                                | yes — only files with this header may                      | yes                   |
| `shared name`  | shared-variables file (see [Shared COMMON](shared-common.md#shared-common)) | no (resolved via `program ... shared name`, not `require`) | no                    |

A file with no header, or with more than one of these, is a transpile-time error. `require`/`import` targets a file that must declare `library`; a `program name shared sharedname` clause resolves its shared file through a separate lookup, not through `require`.

Beyond the header, a `.bcl` file consists of optional sections in the following order:

1.  Mandatory `program` / `library` / `shared` declaration
2.  `require` / `import` dependency declarations (`program`/`library` files only)
3.  Top-level statements (the main program body; a `shared` file's body is `dim` declarations only — every variable in it is COMMON by default, see [Shared COMMON](shared-common.md#shared-common) — and a `library` file should stick to `function`/`procedure` definitions and supporting `dim`/`data`, see [Module Conventions](dependencies-require-and-import.md#module-conventions))
4.  `function` definitions (may appear in any order relative to statements)

### Program Declaration

```bascal
program name
program name shared sharedname
```

Identifies the file as a runnable program, by name, and optionally links it to a shared-variables file (see [Shared COMMON](shared-common.md#shared-common)). Required in every file that isn't a `library` or `shared` file — in particular, the file passed to `bcc` on the command line must have one.

A `program` declaration is **not allowed** in library modules loaded via `require`.

### Library Declaration

```bascal
library name
```

Identifies the file as a library module — the only kind of file `require`/`import` may load. From `com/bascal/stdlib/ucase.bcl`:

```bascal
// Upper-cases s$. Not a real MBASIC/BASCOM 2.00 builtin -- verified against
// a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships its own.
library ucase

function ucase$(s$)
    ...
```

The name isn't validated against anything (unlike `shared name`, which must match the resolved shared file's filename) — it's documentation, not a lookup key. A `library` declaration is **not allowed** in the root file `bcc` was invoked on, and a file `require`d/`import`ed without one is a transpile-time error (see [Module Conventions](dependencies-require-and-import.md#module-conventions)).

### File Encoding

Source files are UTF-8 text. Line endings may be LF or CRLF. Statements are separated by newlines; a colon `:` may also separate statements on one line.

There is no line-continuation syntax -- no trailing `_`, `\`, or similar. The lexer turns every physical newline into a real token, so any single expression (and the tokens making up a statement header, like a `case` value list) must fit on one physical line; a newline there ends the statement/expression rather than continuing it. This applies everywhere, not just to obviously statement-like constructs -- a function call's argument list, for example, can't have a newline before its closing `)` either:

```bascal
' Not allowed -- the newline after "1," ends the statement early:
result% = someFunction(1,
                        2)

' Allowed -- keep the whole call on one line:
result% = someFunction(1, 2)
```

The same rule governs statement headers like `if`/`then`: the condition and the `then` that closes it must be on one physical line. A newline can only appear *after* `then`, where it's meaningful -- it's what selects the block form of `if` over the single-line form (see [IF / ELSEIF / ELSE / END IF](control-flow.md#if-elseif-else-end-if)):

```bascal
' Not allowed -- the newline before "then" ends the statement early:
if score% >= 60
    then PRINT "Pass"

' Not allowed -- same problem, condition split across lines:
if score% >= 60 and
    attendance% >= 80 then PRINT "Pass"

' Allowed -- condition and "then" on one line, body starts after the newline:
if score% >= 60 and attendance% >= 80 then
    PRINT "Pass"
end if
```

</div>

[← Getting Started](getting-started.md) [Data Types and Type Suffixes →](data-types-and-type-suffixes.md)
