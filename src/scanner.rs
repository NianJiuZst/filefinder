use anyhow::Result;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use crate::config::SearchConfig;
use crate::ignore::IgnoreRules;
use crate::matcher::Matcher;

// Use jwalk for parallel directory traversal (faster than walkdir)
use jwalk::WalkDir;

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
        // Collect files to check first (jwalk parallel traversal)
        let mut files_to_check: Vec<PathBuf> = Vec::new();

        let walker = WalkDir::new(&config.path)
            .follow_links(false)
            .max_depth(config.max_depth.unwrap_or(usize::MAX))
            .parallelism(jwalk::Parallelism::RayonNewPool(num_cpus::get()));

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path().to_path_buf();

            // Skip the root path itself
            if path == config.path {
                continue;
            }

            if self.ignore_rules.should_ignore(&path) {
                if entry.file_type().is_dir() {
                    // Skip entire directory
                    continue;
                }
            }

            if entry.file_type().is_file() {
                files_to_check.push(path);
            }
        }

        // Pre-extract filter criteria to avoid borrowing issues in closure
        let name_filter = self.matcher.clone();
        let ext_filter = self.config.ext.clone();
        let size_filter = self.config.size_range.clone();

        // Parallel file processing - only ONE metadata call per file
        let matched_entries: Vec<FileEntry> = files_to_check
            .par_iter()
            .filter_map(|path| {
                // Extension filter (no IO needed)
                if let Some(ref ext) = ext_filter {
                    let path_ext = path
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");
                    if path_ext != *ext {
                        return None;
                    }
                }

                // Pattern match (no IO needed)
                if let Some(ref matcher) = name_filter {
                    if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                        if !matcher.is_match(filename) {
                            return None;
                        }
                    } else {
                        return None;
                    }
                }

                // Get metadata ONCE for size + mtime, then check size filter
                let metadata = match fs::metadata(path) {
                    Ok(m) => m,
                    Err(_) => return None,
                };

                // Size filter (using same metadata we just got)
                if let Some(ref size_range) = size_filter {
                    if !size_range.contains(metadata.len()) {
                        return None;
                    }
                }

                // Extract mtime
                let mtime = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                Some(FileEntry {
                    path: path.clone(),
                    size: metadata.len(),
                    mtime,
                })
            })
            .collect();

        matched_entries
    }
}
