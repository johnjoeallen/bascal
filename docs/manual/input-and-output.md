[Home](../../) / [Manual](../) / Input and Output

[← Arrays](arrays.md) [File Input and Output →](file-input-and-output.md)

<div class="prose" markdown="1">

### PRINT

Prints one or more expressions to the screen. Expressions are separated by commas or concatenated with `+`.

```bascal
PRINT "Hello, World!"
PRINT "Score: " + STR$(score%)
PRINT name$, score%
PRINT                              ' blank line
```

### LPRINT

Sends output to the printer (line printer). Same syntax as `PRINT`.

```bascal
LPRINT "BASCAL screen demo printed at: " + DATE$
LPRINT "Score: " + STR$(score%)
```

### INPUT

Reads values from the keyboard.

```bascal
INPUT name$
INPUT "Enter your name: "; name$
INPUT "Width, height: "; width%, height%
```

A prompt string followed by `;` suppresses the newline after the prompt (the cursor remains on the same line). A prompt followed by `,` adds a `?` and moves to the next print zone. The `;` form is recommended.

Multiple variables may be listed; the user enters values separated by commas.

### LOCATE

Positions the cursor before printing. From `tutorial/11_screen.bcl`:

```bascal
CLS
COLOR 14, 1            ' bright yellow on blue
LOCATE 1, 30
PRINT "  BASCAL DEMO  "

COLOR 7, 0             ' restore white on black
LOCATE 3, 1
PRINT "Screen I/O tutorial"

LOCATE 5, 1 : COLOR 10 : PRINT "Green text"
LOCATE 6, 1 : COLOR 12 : PRINT "Red text"
LOCATE 7, 1 : COLOR  7 : PRINT "Normal text"
```

Rows and columns are 1-based on standard 80×25 displays.

### COLOR

Sets the foreground and optional background colour.

```bascal
COLOR 14          ' bright yellow foreground, background unchanged
COLOR 15, 1       ' bright white on blue
COLOR 7, 0        ' grey on black (restore defaults)
```

Colour values follow CGA/EGA standard colour numbers (0–15 foreground, 0–7 background).

### BEEP

Sounds the system bell.

```bascal
BEEP
```

### CLS

Clears the screen.

```bascal
CLS
```

</div>

[← Arrays](arrays.md) [File Input and Output →](file-input-and-output.md)
