## Name an operation

A function calculates a typed result; a procedure performs an action. Keeping the distinction explicit makes call sites honest about whether a value comes back. A name is yours to choose freely, with one exception: it can’t be the same as a real BASIC builtin — `function sqr%(x%)` is rejected when the program is compiled, since neither silently shadowing `SQR` nor being shadowed by it was ever what the program meant.

```bascal
function larger%(left%, right%)
    if left% > right% then
        return left%
    end if
    return right%
end function

procedure showTotal(amount!)
    print "Total: "; amount!
end procedure

showTotal(larger%(4, 9))
```

Parameters are passed by value unless you say `byref`. That default makes a call read as an input; use `byref` only when the caller’s variable is intentionally an output too.

## Methods

Methods are covered in the dedicated [Methods](methods.md) chapter. It defines the current scalar method syntax, chaining and library/built-in resolution, backend behavior, and the planned record-method model.
