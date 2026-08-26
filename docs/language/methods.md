## Methods

A method is a named operation with an implicit first argument: its receiver. The receiver can be a scalar value today, and the same model is designed to extend to record values. Methods are statically resolved; they are not runtime objects, classes, or dynamic dispatch.

### Scalar methods

The current syntax puts the receiver type in brackets after the method name:

```bascal
method shout[string]()
    return self$.ucase() + "!"
end method

print "hello".shout()
```

The first bracketed type is the receiver. If the result type is omitted, it is the same scalar type as the receiver. A method can therefore fall through and return `self` automatically:

```bascal
method identity[integer]()
end method

score% = 42.identity()
```

An explicit scalar result may follow the receiver, and a scalar suffix on the method name remains accepted as shorthand:

```bascal
method percent[single, single](rate!)
    return self! * rate! / 100
end method

method clamp%[integer, integer](low%, high%)
    if self% < low% then return low%
    if self% > high% then return high%
    return self%
end method
```

The suffix and explicit result, when both are present, must agree. The supported bracketed scalar names are `integer`, `long`, `single`, `double`, and `string`.

### Calling and chaining

Method calls always use parentheses. The receiver is written before the dot, and the method's result becomes the receiver for the next call when its type matches:

```bascal
name$ = "  ada  "
result$ = name$.ltrim().ucase().left(3)
```

The ordinary-call spelling remains available for compatibility. `ucase$(name$)` and `name$.ucase()` resolve to the same method declaration when no ordinary function claims that name. A method and an ordinary function cannot share one callable identity.

Built-in scalar methods such as `left`, `len`, `abs`, and `sin` are syntax for the corresponding BASIC intrinsic call. User methods from a required library use the same receiver, result, chaining, and type checks as local methods.

### Record methods (planned)

Methods are not limited to scalar receivers in the language model. A future record method will use a record type as its receiver and access the record through `self`, just as a scalar method does today. For example, the planned shape is:

```bascal
record Card
    title: string(40) lpad
    author: string(40) lpad
end record

' Planned syntax — not accepted by the current parser yet.
method display[Card, string]()
    return self.title + " by " + self.author
end method

' Planned call:
print card.display()
```

Record methods are intended to work with ordinary in-memory record values, record parameters, record results, and record arrays once those general-purpose record values exist. They must preserve the record type through method chains, check field access and result types at transpile time, and define explicit `byval`/`byref` copy semantics for record receivers. A record-backed random-access file value is not itself a record method receiver; file operations remain the separate `file`/record DSL.

This section describes the design direction, not an implemented feature. Record receiver and result types are follow-up work to the scalar method syntax; see [issue #128](https://github.com/johnjoeallen/bascal/issues/128) for the staged type-annotation plan.

### Backend shape

Methods transpile to ordinary typed calls. The BASIC target uses generated receiver, parameter, and result storage plus `GOSUB`; the C target emits typed functions and temporaries; the JVM target emits native methods for its currently supported scalar and integer-array subset. None of these representations creates a runtime method object.
