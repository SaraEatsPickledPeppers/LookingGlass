use directories::ProjectDirs;
use std::path::PathBuf;

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "looking-glass";
const APPLICATION: &str = "looking-glass";

fn project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
}

/// Returns the XDG config directory: ~/.config/looking-glass/
pub fn config_dir() -> PathBuf {
    project_dirs()
        .map(|dirs| dirs.config_dir().to_path_buf())
        .unwrap_or_else(|| {
            dirs::home_dir()
                .map(|home| home.join(".config").join("looking-glass"))
                .unwrap_or_else(|| PathBuf::from("/tmp/looking-glass/config"))
        })
}

/// Returns the XDG data directory: ~/.local/share/looking-glass/
pub fn data_dir() -> PathBuf {
    project_dirs()
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| {
            dirs::home_dir()
                .map(|home| home.join(".local").join("share").join("looking-glass"))
                .unwrap_or_else(|| PathBuf::from("/tmp/looking-glass/data"))
        })
}

/// Returns the XDG cache directory: ~/.cache/looking-glass/
pub fn cache_dir() -> PathBuf {
    project_dirs()
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .unwrap_or_else(|| {
            dirs::home_dir()
                .map(|home| home.join(".cache").join("looking-glass"))
                .unwrap_or_else(|| PathBuf::from("/tmp/looking-glass/cache"))
        })
}

/// Returns mod directories in priority order:
/// 1. Data directory mods: ~/.local/share/looking-glass/mods/
/// 2. Project root mods: <project_root>/mods/
pub fn mod_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    dirs.push(data_dir().join("mods"));

    // Try to find project root by looking for Cargo.toml or .git
    if let Some(project_root) = find_project_root() {
        dirs.push(project_root.join("mods"));
    }

    dirs
}

/// Returns the saves directory: ~/.local/share/looking-glass/saves/
pub fn save_dir() -> PathBuf {
    data_dir().join("saves")
}

/// Returns the logs directory: ~/.local/share/looking-glass/logs/
pub fn log_dir() -> PathBuf {
    data_dir().join("logs")
}

/// Returns the config file path: ~/.config/looking-glass/config.toml
pub fn config_file_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// Returns the local config file path: <project_root>/private/config.toml
pub fn local_config_file_path() -> Option<PathBuf> {
    find_project_root().map(|root| root.join("private").join("config.toml"))
}

fn find_project_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;

    loop {
        if current.join("Cargo.toml").exists() || current.join(".git").exists() {
            return Some(current);
        }

        if !current.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir_exists() {
        let path = config_dir();
        assert!(path.ends_with("looking-glass"));
    }

    #[test]
    fn test_data_dir_exists() {
        let path = data_dir();
        assert!(path.ends_with("looking-glass"));
    }

    #[test]
    fn test_cache_dir_exists() {
        let path = cache_dir();
        assert!(path.ends_with("looking-glass"));
    }

    #[test]
    fn test_mod_dirs_not_empty() {
        let dirs = mod_dirs();
        assert!(!dirs.is_empty());
        assert!(dirs[0].ends_with("mods"));
    }

    #[test]
    fn test_save_dir() {
        let path = save_dir();
        assert!(path.ends_with("saves"));
    }

    #[test]
    fn test_log_dir() {
        let path = log_dir();
        assert!(path.ends_with("logs"));
    }
}