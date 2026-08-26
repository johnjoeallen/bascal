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

