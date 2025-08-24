# librarylink

A Windows utility that allows you to add Xbox Game Pass (UWP) games to Steam as non-Steam games by launching and monitoring UWP applications.

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

### Command Line Interface

```bash
librarylink <command> [arguments]
```

### Available Commands

#### List Processes
```bash
librarylink processes
```
Lists all running Win32 processes with their Process IDs and executable paths.

#### Launch UWP App
```bash
librarylink uwp-launch <AUMID>
```
Launches a UWP application using its Application User Model ID and monitors the process.

**Example:**
```bash
librarylink uwp-launch Microsoft.WindowsCalculator_8wekyb3d8bbwe!App
```

### Finding Game AUMIDs

To find the AUMID for Xbox Game Pass games:

1. **PowerShell Method:**
   ```powershell
   Get-StartApps | Where-Object {$_.Name -like "*game name*"}
   ```

2. **Registry Method:**
   Look in `HKEY_CURRENT_USER\Software\Classes\ActivatableClasses\Package` for installed UWP packages.

3. **Apps Folder:**
   Navigate to `shell:appsFolder` in Windows Explorer to see all installed apps with their AUMIDs.

### Adding to Steam

1. Build or download `librarylink.exe`
2. In Steam, go to "Games" → "Add a Non-Steam Game to My Library"
3. Click "Browse" and select `librarylink.exe`
4. In the launch options, add: `uwp-launch <AUMID>`
5. Rename the entry to match your game
6. (Optional) Add custom artwork and configure as needed

**Example Steam Launch Options:**
```
uwp-launch Microsoft.MinecraftUWP_8wekyb3d8bbwe!App
```