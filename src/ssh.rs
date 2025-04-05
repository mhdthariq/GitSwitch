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
    let expanded_path = if identity_file.starts_with("~") {
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
    let public_key_path = format!("{}.pub", shellexpand::tilde(identity_file));

    match File::open(&public_key_path) {
        Ok(mut file) => {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                println!("{}", contents.trim());
            } else {
                println!(
                    "❌ Failed to read public key file. Please check the file at: {}",
                    public_key_path
                );
            }
        }
        Err(_) => {
            println!("❌ Public key file not found at: {}", public_key_path);
        }
    }
}

pub fn update_ssh_config(name: &str, identity_file: &str) -> io::Result<()> {
    let config_entry = format!(
        "\n# {} GitHub Account\nHost github-{}\n    HostName github.com\n    User git\n    IdentityFile {}\n",
        name, name, identity_file
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

pub fn add_ssh_key(key_path: &str) -> bool {
    let home = dirs::home_dir().expect("Could not determine home directory");
    let expanded_path = if key_path.starts_with("~") {
        home.join(&key_path[2..]).to_string_lossy().into_owned()
    } else {
        key_path.to_string()
    };

    if !file_exists(&expanded_path) {
        println!("❌ SSH key not found: {}", key_path);
        return false;
    }

    // Handle Windows differently if needed
    if cfg!(windows) {
        println!("🔑 Adding SSH key to agent (Windows): {}", key_path);
        // Use Windows-specific approach or skip if not needed
        true
    } else {
        println!("🔑 Adding SSH key to agent: {}", key_path);
        run_command("ssh-add", &[&expanded_path])
    }
}
