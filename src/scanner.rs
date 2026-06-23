use anyhow::Result;
use jwalk::{Parallelism, WalkDirGeneric};
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use crate::config::SearchConfig;
use crate::ignore::IgnoreRules;
use crate::matcher::Matcher;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub size: u64,
    pub mtime: u64,
}

#[derive(Debug, Clone, Copy, Default)]
struct FileMetadata {
    size: u64,
    mtime: u64,
}

pub struct Scanner {
    config: SearchConfig,
    matcher: Option<Matcher>,
    ignore_rules: IgnoreRules,
}

impl Scanner {
    pub fn new(config: SearchConfig) -> Result<Self> {
        let matcher = config
            .pattern
            .as_ref()
            .map(|p| crate::matcher::create_matcher(p, config.use_regex))
            .transpose()?;

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
        let ignore_rules = self.ignore_rules.clone();
        let name_filter = self.matcher.clone();
        let ext_filter = self.config.ext.clone();
        let size_filter = self.config.size_range.clone();

        let walker = WalkDirGeneric::<((), Option<FileMetadata>)>::new(&config.path)
            .follow_links(false)
            .max_depth(config.max_depth.unwrap_or(usize::MAX))
            .parallelism(Parallelism::RayonDefaultPool {
                busy_timeout: Duration::from_secs(1),
            })
            .process_read_dir(move |depth, _, _, children| {
                if depth.is_none() {
                    return;
                }

                children.retain_mut(|entry_result| {
                    let entry = match entry_result {
                        Ok(entry) => entry,
                        Err(_) => return false,
                    };
                    let path = entry.path();

                    if ignore_rules.should_ignore(&path, entry.file_type().is_dir()) {
                        return false;
                    }

                    if !entry.file_type().is_file() {
                        return true;
                    }

                    if !matches_ext(&path, ext_filter.as_deref()) {
                        return false;
                    }

                    if !matches_name(entry.file_name().to_str(), name_filter.as_ref()) {
                        return false;
                    }

                    let metadata = match entry.metadata() {
                        Ok(metadata) => metadata,
                        Err(_) => return false,
                    };
                    let size = metadata.len();

                    if let Some(ref size_range) = size_filter {
                        if !size_range.contains(size) {
                            return false;
                        }
                    }

                    let mtime = metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);

                    entry.client_state = Some(FileMetadata { size, mtime });
                    true
                });
            });

        walker
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let metadata = entry.client_state?;

                Some(FileEntry {
                    path: entry.path(),
                    size: metadata.size,
                    mtime: metadata.mtime,
                })
            })
            .collect()
    }
}

fn matches_ext(path: &Path, ext_filter: Option<&str>) -> bool {
    match ext_filter {
        Some(ext) => path.extension().and_then(|e| e.to_str()) == Some(ext),
        None => true,
    }
}

fn matches_name(filename: Option<&str>, name_filter: Option<&Matcher>) -> bool {
    match name_filter {
        Some(matcher) => filename.is_some_and(|name| matcher.is_match(name)),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::Scanner;
    use crate::config::SearchConfig;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn scan_prunes_ignored_directories_before_descending() {
        let root = temp_root("prune");
        fs::create_dir_all(root.join("src")).unwrap();
        fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
        fs::write(root.join("src/keep.rs"), "").unwrap();
        fs::write(root.join("node_modules/pkg/skip.rs"), "").unwrap();

        let scanner = Scanner::new(SearchConfig {
            path: root.clone(),
            pattern: None,
            ext: Some("rs".to_string()),
            size_range: None,
            use_regex: false,
            ignore_git: true,
            ignore_node: true,
            max_depth: None,
            interactive: false,
        })
        .unwrap();

        let entries = scanner.scan();
        let names: Vec<_> = entries
            .iter()
            .filter_map(|entry| entry.path.file_name())
            .collect();

        assert_eq!(names, vec!["keep.rs"]);

        fs::remove_dir_all(root).unwrap();
    }

    fn temp_root(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("filefinder-{name}-{}-{now}", std::process::id()));
        path
    }
}
