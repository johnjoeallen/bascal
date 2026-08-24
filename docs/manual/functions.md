[Home](../../) / [Manual](../) / Functions

[← Control Flow](control-flow.md) [Procedures →](procedures.md)

<div class="prose" markdown="1">

### Declaration

```bascal
function name%(param1%, param2%)
    ' body
    return expression
end function
```

The function name carries the return type suffix. Parameter names also carry type suffixes. Functions may have zero or more parameters.

### Scalar methods (front-end)

BASCAL also accepts scalar extension-method declarations. The suffix on `method` selects the receiver type, while the suffix on the method name selects its result type. The receiver is available in the body as the matching implicit `self` variable:

```bascal
method$ capitalize$()
    return UCASE$(self$)
end method

method$ pad$(width%)
    return self$
end method
```

Scalar calls require parentheses and may be chained; each result determines the receiver type of the next call:

```bascal
result$ = name$.capitalize().pad(20)
```

The parser and resolver check receiver types, duplicate declarations, reserved built-in names, and unknown methods. Both targets transpile methods to ordinary typed calls: BASIC uses global parameter/result variables and `GOSUB`, while C uses typed parameters and temporaries.

### Parameter Defaults

A trailing scalar `byval` parameter may specify a default with `=`. Callers may then omit it; BASCAL supplies the value at the call site.

```bascal
const defaultGreeting$ = "Hello"

function greet$(name$, greeting$ = defaultGreeting$)
    return greeting$ + ", " + name$
end function

print greet$("Ada")       ' Hello, Ada
print greet$("Ada", "Hi") ' Hi, Ada
```

Defaults must be literals (including a signed numeric literal) or a top-level `const` whose value is a literal. Once a parameter has a default, every following parameter must have one too. Array and `byref` parameters cannot have defaults.

A function or procedure can't be named the same as a real BASIC builtin — `function sqr%(x%)` is a transpile-time error, checked case-insensitively and independent of type suffix (`sqr%` and `sqr$` both collide with `SQR`). Neither shadowing a builtin nor being shadowed by one is ever what a program means; give it a different name instead. Ordinary functions never collide this way — the check is only against the real builtins that pass straight through with no `require` (see [Standard Library Functions](standard-library-functions.md) for the handful BASCAL ships itself, which *are* ordinary functions and don't trigger this check).

From `tutorial/07_functions.bcl`:

```bascal
' a% -- first value to compare
' b% -- second value to compare
function max%(a%, b%)
    if a% > b% then
        return a%
    else
        return b%
    end if
end function

' a% -- first value to compare
' b% -- second value to compare
function min%(a%, b%)
    if a% < b% then
        return a%
    else
        return b%
    end if
end function

' value% -- number to constrain
' lo%    -- lower bound, inclusive
' hi%    -- upper bound, inclusive
function clamp%(value%, lo%, hi%)
    ' Constrain value to [lo, hi].
    return max%(lo%, min%(value%, hi%))
end function

' word$ -- string to title-case
function titleCase$(word$)
    ' Capitalise first letter, lowercase remainder.
    if LEN(word$) = 0 then
        return ""
    end if
    return UCASE$(LEFT$(word$, 1)) + LCASE$(MID$(word$, 2))
end function
```

### Calling Functions

```bascal
PRINT "max(4, 9)      = " + STR$(max%(4, 9))         ' 9
PRINT "clamp(15,1,10) = " + STR$(clamp%(15, 1, 10))  ' 10
PRINT "clamp(-3,1,10) = " + STR$(clamp%(-3, 1, 10))  ' 1
PRINT titleCase$("bASCAL")                            ' Bascal
```

Functions called only for their side effects (discarding the return value) are written as expression statements. The result variable is overwritten but not read:

```bascal
dummy% = sortArray%(data%)
```

### Return

Every function must contain at least one `return` statement. Implicit returns at end-of-body are not supported.

### Calling the Same Function Twice

Each call writes the shared `fnameResult0` variable, so assignments must be made before the next call overwrites it. BASCAL handles this automatically:

```bascal
a$ = repeat$("x", 3)   ' repeatResult0$ = "xxx"  →  a$ = "xxx"
b$ = repeat$("y", 2)   ' repeatResult0$ = "yy"   →  b$ = "yy"
PRINT a$ + " " + b$    ' xxx yy
```

### Variable Scoping

Variables inside a function body are **local by default**: the transpiler maps them to uniquely-generated BASIC names of the form `stemVar0%`, `stemVar1%`, etc. Two functions can each have a variable named `i%` with no conflict, and a local can never accidentally shadow a global that happens to share the naive prefix. Use `global varname` to access a module-level variable:

```bascal
' n% -- upper bound of the sum, inclusive
function sumTo%(n%)
    acc% = 0                ' local to sumTo%
    for i% = 1 to n%       ' local to sumTo%
        acc% = acc% + i%
    end for
    return acc%
end function

runningTotal% = 0

' x% -- amount to add to the running total
function addToTotal%(x%)
    global runningTotal%    ' refers to the module-level variable
    runningTotal% = runningTotal% + x%
    return runningTotal%
end function
```

`global` must name a real module-level variable, not one of the function's own parameters — `function f%(x%) : global x% : ...` is a transpile-time error, since the parameter always resolves first and the `global` declaration could never take effect.

### Restrictions

- **No recursion, direct or indirect.** Functions and procedures are transpiled to `GOSUB` against shared global parameter storage, not a real call stack, so any call cycle — `f%` calling itself, or `f%` calling `g%` calling back into `f%`, however many hops apart — overwrites in-flight parameters. The transpiler checks the whole call graph and rejects **any** cycle at transpile time, not just direct self-calls. Use an explicit stack array to simulate recursion if needed.
- **No return value from a procedure.** Functions must `return` a value; for side-effect-only subroutines use `procedure` instead.

### How Functions Are Transpiled

Each function call transpiles to: 1. Assign each argument to a generated global variable (e.g. `fnameParam0%`) 2. `GOSUB` to the function's generated label 3. Assign the result from the generated result variable (e.g. `fnameResult0%`)

Local variables in the function body are emitted as uniquely-indexed BASIC globals (`fnameVar0%`, `fnameVar1%`, …). The index is chosen so the name does not clash with any global variable or with any name allocated by an earlier function, making collisions impossible regardless of what names the developer uses at global scope.

Every parameter is copied into its generated name before the call. Whether anything is copied back afterward depends on its passing mode — see [byref / byval](arrays.md#byref-byval).

</div>

[← Control Flow](control-flow.md) [Procedures →](procedures.md)
