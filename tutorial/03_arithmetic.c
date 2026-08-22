#include <stdio.h>
#include <math.h>

int main(void) {
    int bv_i_a = 0;
    int bv_i_b = 0;
    int bv_i_n = 0;
    int bv_i_x = 0;

    // Tutorial 3 — Operators and Expressions
    //
    // Arithmetic:   +  -  *  /  \  MOD  ^
    // Comparison:   =  <>  <  <=  >  >=   (result: -1 true, 0 false)
    // Logical:      AND  OR  NOT  XOR  (bitwise — see note below)
    // String:       + concatenates strings
    //
    // Precedence (highest first):
    // ^                 exponentiation (right-associative)
    // unary -           negation
    // * /               multiply / divide
    // \                 integer (floor) division
    // MOD               modulus (remainder)
    // + -               add / subtract
    // = <> < <= > >=    comparison
    // NOT               bitwise NOT
    // AND               bitwise AND
    // OR                bitwise OR
    // XOR               bitwise XOR
    //
    // IMPORTANT: NOT is bitwise, so NOT 1 = -2, not 0.
    // Test for false with (expr) = 0, not NOT expr.

    // Arithmetic — mix labels and numbers with ;
    bv_i_a = 17;
    bv_i_b = 5;
    printf("%d+ %d=%d\n", bv_i_a, bv_i_b, (bv_i_a + bv_i_b));
    printf("%d- %d=%d\n", bv_i_a, bv_i_b, (bv_i_a - bv_i_b));
    printf("%d* %d=%d\n", bv_i_a, bv_i_b, (bv_i_a * bv_i_b));
    printf("%d/ %d=%g\n", bv_i_a, bv_i_b, ((double)bv_i_a / (double)bv_i_b));

    // Integer division and MOD
    printf("%d\\ %d=%d\n", bv_i_a, bv_i_b, ((int)((long)round((double)bv_i_a) / (long)round((double)bv_i_b))));
    printf("%dMOD %d=%d\n", bv_i_a, bv_i_b, ((int)((long)round((double)bv_i_a) % (long)round((double)bv_i_b))));

    // Exponentiation — right-associative
    printf("2 ^ 8 =%g\n", pow((double)2, (double)8));
    printf("2 ^ 3 ^ 2 =%g\n", pow((double)2, (double)pow((double)3, (double)2)));

    // Precedence
    printf("%d (expect 14 — * before +)\n", (2 + (3 * 4)));
    printf("%d (expect 20 — parens first)\n", ((2 + 3) * 4));

    // Comparison — -1 means true, 0 means false
    printf("%d (expect -1)\n", (-(10 > 3)));
    printf("%d (expect  0)\n", (-(10 < 3)));
    printf("%d (expect -1)\n", (-(7 == 7)));
    printf("%d (expect -1)\n", (-(7 != 8)));

    // Logical — AND, OR, XOR are bitwise but work correctly with 0/-1 values
    bv_i_x = 7;
    if (((int)((long)round((double)(-(bv_i_x > 0))) & (long)round((double)(-(bv_i_x < 10)))))) {
        printf("%dis in 1..9\n", bv_i_x);
    }
    printf("%d (expect 5 — 110 XOR 011 = 101)\n", ((int)((long)round((double)6) ^ (long)round((double)3))));

    // String concatenation
    char bt_s_0[256];
    snprintf(bt_s_0, sizeof(bt_s_0), "%s%s", "Hello", ", ");
    char bt_s_1[256];
    snprintf(bt_s_1, sizeof(bt_s_1), "%s%s", bt_s_0, "World");
    char bt_s_2[256];
    snprintf(bt_s_2, sizeof(bt_s_2), "%s%s", bt_s_1, "!");
    printf("%s\n", bt_s_2);

    // Unary negation
    bv_i_n = 42;
    printf("%d\n", -(bv_i_n));

    return 0;
}
