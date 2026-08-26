# Windows setup

The Windows release ZIP contains `bcc.exe`, the standard library, tutorials,
the project logo, and HTML copies of the README and introduction. Extract the
ZIP to a folder you control; it has no extra wrapper directory, so the files
are immediately visible in Explorer.

## Toolchains

`bcc.exe` can always transpile BASCAL source to `.bas`, `.c`, or JVM assembly
without another compiler. To compile and run the generated output, install the
toolchain for the target you need:

- **C target:** install a MinGW-w64 GCC distribution and add its `bin` folder
  to `PATH`. WinLibs provides standalone builds:
  <https://winlibs.com/>.
- **BASIC target:** install FreeBASIC 1.10 or newer and add the folder
  containing `fbc.exe` to `PATH`:
  <https://www.freebasic.net/get/>
- **JVM target:** install a Java runtime (Java 8 or newer is recommended) and
  Krakatau's `krak2` assembler. Set `BASCAL_KRAK2` to the `krak2` executable,
  or place it on `PATH`.

Check each installation from PowerShell:

```powershell
bcc.exe --help
gcc --version
fbc --version
java -version
krak2 --help
```

Only install the tools for the backend you plan to use. The compiler itself
does not download toolchains or modify `PATH`.
