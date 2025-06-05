use crate::config::Account;
use crate::utils::file_exists;
use std::fs::File;
// Removed Write as it's not directly used at this top level of the module
// use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test helper to create a temporary directory for test files
fn setup_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

// The function `create_mock_config_in_dir` was removed as it was unused.
// The function `setup_test_env_for_config` was also removed as it was related to the unused function.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config; // To allow direct calls to config functions

    #[test]
    fn test_save_and_load_account() {
        // This test, as originally structured, will use the actual default config path.
        // This can be problematic for parallel tests or if the config file has existing data.
        // A better approach would be to make `save_account` and `load_accounts` take a `Path` argument.
        // For now, we proceed with the original logic but acknowledge this limitation.

        let original_config_path_str = config::get_config_path();
        let original_config_path = PathBuf::from(&original_config_path_str);
        let backup_path = PathBuf::from(original_config_path_str.clone() + ".backup_sl");

        // Backup existing config if it exists
        if original_config_path.exists() {
            std::fs::copy(&original_config_path, &backup_path)
                .expect("Failed to backup original config for test_save_and_load_account");
        }

        // Ensure config is empty before test by removing it (it will be recreated by save_account)
        if original_config_path.exists() {
            std::fs::remove_file(&original_config_path).unwrap();
        }

        let test_account = Account {
            name: String::from("test_sl"), // Unique name for this test
            username: String::from("testuser_sl"),
            email: String::from("test_sl@example.com"),
            ssh_key: String::from("~/.ssh/id_rsa_test_sl"),
        };

        // Save account (uses default config path)
        config::save_account(&test_account);

        // Load accounts (uses default config path)
        let accounts = config::load_accounts();
        assert!(!accounts.is_empty(), "No accounts loaded after save");

        let loaded_account = accounts
            .iter()
            .find(|acc| acc.name == "test_sl")
            .expect("Test account 'test_sl' not found after load");

        assert_eq!(loaded_account.username, "testuser_sl");
        assert_eq!(loaded_account.email, "test_sl@example.com");
        assert_eq!(loaded_account.ssh_key, "~/.ssh/id_rsa_test_sl");

        // Cleanup:
        // Remove the test account from the actual config file
        config::delete_account("test_sl").expect("Cleanup: Failed to delete test_sl account");

        // Restore backup if it existed
        if backup_path.exists() {
            std::fs::rename(&backup_path, &original_config_path)
                .expect("Cleanup: Failed to restore backup for test_save_and_load_account");
        } else {
            // If no backup existed, but the test created a new config file,
            // and delete_account didn't remove it (e.g., if it was the last account),
            // ensure it's removed if it's now empty.
            if original_config_path.exists() && config::load_accounts().is_empty() {
                std::fs::remove_file(&original_config_path)
                    .expect("Cleanup: Failed to remove test-created empty config");
            }
        }
    }

    #[test]
    fn test_delete_account() {
        let original_config_path_str = config::get_config_path();
        let original_config_path = PathBuf::from(&original_config_path_str);
        let backup_path = PathBuf::from(original_config_path_str.clone() + ".backup_del");

        if original_config_path.exists() {
            std::fs::copy(&original_config_path, &backup_path)
                .expect("Failed to backup original config for test_delete_account");
        }

        if original_config_path.exists() {
            std::fs::remove_file(&original_config_path).unwrap();
        }

        let acc1 = Account {
            name: String::from("testdel1"),
            username: String::from("userdel1"),
            email: String::from("testdel1@example.com"),
            ssh_key: String::from("~/.ssh/id_rsa_testdel1"),
        };
        let acc2 = Account {
            name: String::from("testdel2"),
            username: String::from("userdel2"),
            email: String::from("testdel2@example.com"),
            ssh_key: String::from("~/.ssh/id_rsa_testdel2"),
        };
        config::save_account(&acc1);
        config::save_account(&acc2);

        // Delete first account
        config::delete_account("testdel1").expect("Failed to delete account 'testdel1'");

        // Verify account was deleted
        let remaining_accounts = config::load_accounts();
        assert_eq!(
            remaining_accounts.len(),
            1,
            "Wrong number of accounts after deletion"
        );
        assert_eq!(
            remaining_accounts[0].name, "testdel2",
            "Wrong account remained"
        );
        assert!(
            remaining_accounts.iter().all(|a| a.name != "testdel1"),
            "testdel1 was not deleted"
        );

        // Cleanup
        config::delete_account("testdel2")
            .unwrap_or_else(|e| eprintln!("Cleanup failed for testdel2: {}", e));
        if backup_path.exists() {
            std::fs::rename(&backup_path, &original_config_path)
                .expect("Cleanup: Failed to restore backup for test_delete_account");
        } else {
            if original_config_path.exists() && config::load_accounts().is_empty() {
                std::fs::remove_file(&original_config_path)
                    .expect("Cleanup: Failed to remove test-created empty config for delete test");
            }
        }
    }

    #[test]
    fn test_file_exists() {
        let temp_dir = setup_test_dir(); // Uses a fresh temp dir, good for this test
        let test_file_path = temp_dir.path().join("test.txt");

        // File should not exist initially
        assert!(!file_exists(&test_file_path.to_string_lossy()));

        // Create file
        File::create(&test_file_path).expect("Failed to create test file");

        // File should exist now
        assert!(file_exists(&test_file_path.to_string_lossy()));
    }
}
