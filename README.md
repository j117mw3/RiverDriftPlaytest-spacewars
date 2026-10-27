# River Drift Playtest — Spacewar candidate

**Not yet tested in-game.** Startup, Spacewar presence, Steam overlay, invitations and multiplayer are unverified. Isolated native proxy tests have passed; they do not establish in-game functionality.

**[Download v0.1.0](https://github.com/j117mw3/RiverDrift-Playtest/releases/tag/v0.1.0)**

## Requirements

- Windows x64 River Drift Playtest Steam build **25594302**. The original shipping executable SHA-256 is `c30ab901103bcdc9f0a5f6b5c5048c11bf361ad74101c581ffa18a60a68a409f`.
- Steam running, Steam overlay enabled, and distinct Steam sessions. Participants need the same game build and package revision.

## Install and play

1. Close the game. Back up any existing files at the four package paths outside the game folder.
2. Extract `RiverDrift-Spacewar-v0.1.0.zip` into the folder containing the original `RiverDrift.exe`, preserving the nested folders.
3. Launch the original `RiverDrift.exe`. No installer, separate launcher or launch arguments are required.
4. Check Steam shows Spacewar and Shift+Tab opens the overlay. Create a lobby and try an invitation with another participant.

Steam supplies the overlay; this package does not replace it. The Steam Play button is outside the candidate's target. Do not combine the package with another `winmm.dll` mod.

## Troubleshooting and removal

The loader log is at `RiverDrift/Binaries/Win64/RiverDrift-Spacewar-Loader.log`. It reports startup settings and Windows multimedia forwarding, not a successful Steam session. The game may write diagnostics under `%LOCALAPPDATA%/RiverDrift_57_Multiplayer/Saved`. Stop at a startup or Steam initialization error before testing co-op.

To uninstall, close the game, remove the four package files after checking their hashes against `manifest.json`, and restore any files backed up before installation. Keep unrelated files and saves. The separate diagnostic log may also be removed.

## Build and notices

See [BUILD.md](BUILD.md), [LICENSE](LICENSE), and [third-party notices](THIRD-PARTY-NOTICES.md). This unofficial project is not affiliated with the game's developer or Valve.
