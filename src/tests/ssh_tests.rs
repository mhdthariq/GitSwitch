use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::ssh::{generate_ssh_key, update_ssh_config, remove_ssh_config_entry};

/// Helper function to create a temporary SSH directory structure
fn setup_ssh_test_env() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ssh_dir = temp_dir.path().join(".ssh");
    fs::create_dir_all(&ssh_dir).expect("Failed to create .ssh directory");
    (temp_dir, ssh_dir)
}

/// Helper function to create a mock SSH config file
fn create_mock_ssh_config(ssh_dir: &PathBuf, content: &str) -> PathBuf {
    let config_path = ssh_dir.join("config");
    let mut file = File::create(&config_path).expect("Failed to create SSH config");
    file.write_all(content.as_bytes()).expect("Failed to write SSH config");
    config_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_ssh_config() {
        let (_, ssh_dir) = setup_ssh_test_env();
        let config_path = create_mock_ssh_config(&ssh_dir, "");

        // Test adding new SSH config
        let result = update_ssh_config("test", "~/.ssh/id_rsa_test");
        assert!(result.is_ok(), "Failed to update SSH config");

        // Verify config content
        let config_content = fs::read_to_string(config_path).expect("Failed to read SSH config");
        assert!(config_content.contains("Host github-test"));
        assert!(config_content.contains("IdentityFile ~/.ssh/id_rsa_test"));
    }

    #[test]
    fn test_remove_ssh_config_entry() {
        let (_, ssh_dir) = setup_ssh_test_env();
        
        // Create initial config with two entries
        let initial_config = r#"# Test1 GitHub Account
Host github-test1
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_rsa_test1

# Test2 GitHub Account
Host github-test2
    HostName github.com
    User git
    IdentityFile ~/.ssh/id_rsa_test2
"#;

        let config_path = create_mock_ssh_config(&ssh_dir, initial_config);

        // Remove one entry
        let result = remove_ssh_config_entry("test1");
        assert!(result.is_ok(), "Failed to remove SSH config entry");

        // Verify remaining content
        let config_content = fs::read_to_string(config_path).expect("Failed to read SSH config");
        assert!(!config_content.contains("Host github-test1"));
        assert!(config_content.contains("Host github-test2"));
    }

    #[test]
    fn test_generate_ssh_key() {
        let (_, ssh_dir) = setup_ssh_test_env();
        let key_path = ssh_dir.join("id_rsa_test").to_string_lossy().to_string();

        generate_ssh_key(&key_path);

        // Verify key files were created
        assert!(ssh_dir.join("id_rsa_test").exists(), "Private key not created");
        assert!(ssh_dir.join("id_rsa_test.pub").exists(), "Public key not created");
    }
}