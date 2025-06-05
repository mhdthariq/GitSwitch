use crate::config::{Account, delete_account, load_accounts, save_account};
use crate::git::update_git_remote;
use crate::ssh::{
    add_ssh_key, delete_ssh_key_files, display_public_key, generate_ssh_key,
    remove_ssh_config_entry, update_ssh_config,
};
use crate::utils::run_command;
use std::io::{self, Write};

pub fn add_account(name: &str, username: &str, email: &str) {
    // Generate SSH key path based on account name
    let ssh_key_path = format!("~/.ssh/id_rsa_{}", name.replace(' ', "_").to_lowercase());

    // Create parent directory if it doesn't exist
    let expanded_key_path = shellexpand::tilde(&ssh_key_path).to_string();
    if let Some(parent) = std::path::Path::new(&expanded_key_path).parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).expect("Failed to create SSH directory");
        }
    }

    // Generate SSH key automatically
    generate_ssh_key(&ssh_key_path);

    // Create and save account
    let account = Account {
        name: name.to_string(),
        username: username.to_string(),
        email: email.to_string(),
        ssh_key: ssh_key_path.clone(),
    };

    save_account(&account);

    if let Err(e) = update_ssh_config(name, &ssh_key_path) {
        eprintln!("❌ Failed to update SSH config: {}", e);
    }

    // Display the public key for the user to copy
    println!("✅ Account '{}' added successfully!", name);
    println!("\n🔑 Here is your public SSH key to add to GitHub:");
    println!("--------------------------------------------------");
    display_public_key(&ssh_key_path);
    println!("--------------------------------------------------");
    println!(
        "Copy this key and add it to your GitHub account at: https://github.com/settings/keys"
    );
}

pub fn use_account(name_or_username: &str) {
    let accounts = load_accounts();

    // Try to find account by name first, then by username
    let account = accounts
        .iter()
        .find(|acc| acc.name == name_or_username || acc.username == name_or_username)
        .cloned();

    match account {
        Some(acc) => {
            // Set Git global config
            run_command("git", &["config", "--global", "user.name", &acc.username]);
            run_command("git", &["config", "--global", "user.email", &acc.email]);

            // Start ssh-agent if not already running
            // Note: ssh-agent -s might output shell commands to be eval'd.
            // For a robust solution, consider parsing its output or using a library.
            // For now, we assume it sets up the agent if not running.
            println!("🔄 Ensuring SSH agent is running...");
            if !cfg!(windows) {
                // `ssh-agent -s` is typical for Unix-like systems
                let output = std::process::Command::new("ssh-agent").arg("-s").output();
                if let Ok(out) = output {
                    if !out.status.success() {
                        eprintln!(
                            "⚠️ Failed to start ssh-agent. SSH key might not be added automatically."
                        );
                        eprintln!("Error: {}", String::from_utf8_lossy(&out.stderr));
                    } else {
                        // On Unix, `ssh-agent -s` prints shell commands to set env vars.
                        // For this tool to affect the parent shell, the user would typically run:
                        // eval $(git-switch use <account>)
                        // or source the output. Directly running `ssh-agent -s` in a subprocess
                        // doesn't set environment variables for the parent shell of git-switch.
                        // This is a common challenge for tools managing ssh-agent.
                        // For simplicity, we'll proceed, but ssh-add might fail if agent isn't truly ready.
                        println!(
                            "ℹ️ ssh-agent command executed. You might need to run `eval $(ssh-agent -s)` in your shell if keys are not added."
                        );
                    }
                } else {
                    eprintln!(
                        "⚠️ Failed to execute ssh-agent. SSH key might not be added automatically."
                    );
                }
            }

            // Add SSH key to agent
            if add_ssh_key(&acc.ssh_key) {
                println!(
                    "✅ Switched to Git account: {} ({})",
                    acc.name, acc.username
                );

                // Ask if user wants to update current repo's remote URL
                print!("Do you want to update remote URL for the current repository? (y/n): ");
                io::stdout().flush().unwrap();
                let mut response = String::new();
                io::stdin().read_line(&mut response).unwrap();

                if response.trim().to_lowercase() == "y" {
                    print!("Enter repository name (e.g., 'username/repo' or just 'repo'): ");
                    io::stdout().flush().unwrap();
                    let mut repo = String::new();
                    io::stdin().read_line(&mut repo).unwrap();
                    // Clippy fix: needless_borrow
                    update_git_remote(&acc.username, repo.trim());
                }
            } else {
                eprintln!(
                    "❌ Failed to add SSH key to agent. Ensure ssh-agent is running and configured."
                );
            }
        }
        None => {
            println!(
                "❌ Account with name or username '{}' not found.",
                name_or_username
            );

            // List available accounts to help the user
            if !accounts.is_empty() {
                println!("\nAvailable accounts:");
                println!("----------------------------------------");
                println!("Account Name | Git Username | Email");
                println!("----------------------------------------");
                for acc in &accounts {
                    println!("{} | {} | {}", acc.name, acc.username, acc.email);
                }
                println!("----------------------------------------");
            }
        }
    }
}

pub fn remove_account(name: &str) {
    let accounts = load_accounts();
    let account_to_delete = accounts.iter().find(|acc| acc.name == name);

    match account_to_delete {
        Some(account) => {
            // 1. Remove from config.rs
            if let Err(e) = delete_account(name) {
                eprintln!("❌ Failed to remove account from config: {}", e);
                // Optionally, decide if you want to proceed with SSH key deletion if config deletion fails
            }

            // 2. Remove SSH config entry
            if let Err(e) = remove_ssh_config_entry(name) {
                eprintln!("❌ Failed to remove SSH config entry: {}", e);
            }

            // 3. Delete SSH key files
            if let Err(e) = delete_ssh_key_files(&account.ssh_key) {
                eprintln!("❌ Failed to delete SSH key files: {}", e);
            }

            println!(
                "✅ Account '{}' and its associated SSH configurations and keys have been removed.",
                name
            );
        }
        None => {
            println!("❌ Account with name '{}' not found.", name);
        }
    }
}

pub fn list_accounts() {
    crate::config::list_accounts();
}
