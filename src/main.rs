#![windows_subsystem = "windows"]

use std::env;
use std::mem;
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

fn main() {
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS).unwrap_or(());
    }

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: {} <command> [arguments]", args[0]);
        println!("Commands:");
        println!("  processes                    - List all Win32 processes");
        println!("  uwp-launch <AUMID>          - Look up UWP app info and launch it");
        println!();
        println!("Examples:");
        println!("  {} processes", args[0]);
        println!(
            "  {} uwp-launch Microsoft.WindowsCalculator_8wekyb3d8bbwe!App",
            args[0]
        );
        return;
    }

    match args[1].as_str() {
        "processes" => enumerate_win32_processes(),
        "uwp-launch" => {
            if args.len() < 3 {
                println!("Error: UWP launch requires an Application User Model ID");
                println!("Usage: {} uwp-launch <AUMID>", args[0]);
                return;
            }
            launch_uwp_app(&args[2]);
        }
        _ => {
            println!("Unknown command: {}", args[1]);
            println!("Use 'processes' or 'uwp-launch'");
        }
    }
}

fn enumerate_win32_processes() {
    unsafe {
        // Buffer to store process IDs
        let mut process_ids: [u32; 1024] = [0; 1024];
        let mut bytes_returned: u32 = 0;

        // Enumerate all processes
        let result = EnumProcesses(
            process_ids.as_mut_ptr(),
            (process_ids.len() * mem::size_of::<u32>()) as u32,
            &mut bytes_returned,
        );

        if result.is_err() {
            println!("Failed to enumerate processes. Error: {:?}", GetLastError());
            return;
        }

        // Calculate number of processes
        let process_count = bytes_returned as usize / mem::size_of::<u32>();

        println!("=== Traditional Win32 Processes ===");
        println!("Found {} processes:", process_count);
        println!("PID\t\tProcess Name");
        println!("---\t\t------------");

        // Iterate through each process ID
        for &process_id in process_ids.iter().take(process_count) {
            // Skip process ID 0 (System Idle Process)
            if process_id == 0 {
                continue;
            }

            // Open the process
            let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id);

            let process_handle = match process_handle {
                Ok(handle) => handle,
                Err(_) => {
                    println!("{}\t\t<Access Denied>", process_id);
                    continue;
                }
            };

            // Get the process image name using QueryFullProcessImageNameW
            let mut image_name: [u16; 260] = [0; 260]; // MAX_PATH
            let mut size: u32 = image_name.len() as u32;
            let result = QueryFullProcessImageNameW(
                process_handle,
                PROCESS_NAME_WIN32,
                PWSTR(image_name.as_mut_ptr()),
                &mut size,
            );
            if result.is_ok() && size > 0 {
                let name_slice = &image_name[0..size as usize];
                let process_name = String::from_utf16_lossy(name_slice);
                println!("{}\t\t{}", process_id, process_name);
            } else {
                println!("{}\t\t<Unknown>", process_id);
            }

            // Close the process handle
            let _ = CloseHandle(process_handle);
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
