This is the complete reference for BASCAL. Read it from the beginning to build the language up through complete programs, then return to its chapters whenever you need a precise rule or example.

<div class="section part" markdown="1">

## Before Part I

What this language and this book are for.

1.  [<span class="chapter-number">00</span><span><span class="chapter-title">Preface</span><span class="chapter-summary">Why BASCAL exists, who this book is for, and how to use the complete reference.</span></span>](preface.md)

</div>

<div class="section part" markdown="1">

## Part I · The language

The core ideas needed to read and write a BASCAL program.

1.  [<span class="chapter-number">01</span><span><span class="chapter-title">A first program</span><span class="chapter-summary">Compile, run, and read a complete BASCAL program before learning its individual pieces.</span></span>](first-program.md)
2.  [<span class="chapter-number">02</span><span><span class="chapter-title">Values, names, and expressions</span><span class="chapter-summary">BASIC type suffixes, declarations, constants, arrays, and the expressions that connect them.</span></span>](values-and-names.md)
3.  [<span class="chapter-number">03</span><span><span class="chapter-title">Making control flow visible</span><span class="chapter-summary">Use blocks, loops, and selection instead of wiring a program together with line numbers.</span></span>](control-flow.md)
4.  [<span class="chapter-number">04</span><span><span class="chapter-title">Arrays and strings</span><span class="chapter-summary">Declare fixed collections, pass them deliberately with `byref`/`byval`, and work with text through BASIC's string functions.</span></span>](arrays-and-strings.md)

</div>

<div class="section part" markdown="1">

## Part II · Building programs

Ways to grow beyond a single page without losing the simplicity of BASIC.

1.  [<span class="chapter-number">05</span><span><span class="chapter-title">Functions, procedures, and methods</span><span class="chapter-summary">Give operations names, pass data deliberately, and declare typed scalar methods you can chain.</span></span>](functions-and-procedures.md)
2.  [<span class="chapter-number">05A</span><span><span class="chapter-title">Libraries</span><span class="chapter-summary">Reuse code across files, and across functions, procedures, and methods, through `require`.</span></span>](libraries.md)
3.  [<span class="chapter-number">06</span><span><span class="chapter-title">Data that outlives a run</span><span class="chapter-summary">Work with sequential input and output, then let typed records remove random-file bookkeeping.</span></span>](data-and-files.md)
4.  [<span class="chapter-number">07</span><span><span class="chapter-title">Errors and labels</span><span class="chapter-summary">Handle runtime errors portably with `try`/`catch`, use BASIC-only recovery when needed, and reach for a label when structure genuinely can't reach.</span></span>](errors-labels-and-data.md)
5.  [<span class="chapter-number">07A</span><span><span class="chapter-title">Embedded data</span><span class="chapter-summary">Ship small constant tables with `data`/`read` and rewind them with `restore`.</span></span>](embedded-data.md)
6.  [<span class="chapter-number">08</span><span><span class="chapter-title">The standard library</span><span class="chapter-summary">The built-in math, string, and conversion functions every BASCAL program can already call.</span></span>](standard-library.md)
7.  [<span class="chapter-number">09</span><span><span class="chapter-title">Running and combining programs</span><span class="chapter-summary">`bcc`'s command line, and linking separately-run programs together with `shared`/`COMMON`/`CHAIN`.</span></span>](running-and-combining-programs.md)
8.  [<span class="chapter-number">10</span><span><span class="chapter-title">The targets beneath the program</span><span class="chapter-summary">Understand the BASIC and C targets without programming at either level every day.</span></span>](the-basic-beneath.md)

</div>
