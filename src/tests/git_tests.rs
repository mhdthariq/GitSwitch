use std::process::Command;
use tempfile::TempDir;

use crate::git::update_git_remote;
use crate::utils::run_command;

/// Helper function to set up a test Git repository
fn setup_test_repo() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to initialize git repository");

    // Set initial remote
    Command::new("git")
        .args(["remote", "add", "origin", "git@github.com:olduser/repo.git"])
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to add git remote");

    temp_dir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_git_remote() {
        let repo_dir = setup_test_repo();

        // Update remote URL
        update_git_remote("newuser", "repo");

        // Get the new remote URL
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(repo_dir.path())
            .output()
            .expect("Failed to get remote URL");

        let new_url = String::from_utf8_lossy(&output.stdout);
        assert!(new_url.contains("newuser/repo"));
    }

    #[test]
    fn test_git_config_update() {
        let repo_dir = setup_test_repo();

        // Set git config
        run_command("git", &["config", "--global", "user.name", "testuser"]);
        run_command("git", &["config", "--global", "user.email", "test@example.com"]);

        // Verify git config
        let name_output = Command::new("git")
            .args(["config", "--global", "user.name"])
            .current_dir(repo_dir.path())
            .output()
            .expect("Failed to get git username");

        let email_output = Command::new("git")
            .args(["config", "--global", "user.email"])
            .current_dir(repo_dir.path())
            .output()
            .expect("Failed to get git email");

        let name = String::from_utf8_lossy(&name_output.stdout).trim().to_string();
        let email = String::from_utf8_lossy(&email_output.stdout).trim().to_string();

        assert_eq!(name, "testuser");
        assert_eq!(email, "test@example.com");
    }
}