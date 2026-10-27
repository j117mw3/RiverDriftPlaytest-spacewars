# Build and isolated tests

Use 64-bit Windows, Python 3, the Rust `x86_64-pc-windows-msvc` toolchain with bundled `lld-link.exe`, and the .NET Framework 4 x64 C# compiler. The release binary was built with Rust 1.94.0. No game files are required for the isolated tests.

```powershell
python native/build.py
python native/test.py
Get-FileHash native/build/winmm.dll -Algorithm SHA256
```

The builder reads the local Windows System32 `winmm.dll` export table and generates forwarding stubs. The probe runs synthetic executables, checks the game-name guard and configuration, export resolution, concurrent initialization, timer and pointer forwarding, and calling conventions. It does not launch the game or verify Steam, overlay, or co-op.

Release DLL SHA-256: `477b757f902ab9e94a42b049b51807e00d9e252de8fc5ec2e00b70bd28a68e01`. Windows export-table or toolchain differences may change a rebuild's hash.
