[Home](../../) / [Tutorials](../) / Screen I/O

<div class="prose" markdown="1">

`cls` clears the screen, `locate row, col` moves the cursor (1-based, 80x25), `color fg[, bg]` sets CGA foreground/background colors, `beep` sounds the bell, and `lprint` sends output to the line printer. `stop` halts execution for controlled debugging, and `system` exits straight to the operating system. These map directly to the same-named BASCOM statements — BASCAL adds no abstraction here, just structure around them.

</div>

<div class="snippet" markdown="1">

### A title banner

```bascal
cls

color 14, 1           // bright yellow text on blue background
locate 1, 30
print "  BASCAL DEMO  "

color 7, 0            // restore white on black
```

</div>



[← File Input and Output](files.md)  ·  [Require and Multi-File Projects →](require.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/screen.bcl</code></summary>



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



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/screen.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
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



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/screen.c</code></summary>



```c

// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <stdlib.h>

static void bcc_color(int fg, int bg);

int main(void) {
    // Tutorial — Screen I/O: cls, locate, color, beep, lprint
    //
    // These statements control the terminal display and connected hardware.
    // They map directly to the same-named BASCOM statements.
    //
    // cls             — clear the screen
    // locate row, col — move cursor; rows and columns are 1-based (80×25)
    // color fg[, bg]  — CGA colour numbers: 0-15 foreground, 0-7 background
    // 0 black  1 blue    2 green   3 cyan
    // 4 red    5 magenta 6 brown   7 white
    // 8-15: bright versions of 0-7
    // beep            — sound the system bell
    // lprint expr     — send output to the line printer
    //
    // stop   — halt execution (may invoke debugger)
    // system — exit to the operating system immediately

    // Clear screen and draw a simple title banner
    printf("\x1b[2J\x1b[H");

    bcc_color(14, 1);
    printf("\x1b[%d;%dH", 1, 30);
    printf("  BASCAL DEMO  \n");

    bcc_color(7, 0);
    printf("\x1b[%d;%dH", 3, 1);
    printf("Screen I/O tutorial\n");

    // Move to specific positions
    printf("\x1b[%d;%dH", 5, 1);
    bcc_color(10, -1);
    printf("Green text\n");
    printf("\x1b[%d;%dH", 6, 1);
    bcc_color(12, -1);
    printf("Red text\n");
    printf("\x1b[%d;%dH", 7, 1);
    bcc_color(11, -1);
    printf("Cyan text\n");
    printf("\x1b[%d;%dH", 8, 1);
    bcc_color(7, -1);
    printf("Normal text\n");

    // Sound the bell
    printf("\a");

    // Printer output — comment out if no printer is attached
    // lprint "BASCAL screen demo printed at: " + DATE$

    // stop and system are for controlled termination:
    // stop   — pause (useful during debugging)
    // system — exit to OS immediately
    // Uncomment to test:
    // stop
    // system

    bcc_color(7, 0);
    printf("\x1b[%d;%dH", 25, 1);
    printf("Demo complete.\n");
    return 0;
}

static const int bcc_ansi_fg[16] = {30, 34, 32, 36, 31, 35, 33, 37, 90, 94, 92, 96, 91, 95, 93, 97};
static const int bcc_ansi_bg[8] = {40, 44, 42, 46, 41, 45, 43, 47};
static int bcc_color_used = 0;

static void bcc_color_reset(void) {
    printf("\x1b[0m");
}

static void bcc_color(int fg, int bg) {
    if (!bcc_color_used) {
        atexit(bcc_color_reset);
        bcc_color_used = 1;
    }
    printf("\x1b[%dm", bcc_ansi_fg[fg & 15]);
    if (bg >= 0) {
        printf("\x1b[%dm", bcc_ansi_bg[bg & 7]);
    }
}


```



</details>

<!-- END generated tutorial source -->
