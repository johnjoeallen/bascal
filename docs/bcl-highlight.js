// Lightweight syntax highlighter for BASCAL (.bcl) and the classic BASIC
// it compiles to. No dependencies, no build step: walks every `pre > code`
// block on the page at load time and wraps tokens in <span class="tok-*">
// so style.css can color them. Keyword matching is case-insensitive since
// BASCAL source is conventionally lowercase and generated BASIC is
// conventionally uppercase.
(function () {
  "use strict";

  var KEYWORDS = [
    "program", "library", "shared", "require", "import", "as",
    "dim", "const", "global", "let",
    "if", "then", "elseif", "else", "end",
    "for", "to", "downto", "step", "next",
    "while", "wend", "do", "loop", "until", "exit",
    "select", "case", "is", "try", "catch", "finally",
    "function", "procedure", "method", "return",
    "record", "field", "file", "open", "close", "get", "put",
    "lset", "rset", "seek", "swap",
    "goto", "gosub", "on", "error", "throw", "resume",
    "print", "lprint", "input", "write", "data", "read", "restore", "using",
    "line", "random", "output", "append", "binary", "randomize",
    "and", "or", "not", "xor", "mod",
    "true", "false",
    "option", "base", "erase", "clear", "width", "out", "poke",
    "locate", "color", "cls", "beep", "stop", "system", "kill", "name", "chain", "run",
    "date", "time", "timer", "inkey", "err", "erl",
    "len", "asc", "chr", "left", "right", "mid", "instr", "str", "val", "eof",
    "sqr", "abs", "int", "fix", "sgn", "cint", "clng", "csng", "cdbl",
    "sin", "cos", "tan", "atn", "log", "exp", "rnd", "sizeof", "lbound", "ubound",
    "tab", "spc", "mki", "mkl", "mks", "mkd", "cvi", "cvl", "cvs", "cvd",
    "rem",
    "int16", "int32", "float32", "float64", "string",
  ];
  var KEYWORD_SET = {};
  for (var i = 0; i < KEYWORDS.length; i++) KEYWORD_SET[KEYWORDS[i]] = true;

  function isDigit(c) { return c >= "0" && c <= "9"; }
  function isIdentStart(c) { return /[A-Za-z_]/.test(c); }
  function isIdentChar(c) { return /[A-Za-z0-9_]/.test(c); }
  function isSuffix(c) { return c === "%" || c === "$" || c === "!" || c === "#" || c === "&"; }

  // Tokenize BASCAL/BASIC source into {type, text} tokens.
  // type is one of: "com" (comment), "str" (string), "num" (number),
  // "kw" (keyword), or null (everything else, rendered unstyled).
  function tokenize(src) {
    var tokens = [];
    var n = src.length;
    var i = 0;
    while (i < n) {
      var ch = src[i];

      // Line comments: ' and //
      if (ch === "'" || (ch === "/" && src[i + 1] === "/")) {
        var eol = src.indexOf("\n", i);
        if (eol === -1) eol = n;
        tokens.push({ type: "com", text: src.slice(i, eol) });
        i = eol;
        continue;
      }

      // Block comments: /* ... */
      if (ch === "/" && src[i + 1] === "*") {
        var close = src.indexOf("*/", i + 2);
        var end = close === -1 ? n : close + 2;
        tokens.push({ type: "com", text: src.slice(i, end) });
        i = end;
        continue;
      }

      // String literals: "..." (no escape handling -- BASIC strings don't
      // support backslash escapes; stop at the next quote or end of line).
      if (ch === '"') {
        var j = i + 1;
        while (j < n && src[j] !== '"' && src[j] !== "\n") j++;
        if (j < n && src[j] === '"') j++;
        tokens.push({ type: "str", text: src.slice(i, j) });
        i = j;
        continue;
      }

      // Hex literals: &H1A2F
      if (ch === "&" && (src[i + 1] === "H" || src[i + 1] === "h")) {
        var k = i + 2;
        while (k < n && /[0-9A-Fa-f]/.test(src[k])) k++;
        tokens.push({ type: "num", text: src.slice(i, k) });
        i = k;
        continue;
      }

      // Numbers: 123, 3.14, 1e-9, with an optional type suffix.
      if (isDigit(ch) || (ch === "." && isDigit(src[i + 1] || ""))) {
        var m = i;
        while (m < n && isDigit(src[m])) m++;
        if (src[m] === ".") {
          m++;
          while (m < n && isDigit(src[m])) m++;
        }
        if (src[m] === "e" || src[m] === "E") {
          var look = m + 1;
          if (src[look] === "+" || src[look] === "-") look++;
          if (isDigit(src[look] || "")) {
            m = look;
            while (m < n && isDigit(src[m])) m++;
          }
        }
        if (isSuffix(src[m] || "")) m++;
        tokens.push({ type: "num", text: src.slice(i, m) });
        i = m;
        continue;
      }

      // Identifiers and keywords (REM swallows the rest of the line).
      if (isIdentStart(ch)) {
        var p = i + 1;
        while (p < n && isIdentChar(src[p])) p++;
        if (isSuffix(src[p] || "")) p++;
        var word = src.slice(i, p);
        var bare = word.replace(/[%$!#&]$/, "").toLowerCase();
        if (bare === "rem") {
          var remEol = src.indexOf("\n", p);
          if (remEol === -1) remEol = n;
          tokens.push({ type: "com", text: src.slice(i, remEol) });
          i = remEol;
          continue;
        }
        tokens.push({ type: KEYWORD_SET[bare] ? "kw" : null, text: word });
        i = p;
        continue;
      }

      // Everything else: run of punctuation/whitespace, passed through
      // unstyled until the next token boundary.
      var q = i + 1;
      while (q < n) {
        var c = src[q];
        if (
          c === "'" ||
          c === '"' ||
          (c === "/" && (src[q + 1] === "/" || src[q + 1] === "*")) ||
          isDigit(c) ||
          isIdentStart(c) ||
          (c === "&" && (src[q + 1] === "H" || src[q + 1] === "h"))
        ) {
          break;
        }
        q++;
      }
      tokens.push({ type: null, text: src.slice(i, q) });
      i = q;
    }
    return tokens;
  }

  function escapeHtml(s) {
    return s
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  function highlight(src) {
    var tokens = tokenize(src);
    var out = "";
    for (var i = 0; i < tokens.length; i++) {
      var tok = tokens[i];
      var text = escapeHtml(tok.text);
      out += tok.type ? '<span class="tok-' + tok.type + '">' + text + "</span>" : text;
    }
    return out;
  }

  function run() {
    var blocks = document.querySelectorAll("pre > code");
    for (var i = 0; i < blocks.length; i++) {
      var block = blocks[i];
      if (block.hasAttribute("data-no-highlight")) continue;
      block.innerHTML = highlight(block.textContent);
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", run);
  } else {
    run();
  }
})();
