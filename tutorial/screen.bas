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
220 ' bright yellow text on blue background
230 LOCATE 1, 30
240 PRINT "  BASCAL DEMO  "
250 COLOR 7, 0
260 ' restore white on black
270 LOCATE 3, 1
280 PRINT "Screen I/O tutorial"
290 ' Move to specific positions
300 LOCATE 5, 1
310 COLOR 10
320 PRINT "Green text"
330 ' bright green
340 LOCATE 6, 1
350 COLOR 12
360 PRINT "Red text"
370 ' bright red
380 LOCATE 7, 1
390 COLOR 11
400 PRINT "Cyan text"
410 ' bright cyan
420 LOCATE 8, 1
430 COLOR 7
440 PRINT "Normal text"
450 ' Sound the bell
460 BEEP
470 ' Printer output — comment out if no printer is attached
480 ' lprint "BASCAL screen demo printed at: " + DATE$
490 ' stop and system are for controlled termination:
500 ' stop   — pause (useful during debugging)
510 ' system — exit to OS immediately
520 ' Uncomment to test:
530 ' stop
540 ' system
550 COLOR 7, 0
560 LOCATE 25, 1
570 PRINT "Demo complete."
580 END
