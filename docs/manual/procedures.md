[Home](../../) / [Manual](../) / Procedures

[← Functions](functions.md) [Arrays →](arrays.md)

<div class="prose" markdown="1">

A procedure is a named subroutine that performs an action but returns no value. It is declared with `procedure` … `end procedure`.

### Declaration

```bascal
procedure name(param1%, param2$)
    ' body
end procedure
```

The procedure name has **no type suffix** — the absence of a suffix signals that there is no return value. Parameter names still carry their usual type suffixes.

Parameters may use the same trailing fixed defaults as functions, so `procedure announce(text$, suffix$ = "!")` may be called as `announce("Ready")`. See [parameter defaults](functions.md#parameter-defaults) for the restrictions.

From `tutorial/procedures.bcl`:

```bascal
procedure printSeparator()
    PRINT "----------------------------"
end procedure

' label$ -- text shown before the score
' score% -- value to print
procedure printScore(label$, score%)
    PRINT label$ + ": " + STR$(score%)
end procedure

' name$  -- person's name
' score% -- score to test against the passing threshold
procedure printIfPass(name$, score%)
    if score% < 60 then
        return          // early exit — nothing printed for failing scores
    end if
    PRINT name$ + " passed with " + STR$(score%)
end procedure

' arr%   -- array to fill; byref because it's mutated in place
' value% -- value written into every element
procedure fillRange(byref arr%(?), value%)
    for i% = 0 to sizeof(arr%) - 1
        arr%(i%) = value%
    end for
end procedure
```

### Calling Procedures

Procedures are called as statements (not inside expressions):

```bascal
printSeparator()
printScore("Alice", 91)
printIfPass("Bob", 54)
fillRange(data%, 99)
```

### Early Exit

A bare `return` (no expression) exits a procedure immediately. Falling through to `end procedure` is equally valid — the transpiler emits an implicit `RETURN`.

```bascal
' name$  -- person's name
' score% -- score to test against the passing threshold
procedure printIfPass(name$, score%)
    if score% < 60 then
        return      ' exit early; nothing is printed
    end if
    PRINT name$ + " passed with " + STR$(score%)
end procedure
```

### Array Parameters

Array parameters use the same [byref / byval](arrays.md#byref-byval) rules and the same `(?, ?, ...)` rank declaration as functions. Pass the plain array name at the call site — no `()`:

```bascal
' arr%   -- array to fill; byref because it's mutated in place
' value% -- value written into every element
procedure fillRange(byref arr%(?), value%)   ' arr%(?) -- 1-D array
    ...
end procedure

fillRange(data%, 99)                         ' plain data%, no ()
```

`fillRange` needs `byref` here because its entire job is to mutate the caller's array — without it, `fillRange` would fill its own private copy and the caller's array would be unchanged.

### Variable Scoping

Same rules as functions: variables in the body are local by default; use `global varname` to access a module-level variable.

```bascal
globalCount% = 0

procedure increment()
    global globalCount%
    globalCount% = globalCount% + 1
end procedure
```

### Restrictions

- **No recursion, direct or indirect.** Same GOSUB transpilation as functions — the transpile-time cycle check covers procedures too, including a cycle that passes through both functions and procedures.
- **No return value.** Do not use a procedure where an expression is expected.

### How Procedures Are Transpiled

Procedures use the same GOSUB mechanism as functions:

1.  Assign each argument to a generated global variable (e.g. `pnameParam0%`)
2.  `GOSUB` to the procedure's generated label
3.  No result variable is read back

Local variables in the body are emitted as uniquely-indexed BASIC globals (`pnameVar0%`, `pnameVar1%`, …) using the same collision-free scheme as functions.

</div>

[← Functions](functions.md) [Arrays →](arrays.md)
