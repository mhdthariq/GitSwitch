use std::path::Path; // Import the Path type
use std::process::Command;

pub fn run_command(command_str: &str, args: &[&str]) -> bool {
    println!("$ {} {}", command_str, args.join(" ")); // Renamed 'command' to 'command_str'
    let status = Command::new(command_str)
        .args(args)
        .status()
        .unwrap_or_else(|e| {
            eprintln!("❌ Failed to execute command '{}': {}", command_str, e);
            std::process::exit(1); // Consider returning a Result instead of exiting
        });

    if !status.success() {
        eprintln!("❌ Error running {} {:?}", command_str, args);
        return false;
    }
    true
}

/// Checks if a file or directory exists at the given path.
/// Now accepts a `&Path` instead of `&str`.
pub fn file_exists(path: &Path) -> bool {
    path.exists()
}
