## A fixed collection, declared once

`dim` gives an array a name and a shape. Classic BASIC has no usable `REDIM` for compiled programs, so an array’s size stays fixed for the program’s lifetime.

```bascal
dim scores%(30)
declare i%, total%

for i% = 0 to 30
    scores%(i%) = i% * 2
    total% = total% + scores%(i%)
end for

print "elements: "; sizeof(scores%)
print "total: "; total%
```

`dim scores%(30)` declares indices 0 through 30: 31 elements. `sizeof(scores%)` reads the bound, so a loop does not need to repeat it. Every array is indexed from base 0 — `OPTION BASE` is rejected outright as a transpile-time error under both targets, not something to reach for. A second axis works the same way: `dim grid%(2, 2)`; use `sizeof(grid%, 0)` or `sizeof(grid%, 1)`. `declare` is another spelling of `dim`.

A scalar normally needs no declaration because BASIC creates it on first use. `--strict-vars` requires declarations and catches misspelled names, at the cost of no longer being a strict superset of BASIC.

## Passing an array without losing its size

A function parameter accepts an array of unknown size by using `?` for each axis. BASCAL works out the storage needed from all call sites and makes the real bound available through `sizeof`.

```bascal
function sumArr%(byref arr%(?))
    declare i%, total%
    for i% = 0 to sizeof(arr%)
        total% = total% + arr%(i%)
    end for
    return total%
end function

dim values%(4)
values%(0) = 10
values%(1) = 20
print sumArr%(values%)
```

Pass an array with its bare name—never `()` and never a separately typed count. `byval`, the default, copies the array in. `byref` copies it in and copies changes back after the call, because classic BASIC has no direct reference to the caller’s array. Use `byref` only when the function changes the array.

## Strings are text, not character arrays

A BASCAL string is a single value with the `$` suffix, not an array indexed by character. `+` joins strings; built-in functions read and rebuild substrings.

```bascal
declare name$
name$ = "Ada Lovelace"
print left$(name$, 3) + " " + mid$(name$, 5, 8)
print instr(name$, "Love")
mid$(name$, 1, 3) = "Amy"
print name$
```

`left$(s$, n)` and `right$(s$, n)` take the first or last `n` characters. `mid$(s$, start, len)` takes characters from a 1-based position; omit `len` to take the rest. `instr(s$, needle$)` returns the first 1-based match position, or `0`. `len`, `str$`, `val`, `chr$`, and `asc` convert between numbers, characters, and text.

`mid$(...)` also has a statement form: `mid$(name$, 1, 3) = "Amy"` overwrites characters in place. It is the one case where the same name behaves differently on the left and right sides of `=`.

<div class="aside" markdown="1">

`ltrim$`, `rtrim$`, `ucase$`, and `lcase$` are BASCAL libraries, not MBASIC/BASCOM builtins. See [The standard library](standard-library.md) for their `require` lines.

</div>

[← Making control flow visible](control-flow.md)[Functions, procedures, and methods →](functions-and-procedures.md)
