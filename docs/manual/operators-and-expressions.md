[Home](../) / [Manual](../manual/) / Operators and Expressions

[← Variables and Constants](variables-and-constants.md) [Comments →](comments.md)

<div class="prose" markdown="1">

### Arithmetic Operators

| Operator | Operation                                                                                                  |
|----------|------------------------------------------------------------------------------------------------------------|
| `+`      | Addition / string concatenation                                                                            |
| `-`      | Subtraction / unary negation                                                                               |
| `*`      | Multiplication                                                                                             |
| `/`      | Division (always floating-point, even between two integers)                                                |
| `\`      | Integer division (each operand is rounded to an integer first, then the quotient is truncated toward zero) |
| `MOD`    | Modulus (remainder after integer division)                                                                 |
| `^`      | Exponentiation (right-associative)                                                                         |

    a% = 17
    b% = 5
    print a%; "+ "; b%; "="; a% + b%    // 22
    print a%; "\ "; b%; "="; a% \ b%    // 3  (integer quotient)
    print a%; "MOD "; b%; "="; a% mod b% // 2  (remainder)
    print "2 ^ 8 ="; 2 ^ 8              // 256
    print "2 ^ 3 ^ 2 ="; 2 ^ 3 ^ 2     // 512  (right-assoc: 2 ^ (3^2))

### Comparison Operators

| Operator | Meaning               |
|----------|-----------------------|
| `=`      | Equal                 |
| `<>`     | Not equal             |
| `<`      | Less than             |
| `<=`     | Less than or equal    |
| `>`      | Greater than          |
| `>=`     | Greater than or equal |

Comparison expressions evaluate to -1 (true) or 0 (false) at the BASIC runtime, consistent with Microsoft BASIC semantics.

### TRUE and FALSE

`TRUE` and `FALSE` are transpile-time sugar for BASIC's own boolean convention — `-1` and `0` — so a programmer-boolean flag can be compared against a name instead of a magic number:

    found% = TRUE
    done%  = FALSE

    if found% = TRUE then
        print "found it"
    end if

They transpile straight through to the literals themselves — `found% = TRUE` generates `found% = -1` — so they're valid anywhere an integer literal is, including `CONST` and array bounds. No boolean type is introduced anywhere else in the language; see the `NOT` caveat above for why explicit `= 0` / `<> 0` comparisons are still how you test a flag.

### Compound Assignment

    x% += n%    ' x% = x% + n%
    x% -= n%    ' x% = x% - n%
    x% *= n%    ' x% = x% * n%
    x% /= n%    ' x% = x% / n%

Shorthand for reassigning a variable in terms of itself — the common case in loop counters and accumulators. `total% += x%` is exactly equivalent to `total% = total% + x%`; it works on array elements and record fields too:

    scores%(i%) += 1
    s.total# -= fee#

Only `+=`, `-=`, `*=`, `/=` are provided — there is no compound form of `\`, `MOD`, `^`, or the bitwise/logical operators.

### Logical Operators

| Operator | Meaning                                                         |
|----------|-----------------------------------------------------------------|
| `AND`    | Bitwise AND (also serves as logical AND when operands are 0/-1) |
| `OR`     | Bitwise OR                                                      |
| `NOT`    | Bitwise NOT                                                     |
| `XOR`    | Bitwise XOR                                                     |

**Important:** `NOT` is bitwise in Microsoft BASIC. `NOT 1` yields `-2`, not `0`. BASCAL's transpiler emits `(expr) = 0` instead of `NOT expr` in generated control-flow conditions so that programmer-boolean values like `found% = 1` behave as expected. Use explicit `= 0` or `<> 0` comparisons in your own code when testing boolean flags.

`AND`/`OR` always evaluate both sides — there's no short-circuit primitive in generated BASIC at all. See [Short-Circuit `&&` and `||`](control-flow.md#short-circuit-and) for BASCAL's condition-only short-circuit operators.

    age%    = 25
    income% = 45000
    if age% >= 18 and income% >= 30000 then
        print "Eligible"
    end if
    print 6 xor 3   // 5  (110 XOR 011 = 101)

### Operator Precedence (highest first)

| Level | Operators                       |
|-------|---------------------------------|
| 9     | `^` (right-associative)         |
| 8     | Unary `-`                       |
| 7     | `*`, `/`                        |
| 6     | `\`                             |
| 5     | `MOD`                           |
| 4     | `+`, `-`                        |
| 3     | `=`, `<>`, `<`, `<=`, `>`, `>=` |
| 2     | `NOT`                           |
| 1     | `AND`                           |
| 0     | `OR`                            |
| -1    | `XOR`                           |

Use parentheses to override precedence.

</div>

[← Variables and Constants](variables-and-constants.md) [Comments →](comments.md)
