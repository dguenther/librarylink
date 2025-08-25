# librarylink

A Windows utility that allows you to add PC Xbox (UWP) games, like via Game Pass, to Steam as non-Steam games. 

> [!NOTE]
> Steam overlay and Steam controller input are not supported. librarylink does not allow for running UWP apps on non-Windows platforms.

This app was developed as a toy project, so I don't intend to provide ongoing support. If you're interested in better-supported projects, check out [UWPHook](https://github.com/BrianLima/UWPHook) and [GlosSI](https://github.com/Alia5/GlosSI).

## Installation

### From Source

1. Ensure you have [Rust](https://rustup.rs/) installed
2. Clone this repository:
   ```bash
   git clone https://github.com/dguenther/librarylink.git
   cd librarylink
   ```
3. Build the project:
   ```bash
   cargo build --release
   ```
4. The executable will be available at `target/release/librarylink.exe`

## Usage

### Launch UWP App
```bash
librarylink uwp-launch <AUMID>
```
Launches a UWP app using its AUMID (Application User Model ID) and monitors the process.

### List Apps
```bash
librarylink list-apps
```
Lists installed apps with AUMIDs (likely UWP/Store apps) in a table format.

You can also search for specific apps:
```bash
librarylink list-apps --search forza
```

### Adding to Steam

1. Build or download `librarylink.exe`
2. Use `librarylink list-apps` to find the AUMID of your game
3. In Steam, go to "Library" → "Add a Game" → "Add a Non-Steam Game..."
4. Click "Browse" and select `librarylink.exe`
5. In the launch options, add: `uwp-launch <AUMID>`
6. Rename the entry to match your game
7. (Optional) Add custom artwork and configure as needed

**Example Steam Launch Options:**
```
uwp-launch Microsoft.624F8B84B80_8wekyb3d8bbwe!Forzahorizon5
```
