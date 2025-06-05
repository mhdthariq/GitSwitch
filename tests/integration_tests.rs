use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Helper function to set up a test environment
fn setup_test_environment() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Create .ssh directory
    let ssh_dir = temp_dir.path().join(".ssh");
    fs::create_dir_all(&ssh_dir).expect("Failed to create .ssh directory");
    
    // Create empty SSH config
    File::create(ssh_dir.join("config")).expect("Failed to create SSH config");
    
    temp_dir
}

/// Helper function to run git-switch command
fn run_git_switch(args: &[&str], temp_dir: &TempDir) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--"])
        .args(args)
        .current_dir(temp_dir.path())
        .output()
        .expect("Failed to execute git-switch command")
}

#[test]
fn test_full_account_lifecycle() {
    let temp_dir = setup_test_environment();

    // Test adding an account
    let add_output = run_git_switch(
        &["add", "work", "workuser", "work@example.com"],
        &temp_dir
    );
    assert!(add_output.status.success());
    let output_str = String::from_utf8_lossy(&add_output.stdout);
    assert!(output_str.contains("Account 'work' added successfully"));

    // Test listing accounts
    let list_output = run_git_switch(&["list"], &temp_dir);
    assert!(list_output.status.success());
    let list_str = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_str.contains("workuser"));
    assert!(list_str.contains("work@example.com"));

    // Test using the account
    let use_output = run_git_switch(&["use", "work"], &temp_dir);
    assert!(use_output.status.success());
    let use_str = String::from_utf8_lossy(&use_output.stdout);
    assert!(use_str.contains("Switched to Git account: work"));

    // Test removing the account
    let remove_output = run_git_switch(&["remove", "work"], &temp_dir);
    assert!(remove_output.status.success());
    let remove_str = String::from_utf8_lossy(&remove_output.stdout);
    assert!(remove_str.contains("Account 'work' and its associated SSH configurations and keys have been removed"));

    // Verify account was removed
    let final_list = run_git_switch(&["list"], &temp_dir);
    let final_list_str = String::from_utf8_lossy(&final_list.stdout);
    assert!(final_list_str.contains("No saved accounts"));
}

#[test]
fn test_invalid_commands() {
    let temp_dir = setup_test_environment();

    // Test invalid account name
    let invalid_use = run_git_switch(&["use", "nonexistent"], &temp_dir);
    let error_str = String::from_utf8_lossy(&invalid_use.stdout);
    assert!(error_str.contains("Account with name or username 'nonexistent' not found"));

    // Test invalid command
    let invalid_cmd = run_git_switch(&["invalid"], &temp_dir);
    let cmd_str = String::from_utf8_lossy(&invalid_cmd.stdout);
    assert!(cmd_str.contains("Use 'git-switch --help'"));
}

#[test]
fn test_multiple_accounts() {
    let temp_dir = setup_test_environment();

    // Add first account
    run_git_switch(
        &["add", "personal", "personaluser", "personal@example.com"],
        &temp_dir
    );

    // Add second account
    run_git_switch(
        &["add", "work", "workuser", "work@example.com"],
        &temp_dir
    );

    // List accounts and verify both exist
    let list_output = run_git_switch(&["list"], &temp_dir);
    let list_str = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_str.contains("personaluser"));
    assert!(list_str.contains("workuser"));

    // Switch between accounts
    let use_personal = run_git_switch(&["use", "personal"], &temp_dir);
    assert!(String::from_utf8_lossy(&use_personal.stdout).contains("Switched to Git account: personal"));

    let use_work = run_git_switch(&["use", "work"], &temp_dir);
    assert!(String::from_utf8_lossy(&use_work.stdout).contains("Switched to Git account: work"));
}