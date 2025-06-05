use std::fs::{self, File};
use std::io::Write; // For flushing stdout/stderr during debugging
use std::process::Command;
use tempfile::TempDir;

/// Helper function to set up a test environment
fn setup_test_environment() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let ssh_dir = temp_dir.path().join(".ssh");
    fs::create_dir_all(&ssh_dir).expect("Failed to create .ssh directory in temp_dir");

    File::create(ssh_dir.join("config")).expect("Failed to create SSH config in temp_dir");

    temp_dir
}

/// Helper function to run git-switch command
fn run_git_switch(args: &[&str], temp_dir: &TempDir) -> std::process::Output {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--"]) // Added --quiet to suppress Cargo's own output
        .args(args)
        // REMOVED: .current_dir(temp_dir.path()) // This was causing the "could not find Cargo.toml" error
        .env("HOME", temp_dir.path()) // Crucial: Set HOME to the temp directory for git-switch's operations
        .env("RUST_BACKTRACE", "1"); // Get full backtrace if git-switch panics

    let output = cmd.output().expect("Failed to execute git-switch command");

    // For debugging, print output if the command failed or if you always want to see it
    // This can be very helpful if tests still fail.
    // if !output.status.success() || std::env::var("VERBOSE_TEST_OUTPUT").is_ok() {
    //     eprintln!("Running 'git-switch {}' with HOME={}", args.join(" "), temp_dir.path().display());
    //     eprintln!("git-switch status: {}", output.status);
    //     eprintln!("git-switch stdout:\n<<<<\n{}\n>>>>", String::from_utf8_lossy(&output.stdout));
    //     eprintln!("git-switch stderr:\n<<<<\n{}\n>>>>", String::from_utf8_lossy(&output.stderr));
    //     std::io::stderr().flush().unwrap();
    // }
    output
}

#[test]
fn test_full_account_lifecycle() {
    let temp_dir = setup_test_environment();

    // Test adding an account
    let add_output = run_git_switch(&["add", "work", "workuser", "work@example.com"], &temp_dir);
    if !add_output.status.success() {
        eprintln!(
            "ADD COMMAND FAILED in test_full_account_lifecycle:\nStatus: {}\nStdout: {}\nStderr: {}",
            add_output.status,
            String::from_utf8_lossy(&add_output.stdout),
            String::from_utf8_lossy(&add_output.stderr)
        );
        std::io::stderr().flush().unwrap();
    }
    assert!(add_output.status.success(), "git-switch add command failed");
    let output_str = String::from_utf8_lossy(&add_output.stdout);
    assert!(
        output_str.contains("Account 'work' added successfully"),
        "Stdout did not contain success message for add. Actual stdout: {}",
        output_str
    );

    // Test listing accounts
    let list_output = run_git_switch(&["list"], &temp_dir);
    if !list_output.status.success() {
        eprintln!(
            "LIST COMMAND (after add) FAILED in test_full_account_lifecycle:\nStatus: {}\nStdout: {}\nStderr: {}",
            list_output.status,
            String::from_utf8_lossy(&list_output.stdout),
            String::from_utf8_lossy(&list_output.stderr)
        );
        std::io::stderr().flush().unwrap();
    }
    assert!(
        list_output.status.success(),
        "git-switch list command failed after add"
    );
    let list_str = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        list_str.contains("workuser"),
        "List output did not contain 'workuser'. Actual: {}",
        list_str
    );
    assert!(
        list_str.contains("work@example.com"),
        "List output did not contain 'work@example.com'. Actual: {}",
        list_str
    );

    // Test using the account
    let use_output = run_git_switch(&["use", "work"], &temp_dir);
    if !use_output.status.success() {
        eprintln!(
            "USE COMMAND FAILED in test_full_account_lifecycle:\nStatus: {}\nStdout: {}\nStderr: {}",
            use_output.status,
            String::from_utf8_lossy(&use_output.stdout),
            String::from_utf8_lossy(&use_output.stderr)
        );
        std::io::stderr().flush().unwrap();
    }
    assert!(use_output.status.success(), "git-switch use command failed");
    let use_str = String::from_utf8_lossy(&use_output.stdout);
    assert!(
        use_str.contains("Switched to Git account: work"),
        "Stdout did not contain success message for use. Actual: {}",
        use_str
    );

    // Test removing the account
    let remove_output = run_git_switch(&["remove", "work"], &temp_dir);
    if !remove_output.status.success() {
        eprintln!(
            "REMOVE COMMAND FAILED in test_full_account_lifecycle:\nStatus: {}\nStdout: {}\nStderr: {}",
            remove_output.status,
            String::from_utf8_lossy(&remove_output.stdout),
            String::from_utf8_lossy(&remove_output.stderr)
        );
        std::io::stderr().flush().unwrap();
    }
    assert!(
        remove_output.status.success(),
        "git-switch remove command failed"
    );
    let remove_str = String::from_utf8_lossy(&remove_output.stdout);
    assert!(
        remove_str.contains(
            "Account 'work' and its associated SSH configurations and keys have been removed"
        ),
        "Stdout did not contain success message for remove. Actual: {}",
        remove_str
    );

    // Verify account was removed
    let final_list = run_git_switch(&["list"], &temp_dir);
    assert!(
        final_list.status.success(),
        "git-switch list command failed after remove"
    );
    let final_list_str = String::from_utf8_lossy(&final_list.stdout);
    assert!(
        final_list_str.contains("No saved accounts"),
        "Final list output did not indicate no saved accounts. Actual: {}",
        final_list_str
    );
}

#[test]
fn test_invalid_commands() {
    let temp_dir = setup_test_environment();

    // This test expects the `add` command to work. If it fails due to Cargo.toml issues,
    // this test might also fail prematurely or in unexpected ways.
    // Ensure an account exists so 'use non-existent' is a valid test of that specific logic.
    let add_dummy_output = run_git_switch(
        &["add", "dummy", "dummyuser", "dummy@example.com"],
        &temp_dir,
    );
    assert!(
        add_dummy_output.status.success(),
        "Setup for test_invalid_commands: failed to add dummy account"
    );

    let invalid_use_output = run_git_switch(&["use", "nonexistent"], &temp_dir);
    if !invalid_use_output.status.success() {
        eprintln!(
            "USE NONEXISTENT COMMAND non-zero exit in test_invalid_commands:\nStatus: {}\nStdout: {}\nStderr: {}",
            invalid_use_output.status,
            String::from_utf8_lossy(&invalid_use_output.stdout),
            String::from_utf8_lossy(&invalid_use_output.stderr)
        );
        std::io::stderr().flush().unwrap();
    }
    assert!(
        invalid_use_output.status.success(),
        "git-switch use nonexistent exited with non-zero status unexpectedly"
    );

    let error_str_stdout = String::from_utf8_lossy(&invalid_use_output.stdout);
    let expected_msg = "Account with name or username 'nonexistent' not found";
    assert!(
        error_str_stdout.contains(expected_msg),
        "Stdout for 'use nonexistent' did not contain '{}'. Actual stdout: {}",
        expected_msg,
        error_str_stdout
    );

    let invalid_cmd_output = run_git_switch(&["invalidsubcommand"], &temp_dir);
    if invalid_cmd_output.status.success() {
        eprintln!(
            "INVALID SUBCOMMAND unexpectedly succeeded in test_invalid_commands:\nStatus: {}\nStdout: {}\nStderr: {}",
            invalid_cmd_output.status,
            String::from_utf8_lossy(&invalid_cmd_output.stdout),
            String::from_utf8_lossy(&invalid_cmd_output.stderr)
        );
        std::io::stderr().flush().unwrap();
    }
    assert!(
        !invalid_cmd_output.status.success(),
        "git-switch with invalid subcommand should fail (exit non-zero)"
    );

    let cmd_error_str_stderr = String::from_utf8_lossy(&invalid_cmd_output.stderr);
    assert!(
        cmd_error_str_stderr.contains("unrecognized subcommand")
            || cmd_error_str_stderr.contains("For more information, try '--help'"),
        "Stderr for invalid subcommand did not contain expected error. Actual stderr: {}",
        cmd_error_str_stderr
    );
}

#[test]
fn test_multiple_accounts() {
    let temp_dir = setup_test_environment();

    let add_personal_output = run_git_switch(
        &["add", "personal", "personaluser", "personal@example.com"],
        &temp_dir,
    );
    if !add_personal_output.status.success() {
        eprintln!(
            "ADD PERSONAL COMMAND FAILED in test_multiple_accounts:\nStatus: {}\nStdout: {}\nStderr: {}",
            add_personal_output.status,
            String::from_utf8_lossy(&add_personal_output.stdout),
            String::from_utf8_lossy(&add_personal_output.stderr)
        );
        std::io::stderr().flush().unwrap();
    }
    assert!(
        add_personal_output.status.success(),
        "Failed to add 'personal' account"
    );

    let add_work_output =
        run_git_switch(&["add", "work", "workuser", "work@example.com"], &temp_dir);
    if !add_work_output.status.success() {
        eprintln!(
            "ADD WORK COMMAND FAILED in test_multiple_accounts:\nStatus: {}\nStdout: {}\nStderr: {}",
            add_work_output.status,
            String::from_utf8_lossy(&add_work_output.stdout),
            String::from_utf8_lossy(&add_work_output.stderr)
        );
        std::io::stderr().flush().unwrap();
    }
    assert!(
        add_work_output.status.success(),
        "Failed to add 'work' account"
    );

    let list_output = run_git_switch(&["list"], &temp_dir);
    assert!(
        list_output.status.success(),
        "List command failed after adding multiple accounts"
    );
    let list_str = String::from_utf8_lossy(&list_output.stdout);
    assert!(
        list_str.contains("personaluser"),
        "List output did not contain 'personaluser'. Actual: {}",
        list_str
    );
    assert!(
        list_str.contains("workuser"),
        "List output did not contain 'workuser'. Actual: {}",
        list_str
    );

    let use_personal_output = run_git_switch(&["use", "personal"], &temp_dir);
    assert!(
        use_personal_output.status.success(),
        "Failed to switch to 'personal' account"
    );
    assert!(
        String::from_utf8_lossy(&use_personal_output.stdout)
            .contains("Switched to Git account: personal"),
        "Switch to 'personal' did not produce expected stdout message."
    );

    let use_work_output = run_git_switch(&["use", "work"], &temp_dir);
    assert!(
        use_work_output.status.success(),
        "Failed to switch to 'work' account"
    );
    assert!(
        String::from_utf8_lossy(&use_work_output.stdout).contains("Switched to Git account: work"),
        "Switch to 'work' did not produce expected stdout message."
    );
}
