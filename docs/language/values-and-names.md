## Type is part of the name

BASCAL follows the classic BASIC convention: a suffix says what a variable holds. Use `%` for integers, `$` for strings, `!` for single-precision numbers, `#` for doubles, and `&` for long integers. A variable needs no declaration to use — like classic BASIC, it springs into existence the first time you assign it — but `dim` (or its synonym `declare`) states its type up front, which a plain scalar assignment leaves the reader to work out from the suffix alone.

    const taxRate! = .23
    declare quantity%, description$, subtotal!

    quantity% = 3
    description$ = "notebooks"
    subtotal! = quantity% * 4.50
    print description$ + ": " + STR$(subtotal! * (1 + taxRate!))

The suffix travels with every use of the name. It is a small cost for a useful property: in a language with a BASIC target, the storage intent is visible at every call site.

## Expressions say what is computed

Arithmetic, comparisons, and string expressions read in the ordinary way. Parentheses make grouping explicit. BASCAL also supplies `TRUE` and `FALSE`; they map to BASIC’s conventional `-1` and `0`.

## Collections are declared, then indexed

    dim scores%(30)    // indices 0 through 30
    scores%(1) = 92
    scores%(2) = 87

Arrays, constants, conversion rules, and type suffixes are developed throughout the following chapters; each declaration should make both its storage and its intended use clear.

[← A first program](first-program.md)[Making control flow visible →](control-flow.md)
