use std::path::Path;
use std::time::SystemTime;

pub struct Output {
    is_tty: bool,
}

// ANSI color codes
const RESET: &str = "\x1b[0m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";

impl Output {
    pub fn new() -> Self {
        Output {
            is_tty: atty::is(atty::Stream::Stdout),
        }
    }

    fn color(&self, code: &str, text: &str) -> String {
        if self.is_tty {
            format!("{}{}{}", code, text, RESET)
        } else {
            text.to_string()
        }
    }

    pub fn print_entry(&self, idx: usize, path: &Path, size: u64, mtime: u64) -> std::io::Result<()> {
        // File number
        print!("{} ", self.color(CYAN, &format!("[{}]", idx)));

        // File name (last component)
        if let Some(name) = path.file_name() {
            print!("{}", self.color(GREEN, &name.to_string_lossy()));
        }

        // Full path in parentheses
        if let Some(parent) = path.parent() {
            let parent_str = parent.to_string_lossy();
            if parent_str != "." {
                print!(" {}", self.color(DIM, &format!("({})", parent_str)));
            }
        }

        // Size
        print!(" {}", self.color(YELLOW, &format_size(size)));

        // Modified time
        if let Some(time) = SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(mtime)) {
            if let Ok(datetime) = time.duration_since(SystemTime::UNIX_EPOCH) {
                let secs = datetime.as_secs();
                let days = secs / 86400;
                let seconds = secs % 86400;
                let hours = seconds / 3600;
                let minutes = (seconds % 3600) / 60;
                // Calculate year/month/day
                let days_since_epoch = days as i64;
                let mut year = 1970;
                let mut remaining_days = days_since_epoch;
                loop {
                    let days_in_year = if is_leap_year(year) { 366 } else { 365 };
                    if remaining_days < days_in_year {
                        break;
                    }
                    remaining_days -= days_in_year;
                    year += 1;
                }
                let days_in_months: [i64; 12] = if is_leap_year(year) {
                    [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
                } else {
                    [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
                };
                let mut month = 1;
                for days_in_month in days_in_months.iter() {
                    if remaining_days < *days_in_month {
                        break;
                    }
                    remaining_days -= *days_in_month;
                    month += 1;
                }
                let day = remaining_days + 1;
                print!(" {}", self.color(DIM, &format!("{:04}-{:02}-{:02} {:02}:{:02}", year, month, day, hours, minutes)));
            }
        }

        println!();
        Ok(())
    }

    pub fn print_prompt(&self, count: usize) -> std::io::Result<()> {
        println!("\nFound {} file(s). Enter number to open (q to quit):", self.color(CYAN, &count.to_string()));
        Ok(())
    }

    #[allow(dead_code)]
    pub fn print_error(&self, msg: &str) -> std::io::Result<()> {
        eprintln!("{}", self.color(RED, &format!("Error: {}", msg)));
        Ok(())
    }
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.1}G", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1}M", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1}K", size as f64 / KB as f64)
    } else {
        format!("{}B", size)
    }
}
