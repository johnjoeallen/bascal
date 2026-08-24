## Start with the whole

BASCAL files use the `.bcl` extension. The `bcc` compiler turns one into classic BASIC or C. Start by describing the work in the order a reader expects to see it.

```bascal
program hello

declare name$

print "What is your name";
input name$

if name$ = "" then
    print "Hello, stranger."
else
    print "Hello, " + name$ + "."
end if
end
```

Save this as `hello.bcl` and run `bcc hello.bcl`. The default backend creates `hello.bas`; `bcc --target c hello.bcl` emits C instead. This example compiles with both backends. The C backend is almost complete, and the manual’s [Backends](../manual/command-line-reference.md#backends) section lists its remaining limits.

## Every file declares what it is

Every `.bcl` file must begin with one declaration: a runnable `program <name>`, a `library <name>`, or a `shared <name>` file containing only variable declarations. `bcc` rejects any file without one instead of guessing. A final bare `end` closes a program. The compiler could infer it from the end of the file, but writing it keeps each program’s shape consistent.

Multi-file projects use `require`, or its alias `import`. Put dependency declarations directly after the file declaration and before any statements.

<div class="aside" markdown="1">

Most later examples are fragments: the useful lines without a `program name` header or final `end`. Add both when you turn a fragment into a file to compile.

</div>

## Read it from top to bottom

`declare name$` declares a string variable. `print` and `input` are familiar BASIC operations. The important difference is the decision: `if` has a visible beginning, alternatives, and an `end if`. There are no numbered exits to find.

<div class="aside" markdown="1">

BASCAL makes control flow easy to read in the source, then emits the needed constructs for the selected target.

</div>

The next chapters explain the declarations, expressions, and blocks used here. Later chapters give the detailed language and command-line rules.

[← Preface](preface.md)[Values, names, and expressions →](values-and-names.md)
