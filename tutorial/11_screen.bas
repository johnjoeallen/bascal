10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 11 — Screen I/O: CLS, LOCATE, COLOR, BEEP, LPRINT
40 ' 
50 ' These statements control the terminal display and connected hardware.
60 ' They map directly to the same-named BASCOM statements.
70 ' 
80 ' CLS             — clear the screen
90 ' LOCATE row, col — move cursor; rows and columns are 1-based (80×25)
100 ' COLOR fg[, bg]  — CGA colour numbers: 0-15 foreground, 0-7 background
110 ' 0 black  1 blue    2 green   3 cyan
120 ' 4 red    5 magenta 6 brown   7 white
130 ' 8-15: bright versions of 0-7
140 ' BEEP            — sound the system bell
150 ' LPRINT expr     — send output to the line printer
160 ' 
170 ' STOP   — halt execution (may invoke debugger)
180 ' SYSTEM — exit to the operating system immediately

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
430 ' LPRINT "BASCAL screen demo printed at: " + DATE$

440 ' STOP and SYSTEM are for controlled termination:
450 ' STOP   — pause (useful during debugging)
460 ' SYSTEM — exit to OS immediately
470 ' Uncomment to test:
480 ' STOP
490 ' SYSTEM

500 COLOR 7, 0
510 LOCATE 25, 1
520 PRINT "Demo complete."
530 END
