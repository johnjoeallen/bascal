# REMLINE Example

`examples/remline` is a real-world BASCAL example inspired by old BASIC
line-number utilities such as `REMLINE.BAS`.

It demonstrates BASCAL's fit for utility-style programs that operate over
line-numbered BASIC source, while keeping the current version intentionally
simple:

- the sample input is embedded as data because file I/O is not yet part of the
  compiler/runtime surface
- the program keeps the core logic split across small `require`d helper files
- comments are retained in generated BASIC, so the lowered output stays readable
- the program analyses direct numeric references in `GOTO`, `GOSUB`, `THEN`,
  `ELSE`, `RESTORE`, `RESUME`, and `RUN`
- referenced target lines stay numbered
- unreferenced ordinary lines lose their line numbers

## Build

```bash
cargo run -- examples/remline/remline.bcl -L examples/remline -o output/remline/remline.bas
```

## Run

Compile the generated BASIC with FreeBASIC in QB mode:

```bash
fbc -lang qb output/remline/remline.bas -x tmp/remline
./tmp/remline > examples/remline/sample/output.bas
```

Compare against the expected output:

```bash
diff -u examples/remline/sample/expected.bas examples/remline/sample/output.bas
```

## Current Behavior

The first version prints the transformed sample listing to standard output.
That is enough to validate the line-number removal algorithm, and it avoids
depending on file I/O before BASCAL has that support end-to-end.
