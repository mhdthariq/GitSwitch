// use crate::utils::file_exists; // Assuming file_exists takes &Path
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Returns the default path for the git-switch accounts configuration file.
pub fn get_default_config_path() -> PathBuf {
    let home_dir = dirs::home_dir().expect("Could not determine home directory");
    home_dir.join(".git-switch-accounts")
}

#[derive(Debug, Clone, PartialEq)]
pub struct Account {
    pub name: String,
    pub username: String,
    pub email: String,
    pub ssh_key: String,
}

// --- Worker functions that operate on a specific path ---
// These are now pub(crate) to be accessible by tests within the same crate

/// Loads accounts from a specified configuration file path.
pub(crate) fn load_accounts_from_path(config_file_path: &Path) -> Vec<Account> {
    // println!("[LOAD_ACCOUNTS_FROM_PATH] Attempting to load from: {}", config_file_path.display());
    if !config_file_path.exists() {
        // println!("[LOAD_ACCOUNTS_FROM_PATH] File not found: {}. Returning empty Vec.", config_file_path.display());
        return Vec::new();
    }

    let file_content = match fs::read_to_string(config_file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "[LOAD_ACCOUNTS_FROM_PATH] Error reading file {} for loading: {}. Returning empty.",
                config_file_path.display(),
                e
            );
            return Vec::new();
        }
    };

    file_content
        .lines()
        .filter_map(|line| {
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() {
                return None;
            }
            let parts: Vec<&str> = trimmed_line.split('|').collect();
            if parts.len() == 4 {
                Some(Account {
                    name: parts[0].trim().to_string(),
                    username: parts[1].trim().to_string(),
                    email: parts[2].trim().to_string(),
                    ssh_key: parts[3].trim().to_string(),
                })
            } else {
                eprintln!(
                    "[LOAD_ACCOUNTS_FROM_PATH] Malformed line (parts count {} not 4): '{}'. In file: {}",
                    parts.len(),
                    trimmed_line,
                    config_file_path.display()
                );
                None
            }
        })
        .collect()
}

/// Saves an account to the specified configuration file path.
pub(crate) fn save_account_to_path(account: &Account, config_file_path: &Path) -> io::Result<()> {
    if let Some(parent_dir) = config_file_path.parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)?;
        }
    }

    let entry = format!(
        "{}|{}|{}|{}\n",
        account.name, account.username, account.email, account.ssh_key
    );
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(config_file_path)?;
    file.write_all(entry.as_bytes())?;
    Ok(())
}

/// Deletes an account from the specified configuration file path.
pub(crate) fn delete_account_from_path(
    name_to_delete: &str,
    config_file_path: &Path,
) -> io::Result<()> {
    let accounts_before_delete = load_accounts_from_path(config_file_path);

    let updated_accounts: Vec<Account> = accounts_before_delete
        .into_iter()
        .filter(|acc| acc.name != name_to_delete)
        .collect();

    if let Some(parent_dir) = config_file_path.parent() {
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)?;
        }
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(config_file_path)?;

    for account_to_write in &updated_accounts {
        let entry = format!(
            "{}|{}|{}|{}\n",
            account_to_write.name,
            account_to_write.username,
            account_to_write.email,
            account_to_write.ssh_key
        );
        file.write_all(entry.as_bytes())?;
    }
    file.flush()?;
    drop(file);
    Ok(())
}

// --- Public wrapper functions using the default path ---

/// Loads all saved Git accounts from the default configuration file.
pub fn load_accounts() -> Vec<Account> {
    let default_path = get_default_config_path();
    load_accounts_from_path(&default_path)
}

/// Saves a new Git account to the default configuration file.
pub fn save_account(account: &Account) {
    let default_path = get_default_config_path();
    match save_account_to_path(account, &default_path) {
        Ok(_) => println!("✅ Account '{}' saved.", account.name),
        Err(e) => eprintln!("❌ Failed to save account '{}': {}", account.name, e),
    }
}

/// Removes a saved Git account from the default configuration file.
pub fn delete_account(name_to_delete: &str) -> io::Result<()> {
    let default_path = get_default_config_path();
    match delete_account_from_path(name_to_delete, &default_path) {
        Ok(_) => {
            println!("🗑️ Account '{}' removed from config.", name_to_delete);
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "❌ Failed to remove account '{}' from config: {}",
                name_to_delete, e
            );
            Err(e)
        }
    }
}

/// Lists all saved Git accounts from the default configuration file.
pub fn list_accounts() {
    let accounts = load_accounts();
    if accounts.is_empty() {
        println!("No saved accounts.");
        return;
    }

    println!("🔹 Saved Git Accounts:");
    println!("------------------------------------------------------------");
    println!(
        "{:<20} | {:<25} | {:<30}",
        "Account Name", "Git Username", "Email"
    );
    println!("------------------------------------------------------------");
    for acc in &accounts {
        println!(
            "{:<20} | {:<25} | {:<30}",
            acc.name, acc.username, acc.email
        );
    }
    println!("------------------------------------------------------------");
}
