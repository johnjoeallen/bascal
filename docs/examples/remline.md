[Home](../../) / [Examples](sort-driver.md) / REMLINE

# REMLINE

[REMLINE](https://github.com/johnjoeallen/bascal/tree/main/examples/remline) analyses a line-numbered BASIC program and removes unnecessary line numbers while preserving real jump targets. Parsing, reference collection, transformation, and string helpers are separate dotted-path libraries within the example folder.

```text
bcc examples/remline/remline.bcl -L examples/remline
fbc -lang qb examples/remline/remline.bas -x tmp/remline
./tmp/remline
diff -u examples/remline/sample/expected.bas examples/remline/sample/output.bas
```

A clean run produces no diff output: `output.bas` matches the checked-in expected listing.

[← Sort Driver](sort-driver.md) [Next: Card Catalog →](card-catalog.md)
