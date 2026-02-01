use diffviz_git::GitRepository;
use diffviz_review::errors::Result as CoreResult;
use std::path::Path;

/// Configuration for the DiffViz environment
#[derive(Debug, Clone)]
pub struct Config {
    /// Author name for reviews and approvals
    pub author: String,
    /// Repository path
    pub repo_path: String,
    /// Enable verbose logging
    pub verbose: bool,
    /// Terminal backend type
    pub terminal_backend: TerminalBackend,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            author: whoami::username(),
            repo_path: ".".to_string(),
            verbose: false,
            terminal_backend: TerminalBackend::Crossterm,
        }
    }
}

/// Supported terminal backends
#[derive(Debug, Clone, PartialEq)]
pub enum TerminalBackend {
    Crossterm,
    // Future: Termion, TestBackend, WebBackend
}

/// Environment that assembles all dependencies for the application
///
/// This follows the Environment pattern from the Sam project, providing
/// dependency injection and composition of all application components.
pub struct Environment {
    #[allow(dead_code)] // Used in tests
    config: Config,
}

impl Environment {
    /// Create a new environment with the given configuration
    pub fn new(config: Config) -> CoreResult<Self> {
        // Validate that the repository exists (for connection testing)
        let _ = GitRepository::open(&config.repo_path).map_err(|e| {
            diffviz_review::errors::DiffVizError::Repository(format!(
                "Failed to open repository at '{}': {}",
                config.repo_path, e
            ))
        })?;

        Ok(Self { config })
    }

    /// Create environment from a repository path (used by tests)
    #[cfg(test)]
    pub fn from_repo_path<P: AsRef<Path>>(repo_path: P) -> CoreResult<Self> {
        let config = Config {
            repo_path: repo_path.as_ref().to_string_lossy().to_string(),
            ..Config::default()
        };
        Self::new(config)
    }
}

/// Builder for creating environments with different configurations
pub struct EnvironmentBuilder {
    config: Config,
}

impl EnvironmentBuilder {
    /// Start building a new environment
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Set the repository path
    pub fn repo_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.repo_path = path.as_ref().to_string_lossy().to_string();
        self
    }

    /// Set the author name
    pub fn author<S: Into<String>>(mut self, author: S) -> Self {
        self.config.author = author.into();
        self
    }

    /// Enable verbose logging
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.config.verbose = verbose;
        self
    }

    /// Set terminal backend
    pub fn terminal_backend(mut self, backend: TerminalBackend) -> Self {
        self.config.terminal_backend = backend;
        self
    }

    /// Build the environment
    pub fn build(self) -> CoreResult<Environment> {
        Environment::new(self.config)
    }
}

impl Default for EnvironmentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_git_repo() -> (TempDir, String) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path().to_string_lossy().to_string();

        // Initialize git repo
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .status()
            .expect("Failed to init git repo");

        // Configure git
        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .status()
            .expect("Failed to configure git user");

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&repo_path)
            .status()
            .expect("Failed to configure git email");

        (temp_dir, repo_path)
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.repo_path, ".");
        assert!(!config.verbose);
        assert_eq!(config.terminal_backend, TerminalBackend::Crossterm);
        assert!(!config.author.is_empty());
    }

    #[test]
    fn test_environment_builder() {
        let (_temp_dir, repo_path) = create_test_git_repo();

        let env = EnvironmentBuilder::new()
            .repo_path(&repo_path)
            .author("test_user")
            .verbose(true)
            .terminal_backend(TerminalBackend::Crossterm)
            .build()
            .expect("Failed to build environment");

        assert_eq!(env.config.repo_path, repo_path);
        assert_eq!(env.config.author, "test_user");
        assert!(env.config.verbose);
        assert_eq!(env.config.terminal_backend, TerminalBackend::Crossterm);
    }

    #[test]
    fn test_environment_from_repo_path() {
        let (_temp_dir, repo_path) = create_test_git_repo();

        let env = Environment::from_repo_path(&repo_path).expect("Failed to create environment");

        assert_eq!(env.config.repo_path, repo_path);
    }

    #[test]
    fn test_environment_fluent_api() {
        let (_temp_dir, repo_path) = create_test_git_repo();

        let env = EnvironmentBuilder::new()
            .repo_path(&repo_path)
            .author("fluent_user")
            .verbose(true)
            .terminal_backend(TerminalBackend::Crossterm)
            .build()
            .expect("Failed to build environment");

        assert_eq!(env.config.author, "fluent_user");
        assert!(env.config.verbose);
    }

    #[test]
    fn test_environment_invalid_repo() {
        let result = Environment::from_repo_path("/non/existent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_terminal_backend_variants() {
        assert_eq!(TerminalBackend::Crossterm, TerminalBackend::Crossterm);
        assert_ne!(format!("{:?}", TerminalBackend::Crossterm), "");
    }
}
