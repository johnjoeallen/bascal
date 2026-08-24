[Home](../../) / [Manual](../) / Comments

[← Operators and Expressions](operators-and-expressions.md) [Control Flow →](control-flow.md)

<div class="prose" markdown="1">

### Single-Line Comments

A single quote `'` or a double slash `//` begins a comment that extends to the end of the line. Both forms are passed through to the generated BASIC output as `'` comments.

```bascal
' This is a single-line comment
// This is also a single-line comment
score% = 0  ' inline comment after a statement
score% = 0  // also valid inline
```

All three comment styles may appear inline after any statement.

### Block Comments

Block comments span multiple lines. The opening delimiter is `/*` and the closing delimiter is `*/`. Block comments may appear anywhere a statement is valid.

```bascal
/*
 * Insertion sort — sorts arr%(0..sizeof(arr%)-1) in ascending order.
 * Time complexity: O(n^2) average and worst case.
 * Space complexity: O(1) — sorts in place.
 */
' arr% -- array to sort; byref because it's mutated in place
function insertionSort%(byref arr%(?))
    for i% = 1 to sizeof(arr%) - 1
        key% = arr%(i%)
        j%   = i% - 1
        while j% >= 0 and arr%(j%) > key%
            arr%(j% + 1) = arr%(j%)
            j% = j% - 1
        end while
        arr%(j% + 1) = key%
    end for
    return 0
end function
```

Each line of a block comment is emitted as a separate `'` comment in the generated BASIC output. Leading `*` characters and surrounding whitespace are stripped. Blank lines within the comment are preserved as blank lines in the output.

One-line block comments are also valid:

```bascal
/* Clear screen and draw title banner */
CLS
LOCATE 1, 30
PRINT "  BASCAL DEMO  "
```

</div>

[← Operators and Expressions](operators-and-expressions.md) [Control Flow →](control-flow.md)
