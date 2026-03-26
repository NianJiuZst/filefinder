mod config;
mod ignore;
mod interactive;
mod matcher;
mod output;
mod scanner;

use anyhow::Result;
use clap::Parser;


use config::{Args, SearchConfig};
use output::Output;
use scanner::Scanner;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    let config = SearchConfig::from(args);

    // Validate path
    if !config.path.exists() {
        anyhow::bail!("Path does not exist: {}", config.path.display());
    }

    // Scan
    let scanner = Scanner::new(config.clone())?;
    let entries = scanner.scan();

    let output = Output::new();

    // Print results
    if entries.is_empty() {
        println!("No files found.");
        return Ok(());
    }

    let search_root = &config.path;
    for (i, entry) in entries.iter().enumerate() {
        let relative_path = if entry.path.starts_with(search_root) {
            entry.path.strip_prefix(search_root).unwrap_or(&entry.path)
        } else {
            &entry.path
        };
        output.print_entry(i + 1, relative_path, entry.size, entry.mtime)?;
    }

    println!("\nFound {} file(s).", entries.len());

    // Interactive mode
    if config.interactive {
        output.print_prompt(entries.len())?;
        interactive::interactive_select(entries, search_root)?;
    }

    Ok(())
}
