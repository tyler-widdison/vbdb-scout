Place bundled ffmpeg binaries here for packaging.

Expected filenames by platform:
- Windows: `ffmpeg.exe`
- macOS: `ffmpeg`
- Linux: `ffmpeg`

At runtime, export will try:
1. `ffmpeg` from system PATH
2. bundled resource paths under app resources

If you ship platform installers, include the platform-specific binary in this folder before build.
