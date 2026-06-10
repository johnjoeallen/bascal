10 ' BASCAL generated BASIC
20 ' Functions are lowered to global variables, labels, and GOSUB

30 ' Tutorial 1 — Hello, World
40 ' 
50 ' The simplest BASCAL program.  print writes a line to the screen.
60 ' END marks the bottom of the main program body; every program needs one.
70 ' 
80 ' Three comment styles are available:
90 ' '  single-line (BASIC style, passed through to generated output)
100 ' // single-line (C style, same behaviour as ')
110 ' /* ... */  block comment, each line becomes a ' comment in the output

120 PRINT "Hello, World!"
130 PRINT "Welcome to BASCAL."
140 ' print "This line is commented out."

150 ' Expected output:
160 ' Hello, World!
170 ' Welcome to BASCAL.

180 END
