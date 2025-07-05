use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use sysinfo::{DiskExt, System, SystemExt};
use walkdir::WalkDir;

fn main() -> io::Result<()> {
    let mut sys = System::new_all();
    sys.refresh_disks_list();

    let mut output = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("output.txt")?;

    for disk in sys.disks() {
        log(&format!("Disk: {:?}", disk.name()), &mut output)?;
        log(&format!("Mount: {:?}", disk.mount_point()), &mut output)?;
        log(
            &format!("Total: {:.2} GB", disk.total_space() as f64 / 1e9),
            &mut output,
        )?;
        log(
            &format!("Available: {:.2} GB", disk.available_space() as f64 / 1e9),
            &mut output,
        )?;

        let mount_path = disk.mount_point();
        if mount_path.exists() && mount_path.is_dir() {
            for entry in WalkDir::new(mount_path)
                .min_depth(1)
                .max_depth(3)
                .into_iter()
                .filter_map(Result::ok)
            {
                let path = entry.path();

                let flag = if is_hidden(path) {
                    "[HIDDEN]"
                } else if path.is_dir() {
                    "[DIR]"
                } else {
                    "[FILE]"
                };

                log(&format!("{flag} {:?}", path.display()), &mut output)?;
            }
        }

        log("--------------------------\n", &mut output)?;
    }

    println!("[*] Scan complete. Output saved to output.txt");
    Ok(())
}

fn log(message: &str, file: &mut std::fs::File) -> io::Result<()> {
    writeln!(file, "{message}")?;
    Ok(())
}

#[cfg(target_family = "unix")]
fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .map(|name| name.to_string_lossy().starts_with('.'))
        .unwrap_or(false)
}

#[cfg(target_family = "windows")]
fn is_hidden(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;

    if let Ok(metadata) = fs::metadata(path) {
        (metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN) != 0
    } else {
        false
    }
}
