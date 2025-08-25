#![windows_subsystem = "windows"]

use std::env;
use std::mem;
use std::process::Command;
use windows::ApplicationModel::AppInfo;
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoUninitialize,
};
use windows::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
use windows::Win32::UI::Shell::{
    AO_NONE, ApplicationActivationManager, IApplicationActivationManager,
};
use windows::core::{HSTRING, PWSTR};
use windows::{Win32::Foundation::*, Win32::System::ProcessStatus::*, Win32::System::Threading::*};

#[derive(Debug)]
struct ProcessInfo {
    name: String,
    path: String,
}

#[derive(Debug)]
struct AppEntry {
    name: String,
    aumid: String,
}

fn find_apps_powershell(search_term: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    // Build PowerShell command
    let command = "Get-StartApps | ForEach-Object { \"$($_.Name)`t$($_.AppID)\" }".to_string();

    let output = Command::new("powershell")
        .args(["-Command", &command])
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "PowerShell command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut apps = Vec::new();

    // Parse PowerShell output
    for line in output_str.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Split by tab character
        if let Some(tab_pos) = line.find('\t') {
            let name = line[..tab_pos].trim().to_string();
            let aumid = line[tab_pos + 1..].trim().to_string();

            // Skip if AUMID is empty (likely a Win32 app)
            if aumid.is_empty() {
                continue;
            }

            // Apply search filter if provided
            if let Some(term) = search_term {
                if !name.to_lowercase().contains(&term.to_lowercase()) {
                    continue;
                }
            }

            apps.push(AppEntry { name, aumid });
        }
    }

    // Sort apps by name
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    // Output results
    print_apps_table(&apps);

    Ok(())
}

fn print_apps_table(apps: &[AppEntry]) {
    if apps.is_empty() {
        println!("No applications found.");
        return;
    }

    println!("=== UWP Applications ===");
    println!("Found {} applications:\n", apps.len());

    // Calculate column widths
    let max_name_width = apps
        .iter()
        .map(|app| app.name.len())
        .max()
        .unwrap_or(12)
        .max(12);

    // Print header
    println!(
        "{:<width$} AUMID",
        "Application Name",
        width = max_name_width
    );
    println!(
        "{:<width$} {}",
        "-".repeat(max_name_width),
        "-".repeat(50),
        width = max_name_width
    );

    // Print apps
    for app in apps {
        println!("{:<width$} {}", app.name, app.aumid, width = max_name_width);
    }
}

fn main() {
    unsafe {
        // If AttachConsole fails, we can still run without a console
        // This is useful for GUI applications or when running in the background
        AttachConsole(ATTACH_PARENT_PROCESS).unwrap_or(());
    }

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <command> [arguments]", args[0]);
        println!("Commands:");
        println!("  uwp-launch <AUMID>          - Look up UWP app info and launch it");
        println!("  list-apps [options]         - List installed UWP apps and their AUMIDs");
        println!();
        println!("List Apps Options:");
        println!("  --search <term>             - Search for apps containing the term");
        println!();
        println!("Examples:");
        println!(
            "  {} uwp-launch Microsoft.WindowsCalculator_8wekyb3d8bbwe!App",
            args[0]
        );
        println!("  {} list-apps", args[0]);
        println!("  {} list-apps --search forza", args[0]);
        return;
    }

    match args[1].as_str() {
        "uwp-launch" => {
            if args.len() < 3 {
                println!(
                    "Error: UWP launch requires an Application User Model ID. Try using librarylink list-apps to find it."
                );
                println!("Usage: {} uwp-launch <AUMID>", args[0]);
                return;
            }
            launch_uwp_app(&args[2]);
        }
        "list-apps" => {
            let mut search_term: Option<&str> = None;

            // Parse arguments
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--search" => {
                        if i + 1 < args.len() {
                            search_term = Some(&args[i + 1]);
                            i += 2;
                        } else {
                            println!("Error: --search requires a search term");
                            println!("Usage: {} list-apps --search <term>", args[0]);
                            return;
                        }
                    }
                    _ => {
                        println!("Error: Unknown option '{}'", args[i]);
                        println!("Usage: {} list-apps [--search <term>]", args[0]);
                        return;
                    }
                }
            }

            match find_apps_powershell(search_term) {
                Ok(()) => {}
                Err(e) => {
                    println!("Error finding applications: {}", e);
                }
            }
        }
        _ => {
            println!("Unknown command: {}", args[1]);
            println!("Use 'uwp-launch' or 'list-apps'");
        }
    }
}

fn launch_uwp_app(aumid: &str) {
    println!("=== UWP App Launch ===");
    println!("Looking up and launching app with AUMID: {}", aumid);
    println!();

    // Convert AUMID to HSTRING for Windows API
    let aumid_hstring = HSTRING::from(aumid);

    // Use GetFromAppUserModelId to get AppInfo
    match AppInfo::GetFromAppUserModelId(&aumid_hstring) {
        Ok(app_info) => {
            println!("Successfully found app information!");

            // Get the display name
            match app_info.DisplayInfo() {
                Ok(display_info) => match display_info.DisplayName() {
                    Ok(display_name) => {
                        println!("App Display Name: {}", display_name);
                    }
                    Err(e) => println!("Could not get display name: {}", e),
                },
                Err(e) => println!("Could not get display info: {}", e),
            }

            // Get the package information
            match app_info.Package() {
                Ok(package) => {
                    // Get package display name
                    match package.DisplayName() {
                        Ok(package_name) => {
                            println!("Package Display Name: {}", package_name);
                        }
                        Err(e) => println!("Could not get package name: {}", e),
                    }

                    // Get install location
                    match package.InstalledPath() {
                        Ok(install_path) => {
                            println!("*** INSTALL DIRECTORY FOUND ***");
                            println!("Installed Path: {}", install_path);
                        }
                        Err(e) => println!("Could not get install path: {}", e),
                    }

                    // Get package ID information
                    match package.Id() {
                        Ok(package_id) => {
                            match package_id.FullName() {
                                Ok(full_name) => {
                                    println!("Package Full Name: {}", full_name);
                                }
                                Err(e) => println!("Could not get full name: {}", e),
                            }

                            match package_id.FamilyName() {
                                Ok(family_name) => {
                                    println!("Package Family Name: {}", family_name);
                                }
                                Err(e) => println!("Could not get family name: {}", e),
                            }
                        }
                        Err(e) => println!("Could not get package ID: {}", e),
                    }
                }
                Err(e) => {
                    println!("Could not get package information: {}", e);
                    println!("This might not be a UWP app or the AUMID might be incorrect.");
                    return;
                }
            }

            println!();
            println!("=== Launching Application ===");

            // Now launch the app using IApplicationActivationManager
            match launch_app_with_activation_manager(aumid) {
                Ok(process_id) => {
                    println!("✅ Successfully launched app!");
                    println!("🚀 Process ID: {}", process_id);
                    println!();

                    // Get process information and start monitoring
                    if let Some(process_info) = get_process_info(process_id) {
                        println!("📋 Launched Process Details:");
                        println!("   Process Path: {}", process_info.path);
                        println!();

                        // Extract directory from the process path
                        let process_dir = get_directory_from_path(&process_info.path);
                        println!("🔍 Starting process monitoring...");
                        println!("   Monitoring directory: {}", process_dir);
                        println!("   Initial process ID: {}", process_id);
                        println!();

                        // Start monitoring the process
                        monitor_process(process_id, &process_dir);
                    } else {
                        println!("⚠️ Could not get process information for monitoring");
                    }
                }
                Err(e) => {
                    println!("❌ Failed to launch app: {}", e);
                    println!("Trying fallback launch method...");

                    // Fallback to using ShellExecute
                    match launch_app_with_shell_execute(aumid) {
                        Ok(()) => {
                            println!(
                                "✅ App launched using fallback method (no process ID available)"
                            );
                            println!("⚠️ Process monitoring not available with fallback method");
                        }
                        Err(e) => {
                            println!("❌ All launch methods failed: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to find app with AUMID '{}': {}", aumid, e);
            println!("Possible reasons:");
            println!("  - The AUMID is incorrect");
            println!("  - The app is not installed for the current user");
            println!("  - The app is not a UWP application");
            println!("  - Access permissions issue");
        }
    }
}

fn launch_app_with_activation_manager(aumid: &str) -> Result<u32, Box<dyn std::error::Error>> {
    unsafe {
        // Initialize COM
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if hr.is_err() {
            return Err("Failed to initialize COM".into());
        }

        // Create ApplicationActivationManager
        let activation_manager: IApplicationActivationManager =
            CoCreateInstance(&ApplicationActivationManager, None, CLSCTX_INPROC_SERVER).map_err(
                |e| {
                    CoUninitialize();
                    format!("Failed to create ApplicationActivationManager: {}", e)
                },
            )?;

        let aumid_hstring: HSTRING = HSTRING::from(aumid);

        // Launch the app and get the process ID (returned directly)
        let result = activation_manager.ActivateApplication(
            &aumid_hstring,
            None, // No arguments
            AO_NONE,
        );

        // Cleanup COM
        CoUninitialize();

        match result {
            Ok(process_id) => Ok(process_id),
            Err(e) => Err(format!("Failed to activate application: {}", e).into()),
        }
    }
}

fn launch_app_with_shell_execute(aumid: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    // Use PowerShell to launch the UWP app
    let powershell_command = format!("Start-Process \"shell:appsFolder\\{}\"", aumid);

    let output = Command::new("powershell")
        .args(["-Command", &powershell_command])
        .output()
        .map_err(|e| format!("Failed to execute PowerShell command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        Err(format!("PowerShell command failed: {}", error_msg).into())
    }
}

fn get_process_info(process_id: u32) -> Option<ProcessInfo> {
    unsafe {
        // Open the process
        let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id);

        let process_handle = match process_handle {
            Ok(handle) => handle,
            Err(_) => return None,
        };

        // Get the process image name
        let mut image_name: [u16; 260] = [0; 260]; // MAX_PATH
        let mut size: u32 = image_name.len() as u32;
        let result = QueryFullProcessImageNameW(
            process_handle,
            PROCESS_NAME_WIN32,
            PWSTR(image_name.as_mut_ptr()),
            &mut size,
        );

        let path = if result.is_ok() && size > 0 {
            let name_slice = &image_name[0..size as usize];
            String::from_utf16_lossy(name_slice)
        } else {
            "<Unknown>".to_string()
        };

        // Extract just the filename from the full path
        let name = path.split('\\').next_back().unwrap_or(&path).to_string();

        // Close the process handle
        let _ = CloseHandle(process_handle);

        Some(ProcessInfo { name, path })
    }
}

fn get_directory_from_path(path: &str) -> String {
    // Extract directory from full path
    if let Some(last_slash) = path.rfind('\\') {
        path[..last_slash].to_string()
    } else {
        path.to_string()
    }
}

fn monitor_process(mut current_process_id: u32, target_directory: &str) {
    loop {
        let process_handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, current_process_id) };

        let process_handle = match process_handle {
            Ok(handle) => handle,
            Err(_) => {
                println!(
                    "❌ Failed to open process {} for monitoring",
                    current_process_id
                );
                println!(
                    "🔍 Searching for replacement process in directory: {}",
                    target_directory
                );

                // Look for another process in the same directory
                match find_process_in_directory(target_directory) {
                    Some(new_process_id) => {
                        println!("🔄 Found replacement process: {}", new_process_id);
                        if let Some(process_info) = get_process_info(new_process_id) {
                            println!("   Process Name: {}", process_info.name);
                            println!("   Process Path: {}", process_info.path);
                        }
                        current_process_id = new_process_id;
                        println!("📍 Now monitoring process {}", current_process_id);
                        println!();
                        continue;
                    }
                    None => {
                        println!("💀 No replacement process found in target directory");
                        println!("🚪 Exiting monitoring...");
                        break;
                    }
                }
            }
        };

        println!(
            "⏳ Waiting for process {} to terminate...",
            current_process_id
        );

        // Wait for the process to terminate (handle becomes signaled)
        let wait_result = unsafe { WaitForSingleObject(process_handle, INFINITE) };

        // Close the handle after waiting
        unsafe {
            let _ = CloseHandle(process_handle);
        };

        match wait_result {
            WAIT_OBJECT_0 => {
                println!("❌ Process {} has terminated", current_process_id);
                println!(
                    "🔍 Searching for replacement process in directory: {}",
                    target_directory
                );

                // Look for another process in the same directory
                match find_process_in_directory(target_directory) {
                    Some(new_process_id) => {
                        println!("🔄 Found replacement process: {}", new_process_id);
                        if let Some(process_info) = get_process_info(new_process_id) {
                            println!("   Process Name: {}", process_info.name);
                            println!("   Process Path: {}", process_info.path);
                        }
                        current_process_id = new_process_id;
                        println!("📍 Now monitoring process {}", current_process_id);
                        println!();
                    }
                    None => {
                        println!("💀 No replacement process found in target directory");
                        println!("🚪 Exiting monitoring...");
                        break;
                    }
                }
            }
            WAIT_FAILED => {
                println!("❌ WaitForSingleObject failed. Error: {:?}", unsafe {
                    GetLastError()
                });
                println!(
                    "🔍 Searching for replacement process in directory: {}",
                    target_directory
                );

                // Look for another process in the same directory
                match find_process_in_directory(target_directory) {
                    Some(new_process_id) => {
                        println!("🔄 Found replacement process: {}", new_process_id);
                        if let Some(process_info) = get_process_info(new_process_id) {
                            println!("   Process Name: {}", process_info.name);
                            println!("   Process Path: {}", process_info.path);
                        }
                        current_process_id = new_process_id;
                        println!("📍 Now monitoring process {}", current_process_id);
                        println!();
                    }
                    None => {
                        println!("💀 No replacement process found in target directory");
                        println!("🚪 Exiting monitoring...");
                        break;
                    }
                }
            }
            _ => {
                println!(
                    "⚠️ Unexpected wait result: {:?}. Continuing monitoring...",
                    wait_result
                );
            }
        }
    }
}

fn find_process_in_directory(target_directory: &str) -> Option<u32> {
    let mut process_ids: [u32; 1024] = [0; 1024];
    let mut bytes_returned: u32 = 0;

    let result = unsafe {
        EnumProcesses(
            process_ids.as_mut_ptr(),
            (process_ids.len() * mem::size_of::<u32>()) as u32,
            &mut bytes_returned,
        )
    };

    if result.is_err() {
        return None;
    }

    let process_count = bytes_returned as usize / mem::size_of::<u32>();

    // Check each process to see if it's in the target directory
    let lowercase_target = target_directory.to_lowercase();
    for &process_id in process_ids.iter().take(process_count) {
        if process_id == 0 {
            continue;
        }

        if let Some(process_info) = get_process_info(process_id) {
            // Case-insensitive comparison for Windows paths
            if process_info
                .path
                .to_lowercase()
                .starts_with(&lowercase_target)
            {
                return Some(process_id);
            }
        }
    }

    None
}
