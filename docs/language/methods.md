## Methods

A method is a statically resolved callable with an implicit receiver. Methods are not runtime objects, classes, interfaces, or dynamic dispatch — a method call is always resolved to one specific callable at compile time, from the receiver's own exact declared type.

The receiver may be declared explicitly, in brackets after the method name, or supplied implicitly by an enclosing `record` declaration. Either way, the return type follows the argument list after `:`.

```text
method name[receiver type](arguments): return type
```

or, inside a record:

```text
method name(arguments): return type
```

Square brackets identify the receiver only — never the return type. The two declaration forms below are the same callable identity, described two ways:

```bascal
record Card
    title: string(40)
    author: string(40)

    method display(): string
        return self.title + " by " + self.author
    end method
end record
```

```bascal
record Card
    title: string(40)
    author: string(40)
end record

method display[Card](): string
    return self.title + " by " + self.author
end method
```

Both are called the same way:

```bascal
let card = { title: "Dune", author: "Frank Herbert" }
print card.display()
```

A call `card.display()` is conceptually just `display(card)`: the receiver is implicit in BASCAL's own call syntax, but explicit — an ordinary first argument — in the callable it resolves to. Inline and external declarations of the same method share one namespace and one set of duplicate-definition rules; declaring the same method both ways is a duplicate definition, exactly like two ordinary functions of the same name, not two separate methods.

### Return types

The return type after `:` is either a full BASCAL type name or its suffix shorthand — the same suffix/type mapping used everywhere else in the language, not a separate one for methods:

| Suffix | Type |
| --- | --- |
| `%` | `integer` |
| `&` | `long` |
| `!` | `single` |
| `#` | `double` |
| `$` | `string` |

`method display[Card](): string` and `method display[Card](): $` mean exactly the same thing. A method name's own suffix (`method display$[Card]()`) is accepted too, and must agree with an explicit `: ReturnType` clause if both are given.

Omitting the return type clause entirely is only meaningful for a scalar receiver, where it defaults to the receiver's own type — the method then falls through and implicitly returns `self`:

```bascal
method shout[string]()
    return self$.ucase() + "!"
end method
```

A record receiver has no single scalar type a whole record value could implicitly become, so a record method with no `: ReturnType` clause is a procedure with no result at all, called only for its effect on `self`.

### Scalar receivers

The receiver type in brackets is one of `integer`, `long`, `single`, `double`, or `string`. The receiver is available in the body as the matching `self` — `self%`, `self!`, `self$`, and so on:

```bascal
method clamp[integer](low%, high%)
    if self% < low% then return low%
    if self% > high% then return high%
    return self%
end method

print 125.clamp(0, 100)
```

### Record receivers

The receiver type in brackets — or, for an inline method, the enclosing `record`'s own name — names a declared record type. `self` inside the method body is that record, and `self.field` is ordinary field access against it, exactly the same field access any other record variable uses:

```bascal
record Card
    title: string(40)
    author: string(40)

    method display(): $
        return self.title + " by " + self.author
    end method
end record
```

A record method's receiver is passed the way a C-level receiver naturally would be — by reference, not by copy — so mutating `self.field` is visible to the caller's own record variable once the call returns:

```bascal
record Card
    title: string(40)
    qty: int

    method bump(amount%)
        self.qty = self.qty + amount%
    end method
end record

let card = { title: "Dune", qty: 1 }
card.bump(4)
print card.qty          ' 5
```

BASCAL records have no inheritance or subtype relationships: a variable has exactly one declared record type, with no upcast, downcast, or covariance between unrelated record types. Two different record types may each declare a same-named method with no ambiguity or collision — method resolution always uses the receiver's own exact declared type, never a dynamic/runtime one:

```bascal
record Animal
    name: string(20)
    legs: int

    method speak(): $
        return self.name + " makes a sound"
    end method
end record

record Dog
    name: string(20)

    method speak(): $
        return self.name + " barks"
    end method
end record
```

`Animal.speak` and `Dog.speak` are two entirely separate callables. There is no virtual override, no vtable, and no runtime dispatch mechanism of any kind — a method call is resolved once, at compile time, from whichever exact record type its receiver expression was declared with.

### Structural composition with `mixin`

A record mixin contributes the fields of one or more existing record types to a new record type. Mixins provide structural composition only: they do not imply inheritance, subtype compatibility, polymorphism, or method inheritance. All effective field names must be unique; duplicate field names are compile-time errors.

`record Dog mixin Animal` makes `Dog` composed of `Animal`'s fields, accessed directly:

```bascal
record Animal
    species: string(20)
end record

record Dog mixin Animal
    breed: string(20)
end record

let d = { species: "Canis", breed: "Labrador" }
print d.species              ' "Canis" -- Animal's own field
print d.breed                ' "Labrador" -- Dog's own field
```

`Dog`'s *effective* field list is `Animal`'s fields followed by `Dog`'s own (`species`, `breed`), so a `Dog` record literal accepts all of them, and `d.species` is ordinary field access exactly like any other field. Multiple sources can be mixed in, comma-separated, and mixins can be transitive (a mixed-in record can itself mixin others) — every source's effective fields are computed first, then combined, before any duplicate checking:

```bascal
record Pet
    called: string(20)
end record

record Dog mixin Animal, Pet
    breed: string(20)
end record
```

`Dog` now has `species` (from `Animal`), `called` (from `Pet`), and `breed` (its own).

**Mixins never contribute methods.** A method declared for an `Animal` receiver applies only to an `Animal` receiver, even when `Dog` is composed of every one of `Animal`'s fields:

```bascal
record Animal
    species: string(20)

    method describe(): $
        return "a " + self.species
    end method
end record

record Dog mixin Animal
    breed: string(20)
end record

let d = { species: "Canis", breed: "Labrador" }
print d.describe()           ' compile-time error -- no describe() for Dog
```

If `Dog` needs that behavior, it declares its own method — inline or external, same as any other record method:

```bascal
method describe[Dog](): $
    return self.breed + " (" + self.species + ")"
end method
```

`mixin` does **not** make `Dog` assignable to or from `Animal`. Each remains its own exact record type:

```bascal
dim a as Animal
dim d as Dog

a = d   ' rejected -- Dog is not assignable to Animal
d = a   ' rejected -- Animal is not assignable to Dog
```

There is no upcast, no downcast, no covariance, and no runtime containment of any kind. A record mixing in an undeclared type, or mixing in itself (directly or through a longer cycle), is a compile-time error. A duplicate field name — whether contributed by two different mixin sources, reached through two different transitive mixin paths, or colliding with a field the record declares directly — is always a compile-time error too; BASCAL introduces no aliasing, qualification, or "last one wins" rule to paper over it:

```text
Duplicate field 'name' in record 'Dog':
field contributed by both 'Animal' and 'Pet'
```

### Calling and chaining

Method calls always use parentheses. Scalar method chaining works the same as before — the receiver is written before the dot, and the method's result becomes the receiver for the next call when its type matches:

```bascal
name$ = "  ada  "
result$ = name$.ltrim().ucase().left(3)
```

The ordinary-call spelling remains available for scalar methods: `ucase$(name$)` and `name$.ucase()` resolve to the same declaration when no ordinary function claims that name. A method and an ordinary function cannot share one callable identity.

Built-in scalar methods such as `left`, `len`, `abs`, and `sin` are syntax for the corresponding BASIC intrinsic call, unaffected by any of the above — the new declaration grammar changes how a *method* is declared, not what a built-in method call means.

### Backend shape

Every method — scalar or record, inline or external — is fully resolved before any backend runs: by the time BASIC, C, or JVM codegen sees the program, a method is indistinguishable from a hand-written ordinary function. A record method's `self` becomes one ordinary parameter per record field (`byref`, so mutations propagate back to the caller); a call site becomes an ordinary call, the receiver's own fields passed as its leading arguments.

The JVM backend in particular never needs `invokevirtual`, a Java interface, a vtable, or any other object-oriented dispatch mechanism to implement this: `card.display()` lowers to loading `card`'s own fields and an ordinary `invokestatic`, the same as any other function call. Conceptually:

```text
method display[Card](): $
```

lowers to something equivalent to:

```text
Card_display(Card receiver) -> String
```

and the C backend's own shape is the direct analogue:

```c
char *Card_display(Card *self);
```

None of this creates a runtime method object, and none of it requires the three backends to represent BASCAL methods differently from one another — the source-language semantics are identical everywhere; only the generated call shape differs per backend, the same way it already does for ordinary functions.

### What record methods do not do

Record methods do not give BASCAL runtime polymorphism, inheritance-based dispatch, or Java-style object semantics. A record is a fixed-layout value type, not a class; a record variable's type is fixed at declaration and never changes at runtime; and there is no way for two record types to share a method identity the way a subclass overrides a parent's method in an object-oriented language. `mixin` (see above) reuses field declarations only — it does not mix in methods, and it does not imply assignability or method-dispatch substitutability between the mixing record and its sources — each retains its own exact type and its own exact method resolution.
