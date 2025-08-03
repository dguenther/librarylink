use windows_sys::{Win32::Foundation::*, Win32::System::ProcessStatus::*, Win32::System::Threading::*};
use std::mem;

fn main() {
    enumerate_win32_processes();
}

fn enumerate_win32_processes() {
    unsafe {
        // Buffer to store process IDs
        let mut process_ids: [u32; 1024] = [0; 1024];
        let mut bytes_returned: u32 = 0;
        
        // Enumerate all processes
        let success = EnumProcesses(
            process_ids.as_mut_ptr(),
            (process_ids.len() * mem::size_of::<u32>()) as u32,
            &mut bytes_returned,
        );
        
        if success == 0 {
            println!("Failed to enumerate processes. Error: {}", GetLastError());
            return;
        }
        
        // Calculate number of processes
        let process_count = bytes_returned as usize / mem::size_of::<u32>();
        
        println!("=== Traditional Win32 Processes ===");
        println!("Found {} processes:", process_count);
        println!("PID\t\tProcess Name");
        println!("---\t\t------------");
        
        // Iterate through each process ID
        for i in 0..process_count {
            let process_id = process_ids[i];
            
            // Skip process ID 0 (System Idle Process)
            if process_id == 0 {
                continue;
            }
            
            // Open the process
            let process_handle: *mut std::ffi::c_void = OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION,
                FALSE,
                process_id,
            );
            
            if process_handle.is_null() {
                println!("{}\t\t<Access Denied>", process_id);
                continue;
            }
            
            // Get the process image name using QueryFullProcessImageNameW
            let mut image_name: [u16; 260] = [0; 260]; // MAX_PATH
            let mut size: u32 = image_name.len() as u32;
            let result = QueryFullProcessImageNameW(
                process_handle,
                0,
                image_name.as_mut_ptr(),
                &mut size,
            );
            if result != 0 && size > 0 {
                let name_slice = &image_name[0..size as usize];
                let process_name = String::from_utf16_lossy(name_slice);
                println!("{}\t\t{}", process_id, process_name);
            } else {
                println!("{}\t\t<Unknown>", process_id);
            }
            
            // Close the process handle
            CloseHandle(process_handle);
        }
    }
}