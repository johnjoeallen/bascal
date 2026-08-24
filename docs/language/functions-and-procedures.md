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

## Declare and call a method

A scalar method extends a BASIC scalar type instead of standing alone. A method declaration names its receiver type with the suffix on `method`: `method$` receives a string, `method%` an integer, and `method!` a single-precision number. The method name carries its result suffix, and the receiver is available in the body as the matching implicit `self` value.

```bascal
method$ capitalize$()
    return UCASE$(self$)
end method

name$ = "ada"
result$ = name$.capitalize()
```

Calls always use parentheses. Calls can chain when the previous result has the receiver type required by the next method:

```bascal
result$ = name$.capitalize().pad(20)
```

Methods are statically typed. Before code generation, the parser and resolver check the receiver suffix, method name, argument count, argument suffixes, result suffix, duplicate declarations, reserved built-in names, and unknown methods. A string receiver cannot call an integer method, and a chain is valid only when each result has the receiver type required by the next call. These checks apply equally to user methods, methods from required libraries, and built-in methods. Both targets transpile methods to ordinary typed calls: the BASIC target uses its global parameter/result variables and `GOSUB`, while the C target emits typed calls and result temporaries.

A method's receiver is really just an implicit first parameter, so ordinary-call syntax works too, with no separate declaration needed: `capitalize$(name$)` resolves straight to `method$ capitalize$()` above, with `name$` filling `self$` — as long as no ordinary function of that name already exists. Because of that, a program can’t declare both a function and a method sharing one name; the two would be ambiguous claims on the same callable identity, and it’s a transpile-time error.

## Methods from libraries

A required library’s methods work the same way as ones declared in the program: `require` the library, then call its methods through the same receiver syntax. They participate in the same compile-time receiver and result-type checks.

```bascal
require com.example.text

result$ = name$.capitalize()
```

## Built-in methods

A fixed set of real BASIC intrinsics are callable as methods too, with no declaration needed — `left`, `right`, `mid`, `len`, and `instr` on a string receiver; `abs`, `sqr`, `sin`, `cos`, `tan`, `int`, `fix`, and `sgn` on any numeric receiver. Each is sugar for the identical ordinary call — `s$.left(3)` transpiles to exactly the same output as `left$(s$, 3)` — so they participate in chaining like any other method:

```bascal
name$ = "  Ada  "
print name$.left(5).len()   ' same as len(left$(name$, 5))
```

Their names are reserved: a program can’t declare its own `method$ left$(...)`, the same way it can’t declare a function named after a real BASIC builtin. An invalid receiver or argument is rejected during transpilation rather than silently becoming a different ordinary call.

[← Arrays and strings](arrays-and-strings.md)[Libraries →](libraries.md)
