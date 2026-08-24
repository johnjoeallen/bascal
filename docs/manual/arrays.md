[Home](../) / [Manual](../manual/) / Arrays

[← Procedures](procedures.md) [Input and Output →](input-and-output.md)

<div class="prose" markdown="1">

### Declaration

```bascal
DIM values%(100)    ' 101 elements: values%(0) .. values%(100)
DIM names$(50)
```

Array indices run from 0 to *size* (i.e., *size*+1 elements in total). `OPTION BASE` is rejected outright — a transpile-time error under both targets — see [OPTION BASE](variables-and-constants.md#option-base) in Variables and Constants. Every array is indexed from base 0.

### Access

```bascal
values%(0) = 42
PRINT values%(i%)
```

### Passing Arrays to Functions

An array parameter must declare its rank right in the signature — one `?` per dimension, in parens after the name: `arr%(?)` for 1-D, `grid%(?, ?)` for 2-D, and so on. A scalar parameter stays a bare name. There's no way to declare an array parameter without stating its rank this way — a parameter that's indexed as an array in the function body but declared without one is a **transpile-time error**, not a warning.

At the call site, just write the plain array name — **no `()` needed**. The transpiler already knows that parameter is an array from its declaration, so there's nothing left for the call site to mark.

And separately: `byref` does **not** give the function a real reference to the caller's array — BASIC has no pointers or aliasing at this level. `byref` copies the array's elements in before the call and copies them back out after; `byval` (the default) only does the copy-in half. Either way the function always works on its own private copy — `byref` just *simulates* "the caller sees the result" by copying twice instead of once. See [byref / byval](#byref-byval) for the full mechanism.

`insertionSort%` mutates the array in place, so its `arr%` parameter needs `byref`; `indexOf%` only reads it, so the unmarked (`byval`) default is correct as-is:

```bascal
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

' arr%    -- array to search; byval, since indexOf% only reads it
' target% -- value to search for
function indexOf%(arr%(?), target%)
    for i% = 0 to sizeof(arr%) - 1
        if arr%(i%) = target% then
            return i%
        end if
    end for
    return -1
end function
```

From `tutorial/08_arrays.bcl`:

```bascal
CONST N% = 6
DIM data%(N%)
data%(0) = 64 : data%(1) = 25 : data%(2) = 12
data%(3) = 22 : data%(4) =  3 : data%(5) = 11

dummy% = insertionSort%(data%)   ' sorts in place -- arr% is byref

idx% = indexOf%(data%, 22)
if idx% >= 0 then
    PRINT "22 found at index " + STR$(idx%)
end if
```

See [byref / byval](#byref-byval) for exactly what gets copied, and when.

### `byref` / `byval`

Every parameter — scalar or array — is copied into its generated storage before the call. Whether that value is copied back to the caller afterward depends on how the parameter is declared:

```bascal
function insertionSort%(byref arr%(?))   ' byref: copied in, then back out
function indexOf%(arr%(?), target%)      ' unmarked = byval: copied in only
```

- **`byval`** (the default — an unmarked parameter is `byval`): the function gets its own private copy. Nothing is written back when the call returns, no matter what the function does to its copy internally.
- **`byref`**: copied in before the call, same as `byval` — but also copied back out to the caller after the call returns.

This applies uniformly to both parameter kinds:

- **Array parameters**: `byval` copies elements in; `byref` copies them in *and* back out. A function that only reads its array argument (like `indexOf%` above) should stay `byval` — a `byref` array with no writes is just a slower `byval`, since the transpiler still generates the copy-out loop.

- **Scalar parameters**: `byval` is the classic behavior scalar parameters have always had — a plain assignment in, nothing written back. `byref` turns a scalar parameter into a true output parameter:

```bascal
  ' n% -- value to increment; byref so the caller sees the result
  procedure increment(byref n%)
      n% = n% + 1
  end procedure

  x% = 5
  increment(x%)   ' x% is now 6
```

  A `byref` argument must be a plain variable — `increment(x% + 1)` is a transpile-time error, because there's nowhere for the result to be written back to.

If you're coming from classic MBASIC/BASCOM: there's no local scope there at all, so a `GOSUB`-based "subroutine" touching an array was always touching the *one* array that exists — mutations were visible everywhere, instantly, because there was never more than one copy. BASCAL's parameters don't work that way by default. `byval` (the default) gives the function its own copy, and `byref` is what asks for the old always-visible, always-shared behavior back, deliberately, per parameter.

### Why Copy-In/Copy-Out, Not Just Globals?

MBASIC/BASCOM has exactly one subroutine primitive: `GOSUB`/`RETURN`. There is no `SUB`, no `FUNCTION`, no parameter list of any kind — the most disciplined code the raw dialect allows is a `GOSUB` target with a contract enforced only by comments (*"expects `A$` and `B%` set, leaves the result in `C%`"*). That convention-only discipline, and the maintenance burden of getting it wrong across a team and a growing codebase, is exactly what BASCAL exists to replace.

So the choice was never "copy-in/copy-out versus some simpler globals-based way to pass parameters" — MBASIC/BASCOM has no other mechanism to build parameters out of at all. `GOSUB` and global variables are the only two primitives available. Giving `.bcl` functions real parameters means simulating them using only those two primitives, and copy-in/copy-out is what that simulation necessarily looks like: assign the argument into the parameter's own storage before the `GOSUB`, and — for `byref` — copy the result back out after.

There's also a structural reason it has to work this way, independent of the target dialect. Each function body is transpiled exactly once, and every call site `GOSUB`s to that same shared label. A shared body needs one stable name for "its first parameter" — but different call sites pass different things: different variable names, or a whole expression (`f(a% + b%)`, `f(5)`) that isn't a variable at all. There's no single caller-side location to just operate on directly in the general case, so the value has to land somewhere fixed before the shared body runs.

`global` is the escape hatch for when you deliberately want the old always-shared, no-copy behavior back for one specific variable — see [Variable Scoping](functions.md#variable-scoping). It works precisely because it commits to one hardcoded name forever, which is exactly what makes it not a reusable, callable-with-different-data routine anymore.

### Multi-Dimensional Array Parameters

A 2-D (or higher) array parameter declares its rank the same way as 1-D — one `?` per dimension — and passes the same way, too: just the plain array name, no `()` and no count arguments of any kind. The transpiler already knows the real array's bounds (from its `DIM`, or — if it's itself a parameter being forwarded onward — from what *its* caller passed), and carries them alongside the array automatically. Nothing about that is visible in `.bcl` source; use [`sizeof()`](#sizeof) inside the function body wherever the bound is needed.

```bascal
' grid% -- 2-D array to sum
function sumGrid%(byref grid%(?, ?))
    total% = 0
    for r% = 0 to sizeof(grid%, 0) - 1
        for c% = 0 to sizeof(grid%, 1) - 1
            total% = total% + grid%(r%, c%)
        end for
    end for
    return total%
end function

dim g%(2, 2)
print sumGrid%(g%)
```

There's no way to pass the wrong bounds by hand here — unlike a manually typed count argument, which could silently drift out of sync with the array's real `DIM` (say, `3, 3` for an array actually `dim`ed `(2, 2)`, reading one row and one column past the end of the real array at runtime), the transpiler reads `grid%`'s bounds directly from `g%`'s own `DIM` and there's no hand-typed number in the picture to get wrong.

`grid%(?, ?)` is cross-checked two ways, both at transpile time:

- Against the function's own body — `grid%(r%, c%)` above indexes with two subscripts, matching the declared two `?`s. A declaration that disagrees with how the body actually uses the parameter is an error.
- Against whatever array is actually passed at each call site — passing a 1-D array where the parameter declares two dimensions (or vice versa) is also an error.

Either mismatch is caught before it ever reaches generated BASIC: the two shapes genuinely can't share one copy loop, so BASCAL refuses rather than emit a `DIM`/subscript mismatch that real BASIC would only catch at runtime.

### Array Parameter Storage Capacity

An array parameter's storage is one shared, generated variable, reused by every call to that function — the same reason a scalar parameter's storage is shared. Arrays additionally need a fixed *size*, though, and classic BASIC has no `REDIM`: once an array is `DIM`ed, it can never be resized, and a second `DIM` on the same array is a fatal runtime error. So a parameter's storage is `DIM`ed exactly once, at the very top of the generated program — before any call happens — sized to the biggest array anything anywhere ever passes it.

Normally this needs no attention at all. Write `?` for every axis, same as always; the transpiler works out a safe capacity itself by scanning every call site in the program and taking the largest resolved size. Below, `sumArr%`'s storage ends up sized for 10 elements even though its first call only ever passes it 3:

```bascal
function sumArr%(arr%(?))
    ' ...
end function

dim small%(2)
dim big%(9)
dummy% = sumArr%(small%)
dummy% = sumArr%(big%)
```

This works whenever every call site's array size is knowable at transpile time — a literal `DIM` bound, a `const`, or (when the array being passed is itself another function's array parameter, forwarded onward) that parameter's own already-resolved capacity. It genuinely can't work when a call site's array size is a real runtime value:

```bascal
input n%
dim data%(n%)
dummy% = sumArr%(data%)   ' error: arr%'s capacity can't be inferred
```

There's no way to know at transpile time how big `data%` will turn out to be, so there's no safe number to give its shared storage automatically. Write an explicit capacity instead of `?` for that axis — a literal integer, chosen to comfortably cover every use:

```bascal
function sumArr%(arr%(100))
    ' ...
end function
```

Whichever way a capacity is decided — inferred or explicit — every call site still checks the array's *actual* size against it at runtime, right before copying in, and halts with a clear error if it doesn't fit:

```bascal
IF sumarrArrDim00% > 100 THEN PRINT "runtime error: ..." : STOP
```

This is a backstop, not the primary defense — a call site whose size *is* a transpile-time constant and provably too big for its capacity is already rejected at transpile time, before generated BASIC exists to run at all. The runtime check exists for the one case that's genuinely unprovable ahead of time: a capacity chosen to comfortably cover today's inputs that a later, larger runtime value turns out to exceed.

### `sizeof()`

`sizeof(name)` returns a `dim`ed array's real element count along an axis — one more than the bound written in its own `DIM`, matching real BASIC's inclusive-bound convention (`dim arr%(N)` holds `N + 1` elements, indices `0..=N`) — resolved entirely at transpile time; it never appears in generated BASIC, only whatever value or name it resolves to. For a 1-D array the axis is implicit:

```bascal
dim data%(9)
print sizeof(data%)   ' 10 -- one more than the bound used in the dim
```

For 2-D or higher, the axis is required — `sizeof(name, axis)`, zero-based in the same order as the array's own `DIM`:

```bascal
dim grid%(2, 2)
print sizeof(grid%, 0)   ' 3 -- first DIM bound's element count
print sizeof(grid%, 1)   ' 3 -- second DIM bound's element count
```

The axis must be a literal integer — it selects which frozen value to substitute at transpile time, not something computed at runtime.

**What "resolved at transpile time" means in practice:** if the bound is a literal, `sizeof` just re-emits that literal. If it's an expression (a variable, a `const`, anything not a bare number), the transpiler captures its value into a hidden variable right at the `dim` site, and `sizeof` always reads that captured value — never the live variable, which might change afterward:

```bascal
n% = 5
dim data%(n%)
n% = 99
print sizeof(data%)   ' 6 -- one more than the value dim actually used, not the later 99
```

**Inside a function, `sizeof` on one of the function's own array parameters works differently** — there's no local `dim` to freeze a value from, since the array parameter's real size depends on whatever the caller happens to pass. There's no manually declared count parameter to read either: every array parameter's bounds are carried automatically, one hidden transpiler-generated variable per axis, set by the caller (from the real argument array's own resolved bounds) immediately before the call. `sizeof(grid%, 0)` inside `sumGrid%`'s own body just reads that hidden variable back:

```bascal
function sumGrid%(byref grid%(?, ?))
    total% = 0
    for r% = 0 to sizeof(grid%, 0) - 1     ' reads the auto-passed row count
        for c% = 0 to sizeof(grid%, 1) - 1 ' reads the auto-passed column count
            total% = total% + grid%(r%, c%)
        end for
    end for
    return total%
end function
```

Nothing here is written by the `.bcl` author, at the call site or in the signature — the bound simply isn't a value you pass, it's a value you ask for with `sizeof()` wherever you need it.

`sizeof()` and [storage capacity](#array-parameter-storage-capacity) are two different numbers that happen to often be equal. `sizeof(arr%)` is always the *actual* array this particular call passed — it can be smaller than capacity (e.g. `sumArr%` above sees `sizeof(arr%) = 3` on the call that passes `small%`, even though its storage was sized for 10 to also fit `big%`). Capacity is the fixed ceiling that storage was built for, decided once, up front, for every call the program will ever make.

</div>

[← Procedures](procedures.md) [Input and Output →](input-and-output.md)
