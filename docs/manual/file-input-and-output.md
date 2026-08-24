[Home](../) / [Manual](../manual/) / File Input and Output

[← Input and Output](input-and-output.md) [Random-Access File I/O →](random-access-file-io.md)

<div class="prose" markdown="1">

From `tutorial/10_files.bcl`:

### OPEN

Opens a file for reading, writing, or appending.

```bascal
OPEN filename$ FOR INPUT  AS #1
OPEN filename$ FOR OUTPUT AS #2
OPEN filename$ FOR APPEND AS #3
```

The file number (`#1`, `#2`, etc.) is used in subsequent file I/O statements.

### CLOSE

Closes an open file.

```bascal
CLOSE #1
```

### KILL

Deletes a file from disk.

```bascal
kill "temp.dat"
kill tempFile$
```

Generates `KILL filename$`. The file must exist or a runtime error occurs.

### NAME ... AS

Renames (or moves) a file.

```bascal
name "old.dat" as "new.dat"
name srcFile$ as destFile$
```

Generates `NAME old AS new`. Both arguments are expressions; string variables or literals work equally well.

### WRITE \# and INPUT

`WRITE #` stores values in a quoted, comma-separated format that `INPUT #` can read back reliably:

```bascal
csvFile$ = "tutorial_scores.csv"

OPEN csvFile$ FOR OUTPUT AS #1
WRITE #1, "Alice", 95, "pass"
WRITE #1, "Bob",   54, "fail"
WRITE #1, "Carol", 78, "pass"
CLOSE #1

OPEN csvFile$ FOR APPEND AS #1
WRITE #1, "Dave", 88, "pass"
CLOSE #1

PRINT "Records in " + csvFile$ + ":"
OPEN csvFile$ FOR INPUT AS #1
while EOF(1) = 0
    INPUT #1, name$, score%, result$
    PRINT "  " + name$ + ": " + STR$(score%) + "  [" + result$ + "]"
end while
CLOSE #1
```

Output:

```bascal
Records in tutorial_scores.csv:
  Alice: 95  [pass]
  Bob: 54  [fail]
  Carol: 78  [pass]
  Dave: 88  [pass]
```

### LINE INPUT

Reads one complete line (including commas) from a file into a string variable:

```bascal
OPEN csvFile$ FOR INPUT AS #1
while EOF(1) = 0
    LINE INPUT #1, line$
    PRINT "  " + line$
end while
CLOSE #1
```

### PRINT \# (File Print)

Writes expressions to a file without the quoting that `WRITE #` adds:

```bascal
PRINT #2, "Header line"
PRINT #2, count%, value!
```

### PRINT USING

Formats output with a template string before printing to the screen, printer, or a file. The format string uses MS-BASIC format characters (`#` for digit positions, `.` for the decimal point, `,` for thousands separator, `+`/`-` for sign, etc.).

```bascal
print using "####.##"; amount!          ' screen
lprint using "####.##"; amount!         ' printer
print #1, using "####.##"; amount!      ' file channel #1
```

Multiple values are separated by `;` or `,` exactly like a normal `PRINT`:

```bascal
print using "Item ##: ####.##"; itemNo%, price!
```

The format string is any string expression; it does not have to be a literal:

```bascal
fmt$ = "###.#"
print using fmt$; x!; y!; z!
```

### The file-handle DSL

From Part 2 of `tutorial/10_files.bcl`. `file <var> = open(<path>) for input|output|append` is sugar over the same `OPEN ... AS #n` above it, except the compiler allocates and remembers the channel number itself, and `<var>.write(...)`/`<var>.read(...)`/`<var>.eof()`/`<var>.close()` replace every subsequent reference to `#n`. It shares its channel-numbering with the [record/file DSL](record-files.md)'s own `file ... as ... = open(...)` — every `file` declaration in a program, sequential or record, is numbered in one sequence, in declaration order, so the two forms can be mixed in the same program without colliding.

```bascal
file out = open(csvFile$) for output
out.write("Alice", 95, "pass")
out.close()

file scores = open(csvFile$) for input
while not scores.eof()
    scores.read(name$, score%, result$)
    print "  " + name$ + ": " + str$(score%) + "  [" + result$ + "]"
end while
scores.close()
```

Transpiles to exactly `OPEN csvFile$ FOR OUTPUT AS #1` / `WRITE #1, "Alice", 95, "pass"` / `CLOSE #1`, then `OPEN csvFile$ FOR INPUT AS #2` / a `WHILE NOT (EOF(2))` loop around `INPUT #2, name$, score%, result$` / `CLOSE #2` — the same statements the hand-written form above uses, just without a literal channel number anywhere in the source.

`.write(...)`/`.eof()`/`.read(...)` are each rejected at transpile time, not merely at runtime, when used against a file opened the wrong way:

- `.write(...)` needs a file opened `for output` or `for append`.
- `.read(...)` and `.eof()` each need a file opened `for input`.
- Any of the three against a *record* file (`file db as Student = open(...)`) is also a transpile-time error — those files use `db[i]`/`.field` instead, covered in [Record Files](record-files.md).

`.close()` is the one method valid on either kind of `file` — sequential or record — and generates a plain `CLOSE #n` either way.

There is currently no DSL sugar for `PRINT #`, `PRINT #` ... `USING`, or `LINE INPUT #` — write those against the file's own channel number, obtained by opening it with a raw `OPEN ... AS #n` instead of `file ... = open(...)` if a program needs both forms together.

</div>

[← Input and Output](input-and-output.md) [Random-Access File I/O →](random-access-file-io.md)
