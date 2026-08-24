## Functions ready to use

BASCAL recognises the standard functions supplied by classic BASIC. They need no declaration or `require`: the BASIC target passes them through, while the C target provides the documented equivalent where it supports the function.

### Text and conversion

`len`, `asc`, `chr$`, `left$`, `right$`, `mid$`, `instr`, `str$`, `val`, `string$`, `space$`, `hex$`, `oct$`, `format$`, and `trim$` work with text and values. `cint`, `clng`, `csng`, and `cdbl` explicitly convert numeric values.

### Math and random numbers

`sqr`, `abs`, `int`, `fix`, `sgn`, `sin`, `cos`, `tan`, `atn`, `log`, and `exp` are available without setup. Angles for `sin`, `cos`, `tan`, and `atn` use radians; `log` is the natural logarithm. `rnd` produces random values, and `randomize` seeds it.

```bascal
print sin(0.0); " "; cos(0.0)
print sqr(81); " "; abs(-7); " "; cint(3.7)
```

### Files, records, and the environment

The classic file and environment functions also remain available: `eof`, `lof`, `loc`, `pos`, `csrlin`, `freefile`, `fre`, `lpos`, `date$`, `time$`, `timer`, `inkey$`, `environ$`, `command$`, `peek`, `inp`, `varptr`, `ubound`, `lbound`, and `iif`. For classic random-access records, `mki$`, `mkl$`, `mks$`, `mkd$`, `cvi`, `cvl`, `cvs`, and `cvd` pack and unpack values.

<div class="aside" markdown="1">

The BASIC target supports the complete list above. The C backend is almost complete but has a documented subset of target-specific features; consult the command-line reference when portability matters.

</div>

## Functions BASCAL supplies

Real MBASIC/BASCOM does not provide every useful text function. BASCAL supplies `ltrim$`, `rtrim$`, `ucase$`, and `lcase$` as scalar methods (see [Declare and call a method](functions-and-procedures.md#declare-and-call-a-method)), and `error$` as an ordinary function -- each needs its own `require` line:

```bascal
require com.bascal.stdlib.ltrim
require com.bascal.stdlib.rtrim
require com.bascal.stdlib.ucase
require com.bascal.stdlib.lcase
require com.bascal.stdlib.error

declare s$
s$ = "  hello world  "
print s$.ltrim().rtrim().ucase()
```

A method is really just a function with its receiver as an implicit first parameter, so the ordinary call form still works too -- `ucase$(rtrim$(ltrim$(s$)))` resolves to the exact same declarations as the chained call above.

`ltrim$(s$)` removes leading spaces and `rtrim$(s$)` removes trailing spaces. `ucase$(s$)` changes `a`–`z` to uppercase, while `lcase$(s$)` changes `A`–`Z` to lowercase; other characters are unchanged. `error$(code%)` turns a classic BASIC error number into a readable message, such as `error$(errFileNotFound%)` returning `"File not found"`.

The `error` library also exports named constants for its complete shared MBASIC/GW-BASIC/BASCOM error subset. Use these with `throw`, `error$`, and filtered `catch` clauses: `errSyntax%`, `errReturnWithoutGosub%`, `errOutOfData%`, `errIllegalFunctionCall%`, `errOverflow%`, `errOutOfMemory%`, `errSubscriptOutOfRange%`, `errDuplicateDefinition%`, `errDivisionByZero%`, `errTypeMismatch%`, `errOutOfStringSpace%`, `errNoResume%`, `errResumeWithoutError%`, `errDeviceTimeout%`, `errDeviceFault%`, `errOutOfPaper%`, `errBadFileNumber%`, `errFileNotFound%`, `errBadFileMode%`, `errFileAlreadyOpen%`, `errDeviceIo%`, `errFileAlreadyExists%`, `errDiskFull%`, `errInputPastEnd%`, `errBadRecordNumber%`, `errBadFileName%`, `errTooManyFiles%`, `errDeviceUnavailable%`, `errDiskWriteProtected%`, `errDiskNotReady%`, `errDiskMediaError%`, `errPathFileAccess%`, and `errPathNotFound%`.

These modules live under `com.bascal.stdlib` and use the same dependency rules as any other library. BASCAL also adds its internal `midAssign` helper automatically when you use the statement form of `mid$`; you never need to require it yourself.

For signatures, examples, and target details, see the manual’s [Standard Library Functions](../manual/standard-library-functions.md) chapter.
