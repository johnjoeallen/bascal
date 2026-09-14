# BASCAL syntax highlighting

A TextMate grammar for BASCAL (`.bcl`) source files, packaged as a VS Code
extension. The same grammar works directly in JetBrains IDEs (IntelliJ,
PyCharm, WebStorm, ...) via their built-in TextMate bundle support -- no
separate bundle needed.

Covers: `'`, `//`, and `/* */` comments; double-quoted strings; integer,
float, hex (`&H`), and octal (`&O`) numeric literals; every BASCAL
statement keyword (`program`, `procedure`, `function`, `if`/`then`/
`elseif`/`else`/`end if`, `for`/`to`/`step`/`downto`/`next`, `while`/
`end while`, `do`/`loop`/`until`, `select case`, `try`/`catch`/`finally`,
`record`/`extends`/`combines`, `global`/`dim`/`const`, `require`/`import`,
file I/O, `data`/`read`/`restore`, ...); the `%`/`&`/`!`/`#`/`$` type
suffixes; word operators (`and`/`or`/`not`/`xor`/`mod`); and the built-in
function set from `bcc`'s own `BASIC_BUILTINS` table.

## VS Code

**Local install (no packaging needed):**

1. Copy or symlink this directory into your VS Code extensions folder:
   - Linux/macOS: `~/.vscode/extensions/bascal-language-0.1.0`
   - Windows: `%USERPROFILE%\.vscode\extensions\bascal-language-0.1.0`
2. Restart VS Code (or run "Developer: Reload Window"). Opening any `.bcl`
   file now highlights as BASCAL.

**Packaged install:** with [`vsce`](https://github.com/microsoft/vscode-vsce)
installed (`npm install -g @vscode/vsce`), run `vsce package` in this
directory to produce a `.vsix`, then in VS Code use Extensions ->
"..." menu -> "Install from VSIX...".

## IntelliJ / JetBrains IDEs

JetBrains' built-in TextMate bundle support reads a VS Code extension
folder directly (it understands `package.json`'s `contributes.grammars`
the same way VS Code does), so no separate bundle format is needed:

1. Settings/Preferences -> Editor -> TextMate Bundles.
2. Click **+** and select this directory (`editors/vscode-bascal`).
3. Opening a `.bcl` file now highlights using this grammar.

## Regenerating from a language change

This grammar is hand-maintained against `src/lexer.rs`/`src/parser.rs`'s
own keyword and builtin-function tables (`classify_keyword`,
`check_keyword`/`expect_keyword` call sites, and `codegen_basic.rs`'s
`BASIC_BUILTINS`). If a language change adds/renames/removes a keyword or
builtin, update `syntaxes/bascal.tmLanguage.json`'s corresponding
`repository` pattern to match.
