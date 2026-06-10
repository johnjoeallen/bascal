' BASCAL generated BASIC
' Functions are lowered to global variables, labels, and GOSUB

' Tutorial 11 — Screen I/O: CLS, LOCATE, COLOR, BEEP, LPRINT
' 
' These statements control the terminal display and connected hardware.
' They map directly to the same-named BASCOM statements.
' 
' CLS             — clear the screen
' LOCATE row, col — move cursor; rows and columns are 1-based (80×25)
' COLOR fg[, bg]  — CGA colour numbers: 0-15 foreground, 0-7 background
' 0 black  1 blue    2 green   3 cyan
' 4 red    5 magenta 6 brown   7 white
' 8-15: bright versions of 0-7
' BEEP            — sound the system bell
' LPRINT expr     — send output to the line printer
' 
' STOP   — halt execution (may invoke debugger)
' SYSTEM — exit to the operating system immediately

' Clear screen and draw a simple title banner
CLS

COLOR 14, 1
LOCATE 1, 30
PRINT "  BASCAL DEMO  "

COLOR 7, 0
LOCATE 3, 1
PRINT "Screen I/O tutorial"

' Move to specific positions
LOCATE 5, 1
COLOR 10
PRINT "Green text"
LOCATE 6, 1
COLOR 12
PRINT "Red text"
LOCATE 7, 1
COLOR 11
PRINT "Cyan text"
LOCATE 8, 1
COLOR 7
PRINT "Normal text"

' Sound the bell
BEEP

' Printer output — comment out if no printer is attached
' LPRINT "BASCAL screen demo printed at: " + DATE$

' STOP and SYSTEM are for controlled termination:
' STOP   — pause (useful during debugging)
' SYSTEM — exit to OS immediately
' Uncomment to test:
' STOP
' SYSTEM

COLOR 7, 0
LOCATE 25, 1
PRINT "Demo complete."
END
