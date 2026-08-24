## Put reusable code in a library file

Use `require` for code a program needs. A dotted path is a file lookup path, not a runtime namespace.

```bascal
require com.bascal.sort.quickSort

quickSort(values%())
```

Put reusable code in a library file, then include that library with `require`. The compiler resolves the dotted path beneath its search paths: dots become directory separators, so this example identifies `com/bascal/sort/quickSort.bcl`. It then incorporates the specified library code into the generated program. This Java-inspired path mechanism keeps dependencies named and located consistently while producing one complete program.

This keeps a larger project simple: source files have useful locations, while the generated BASIC remains compatible with the global model of its runtime. Functions, procedures, methods, and dependencies each have precise declaration and call rules; use them consistently so that a program’s interfaces remain visible in its source.
