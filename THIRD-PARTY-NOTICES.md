# Third-party notices

The compiled DLL statically links portions of the Rust standard library. Its
licenses and attribution, including notices for other components, are provided
in [third-party/rust/COPYRIGHT-library.html](third-party/rust/COPYRIGHT-library.html)
and the adjacent `licenses` directory, copied from Rust 1.94.0's distribution.
The Rust library is available under MIT or Apache-2.0 terms; the copyright file
identifies additional notices where applicable.

The build script reads export names and ordinals from the builder's Windows
System32 `winmm.dll` to generate forwarding stubs. No Windows binary, game
binary, third-party online-fix DLL, or game data is distributed here. Windows,
Steam, Spacewar, and River Drift are names of their respective owners.
