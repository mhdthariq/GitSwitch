use crate::utils::run_command;
use std::process::Command;

pub fn update_git_remote(username: &str, repo_url_input: &str) {
    let repo_name = if repo_url_input.contains('/') {
        // Handle full repo path like "username/repo.git" or "username/repo"
        // Clippy fix: use next_back() for DoubleEndedIterator
        repo_url_input
            .split('/')
            .next_back() // Get the last part after splitting by '/'
            .unwrap_or("") // Handle cases where split might be empty (though unlikely for valid repo URLs)
            .trim_end_matches(".git") // Remove .git suffix if present
            .to_string()
    } else {
        // Handle just repo name like "repo.git" or "repo"
        repo_url_input.trim_end_matches(".git").to_string()
    };

    // Create remote URL using the host alias from SSH config
    // The host alias in SSH config is `github-{account_name_lowercase_underscored}`
    // However, the actual remote URL should be `git@github-{account_name_lowercase_underscored}:{username}/{repo_name}.git`
    // OR, if not using custom host aliases in the remote URL (more common): `git@github.com:{username}/{repo_name}.git`
    // The current SSH config setup implies the latter is intended for git remote.
    // The `github-{name}` host alias is for SSH to pick the right key.
    let remote_url = format!("git@github.com:{}/{}.git", username, repo_name);
    // If you intend to use the SSH host alias in the git remote URL itself, it would be:
    // let remote_url = format!("git@github-{}:{}/{}.git", account_name_for_ssh_host_alias, username, repo_name);
    // This requires passing `account_name_for_ssh_host_alias` to this function.
    // For now, sticking to the standard `git@github.com:...` which relies on SSH config to resolve the key.

    println!("🔄 Updating Git remote URL to: {}", remote_url);

    // Check if origin remote exists
    let output = Command::new("git")
        .args(["remote"])
        .output()
        .expect("Failed to execute git remote command");

    let remotes = String::from_utf8_lossy(&output.stdout);

    if remotes.lines().any(|line| line.trim() == "origin") {
        println!("Removing existing 'origin' remote...");
        run_command("git", &["remote", "remove", "origin"]);
    }

    println!("Adding new 'origin' remote...");
    run_command("git", &["remote", "add", "origin", &remote_url]);

    println!("✅ Git remote URL updated successfully!");
}
