use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;
use std::net::TcpStream;

fn check_internet_connectivity() -> bool {
    TcpStream::connect("github.com:443").is_ok()
}

fn main() {
    // Check internet connectivity first
    if !check_internet_connectivity() {
        panic!("No internet connection available");
    }

    // Print current working directory and environment for debugging
    println!("Current directory: {}", env::current_dir().unwrap().display());
    println!("Cargo manifest dir: {}", env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| "Not found".to_string()));

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
    
    // Ensure directories exist
    fs::create_dir_all("src/nets/value").expect("Failed to create value network directory");
    fs::create_dir_all("src/nets/policy").expect("Failed to create policy network directory");
    
    let value_path = PathBuf::from(format!("src/nets/value/{}", value_filename));
    let policy_path = PathBuf::from(format!("src/nets/policy/{}", policy_filename));
    
    // Print full paths for debugging
    println!("Value network path: {}", value_path.display());
    println!("Policy network path: {}", policy_path.display());

    download_network_if_not_exists(&value_network_name, &value_path, ".vn");
    download_network_if_not_exists(&policy_network_name, &policy_path, ".pn");

    println!("cargo:rerun-if-changed=value.txt");
    println!("cargo:rerun-if-changed=policy.txt");
    println!("cargo:rerun-if-changed={}", value_path.display());
    println!("cargo:rerun-if-changed={}", policy_path.display());
}
fn download_network_if_not_exists(network_name: &str, dest_path: &Path, file_extension: &str) {
    if dest_path.exists() {
        println!(
            "Network file {} already exists. Skipping download.",
            dest_path.display()
        );
        return;
    }
    
    let url = format!(
        "https://github.com/Vast342/anura-nets/releases/download/{}/{}{}",
        network_name, network_name, file_extension
    );
    
    // Print detailed URL information
    println!("Attempting to download:");
    println!("Network Name: {}", network_name);
    println!("Full URL: {}", url);
    
    // Try curl first
    let curl_result = Command::new("curl")
        .args(&[
            "-sL",     // Silent mode, follow redirects
            "-f",      // Fail silently on server errors
            "-v",      // Verbose output
            "-o", dest_path.to_str().unwrap(), 
            &url
        ])
        .output();
    
    match curl_result {
        Ok(output) => {
            if output.status.success() {
                println!("Downloaded {} to {} using curl", network_name, dest_path.display());
                return;
            } else {
                eprintln!("Curl download failed");
                eprintln!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
                eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            eprintln!("Failed to execute curl: {}", e);
        }
    }
    
    // Try wget as a fallback
    let wget_result = Command::new("wget")
        .args(&[
            "-O", dest_path.to_str().unwrap(),
            &url
        ])
        .output();
    
    match wget_result {
        Ok(output) => {
            if output.status.success() {
                println!("Downloaded {} to {} using wget", network_name, dest_path.display());
                return;
            } else {
                eprintln!("Wget download failed");
                eprintln!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
                eprintln!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            eprintln!("Failed to execute wget: {}", e);
        }
    }
    
    // If both methods fail, panic with detailed error
    let _ = fs::remove_file(dest_path);
    panic!(
        "Failed to download network file. URL: {}. Check network connectivity and URL.",
        url
    );
}