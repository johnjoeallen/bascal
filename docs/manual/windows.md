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
- **BASIC target (FreeBASIC):** install FreeBASIC 1.10 or newer and add the
  folder containing `fbc.exe` to `PATH`:
  <https://www.freebasic.net/get/>.
- **BASIC target (real BASCOM):** BASCAL's `.bas` output is intended to be
  compatible with IBM Personal Computer BASIC Compiler 2.00 (BASCOM). BASCOM
  is a copyrighted DOS program, so it is not bundled; install or supply your
  own copy and run it in DOSBox-X. On Windows, install DOSBox-X, mount the
  folder containing `bcc.exe` and BASCOM, then run `bcc source.bcl` and invoke
  BASCOM on the generated `.bas`. The same workflow works on Linux and macOS
  with DOSBox-X; see the [BASCOM fixture instructions](https://github.com/johnjoeallen/bascal/blob/main/test-fixtures/README.md)
  for the verified compiler setup and conformance command.
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

## BASCOM on Linux and macOS

Install DOSBox-X from its [download and installation
guide](https://dosbox-x.com/wiki/Download-and-Installation), then place your
legally obtained BASCOM files in a DOS directory mounted by DOSBox-X. Generate
classic BASIC with:

```sh
bcc program.bcl --target basic
```

Run BASCOM inside DOSBox-X against the resulting `program.bas`. BASCAL's
optional real-BASCOM conformance tests use the same arrangement; from the
repository root, `scripts/fetch-ibm-basic-compiler.sh` prepares a local,
non-redistributed BASCOM fixture for `cargo test`.
