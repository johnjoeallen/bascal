[Home](../../) / [Tutorials](../) / Procedures

<div class="prose" markdown="1">

A procedure is declared `procedure name(...) ... end procedure` — the name carries no type suffix, since it returns nothing. Variables are local by default, exactly like functions, and `global varname` reaches out to shared state. A bare `return` exits early; falling through to `end procedure` is also fine.

</div>

<div class="snippet" markdown="1">

### Early return from a procedure

```bascal
procedure printIfPass(name$, score%)
    if score% < 60 then
        return          // early exit -- nothing printed for failing scores
    end if
    print name$ + " passed with " + str$(score%)
end procedure
```

</div>



[← Shared COMMON](shared.md)  ·  [Random-Access and Record Files →](random_and_record_files.md)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/procedures.bcl</code></summary>



```bascal

// Tutorial — Procedures
//
// A procedure is like a function but returns no value.  Declare it with
// PROCEDURE ... END PROCEDURE.  The name must not carry a type suffix.
//
// Variables inside a procedure are LOCAL by default: the compiler prefixes
// them with the procedure name.  To access a global variable, declare it
// inside the body with:  global varname
//
// Use procedures for actions that produce side effects (output, file I/O,
// modifying arrays) rather than for computing a value.
//
// A bare RETURN exits a procedure early.  Falling through to END PROCEDURE
// is also valid — an implicit RETURN is emitted.
program procedures

/* Procedure with no parameters */
procedure printSeparator()
    print "----------------------------"
end procedure

/* Procedure that prints a labelled value */
// label$ -- text shown before the score
// score% -- value to print
procedure printScore(label$, score%)
    print label$ + ": " + str$(score%)
end procedure

/* Procedure with early exit */
// name$  -- person's name
// score% -- score to test against the passing threshold
procedure printIfPass(name$, score%)
    if score% < 60 then
        return // early exit — nothing printed for failing scores
    end if
    print name$ + " passed with " + str$(score%)
end procedure

/* Procedure that modifies an array in place -- byref copies the result */
/* back to the caller; the default byval would fill a private copy only. */
// arr%   -- array to fill; byref because it's mutated in place
// value% -- value written into every element
procedure fillRange(byref arr%(?), value%)
    for i% = 0 to sizeof(arr%) - 1
        arr%(i%) = value%
    end for
end procedure

/* Procedure that uses a global variable */
globalCount% = 0

procedure increment()
    global globalCount%
    globalCount% = globalCount% + 1
end procedure

/* --- Drive the procedures --- */

printSeparator()
printScore("Alice", 91)
printScore("Bob", 54)
printScore("Carol", 78)
printSeparator()

print "Passes only:"
printIfPass("Alice", 91) // printed
printIfPass("Bob", 54) // skipped (score < 60)
printIfPass("Carol", 78) // printed

const N = 5
dim data%(N)
fillRange(data%, 99)
print "Filled array:"
for i% = 0 to N - 1
    print "  data%(" + str$(i%) + ") = " + str$(data%(i%))
end for

increment()
increment()
increment()
print "globalCount = " + str$(globalCount%) // 3

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/procedures.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Storage for array parameters, sized to fit every call site
40 DIM fillrangeArr0%(5)

50 ' Tutorial — Procedures
60 '
70 ' A procedure is like a function but returns no value.  Declare it with
80 ' PROCEDURE ... END PROCEDURE.  The name must not carry a type suffix.
90 '
100 ' Variables inside a procedure are LOCAL by default: the compiler prefixes
110 ' them with the procedure name.  To access a global variable, declare it
120 ' inside the body with:  global varname
130 '
140 ' Use procedures for actions that produce side effects (output, file I/O,
150 ' modifying arrays) rather than for computing a value.
160 '
170 ' A bare RETURN exits a procedure early.  Falling through to END PROCEDURE
180 ' is also valid — an implicit RETURN is emitted.
190 ' Procedure with no parameters
200 ' Procedure that prints a labelled value
210 ' label$ -- text shown before the score
220 ' score% -- value to print
230 ' Procedure with early exit
240 ' name$  -- person's name
250 ' score% -- score to test against the passing threshold
260 ' Procedure that modifies an array in place -- byref copies the result
270 ' back to the caller; the default byval would fill a private copy only.
280 ' arr%   -- array to fill; byref because it's mutated in place
290 ' value% -- value written into every element
300 ' Procedure that uses a global variable
310 globalcount% = 0
320 ' --- Drive the procedures ---
330 GOSUB 830
340 printscoreLabel0$ = "Alice"
350 printscoreScore0% = 91
360 GOSUB 870
370 printscoreLabel0$ = "Bob"
380 printscoreScore0% = 54
390 GOSUB 870
400 printscoreLabel0$ = "Carol"
410 printscoreScore0% = 78
420 GOSUB 870
430 GOSUB 830
440 PRINT "Passes only:"
450 printifpassName0$ = "Alice"
460 printifpassScore0% = 91
470 GOSUB 910
480 ' printed
490 printifpassName0$ = "Bob"
500 printifpassScore0% = 54
510 GOSUB 910
520 ' skipped (score < 60)
530 printifpassName0$ = "Carol"
540 printifpassScore0% = 78
550 GOSUB 910
560 ' printed
570 CONSTN% = 5
580 DIM data%(CONSTN%)
590 BCCT1% = CONSTN%
600 fillrangeArrDim00% = BCCT1%
610 IF fillrangeArrDim00% > 5 THEN PRINT "runtime error: `arr%` of `fillRange` needs "; fillrangeArrDim00%; " elements along axis 0, but its storage only holds 5" : STOP

620 ' copy array argument into transpiled procedure storage: data%() -> fillrangeArr0%()
630 FOR BCCT2% = 0 TO fillrangeArrDim00%
640     fillrangeArr0%(BCCT2%) = data%(BCCT2%)
650 NEXT BCCT2%

660 fillrangeValue0% = 99
670 GOSUB 990

680 ' copy mutated array argument back to caller storage: fillrangeArr0%() -> data%()
690 FOR BCCT3% = 0 TO fillrangeArrDim00%
700     data%(BCCT3%) = fillrangeArr0%(BCCT3%)
710 NEXT BCCT3%

720 PRINT "Filled array:"
730 FOR i% = 0 TO CONSTN% - 1
740     PRINT (("  data%(" + STR$(i%)) + ") = ") + STR$(data%(i%))
750 NEXT i%
760 GOSUB 1050
770 GOSUB 1050
780 GOSUB 1050
790 PRINT "globalCount = " + STR$(globalcount%)
800 ' 3
810 END

820 ' procedure printseparator()
830     PRINT "----------------------------"
840     RETURN
850 ' end procedure printseparator

860 ' procedure printscore(label$, score%)
870     PRINT (printscoreLabel0$ + ": ") + STR$(printscoreScore0%)
880     RETURN
890 ' end procedure printscore

900 ' procedure printifpass(name$, score%)
910     IF (printifpassScore0% < 60) = 0 THEN GOTO 940
920         RETURN
930         ' early exit — nothing printed for failing scores
940     REM END IF
950     PRINT (printifpassName0$ + " passed with ") + STR$(printifpassScore0%)
960     RETURN
970 ' end procedure printifpass

980 ' procedure fillrange(arr%, value%)
990     FOR fillrangeI0% = 0 TO (fillrangeArrDim00% + 1) - 1
1000         fillrangeArr0%(fillrangeI0%) = fillrangeValue0%
1010     NEXT fillrangeI0%
1020     RETURN
1030 ' end procedure fillrange

1040 ' procedure increment()
1050     globalcount% = globalcount% + 1
1060     RETURN
1070 ' end procedure increment

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/procedures.c</code></summary>



```c

// BASCAL generated C -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
#include <stdio.h>
#include <string.h>

#define BCC_STRBUF_COUNT 8
static char bcc_strbuf[BCC_STRBUF_COUNT][256];
static int bcc_strbuf_next = 0;

static char* bcc_strbuf_take(void);
static const char* bcc_mid(const char* s, int start, int length);
static const char* bcc_chr(int code);
static const char* bcc_stri(int value);
static const char* bcc_strd(double value);

static int bv_i_globalcount = 0;
static int bv_i_i = 0;
static int bv_i_n = 0;
static int bv_i_data[6] = {0};

void bf_i_printseparator(void);
void bf_i_printscore(const char* bv_s_label_in, int bv_i_score);
void bf_i_printifpass(const char* bv_s_name_in, int bv_i_score);
void bf_i_fillrange(int bv_i_arr_len0, int* bv_i_arr, int bv_i_value);
void bf_i_increment(void);

void bf_i_printseparator(void) {
    printf("----------------------------\n");
}

void bf_i_printscore(const char* bv_s_label_in, int bv_i_score) {
    char bv_s_label[256];
    snprintf(bv_s_label, sizeof(bv_s_label), "%s", bv_s_label_in);

    char bt_s_0[256];
    snprintf(bt_s_0, sizeof(bt_s_0), "%s%s", bv_s_label, ": ");
    char bt_s_1[256];
    snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", bt_s_0, bcc_stri(bv_i_score));
    printf("%s\n", bt_s_1);
}

void bf_i_printifpass(const char* bv_s_name_in, int bv_i_score) {
    char bv_s_name[256];
    snprintf(bv_s_name, sizeof(bv_s_name), "%s", bv_s_name_in);

    if ((-(bv_i_score < 60))) {
        return;
        // early exit — nothing printed for failing scores
    }
    char bt_s_2[256];
    snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", bv_s_name, " passed with ");
    char bt_s_3[256];
    snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bt_s_2, bcc_stri(bv_i_score));
    printf("%s\n", bt_s_3);
}

void bf_i_fillrange(int bv_i_arr_len0, int* bv_i_arr, int bv_i_value) {
    int bv_i_i = 0;

    int bt_start_4 = 0;
    int bt_lim_4 = (((bv_i_arr_len0 - 1) + 1) - 1);
    int bt_step_4 = 1;
    for (bv_i_i = bt_start_4; bt_step_4 >= 0 ? bv_i_i <= bt_lim_4 : bv_i_i >= bt_lim_4; bv_i_i += bt_step_4) {
        bv_i_arr[(bv_i_i)] = bv_i_value;
    }
}

void bf_i_increment(void) {
    bv_i_globalcount = (bv_i_globalcount + 1);
}

int main(void) {
    // Tutorial — Procedures
    //
    // A procedure is like a function but returns no value.  Declare it with
    // PROCEDURE ... END PROCEDURE.  The name must not carry a type suffix.
    //
    // Variables inside a procedure are LOCAL by default: the compiler prefixes
    // them with the procedure name.  To access a global variable, declare it
    // inside the body with:  global varname
    //
    // Use procedures for actions that produce side effects (output, file I/O,
    // modifying arrays) rather than for computing a value.
    //
    // A bare RETURN exits a procedure early.  Falling through to END PROCEDURE
    // is also valid — an implicit RETURN is emitted.

    // Procedure with no parameters

    // Procedure that prints a labelled value
    // label$ -- text shown before the score
    // score% -- value to print

    // Procedure with early exit
    // name$  -- person's name
    // score% -- score to test against the passing threshold

    // Procedure that modifies an array in place -- byref copies the result
    // back to the caller; the default byval would fill a private copy only.
    // arr%   -- array to fill; byref because it's mutated in place
    // value% -- value written into every element

    // Procedure that uses a global variable
    bv_i_globalcount = 0;

    // --- Drive the procedures ---

    bf_i_printseparator();
    char bt_arg_5[256];
    snprintf(bt_arg_5, sizeof(bt_arg_5), "%s", "Alice");
    int bt_arg_6 = 91;
    bf_i_printscore(bt_arg_5, bt_arg_6);
    char bt_arg_7[256];
    snprintf(bt_arg_7, sizeof(bt_arg_7), "%s", "Bob");
    int bt_arg_8 = 54;
    bf_i_printscore(bt_arg_7, bt_arg_8);
    char bt_arg_9[256];
    snprintf(bt_arg_9, sizeof(bt_arg_9), "%s", "Carol");
    int bt_arg_10 = 78;
    bf_i_printscore(bt_arg_9, bt_arg_10);
    bf_i_printseparator();

    printf("Passes only:\n");
    char bt_arg_11[256];
    snprintf(bt_arg_11, sizeof(bt_arg_11), "%s", "Alice");
    int bt_arg_12 = 91;
    bf_i_printifpass(bt_arg_11, bt_arg_12);
    char bt_arg_13[256];
    snprintf(bt_arg_13, sizeof(bt_arg_13), "%s", "Bob");
    int bt_arg_14 = 54;
    bf_i_printifpass(bt_arg_13, bt_arg_14);
    char bt_arg_15[256];
    snprintf(bt_arg_15, sizeof(bt_arg_15), "%s", "Carol");
    int bt_arg_16 = 78;
    bf_i_printifpass(bt_arg_15, bt_arg_16);

    bv_i_n = 5;
    int bt_arg_17 = 99;
    bf_i_fillrange(6, bv_i_data, bt_arg_17);
    printf("Filled array:\n");
    int bt_lim_18 = (bv_i_n - 1);
    int bt_step_18 = 1;
    for (bv_i_i = 0; bt_step_18 >= 0 ? bv_i_i <= bt_lim_18 : bv_i_i >= bt_lim_18; bv_i_i += bt_step_18) {
        char bt_s_19[256];
        snprintf(bt_s_19, sizeof(bt_s_19), "%s%s", "  data%(", bcc_stri(bv_i_i));
        char bt_s_20[256];
        snprintf(bt_s_20, sizeof(bt_s_20), "%s%s", bt_s_19, ") = ");
        char bt_s_21[256];
        snprintf(bt_s_21, sizeof(bt_s_21), "%s%s", bt_s_20, bcc_stri(bv_i_data[(bv_i_i)]));
        printf("%s\n", bt_s_21);
    }

    bf_i_increment();
    bf_i_increment();
    bf_i_increment();
    char bt_s_22[256];
    snprintf(bt_s_22, sizeof(bt_s_22), "%s%s", "globalCount = ", bcc_stri(bv_i_globalcount));
    printf("%s\n", bt_s_22);

    return 0;
}

static char* bcc_strbuf_take(void) {
    char* buf = bcc_strbuf[bcc_strbuf_next];
    bcc_strbuf_next = (bcc_strbuf_next + 1) % BCC_STRBUF_COUNT;
    return buf;
}

static const char* bcc_mid(const char* s, int start, int length) {
    char* out = bcc_strbuf_take();
    int len = (int)strlen(s);
    int from = start - 1;
    if (from < 0) from = 0;
    if (from > len) from = len;
    int avail = len - from;
    if (length < 0) length = 0;
    if (length > avail) length = avail;
    snprintf(out, 256, "%.*s", length, s + from);
    return out;
}

static const char* bcc_chr(int code) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "%c", code);
    return out;
}

static const char* bcc_stri(int value) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "% d", value);
    return out;
}

static const char* bcc_strd(double value) {
    char* out = bcc_strbuf_take();
    snprintf(out, 256, "% g", value);
    return out;
}


```



</details>

<!-- END generated tutorial source -->
