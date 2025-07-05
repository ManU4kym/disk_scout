use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use sysinfo::{DiskExt, System, SystemExt};
use walkdir::WalkDir;

fn main() -> io::Result<()> {
    let mut sys = System::new_all();
    sys.refresh_disks_list();

    let mut file = File::create("output.html")?;

    writeln!(
        file,
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Disk Scout Report</title>
    <style>
        body {{ font-family: sans-serif; background: #f4f4f4; padding: 20px; }}
        h2 {{ margin-top: 40px; }}
        .disk {{ background: #fff; padding: 15px; margin-bottom: 30px; border-radius: 8px; box-shadow: 0 0 5px rgba(0,0,0,0.1); }}
        .tag {{ display: inline-block; padding: 2px 8px; border-radius: 5px; font-size: 12px; margin-right: 10px; }}
        .file {{ color: #555; }}
        .dir {{ color: #0645ad; }}
        .hidden {{ color: #a00; }}
        .tag-hidden {{ background: #fdd; color: #a00; border: 1px solid #a00; }}
        .tag-dir {{ background: #def; color: #0645ad; border: 1px solid #0645ad; }}
        .tag-file {{ background: #eee; color: #444; border: 1px solid #ccc; }}
        ul {{ list-style: none; padding-left: 20px; }}
    </style>
</head>
<body>
<h1>Disk Scout Scan Report</h1>
"#
    )?;

    for disk in sys.disks() {
        let name = disk.name().to_string_lossy();
        let mount = disk.mount_point().display().to_string();
        let total = disk.total_space() as f64 / 1e9;
        let available = disk.available_space() as f64 / 1e9;

        writeln!(
            file,
            r#"<div class="disk">
<h2>Disk: {name}</h2>
<p><strong>Mount:</strong> {mount}</p>
<p><strong>Total:</strong> {total:.2} GB</p>
<p><strong>Available:</strong> {available:.2} GB</p>
<ul>
"#
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
                let display = path.display();
                let (class, label) = if is_hidden(path) {
                    ("hidden", "tag-hidden")
                } else if path.is_dir() {
                    ("dir", "tag-dir")
                } else {
                    ("file", "tag-file")
                };

                writeln!(
                    file,
                    r#"<li class="{class}"><span class="tag {label}">[{class}]</span> {display}</li>"#
                )?;
            }
        }

        writeln!(file, "</ul></div>")?;
    }

    writeln!(file, "</body></html>")?;
    println!("HTML report saved as output.html");
    Ok(())
}

#[cfg(target_family = "windows")]
fn is_hidden(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;

    if let Ok(metadata) = std::fs::metadata(path) {
        (metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN) != 0
    } else {
        false
    }
}

#[cfg(target_family = "unix")]
fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .map(|name| name.to_string_lossy().starts_with('.'))
        .unwrap_or(false)
}
