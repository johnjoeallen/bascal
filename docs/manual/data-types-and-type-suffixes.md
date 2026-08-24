[Home](../../) / [Manual](../) / Data Types and Type Suffixes

[← Program Structure](program-structure.md) [Variables and Constants →](variables-and-constants.md)

<div class="prose" markdown="1">

BASCAL uses Microsoft BASIC's type-suffix convention. Every variable or function name carries its type in the final character:

| Suffix | Type    | Range / Notes                          |
|--------|---------|----------------------------------------|
| `%`    | Integer | 16-bit signed, -32768 to 32767         |
| `$`    | String  | Variable-length string                 |
| `!`    | Single  | 32-bit IEEE 754 single-precision float |
| `#`    | Double  | 64-bit IEEE 754 double-precision float |
| `&`    | Long    | 32-bit signed integer                  |

Variables without a suffix follow the DEFtype settings of the BASIC runtime (default: single precision). In BASCAL source it is strongly recommended to always use explicit suffixes.

All type checking is deferred to the BASIC runtime. The BASCAL transpiler does not perform static type inference.

</div>

[← Program Structure](program-structure.md) [Variables and Constants →](variables-and-constants.md)
