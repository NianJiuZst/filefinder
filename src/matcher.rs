use anyhow::Result;
use glob::Pattern;
use regex::Regex;

#[derive(Debug, Clone)]
pub enum Matcher {
    Glob(Pattern),
    Regex(Regex),
}

impl Matcher {
    pub fn new_glob(pattern: &str) -> Result<Self> {
        let p = Pattern::new(pattern)?;
        Ok(Matcher::Glob(p))
    }

    pub fn new_regex(pattern: &str) -> Result<Self> {
        let re = Regex::new(pattern)?;
        Ok(Matcher::Regex(re))
    }

    pub fn is_match(&self, text: &str) -> bool {
        match self {
            Matcher::Glob(p) => p.matches(text),
            Matcher::Regex(r) => r.is_match(text),
        }
    }
}

pub fn create_matcher(pattern: &str, use_regex: bool) -> Result<Matcher> {
    if use_regex {
        Matcher::new_regex(pattern)
    } else {
        // Convert glob pattern to glob crate format
        Matcher::new_glob(pattern)
    }
}
