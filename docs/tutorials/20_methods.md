[Home](../../) / [Tutorials](../) / Scalar Methods

<div class="prose" markdown="1">

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/20_methods.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/20_methods.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/20_methods.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/20_methods.j).

A method is a typed operation attached to a scalar value. The suffix after `method` declares the receiver type: `method$` receives a string, `method%` receives an integer, and `method!` receives a single-precision number. The method name has its own result suffix, and the receiver is available as the matching implicit `self` variable.

</div>

<div class="snippet" markdown="1">

### Declare and call a method

```bascal
method$ shout$()
    return self$.ucase() + "!"
end method

print "hello".shout()
```

The receiver is passed implicitly. The method is still a typed callable; it is not a runtime object or a dynamically dispatched class method.

</div>

<div class="snippet" markdown="1">

### Chain only compatible results

```bascal
name$ = "bascal"
print name$.left(3).ucase()

score% = 125
print score%.clamp(0, 100)
```

`left` returns a string, so another string method can follow it. `clamp%` returns an integer, so its result can be assigned to an integer variable or passed where an integer is expected.

</div>

<div class="snippet" markdown="1">

### Type errors are compile-time errors

The resolver checks the receiver suffix, method name, argument count, argument types, and result suffix before either backend emits code. These examples are rejected:

```bascal
score%.shout$()       ' no shout$ method for an integer receiver
name$.clamp%(0, 10)  ' no clamp% method for a string receiver
price!.percent%(15)  ' wrong receiver type if percent% is declared
```

The same checking applies to user methods, methods from `require`d libraries, and built-in scalar methods such as `left`, `len`, `abs`, and `sin`. An unknown method or a mismatched argument cannot silently become a different ordinary call.

</div>

<div class="snippet" markdown="1">

### Methods transpile to ordinary typed calls

Methods are syntax for an implicit first parameter. The BASIC backend emits its normal typed parameter/result variables and `GOSUB`; the C backend emits typed calls and temporaries. The source-level method syntax and type checks are shared by both targets.

</div>

Full, real, transpiling source: [`20_methods.bcl`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/20_methods.bcl), [`20_methods.bas`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/20_methods.bas), and [`20_methods.c`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/20_methods.c).

[← Functions](07_functions.md)[Tutorials →](./)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/20_methods.bcl</code></summary>



```bascal

// Tutorial — Scalar methods
//
// A method has a typed scalar receiver, written after `method`, and a typed
// result suffix on its name. The receiver is available as self%/self!/self$
// in the body. Dot calls can chain when each result has the next receiver's
// type. Methods transpile to ordinary typed calls for both backends.
program methods

require com.bascal.stdlib.ucase

method$ shout$()
    return self$.ucase() + "!"
end method

method$ surround$(left$, right$)
    return left$ + self$ + right$
end method

method% clamp%(low%, high%)
    if self% < low% then
        return low%
    elseif self% > high% then
        return high%
    end if
    return self%
end method

method! percent!(rate!)
    return self! * rate! / 100
end method

name$ = "bascal"
result$ = name$.surround("[", "]")
print result$
shoutResult$ = name$.shout()
length% = name$.left(5).len()
print "length = "; length%

score% = 125
print "clamped score = "; score%.clamp(0, 100)

price! = 80
print "discount amount = "; price!.percent(15)

firstThree$ = name$.left(3).ucase()
print "first three = "; firstThree$

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/20_methods.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
40 ' against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
50 ' its own. Declared as a scalar method (see GitHub issue #41 and
60 ' ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
70 ' via ordinary-call syntax resolving to this same declaration.

80 ' Tutorial — Scalar methods
90 '
100 ' A method has a typed scalar receiver, written after `method`, and a typed
110 ' result suffix on its name. The receiver is available as self%/self!/self$
120 ' in the body. Dot calls can chain when each result has the next receiver's
130 ' type. Methods transpile to ordinary typed calls for both backends.

140 name$ = "bascal"
150 surroundSelf0$ = name$
160 surroundLeft0$ = "["
170 surroundRight0$ = "]"
180 GOSUB 620
190 result$ = surroundResult0$
200 PRINT result$
210 shoutSelf0$ = name$
220 GOSUB 560
230 shoutresult$ = shoutResult0$
240 length% = LEN(LEFT$(name$, 5))
250 PRINT "length = "; length%

260 score% = 125
270 clampSelf0% = score%
280 clampLow0% = 0
290 clampHigh0% = 100
300 GOSUB 660
310 PRINT "clamped score = "; clampResult0%

320 price! = 80
330 percentSelf0! = price!
340 percentRate0! = 15
350 GOSUB 790
360 PRINT "discount amount = "; percentResult0!

370 ucaseSelf0$ = LEFT$(name$, 3)
380 GOSUB 430
390 firstthree$ = ucaseResult0$
400 PRINT "first three = "; firstthree$

410 END

420 ' function ucase$()
430     ucaseOut0$ = ""
440     FOR ucaseI0% = 1 TO LEN(ucaseSelf0$)
450         ucaseC0% = ASC(MID$(ucaseSelf0$, ucaseI0%, 1))
460         IF (ucaseC0% >= 97) = 0 THEN GOTO 490
470         IF (ucaseC0% <= 122) = 0 THEN GOTO 490
480             ucaseC0% = ucaseC0% - 32
490         REM END IF
500         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
510     NEXT ucaseI0%
520     ucaseResult0$ = ucaseOut0$
530     RETURN
540 ' end function ucase$

550 ' function shout$()
560     ucaseSelf0$ = shoutSelf0$
570     GOSUB 430
580     shoutResult0$ = ucaseResult0$ + "!"
590     RETURN
600 ' end function shout$

610 ' function surround$(left$, right$)
620     surroundResult0$ = (surroundLeft0$ + surroundSelf0$) + surroundRight0$
630     RETURN
640 ' end function surround$

650 ' function clamp%(low%, high%)
660     IF (clampSelf0% < clampLow0%) = 0 THEN GOTO 700
670         clampResult0% = clampLow0%
680         RETURN
690         GOTO 740
700         IF (clampSelf0% > clampHigh0%) = 0 THEN GOTO 730
710             clampResult0% = clampHigh0%
720             RETURN
730         REM END IF
740     REM END IF
750     clampResult0% = clampSelf0%
760     RETURN
770 ' end function clamp%

780 ' function percent!(rate!)
790     percentResult0! = (percentSelf0! * percentRate0!) / 100
800     RETURN
810 ' end function percent!

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/20_methods.c</code></summary>



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

static float bv_f_price = 0;
static int bv_i_length = 0;
static int bv_i_score = 0;
static char bv_s_firstthree[256] = {0};
static char bv_s_name[256] = {0};
static char bv_s_result[256] = {0};
static char bv_s_shoutresult[256] = {0};

void bf_s_ucase_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_shout_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_surround_s(const char* bv_s_self_in, const char* bv_s_left_in, const char* bv_s_right_in, char* bcc_out);
int bf_i_clamp_i(int bv_i_self, int bv_i_low, int bv_i_high);
float bf_f_percent_f(float bv_f_self, float bv_f_rate);

void bf_s_ucase_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    int bv_i_c = 0;
    int bv_i_i = 0;
    char bv_s_out[256] = {0};

    snprintf(bv_s_out, sizeof(bv_s_out), "%s", "");
    int bt_lim_0 = ((int)strlen(bv_s_self));
    int bt_step_0 = 1;
    for (bv_i_i = 1; bt_step_0 >= 0 ? bv_i_i <= bt_lim_0 : bv_i_i >= bt_lim_0; bv_i_i += bt_step_0) {
        bv_i_c = ((int)(unsigned char)bcc_mid(bv_s_self, bv_i_i, 1)[0]);
        if (((-(bv_i_c >= 97)) && (-(bv_i_c <= 122)))) {
            bv_i_c = (bv_i_c - 32);
        }
        char bt_s_1[256];
        snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", bv_s_out, bcc_chr(bv_i_c));
        snprintf(bv_s_out, sizeof(bv_s_out), "%s", bt_s_1);
    }
    snprintf(bcc_out, 256, "%s", bv_s_out);
    return;
}

void bf_s_shout_s(const char* bv_s_self_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);

    char bt_s_2[256];
    bf_s_ucase_s(bv_s_self, bt_s_2);
    char bt_s_3[256];
    snprintf(bt_s_3, sizeof(bt_s_3), "%s%s", bt_s_2, "!");
    snprintf(bcc_out, 256, "%s", bt_s_3);
    return;
}

void bf_s_surround_s(const char* bv_s_self_in, const char* bv_s_left_in, const char* bv_s_right_in, char* bcc_out) {
    char bv_s_self[256];
    snprintf(bv_s_self, sizeof(bv_s_self), "%s", bv_s_self_in);
    char bv_s_left[256];
    snprintf(bv_s_left, sizeof(bv_s_left), "%s", bv_s_left_in);
    char bv_s_right[256];
    snprintf(bv_s_right, sizeof(bv_s_right), "%s", bv_s_right_in);

    char bt_s_4[256];
    snprintf(bt_s_4, sizeof(bt_s_4), "%s%s", bv_s_left, bv_s_self);
    char bt_s_5[256];
    snprintf(bt_s_5, sizeof(bt_s_5), "%s%s", bt_s_4, bv_s_right);
    snprintf(bcc_out, 256, "%s", bt_s_5);
    return;
}

int bf_i_clamp_i(int bv_i_self, int bv_i_low, int bv_i_high) {
    if ((-(bv_i_self < bv_i_low))) {
        return bv_i_low;
    } else {
        if ((-(bv_i_self > bv_i_high))) {
            return bv_i_high;
        }
    }
    return bv_i_self;
}

float bf_f_percent_f(float bv_f_self, float bv_f_rate) {
    return ((double)(bv_f_self * bv_f_rate) / (double)100);
}

int main(void) {
    // Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    // against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    // its own. Declared as a scalar method (see GitHub issue #41 and
    // ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
    // via ordinary-call syntax resolving to this same declaration.

    // Tutorial — Scalar methods
    //
    // A method has a typed scalar receiver, written after `method`, and a typed
    // result suffix on its name. The receiver is available as self%/self!/self$
    // in the body. Dot calls can chain when each result has the next receiver's
    // type. Methods transpile to ordinary typed calls for both backends.






    snprintf(bv_s_name, sizeof(bv_s_name), "%s", "bascal");
    char bt_s_6[256];
    bf_s_surround_s(bv_s_name, "[", "]", bt_s_6);
    snprintf(bv_s_result, sizeof(bv_s_result), "%s", bt_s_6);
    printf("%s\n", bv_s_result);
    char bt_s_7[256];
    bf_s_shout_s(bv_s_name, bt_s_7);
    snprintf(bv_s_shoutresult, sizeof(bv_s_shoutresult), "%s", bt_s_7);
    bv_i_length = ((int)strlen(bcc_mid(bv_s_name, 1, 5)));
    printf("length = %d\n", bv_i_length);

    bv_i_score = 125;
    printf("clamped score = %d\n", bf_i_clamp_i(bv_i_score, 0, 100));

    bv_f_price = 80;
    printf("discount amount = %g\n", bf_f_percent_f(bv_f_price, 15));

    char bt_s_8[256];
    bf_s_ucase_s(bcc_mid(bv_s_name, 1, 3), bt_s_8);
    snprintf(bv_s_firstthree, sizeof(bv_s_firstthree), "%s", bt_s_8);
    printf("first three = %s\n", bv_s_firstthree);

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
