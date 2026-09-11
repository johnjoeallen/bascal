[Home](../../) / [Tutorials](../) / Methods

<div class="prose" markdown="1">

Generated sources: [BCL](https://github.com/johnjoeallen/bascal/blob/main/tutorial/methods.bcl), [BASIC](https://github.com/johnjoeallen/bascal/blob/main/tutorial/methods.bas), [C](https://github.com/johnjoeallen/bascal/blob/main/tutorial/methods.c), and [JVM assembly](https://github.com/johnjoeallen/bascal/blob/main/tutorial/methods.j).

A method is a typed operation attached to a receiver. Both scalar and record receivers are supported (record receivers are covered in the [Methods](../language/methods.md) chapter). The bracket after a method's name declares its receiver type: `method shout[string]()` receives a string. The return type, if any, follows the parameter list after `:` (`method shout[string](): string`, or its suffix shorthand `method shout[string](): $`). If no return type is supplied, a scalar method returns the receiver's own scalar type, falling through to an implicit `return self`. The receiver is available in the body as the matching implicit `self` variable.

</div>

<div class="snippet" markdown="1">

### Declare and call a method

```bascal
method shout[string]()
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

### Record methods

Methods are not limited to scalars. A record type is a valid method receiver, declared either externally (in brackets, same as a scalar receiver) or inline, directly inside the `record` declaration itself:

```bascal
record Card
    title:  string(40)
    author: string(40)
    copies: int

    method display(): $
        return self.title + " by " + self.author
    end method
end record

method restock[Card](amount%)
    self.copies = self.copies + amount%
end method

let card = { title: "Dune", author: "Frank Herbert", copies: 2 }
print card.display()
card.restock(3)
print "copies on hand = "; card.copies
```

`display` is declared inline, `restock` externally — both are the same kind of callable, resolved the same way. `self.field` inside a record method is ordinary field access against the receiver, and mutating `self.field` (as `restock` does) is visible to the caller once the call returns: `card.copies` reads `5`, not `2`, after `card.restock(3)`. The receiver is passed the same way a C-level receiver naturally would be, by reference rather than by copy. Record methods do not yet extend to nested record fields, record parameters, or arrays of records — the existing random-access `file`/record DSL remains the separate mechanism for on-disk records.

</div>

<div class="snippet" markdown="1">

### Structural composition with `combines`

`record SignedCard combines Card` makes `SignedCard` composed of `Card`'s fields — structurally, not as subtype polymorphism:

```bascal
record SignedCard combines Card
    signature: string(40)

    method display(): $
        return self.title + " by " + self.author + ", signed " + self.signature
    end method
end record

method restock[SignedCard](amount%)
    self.copies = self.copies + amount%
end method

let signed = { title: "Dune", author: "Frank Herbert", copies: 1, signature: "F.H." }
print signed.display()
signed.restock(2)
print "signed copies after restock = "; signed.copies
```

`SignedCard`'s effective field list is `Card`'s fields (`title`, `author`, `copies`) followed by its own (`signature`), so its record literal accepts all four. `SignedCard` declares its own `display()`, since `combines` never combines methods — `Card`'s own `display()`/`restock()` only ever apply to a `Card` receiver, so `SignedCard` needs its own `restock()` too (shown here as an external method, just to demonstrate both forms still work the same way once combined fields are involved). `combines` never makes `SignedCard` assignable to or from `Card`: each remains its own exact record type. See the [Methods](../language/methods.md) chapter for the full grammar, including multiple/transitive combination and the compile errors an undeclared source, a combines cycle, or a duplicate field name produce.

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

Methods are syntax for an implicit first parameter. The BASIC backend emits its normal typed parameter/result variables and `GOSUB`; the C backend emits typed calls and temporaries; the JVM backend emits an ordinary `invokestatic` call, the receiver passed as explicit leading arguments — never `invokevirtual`, an interface, or any other object-dispatch mechanism, since a record method's `self` is fully resolved away (into ordinary parameters) before any backend runs. The source-level method syntax and type checks are shared by all three targets.

</div>

Full, real, transpiling source: [`methods.bcl`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/methods.bcl), [`methods.bas`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/methods.bas), [`methods.c`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/methods.c), and [`methods.j`](https://github.com/johnjoeallen/bascal/blob/main/tutorial/methods.j).

[← Functions](functions.md)[Tutorials →](./)


<!-- BEGIN generated tutorial source -->

<details class="source-embed" markdown="1">

<summary><code>tutorial/methods.bcl</code></summary>



```bascal

// Tutorial — Methods
//
// A method is a statically resolved callable with an implicit receiver,
// written in brackets after the method name: `method shout[string]()`
// receives a string. The return type follows the parameter list after
// `:` (or its suffix shorthand, `: $`); omitting it for a scalar receiver
// makes the method return the receiver's own type, falling through to an
// implicit `return self`. Dot calls chain when each result has the next
// receiver's type. Methods transpile to ordinary typed calls for every
// backend -- there is no runtime method object, virtual dispatch, or
// vtable of any kind.
//
// A record type is a valid receiver too, declared either externally (in
// brackets, same as a scalar receiver) or inline, directly inside the
// record itself. `self.field` is then ordinary field access against the
// receiver, and mutating it is visible to the caller once the call
// returns -- the receiver is passed by reference, not by copy.
program methods

require com.bascal.stdlib.ucase

method shout[string]()
    return self$.ucase() + "!"
end method

method surround[string](left$, right$)
    return left$ + self$ + right$
end method

method clamp[integer](low%, high%)
    if self% < low% then
        return low%
    elseif self% > high% then
        return high%
    end if
    return self%
end method

method percent[single](rate!)
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

// -------------------- Record methods --------------------

// Declared inline: the enclosing record supplies the receiver type, so
// there is no `[Card]` bracket here at all.
record Card
    title:  string(40)
    author: string(40)
    copies: int

    method display(): $
        return self.title + " by " + self.author
    end method
end record

// Declared externally: same receiver, same callable identity as an
// inline method -- an external method just lets behavior be attached to
// a record without editing its own declaration.
method restock[Card](amount%)
    self.copies = self.copies + amount%
end method

let card = { title: "Dune", author: "Frank Herbert", copies: 2 }
print card.display()
print "copies on hand = "; card.copies

card.restock(3)
print "copies after restock = "; card.copies

// `combines` is structural field composition only, not inheritance:
// SignedCard's own field list is Card's fields (title, author, copies)
// followed by its own (signature), so its record literal accepts all
// four -- but Card's methods aren't combined in. SignedCard needs its
// own display() (below); calling signed.restock(...) without declaring
// SignedCard's own restock() would be a compile error, since a method
// is only ever visible for the exact record type it was declared for.
record SignedCard combines Card
    signature: string(40)

    method display(): $
        return self.title + " by " + self.author + ", signed " + self.signature
    end method
end record

method restock[SignedCard](amount%)
    self.copies = self.copies + amount%
end method

let signed = { title: "Dune", author: "Frank Herbert", copies: 1, signature: "F.H." }
print signed.display()
signed.restock(2)
print "signed copies after restock = "; signed.copies

end

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/methods.bas</code></summary>



```basic

10 ' BASCAL generated BASIC -- DO NOT EDIT, ANY CHANGES WILL BE OVERWRITTEN BY THE NEXT COMPILE
20 ' Functions are transpiled to global variables, labels, and GOSUB

30 ' Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
40 ' against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
50 ' its own. Declared as a scalar method (see GitHub issue #41 and
60 ' ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
70 ' via ordinary-call syntax resolving to this same declaration.

80 ' Tutorial — Methods
90 '
100 ' A method is a statically resolved callable with an implicit receiver,
110 ' written in brackets after the method name: `method shout[string]()`
120 ' receives a string. The return type follows the parameter list after
130 ' `:` (or its suffix shorthand, `: $`); omitting it for a scalar receiver
140 ' makes the method return the receiver's own type, falling through to an
150 ' implicit `return self`. Dot calls chain when each result has the next
160 ' receiver's type. Methods transpile to ordinary typed calls for every
170 ' backend -- there is no runtime method object, virtual dispatch, or
180 ' vtable of any kind.
190 '
200 ' A record type is a valid receiver too, declared either externally (in
210 ' brackets, same as a scalar receiver) or inline, directly inside the
220 ' record itself. `self.field` is then ordinary field access against the
230 ' receiver, and mutating it is visible to the caller once the call
240 ' returns -- the receiver is passed by reference, not by copy.

250 name$ = "bascal"
260 surroundSelf0$ = name$
270 surroundLeft0$ = "["
280 surroundRight0$ = "]"
290 GOSUB 1360
300 result$ = surroundResult0$
310 PRINT result$
320 shoutSelf0$ = name$
330 GOSUB 1280
340 shoutresult$ = shoutResult0$
350 length% = LEN(LEFT$(name$, 5))
360 PRINT "length = "; length%

370 score% = 125
380 clampSelf0% = score%
390 clampLow0% = 0
400 clampHigh0% = 100
410 GOSUB 1420
420 PRINT "clamped score = "; clampResult0%

430 price! = 80
440 percentSelf0! = price!
450 percentRate0! = 15
460 GOSUB 1570
470 PRINT "discount amount = "; percentResult0!

480 ucaseSelf0$ = LEFT$(name$, 3)
490 GOSUB 1130
500 firstthree$ = ucaseResult0$
510 PRINT "first three = "; firstthree$

520 ' -------------------- Record methods --------------------

530 ' Declared inline: the enclosing record supplies the receiver type, so
540 ' there is no `[Card]` bracket here at all.

550 ' Declared externally: same receiver, same callable identity as an
560 ' inline method -- an external method just lets behavior be attached to
570 ' a record without editing its own declaration.

580 cardtitle$ = "Dune"
590 cardauthor$ = "Frank Herbert"
600 cardcopies& = 2
610 carddisplaySelfTitle0$ = cardtitle$
620 carddisplaySelfAuthor0$ = cardauthor$
630 carddisplaySelfCopies0& = cardcopies&
640 GOSUB 1710
650 cardtitle$ = carddisplaySelfTitle0$
660 cardauthor$ = carddisplaySelfAuthor0$
670 cardcopies& = carddisplaySelfCopies0&
680 PRINT carddisplayResult0$
690 PRINT "copies on hand = "; cardcopies&

700 cardrestockSelfTitle0$ = cardtitle$
710 cardrestockSelfAuthor0$ = cardauthor$
720 cardrestockSelfCopies0& = cardcopies&
730 cardrestockAmount0% = 3
740 GOSUB 1630
750 cardtitle$ = cardrestockSelfTitle0$
760 cardauthor$ = cardrestockSelfAuthor0$
770 cardcopies& = cardrestockSelfCopies0&
780 PRINT "copies after restock = "; cardcopies&

790 ' `combines` is structural field composition only, not inheritance:
800 ' SignedCard's own field list is Card's fields (title, author, copies)
810 ' followed by its own (signature), so its record literal accepts all
820 ' four -- but Card's methods aren't combined in. SignedCard needs its
830 ' own display() (below); calling signed.restock(...) without declaring
840 ' SignedCard's own restock() would be a compile error, since a method
850 ' is only ever visible for the exact record type it was declared for.

860 signedtitle$ = "Dune"
870 signedauthor$ = "Frank Herbert"
880 signedcopies& = 1
890 signedsignature$ = "F.H."
900 signedcarddisplaySelfTitle0$ = signedtitle$
910 signedcarddisplaySelfAuthor0$ = signedauthor$
920 signedcarddisplaySelfCopies0& = signedcopies&
930 signedcarddisplaySelfSignature0$ = signedsignature$
940 GOSUB 1750
950 signedtitle$ = signedcarddisplaySelfTitle0$
960 signedauthor$ = signedcarddisplaySelfAuthor0$
970 signedcopies& = signedcarddisplaySelfCopies0&
980 signedsignature$ = signedcarddisplaySelfSignature0$
990 PRINT signedcarddisplayResult0$
1000 signedcardrestockSelfTitle0$ = signedtitle$
1010 signedcardrestockSelfAuthor0$ = signedauthor$
1020 signedcardrestockSelfCopies0& = signedcopies&
1030 signedcardrestockSelfSignature0$ = signedsignature$
1040 signedcardrestockAmount0% = 2
1050 GOSUB 1670
1060 signedtitle$ = signedcardrestockSelfTitle0$
1070 signedauthor$ = signedcardrestockSelfAuthor0$
1080 signedcopies& = signedcardrestockSelfCopies0&
1090 signedsignature$ = signedcardrestockSelfSignature0$
1100 PRINT "signed copies after restock = "; signedcopies&

1110 END

1120 ' function ucase$()
1130     ucaseOut0$ = ""
1140     FOR ucaseI0% = 1 TO LEN(ucaseSelf0$)
1150         ucaseC0% = ASC(MID$(ucaseSelf0$, ucaseI0%, 1))
1160         IF (ucaseC0% >= 97) = 0 THEN GOTO 1190
1170         IF (ucaseC0% <= 122) = 0 THEN GOTO 1190
1180             ucaseC0% = ucaseC0% - 32
1190         REM END IF
1200         ucaseOut0$ = ucaseOut0$ + CHR$(ucaseC0%)
1210     NEXT ucaseI0%
1220     ucaseResult0$ = ucaseOut0$
1230     RETURN
1240     ucaseResult0$ = ucaseSelf0$
1250     RETURN
1260 ' end function ucase$

1270 ' function shout$()
1280     ucaseSelf0$ = shoutSelf0$
1290     GOSUB 1130
1300     shoutResult0$ = ucaseResult0$ + "!"
1310     RETURN
1320     shoutResult0$ = shoutSelf0$
1330     RETURN
1340 ' end function shout$

1350 ' function surround$(left$, right$)
1360     surroundResult0$ = (surroundLeft0$ + surroundSelf0$) + surroundRight0$
1370     RETURN
1380     surroundResult0$ = surroundSelf0$
1390     RETURN
1400 ' end function surround$

1410 ' function clamp%(low%, high%)
1420     IF (clampSelf0% < clampLow0%) = 0 THEN GOTO 1460
1430         clampResult0% = clampLow0%
1440         RETURN
1450         GOTO 1500
1460         IF (clampSelf0% > clampHigh0%) = 0 THEN GOTO 1490
1470             clampResult0% = clampHigh0%
1480             RETURN
1490         REM END IF
1500     REM END IF
1510     clampResult0% = clampSelf0%
1520     RETURN
1530     clampResult0% = clampSelf0%
1540     RETURN
1550 ' end function clamp%

1560 ' function percent!(rate!)
1570     percentResult0! = (percentSelf0! * percentRate0!) / 100
1580     RETURN
1590     percentResult0! = percentSelf0!
1600     RETURN
1610 ' end function percent!

1620 ' procedure cardrestock(selftitle$, selfauthor$, selfcopies&, amount%)
1630     cardrestockSelfCopies0& = cardrestockSelfCopies0& + cardrestockAmount0%
1640     RETURN
1650 ' end procedure cardrestock

1660 ' procedure signedcardrestock(selftitle$, selfauthor$, selfcopies&, selfsignature$, amount%)
1670     signedcardrestockSelfCopies0& = signedcardrestockSelfCopies0& + signedcardrestockAmount0%
1680     RETURN
1690 ' end procedure signedcardrestock

1700 ' function carddisplay$(selftitle$, selfauthor$, selfcopies&)
1710     carddisplayResult0$ = (carddisplaySelfTitle0$ + " by ") + carddisplaySelfAuthor0$
1720     RETURN
1730 ' end function carddisplay$

1740 ' function signedcarddisplay$(selftitle$, selfauthor$, selfcopies&, selfsignature$)
1750     signedcarddisplayResult0$ = (((signedcarddisplaySelfTitle0$ + " by ") + signedcarddisplaySelfAuthor0$) + ", signed ") + signedcarddisplaySelfSignature0$
1760     RETURN
1770 ' end function signedcarddisplay$

```



</details>

<details class="source-embed" markdown="1">

<summary><code>tutorial/methods.c</code></summary>



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
static int bv_l_cardcopies = 0;
static int bv_l_signedcopies = 0;
static char bv_s_cardauthor[256] = {0};
static char bv_s_cardtitle[256] = {0};
static char bv_s_firstthree[256] = {0};
static char bv_s_name[256] = {0};
static char bv_s_result[256] = {0};
static char bv_s_shoutresult[256] = {0};
static char bv_s_signedauthor[256] = {0};
static char bv_s_signedsignature[256] = {0};
static char bv_s_signedtitle[256] = {0};

void bf_s_ucase_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_shout_s(const char* bv_s_self_in, char* bcc_out);
void bf_s_surround_s(const char* bv_s_self_in, const char* bv_s_left_in, const char* bv_s_right_in, char* bcc_out);
int bf_i_clamp_i(int bv_i_self, int bv_i_low, int bv_i_high);
float bf_f_percent_f(float bv_f_self, float bv_f_rate);
void bf_i_cardrestock(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, int bv_i_amount);
void bf_i_signedcardrestock(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, char* bv_s_selfsignature_in, int bv_i_amount);
void bf_s_carddisplay(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, char* bcc_out);
void bf_s_signedcarddisplay(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, char* bv_s_selfsignature_in, char* bcc_out);

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
    snprintf(bcc_out, 256, "%s", bv_s_self);
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
    snprintf(bcc_out, 256, "%s", bv_s_self);
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
    snprintf(bcc_out, 256, "%s", bv_s_self);
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
    return bv_i_self;
}

float bf_f_percent_f(float bv_f_self, float bv_f_rate) {
    return ((double)(bv_f_self * bv_f_rate) / (double)100);
    return bv_f_self;
}

void bf_i_cardrestock(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, int bv_i_amount) {
    char bv_s_selftitle[256];
    snprintf(bv_s_selftitle, sizeof(bv_s_selftitle), "%s", bv_s_selftitle_in);
    char bv_s_selfauthor[256];
    snprintf(bv_s_selfauthor, sizeof(bv_s_selfauthor), "%s", bv_s_selfauthor_in);
    int bv_l_selfcopies = *bv_l_selfcopies_in;

    bv_l_selfcopies = (bv_l_selfcopies + bv_i_amount);
    snprintf(bv_s_selftitle_in, 256, "%s", bv_s_selftitle);
    snprintf(bv_s_selfauthor_in, 256, "%s", bv_s_selfauthor);
    *bv_l_selfcopies_in = bv_l_selfcopies;
}

void bf_i_signedcardrestock(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, char* bv_s_selfsignature_in, int bv_i_amount) {
    char bv_s_selftitle[256];
    snprintf(bv_s_selftitle, sizeof(bv_s_selftitle), "%s", bv_s_selftitle_in);
    char bv_s_selfauthor[256];
    snprintf(bv_s_selfauthor, sizeof(bv_s_selfauthor), "%s", bv_s_selfauthor_in);
    int bv_l_selfcopies = *bv_l_selfcopies_in;
    char bv_s_selfsignature[256];
    snprintf(bv_s_selfsignature, sizeof(bv_s_selfsignature), "%s", bv_s_selfsignature_in);

    bv_l_selfcopies = (bv_l_selfcopies + bv_i_amount);
    snprintf(bv_s_selftitle_in, 256, "%s", bv_s_selftitle);
    snprintf(bv_s_selfauthor_in, 256, "%s", bv_s_selfauthor);
    *bv_l_selfcopies_in = bv_l_selfcopies;
    snprintf(bv_s_selfsignature_in, 256, "%s", bv_s_selfsignature);
}

void bf_s_carddisplay(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, char* bcc_out) {
    char bv_s_selftitle[256];
    snprintf(bv_s_selftitle, sizeof(bv_s_selftitle), "%s", bv_s_selftitle_in);
    char bv_s_selfauthor[256];
    snprintf(bv_s_selfauthor, sizeof(bv_s_selfauthor), "%s", bv_s_selfauthor_in);
    int bv_l_selfcopies = *bv_l_selfcopies_in;

    char bt_s_6[256];
    snprintf(bt_s_6, sizeof(bt_s_6), "%s%s", bv_s_selftitle, " by ");
    char bt_s_7[256];
    snprintf(bt_s_7, sizeof(bt_s_7), "%s%s", bt_s_6, bv_s_selfauthor);
    snprintf(bcc_out, 256, "%s", bt_s_7);
    snprintf(bv_s_selftitle_in, 256, "%s", bv_s_selftitle);
    snprintf(bv_s_selfauthor_in, 256, "%s", bv_s_selfauthor);
    *bv_l_selfcopies_in = bv_l_selfcopies;
    return;
    snprintf(bv_s_selftitle_in, 256, "%s", bv_s_selftitle);
    snprintf(bv_s_selfauthor_in, 256, "%s", bv_s_selfauthor);
    *bv_l_selfcopies_in = bv_l_selfcopies;
}

void bf_s_signedcarddisplay(char* bv_s_selftitle_in, char* bv_s_selfauthor_in, int* bv_l_selfcopies_in, char* bv_s_selfsignature_in, char* bcc_out) {
    char bv_s_selftitle[256];
    snprintf(bv_s_selftitle, sizeof(bv_s_selftitle), "%s", bv_s_selftitle_in);
    char bv_s_selfauthor[256];
    snprintf(bv_s_selfauthor, sizeof(bv_s_selfauthor), "%s", bv_s_selfauthor_in);
    int bv_l_selfcopies = *bv_l_selfcopies_in;
    char bv_s_selfsignature[256];
    snprintf(bv_s_selfsignature, sizeof(bv_s_selfsignature), "%s", bv_s_selfsignature_in);

    char bt_s_8[256];
    snprintf(bt_s_8, sizeof(bt_s_8), "%s%s", bv_s_selftitle, " by ");
    char bt_s_9[256];
    snprintf(bt_s_9, sizeof(bt_s_9), "%s%s", bt_s_8, bv_s_selfauthor);
    char bt_s_10[256];
    snprintf(bt_s_10, sizeof(bt_s_10), "%s%s", bt_s_9, ", signed ");
    char bt_s_11[256];
    snprintf(bt_s_11, sizeof(bt_s_11), "%s%s", bt_s_10, bv_s_selfsignature);
    snprintf(bcc_out, 256, "%s", bt_s_11);
    snprintf(bv_s_selftitle_in, 256, "%s", bv_s_selftitle);
    snprintf(bv_s_selfauthor_in, 256, "%s", bv_s_selfauthor);
    *bv_l_selfcopies_in = bv_l_selfcopies;
    snprintf(bv_s_selfsignature_in, 256, "%s", bv_s_selfsignature);
    return;
    snprintf(bv_s_selftitle_in, 256, "%s", bv_s_selftitle);
    snprintf(bv_s_selfauthor_in, 256, "%s", bv_s_selfauthor);
    *bv_l_selfcopies_in = bv_l_selfcopies;
    snprintf(bv_s_selfsignature_in, 256, "%s", bv_s_selfsignature);
}

int main(void) {
    // Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    // against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    // its own. Declared as a scalar method (see GitHub issue #41 and
    // ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
    // via ordinary-call syntax resolving to this same declaration.

    // Tutorial — Methods
    //
    // A method is a statically resolved callable with an implicit receiver,
    // written in brackets after the method name: `method shout[string]()`
    // receives a string. The return type follows the parameter list after
    // `:` (or its suffix shorthand, `: $`); omitting it for a scalar receiver
    // makes the method return the receiver's own type, falling through to an
    // implicit `return self`. Dot calls chain when each result has the next
    // receiver's type. Methods transpile to ordinary typed calls for every
    // backend -- there is no runtime method object, virtual dispatch, or
    // vtable of any kind.
    //
    // A record type is a valid receiver too, declared either externally (in
    // brackets, same as a scalar receiver) or inline, directly inside the
    // record itself. `self.field` is then ordinary field access against the
    // receiver, and mutating it is visible to the caller once the call
    // returns -- the receiver is passed by reference, not by copy.






    snprintf(bv_s_name, sizeof(bv_s_name), "%s", "bascal");
    char bt_s_12[256];
    bf_s_surround_s(bv_s_name, "[", "]", bt_s_12);
    snprintf(bv_s_result, sizeof(bv_s_result), "%s", bt_s_12);
    printf("%s\n", bv_s_result);
    char bt_s_13[256];
    bf_s_shout_s(bv_s_name, bt_s_13);
    snprintf(bv_s_shoutresult, sizeof(bv_s_shoutresult), "%s", bt_s_13);
    bv_i_length = ((int)strlen(bcc_mid(bv_s_name, 1, 5)));
    printf("length = %d\n", bv_i_length);

    bv_i_score = 125;
    printf("clamped score = %d\n", bf_i_clamp_i(bv_i_score, 0, 100));

    bv_f_price = 80;
    printf("discount amount = %g\n", bf_f_percent_f(bv_f_price, 15));

    char bt_s_14[256];
    bf_s_ucase_s(bcc_mid(bv_s_name, 1, 3), bt_s_14);
    snprintf(bv_s_firstthree, sizeof(bv_s_firstthree), "%s", bt_s_14);
    printf("first three = %s\n", bv_s_firstthree);

    // -------------------- Record methods --------------------

    // Declared inline: the enclosing record supplies the receiver type, so
    // there is no `[Card]` bracket here at all.

    // Declared externally: same receiver, same callable identity as an
    // inline method -- an external method just lets behavior be attached to
    // a record without editing its own declaration.

    snprintf(bv_s_cardtitle, sizeof(bv_s_cardtitle), "%s", "Dune");
    snprintf(bv_s_cardauthor, sizeof(bv_s_cardauthor), "%s", "Frank Herbert");
    bv_l_cardcopies = 2;
    char bt_s_15[256];
    bf_s_carddisplay(bv_s_cardtitle, bv_s_cardauthor, &bv_l_cardcopies, bt_s_15);
    printf("%s\n", bt_s_15);
    printf("copies on hand = %d\n", bv_l_cardcopies);

    bf_i_cardrestock(bv_s_cardtitle, bv_s_cardauthor, &bv_l_cardcopies, 3);
    printf("copies after restock = %d\n", bv_l_cardcopies);

    // `combines` is structural field composition only, not inheritance:
    // SignedCard's own field list is Card's fields (title, author, copies)
    // followed by its own (signature), so its record literal accepts all
    // four -- but Card's methods aren't combined in. SignedCard needs its
    // own display() (below); calling signed.restock(...) without declaring
    // SignedCard's own restock() would be a compile error, since a method
    // is only ever visible for the exact record type it was declared for.


    snprintf(bv_s_signedtitle, sizeof(bv_s_signedtitle), "%s", "Dune");
    snprintf(bv_s_signedauthor, sizeof(bv_s_signedauthor), "%s", "Frank Herbert");
    bv_l_signedcopies = 1;
    snprintf(bv_s_signedsignature, sizeof(bv_s_signedsignature), "%s", "F.H.");
    char bt_s_16[256];
    bf_s_signedcarddisplay(bv_s_signedtitle, bv_s_signedauthor, &bv_l_signedcopies, bv_s_signedsignature, bt_s_16);
    printf("%s\n", bt_s_16);
    bf_i_signedcardrestock(bv_s_signedtitle, bv_s_signedauthor, &bv_l_signedcopies, bv_s_signedsignature, 2);
    printf("signed copies after restock = %d\n", bv_l_signedcopies);

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

<details class="source-embed" markdown="1">

<summary><code>tutorial/methods.j</code></summary>



```basic

.version 50 0
.class public Methods
.super java/lang/Object

.field public static g1 Ljava/lang/String;
.field public static g2 J
.field public static g4 Ljava/lang/String;
.field public static g5 Ljava/lang/String;
.field public static g6 I
.field public static g7 Ljava/lang/String;
.field public static g8 D
.field public static g10 Ljava/lang/String;
.field public static g11 I
.field public static g12 Ljava/lang/String;
.field public static g13 Ljava/lang/String;
.field public static g14 J
.field public static g16 Ljava/lang/String;
.field public static g17 Ljava/lang/String;
.method public static ucase : (Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 8

    iconst_0
    istore 1
    iconst_0
    istore 2
    ldc ""
    astore 3
    ldc ""
    astore 3
    ldc 1
    istore 2
L_for_0_top:
    iload 2
    aload 0
    invokevirtual java/lang/String/length ()I
    if_icmpgt L_for_0_end
    aload 0
    iload 2
    iconst_1
    isub
    dup
    ldc 1
    iadd
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    iconst_0
    invokevirtual java/lang/String/charAt (I)C
    istore 1
    iload 1
    ldc 97
    invokestatic java/lang/Integer/compare (II)I
    ineg
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_1_else
    iload 1
    ldc 122
    invokestatic java/lang/Integer/compare (II)I
    iconst_1
    isub
    bipush 31
    ishr
    ifeq L_if_1_else
    iload 1
    ldc 32
    isub
    istore 1
L_if_1_else:
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 3
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    iload 1
    i2c
    invokestatic java/lang/String/valueOf (C)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    astore 3
    iload 2
    ldc 1
    iadd
    istore 2
    goto L_for_0_top
L_for_0_end:
    aload 3
    areturn
    aload 0
    areturn
    ldc ""
    areturn
.end method

.method public static shout : (Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 5

    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 0
    invokestatic Methods/ucase (Ljava/lang/String;)Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc "!"
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    areturn
    aload 0
    areturn
    ldc ""
    areturn
.end method

.method public static surround : (Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 7

    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 1
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 0
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 2
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    areturn
    aload 0
    areturn
    ldc ""
    areturn
.end method

.method public static clamp : (III)I
    .limit stack 16
    .limit locals 7

    iload 0
    iload 1
    invokestatic java/lang/Integer/compare (II)I
    bipush 31
    ishr
    ifeq L_if_0_else
    iload 1
    ireturn
    goto L_if_0_end
L_if_0_else:
    iload 0
    iload 2
    invokestatic java/lang/Integer/compare (II)I
    ineg
    bipush 31
    ishr
    ifeq L_if_1_else
    iload 2
    ireturn
L_if_1_else:
L_if_0_end:
    iload 0
    ireturn
    iload 0
    ireturn
    iconst_0
    ireturn
.end method

.method public static percent : (DD)D
    .limit stack 16
    .limit locals 8

    dload 0
    dload 2
    dmul
    ldc 100
    i2d
    ddiv
    dreturn
    dload 0
    dreturn
    dconst_0
    dreturn
.end method

.method public static cardRestock : ([Ljava/lang/String;[Ljava/lang/String;[JI)V
    .limit stack 16
    .limit locals 12

    aload 0
    iconst_0
    aaload
    astore 4
    aload 1
    iconst_0
    aaload
    astore 5
    aload 2
    iconst_0
    laload
    lstore 6
    lload 6
    iload 3
    i2l
    ladd
    lstore 6
    aload 0
    iconst_0
    aload 4
    aastore
    aload 1
    iconst_0
    aload 5
    aastore
    aload 2
    iconst_0
    lload 6
    lastore
    return
.end method

.method public static signedcardRestock : ([Ljava/lang/String;[Ljava/lang/String;[J[Ljava/lang/String;I)V
    .limit stack 16
    .limit locals 14

    aload 0
    iconst_0
    aaload
    astore 5
    aload 1
    iconst_0
    aaload
    astore 6
    aload 2
    iconst_0
    laload
    lstore 7
    aload 3
    iconst_0
    aaload
    astore 9
    lload 7
    iload 4
    i2l
    ladd
    lstore 7
    aload 0
    iconst_0
    aload 5
    aastore
    aload 1
    iconst_0
    aload 6
    aastore
    aload 2
    iconst_0
    lload 7
    lastore
    aload 3
    iconst_0
    aload 9
    aastore
    return
.end method

.method public static cardDisplay : ([Ljava/lang/String;[Ljava/lang/String;[J)Ljava/lang/String;
    .limit stack 16
    .limit locals 11

    aload 0
    iconst_0
    aaload
    astore 3
    aload 1
    iconst_0
    aaload
    astore 4
    aload 2
    iconst_0
    laload
    lstore 5
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 3
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " by "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 4
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    aload 0
    iconst_0
    aload 3
    aastore
    aload 1
    iconst_0
    aload 4
    aastore
    aload 2
    iconst_0
    lload 5
    lastore
    areturn
    aload 0
    iconst_0
    aload 3
    aastore
    aload 1
    iconst_0
    aload 4
    aastore
    aload 2
    iconst_0
    lload 5
    lastore
    ldc ""
    areturn
.end method

.method public static signedcardDisplay : ([Ljava/lang/String;[Ljava/lang/String;[J[Ljava/lang/String;)Ljava/lang/String;
    .limit stack 16
    .limit locals 13

    aload 0
    iconst_0
    aaload
    astore 4
    aload 1
    iconst_0
    aaload
    astore 5
    aload 2
    iconst_0
    laload
    lstore 6
    aload 3
    iconst_0
    aaload
    astore 8
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    new java/lang/StringBuilder
    dup
    invokespecial java/lang/StringBuilder/<init> ()V
    aload 4
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc " by "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 5
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    ldc ", signed "
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    aload 8
    invokevirtual java/lang/StringBuilder/append (Ljava/lang/String;)Ljava/lang/StringBuilder;
    invokevirtual java/lang/StringBuilder/toString ()Ljava/lang/String;
    aload 0
    iconst_0
    aload 4
    aastore
    aload 1
    iconst_0
    aload 5
    aastore
    aload 2
    iconst_0
    lload 6
    lastore
    aload 3
    iconst_0
    aload 8
    aastore
    areturn
    aload 0
    iconst_0
    aload 4
    aastore
    aload 1
    iconst_0
    aload 5
    aastore
    aload 2
    iconst_0
    lload 6
    lastore
    aload 3
    iconst_0
    aload 8
    aastore
    ldc ""
    areturn
.end method

.method public static bccStr : (D)Ljava/lang/String;
    .limit stack 6
    .limit locals 2

    new java/math/BigDecimal
    dup
    dload 0
    invokespecial java/math/BigDecimal/<init> (D)V
    new java/math/MathContext
    dup
    bipush 6
    invokespecial java/math/MathContext/<init> (I)V
    invokevirtual java/math/BigDecimal/round (Ljava/math/MathContext;)Ljava/math/BigDecimal;
    invokevirtual java/math/BigDecimal/stripTrailingZeros ()Ljava/math/BigDecimal;
    invokevirtual java/math/BigDecimal/toPlainString ()Ljava/lang/String;
    areturn
.end method

.method public static main : ([Ljava/lang/String;)V
    .limit stack 16
    .limit locals 22

    ldc ""
    putstatic Methods/g1 Ljava/lang/String;
    lconst_0
    putstatic Methods/g2 J
    ldc ""
    putstatic Methods/g4 Ljava/lang/String;
    ldc ""
    putstatic Methods/g5 Ljava/lang/String;
    iconst_0
    putstatic Methods/g6 I
    ldc ""
    putstatic Methods/g7 Ljava/lang/String;
    dconst_0
    putstatic Methods/g8 D
    ldc ""
    putstatic Methods/g10 Ljava/lang/String;
    iconst_0
    putstatic Methods/g11 I
    ldc ""
    putstatic Methods/g12 Ljava/lang/String;
    ldc ""
    putstatic Methods/g13 Ljava/lang/String;
    lconst_0
    putstatic Methods/g14 J
    ldc ""
    putstatic Methods/g16 Ljava/lang/String;
    ldc ""
    putstatic Methods/g17 Ljava/lang/String;
    ; Upper-cases self$. Not a real MBASIC/BASCOM 2.00 builtin -- verified
    ; against a real IBM BASIC Compiler 2.00 under dosbox-x -- so BASCAL ships
    ; its own. Declared as a scalar method (see GitHub issue #41 and
    ; ltrim.bcl's own doc comment for the reasoning) -- ucase$(s$) still works
    ; via ordinary-call syntax resolving to this same declaration.

    ; Tutorial — Methods
    ;
    ; A method is a statically resolved callable with an implicit receiver,
    ; written in brackets after the method name: `method shout[string]()`
    ; receives a string. The return type follows the parameter list after
    ; `:` (or its suffix shorthand, `: $`); omitting it for a scalar receiver
    ; makes the method return the receiver's own type, falling through to an
    ; implicit `return self`. Dot calls chain when each result has the next
    ; receiver's type. Methods transpile to ordinary typed calls for every
    ; backend -- there is no runtime method object, virtual dispatch, or
    ; vtable of any kind.
    ;
    ; A record type is a valid receiver too, declared either externally (in
    ; brackets, same as a scalar receiver) or inline, directly inside the
    ; record itself. `self.field` is then ordinary field access against the
    ; receiver, and mutating it is visible to the caller once the call
    ; returns -- the receiver is passed by reference, not by copy.






    ldc "bascal"
    putstatic Methods/g7 Ljava/lang/String;
    getstatic Methods/g7 Ljava/lang/String;
    ldc "["
    ldc "]"
    invokestatic Methods/surround (Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;
    putstatic Methods/g10 Ljava/lang/String;
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g10 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic Methods/g7 Ljava/lang/String;
    invokestatic Methods/shout (Ljava/lang/String;)Ljava/lang/String;
    putstatic Methods/g12 Ljava/lang/String;
    getstatic Methods/g7 Ljava/lang/String;
    iconst_0
    ldc 5
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    invokevirtual java/lang/String/length ()I
    putstatic Methods/g6 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "length = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g6 I
    invokevirtual java/io/PrintStream/println (I)V

    ldc 125
    putstatic Methods/g11 I
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "clamped score = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g11 I
    ldc 0
    ldc 100
    invokestatic Methods/clamp (III)I
    invokevirtual java/io/PrintStream/println (I)V

    ldc 80
    i2d
    putstatic Methods/g8 D
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "discount amount = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g8 D
    ldc 15
    i2d
    invokestatic Methods/percent (DD)D
    invokestatic Methods/bccStr (D)Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    getstatic Methods/g7 Ljava/lang/String;
    iconst_0
    ldc 3
    invokevirtual java/lang/String/substring (II)Ljava/lang/String;
    invokestatic Methods/ucase (Ljava/lang/String;)Ljava/lang/String;
    putstatic Methods/g5 Ljava/lang/String;
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "first three = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g5 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V

    ; -------------------- Record methods --------------------

    ; Declared inline: the enclosing record supplies the receiver type, so
    ; there is no `[Card]` bracket here at all.

    ; Declared externally: same receiver, same callable identity as an
    ; inline method -- an external method just lets behavior be attached to
    ; a record without editing its own declaration.

    ldc "Dune"
    putstatic Methods/g4 Ljava/lang/String;
    ldc "Frank Herbert"
    putstatic Methods/g1 Ljava/lang/String;
    ldc 2
    i2l
    putstatic Methods/g2 J
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iconst_1
    anewarray java/lang/String
    dup
    iconst_0
    getstatic Methods/g4 Ljava/lang/String;
    aastore
    astore 18
    aload 18
    iconst_1
    anewarray java/lang/String
    dup
    iconst_0
    getstatic Methods/g1 Ljava/lang/String;
    aastore
    astore 19
    aload 19
    iconst_1
    newarray long
    dup
    iconst_0
    getstatic Methods/g2 J
    lastore
    astore 20
    aload 20
    invokestatic Methods/cardDisplay ([Ljava/lang/String;[Ljava/lang/String;[J)Ljava/lang/String;
    aload 18
    iconst_0
    aaload
    putstatic Methods/g4 Ljava/lang/String;
    aload 19
    iconst_0
    aaload
    putstatic Methods/g1 Ljava/lang/String;
    aload 20
    iconst_0
    laload
    putstatic Methods/g2 J
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "copies on hand = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g2 J
    invokevirtual java/io/PrintStream/println (J)V

    iconst_1
    anewarray java/lang/String
    dup
    iconst_0
    getstatic Methods/g4 Ljava/lang/String;
    aastore
    astore 18
    aload 18
    iconst_1
    anewarray java/lang/String
    dup
    iconst_0
    getstatic Methods/g1 Ljava/lang/String;
    aastore
    astore 19
    aload 19
    iconst_1
    newarray long
    dup
    iconst_0
    getstatic Methods/g2 J
    lastore
    astore 20
    aload 20
    ldc 3
    invokestatic Methods/cardRestock ([Ljava/lang/String;[Ljava/lang/String;[JI)V
    aload 18
    iconst_0
    aaload
    putstatic Methods/g4 Ljava/lang/String;
    aload 19
    iconst_0
    aaload
    putstatic Methods/g1 Ljava/lang/String;
    aload 20
    iconst_0
    laload
    putstatic Methods/g2 J
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "copies after restock = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g2 J
    invokevirtual java/io/PrintStream/println (J)V

    ; `combines` is structural field composition only, not inheritance:
    ; SignedCard's own field list is Card's fields (title, author, copies)
    ; followed by its own (signature), so its record literal accepts all
    ; four -- but Card's methods aren't combined in. SignedCard needs its
    ; own display() (below); calling signed.restock(...) without declaring
    ; SignedCard's own restock() would be a compile error, since a method
    ; is only ever visible for the exact record type it was declared for.


    ldc "Dune"
    putstatic Methods/g17 Ljava/lang/String;
    ldc "Frank Herbert"
    putstatic Methods/g13 Ljava/lang/String;
    ldc 1
    i2l
    putstatic Methods/g14 J
    ldc "F.H."
    putstatic Methods/g16 Ljava/lang/String;
    getstatic java/lang/System/out Ljava/io/PrintStream;
    iconst_1
    anewarray java/lang/String
    dup
    iconst_0
    getstatic Methods/g17 Ljava/lang/String;
    aastore
    astore 18
    aload 18
    iconst_1
    anewarray java/lang/String
    dup
    iconst_0
    getstatic Methods/g13 Ljava/lang/String;
    aastore
    astore 19
    aload 19
    iconst_1
    newarray long
    dup
    iconst_0
    getstatic Methods/g14 J
    lastore
    astore 20
    aload 20
    iconst_1
    anewarray java/lang/String
    dup
    iconst_0
    getstatic Methods/g16 Ljava/lang/String;
    aastore
    astore 21
    aload 21
    invokestatic Methods/signedcardDisplay ([Ljava/lang/String;[Ljava/lang/String;[J[Ljava/lang/String;)Ljava/lang/String;
    aload 18
    iconst_0
    aaload
    putstatic Methods/g17 Ljava/lang/String;
    aload 19
    iconst_0
    aaload
    putstatic Methods/g13 Ljava/lang/String;
    aload 20
    iconst_0
    laload
    putstatic Methods/g14 J
    aload 21
    iconst_0
    aaload
    putstatic Methods/g16 Ljava/lang/String;
    invokevirtual java/io/PrintStream/println (Ljava/lang/String;)V
    iconst_1
    anewarray java/lang/String
    dup
    iconst_0
    getstatic Methods/g17 Ljava/lang/String;
    aastore
    astore 18
    aload 18
    iconst_1
    anewarray java/lang/String
    dup
    iconst_0
    getstatic Methods/g13 Ljava/lang/String;
    aastore
    astore 19
    aload 19
    iconst_1
    newarray long
    dup
    iconst_0
    getstatic Methods/g14 J
    lastore
    astore 20
    aload 20
    iconst_1
    anewarray java/lang/String
    dup
    iconst_0
    getstatic Methods/g16 Ljava/lang/String;
    aastore
    astore 21
    aload 21
    ldc 2
    invokestatic Methods/signedcardRestock ([Ljava/lang/String;[Ljava/lang/String;[J[Ljava/lang/String;I)V
    aload 18
    iconst_0
    aaload
    putstatic Methods/g17 Ljava/lang/String;
    aload 19
    iconst_0
    aaload
    putstatic Methods/g13 Ljava/lang/String;
    aload 20
    iconst_0
    laload
    putstatic Methods/g14 J
    aload 21
    iconst_0
    aaload
    putstatic Methods/g16 Ljava/lang/String;
    getstatic java/lang/System/out Ljava/io/PrintStream;
    ldc "signed copies after restock = "
    invokevirtual java/io/PrintStream/print (Ljava/lang/String;)V
    getstatic java/lang/System/out Ljava/io/PrintStream;
    getstatic Methods/g14 J
    invokevirtual java/io/PrintStream/println (J)V

    return
.end method

```



</details>

<!-- END generated tutorial source -->
