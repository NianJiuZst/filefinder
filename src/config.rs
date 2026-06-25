use anyhow::Result;
use clap::{Parser, ValueHint};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SizeRange {
    pub min: Option<u64>,
    pub max: Option<u64>,
}

impl SizeRange {
    pub fn parse(s: &str) -> Result<Self> {
        let s = s.trim();
        let (min_str, max_str) = s.split_once("..").unwrap_or((s, s));
        let min = if min_str.is_empty() {
            None
        } else {
            Some(parse_size(min_str)?)
        };
        let max = if max_str.is_empty() {
            None
        } else {
            Some(parse_size(max_str)?)
        };
        Ok(SizeRange { min, max })
    }

    pub fn contains(&self, size: u64) -> bool {
        if let Some(min) = self.min {
            if size < min {
                return false;
            }
        }
        if let Some(max) = self.max {
            if size > max {
                return false;
            }
        }
        true
    }
}

fn parse_size(s: &str) -> Result<u64> {
    let s = s.trim();
    let split_at = s
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(s.len());
    let (num_str, unit) = s.split_at(split_at);

    let num: f64 = num_str.parse()?;
    let multiplier: u64 = if unit.is_empty() || unit.eq_ignore_ascii_case("B") {
        1
    } else if unit.eq_ignore_ascii_case("K") || unit.eq_ignore_ascii_case("KB") {
        1024
    } else if unit.eq_ignore_ascii_case("M") || unit.eq_ignore_ascii_case("MB") {
        1024 * 1024
    } else if unit.eq_ignore_ascii_case("G") || unit.eq_ignore_ascii_case("GB") {
        1024 * 1024 * 1024
    } else if unit.eq_ignore_ascii_case("T") || unit.eq_ignore_ascii_case("TB") {
        1024 * 1024 * 1024 * 1024
    } else {
        anyhow::bail!("Unknown size unit: {}", s);
    };

    Ok((num * multiplier as f64) as u64)
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, verbatim_doc_comment)]
pub struct Args {
    #[arg(value_hint = ValueHint::DirPath)]
    pub path: Option<PathBuf>,

    #[arg(value_hint = ValueHint::FilePath)]
    pub name: Option<String>,

    #[arg(short, long, help = "Filter by file extension (e.g. rs, txt, md)")]
    pub ext: Option<String>,

    #[arg(
        short,
        long,
        help = "Filter by size range (e.g. 10K..100M, 1M.., ..1G)"
    )]
    pub size: Option<String>,

    #[arg(short, long, help = "Use regex matching instead of glob")]
    pub regex: bool,

    #[arg(long, action = clap::ArgAction::SetTrue, help = "Ignore .git directories")]
    pub ignore_git: bool,

    #[arg(long, action = clap::ArgAction::SetTrue, help = "Ignore node_modules directories")]
    pub ignore_node: bool,

    #[arg(long, help = "Maximum directory recursion depth")]
    pub max_depth: Option<usize>,

    #[arg(short, long, action = clap::ArgAction::SetTrue, help = "Interactive mode: select a file to open")]
    pub interactive: bool,
}

#[derive(Debug, Clone)]
pub struct SearchConfig {
    pub path: PathBuf,
    pub pattern: Option<String>,
    pub ext: Option<String>,
    pub size_range: Option<SizeRange>,
    pub use_regex: bool,
    pub ignore_git: bool,
    pub ignore_node: bool,
    pub max_depth: Option<usize>,
    pub interactive: bool,
}

impl From<Args> for SearchConfig {
    fn from(args: Args) -> Self {
        let size_range = args.size.as_ref().and_then(|s| SizeRange::parse(s).ok());

        SearchConfig {
            path: args.path.unwrap_or_else(|| PathBuf::from(".")),
            pattern: args.name,
            ext: args.ext,
            size_range,
            use_regex: args.regex,
            ignore_git: args.ignore_git,
            ignore_node: args.ignore_node,
            max_depth: args.max_depth,
            interactive: args.interactive,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SizeRange;

    #[test]
    fn parses_size_units_without_dropping_filters() {
        let range = SizeRange::parse("10K..1MB").unwrap();

        assert_eq!(range.min, Some(10 * 1024));
        assert_eq!(range.max, Some(1024 * 1024));
        assert!(range.contains(64 * 1024));
        assert!(!range.contains(2 * 1024 * 1024));
    }

    #[test]
    fn parses_open_ended_size_ranges() {
        let min_only = SizeRange::parse("1.5M..").unwrap();
        let max_only = SizeRange::parse("..2G").unwrap();

        assert_eq!(min_only.min, Some(1_572_864));
        assert_eq!(min_only.max, None);
        assert_eq!(max_only.min, None);
        assert_eq!(max_only.max, Some(2 * 1024 * 1024 * 1024));
    }
}
