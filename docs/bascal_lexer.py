"""Pygments lexer for BASCAL source files."""

import re

from pygments.lexer import RegexLexer
from pygments.token import Comment, Keyword, Literal, Name, Text

__all__ = ["BascalLexer"]


class BascalLexer(RegexLexer):
    """Highlight BASCAL keywords, suffixes, comments, strings and numbers."""

    name = "BASCAL"
    aliases = ["bascal", "bcl"]
    filenames = ["*.bcl"]
    flags = re.IGNORECASE | re.MULTILINE

    _keywords = (
        "program library shared require import as dim declare const global let "
        "if then elseif else end for to downto step next while wend do loop until exit "
        "select case is try catch finally function procedure method return record field "
        "file open close get put lset rset seek swap goto gosub on error throw resume "
        "print lprint input write data read restore using line random output append binary "
        "randomize and or not xor mod true false option base erase clear width out poke "
        "locate color cls beep stop system kill name chain run date time timer inkey err erl "
        "len asc chr left right mid instr str val eof sqr abs int fix sgn cint clng csng cdbl "
        "sin cos tan atn log exp rnd sizeof lbound ubound tab spc mki mkl mks mkd cvi cvl cvs cvd "
        "int16 int32 float32 float64 string"
    )

    tokens = {
        "root": [
            (r"'.*$|//.*$|/\*[\s\S]*?\*/", Comment),
            (r'"[^"\n]*"', Literal.String),
            (r"&[Hh][0-9a-f]+|\b(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?[%$!#&]?", Literal.Number),
            (r"\brem\b.*$", Comment),
            (r"[A-Za-z_][A-Za-z0-9_]*[%$!#&]", Name.Variable),
            (rf"\b(?:{_keywords.replace(' ', '|')})\b", Keyword),
            (r"[A-Za-z_][A-Za-z0-9_]*[%$!#&]?", Name),
            (r"\s+", Text.Whitespace),
            (r".", Text),
        ]
    }
