[Home](../../) / Examples / Sort Driver

# Sort Driver

The [Sort Driver](https://github.com/johnjoeallen/bascal/tree/main/examples/sort_driver) exercises recursive `require`, array argument passing, and timing. It fills 5000 reverse-sorted elements and runs bubble, shaker, shell, and quick sort implementations loaded by dotted path from its own `com/bascal/sort/` library tree.

```text
bcc examples/sort_driver/sort_driver.bcl -L examples/sort_driver
fbc -lang qb examples/sort_driver/sort_driver.bas -x tmp/sort_driver
./tmp/sort_driver
```

Timings vary by machine; each sort reports `OK` when it has produced sorted output.

[Next: REMLINE →](remline.md)
