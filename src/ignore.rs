use ignore::gitignore::Gitignore;
use std::path::Path;

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

    pub fn should_ignore(&self, path: &Path) -> bool {
        // Check for .git directory
        if self.ignore_git {
            for component in path.components() {
                if component.as_os_str() == ".git" {
                    return true;
                }
            }
        }

        // Check for node_modules
        if self.ignore_node {
            for component in path.components() {
                if component.as_os_str() == "node_modules" {
                    return true;
                }
            }
        }

        // Check against .gitignore patterns
        let path_str = path.to_string_lossy();
        for glob in &self.extra_ignores {
            if let ignore::Match::Ignore(_) = glob.matched(path_str.as_ref(), path.is_dir())
            {
                return true;
            }
        }

        false
    }
}
