## Naming a destination without a line number

Structured `if` and loops cover ordinary control flow, but a few classic BASIC mechanisms need to jump to a specific place. BASCAL keeps that capability without numbered source lines: a `label:` names a destination, and the compiler assigns its real line number.

```bascal
gosub greet
goto after

greet:
    print "Hello!"
    return

after:
    print "done"
```

`goto label` jumps unconditionally; `gosub label` jumps and remembers where to return. This BASIC subroutine mechanism is distinct from a BASCAL `function`/`procedure` call.

## Handling an error at the point that needs it

For code that should run on both targets, use structured `try`/`catch`/`finally`. It abandons the complete `try` region when a runtime error occurs, makes the error code, source line, and source file available to `catch`, always runs `finally`, and then continues after `end try`.

```bascal
try
    checkPart()
catch err%, erl%, source$
    print "error "; err%; " at "; source$; ":"; erl%
finally
    closePart()
end try
```

`err%`, `erl%`, and `source$` are ordinary locals scoped to the `catch` block; they are not the ambient `err`/`erl` pseudo-variables. `source$` identifies the `.bcl` source file that raised the error. `catch` is optional when only cleanup is needed: an unhandled error is re-raised after `finally` runs. There is no `resume`: a failed action is always abandoned, never resumed partway through a procedure.

On the BASIC target, `try`/`catch`/`finally` transpiles to real `ON ERROR GOTO`/`RESUME <label>`. On the C target, a raise reaches the owning `catch` when it occurs in the `try` block or in a procedure/function called from it, including when that function call is part of a larger expression such as `x% = boom%() + 1`. Tries may be nested: each `finally` runs before an unhandled error reaches the enclosing `catch` or escapes its function. A catch that completes normally suppresses its error; bare `throw` rethrows it after that finally block, while `throw n` throws error `n`.

### Classic BASIC recovery: `--target basic` only

`on error goto`/`resume` remains available when generating BASIC, for programs that need its exact label-and-resume control flow. It is not portable and is not available on the C backend.

```bascal
on error goto handler
result% = 10 / zero%
print "unreachable"
goto continue

handler:
    print "error "; err; " on line "; erl
    resume continue

continue:
    print "recovered"
```

Inside the handler, the no-suffix pseudo-variables `err` and `erl` hold the failing error code and line. `resume` retries the failed statement; `resume next` skips it; `resume label` jumps elsewhere, as above. `on error goto 0` disables the trap, and `error code%` raises or re-raises an error deliberately.
