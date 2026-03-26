use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command;

pub fn interactive_select(entries: Vec<crate::scanner::FileEntry>, search_root: &Path) -> io::Result<()> {
    if entries.is_empty() {
        println!("No files found.");
        return Ok(());
    }

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Show numbered list
    for (i, entry) in entries.iter().enumerate() {
        let relative_path = if entry.path.starts_with(search_root) {
            entry.path.strip_prefix(search_root).unwrap_or(&entry.path)
        } else {
            &entry.path
        };

        writeln!(
            handle,
            "[{}] {}",
            i + 1,
            relative_path.display()
        )?;
    }
    drop(handle);

    print!("Enter number to open (q to quit): ");
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut line = String::new();
    let mut input_file = stdin.lock();

    loop {
        line.clear();
        let bytes_read = input_file.read_line(&mut line)?;
        if bytes_read == 0 {
            // EOF
            break;
        }

        let line = line.trim();
        if line.eq_ignore_ascii_case("q") {
            break;
        }

        match line.parse::<usize>() {
            Ok(num) if num > 0 && num <= entries.len() => {
                let entry = &entries[num - 1];
                if let Err(e) = open_file(&entry.path) {
                    eprintln!("Failed to open file: {}", e);
                }
                break;
            }
            _ => {
                print!("Invalid selection. Enter a number (1-{}), or q to quit: ", entries.len());
                io::stdout().flush()?;
            }
        }
    }

    Ok(())
}

fn open_file(path: &Path) -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(path).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn()?;
    }

    Ok(())
}
