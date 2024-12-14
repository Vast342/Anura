use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let value_network_name = match fs::read_to_string("value.txt") {
        Ok(name) => name.trim().to_string(),
        Err(e) => panic!("Error reading value.txt: {}", e),
    };
    let policy_network_name = match fs::read_to_string("policy.txt") {
        Ok(name) => name.trim().to_string(),
        Err(e) => panic!("Error reading policy.txt: {}", e),
    };
    if value_network_name.is_empty() {
        panic!("value.txt contains no network name");
    }
    if policy_network_name.is_empty() {
        panic!("policy.txt contains no network name");
    }
    let value_filename = format!("{}.vn", value_network_name);
    let policy_filename = format!("{}.pn", policy_network_name);
    let value_path = PathBuf::from(format!("src/nets/value/{}", value_filename));
    let policy_path = PathBuf::from(format!("src/nets/policy/{}", policy_filename));
    download_network_if_not_exists(&value_network_name, &value_path, ".vn");
    download_network_if_not_exists(&policy_network_name, &policy_path, ".pn");
    println!("cargo:rerun-if-changed=value.txt");
    println!("cargo:rerun-if-changed=policy.txt");
    println!("cargo:rerun-if-changed={}", value_path.display());
    println!("cargo:rerun-if-changed={}", policy_path.display());
}

fn download_network_if_not_exists(network_name: &str, dest_path: &Path, file_extension: &str) {
    if dest_path.exists() {
        println!("Network file {} already exists. Skipping download.", dest_path.display());
        return;
    }
    let url = format!(
        "https://github.com/Vast342/anura-nets/releases/download/{}/{}{}", 
        network_name, 
        network_name, 
        file_extension
    );
    let output = Command::new("curl")
        .args(&["-sL", "-f", "-o", dest_path.to_str().unwrap(), &url])
        .output()
        .expect("Failed to execute curl");

    if output.status.success() {
        println!("Downloaded {} to {}", network_name, dest_path.display());
    } else {
        let _ = fs::remove_file(dest_path);
        panic!(
            "Failed to download network file. URL: {}, Error: {}",
            url,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}