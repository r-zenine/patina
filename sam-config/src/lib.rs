use fsutils::walk_dir;
use fsutils::ErrorsFS;
use sam_core::entities::choices::Choice;
use sam_core::entities::identifiers::Identifier;
use sam_persistence::CacheError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

const CONFIG_FILE_NAME: &str = ".sam_rc.toml";
const HISTORY_DIR: &str = ".local/share/sam/";
const CACHE_DIR: &str = ".cache/";

fn default_min_cache_duration_secs() -> u64 {
    2
}

fn default_ttl_secs() -> u64 {
    3600
}

/// Always-on root directories, included only when they exist on disk:
/// `./.sam` (cwd-relative) and `~/.config/sam` (home-based). Doubles as the
/// serde default when `root_dir` is absent from the configuration file.
fn default_root_dir() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let sam = PathBuf::from(".sam");
    if sam.is_dir() {
        dirs.push(sam);
    }
    if let Some(home) = dirs::home_dir() {
        let cfg = home.join(".config").join("sam");
        if cfg.is_dir() {
            dirs.push(cfg);
        }
    }
    dirs
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AppSettings {
    #[serde(default = "default_root_dir")]
    root_dir: Vec<PathBuf>,
    #[serde(default = "default_ttl_secs")]
    ttl: u64,
    #[serde(default = "default_min_cache_duration_secs")]
    min_cache_duration_secs: u64,
    #[serde(flatten)]
    pub env_variables: HashMap<String, String>,
    #[serde(skip)]
    cache_dir: PathBuf,
    #[serde(skip)]
    history_file: PathBuf,
    #[serde(skip)]
    pub dry: bool,
    #[serde(skip)]
    pub silent: bool,
    #[serde(skip)]
    pub no_cache: bool,
    #[serde(skip)]
    pub defaults: HashMap<Identifier, Vec<Choice>>,
}

pub type Result<T> = std::result::Result<T, ErrorsSettings>;

impl AppSettings {
    fn read_config(path: PathBuf) -> Result<AppSettings> {
        let path = fsutils::ensure_exists(path)
            .and_then(fsutils::ensure_is_file)
            .and_then(fsutils::ensure_sufficient_permisions)?;
        let content = fs::read_to_string(&path)?;
        let conf: AppSettings = toml::from_str(content.as_str())?;
        Ok(conf)
    }

    pub fn load_from(path: PathBuf) -> Result<Self> {
        let cache_dir =
            Self::file_path_with_suffix(CACHE_DIR, "sam", ErrorsSettings::CantFindCacheDirectory)?;
        let history_file = Self::file_path_with_suffix(
            HISTORY_DIR,
            "history",
            ErrorsSettings::CantFindHistoryDirectory(HISTORY_DIR.to_string()),
        )?;

        Self::read_config(path)
            .and_then(AppSettings::validate)
            .map(|mut e| {
                e.resolve_root_dirs();
                e.cache_dir = cache_dir;
                e.history_file = history_file;
                e
            })
    }

    pub fn load() -> Result<Self> {
        let home_dir_o = Self::home_dir_config_path()?;
        let current_dir_o = Self::current_dir_config_path();

        let config_home_dir = Self::read_config(home_dir_o);
        let config_current_dir = current_dir_o.and_then(Self::read_config);

        let cache_dir =
            Self::file_path_with_suffix(CACHE_DIR, "sam", ErrorsSettings::CantFindCacheDirectory)?;
        let history_file = Self::file_path_with_suffix(
            HISTORY_DIR,
            "history",
            ErrorsSettings::CantFindHistoryDirectory(HISTORY_DIR.to_string()),
        )?;

        config_current_dir
            .or(config_home_dir)
            .and_then(AppSettings::validate)
            .map(|mut e| {
                e.resolve_root_dirs();
                e.cache_dir = cache_dir;
                e.history_file = history_file;
                e
            })
    }

    pub fn merge_session_defaults(&mut self, session_defaults: HashMap<Identifier, Vec<Choice>>) {
        for (identifier, choices) in session_defaults {
            self.defaults.entry(identifier).or_insert(choices);
        }
    }

    pub const fn ttl(&self) -> Duration {
        Duration::from_secs(self.ttl)
    }

    pub const fn min_cache_duration(&self) -> Duration {
        Duration::from_secs(self.min_cache_duration_secs)
    }

    pub fn cache_dir(&self) -> &'_ Path {
        self.cache_dir.as_ref()
    }

    pub fn history_file(&self) -> &'_ Path {
        self.history_file.as_ref()
    }

    /// Expands `~`/`$HOME` in every configured `root_dir` entry, then ensures
    /// the always-on directories (`default_root_dir`) are present without
    /// introducing duplicates.
    fn resolve_root_dirs(&mut self) {
        self.root_dir = Self::merge_root_dirs(&self.root_dir, default_root_dir());
    }

    /// Pure merge step: expands `~`/`$HOME` in `configured` entries, then
    /// appends each of `extras` that is not already present.
    fn merge_root_dirs(configured: &[PathBuf], extras: Vec<PathBuf>) -> Vec<PathBuf> {
        let mut result: Vec<PathBuf> = configured.iter().map(|p| fsutils::expand_home(p)).collect();
        for candidate in extras {
            if !result.contains(&candidate) {
                result.push(candidate);
            }
        }
        result
    }

    fn validate(orig: AppSettings) -> Result<AppSettings> {
        for path in &orig.root_dir {
            if let Ok(files) = fsutils::walk_dir(path) {
                for f in files {
                    fsutils::ensure_exists(f).and_then(fsutils::ensure_sufficient_permisions)?;
                }
            }
        }
        Ok(orig)
    }

    fn home_dir_config_path() -> Result<PathBuf> {
        dirs::home_dir()
            .map(|e| e.join(CONFIG_FILE_NAME))
            .ok_or(ErrorsSettings::CantFindHomeDirectory)
    }

    fn file_path_with_suffix(path: &str, file_name: &str, err: ErrorsSettings) -> Result<PathBuf> {
        dirs::home_dir()
            .map(|e| e.join(path))
            .and_then(|path| path.exists().then(|| path.join(file_name)))
            .ok_or(err)
    }

    fn current_dir_config_path() -> Result<PathBuf> {
        std::env::current_dir()
            .map_err(|_| ErrorsSettings::CantFindCurrentDirectory)
            .map(|e| e.join(CONFIG_FILE_NAME))
    }

    pub fn variables(&self) -> HashMap<String, String> {
        self.env_variables.clone()
    }

    fn sam_files(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.root_dir
            .iter()
            .map(AsRef::as_ref)
            .flat_map(walk_dir)
            .flatten()
    }

    pub fn aliases_files(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.sam_files().filter(|f| {
            if let Some(file_name) = f.file_name() {
                file_name == "aliases.yaml" || file_name == "aliases.yml"
            } else {
                false
            }
        })
    }

    pub fn vars_files(&self) -> impl Iterator<Item = PathBuf> + '_ {
        self.sam_files().filter(|f| {
            if let Some(file_name) = f.file_name() {
                file_name == "vars.yaml" || file_name == "vars.yml"
            } else {
                false
            }
        })
    }
}

#[derive(Debug, Error)]
pub enum ErrorsSettings {
    #[error("got deserialize the configuration file because\n-> {0}")]
    CantDeserialize(#[from] toml::de::Error),
    #[error("can't read the configuration file because\n-> {0}")]
    CantReadConfigFile(#[from] io::Error),
    #[error("got the following file-system related error\n-> {0}")]
    FileSystem(#[from] ErrorsFS),
    #[error("could not initialize the cache\n-> {0}")]
    VarsCache(#[from] CacheError),
    #[error("we were unable to locate the home directory for the current user")]
    CantFindHomeDirectory,
    #[error("we were unable to locate the cache directory for the current user")]
    CantFindCacheDirectory,
    #[error("we were unable to locate the current directory for the current user")]
    CantFindCurrentDirectory,
    #[error(
        "we were unable to locate the history directory for the current user, make sure {0} exists"
    )]
    CantFindHistoryDirectory(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_expands_configured_entries() {
        let home = dirs::home_dir().expect("home directory required for this test");
        let configured = vec![PathBuf::from("~/work")];
        let merged = AppSettings::merge_root_dirs(&configured, Vec::new());
        assert_eq!(merged, vec![home.join("work")]);
    }

    #[test]
    fn merge_appends_existing_extras() {
        let tmp = std::env::temp_dir();
        let configured = vec![PathBuf::from("/some/configured")];
        let merged = AppSettings::merge_root_dirs(&configured, vec![tmp.clone()]);
        assert_eq!(merged, vec![PathBuf::from("/some/configured"), tmp]);
    }

    #[test]
    fn merge_does_not_duplicate_extras() {
        let shared = PathBuf::from("/shared/dir");
        let configured = vec![shared.clone()];
        let merged = AppSettings::merge_root_dirs(&configured, vec![shared.clone()]);
        assert_eq!(merged, vec![shared]);
    }
}
