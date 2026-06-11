use ignore::gitignore::Gitignore;
use std::ffi::OsStr;
use std::path::{Component, Path};

#[derive(Debug, Clone)]
pub struct IgnoreRules {
    pub ignore_git: bool,
    pub ignore_node: bool,
    pub extra_ignores: Vec<Gitignore>,
}

impl IgnoreRules {
    pub fn new(ignore_git: bool, ignore_node: bool) -> Self {
        IgnoreRules {
            ignore_git,
            ignore_node,
            extra_ignores: Vec::new(),
        }
    }

    pub fn add_gitignore(&mut self, path: &Path) -> std::io::Result<()> {
        let gitignore_path = path.join(".gitignore");
        if gitignore_path.exists() {
            let (glob, err) = Gitignore::new(&gitignore_path);
            if let Some(e) = err {
                eprintln!("Warning: Failed to parse {}: {}", gitignore_path.display(), e);
            }
            self.extra_ignores.push(glob);
        }
        Ok(())
    }

    pub fn should_ignore(&self, path: &Path, is_dir: bool) -> bool {
        if self.ignore_git && has_component(path, ".git") {
            return true;
        }

        if self.ignore_node && has_component(path, "node_modules") {
            return true;
        }

        for glob in &self.extra_ignores {
            if let ignore::Match::Ignore(_) = glob.matched(path, is_dir) {
                return true;
            }
        }

        false
    }
}

fn has_component(path: &Path, name: &str) -> bool {
    let name = OsStr::new(name);
    path.components().any(|component| {
        matches!(component, Component::Normal(part) if part == name)
    })
}

#[cfg(test)]
mod tests {
    use super::IgnoreRules;
    use std::path::Path;

    #[test]
    fn ignores_common_heavy_directories_by_component() {
        let rules = IgnoreRules::new(true, true);

        assert!(rules.should_ignore(Path::new("/repo/.git/config"), false));
        assert!(rules.should_ignore(
            Path::new("/repo/packages/node_modules/lib/index.js"),
            false
        ));
        assert!(!rules.should_ignore(Path::new("/repo/src/git_helpers.rs"), false));
    }
}
