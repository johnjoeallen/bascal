[Home](../) / [Tutorials](./) / Screen I/O

<div class="prose" markdown="1">

`cls` clears the screen, `locate row, col` moves the cursor (1-based, 80x25), `color fg[, bg]` sets CGA foreground/background colors, `beep` sounds the bell, and `lprint` sends output to the line printer. `stop` halts execution for controlled debugging, and `system` exits straight to the operating system. These map directly to the same-named BASCOM statements — BASCAL adds no abstraction here, just structure around them.

</div>

<div class="snippet" markdown="1">

### A title banner

    cls

    color 14, 1           // bright yellow text on blue background
    locate 1, 30
    print "  BASCAL DEMO  "

    color 7, 0            // restore white on black

</div>



[← File Input and Output](10_files.md)  ·  [Require and Multi-File Projects →](12_require.md)


<!-- BEGIN generated tutorial source -->

### `tutorial/11_screen.bcl`

```bascal

// Tutorial — Screen I/O: cls, locate, color, beep, lprint
//
// These statements control the terminal display and connected hardware.
// They map directly to the same-named BASCOM statements.
//
// cls             — clear the screen
// locate row, col — move cursor; rows and columns are 1-based (80×25)
// color fg[, bg]  — CGA colour numbers: 0-15 foreground, 0-7 background
//                   0 black  1 blue    2 green   3 cyan
//                   4 red    5 magenta 6 brown   7 white
//                   8-15: bright versions of 0-7
// beep            — sound the system bell
// lprint expr     — send output to the line printer
//
// stop   — halt execution (may invoke debugger)
// system — exit to the operating system immediately
program screen

/* Clear screen and draw a simple title banner */
cls

color 14, 1           // bright yellow text on blue background
locate 1, 30
print "  BASCAL DEMO  "

color 7, 0            // restore white on black
locate 3, 1
print "Screen I/O tutorial"

/* Move to specific positions */
locate 5, 1  : color 10 : print "Green text"   // bright green
locate 6, 1  : color 12 : print "Red text"     // bright red
locate 7, 1  : color 11 : print "Cyan text"    // bright cyan
locate 8, 1  : color 7  : print "Normal text"

/* Sound the bell */
beep

/* Printer output — comment out if no printer is attached */
' lprint "BASCAL screen demo printed at: " + DATE$

/* stop and system are for controlled termination:
 *   stop   — pause (useful during debugging)
 *   system — exit to OS immediately
 * Uncomment to test:
 */
' stop
' system

color 7, 0
locate 25, 1
print "Demo complete."
end

```

### `tutorial/11_screen.bas`

```basic

10 ' BASCAL generated BASIC
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Tutorial — Screen I/O: cls, locate, color, beep, lprint
40 '
50 ' These statements control the terminal display and connected hardware.
60 ' They map directly to the same-named BASCOM statements.
70 '
80 ' cls             — clear the screen
90 ' locate row, col — move cursor; rows and columns are 1-based (80×25)
100 ' color fg[, bg]  — CGA colour numbers: 0-15 foreground, 0-7 background
110 ' 0 black  1 blue    2 green   3 cyan
120 ' 4 red    5 magenta 6 brown   7 white
130 ' 8-15: bright versions of 0-7
140 ' beep            — sound the system bell
150 ' lprint expr     — send output to the line printer
160 '
170 ' stop   — halt execution (may invoke debugger)
180 ' system — exit to the operating system immediately

190 ' Clear screen and draw a simple title banner
200 CLS

210 COLOR 14, 1
220 LOCATE 1, 30
230 PRINT "  BASCAL DEMO  "

240 COLOR 7, 0
250 LOCATE 3, 1
260 PRINT "Screen I/O tutorial"

270 ' Move to specific positions
280 LOCATE 5, 1
290 COLOR 10
300 PRINT "Green text"
310 LOCATE 6, 1
320 COLOR 12
330 PRINT "Red text"
340 LOCATE 7, 1
350 COLOR 11
360 PRINT "Cyan text"
370 LOCATE 8, 1
380 COLOR 7
390 PRINT "Normal text"

400 ' Sound the bell
410 BEEP

420 ' Printer output — comment out if no printer is attached
430 ' lprint "BASCAL screen demo printed at: " + DATE$

440 ' stop and system are for controlled termination:
450 ' stop   — pause (useful during debugging)
460 ' system — exit to OS immediately
470 ' Uncomment to test:
480 ' stop
490 ' system

500 COLOR 7, 0
510 LOCATE 25, 1
520 PRINT "Demo complete."
530 END

```

<!-- END generated tutorial source -->
