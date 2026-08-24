[Home](../../) / [Manual](../) / Shared COMMON

[← Dependencies — REQUIRE and IMPORT](dependencies-require-and-import.md) [Generated BASIC Shape →](generated-basic-shape.md)

<div class="prose" markdown="1">

In classic BASCOM programs, multiple programs chained together with `CHAIN` share variables through `COMMON` declarations. For this to work correctly, every program in the chain must declare **identical** `COMMON` lists — the variable positions in the `COMMON` block must match exactly.

BASCAL coordinates `COMMON` through shared files. A shared file contains only `dim` declarations (see below) — every variable in it is COMMON by default, with no separate keyword needed to opt in — and programs that use it reference it with a `shared` clause on their `program` declaration.

### Shared File

A shared file is a `.bcl` file containing only `dim` declarations (see [DIM Declaration](#dim-declaration) below), plus blank lines and comments.

It starts with a mandatory `shared <name>` header, analogous to a regular file's `program <name>` header, and declares its shared variables with ordinary `dim`:

From `tutorial/13_shared/state.bcl`:

```bascal
/*
 * Shared file for Tutorial 13 — COMMON / CHAIN.
 *
 * Every program that begins with "program name shared state" receives
 * an identical COMMON block at the top of its generated BASIC, so the
 * listed variables survive a CHAIN to the next program.
 */
shared state

declare count%
declare label$
```

Rules for shared files: - The `shared <name>` header is mandatory, and its name must match the filename the transpiler resolved it as (`state.bcl` → `shared state`). - Only `dim` declarations, blank lines, and comments are allowed. - `require`, `function`, executable statements, and `program`/`library` declarations are all rejected with a diagnostic error. - The shared file must contain at least one `dim` declaration. - A file may declare at most one of `program`, `library`, or `shared` — a shared file can't also be an ordinary program or library module.

### DIM Declaration

```bascal
shared state

dim count%
dim label$
dim scores%()
```

Inside a `shared <name>`-headed file, every top-level `dim` becomes one shared (COMMON) variable, in declaration order — exactly the [DIM](variables-and-constants.md#dim) statement used anywhere else in BASCAL, including its multi-name comma form (`dim count%, label$`) and array declarations (`dim scores%()`, empty-parens, same as a `COMMON` array). No bounds are stored either way — a shared file's `dim` only ever declares *that* a name is an array, not its size.

### Program Declaration with Shared File

```bascal
program start shared state
```

When a shared-file name is present, the transpiler: 1. Searches for `state.bcl` in the source file's directory (then `-L` paths). 2. Validates that the shared file contains only `dim` declarations. 3. Emits the `COMMON` lines at the very top of the generated `.bas` file, before any other output.

### Using a Shared File

From `tutorial/13_shared/` — two programs that share `count%` and `label$`:

**`state.bcl`** (shared file):

```bascal
shared state

declare count%
declare label$
```

**`start.bcl`** (program 1):

```bascal
program start shared state

label$ = "Counter demo"
count% = 0
count% = count% + 1
count% = count% + 1
count% = count% + 1

PRINT "Initialised: " + label$
PRINT "Count after 3 increments: " + STR$(count%)

/* CHAIN "show" */
END
```

**`show.bcl`** (program 2):

```bascal
program show shared state

PRINT "Label:  " + label$
PRINT "Count:  " + STR$(count%)
END
```

Both `start.bas` and `show.bas` begin with:

```bascal
COMMON count%, label$
```

ensuring that `CHAIN "show"` from `start.bas` leaves the variables in the correct slots. `CHAIN` names the compiled program, not the `.bas` source it came from — verified directly against real BASCOM 2.00 under dosbox-x: `CHAIN "show.bas"` tries to run the source text itself and corrupts, while `CHAIN "show"` (or the explicit `CHAIN "show.exe"`) correctly runs `show.exe` and carries `COMMON` across.

### Restrictions

- A `shared <name>` header is illegal everywhere except in a shared file being loaded as one — a stray `shared` header in an ordinary program or library module is a transpile error. A shared file without one is also an error — the header is mandatory.
- A `program` declaration is illegal in library modules (files loaded via `require`), and mandatory in the root file `bcc` was invoked on.
- A `library` declaration is illegal in the root file `bcc` was invoked on, and mandatory in every file loaded via `require`/`import`.
- A file may declare at most one of `program`, `library`, or `shared`.
- If the named shared file does not exist, the program transpiles without a `COMMON` block (no error). This allows incremental development.

</div>

[← Dependencies — REQUIRE and IMPORT](dependencies-require-and-import.md) [Generated BASIC Shape →](generated-basic-shape.md)
