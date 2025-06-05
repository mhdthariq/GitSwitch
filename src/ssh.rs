use crate::utils::{file_exists, run_command};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

pub fn get_ssh_config_path() -> String {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".ssh")
        .join("config")
        .to_string_lossy()
        .into_owned()
}

pub fn generate_ssh_key(identity_file: &str) {
    let expanded_path = if identity_file.starts_with('~') {
        let home = dirs::home_dir().expect("Could not determine home directory");
        home.join(&identity_file[2..])
            .to_string_lossy()
            .into_owned()
    } else {
        identity_file.to_string()
    };

    if file_exists(&expanded_path) {
        println!("✅ SSH key already exists: {}", identity_file);
        return;
    }

    // Ensure the directory exists
    if let Some(parent) = Path::new(&expanded_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Failed to create SSH directory");
        }
    }

    println!("🔑 Generating SSH key: {}", identity_file);
    run_command(
        "ssh-keygen",
        &["-t", "rsa", "-b", "4096", "-f", &expanded_path, "-N", ""],
    );
}

pub fn display_public_key(identity_file: &str) {
    let public_key_path_str = format!("{}.pub", shellexpand::tilde(identity_file));
    let public_key_path = Path::new(&public_key_path_str);

    match File::open(public_key_path) {
        Ok(mut file) => {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                println!("{}", contents.trim());
            } else {
                println!(
                    "❌ Failed to read public key file. Please check the file at: {}",
                    public_key_path.display()
                );
            }
        }
        Err(_) => {
            println!(
                "❌ Public key file not found at: {}",
                public_key_path.display()
            );
        }
    }
}

pub fn update_ssh_config(name: &str, identity_file: &str) -> io::Result<()> {
    let config_entry = format!(
        "\n# {} GitHub Account\nHost github-{}\n    HostName github.com\n    User git\n    IdentityFile {}\n",
        name,
        name.replace(' ', "_").to_lowercase(), // Ensure name is valid for Host alias
        identity_file
    );

    let expanded_path = get_ssh_config_path();
    let path = Path::new(&expanded_path);

    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    file.write_all(config_entry.as_bytes())?;
    println!("✅ Updated SSH config for account: {}", name);
    Ok(())
}

pub fn remove_ssh_config_entry(name: &str) -> io::Result<()> {
    let config_path_str = get_ssh_config_path();
    let path = Path::new(&config_path_str);

    if !path.exists() {
        println!(
            "ℹ️ SSH config file not found, nothing to remove for account '{}'.",
            name
        );
        return Ok(());
    }

    let file_content = fs::read_to_string(path)?;
    let mut new_content = String::new();
    let mut lines = file_content.lines().peekable();
    let entry_header_check = format!("# {} GitHub Account", name);
    let host_check = format!("Host github-{}", name.replace(' ', "_").to_lowercase());

    let mut skip_block = false;

    while let Some(line) = lines.next() {
        if line.trim() == entry_header_check {
            // Check if the next line is the Host entry for this account
            if lines
                .peek()
                .map_or(false, |next_line| next_line.trim().starts_with(&host_check))
            {
                skip_block = true;
                // Skip the header line and the next 3 lines of the config block
                for _ in 0..3 {
                    lines.next();
                }
                continue;
            }
        }

        if !skip_block {
            new_content.push_str(line);
            new_content.push('\n');
        } else {
            // If we were skipping, and the current line is not empty, it means the block ended.
            // However, our logic above already consumes the block.
            // If the line is empty or a new block starts, reset skip_block.
            if line.trim().is_empty() || line.trim().starts_with('#') {
                skip_block = false;
                // If it's a new block's header or an empty line that we didn't mean to skip
                if !(line.trim() == entry_header_check
                    && lines
                        .peek()
                        .map_or(false, |next_line| next_line.trim().starts_with(&host_check)))
                {
                    new_content.push_str(line);
                    new_content.push('\n');
                }
            }
        }
    }

    // Clean up excessive newlines at the end
    while new_content.ends_with("\n\n") {
        new_content.pop();
    }
    // If the entire content was just one block and it was removed, new_content might be empty or just "\n"
    if new_content == "\n" {
        new_content.clear();
    }

    let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
    file.write_all(new_content.as_bytes())?;
    println!("🗑️ SSH config entry for '{}' removed.", name);
    Ok(())
}

pub fn delete_ssh_key_files(identity_file_base: &str) -> io::Result<()> {
    let base_path = shellexpand::tilde(identity_file_base).to_string();
    let private_key_path = Path::new(&base_path);

    // Bind the formatted string to a variable with a longer lifetime
    let public_key_path_str = format!("{}.pub", base_path);
    let public_key_path = Path::new(&public_key_path_str);

    if private_key_path.exists() {
        fs::remove_file(private_key_path)?;
        println!("🗑️ Deleted private SSH key: {}", private_key_path.display());
    }

    if public_key_path.exists() {
        fs::remove_file(public_key_path)?;
        println!("🗑️ Deleted public SSH key: {}", public_key_path.display());
    }

    Ok(())
}

pub fn add_ssh_key(key_path: &str) -> bool {
    let home = dirs::home_dir().expect("Could not determine home directory");
    let expanded_path_str = if key_path.starts_with('~') {
        home.join(&key_path[2..]).to_string_lossy().into_owned()
    } else {
        key_path.to_string()
    };
    let expanded_path = Path::new(&expanded_path_str);

    if !expanded_path.exists() {
        println!("❌ SSH key not found: {}", expanded_path.display());
        return false;
    }

    // Handle Windows differently if needed
    if cfg!(windows) {
        println!(
            "🔑 Adding SSH key to agent (Windows): {}",
            expanded_path.display()
        );
        // For Windows, ssh-agent might be handled by Pageant or Windows OpenSSH Agent.
        // A simple `ssh-add` might work if OpenSSH agent is running and configured.
        // This example assumes `ssh-add` is available and works.
        // More robust Windows support might require checking agent status or using specific APIs.
        run_command("ssh-add", &[expanded_path.to_str().unwrap()])
    } else {
        println!("🔑 Adding SSH key to agent: {}", expanded_path.display());
        run_command("ssh-add", &[expanded_path.to_str().unwrap()])
    }
}
