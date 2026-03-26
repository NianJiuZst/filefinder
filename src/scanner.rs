use anyhow::Result;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::SearchConfig;
use crate::ignore::IgnoreRules;
use crate::matcher::Matcher;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub mtime: u64,
}

pub struct Scanner {
    config: SearchConfig,
    matcher: Option<Matcher>,
    ignore_rules: IgnoreRules,
}

impl Scanner {
    pub fn new(config: SearchConfig) -> Result<Self> {
        let matcher = config.pattern.as_ref().map(|p| {
            crate::matcher::create_matcher(p, config.use_regex)
                .expect("Failed to create matcher")
        });

        let mut ignore_rules = IgnoreRules::new(config.ignore_git, config.ignore_node);
        ignore_rules.add_gitignore(&config.path).ok();

        Ok(Scanner {
            config,
            matcher,
            ignore_rules,
        })
    }

    pub fn scan(&self) -> Vec<FileEntry> {
        let config = &self.config;
        let _path_str = config.path.to_string_lossy().to_string();

        // Collect directories first (for parallel processing)
        let mut dirs_to_scan: Vec<PathBuf> = Vec::new();
        let mut files_to_check: Vec<PathBuf> = Vec::new();

        let walker = WalkDir::new(&config.path)
            .follow_links(false)
            .max_depth(config.max_depth.unwrap_or(usize::MAX));

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path().to_path_buf();

            // Skip the root path itself
            if path == config.path {
                continue;
            }

            if self.ignore_rules.should_ignore(&path) {
                if path.is_dir() {
                    // Skip entire directory
                    continue;
                }
            }

            if path.is_dir() {
                dirs_to_scan.push(path);
            } else {
                files_to_check.push(path);
            }
        }

        // Parallel file processing
        let matched_entries: Vec<FileEntry> = files_to_check
            .par_iter()
            .filter_map(|path| {
                self.matches(path).then(|| {
                    let metadata = fs::metadata(path).ok();
                    let (size, mtime) = metadata
                        .map(|m| {
                            let size = m.len();
                            let mtime = m
                                .modified()
                                .ok()
                                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            (size, mtime)
                        })
                        .unwrap_or((0, 0));

                    FileEntry {
                        path: path.clone(),
                        size,
                        mtime,
                    }
                })
            })
            .collect();

        matched_entries
    }

    fn matches(&self, path: &Path) -> bool {
        // Extension filter
        if let Some(ref ext) = self.config.ext {
            let path_ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            if path_ext != *ext {
                return false;
            }
        }

        // Pattern match
        if let Some(ref matcher) = self.matcher {
            // Match against the filename
            if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                if !matcher.is_match(filename) {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Size filter
        if let Some(ref size_range) = self.config.size_range {
            if let Ok(metadata) = fs::metadata(path) {
                if !size_range.contains(metadata.len()) {
                    return false;
                }
            }
        }

        true
    }
}

use std::time::UNIX_EPOCH;
