[Home](../) / [Tutorials](./) / Shared COMMON

<div class="prose" markdown="1">

Classic multi-program BASIC systems pass shared variables across a `CHAIN` with `COMMON` — every chained program has to declare an **identical** `COMMON` list, or values land in the wrong slots. A BASCAL shared file contains only `dim` declarations — every variable in it is COMMON by default, no separate keyword needed — and any program that opens with `program name shared sharedname` gets those declarations emitted verbatim at the top of its generated output, so two programs can't drift out of sync by hand.

</div>

<div class="snippet" markdown="1">

### The shared file

`shared <name>` declares this file as the shared-variables file itself (mirroring a regular file's `program <name>`), and each `dim` below becomes one shared variable -- no statements, no functions.

    shared state

    dim count%
    dim label$

</div>

<div class="snippet" markdown="1">

### A program that references the shared file

    program start shared state

    count% = count% + 1
    count% = count% + 1
    count% = count% + 1

    ' In a real multi-program application: CHAIN "show" -- the compiled
    ' program name, not the .bas source (CHAIN "show.bas" tries to run the
    ' source file itself; verified against real BASCOM 2.00 under dosbox-x)

</div>



[← Require and Multi-File Projects](12_require.md)  ·  [Procedures →](14_procedures.md)


<!-- BEGIN generated tutorial source -->

### `tutorial/13_shared/state.bcl`

```bascal

/*
 * Shared file for Tutorial — COMMON / CHAIN.
 *
 * `shared state` declares this file as the shared-variables file itself,
 * named `state` (matching this filename, state.bcl). Every program that
 * begins with "program name shared state" receives an identical COMMON
 * block at the top of its generated BASIC, built from the `declare`
 * declarations below -- so the listed variables are shared across
 * CHAINed programs in a multi-program application. Variables here are
 * COMMON by default; there's no separate keyword to opt in. `declare` is
 * an interchangeable synonym for `dim` -- used here since both variables
 * are scalars, not arrays.
 *
 * This file shares a counter and a label between two programs:
 *   start.bcl  — initialises and increments the counter
 *   show.bcl   — reports the final value
 */
shared state

declare count%
declare label$

```

### `tutorial/13_shared/start.bcl`

```bascal

// Tutorial — Shared COMMON, program 1 of 2
//
// "program name shared sharedname" tells bcc to load sharedname.bcl and
// emit its COMMON declarations at the very top of the generated output.
// Every program referencing the same shared file emits the same COMMON
// block, so the variables survive a CHAIN to the next program.
//
// Compile:
//   bcc tutorial/13_shared/start.bcl
//
// The generated .bas will open with COMMON count%, label$ followed by
// the program body below.

program start shared state

label$ = "Counter demo"
count% = 0

count% = count% + 1
count% = count% + 1
count% = count% + 1

print "Initialised: " + label$
print "Count after 3 increments: " + str$(count%)

/* In a real multi-program application you would chain to the compiled
   show.exe -- CHAIN takes a program name, not the .bas source it was
   compiled from (verified against real BASCOM 2.00 under dosbox-x:
   CHAIN "show.bas" tries to run the source file itself and corrupts): */
/* CHAIN "show" */
end

```

### `tutorial/13_shared/show.bcl`

```bascal

// Tutorial — Shared COMMON, program 2 of 2
//
// This program references the same shared file as start.bcl.  Its
// generated BASIC will begin with the same COMMON block, so count% and
// label$ contain whatever values start.bas left in them when it CHAINed
// here.
//
// Compile:
//   bcc tutorial/13_shared/show.bcl

program show shared state

print "Label:  " + label$
print "Count:  " + str$(count%)

if count% > 0 then
    print "Counter was incremented " + str$(count%) + " time(s)."
else
    print "Counter was never incremented."
end if

end

```

<!-- END generated tutorial source -->
