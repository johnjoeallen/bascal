.version 50 0
.class public Screen
.super java/lang/Object

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 1

    ; Tutorial — Screen I/O: cls, locate, color, beep, lprint
    ;
    ; These statements control the terminal display and connected hardware.
    ; They map directly to the same-named BASCOM statements.
    ;
    ; cls             — clear the screen
    ; locate row, col — move cursor; rows and columns are 1-based (80×25)
    ; color fg[, bg]  — CGA colour numbers: 0-15 foreground, 0-7 background
    ; 0 black  1 blue    2 green   3 cyan
    ; 4 red    5 magenta 6 brown   7 white
    ; 8-15: bright versions of 0-7
    ; beep            — sound the system bell
    ; lprint expr     — send output to the line printer
    ;
    ; stop   — halt execution (may invoke debugger)
    ; system — exit to the operating system immediately

    ; Clear screen and draw a simple title banner
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[2J[H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[36;41m"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[1;30H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "  BASCAL DEMO  "
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[37;40m"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[3;1H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Screen I/O tutorial"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; Move to specific positions
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[5;1H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[32m"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Green text"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[6;1H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[34m"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Red text"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[7;1H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[33m"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Cyan text"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[8;1H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[37m"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Normal text"
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; Sound the bell
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc ""
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V

    ; Printer output — comment out if no printer is attached
    ; lprint "BASCAL screen demo printed at: " + DATE$

    ; stop and system are for controlled termination:
    ; stop   — pause (useful during debugging)
    ; system — exit to OS immediately
    ; Uncomment to test:
    ; stop
    ; system

    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[37;40m"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "[25;1H"
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "Demo complete."
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    return
.end method
