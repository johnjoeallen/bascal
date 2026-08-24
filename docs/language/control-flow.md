## Blocks replace branch plumbing

A program is easier to change when its source shows where an alternative or repetition begins and ends. BASCAL’s block forms do that while retaining BASIC’s familiar vocabulary.

```bascal
for row% = 1 to 10
    if row% mod 2 = 0 then
        print "even";
    else
        print "odd";
    end if
    print row%
end for
```

`if`, `elseif`, `else`, and `end if` make a selection one unit; `for`/`end for`, `while`/`end while`, and `do`/`loop` make repetition one unit. The target BASIC still uses its branches, but they are generated from the structure rather than maintained by hand — nothing renumbers a hand-written `GOTO` when a line is inserted above it.

## Four ways to repeat, chosen by when the test runs

`for` counts; the rest test a condition, and differ only in *when* that test runs and which way it points.

```bascal
while attempts% < maxAttempts%       ' test before, "while true"
    attempts% = attempts% + 1
end while

do until attempts% >= maxAttempts%   ' test before, "until true"
    attempts% = attempts% + 1
end do

do                                    ' test after, so the body
    attempts% = attempts% + 1        ' always runs at least once
loop until attempts% >= maxAttempts%
```

`while` and `do until` both test before the body runs, so the body may execute zero times; they differ only in which way the condition points (keep going *while* true, versus keep going *until* true). `do ... loop until` tests after, so the body is guaranteed to run once even if the condition was already satisfied going in — the natural shape for “read a line, then decide whether to read another.” `exit` leaves the innermost loop early from anywhere in its body, replacing a hand-aimed `GOTO` past the loop’s end.

## Choose among many cases

```bascal
select case score%
case 90 to 100
    grade$ = "A"
case is >= 80
    grade$ = "B"
case else
    grade$ = "C"
end select
```

`select case` is particularly useful when a ladder of `elseif`s is really one decision about a single value — each `case` reads as one row of that decision instead of a repeated condition.

## Short-circuit evaluation: `&&` and `||`

This is the one piece of BASCAL’s control flow with no classic-BASIC equivalent at all, so it is worth slowing down for. Classic BASIC’s `AND`/`OR` are *bitwise* operators: both sides are full expressions, both get evaluated, and only then are the two results combined. That is fine when both sides are cheap and safe to evaluate unconditionally — it stops being fine the moment evaluating the second side is only safe, or only makes sense, when the first side already came out a certain way.

```bascal
if ptr% >= 0 && scores%(ptr%) > 0 then
    print "safe to read"
end if
```

`&&` stops as soon as the answer is known: if `ptr% >= 0` is false, `scores%(ptr%)` is never touched at all — no out-of-range read, no crash. `||` is the mirror image: it stops at the first operand that comes out true, so later operands (a slower check, a fallback lookup) only run when every earlier one has already failed. Written with classic bitwise `AND`, the same guard would need `ptr% >= 0` checked in its own `if` *before* the array read could safely appear at all, since bitwise `AND` would evaluate `scores%(ptr%)` unconditionally, out-of-range index or not.

<div class="aside" markdown="1">

Two real restrictions worth remembering: `&&`/`||` are usable only directly in the condition of `if`/`elseif`/`while`/`do` — never assigned to a variable or passed as a function argument — and a single condition may chain any number of the *same* operator, but mixing `&&` and `||` in one condition is a transpile-time error. Split into nested `if` statements instead, the way the generated BASIC itself does under the hood.

</div>

A retry loop shows the same idea on the repeating side: instead of a bare `do` with a separate `exit` for every way it might stop, both stopping conditions can live in one `do until` clause, short-circuiting so only the checks that still matter actually run each time through.

```bascal
do until succeeded% <> 0 || attempts% >= maxAttempts%
    attempts% = attempts% + 1
    ' ...
end do
```

[← Values, names, and expressions](values-and-names.md)[Arrays and strings →](arrays-and-strings.md)
