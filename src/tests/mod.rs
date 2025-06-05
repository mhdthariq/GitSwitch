use crate::config::{self, Account}; // Use config:: for public functions
use crate::utils::file_exists; // Keep if used by test_file_exists
use std::fs::{self, File}; // fs needed for reading in debug helper
// use std::io::Write; // For File::create if needed, not directly used in test logic now
use std::path::PathBuf;
use tempfile::TempDir;

/// Test helper to create a temporary directory and a config file path within it.
/// The TempDir must be kept in scope for the duration of its use.
fn setup_temp_config_env() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory for test");
    let config_file_path = temp_dir.path().join(".test-git-switch-accounts");
    (temp_dir, config_file_path)
}

#[cfg(test)]
mod tests {
    use super::*; // To get setup_temp_config_env

    // Optional debug helper to print content of a temp config file
    #[allow(dead_code)] // Allow if not used in all test variations
    fn print_temp_config_content(config_path: &PathBuf, context_msg: &str) {
        println!(
            "\n[TEST DEBUG] {} - Config path: {}",
            context_msg,
            config_path.display()
        );
        if config_path.exists() {
            match fs::read_to_string(config_path) {
                Ok(content) => println!(
                    "[TEST DEBUG] File content:\n--START--\n{}--END--",
                    content.trim_end()
                ),
                Err(e) => println!("[TEST DEBUG] Error reading temp config file: {}", e),
            }
        } else {
            println!("[TEST DEBUG] Temp config file does not exist.");
        }
    }

    #[test]
    fn test_save_and_load_account() {
        let (_temp_dir, temp_config_path) = setup_temp_config_env();
        // println!("test_save_and_load_account using temp config: {}", temp_config_path.display());

        let test_account = Account {
            name: String::from("test_sl_temp"),
            username: String::from("testuser_sl_temp"),
            email: String::from("test_sl_temp@example.com"),
            ssh_key: String::from("~/.ssh/id_rsa_test_sl_temp"),
        };

        // Use the internal worker function with the temp path
        config::save_account_to_path(&test_account, &temp_config_path)
            .expect("Failed to save account to temp path");
        // print_temp_config_content(&temp_config_path, "After save_account_to_path");

        let accounts = config::load_accounts_from_path(&temp_config_path);
        assert!(
            !accounts.is_empty(),
            "No accounts loaded from temp path after save"
        );

        let loaded_account = accounts
            .iter()
            .find(|acc| acc.name == "test_sl_temp")
            .expect("Test account 'test_sl_temp' not found after load from temp path");

        assert_eq!(loaded_account.username, "testuser_sl_temp");
        assert_eq!(loaded_account.email, "test_sl_temp@example.com");
        assert_eq!(loaded_account.ssh_key, "~/.ssh/id_rsa_test_sl_temp");

        // _temp_dir goes out of scope here, and the temp directory (with the file) is cleaned up.
    }

    #[test]
    fn test_delete_account() {
        let (_temp_dir, temp_config_path) = setup_temp_config_env();
        // println!("test_delete_account using temp config: {}", temp_config_path.display());

        let acc1 = Account {
            name: String::from("testdel1_temp"),
            username: String::from("userdel1_temp"),
            email: String::from("testdel1_temp@example.com"),
            ssh_key: String::from("~/.ssh/id_rsa_testdel1_temp"),
        };
        let acc2 = Account {
            name: String::from("testdel2_temp"),
            username: String::from("userdel2_temp"),
            email: String::from("testdel2_temp@example.com"),
            ssh_key: String::from("~/.ssh/id_rsa_testdel2_temp"),
        };

        config::save_account_to_path(&acc1, &temp_config_path).expect("Save acc1 to temp failed");
        config::save_account_to_path(&acc2, &temp_config_path).expect("Save acc2 to temp failed");
        // print_temp_config_content(&temp_config_path, "After saving acc1 and acc2");

        // Delete first account using the worker function with the temp path
        config::delete_account_from_path("testdel1_temp", &temp_config_path)
            .expect("Failed to delete account 'testdel1_temp' from temp path");
        // print_temp_config_content(&temp_config_path, "After deleting testdel1_temp");

        let remaining_accounts = config::load_accounts_from_path(&temp_config_path);

        // The assertion that was failing previously
        assert_eq!(
            remaining_accounts.len(),
            1,
            "Wrong number of accounts after deletion from temp path. Expected 1, got {}. Accounts: {:?}",
            remaining_accounts.len(),
            remaining_accounts
                .iter()
                .map(|a| &a.name)
                .collect::<Vec<_>>()
        );

        // This was the problematic assertion: `left: "test_sl", right: "testdel2"`
        // Now it should correctly be about "testdel2_temp"
        assert_eq!(
            remaining_accounts[0].name, "testdel2_temp",
            "Wrong account remained in temp path. Expected 'testdel2_temp', got '{}'",
            remaining_accounts[0].name
        );

        assert!(
            remaining_accounts.iter().all(|a| a.name != "testdel1_temp"),
            "'testdel1_temp' was not deleted from temp path"
        );
        // _temp_dir goes out of scope here, cleaning up.
    }

    #[test]
    fn test_file_exists() {
        // This test is inherently isolated if it creates its own temp file.
        let temp_dir = TempDir::new().expect("Failed to create temp_dir for test_file_exists");
        let test_file_path = temp_dir.path().join("test_exists.txt");

        assert!(!file_exists(&test_file_path)); // Assuming file_exists takes &Path

        File::create(&test_file_path).expect("Failed to create test file for existence check");

        assert!(file_exists(&test_file_path));
    }
}
