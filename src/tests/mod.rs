use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::config::{Account, get_config_path, load_accounts, save_account, delete_account};
use crate::utils::file_exists;

/// Test helper to create a temporary directory for test files
fn setup_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Test helper to create a mock config file with test accounts
fn create_mock_config(accounts: &[Account]) -> PathBuf {
    let temp_dir = setup_test_dir();
    let config_path = temp_dir.path().join(".git-switch-accounts");
    
    let mut file = File::create(&config_path).expect("Failed to create mock config file");
    for account in accounts {
        let entry = format!(
            "{}|{}|{}|{}\n",
            account.name, account.username, account.email, account.ssh_key
        );
        file.write_all(entry.as_bytes()).expect("Failed to write mock account");
    }
    
    config_path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load_account() {
        let temp_dir = setup_test_dir();
        let test_account = Account {
            name: String::from("test"),
            username: String::from("testuser"),
            email: String::from("test@example.com"),
            ssh_key: String::from("~/.ssh/id_rsa_test"),
        };

        // Save account
        save_account(&test_account);

        // Load accounts and verify
        let accounts = load_accounts();
        assert!(!accounts.is_empty(), "No accounts loaded");
        
        let loaded_account = accounts.iter()
            .find(|acc| acc.name == "test")
            .expect("Test account not found");

        assert_eq!(loaded_account.username, "testuser");
        assert_eq!(loaded_account.email, "test@example.com");
        assert_eq!(loaded_account.ssh_key, "~/.ssh/id_rsa_test");
    }

    #[test]
    fn test_delete_account() {
        let test_accounts = vec![
            Account {
                name: String::from("test1"),
                username: String::from("user1"),
                email: String::from("test1@example.com"),
                ssh_key: String::from("~/.ssh/id_rsa_test1"),
            },
            Account {
                name: String::from("test2"),
                username: String::from("user2"),
                email: String::from("test2@example.com"),
                ssh_key: String::from("~/.ssh/id_rsa_test2"),
            },
        ];

        let config_path = create_mock_config(&test_accounts);

        // Delete first account
        delete_account("test1").expect("Failed to delete account");

        // Verify account was deleted
        let remaining_accounts = load_accounts();
        assert_eq!(remaining_accounts.len(), 1, "Wrong number of accounts after deletion");
        assert_eq!(remaining_accounts[0].name, "test2", "Wrong account remained");
    }

    #[test]
    fn test_file_exists() {
        let temp_dir = setup_test_dir();
        let test_file = temp_dir.path().join("test.txt");

        // File should not exist initially
        assert!(!file_exists(&test_file.to_string_lossy()));

        // Create file
        File::create(&test_file).expect("Failed to create test file");

        // File should exist now
        assert!(file_exists(&test_file.to_string_lossy()));
    }
}